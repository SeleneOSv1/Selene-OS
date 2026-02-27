#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ph1d::{
    decode_normalized_output_json, reason_codes as d_reason_codes, Ph1dProviderAdapter,
    Ph1dProviderAdapterError,
};
use crate::ph1n::{Ph1nConfig, Ph1nRuntime};
use crate::ph1srl::{Ph1SrlConfig, Ph1SrlRuntime};
use selene_kernel_contracts::ph1c::{
    ConfidenceBucket, LanguageHintConfidence, LanguageTag, PartialTranscript,
    PartialTranscriptBatch, Ph1cAuditMeta, Ph1cRequest, Ph1cResponse, Ph1cSttStrategy,
    QualityBucket, RetryAdvice, RouteClassUsed, RoutingModeUsed, SelectedSlot, SessionStateRef,
    TranscriptOk, TranscriptReject,
};
use selene_kernel_contracts::ph1d::{
    Ph1dProviderCallRequest, Ph1dProviderCallResponse, Ph1dProviderInputPayloadKind,
    Ph1dProviderRouteClass, Ph1dProviderStatus, Ph1dProviderTask, Ph1dProviderValidationStatus,
    RequestId, SafetyTier, SchemaHash, PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1k::{
    CaptureQualityClass, Confidence, DeviceHealth, InterruptCandidateConfidenceBand,
    NetworkStabilityClass, RecoverabilityClass, VadDecisionConfidenceBand,
};
use selene_kernel_contracts::ph1n::{OverallConfidence, Ph1nRequest, Ph1nResponse};
use selene_kernel_contracts::ph1srl::{
    Ph1SrlRequest, Ph1SrlResponse, SrlArgumentNormalizeRequest, SrlFrameBuildRequest, SrlFrameSpan,
    SrlRequestEnvelope, SrlValidationStatus,
};
use selene_kernel_contracts::ph1w::SessionState;
use selene_kernel_contracts::{ReasonCodeId, SchemaVersion, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.C reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const STT_FAIL_EMPTY: ReasonCodeId = ReasonCodeId(0x4300_0001);
    pub const STT_FAIL_LOW_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x4300_0002);
    pub const STT_FAIL_LOW_COVERAGE: ReasonCodeId = ReasonCodeId(0x4300_0003);
    pub const STT_FAIL_GARBLED: ReasonCodeId = ReasonCodeId(0x4300_0004);
    pub const STT_FAIL_LANGUAGE_MISMATCH: ReasonCodeId = ReasonCodeId(0x4300_0005);
    pub const STT_FAIL_AUDIO_DEGRADED: ReasonCodeId = ReasonCodeId(0x4300_0006);
    pub const STT_FAIL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4300_0007);
    pub const STT_FAIL_POLICY_RESTRICTED: ReasonCodeId = ReasonCodeId(0x4300_0008);
    pub const STT_FAIL_PROVIDER_TIMEOUT: ReasonCodeId = ReasonCodeId(0x4300_0009);
    pub const STT_FAIL_NETWORK_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4300_000A);
    pub const STT_FAIL_BACKGROUND_SPEECH: ReasonCodeId = ReasonCodeId(0x4300_000B);
    pub const STT_FAIL_ECHO_SUSPECTED: ReasonCodeId = ReasonCodeId(0x4300_000C);
    pub const STT_FAIL_QUOTA_THROTTLED: ReasonCodeId = ReasonCodeId(0x4300_000D);
    pub const STT_FAIL_PARTIAL_INVALID: ReasonCodeId = ReasonCodeId(0x4300_000E);
    pub const STT_FAIL_PARTIAL_ORDER: ReasonCodeId = ReasonCodeId(0x4300_000F);
    pub const STT_FAIL_PROVIDER_CIRCUIT_OPEN: ReasonCodeId = ReasonCodeId(0x4300_0010);
    pub const STT_FAIL_SHADOW_INPUT_INVALID: ReasonCodeId = ReasonCodeId(0x4300_0011);
    pub const STT_FAIL_SHADOW_PROVIDER_TRUTH_INVALID: ReasonCodeId = ReasonCodeId(0x4300_0012);
    pub const STT_FAIL_SHADOW_PROMOTION_BLOCKED: ReasonCodeId = ReasonCodeId(0x4300_0013);
    pub const STT_FAIL_PROVIDER_DISAGREEMENT: ReasonCodeId = ReasonCodeId(0x4300_0014);
    pub const STT_FAIL_SPEAKER_OVERLAP_AMBIGUOUS: ReasonCodeId = ReasonCodeId(0x4300_0015);
    pub const STT_FAIL_LOW_SEMANTIC_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x4300_0016);
}

const INTENT_REPAIR_MAX_TOKENS: usize = 48;
const INTENT_REPAIR_MAX_HINTS: usize = 12;
const INTENT_REPAIR_MIN_OVERLAP_RATIO: f32 = 0.45;
const INTENT_REPAIR_CONFIDENCE_BOOST: f32 = 0.06;
const INTENT_REPAIR_LOW_RATIO_SCALE: f32 = 0.72;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderSlot {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone)]
pub struct SttAttempt {
    pub provider: ProviderSlot,
    pub latency_ms: u32,
    pub transcript_text: String,
    pub language_tag: LanguageTag,
    pub avg_word_confidence: f32,
    pub low_confidence_ratio: f32,
    pub stable: bool,
}

#[derive(Debug, Clone)]
pub struct SttPartialAttempt {
    pub text_chunk: String,
    pub confidence: f32,
    pub stable: bool,
    pub revision_id: u32,
}

#[derive(Debug, Clone)]
pub struct Ph1cStreamCommit {
    pub partial_batch: Option<PartialTranscriptBatch>,
    pub response: Ph1cResponse,
    pub low_latency_commit: bool,
    pub finalized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cShadowSliceKey {
    pub locale: String,
    pub device_route: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1cShadowRouteDecision {
    HoldShadow,
    EligibleForPromotion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cShadowRouteOutcome {
    pub slice_key: Ph1cShadowSliceKey,
    pub transcript_overlap_bp: u16,
    pub confidence_delta_bp: i16,
    pub latency_delta_ms: i32,
    pub governed_gate_passed: bool,
    pub decision: Ph1cShadowRouteDecision,
    pub block_reason_code: Option<ReasonCodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1cLexiconSourceScope {
    Global,
    Tenant,
    Domain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cLexiconTermV2 {
    pub term: String,
    pub weight_bp: u16,
    pub expires_at_unix_s: u64,
    pub source_scope: Ph1cLexiconSourceScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cLiveProviderContext {
    pub correlation_id: u64,
    pub turn_id: u64,
    pub tenant_id: String,
    pub request_id: RequestId,
    pub idempotency_key: String,
    pub primary_provider_id: String,
    pub primary_model_id: String,
    pub secondary_provider_id: String,
    pub secondary_model_id: String,
    pub timeout_ms: u32,
    pub retry_budget: u8,
    pub tool_catalog_hash: SchemaHash,
    pub policy_context_hash: SchemaHash,
    pub safety_tier: SafetyTier,
    pub privacy_mode: bool,
    pub do_not_disturb: bool,
    pub tenant_vocabulary_pack_id: Option<String>,
    pub user_vocabulary_pack_id: Option<String>,
    pub tenant_lexicon_terms: Vec<String>,
    pub domain_lexicon_terms: Vec<String>,
    pub global_lexicon_terms_v2: Vec<Ph1cLexiconTermV2>,
    pub provider_disagreement_divergence_bp_threshold: u16,
    pub enforce_provider_disagreement_clarify: bool,
    pub enable_cost_quality_routing: bool,
    pub primary_cost_microunits: u32,
    pub secondary_cost_microunits: u32,
}

impl Ph1cLiveProviderContext {
    pub fn mvp_openai_google_v1(correlation_id: u64, turn_id: u64, tenant_id: String) -> Self {
        let request_id = RequestId(nonzero_u64(fnv1a64(
            format!("ph1c_live_stt:{correlation_id}:{turn_id}:{tenant_id}").as_bytes(),
        )));
        let idempotency_key = format!("ph1c_live_stt_{correlation_id}_{turn_id}");
        Self {
            correlation_id,
            turn_id,
            tenant_id,
            request_id,
            idempotency_key,
            primary_provider_id: "openai".to_string(),
            primary_model_id: "gpt-4o-mini-transcribe".to_string(),
            secondary_provider_id: "google".to_string(),
            secondary_model_id: "chirp_2".to_string(),
            timeout_ms: 4_000,
            retry_budget: 1,
            tool_catalog_hash: SchemaHash(8101),
            policy_context_hash: SchemaHash(8102),
            safety_tier: SafetyTier::Standard,
            privacy_mode: false,
            do_not_disturb: false,
            tenant_vocabulary_pack_id: None,
            user_vocabulary_pack_id: None,
            tenant_lexicon_terms: Vec::new(),
            domain_lexicon_terms: Vec::new(),
            global_lexicon_terms_v2: Vec::new(),
            provider_disagreement_divergence_bp_threshold: 3_800,
            enforce_provider_disagreement_clarify: true,
            enable_cost_quality_routing: true,
            primary_cost_microunits: 22_00,
            secondary_cost_microunits: 28_00,
        }
    }

    pub(crate) fn validate(&self) -> Result<(), ContractViolationLocal> {
        if self.correlation_id == 0 || self.turn_id == 0 {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.correlation_or_turn_id",
            ));
        }
        if self.tenant_id.trim().is_empty() || !is_provider_token(&self.tenant_id, 128) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.tenant_id",
            ));
        }
        if self.idempotency_key.trim().is_empty() || !is_provider_token(&self.idempotency_key, 128)
        {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.idempotency_key",
            ));
        }
        if !is_provider_token(&self.primary_provider_id, 64)
            || !is_provider_token(&self.secondary_provider_id, 64)
            || !is_provider_token(&self.primary_model_id, 128)
            || !is_provider_token(&self.secondary_model_id, 128)
        {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.provider_or_model",
            ));
        }
        if !is_openai_provider_id(&self.primary_provider_id) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.primary_provider_id",
            ));
        }
        if !is_google_provider_id(&self.secondary_provider_id) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.secondary_provider_id",
            ));
        }
        if !(100..=120_000).contains(&self.timeout_ms) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.timeout_ms",
            ));
        }
        if self.retry_budget > 10 {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.retry_budget",
            ));
        }
        if self.request_id.0 == 0
            || self.tool_catalog_hash.0 == 0
            || self.policy_context_hash.0 == 0
        {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.hash_or_request_id",
            ));
        }
        if !valid_opt_pack_id(&self.tenant_vocabulary_pack_id)
            || !valid_opt_pack_id(&self.user_vocabulary_pack_id)
        {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.vocabulary_pack_id",
            ));
        }
        if !valid_lexicon_terms(&self.tenant_lexicon_terms)
            || !valid_lexicon_terms(&self.domain_lexicon_terms)
            || !valid_lexicon_terms_v2(&self.global_lexicon_terms_v2)
        {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.lexicon_terms",
            ));
        }
        if !(500..=10_000).contains(&self.provider_disagreement_divergence_bp_threshold) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.provider_disagreement_divergence_bp_threshold",
            ));
        }
        if self.primary_cost_microunits == 0 || self.secondary_cost_microunits == 0 {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_live_provider_context.cost_microunits",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContractViolationLocal {
    InvalidValue(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1cCircuitBreakerConfig {
    pub failure_threshold: u8,
    pub cooldown_ms: u32,
}

impl Ph1cCircuitBreakerConfig {
    pub fn mvp_v1() -> Self {
        Self {
            failure_threshold: 3,
            cooldown_ms: 30_000,
        }
    }

    fn validate(self) -> Result<(), ContractViolationLocal> {
        if self.failure_threshold == 0 {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_circuit_breaker_config.failure_threshold",
            ));
        }
        if !(100..=120_000).contains(&self.cooldown_ms) {
            return Err(ContractViolationLocal::InvalidValue(
                "ph1c_circuit_breaker_config.cooldown_ms",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CircuitBreakerKey {
    tenant_id: String,
    provider_id: String,
    model_id: String,
}

impl CircuitBreakerKey {
    fn new(tenant_id: &str, provider_id: &str, model_id: &str) -> Self {
        Self {
            tenant_id: tenant_id.trim().to_ascii_lowercase(),
            provider_id: provider_id.trim().to_ascii_lowercase(),
            model_id: model_id.trim().to_ascii_lowercase(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CircuitBreakerEntry {
    consecutive_failures: u8,
    open_until_ms: u64,
}

#[derive(Debug, Default, Clone)]
struct CircuitBreakerBook {
    entries: BTreeMap<CircuitBreakerKey, CircuitBreakerEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ph1cConfig {
    pub max_attempts_per_turn: u8,
    pub max_total_latency_budget_ms: u32,
    pub max_retries_per_provider: u8,

    pub min_avg_word_confidence: f32,
    pub max_low_confidence_ratio: f32,
    pub require_stable: bool,

    pub min_confidence_bucket_to_pass: ConfidenceBucket,

    pub min_chars_per_second: f32,
    pub min_chars_absolute: usize,
    pub routing_mode_used: RoutingModeUsed,
    pub stream_max_revisions: u32,
    pub stream_low_latency_confidence_min: f32,
    pub stream_low_latency_min_chars: usize,
    pub semantic_gate_enabled: bool,
    pub min_semantic_quality: u8,
    pub require_overlap_disambiguation: bool,
    pub cost_quality_confidence_tolerance_bp: u16,
}

impl Ph1cConfig {
    pub fn mvp_desktop_v1() -> Self {
        Self {
            max_attempts_per_turn: 3,
            max_total_latency_budget_ms: 2_000,
            max_retries_per_provider: 1,
            min_avg_word_confidence: 0.85,
            max_low_confidence_ratio: 0.15,
            require_stable: true,
            // Spec: "MED must not pass in MVP".
            min_confidence_bucket_to_pass: ConfidenceBucket::High,
            // Coverage heuristics are deliberately conservative in the skeleton.
            min_chars_per_second: 1.5,
            min_chars_absolute: 2,
            routing_mode_used: RoutingModeUsed::Lead,
            stream_max_revisions: 8,
            stream_low_latency_confidence_min: 0.93,
            stream_low_latency_min_chars: 12,
            semantic_gate_enabled: true,
            min_semantic_quality: 1,
            require_overlap_disambiguation: true,
            cost_quality_confidence_tolerance_bp: 300,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1cRuntime {
    config: Ph1cConfig,
    circuit_breaker_config: Ph1cCircuitBreakerConfig,
    circuit_breakers: Arc<Mutex<CircuitBreakerBook>>,
}

impl Ph1cRuntime {
    pub fn new(config: Ph1cConfig) -> Self {
        let preferred = Ph1cCircuitBreakerConfig::mvp_v1();
        match Self::with_circuit_breaker_config(config.clone(), preferred) {
            Ok(runtime) => runtime,
            Err(_) => {
                let fallback = Ph1cCircuitBreakerConfig {
                    failure_threshold: 1,
                    cooldown_ms: 100,
                };
                // Fallback is hardcoded valid, but still guarded fail-closed.
                Self::with_circuit_breaker_config(config, fallback).unwrap_or_else(|_| Self {
                    config: Ph1cConfig::mvp_desktop_v1(),
                    circuit_breaker_config: fallback,
                    circuit_breakers: Arc::new(Mutex::new(CircuitBreakerBook::default())),
                })
            }
        }
    }

    pub(crate) fn with_circuit_breaker_config(
        config: Ph1cConfig,
        circuit_breaker_config: Ph1cCircuitBreakerConfig,
    ) -> Result<Self, ContractViolationLocal> {
        circuit_breaker_config.validate()?;
        Ok(Self {
            config,
            circuit_breaker_config,
            circuit_breakers: Arc::new(Mutex::new(CircuitBreakerBook::default())),
        })
    }

    /// Deterministic evaluation over already-produced attempt outputs.
    ///
    /// In production, attempts would be produced by calling STT providers; this skeleton focuses on:
    /// budgets, quality gating, and non-leaky output contracts.
    pub fn run(&self, req: &Ph1cRequest, attempts: &[SttAttempt]) -> Ph1cResponse {
        if req.device_state_ref.health != DeviceHealth::Healthy {
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason_codes::STT_FAIL_AUDIO_DEGRADED,
                RetryAdvice::MoveCloser,
            ));
        }

        let strategy = select_stt_strategy(req);
        if matches!(strategy, Ph1cSttStrategy::ClarifyOnly) {
            let reject_meta = build_audit_meta(
                &self.config,
                0,
                0,
                SelectedSlot::None,
                0,
                QualityBucket::Low,
                QualityBucket::Low,
                QualityBucket::Low,
                None,
                None,
                Some(strategy_policy_profile_id(strategy).to_string()),
                Some("ph1k_handoff_clarify_only".to_string()),
                false,
            )
            .ok();
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1_with_metadata(
                reason_codes::STT_FAIL_AUDIO_DEGRADED,
                RetryAdvice::SwitchToText,
                reject_meta,
            ));
        }
        let ladder = select_provider_ladder(strategy);

        let mut attempts_used: u8 = 0;
        let mut total_latency_ms: u32 = 0;

        let mut best_fail: Option<ReasonCodeId> = None;
        let mut budget_exceeded = false;

        for slot in ladder {
            let mut slot_fail: Option<ReasonCodeId> = Some(reason_codes::STT_FAIL_PROVIDER_TIMEOUT);
            let mut slot_attempts: u8 = 0;
            let provider_attempt_cap = 1u8.saturating_add(self.config.max_retries_per_provider);

            for att in attempts.iter().filter(|a| a.provider == slot) {
                if slot_attempts >= provider_attempt_cap {
                    break;
                }
                if attempts_used >= self.config.max_attempts_per_turn {
                    budget_exceeded = true;
                    break;
                }
                if total_latency_ms.saturating_add(att.latency_ms)
                    > self.config.max_total_latency_budget_ms
                {
                    budget_exceeded = true;
                    break;
                }

                slot_attempts = slot_attempts.saturating_add(1);
                attempts_used = attempts_used.saturating_add(1);
                total_latency_ms = total_latency_ms.saturating_add(att.latency_ms);

                match self.eval_attempt(req, att) {
                    AttemptEval::Ok { out } => {
                        let audit_meta = match build_audit_meta(
                            &self.config,
                            attempts_used,
                            attempts_used,
                            selected_slot_for_provider(slot),
                            total_latency_ms,
                            quality_bucket_from_confidence(out.confidence_bucket),
                            quality_bucket_from_confidence(out.confidence_bucket),
                            QualityBucket::High,
                            None,
                            None,
                            Some(strategy_policy_profile_id(strategy).to_string()),
                            Some("openai_google_clarify_v1".to_string()),
                            attempts_used > 1,
                        ) {
                            Ok(meta) => meta,
                            Err(_) => {
                                return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                                    reason_codes::STT_FAIL_POLICY_RESTRICTED,
                                    RetryAdvice::Repeat,
                                ));
                            }
                        };
                        let ok = TranscriptOk::v1_with_metadata(
                            out.transcript_text,
                            out.language_tag,
                            out.confidence_bucket,
                            out.uncertain_spans,
                            Some(audit_meta),
                        );
                        let Ok(ok) = ok else {
                            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                                reason_codes::STT_FAIL_POLICY_RESTRICTED,
                                RetryAdvice::Repeat,
                            ));
                        };
                        return Ph1cResponse::TranscriptOk(ok);
                    }
                    AttemptEval::Reject { reason } => {
                        slot_fail = Some(select_more_specific_failure(slot_fail, reason));
                    }
                }
            }

            if budget_exceeded {
                break;
            }

            if let Some(reason) = slot_fail {
                best_fail = Some(select_more_specific_failure(best_fail, reason));
            }

            if attempts_used >= self.config.max_attempts_per_turn {
                budget_exceeded = true;
                break;
            }
        }

        let reason = if budget_exceeded {
            reason_codes::STT_FAIL_BUDGET_EXCEEDED
        } else {
            best_fail.unwrap_or(reason_codes::STT_FAIL_BUDGET_EXCEEDED)
        };
        let reject_meta = build_audit_meta(
            &self.config,
            attempts_used,
            attempts_used,
            SelectedSlot::None,
            total_latency_ms,
            QualityBucket::Low,
            QualityBucket::Low,
            QualityBucket::Low,
            None,
            None,
            Some(strategy_policy_profile_id(strategy).to_string()),
            Some("openai_google_clarify_v1".to_string()),
            attempts_used > 1,
        )
        .ok();
        Ph1cResponse::TranscriptReject(TranscriptReject::v1_with_metadata(
            reason,
            retry_advice_for(reason),
            reject_meta,
        ))
    }

    /// Live PH1.D provider path for STT, consumed directly by PH1.C before transcript gating.
    pub fn run_via_live_provider_adapter<A: Ph1dProviderAdapter>(
        &self,
        req: &Ph1cRequest,
        live: &Ph1cLiveProviderContext,
        adapter: &A,
    ) -> Ph1cResponse {
        self.run_via_live_provider_adapter_at_ms(req, live, adapter, unix_now_ms())
    }

    fn run_via_live_provider_adapter_at_ms<A: Ph1dProviderAdapter>(
        &self,
        req: &Ph1cRequest,
        live: &Ph1cLiveProviderContext,
        adapter: &A,
        now_ms: u64,
    ) -> Ph1cResponse {
        if req.validate().is_err() || live.validate().is_err() {
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason_codes::STT_FAIL_POLICY_RESTRICTED,
                RetryAdvice::SwitchToText,
            ));
        }

        let strategy = select_stt_strategy(req);
        if matches!(strategy, Ph1cSttStrategy::ClarifyOnly) {
            return self.run(req, &[]);
        }
        let ladder = select_provider_ladder(strategy);
        let ladder_has_secondary = ladder.iter().any(|slot| *slot == ProviderSlot::Secondary);
        let mut attempts = Vec::new();
        let mut provider_fail: Option<ReasonCodeId> = None;
        let retries = live.retry_budget.min(self.config.max_retries_per_provider);
        let mut primary_success: Option<(SttAttempt, TranscriptOk)> = None;
        let mut primary_total_latency_ms: u32 = 0;

        for slot in ladder {
            let circuit_key = circuit_breaker_key_for_slot(live, slot);
            if self.is_circuit_open(&circuit_key, now_ms) {
                provider_fail = Some(select_more_specific_failure(
                    provider_fail,
                    reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN,
                ));
                continue;
            }

            for retry_ix in 0..=retries {
                if self.is_circuit_open(&circuit_key, now_ms) {
                    provider_fail = Some(select_more_specific_failure(
                        provider_fail,
                        reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN,
                    ));
                    break;
                }

                let provider_req = match build_stt_provider_call_request(req, live, slot, retry_ix)
                {
                    Ok(v) => v,
                    Err(_) => {
                        provider_fail = Some(select_more_specific_failure(
                            provider_fail,
                            reason_codes::STT_FAIL_POLICY_RESTRICTED,
                        ));
                        continue;
                    }
                };

                match adapter.execute(&provider_req) {
                    Ok(provider_resp) => {
                        match stt_attempt_from_provider_response(slot, &provider_resp, live, now_ms)
                        {
                            Ok(attempt) => {
                                self.on_provider_success(&circuit_key);
                                let eval = self.eval_attempt(req, &attempt);
                                attempts.push(attempt);
                                if let AttemptEval::Ok { out } = eval {
                                    if slot == ProviderSlot::Primary
                                        && live.enforce_provider_disagreement_clarify
                                        && ladder_has_secondary
                                    {
                                        primary_total_latency_ms =
                                            attempts.iter().map(|a| a.latency_ms).sum();
                                        let Some(primary_attempt) = attempts.last().cloned() else {
                                            provider_fail = Some(select_more_specific_failure(
                                                provider_fail,
                                                reason_codes::STT_FAIL_POLICY_RESTRICTED,
                                            ));
                                            continue;
                                        };
                                        primary_success = Some((primary_attempt, out));
                                        break;
                                    }

                                    if slot == ProviderSlot::Secondary {
                                        if let Some((primary_attempt, primary_ok)) =
                                            &primary_success
                                        {
                                            let Some(secondary_attempt) = attempts.last().cloned()
                                            else {
                                                provider_fail = Some(select_more_specific_failure(
                                                    provider_fail,
                                                    reason_codes::STT_FAIL_POLICY_RESTRICTED,
                                                ));
                                                continue;
                                            };
                                            let divergence_bp = transcript_divergence_basis_points(
                                                &primary_attempt.transcript_text,
                                                &secondary_attempt.transcript_text,
                                            );
                                            if live.enforce_provider_disagreement_clarify
                                                && divergence_bp
                                                    > live
                                                        .provider_disagreement_divergence_bp_threshold
                                            {
                                                return Ph1cResponse::TranscriptReject(
                                                    TranscriptReject::v1(
                                                        reason_codes::STT_FAIL_PROVIDER_DISAGREEMENT,
                                                        RetryAdvice::Repeat,
                                                    ),
                                                );
                                            }
                                            let preferred_slot = choose_cost_quality_slot_for_pair(
                                                primary_attempt,
                                                &secondary_attempt,
                                                live,
                                                self.config.cost_quality_confidence_tolerance_bp,
                                            );
                                            let (chosen_out, chosen_slot) =
                                                if preferred_slot == ProviderSlot::Secondary {
                                                    (out, SelectedSlot::Secondary)
                                                } else {
                                                    (primary_ok.clone(), SelectedSlot::Primary)
                                                };
                                            let total_latency = primary_total_latency_ms
                                                .saturating_add(secondary_attempt.latency_ms);
                                            return transcript_ok_with_audit(
                                                &self.config,
                                                strategy,
                                                &chosen_out,
                                                chosen_slot,
                                                2,
                                                2,
                                                total_latency,
                                                true,
                                            );
                                        }
                                    }

                                    return transcript_ok_with_audit(
                                        &self.config,
                                        strategy,
                                        &out,
                                        selected_slot_for_provider(slot),
                                        attempts.len().min(u8::MAX as usize) as u8,
                                        attempts.len().min(u8::MAX as usize) as u8,
                                        attempts.iter().map(|a| a.latency_ms).sum(),
                                        attempts.len() > 1,
                                    );
                                }
                            }
                            Err(reason) => {
                                if is_provider_failure_reason(reason) {
                                    self.on_provider_failure(&circuit_key, now_ms);
                                }
                                provider_fail =
                                    Some(select_more_specific_failure(provider_fail, reason));
                            }
                        }
                    }
                    Err(Ph1dProviderAdapterError { retryable, .. }) => {
                        self.on_provider_failure(&circuit_key, now_ms);
                        provider_fail = Some(select_more_specific_failure(
                            provider_fail,
                            reason_codes::STT_FAIL_PROVIDER_TIMEOUT,
                        ));
                        if !retryable {
                            break;
                        }
                    }
                }
            }
        }

        if let Some((_primary_attempt, primary_ok)) = primary_success {
            if live.enforce_provider_disagreement_clarify && ladder_has_secondary {
                return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                    reason_codes::STT_FAIL_PROVIDER_DISAGREEMENT,
                    RetryAdvice::Repeat,
                ));
            }
            return transcript_ok_with_audit(
                &self.config,
                strategy,
                &primary_ok,
                SelectedSlot::Primary,
                1,
                1,
                primary_total_latency_ms,
                true,
            );
        }

        if attempts.is_empty() {
            let reason = provider_fail.unwrap_or(reason_codes::STT_FAIL_PROVIDER_TIMEOUT);
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason,
                retry_advice_for(reason),
            ));
        }

        self.run(req, &attempts)
    }

    pub fn run_stream_via_live_provider_adapter<A: Ph1dProviderAdapter>(
        &self,
        req: &Ph1cRequest,
        live: &Ph1cLiveProviderContext,
        adapter: &A,
    ) -> Ph1cStreamCommit {
        self.run_stream_via_live_provider_adapter_at_ms(req, live, adapter, unix_now_ms())
    }

    fn run_stream_via_live_provider_adapter_at_ms<A: Ph1dProviderAdapter>(
        &self,
        req: &Ph1cRequest,
        live: &Ph1cLiveProviderContext,
        adapter: &A,
        now_ms: u64,
    ) -> Ph1cStreamCommit {
        if req.validate().is_err() || live.validate().is_err() {
            let reason = reason_codes::STT_FAIL_POLICY_RESTRICTED;
            return Ph1cStreamCommit {
                partial_batch: None,
                response: Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                    reason,
                    retry_advice_for(reason),
                )),
                low_latency_commit: false,
                finalized: false,
            };
        }

        let strategy = select_stt_strategy(req);
        if matches!(strategy, Ph1cSttStrategy::ClarifyOnly) {
            return Ph1cStreamCommit {
                partial_batch: None,
                response: self.run(req, &[]),
                low_latency_commit: false,
                finalized: false,
            };
        }

        let ladder = select_provider_ladder(strategy);
        let retries = live.retry_budget.min(self.config.max_retries_per_provider);
        let mut provider_fail: Option<ReasonCodeId> = None;

        for slot in ladder {
            let circuit_key = circuit_breaker_key_for_slot(live, slot);
            if self.is_circuit_open(&circuit_key, now_ms) {
                provider_fail = Some(select_more_specific_failure(
                    provider_fail,
                    reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN,
                ));
                continue;
            }

            let mut stream_partials: Vec<SttPartialAttempt> = Vec::new();
            let mut next_revision_id: u32 = 1;

            for stream_ix in 0..self.config.stream_max_revisions {
                if self.is_circuit_open(&circuit_key, now_ms) {
                    provider_fail = Some(select_more_specific_failure(
                        provider_fail,
                        reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN,
                    ));
                    break;
                }

                let mut got_frame: Option<LiveSttStreamFrame> = None;
                for retry_ix in 0..=retries {
                    let provider_req = match build_streaming_stt_provider_call_request(
                        req,
                        live,
                        slot,
                        retry_ix,
                        stream_ix,
                        next_revision_id,
                    ) {
                        Ok(v) => v,
                        Err(_) => {
                            provider_fail = Some(select_more_specific_failure(
                                provider_fail,
                                reason_codes::STT_FAIL_POLICY_RESTRICTED,
                            ));
                            continue;
                        }
                    };

                    match adapter.execute(&provider_req) {
                        Ok(provider_resp) => {
                            match stt_stream_frame_from_provider_response(
                                slot,
                                &provider_resp,
                                live,
                                now_ms,
                            ) {
                                Ok(frame) => {
                                    self.on_provider_success(&circuit_key);
                                    got_frame = Some(frame);
                                    break;
                                }
                                Err(reason) => {
                                    if is_provider_failure_reason(reason) {
                                        self.on_provider_failure(&circuit_key, now_ms);
                                    }
                                    provider_fail =
                                        Some(select_more_specific_failure(provider_fail, reason));
                                }
                            }
                        }
                        Err(Ph1dProviderAdapterError { retryable, .. }) => {
                            self.on_provider_failure(&circuit_key, now_ms);
                            provider_fail = Some(select_more_specific_failure(
                                provider_fail,
                                reason_codes::STT_FAIL_PROVIDER_TIMEOUT,
                            ));
                            if !retryable {
                                break;
                            }
                        }
                    }
                }

                let Some(frame) = got_frame else {
                    break;
                };
                next_revision_id = frame.partial.revision_id.saturating_add(1);
                stream_partials.push(frame.partial.clone());

                let partial_batch = match self
                    .canonicalize_partial_transcripts(&stream_partials, frame.finalized)
                {
                    Ok(v) => v,
                    Err(reject) => {
                        provider_fail = Some(select_more_specific_failure(
                            provider_fail,
                            reject.reason_code,
                        ));
                        break;
                    }
                };

                if frame.finalized {
                    let response = self.run(req, &[frame.attempt]);
                    return Ph1cStreamCommit {
                        partial_batch: Some(partial_batch),
                        response,
                        low_latency_commit: false,
                        finalized: true,
                    };
                }

                if self.is_low_latency_commit_candidate(req, &frame.attempt) {
                    let response = self.run(req, &[frame.attempt]);
                    return Ph1cStreamCommit {
                        partial_batch: Some(partial_batch),
                        response,
                        low_latency_commit: true,
                        finalized: false,
                    };
                }
            }
        }

        let reason = provider_fail.unwrap_or(reason_codes::STT_FAIL_PROVIDER_TIMEOUT);
        Ph1cStreamCommit {
            partial_batch: None,
            response: Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason,
                retry_advice_for(reason),
            )),
            low_latency_commit: false,
            finalized: false,
        }
    }

    pub fn evaluate_inhouse_shadow_route(
        &self,
        req: &Ph1cRequest,
        slice_key: Ph1cShadowSliceKey,
        provider_truth: &SttAttempt,
        inhouse_shadow: &SttAttempt,
        governed_gate_passed: bool,
    ) -> Result<Ph1cShadowRouteOutcome, ReasonCodeId> {
        if !valid_shadow_slice_key(&slice_key) {
            return Err(reason_codes::STT_FAIL_SHADOW_INPUT_INVALID);
        }
        if !locale_family_matches(&slice_key.locale, provider_truth.language_tag.as_str())
            || !locale_family_matches(&slice_key.locale, inhouse_shadow.language_tag.as_str())
        {
            return Err(reason_codes::STT_FAIL_SHADOW_INPUT_INVALID);
        }

        if !matches!(
            self.eval_attempt(req, provider_truth),
            AttemptEval::Ok { .. }
        ) {
            return Err(reason_codes::STT_FAIL_SHADOW_PROVIDER_TRUTH_INVALID);
        }

        let overlap_bp = overlap_basis_points(token_overlap_ratio(
            &provider_truth.transcript_text,
            &inhouse_shadow.transcript_text,
        ));
        let confidence_delta_bp = confidence_delta_basis_points(
            provider_truth.avg_word_confidence,
            inhouse_shadow.avg_word_confidence,
        );
        let latency_delta_ms = inhouse_shadow.latency_ms as i32 - provider_truth.latency_ms as i32;

        let inhouse_eval = self.eval_attempt(req, inhouse_shadow);
        let decision = if !governed_gate_passed {
            Ph1cShadowRouteDecision::HoldShadow
        } else if matches!(inhouse_eval, AttemptEval::Ok { .. })
            && overlap_bp >= 9_200
            && confidence_delta_bp >= -300
            && inhouse_shadow.latency_ms <= provider_truth.latency_ms.saturating_add(250)
        {
            Ph1cShadowRouteDecision::EligibleForPromotion
        } else {
            Ph1cShadowRouteDecision::HoldShadow
        };

        let block_reason_code = if matches!(decision, Ph1cShadowRouteDecision::EligibleForPromotion)
        {
            None
        } else if !governed_gate_passed {
            Some(reason_codes::STT_FAIL_SHADOW_PROMOTION_BLOCKED)
        } else {
            Some(match inhouse_eval {
                AttemptEval::Reject { reason } => reason,
                AttemptEval::Ok { .. } => reason_codes::STT_FAIL_SHADOW_PROMOTION_BLOCKED,
            })
        };

        Ok(Ph1cShadowRouteOutcome {
            slice_key,
            transcript_overlap_bp: overlap_bp,
            confidence_delta_bp,
            latency_delta_ms,
            governed_gate_passed,
            decision,
            block_reason_code,
        })
    }

    fn is_low_latency_commit_candidate(&self, req: &Ph1cRequest, attempt: &SttAttempt) -> bool {
        if !attempt.stable {
            return false;
        }
        if attempt.avg_word_confidence < self.config.stream_low_latency_confidence_min {
            return false;
        }
        if attempt.transcript_text.chars().count() < self.config.stream_low_latency_min_chars {
            return false;
        }
        matches!(self.eval_attempt(req, attempt), AttemptEval::Ok { .. })
    }

    fn is_circuit_open(&self, key: &CircuitBreakerKey, now_ms: u64) -> bool {
        let mut book = match self.circuit_breakers.lock() {
            Ok(book) => book,
            Err(_) => {
                // Fail closed on internal state corruption.
                return true;
            }
        };
        let Some(entry) = book.entries.get_mut(key) else {
            return false;
        };
        if entry.open_until_ms > now_ms {
            return true;
        }
        if entry.consecutive_failures == 0 {
            book.entries.remove(key);
        }
        false
    }

    fn on_provider_failure(&self, key: &CircuitBreakerKey, now_ms: u64) {
        let mut book = match self.circuit_breakers.lock() {
            Ok(book) => book,
            Err(_) => {
                return;
            }
        };
        let threshold = self.circuit_breaker_config.failure_threshold;
        let cooldown_ms = u64::from(self.circuit_breaker_config.cooldown_ms);
        let entry = book
            .entries
            .entry(key.clone())
            .or_insert(CircuitBreakerEntry {
                consecutive_failures: 0,
                open_until_ms: 0,
            });
        if entry.open_until_ms > now_ms {
            return;
        }
        entry.consecutive_failures = entry.consecutive_failures.saturating_add(1);
        if entry.consecutive_failures >= threshold {
            entry.consecutive_failures = 0;
            entry.open_until_ms = now_ms.saturating_add(cooldown_ms);
        }
    }

    fn on_provider_success(&self, key: &CircuitBreakerKey) {
        let mut book = match self.circuit_breakers.lock() {
            Ok(book) => book,
            Err(_) => {
                return;
            }
        };
        book.entries.remove(key);
    }

    pub fn canonicalize_partial_transcripts(
        &self,
        attempts: &[SttPartialAttempt],
        finalized: bool,
    ) -> Result<PartialTranscriptBatch, TranscriptReject> {
        if attempts.is_empty() {
            return Err(partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID));
        }

        let mut by_revision: std::collections::BTreeMap<u32, PartialTranscript> =
            std::collections::BTreeMap::new();
        for attempt in attempts {
            let confidence = Confidence::new(attempt.confidence)
                .map_err(|_| partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID))?;
            let candidate = PartialTranscript::v1(
                attempt.text_chunk.clone(),
                confidence,
                attempt.stable,
                attempt.revision_id,
            )
            .map_err(|_| partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID))?;

            if let Some(existing) = by_revision.get(&attempt.revision_id) {
                if prefer_partial_candidate(existing, &candidate) {
                    by_revision.insert(attempt.revision_id, candidate);
                }
            } else {
                by_revision.insert(attempt.revision_id, candidate);
            }
        }

        if by_revision.is_empty() {
            return Err(partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID));
        }

        let mut partials = Vec::with_capacity(by_revision.len());
        for (idx, partial) in by_revision.values().enumerate() {
            let expected_revision = (idx as u32) + 1;
            if partial.revision_id != expected_revision {
                return Err(partial_reject(reason_codes::STT_FAIL_PARTIAL_ORDER));
            }
            partials.push(partial.clone());
        }

        if finalized && !partials.last().is_some_and(|p| p.stable) {
            return Err(partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID));
        }

        PartialTranscriptBatch::v1(partials, finalized)
            .map_err(|_| partial_reject(reason_codes::STT_FAIL_PARTIAL_INVALID))
    }

    fn eval_attempt(&self, req: &Ph1cRequest, att: &SttAttempt) -> AttemptEval {
        let first_pass = self.eval_attempt_raw(req, att);
        if matches!(first_pass, AttemptEval::Ok { .. }) {
            return first_pass;
        }

        // Two-pass decode: first pass is fast raw gating; second pass is bounded repair.
        if let Some(repaired) = self.maybe_intent_aware_repair(req, att) {
            let second_pass = self.eval_attempt_raw(req, &repaired);
            if matches!(second_pass, AttemptEval::Ok { .. }) {
                return second_pass;
            }
        }
        first_pass
    }

    fn eval_attempt_raw(&self, req: &Ph1cRequest, att: &SttAttempt) -> AttemptEval {
        let t = att.transcript_text.trim();
        if t.is_empty() {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_EMPTY,
            };
        }

        if is_garbled_or_stutter(t) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_GARBLED,
            };
        }

        if is_language_mismatch(req, &att.language_tag) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LANGUAGE_MISMATCH,
            };
        }

        if !coverage_ok(&self.config, req, t) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_COVERAGE,
            };
        }

        if self.config.require_overlap_disambiguation
            && overlap_hint_requires_disambiguation(req, t)
        {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_SPEAKER_OVERLAP_AMBIGUOUS,
            };
        }

        let calibrated = calibrated_confidence(req, att);
        let conf_bucket = calibrated.bucket;
        if bucket_rank(conf_bucket) < bucket_rank(self.config.min_confidence_bucket_to_pass) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_CONFIDENCE,
            };
        }

        if !confidence_ok(&self.config, att, calibrated.score) {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_CONFIDENCE,
            };
        }

        if self.config.semantic_gate_enabled
            && !semantic_quality_ok(req, t, &att.language_tag, self.config.min_semantic_quality)
        {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_LOW_SEMANTIC_CONFIDENCE,
            };
        }

        let Ok(ok) = TranscriptOk::v1(t.to_string(), att.language_tag.clone(), conf_bucket) else {
            return AttemptEval::Reject {
                reason: reason_codes::STT_FAIL_POLICY_RESTRICTED,
            };
        };
        AttemptEval::Ok { out: ok }
    }

    fn maybe_intent_aware_repair(&self, req: &Ph1cRequest, att: &SttAttempt) -> Option<SttAttempt> {
        let original = att.transcript_text.trim();
        if !is_intent_repair_candidate(att, original) {
            return None;
        }

        let correlation_seed = fnv1a64(
            format!(
                "ph1c_srl:{}:{}:{}:{}",
                req.bounded_audio_segment_ref.stream_id.0,
                req.bounded_audio_segment_ref.t_start.0,
                req.bounded_audio_segment_ref.t_end.0,
                original
            )
            .as_bytes(),
        );
        let correlation_id = CorrelationId(
            ((nonzero_u64(correlation_seed) as u128) << 64)
                | (nonzero_u64(correlation_seed.rotate_left(13)) as u128),
        );
        let turn_id = TurnId(nonzero_u64(fnv1a64(
            format!(
                "ph1c_srl_turn:{}:{}",
                req.bounded_audio_segment_ref.stream_id.0, req.bounded_audio_segment_ref.t_end.0
            )
            .as_bytes(),
        )));
        let envelope = SrlRequestEnvelope::v1(correlation_id, turn_id, 32, 16, 8, 8).ok()?;
        let transcript_hash = format!("{:016x}", fnv1a64(original.as_bytes()));
        let hints = collect_intent_hint_tokens(original);
        let language_tag = normalize_locale_tag(att.language_tag.as_str());
        let srl_runtime = Ph1SrlRuntime::new(Ph1SrlConfig::mvp_v1());

        let frame_req = SrlFrameBuildRequest::v1(
            envelope.clone(),
            transcript_hash.clone(),
            original.to_string(),
            language_tag.clone(),
            vec![],
            hints,
            true,
            true,
            true,
        )
        .ok()?;

        let frame_ok = match srl_runtime.run(&Ph1SrlRequest::SrlFrameBuild(frame_req)) {
            Ph1SrlResponse::SrlFrameBuildOk(ok) => ok,
            _ => return None,
        };

        let normalize_req = SrlArgumentNormalizeRequest::v1(
            envelope,
            transcript_hash,
            frame_ok.repaired_transcript_text.clone(),
            frame_ok.frame_spans.clone(),
            frame_ok.repair_notes.clone(),
            frame_ok.ambiguity_flags.clone(),
            true,
            true,
            true,
            true,
        )
        .ok()?;

        let normalize_ok =
            match srl_runtime.run(&Ph1SrlRequest::SrlArgumentNormalize(normalize_req)) {
                Ph1SrlResponse::SrlArgumentNormalizeOk(ok)
                    if ok.validation_status == SrlValidationStatus::Ok && !ok.clarify_required =>
                {
                    ok
                }
                _ => return None,
            };

        let repaired = collapse_disfluencies_from_spans(&normalize_ok.normalized_frame_spans);
        if repaired.is_empty()
            || lowercase_text(&repaired) == lowercase_text(original)
            || token_overlap_ratio(original, &repaired) < INTENT_REPAIR_MIN_OVERLAP_RATIO
        {
            return None;
        }

        if !nlp_repair_accept(req, original, &repaired, &att.language_tag) {
            return None;
        }

        let mut repaired_attempt = att.clone();
        repaired_attempt.transcript_text = repaired;
        repaired_attempt.avg_word_confidence =
            clamp01(repaired_attempt.avg_word_confidence + INTENT_REPAIR_CONFIDENCE_BOOST);
        repaired_attempt.low_confidence_ratio =
            clamp01(repaired_attempt.low_confidence_ratio * INTENT_REPAIR_LOW_RATIO_SCALE);
        Some(repaired_attempt)
    }
}

#[derive(Debug, Clone)]
enum AttemptEval {
    Ok { out: TranscriptOk },
    Reject { reason: ReasonCodeId },
}

#[derive(Debug, Clone)]
struct LiveSttStreamFrame {
    attempt: SttAttempt,
    partial: SttPartialAttempt,
    finalized: bool,
}

fn build_stt_provider_call_request(
    req: &Ph1cRequest,
    live: &Ph1cLiveProviderContext,
    slot: ProviderSlot,
    retry_ix: u8,
) -> Result<Ph1dProviderCallRequest, ContractViolationLocal> {
    let (provider_id, model_id) = provider_and_model_for_slot(live, slot);
    let route_class = provider_route_for_slot(slot);
    let slot_label = provider_slot_label(slot);
    let scoped_idempotency_key =
        scoped_idempotency_key(&live.idempotency_key, slot_label, retry_ix);
    let input_payload_ref = format!(
        "ph1c_audio/{}/{}/{}",
        req.bounded_audio_segment_ref.stream_id.0,
        req.bounded_audio_segment_ref.t_start.0,
        req.bounded_audio_segment_ref.t_end.0
    );
    let payload_inline = build_audio_inline_payload(req, live)?;
    let payload_hash = SchemaHash(nonzero_u64(fnv1a64(payload_inline.as_bytes())));

    Ph1dProviderCallRequest::v1(
        live.correlation_id,
        live.turn_id,
        live.tenant_id.clone(),
        live.request_id,
        scoped_idempotency_key,
        Ph1dProviderTask::SttTranscribe,
        route_class,
        provider_id.to_string(),
        model_id.to_string(),
        live.timeout_ms,
        live.retry_budget,
        Some("ph1c_live_stt_v1".to_string()),
        None,
        SchemaVersion(1),
        PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
        live.tool_catalog_hash,
        live.policy_context_hash,
        None,
        input_payload_ref,
        Ph1dProviderInputPayloadKind::Audio,
        payload_hash,
        Some(payload_inline),
        Some("application/json".to_string()),
        live.safety_tier,
        live.privacy_mode,
        live.do_not_disturb,
    )
    .map_err(|_| {
        ContractViolationLocal::InvalidValue("ph1c_live_provider_context.provider_request")
    })
}

fn build_streaming_stt_provider_call_request(
    req: &Ph1cRequest,
    live: &Ph1cLiveProviderContext,
    slot: ProviderSlot,
    retry_ix: u8,
    stream_ix: u32,
    next_revision_id: u32,
) -> Result<Ph1dProviderCallRequest, ContractViolationLocal> {
    let mut provider_req = build_stt_provider_call_request(req, live, slot, retry_ix)?;
    provider_req.prompt_template_ref = Some("ph1c_live_stt_stream_v1".to_string());
    provider_req.transcript_ref = Some(format!(
        "ph1c_stt_stream:{}:{}:{}",
        live.request_id.0, stream_ix, next_revision_id
    ));
    provider_req.validate().map_err(|_| {
        ContractViolationLocal::InvalidValue("ph1c_live_provider_context.stream_request")
    })?;
    Ok(provider_req)
}

fn provider_and_model_for_slot(live: &Ph1cLiveProviderContext, slot: ProviderSlot) -> (&str, &str) {
    match slot {
        ProviderSlot::Primary => (&live.primary_provider_id, &live.primary_model_id),
        ProviderSlot::Secondary => (&live.secondary_provider_id, &live.secondary_model_id),
        ProviderSlot::Tertiary => (&live.secondary_provider_id, &live.secondary_model_id),
    }
}

fn provider_route_for_slot(slot: ProviderSlot) -> Ph1dProviderRouteClass {
    match slot {
        ProviderSlot::Primary => Ph1dProviderRouteClass::Primary,
        ProviderSlot::Secondary => Ph1dProviderRouteClass::Secondary,
        ProviderSlot::Tertiary => Ph1dProviderRouteClass::Tertiary,
    }
}

fn provider_slot_label(slot: ProviderSlot) -> &'static str {
    match slot {
        ProviderSlot::Primary => "primary",
        ProviderSlot::Secondary => "secondary",
        ProviderSlot::Tertiary => "tertiary",
    }
}

fn circuit_breaker_key_for_slot(
    live: &Ph1cLiveProviderContext,
    slot: ProviderSlot,
) -> CircuitBreakerKey {
    let (provider_id, model_id) = provider_and_model_for_slot(live, slot);
    CircuitBreakerKey::new(&live.tenant_id, provider_id, model_id)
}

fn is_provider_failure_reason(reason: ReasonCodeId) -> bool {
    matches!(
        reason,
        reason_codes::STT_FAIL_PROVIDER_TIMEOUT
            | reason_codes::STT_FAIL_POLICY_RESTRICTED
            | reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN
    )
}

fn scoped_idempotency_key(base: &str, slot_label: &str, retry_ix: u8) -> String {
    let candidate = format!("{base}:{slot_label}:{retry_ix}");
    if candidate.len() <= 128 && is_provider_token(&candidate, 128) {
        return candidate;
    }
    format!("ph1c_live_stt:{:016x}", fnv1a64(candidate.as_bytes()))
}

fn build_audio_inline_payload(
    req: &Ph1cRequest,
    live: &Ph1cLiveProviderContext,
) -> Result<String, ContractViolationLocal> {
    let tenant_lexicon_terms = normalized_lexicon_terms(&live.tenant_lexicon_terms);
    let domain_lexicon_terms = normalized_lexicon_terms(&live.domain_lexicon_terms);
    let now_s = unix_now_ms() / 1_000;
    let global_lexicon_terms_v2 = live
        .global_lexicon_terms_v2
        .iter()
        .filter(|entry| entry.expires_at_unix_s > now_s)
        .map(|entry| {
            serde_json::json!({
                "term": normalize_lexicon_term(&entry.term),
                "weight_bp": entry.weight_bp,
                "expires_at_unix_s": entry.expires_at_unix_s,
                "source_scope": match entry.source_scope {
                    Ph1cLexiconSourceScope::Global => "global",
                    Ph1cLexiconSourceScope::Tenant => "tenant",
                    Ph1cLexiconSourceScope::Domain => "domain",
                },
            })
        })
        .collect::<Vec<_>>();
    let payload = serde_json::json!({
        "stream_id": req.bounded_audio_segment_ref.stream_id.0.to_string(),
        "source_pre_roll_buffer_id": req.bounded_audio_segment_ref.source_pre_roll_buffer_id.0,
        "t_start_ns": req.bounded_audio_segment_ref.t_start.0,
        "t_end_ns": req.bounded_audio_segment_ref.t_end.0,
        "t_candidate_start_ns": req.bounded_audio_segment_ref.t_candidate_start.0,
        "t_confirmed_ns": req.bounded_audio_segment_ref.t_confirmed.0,
        "language_hint": req
            .language_hint
            .as_ref()
            .map(|hint| hint.language_tag.as_str().to_string()),
        "tenant_vocabulary_pack_id": live.tenant_vocabulary_pack_id.clone(),
        "user_vocabulary_pack_id": live.user_vocabulary_pack_id.clone(),
        "tenant_lexicon_terms": tenant_lexicon_terms,
        "domain_lexicon_terms": domain_lexicon_terms,
        "global_lexicon_terms_v2": global_lexicon_terms_v2,
    });
    serde_json::to_string(&payload)
        .map_err(|_| ContractViolationLocal::InvalidValue("ph1c_live_provider_context.payload"))
}

fn stt_attempt_from_provider_response(
    slot: ProviderSlot,
    provider_resp: &Ph1dProviderCallResponse,
    live: &Ph1cLiveProviderContext,
    now_ms: u64,
) -> Result<SttAttempt, ReasonCodeId> {
    if provider_resp.provider_task != Ph1dProviderTask::SttTranscribe {
        return Err(reason_codes::STT_FAIL_POLICY_RESTRICTED);
    }
    if provider_resp.provider_status != Ph1dProviderStatus::Ok
        || provider_resp.validation_status != Ph1dProviderValidationStatus::SchemaOk
    {
        return Err(provider_error_to_stt_reason(provider_resp));
    }
    if provider_resp.normalized_output_schema_hash
        != Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
    {
        return Err(reason_codes::STT_FAIL_POLICY_RESTRICTED);
    }
    let normalized_json = provider_resp
        .normalized_output_json
        .as_deref()
        .ok_or(reason_codes::STT_FAIL_POLICY_RESTRICTED)?;
    let normalized = decode_normalized_output_json(normalized_json)
        .map_err(|_| reason_codes::STT_FAIL_POLICY_RESTRICTED)?;
    if normalized.provider_task != Ph1dProviderTask::SttTranscribe {
        return Err(reason_codes::STT_FAIL_POLICY_RESTRICTED);
    }

    let transcript_text = normalized
        .text_output
        .map(|t| t.trim().to_string())
        .filter(|t| !t.is_empty())
        .ok_or(reason_codes::STT_FAIL_EMPTY)?;
    let fallback_language_tag = LanguageTag::new("und".to_string())
        .map_err(|_| reason_codes::STT_FAIL_POLICY_RESTRICTED)?;
    let language_tag = normalized
        .language_tag
        .and_then(|lang| LanguageTag::new(lang).ok())
        .unwrap_or(fallback_language_tag);
    let confidence_bp = normalized
        .confidence_bp
        .or(provider_resp.provider_confidence_bp)
        .unwrap_or(8_000)
        .min(10_000);
    let base_avg_word_confidence = (confidence_bp as f32) / 10_000.0;
    let mut avg_word_confidence = base_avg_word_confidence;
    let mut low_confidence_ratio = (1.0f32 - avg_word_confidence).clamp(0.0, 1.0);
    let lexicon_boost = lexicon_confidence_boost(
        &transcript_text,
        &live.tenant_lexicon_terms,
        &live.domain_lexicon_terms,
    );
    let lexicon_v2_boost =
        lexicon_confidence_boost_v2(&transcript_text, &live.global_lexicon_terms_v2, now_ms);
    let merged_boost = clamp01(lexicon_boost + lexicon_v2_boost).min(0.22);
    if merged_boost > 0.0 {
        avg_word_confidence = clamp01(avg_word_confidence + merged_boost);
        low_confidence_ratio = clamp01(low_confidence_ratio * (1.0 - (merged_boost * 0.8)));
    }
    let stable = normalized.stable.unwrap_or(true);

    Ok(SttAttempt {
        provider: slot,
        latency_ms: provider_resp.provider_latency_ms,
        transcript_text,
        language_tag,
        avg_word_confidence,
        low_confidence_ratio,
        stable,
    })
}

fn stt_stream_frame_from_provider_response(
    slot: ProviderSlot,
    provider_resp: &Ph1dProviderCallResponse,
    live: &Ph1cLiveProviderContext,
    now_ms: u64,
) -> Result<LiveSttStreamFrame, ReasonCodeId> {
    let attempt = stt_attempt_from_provider_response(slot, provider_resp, live, now_ms)?;
    let normalized_json = provider_resp
        .normalized_output_json
        .as_deref()
        .ok_or(reason_codes::STT_FAIL_POLICY_RESTRICTED)?;
    let (revision_id, finalized) = parse_streaming_revision_metadata(normalized_json)?;
    let partial = SttPartialAttempt {
        text_chunk: attempt.transcript_text.clone(),
        confidence: attempt.avg_word_confidence,
        stable: attempt.stable,
        revision_id,
    };
    Ok(LiveSttStreamFrame {
        attempt,
        partial,
        finalized,
    })
}

fn parse_streaming_revision_metadata(json_text: &str) -> Result<(u32, bool), ReasonCodeId> {
    let value: serde_json::Value =
        serde_json::from_str(json_text).map_err(|_| reason_codes::STT_FAIL_PARTIAL_INVALID)?;
    let obj = value
        .as_object()
        .ok_or(reason_codes::STT_FAIL_PARTIAL_INVALID)?;
    let revision_id = obj
        .get("revision_id")
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
        .filter(|v| *v > 0)
        .ok_or(reason_codes::STT_FAIL_PARTIAL_INVALID)?;
    let finalized = obj
        .get("finalized")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    Ok((revision_id, finalized))
}

fn provider_error_to_stt_reason(provider_resp: &Ph1dProviderCallResponse) -> ReasonCodeId {
    match provider_resp.reason_code {
        d_reason_codes::D_PROVIDER_TIMEOUT => reason_codes::STT_FAIL_PROVIDER_TIMEOUT,
        d_reason_codes::D_PROVIDER_CONTRACT_MISMATCH | d_reason_codes::D_PROVIDER_SCHEMA_DRIFT => {
            reason_codes::STT_FAIL_POLICY_RESTRICTED
        }
        _ => {
            if provider_resp.provider_status == Ph1dProviderStatus::Error {
                reason_codes::STT_FAIL_PROVIDER_TIMEOUT
            } else {
                reason_codes::STT_FAIL_POLICY_RESTRICTED
            }
        }
    }
}

fn is_openai_provider_id(provider_id: &str) -> bool {
    provider_id.trim().to_ascii_lowercase().contains("openai")
}

fn is_google_provider_id(provider_id: &str) -> bool {
    let normalized = provider_id.trim().to_ascii_lowercase();
    normalized.contains("google") || normalized.contains("gcp")
}

fn unix_now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|d| u64::try_from(d.as_millis()).ok())
        .unwrap_or(0)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0001_0000_01B3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

fn nonzero_u64(h: u64) -> u64 {
    if h == 0 {
        1
    } else {
        h
    }
}

fn is_provider_token(value: &str, max_len: usize) -> bool {
    if value.trim().is_empty() || value.len() > max_len {
        return false;
    }
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':' | '/'))
}

fn valid_shadow_slice_key(slice_key: &Ph1cShadowSliceKey) -> bool {
    if normalize_locale_tag(&slice_key.locale).is_empty() {
        return false;
    }
    if !is_provider_token(&slice_key.device_route, 64) {
        return false;
    }
    is_provider_token(&slice_key.tenant_id, 128)
}

fn overlap_basis_points(value: f32) -> u16 {
    (clamp01(value) * 10_000.0).round().clamp(0.0, 10_000.0) as u16
}

fn confidence_delta_basis_points(provider: f32, inhouse: f32) -> i16 {
    let delta = ((clamp01(inhouse) - clamp01(provider)) * 10_000.0).round();
    delta.clamp(i16::MIN as f32, i16::MAX as f32) as i16
}

fn valid_opt_pack_id(value: &Option<String>) -> bool {
    value
        .as_ref()
        .map(|v| is_provider_token(v, 128))
        .unwrap_or(true)
}

fn valid_lexicon_terms(terms: &[String]) -> bool {
    if terms.len() > 128 {
        return false;
    }
    terms.iter().all(|term| {
        !term.trim().is_empty() && term.len() <= 64 && !term.chars().any(|c| c.is_control())
    })
}

fn valid_lexicon_terms_v2(terms: &[Ph1cLexiconTermV2]) -> bool {
    if terms.len() > 256 {
        return false;
    }
    terms.iter().all(|entry| {
        !entry.term.trim().is_empty()
            && entry.term.len() <= 64
            && !entry.term.chars().any(|c| c.is_control())
            && entry.weight_bp <= 10_000
            && entry.expires_at_unix_s > 0
    })
}

fn normalized_lexicon_terms(terms: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for term in terms {
        let normalized = normalize_lexicon_term(term);
        if normalized.is_empty() || normalized.len() > 64 {
            continue;
        }
        if !out.iter().any(|existing| existing == &normalized) {
            out.push(normalized);
        }
    }
    out.sort();
    out
}

fn normalize_lexicon_term(term: &str) -> String {
    let mut out = String::with_capacity(term.len());
    for c in term.trim().chars() {
        if c.is_alphanumeric() {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else if c.is_whitespace() || matches!(c, '-' | '_' | '.' | '/') {
            if !out.ends_with(' ') {
                out.push(' ');
            }
        }
    }
    out.trim().to_string()
}

fn lexicon_confidence_boost(
    transcript_text: &str,
    tenant_terms: &[String],
    domain_terms: &[String],
) -> f32 {
    let transcript = normalize_lexicon_term(transcript_text);
    if transcript.is_empty() {
        return 0.0;
    }
    let mut merged_terms = normalized_lexicon_terms(tenant_terms);
    for term in normalized_lexicon_terms(domain_terms) {
        if !merged_terms.iter().any(|t| t == &term) {
            merged_terms.push(term);
        }
    }
    if merged_terms.is_empty() {
        return 0.0;
    }

    let matched = merged_terms
        .iter()
        .filter(|term| transcript.contains(term.as_str()))
        .count();
    if matched == 0 {
        return 0.0;
    }

    let denom = merged_terms.len().min(8) as f32;
    let ratio = (matched as f32 / denom).clamp(0.0, 1.0);
    clamp01((ratio * 0.12) + 0.03).min(0.15)
}

fn lexicon_confidence_boost_v2(
    transcript_text: &str,
    terms: &[Ph1cLexiconTermV2],
    now_ms: u64,
) -> f32 {
    let transcript = normalize_lexicon_term(transcript_text);
    if transcript.is_empty() || terms.is_empty() {
        return 0.0;
    }
    let now_s = now_ms / 1_000;
    let mut weighted_match_bp: u32 = 0;
    let mut weighted_total_bp: u32 = 0;
    for term in terms {
        if term.expires_at_unix_s <= now_s {
            continue;
        }
        let normalized_term = normalize_lexicon_term(&term.term);
        if normalized_term.is_empty() {
            continue;
        }
        weighted_total_bp = weighted_total_bp.saturating_add(u32::from(term.weight_bp));
        if transcript.contains(normalized_term.as_str()) {
            weighted_match_bp = weighted_match_bp.saturating_add(u32::from(term.weight_bp));
        }
    }
    if weighted_total_bp == 0 || weighted_match_bp == 0 {
        return 0.0;
    }
    let ratio = (weighted_match_bp as f32 / weighted_total_bp as f32).clamp(0.0, 1.0);
    clamp01((ratio * 0.15) + 0.02).min(0.18)
}

fn strategy_policy_profile_id(strategy: Ph1cSttStrategy) -> &'static str {
    match strategy {
        Ph1cSttStrategy::Standard => "ph1k_handoff_standard",
        Ph1cSttStrategy::NoiseRobust => "ph1k_handoff_noise_robust",
        Ph1cSttStrategy::CloudAssist => "ph1k_handoff_cloud_assist",
        Ph1cSttStrategy::ClarifyOnly => "ph1k_handoff_clarify_only",
    }
}

fn select_stt_strategy(req: &Ph1cRequest) -> Ph1cSttStrategy {
    let Some(h) = &req.ph1k_handoff else {
        return Ph1cSttStrategy::Standard;
    };

    if matches!(
        h.degradation_class_bundle.capture_quality_class,
        CaptureQualityClass::Critical
    ) || matches!(
        h.degradation_class_bundle.recoverability_class,
        RecoverabilityClass::FailoverRequired
    ) {
        return Ph1cSttStrategy::ClarifyOnly;
    }

    if h.quality_metrics.packet_loss_pct >= 4.0
        || h.quality_metrics.snr_db < 14.0
        || matches!(
            h.degradation_class_bundle.network_stability_class,
            NetworkStabilityClass::Flaky | NetworkStabilityClass::Unstable
        )
    {
        return Ph1cSttStrategy::NoiseRobust;
    }

    if matches!(
        h.interrupt_confidence_band,
        InterruptCandidateConfidenceBand::Low
    ) || matches!(h.vad_confidence_band, VadDecisionConfidenceBand::Low)
        || matches!(
            h.degradation_class_bundle.capture_quality_class,
            CaptureQualityClass::Degraded
        )
    {
        return Ph1cSttStrategy::CloudAssist;
    }

    Ph1cSttStrategy::Standard
}

fn select_provider_ladder(strategy: Ph1cSttStrategy) -> [ProviderSlot; 2] {
    match strategy {
        Ph1cSttStrategy::Standard
        | Ph1cSttStrategy::NoiseRobust
        | Ph1cSttStrategy::CloudAssist
        | Ph1cSttStrategy::ClarifyOnly => [ProviderSlot::Primary, ProviderSlot::Secondary],
    }
}

fn is_language_mismatch(req: &Ph1cRequest, actual: &LanguageTag) -> bool {
    let Some(hint) = &req.language_hint else {
        return false;
    };

    // Only enforce mismatch when the hint is strong.
    if hint.confidence != LanguageHintConfidence::High {
        return false;
    }

    !locale_family_matches(hint.language_tag.as_str(), actual.as_str())
}

fn locale_family_matches(hint_tag: &str, actual_tag: &str) -> bool {
    let hint = parse_locale_components(hint_tag);
    let actual = parse_locale_components(actual_tag);

    if hint.normalized == actual.normalized {
        return true;
    }
    if hint.language.is_empty() || actual.language.is_empty() {
        return false;
    }
    if hint.language != actual.language {
        return false;
    }

    // Keep script-safe behavior: if both script subtags are present they must match.
    match (&hint.script, &actual.script) {
        (Some(lhs), Some(rhs)) => lhs == rhs,
        _ => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocaleComponents {
    normalized: String,
    language: String,
    script: Option<String>,
}

fn parse_locale_components(tag: &str) -> LocaleComponents {
    let normalized = normalize_locale_tag(tag);
    let mut language = String::new();
    let mut script: Option<String> = None;
    for (idx, part) in normalized.split('-').enumerate() {
        if part.is_empty() {
            continue;
        }
        if idx == 0 && part.chars().all(|c| c.is_ascii_alphabetic()) {
            language = part.to_string();
            continue;
        }
        if script.is_none() && part.len() == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
            script = Some(part.to_string());
        }
    }

    LocaleComponents {
        normalized,
        language,
        script,
    }
}

fn normalize_locale_tag(tag: &str) -> String {
    let mut out = String::with_capacity(tag.len());
    for c in tag.trim().chars() {
        if c == '_' {
            out.push('-');
        } else if c.is_ascii_alphanumeric() || c == '-' {
            for lc in c.to_lowercase() {
                out.push(lc);
            }
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

fn is_intent_repair_candidate(att: &SttAttempt, transcript: &str) -> bool {
    if transcript.is_empty() {
        return false;
    }
    if is_garbled_or_stutter(transcript) {
        return true;
    }

    let tokens = transcript.split_whitespace().count();
    if tokens < 3 || tokens > INTENT_REPAIR_MAX_TOKENS {
        return false;
    }

    let normalized = lowercase_text(transcript);
    let has_scramble_markers = [
        " uh ",
        " um ",
        " like ",
        " you know ",
        " sort of ",
        " kind of ",
        " pues ",
        " este ",
        " o sea ",
        " euh ",
        " genre ",
        " يعني ",
        " طيب ",
        " मतलब ",
        "那个",
        "就是",
    ]
    .iter()
    .any(|marker| normalized.contains(marker));
    has_scramble_markers
        || att.low_confidence_ratio >= 0.25
        || (att.avg_word_confidence < 0.82 && tokens >= 5)
}

fn collect_intent_hint_tokens(transcript: &str) -> Vec<String> {
    const HINTS: [&str; 52] = [
        "remind",
        "reminder",
        "meeting",
        "schedule",
        "book",
        "table",
        "send",
        "money",
        "weather",
        "time",
        "remember",
        "forget",
        "invoice",
        "payment",
        "link",
        "invite",
        "access",
        "schema",
        "capreq",
        "vote",
        "compile",
        "refresh",
        "tomorrow",
        "today",
        "recuerda",
        "recordatorio",
        "agenda",
        "programa",
        "clima",
        "factura",
        "pago",
        "rappelle",
        "rappel",
        "planifie",
        "meteo",
        "facture",
        "paiement",
        "提醒",
        "安排",
        "天气",
        "发票",
        "支付",
        "ذكرني",
        "جدول",
        "الطقس",
        "فاتورة",
        "دفع",
        "याद",
        "शेड्यूल",
        "मौसम",
        "चालान",
        "भुगतान",
    ];

    let normalized = normalize_lexicon_term(transcript);
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for hint in HINTS {
        if normalized.contains(hint) {
            out.push(hint.to_string());
            if out.len() >= INTENT_REPAIR_MAX_HINTS {
                break;
            }
        }
    }
    out
}

fn collapse_disfluencies_from_spans(spans: &[SrlFrameSpan]) -> String {
    if spans.is_empty() {
        return String::new();
    }

    let mut out: Vec<String> = Vec::new();
    let mut prev_normalized: Option<String> = None;
    let mut duplicate_run: usize = 0;
    for span in spans.iter().take(INTENT_REPAIR_MAX_TOKENS) {
        let token = span.normalized_text.trim();
        if token.is_empty() {
            continue;
        }
        let normalized = normalize_repair_token(token);
        if normalized.is_empty() {
            continue;
        }
        if is_filler_repair_token(&normalized) {
            continue;
        }

        if prev_normalized.as_deref() == Some(normalized.as_str()) {
            duplicate_run = duplicate_run.saturating_add(1);
            if duplicate_run >= 2 {
                continue;
            }
        } else {
            duplicate_run = 0;
            prev_normalized = Some(normalized.clone());
        }

        out.push(token.to_string());
    }

    out.join(" ")
}

fn normalize_repair_token(token: &str) -> String {
    let mut out = String::new();
    for c in token.chars().filter(|c| c.is_alphanumeric()) {
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out
}

fn is_filler_repair_token(token: &str) -> bool {
    matches!(
        token,
        "uh" | "um"
            | "erm"
            | "hmm"
            | "mmm"
            | "like"
            | "kinda"
            | "sorta"
            | "eh"
            | "pues"
            | "este"
            | "osea"
            | "euh"
            | "genre"
            | "يعني"
            | "طيب"
            | "मतलब"
            | "那个"
            | "就是"
    )
}

fn token_overlap_ratio(original: &str, repaired: &str) -> f32 {
    let orig_tokens = normalize_lexicon_term(original)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<std::collections::BTreeSet<_>>();
    let repaired_tokens = normalize_lexicon_term(repaired)
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<std::collections::BTreeSet<_>>();

    if orig_tokens.is_empty() || repaired_tokens.is_empty() {
        return 0.0;
    }
    let overlap = repaired_tokens
        .iter()
        .filter(|token| orig_tokens.contains(*token))
        .count();
    (overlap as f32 / repaired_tokens.len() as f32).clamp(0.0, 1.0)
}

fn nlp_repair_accept(
    req: &Ph1cRequest,
    original: &str,
    repaired: &str,
    language_tag: &LanguageTag,
) -> bool {
    let nlp = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
    let Some(original_req) = nlp_request_for_transcript(req, original, language_tag) else {
        return false;
    };
    let Some(repaired_req) = nlp_request_for_transcript(req, repaired, language_tag) else {
        return false;
    };

    let Ok(original_resp) = nlp.run(&original_req) else {
        return false;
    };
    let Ok(repaired_resp) = nlp.run(&repaired_req) else {
        return false;
    };

    let original_quality = nlp_response_quality(&original_resp);
    let repaired_quality = nlp_response_quality(&repaired_resp);
    if repaired_quality < original_quality {
        return false;
    }

    if let (Ph1nResponse::IntentDraft(a), Ph1nResponse::IntentDraft(b)) =
        (&original_resp, &repaired_resp)
    {
        if a.intent_type != b.intent_type {
            return false;
        }
    }

    if matches!(repaired_resp, Ph1nResponse::IntentDraft(_))
        && matches!(original_resp, Ph1nResponse::Chat(_))
        && !looks_actionable(repaired)
    {
        return false;
    }

    true
}

fn nlp_request_for_transcript(
    req: &Ph1cRequest,
    transcript: &str,
    language_tag: &LanguageTag,
) -> Option<Ph1nRequest> {
    let transcript_ok = TranscriptOk::v1(
        transcript.to_string(),
        language_tag.clone(),
        if looks_actionable(transcript) {
            ConfidenceBucket::High
        } else {
            ConfidenceBucket::Med
        },
    )
    .ok()?;
    let mut n_req = Ph1nRequest::v1(
        transcript_ok,
        SessionStateRef::v1(
            if req.session_state_ref.session_state == SessionState::Suspended {
                SessionState::Active
            } else {
                req.session_state_ref.session_state
            },
            req.session_state_ref.tts_playback_active,
        ),
    )
    .ok()?
    .with_runtime_tenant_id(None)
    .ok()?;
    if transcript.len() > 220 {
        n_req.confirmed_context = None;
    }
    Some(n_req)
}

fn nlp_response_quality(resp: &Ph1nResponse) -> u8 {
    match resp {
        Ph1nResponse::IntentDraft(draft) => match draft.overall_confidence {
            OverallConfidence::High => 3,
            OverallConfidence::Med => 2,
            OverallConfidence::Low => 1,
        },
        Ph1nResponse::Clarify(_) => 2,
        Ph1nResponse::Chat(_) => 1,
    }
}

fn semantic_quality_ok(
    req: &Ph1cRequest,
    transcript: &str,
    language_tag: &LanguageTag,
    min_quality: u8,
) -> bool {
    let threshold = min_quality.max(1);
    let Some(n_req) = nlp_request_for_transcript(req, transcript, language_tag) else {
        return false;
    };
    let nlp = Ph1nRuntime::new(Ph1nConfig::mvp_v1());
    let Ok(resp) = nlp.run(&n_req) else {
        return false;
    };
    nlp_response_quality(&resp) >= threshold
}

fn overlap_hint_requires_disambiguation(req: &Ph1cRequest, transcript: &str) -> bool {
    let Some(overlap_hint) = &req.speaker_overlap_hint else {
        return false;
    };
    let overlap_confidence = overlap_hint.confidence.0;
    if overlap_confidence < 0.7 {
        return false;
    }
    match overlap_hint.overlap_class {
        selene_kernel_contracts::ph1c::SpeakerOverlapClass::SingleSpeaker => false,
        selene_kernel_contracts::ph1c::SpeakerOverlapClass::Unknown => false,
        selene_kernel_contracts::ph1c::SpeakerOverlapClass::MultiSpeaker
        | selene_kernel_contracts::ph1c::SpeakerOverlapClass::InterruptionOverlap => {
            likely_overlap_ambiguous(transcript)
        }
    }
}

fn likely_overlap_ambiguous(transcript: &str) -> bool {
    let lower = lowercase_text(transcript);
    [
        "speaker 1",
        "speaker 2",
        "both talking",
        "overlap",
        "i said",
        "dos personas hablan",
        "deux personnes parlent",
        "两个人",
        "同时说",
        "يتحدثان",
        "दो लोग",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
        || lower.matches(':').count() >= 2
}

fn looks_actionable(transcript: &str) -> bool {
    let lower = lowercase_text(transcript);
    let actionable_markers = [
        "remind",
        "set",
        "schedule",
        "book",
        "send money",
        "remember",
        "forget",
        "time",
        "weather",
        "invite",
        "access",
        "capreq",
        "recuerda",
        "programa",
        "agenda",
        "envia",
        "clima",
        "rappelle",
        "planifie",
        "reserve",
        "meteo",
        "提醒",
        "安排",
        "预订",
        "天气",
        "ذكرني",
        "جدول",
        "احجز",
        "الطقس",
        "याद",
        "शेड्यूल",
        "बुक",
        "मौसम",
    ];
    actionable_markers
        .iter()
        .any(|marker| lower.contains(marker))
}

#[derive(Debug, Clone, Copy)]
struct CalibratedConfidence {
    score: f32,
    bucket: ConfidenceBucket,
}

fn calibrated_confidence(req: &Ph1cRequest, att: &SttAttempt) -> CalibratedConfidence {
    let token_score = token_confidence_score(att.avg_word_confidence, att.low_confidence_ratio);
    let acoustic_score = acoustic_confidence_score(req);
    let context_score = context_confidence_score(req, &att.language_tag);
    // Weighted calibrated score; missing acoustic/context fall back to token confidence.
    let score = clamp01(
        (token_score * 0.70)
            + (if has_acoustic_signals(req) {
                acoustic_score
            } else {
                token_score
            } * 0.20)
            + (if has_context_signals(req) {
                context_score
            } else {
                token_score
            } * 0.10),
    );
    let bucket = if score >= 0.87 {
        ConfidenceBucket::High
    } else if score >= 0.74 {
        ConfidenceBucket::Med
    } else {
        ConfidenceBucket::Low
    };
    CalibratedConfidence { score, bucket }
}

fn bucket_rank(b: ConfidenceBucket) -> u8 {
    use ConfidenceBucket::*;
    match b {
        Low => 0,
        Med => 1,
        High => 2,
    }
}

fn confidence_ok(cfg: &Ph1cConfig, att: &SttAttempt, calibrated_score: f32) -> bool {
    if !(att.avg_word_confidence.is_finite() && att.low_confidence_ratio.is_finite()) {
        return false;
    }
    if !(0.0..=1.0).contains(&att.avg_word_confidence) {
        return false;
    }
    if !(0.0..=1.0).contains(&att.low_confidence_ratio) {
        return false;
    }
    if cfg.require_stable && !att.stable {
        return false;
    }
    if !(0.0..=1.0).contains(&calibrated_score) {
        return false;
    }
    if calibrated_score < min_calibrated_score_for_bucket(cfg.min_confidence_bucket_to_pass) {
        return false;
    }
    att.avg_word_confidence >= cfg.min_avg_word_confidence
        && att.low_confidence_ratio <= cfg.max_low_confidence_ratio
}

fn token_confidence_score(avg_word_conf: f32, low_ratio: f32) -> f32 {
    let avg = clamp01(avg_word_conf);
    let low = clamp01(low_ratio);
    clamp01(avg * (1.0 - (low * 0.75)))
}

fn acoustic_confidence_score(req: &Ph1cRequest) -> f32 {
    let mut weighted_sum = 0.0f32;
    let mut weight = 0.0f32;

    if let Some(noise_hint) = req.noise_level_hint {
        weighted_sum += clamp01(1.0 - noise_hint.0) * 0.20;
        weight += 0.20;
    }
    if let Some(vad_hint) = req.vad_quality_hint {
        weighted_sum += clamp01(vad_hint.0) * 0.20;
        weight += 0.20;
    }
    if let Some(handoff) = &req.ph1k_handoff {
        let m = handoff.quality_metrics;
        let snr_score = linear_score(m.snr_db, 6.0, 30.0);
        let packet_score = linear_score(20.0 - m.packet_loss_pct, 0.0, 20.0);
        let clipping_score = linear_score(0.35 - m.clipping_ratio, 0.0, 0.35);
        let double_talk_score = linear_score(1.0 - m.double_talk_score, 0.0, 1.0);
        let echo_score = linear_score(400.0 - m.echo_delay_ms, 0.0, 400.0);
        let erle_score = linear_score(m.erle_db, 6.0, 24.0);
        let handoff_score = clamp01(
            (snr_score * 0.28)
                + (packet_score * 0.24)
                + (clipping_score * 0.14)
                + (double_talk_score * 0.12)
                + (echo_score * 0.12)
                + (erle_score * 0.10),
        );
        weighted_sum += handoff_score * 0.60;
        weight += 0.60;
    }

    if weight <= 0.0 {
        0.70
    } else {
        clamp01(weighted_sum / weight)
    }
}

fn has_acoustic_signals(req: &Ph1cRequest) -> bool {
    req.noise_level_hint.is_some() || req.vad_quality_hint.is_some() || req.ph1k_handoff.is_some()
}

fn context_confidence_score(req: &Ph1cRequest, actual_language_tag: &LanguageTag) -> f32 {
    let language_score = req
        .language_hint
        .as_ref()
        .map(|hint| {
            let family_match =
                locale_family_matches(hint.language_tag.as_str(), actual_language_tag.as_str());
            match hint.confidence {
                LanguageHintConfidence::High => {
                    if family_match {
                        1.0
                    } else {
                        0.0
                    }
                }
                LanguageHintConfidence::Med => {
                    if family_match {
                        0.85
                    } else {
                        0.25
                    }
                }
                LanguageHintConfidence::Low => {
                    if family_match {
                        0.70
                    } else {
                        0.45
                    }
                }
            }
        })
        .unwrap_or(0.60);

    let vad_bias = req
        .vad_quality_hint
        .map(|v| (clamp01(v.0) * 0.30) + 0.70)
        .unwrap_or(0.85);
    clamp01((language_score * 0.85) + (vad_bias * 0.15))
}

fn has_context_signals(req: &Ph1cRequest) -> bool {
    req.language_hint.is_some() || req.vad_quality_hint.is_some()
}

fn min_calibrated_score_for_bucket(bucket: ConfidenceBucket) -> f32 {
    match bucket {
        ConfidenceBucket::High => 0.87,
        ConfidenceBucket::Med => 0.74,
        ConfidenceBucket::Low => 0.0,
    }
}

fn linear_score(value: f32, min: f32, max: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    if max <= min {
        return 0.0;
    }
    clamp01((value - min) / (max - min))
}

fn clamp01(value: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, 1.0)
}

fn coverage_ok(cfg: &Ph1cConfig, req: &Ph1cRequest, transcript: &str) -> bool {
    if transcript.chars().count() < cfg.min_chars_absolute {
        return false;
    }

    // Use the bounded segment duration as a conservative proxy for expected content length.
    let dur_ns = req
        .bounded_audio_segment_ref
        .t_end
        .0
        .saturating_sub(req.bounded_audio_segment_ref.t_start.0);
    let dur_s = (dur_ns as f32) / 1_000_000_000.0;
    if dur_s <= 0.0 {
        return false;
    }

    let min_chars = (cfg.min_chars_per_second * dur_s).ceil() as usize;
    transcript.chars().count() >= min_chars
}

fn is_garbled_or_stutter(transcript: &str) -> bool {
    // Detect extreme repetition ("I I I I"), which can appear as stutter or duplicate garbage.
    let tokens: Vec<&str> = transcript.split_whitespace().collect();
    if tokens.len() >= 4 {
        let mut run = 1usize;
        for i in 1..tokens.len() {
            if normalize_repeat_token(tokens[i]) == normalize_repeat_token(tokens[i - 1]) {
                run += 1;
                if run >= 4 {
                    return true;
                }
            } else {
                run = 1;
            }
        }
    }

    // Provider "unknown" token patterns (keep conservative).
    let lower = lowercase_text(transcript);
    lower.contains("<unk>") || lower.contains("[unk]") || lower.contains("???")
}

fn normalize_repeat_token(token: &str) -> String {
    let mut out = String::new();
    for c in token.chars() {
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out
}

fn lowercase_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        for lc in c.to_lowercase() {
            out.push(lc);
        }
    }
    out
}

fn partial_reject(reason_code: ReasonCodeId) -> TranscriptReject {
    TranscriptReject::v1(reason_code, RetryAdvice::Repeat)
}

fn prefer_partial_candidate(existing: &PartialTranscript, candidate: &PartialTranscript) -> bool {
    let existing_rank = partial_rank(existing);
    let candidate_rank = partial_rank(candidate);
    candidate_rank > existing_rank
}

fn partial_rank(partial: &PartialTranscript) -> (u8, u16, &str) {
    let stability_rank = if partial.stable { 1 } else { 0 };
    let confidence_bp = (partial.confidence.0 * 10_000.0).round() as u16;
    (stability_rank, confidence_bp, partial.text_chunk.as_str())
}

fn select_more_specific_failure(prev: Option<ReasonCodeId>, next: ReasonCodeId) -> ReasonCodeId {
    // Deterministic priority order: pick the "strongest" known failure to explain upstream.
    fn rank(rc: ReasonCodeId) -> u8 {
        match rc {
            reason_codes::STT_FAIL_AUDIO_DEGRADED => 0,
            reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN => 1,
            reason_codes::STT_FAIL_PROVIDER_DISAGREEMENT => 2,
            reason_codes::STT_FAIL_SPEAKER_OVERLAP_AMBIGUOUS => 3,
            reason_codes::STT_FAIL_LOW_SEMANTIC_CONFIDENCE => 4,
            reason_codes::STT_FAIL_BUDGET_EXCEEDED => 5,
            reason_codes::STT_FAIL_LANGUAGE_MISMATCH => 6,
            reason_codes::STT_FAIL_LOW_COVERAGE => 7,
            reason_codes::STT_FAIL_LOW_CONFIDENCE => 8,
            reason_codes::STT_FAIL_GARBLED => 9,
            reason_codes::STT_FAIL_EMPTY => 10,
            reason_codes::STT_FAIL_PARTIAL_ORDER => 11,
            reason_codes::STT_FAIL_PARTIAL_INVALID => 12,
            reason_codes::STT_FAIL_PROVIDER_TIMEOUT => 99,
            _ => 100,
        }
    }

    match prev {
        None => next,
        Some(p) => {
            if rank(next) <= rank(p) {
                next
            } else {
                p
            }
        }
    }
}

fn retry_advice_for(reason: ReasonCodeId) -> RetryAdvice {
    match reason {
        reason_codes::STT_FAIL_EMPTY => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_LOW_COVERAGE => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_LOW_CONFIDENCE => RetryAdvice::SpeakSlower,
        reason_codes::STT_FAIL_GARBLED => RetryAdvice::QuietEnv,
        reason_codes::STT_FAIL_LANGUAGE_MISMATCH => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_AUDIO_DEGRADED => RetryAdvice::MoveCloser,
        reason_codes::STT_FAIL_BUDGET_EXCEEDED => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_QUOTA_THROTTLED => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_POLICY_RESTRICTED => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_PROVIDER_CIRCUIT_OPEN => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_NETWORK_UNAVAILABLE => RetryAdvice::SwitchToText,
        reason_codes::STT_FAIL_PROVIDER_DISAGREEMENT => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_SPEAKER_OVERLAP_AMBIGUOUS => RetryAdvice::Repeat,
        reason_codes::STT_FAIL_LOW_SEMANTIC_CONFIDENCE => RetryAdvice::Repeat,
        _ => RetryAdvice::Repeat,
    }
}

fn selected_slot_for_provider(provider: ProviderSlot) -> SelectedSlot {
    match provider {
        ProviderSlot::Primary => SelectedSlot::Primary,
        ProviderSlot::Secondary => SelectedSlot::Secondary,
        ProviderSlot::Tertiary => SelectedSlot::Tertiary,
    }
}

fn transcript_ok_with_audit(
    cfg: &Ph1cConfig,
    strategy: Ph1cSttStrategy,
    out: &TranscriptOk,
    selected_slot: SelectedSlot,
    attempt_count: u8,
    candidate_count: u8,
    total_latency_ms: u32,
    second_pass_used: bool,
) -> Ph1cResponse {
    let audit_meta = match build_audit_meta(
        cfg,
        attempt_count,
        candidate_count,
        selected_slot,
        total_latency_ms,
        quality_bucket_from_confidence(out.confidence_bucket),
        quality_bucket_from_confidence(out.confidence_bucket),
        QualityBucket::High,
        None,
        None,
        Some(strategy_policy_profile_id(strategy).to_string()),
        Some("openai_google_clarify_v1".to_string()),
        second_pass_used,
    ) {
        Ok(meta) => meta,
        Err(_) => {
            return Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                reason_codes::STT_FAIL_POLICY_RESTRICTED,
                RetryAdvice::Repeat,
            ));
        }
    };
    match TranscriptOk::v1_with_metadata(
        out.transcript_text.clone(),
        out.language_tag.clone(),
        out.confidence_bucket,
        out.uncertain_spans.clone(),
        Some(audit_meta),
    ) {
        Ok(ok) => Ph1cResponse::TranscriptOk(ok),
        Err(_) => Ph1cResponse::TranscriptReject(TranscriptReject::v1(
            reason_codes::STT_FAIL_POLICY_RESTRICTED,
            RetryAdvice::Repeat,
        )),
    }
}

fn choose_cost_quality_slot_for_pair(
    primary: &SttAttempt,
    secondary: &SttAttempt,
    live: &Ph1cLiveProviderContext,
    confidence_tolerance_bp: u16,
) -> ProviderSlot {
    if !live.enable_cost_quality_routing {
        return ProviderSlot::Primary;
    }

    let primary_conf_bp = (clamp01(primary.avg_word_confidence) * 10_000.0).round() as i32;
    let secondary_conf_bp = (clamp01(secondary.avg_word_confidence) * 10_000.0).round() as i32;
    let tolerance_bp = i32::from(confidence_tolerance_bp);
    let secondary_within_tolerance = secondary_conf_bp + tolerance_bp >= primary_conf_bp;

    if secondary_within_tolerance && live.secondary_cost_microunits <= live.primary_cost_microunits
    {
        ProviderSlot::Secondary
    } else {
        ProviderSlot::Primary
    }
}

fn transcript_divergence_basis_points(a: &str, b: &str) -> u16 {
    let overlap = token_overlap_ratio(a, b);
    10_000u16.saturating_sub(overlap_basis_points(overlap))
}

fn quality_bucket_from_confidence(c: ConfidenceBucket) -> QualityBucket {
    match c {
        ConfidenceBucket::High => QualityBucket::High,
        ConfidenceBucket::Med => QualityBucket::Med,
        ConfidenceBucket::Low => QualityBucket::Low,
    }
}

#[allow(clippy::too_many_arguments)]
fn build_audit_meta(
    cfg: &Ph1cConfig,
    attempt_count: u8,
    candidate_count: u8,
    selected_slot: SelectedSlot,
    total_latency_ms: u32,
    quality_coverage_bucket: QualityBucket,
    quality_confidence_bucket: QualityBucket,
    quality_plausibility_bucket: QualityBucket,
    tenant_vocabulary_pack_id: Option<String>,
    user_vocabulary_pack_id: Option<String>,
    policy_profile_id: Option<String>,
    stt_routing_policy_pack_id: Option<String>,
    second_pass_used: bool,
) -> Result<Ph1cAuditMeta, ContractViolationLocal> {
    Ph1cAuditMeta::v1(
        RouteClassUsed::OnDevice,
        attempt_count,
        candidate_count,
        selected_slot,
        cfg.routing_mode_used,
        second_pass_used,
        total_latency_ms,
        quality_coverage_bucket,
        quality_confidence_bucket,
        quality_plausibility_bucket,
        tenant_vocabulary_pack_id,
        user_vocabulary_pack_id,
        policy_profile_id,
        stt_routing_policy_pack_id,
    )
    .map_err(|_| ContractViolationLocal::InvalidValue("ph1c_runtime.audit_meta"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{
        LanguageHint, Ph1kToPh1cHandoff, SessionStateRef, SpeakerOverlapClass, SpeakerOverlapHint,
    };
    use selene_kernel_contracts::ph1d::{
        Ph1dProviderRouteClass, PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
    };
    use selene_kernel_contracts::ph1k::{
        AdvancedAudioQualityMetrics, AudioDeviceId, AudioStreamId, CaptureQualityClass,
        DegradationClassBundle, DeviceHealth, DeviceState, EchoRiskClass,
        InterruptCandidateConfidenceBand, NetworkStabilityClass, PreRollBufferId,
        RecoverabilityClass, VadDecisionConfidenceBand,
    };
    use selene_kernel_contracts::ph1w::BoundedAudioSegmentRef;
    use selene_kernel_contracts::ph1w::SessionState;
    use selene_kernel_contracts::MonotonicTimeNs;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    #[derive(Debug, Clone)]
    enum AdapterAction {
        OkStt {
            text: &'static str,
            language: &'static str,
            confidence_bp: u16,
            stable: bool,
            latency_ms: u32,
        },
        OkSttStream {
            text: &'static str,
            language: &'static str,
            confidence_bp: u16,
            stable: bool,
            latency_ms: u32,
            revision_id: u32,
            finalized: bool,
        },
        ProviderTimeout,
        ProviderContractMismatch,
    }

    #[derive(Debug)]
    struct ScriptedAdapter {
        actions: Mutex<VecDeque<AdapterAction>>,
        seen_provider_ids: Mutex<Vec<String>>,
        seen_routes: Mutex<Vec<Ph1dProviderRouteClass>>,
        seen_input_payload_inline: Mutex<Vec<Option<String>>>,
    }

    impl ScriptedAdapter {
        fn new(actions: Vec<AdapterAction>) -> Self {
            Self {
                actions: Mutex::new(actions.into()),
                seen_provider_ids: Mutex::new(Vec::new()),
                seen_routes: Mutex::new(Vec::new()),
                seen_input_payload_inline: Mutex::new(Vec::new()),
            }
        }

        fn seen_provider_ids(&self) -> Vec<String> {
            self.seen_provider_ids
                .lock()
                .expect("mutex poisoned")
                .clone()
        }

        fn seen_routes(&self) -> Vec<Ph1dProviderRouteClass> {
            self.seen_routes.lock().expect("mutex poisoned").clone()
        }

        fn seen_input_payload_inline(&self) -> Vec<Option<String>> {
            self.seen_input_payload_inline
                .lock()
                .expect("mutex poisoned")
                .clone()
        }
    }

    impl Ph1dProviderAdapter for ScriptedAdapter {
        fn execute(
            &self,
            req: &Ph1dProviderCallRequest,
        ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError> {
            self.seen_provider_ids
                .lock()
                .expect("mutex poisoned")
                .push(req.provider_id.clone());
            self.seen_routes
                .lock()
                .expect("mutex poisoned")
                .push(req.provider_route_class);
            self.seen_input_payload_inline
                .lock()
                .expect("mutex poisoned")
                .push(req.input_payload_inline.clone());

            let action = self
                .actions
                .lock()
                .expect("mutex poisoned")
                .pop_front()
                .expect("scripted adapter ran out of actions");
            match action {
                AdapterAction::OkStt {
                    text,
                    language,
                    confidence_bp,
                    stable,
                    latency_ms,
                } => {
                    let normalized = serde_json::json!({
                        "schema_version": 1,
                        "provider_task": Ph1dProviderTask::SttTranscribe.as_str(),
                        "text_output": text,
                        "language_tag": language,
                        "confidence_bp": confidence_bp,
                        "stable": stable,
                        "audio_output_ref": serde_json::Value::Null,
                        "audio_content_type": serde_json::Value::Null,
                        "estimated_duration_ms": serde_json::Value::Null,
                    });
                    let normalized_json = serde_json::to_string(&normalized)
                        .expect("normalized output serialization must succeed");
                    Ph1dProviderCallResponse::v1(
                        req.correlation_id,
                        req.turn_id,
                        req.request_id,
                        req.idempotency_key.clone(),
                        Some("prov_call_1".to_string()),
                        req.provider_id.clone(),
                        req.provider_task,
                        req.model_id.clone(),
                        Ph1dProviderStatus::Ok,
                        latency_ms,
                        0,
                        Some(confidence_bp),
                        Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1),
                        Some(normalized_json),
                        Ph1dProviderValidationStatus::SchemaOk,
                        d_reason_codes::D_PROVIDER_OK,
                    )
                    .map_err(|e| {
                        Ph1dProviderAdapterError::terminal(format!(
                            "scripted adapter ok response build failed: {e:?}"
                        ))
                    })
                }
                AdapterAction::OkSttStream {
                    text,
                    language,
                    confidence_bp,
                    stable,
                    latency_ms,
                    revision_id,
                    finalized,
                } => {
                    let normalized = serde_json::json!({
                        "schema_version": 1,
                        "provider_task": Ph1dProviderTask::SttTranscribe.as_str(),
                        "text_output": text,
                        "language_tag": language,
                        "confidence_bp": confidence_bp,
                        "stable": stable,
                        "audio_output_ref": serde_json::Value::Null,
                        "audio_content_type": serde_json::Value::Null,
                        "estimated_duration_ms": serde_json::Value::Null,
                        "revision_id": revision_id,
                        "finalized": finalized,
                    });
                    let normalized_json = serde_json::to_string(&normalized)
                        .expect("normalized stream output serialization must succeed");
                    Ph1dProviderCallResponse::v1(
                        req.correlation_id,
                        req.turn_id,
                        req.request_id,
                        req.idempotency_key.clone(),
                        Some("prov_call_stream".to_string()),
                        req.provider_id.clone(),
                        req.provider_task,
                        req.model_id.clone(),
                        Ph1dProviderStatus::Ok,
                        latency_ms,
                        0,
                        Some(confidence_bp),
                        Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1),
                        Some(normalized_json),
                        Ph1dProviderValidationStatus::SchemaOk,
                        d_reason_codes::D_PROVIDER_OK,
                    )
                    .map_err(|e| {
                        Ph1dProviderAdapterError::terminal(format!(
                            "scripted adapter stream response build failed: {e:?}"
                        ))
                    })
                }
                AdapterAction::ProviderTimeout => Ph1dProviderCallResponse::v1(
                    req.correlation_id,
                    req.turn_id,
                    req.request_id,
                    req.idempotency_key.clone(),
                    Some("prov_call_timeout".to_string()),
                    req.provider_id.clone(),
                    req.provider_task,
                    req.model_id.clone(),
                    Ph1dProviderStatus::Error,
                    req.timeout_ms,
                    0,
                    None,
                    None,
                    None,
                    Ph1dProviderValidationStatus::SchemaFail,
                    d_reason_codes::D_PROVIDER_TIMEOUT,
                )
                .map_err(|e| {
                    Ph1dProviderAdapterError::terminal(format!(
                        "scripted adapter timeout response build failed: {e:?}"
                    ))
                }),
                AdapterAction::ProviderContractMismatch => Ph1dProviderCallResponse::v1(
                    req.correlation_id,
                    req.turn_id,
                    req.request_id,
                    req.idempotency_key.clone(),
                    Some("prov_call_mismatch".to_string()),
                    req.provider_id.clone(),
                    req.provider_task,
                    req.model_id.clone(),
                    Ph1dProviderStatus::Error,
                    req.timeout_ms,
                    0,
                    None,
                    None,
                    None,
                    Ph1dProviderValidationStatus::SchemaFail,
                    d_reason_codes::D_PROVIDER_CONTRACT_MISMATCH,
                )
                .map_err(|e| {
                    Ph1dProviderAdapterError::terminal(format!(
                        "scripted adapter mismatch response build failed: {e:?}"
                    ))
                }),
            }
        }
    }

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    fn seg(duration_ms: u64) -> BoundedAudioSegmentRef {
        BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(0),
            MonotonicTimeNs(duration_ms * 1_000_000),
            MonotonicTimeNs(0),
            MonotonicTimeNs(0),
        )
        .unwrap()
    }

    fn req_with_duration(duration_ms: u64) -> Ph1cRequest {
        Ph1cRequest::v1(
            seg(duration_ms),
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn handoff(
        interrupt_band: InterruptCandidateConfidenceBand,
        vad_band: VadDecisionConfidenceBand,
        snr_db: f32,
        packet_loss_pct: f32,
        degradation_class_bundle: DegradationClassBundle,
    ) -> Ph1kToPh1cHandoff {
        Ph1kToPh1cHandoff::v1(
            interrupt_band,
            vad_band,
            AdvancedAudioQualityMetrics::v1(snr_db, 0.03, 40.0, packet_loss_pct, 0.15, 16.0)
                .unwrap(),
            degradation_class_bundle,
        )
        .unwrap()
    }

    #[test]
    fn at_c_5h_step2_live_ph1d_primary_success_consumed_directly() {
        let req = req_with_duration(900);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2001, 3001, "tenant_a".to_string());
        live.enforce_provider_disagreement_clarify = false;
        let adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "openai primary transcript",
            language: "en",
            confidence_bp: 9_700,
            stable: true,
            latency_ms: 88,
        }]);

        match rt.run_via_live_provider_adapter(&req, &live, &adapter) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "openai primary transcript");
                let audit_meta = ok.audit_meta.expect("audit metadata must be present");
                assert_eq!(audit_meta.selected_slot, SelectedSlot::Primary);
                assert!(!audit_meta.second_pass_used);
            }
            other => panic!("expected transcript_ok from live primary provider, got: {other:?}"),
        }

        assert_eq!(adapter.seen_provider_ids(), vec!["openai".to_string()]);
        assert_eq!(adapter.seen_routes(), vec![Ph1dProviderRouteClass::Primary]);
    }

    #[test]
    fn at_c_5h_step2_live_ph1d_secondary_fallback_after_primary_timeout() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::new(cfg);
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2002, 3002, "tenant_a".to_string());
        live.retry_budget = 0;

        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::ProviderTimeout,
            AdapterAction::OkStt {
                text: "google secondary transcript",
                language: "en",
                confidence_bp: 9_500,
                stable: true,
                latency_ms: 110,
            },
        ]);

        match rt.run_via_live_provider_adapter(&req, &live, &adapter) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "google secondary transcript");
                let audit_meta = ok.audit_meta.expect("audit metadata must be present");
                assert_eq!(audit_meta.selected_slot, SelectedSlot::Secondary);
            }
            other => panic!("expected transcript_ok from secondary fallback, got: {other:?}"),
        }

        assert_eq!(
            adapter.seen_provider_ids(),
            vec!["openai".to_string(), "google".to_string()]
        );
        assert_eq!(
            adapter.seen_routes(),
            vec![
                Ph1dProviderRouteClass::Primary,
                Ph1dProviderRouteClass::Secondary
            ]
        );
    }

    #[test]
    fn at_c_5h_step2_live_ph1d_both_slots_fail_closed() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::new(cfg);
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2003, 3003, "tenant_a".to_string());
        live.retry_budget = 0;

        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::ProviderTimeout,
            AdapterAction::ProviderContractMismatch,
        ]);

        match rt.run_via_live_provider_adapter(&req, &live, &adapter) {
            Ph1cResponse::TranscriptReject(reject) => {
                assert_eq!(reject.reason_code, reason_codes::STT_FAIL_PROVIDER_TIMEOUT);
                assert_eq!(reject.retry_advice, RetryAdvice::Repeat);
            }
            other => panic!("expected fail-closed transcript_reject, got: {other:?}"),
        }

        assert_eq!(
            adapter.seen_provider_ids(),
            vec!["openai".to_string(), "google".to_string()]
        );
    }

    #[test]
    fn at_c_5h_step3_primary_circuit_open_skips_to_google_secondary() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::with_circuit_breaker_config(
            cfg,
            Ph1cCircuitBreakerConfig {
                failure_threshold: 1,
                cooldown_ms: 5_000,
            },
        )
        .expect("valid breaker config");
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2101, 3101, "tenant_a".to_string());
        live.retry_budget = 0;

        let first_adapter = ScriptedAdapter::new(vec![
            AdapterAction::ProviderTimeout,
            AdapterAction::OkStt {
                text: "fallback after timeout",
                language: "en",
                confidence_bp: 9_600,
                stable: true,
                latency_ms: 90,
            },
        ]);
        let first = rt.run_via_live_provider_adapter_at_ms(&req, &live, &first_adapter, 10_000);
        assert!(matches!(first, Ph1cResponse::TranscriptOk(_)));
        assert_eq!(
            first_adapter.seen_provider_ids(),
            vec!["openai".to_string(), "google".to_string()]
        );

        let second_adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "secondary only due breaker",
            language: "en",
            confidence_bp: 9_550,
            stable: true,
            latency_ms: 95,
        }]);
        let second = rt.run_via_live_provider_adapter_at_ms(&req, &live, &second_adapter, 11_000);
        match second {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "secondary only due breaker")
            }
            other => panic!("expected fallback transcript during breaker cooldown, got: {other:?}"),
        }
        assert_eq!(
            second_adapter.seen_provider_ids(),
            vec!["google".to_string()]
        );
        assert_eq!(
            second_adapter.seen_routes(),
            vec![Ph1dProviderRouteClass::Secondary]
        );
    }

    #[test]
    fn at_c_5h_step3_primary_circuit_cooldown_reopens_primary() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::with_circuit_breaker_config(
            cfg,
            Ph1cCircuitBreakerConfig {
                failure_threshold: 1,
                cooldown_ms: 1_000,
            },
        )
        .expect("valid breaker config");
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2102, 3102, "tenant_a".to_string());
        live.retry_budget = 0;
        live.enforce_provider_disagreement_clarify = false;

        let first_adapter = ScriptedAdapter::new(vec![
            AdapterAction::ProviderTimeout,
            AdapterAction::OkStt {
                text: "fallback path",
                language: "en",
                confidence_bp: 9_500,
                stable: true,
                latency_ms: 85,
            },
        ]);
        let _ = rt.run_via_live_provider_adapter_at_ms(&req, &live, &first_adapter, 20_000);

        let second_adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "secondary while open",
            language: "en",
            confidence_bp: 9_400,
            stable: true,
            latency_ms: 88,
        }]);
        let _ = rt.run_via_live_provider_adapter_at_ms(&req, &live, &second_adapter, 20_500);
        assert_eq!(
            second_adapter.seen_provider_ids(),
            vec!["google".to_string()]
        );

        let third_adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "primary returns after cooldown",
            language: "en",
            confidence_bp: 9_700,
            stable: true,
            latency_ms: 80,
        }]);
        let third = rt.run_via_live_provider_adapter_at_ms(&req, &live, &third_adapter, 21_100);
        match third {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "primary returns after cooldown")
            }
            other => panic!("expected primary retry after cooldown, got: {other:?}"),
        }
        assert_eq!(
            third_adapter.seen_provider_ids(),
            vec!["openai".to_string()]
        );
        assert_eq!(
            third_adapter.seen_routes(),
            vec![Ph1dProviderRouteClass::Primary]
        );
    }

    #[test]
    fn at_c_5h_step3_circuit_scope_is_tenant_provider_model_specific() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::with_circuit_breaker_config(
            cfg,
            Ph1cCircuitBreakerConfig {
                failure_threshold: 1,
                cooldown_ms: 5_000,
            },
        )
        .expect("valid breaker config");

        let mut live_tenant_a =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2103, 3103, "tenant_a".to_string());
        live_tenant_a.retry_budget = 0;
        live_tenant_a.enforce_provider_disagreement_clarify = false;
        let mut live_tenant_b =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2104, 3104, "tenant_b".to_string());
        live_tenant_b.retry_budget = 0;
        live_tenant_b.enforce_provider_disagreement_clarify = false;

        let a_adapter = ScriptedAdapter::new(vec![
            AdapterAction::ProviderTimeout,
            AdapterAction::OkStt {
                text: "tenant a fallback",
                language: "en",
                confidence_bp: 9_500,
                stable: true,
                latency_ms: 90,
            },
        ]);
        let _ = rt.run_via_live_provider_adapter_at_ms(&req, &live_tenant_a, &a_adapter, 30_000);
        assert_eq!(
            a_adapter.seen_provider_ids(),
            vec!["openai".to_string(), "google".to_string()]
        );

        let b_adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "tenant b primary unaffected",
            language: "en",
            confidence_bp: 9_650,
            stable: true,
            latency_ms: 87,
        }]);
        let b_result =
            rt.run_via_live_provider_adapter_at_ms(&req, &live_tenant_b, &b_adapter, 30_100);
        assert!(matches!(b_result, Ph1cResponse::TranscriptOk(_)));
        assert_eq!(b_adapter.seen_provider_ids(), vec!["openai".to_string()]);
    }

    #[test]
    fn at_c_5h_step4_streaming_low_latency_commit_before_finalization() {
        let req = req_with_duration(1200);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        cfg.stream_max_revisions = 4;
        cfg.stream_low_latency_confidence_min = 0.90;
        cfg.stream_low_latency_min_chars = 10;
        let rt = Ph1cRuntime::new(cfg);
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2201, 3201, "tenant_a".to_string());
        live.retry_budget = 0;

        let adapter = ScriptedAdapter::new(vec![AdapterAction::OkSttStream {
            text: "set reminder for tomorrow",
            language: "en",
            confidence_bp: 9_700,
            stable: true,
            latency_ms: 78,
            revision_id: 1,
            finalized: false,
        }]);
        let commit = rt.run_stream_via_live_provider_adapter_at_ms(&req, &live, &adapter, 40_000);
        assert!(commit.low_latency_commit);
        assert!(!commit.finalized);
        let batch = commit
            .partial_batch
            .expect("streaming commit should retain partial batch");
        assert_eq!(batch.partials.len(), 1);
        assert!(!batch.finalized);
        match commit.response {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set reminder for tomorrow")
            }
            other => panic!("expected transcript_ok low-latency commit, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_5h_step4_streaming_revision_replacement_and_finalization_commit() {
        let req = req_with_duration(1600);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        cfg.stream_max_revisions = 6;
        cfg.stream_low_latency_confidence_min = 0.995;
        cfg.stream_low_latency_min_chars = 20;
        let rt = Ph1cRuntime::new(cfg);
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2202, 3202, "tenant_a".to_string());
        live.retry_budget = 0;

        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::OkSttStream {
                text: "set reminder",
                language: "en",
                confidence_bp: 9_100,
                stable: false,
                latency_ms: 60,
                revision_id: 1,
                finalized: false,
            },
            AdapterAction::OkSttStream {
                text: "set reminder for tomorrow",
                language: "en",
                confidence_bp: 9_300,
                stable: true,
                latency_ms: 61,
                revision_id: 2,
                finalized: false,
            },
            AdapterAction::OkSttStream {
                text: "set reminder for tomorrow at noon",
                language: "en",
                confidence_bp: 9_600,
                stable: true,
                latency_ms: 62,
                revision_id: 3,
                finalized: true,
            },
        ]);
        let commit = rt.run_stream_via_live_provider_adapter_at_ms(&req, &live, &adapter, 41_000);
        assert!(!commit.low_latency_commit);
        assert!(commit.finalized);
        let batch = commit
            .partial_batch
            .expect("finalized stream must include partial batch");
        assert_eq!(batch.partials.len(), 3);
        assert!(batch.finalized);
        assert_eq!(batch.partials[2].revision_id, 3);
        assert_eq!(
            batch.partials[2].text_chunk,
            "set reminder for tomorrow at noon"
        );
        match commit.response {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set reminder for tomorrow at noon")
            }
            other => panic!("expected finalized transcript_ok, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_5h_step4_streaming_revision_gap_fails_closed() {
        let req = req_with_duration(1400);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        cfg.stream_max_revisions = 4;
        let rt = Ph1cRuntime::new(cfg);
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2203, 3203, "tenant_a".to_string());
        live.retry_budget = 0;

        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::OkSttStream {
                text: "set reminder",
                language: "en",
                confidence_bp: 9_200,
                stable: true,
                latency_ms: 58,
                revision_id: 1,
                finalized: false,
            },
            AdapterAction::OkSttStream {
                text: "set reminder for friday",
                language: "en",
                confidence_bp: 9_300,
                stable: true,
                latency_ms: 59,
                revision_id: 3,
                finalized: true,
            },
            AdapterAction::ProviderTimeout,
        ]);
        let commit = rt.run_stream_via_live_provider_adapter_at_ms(&req, &live, &adapter, 42_000);
        assert!(commit.partial_batch.is_none());
        assert!(!commit.low_latency_commit);
        assert!(!commit.finalized);
        match commit.response {
            Ph1cResponse::TranscriptReject(reject) => {
                assert_eq!(reject.reason_code, reason_codes::STT_FAIL_PARTIAL_ORDER);
            }
            other => panic!("expected fail-closed partial-order reject, got: {other:?}"),
        }
    }

    #[test]
    fn rejects_when_audio_degraded() {
        let mut req = req_with_duration(500);
        req.device_state_ref.health = DeviceHealth::Degraded;
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let out = rt.run(&req, &[]);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_AUDIO_DEGRADED)
        );
    }

    #[test]
    fn retries_and_returns_best_passing_without_leaking_provider() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 200,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.0,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 300,
                transcript_text: "set meeting tomorrow".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.92,
                low_confidence_ratio: 0.05,
                stable: true,
            },
        ];

        let out = rt.run(&req, &attempts);
        match out {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set meeting tomorrow");
            }
            _ => panic!("expected transcript_ok"),
        }
    }

    #[test]
    fn detects_language_mismatch_when_hint_is_high() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en").unwrap(),
            LanguageHintConfidence::High,
        ));

        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "hola".to_string(),
            language_tag: LanguageTag::new("es").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LANGUAGE_MISMATCH)
        );
    }

    #[test]
    fn locale_family_match_accepts_en_us_hint_with_en_actual() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en-US").unwrap(),
            LanguageHintConfidence::High,
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 140,
            transcript_text: "set reminder".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn locale_family_match_accepts_en_hint_with_en_gb_actual() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en").unwrap(),
            LanguageHintConfidence::High,
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 130,
            transcript_text: "set reminder".to_string(),
            language_tag: LanguageTag::new("en-GB").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn locale_family_match_rejects_when_script_conflicts() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("zh-Hans-CN").unwrap(),
            LanguageHintConfidence::High,
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 150,
            transcript_text: "测试".to_string(),
            language_tag: LanguageTag::new("zh-Hant-TW").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LANGUAGE_MISMATCH)
        );
    }

    #[test]
    fn locale_family_match_accepts_underscore_and_case_variants() {
        let mut req = req_with_duration(800);
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("EN-US").unwrap(),
            LanguageHintConfidence::High,
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 125,
            transcript_text: "set reminder".to_string(),
            language_tag: LanguageTag::new("en_us").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn low_coverage_fails_for_long_audio_short_text() {
        let req = req_with_duration(5_000);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "ok".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LOW_COVERAGE)
        );
    }

    #[test]
    fn medium_confidence_transcript_fails_closed_no_guess_words() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "set meeting for tomorrow".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.84,
            low_confidence_ratio: 0.10,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LOW_CONFIDENCE)
        );
    }

    #[test]
    fn at_c_5h_step6_calibration_penalizes_poor_acoustic_path() {
        let mut req = req_with_duration(800);
        req.noise_level_hint =
            Some(selene_kernel_contracts::ph1c::NoiseLevelHint::new(0.92).unwrap());
        req.vad_quality_hint =
            Some(selene_kernel_contracts::ph1c::VadQualityHint::new(0.35).unwrap());
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en").unwrap(),
            LanguageHintConfidence::High,
        ));
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Low,
            VadDecisionConfidenceBand::Low,
            7.0,
            16.0,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Degraded,
                echo_risk_class: EchoRiskClass::High,
                network_stability_class: NetworkStabilityClass::Unstable,
                recoverability_class: RecoverabilityClass::Guarded,
            },
        ));

        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 140,
            transcript_text: "set the reminder for tomorrow".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.03,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LOW_CONFIDENCE)
        );
    }

    #[test]
    fn at_c_5h_step6_calibration_promotes_strong_multisignal_path() {
        let mut req = req_with_duration(800);
        req.noise_level_hint =
            Some(selene_kernel_contracts::ph1c::NoiseLevelHint::new(0.08).unwrap());
        req.vad_quality_hint =
            Some(selene_kernel_contracts::ph1c::VadQualityHint::new(0.96).unwrap());
        req.language_hint = Some(LanguageHint::v1(
            LanguageTag::new("en-US").unwrap(),
            LanguageHintConfidence::High,
        ));
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::High,
            VadDecisionConfidenceBand::High,
            30.0,
            0.5,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Clear,
                echo_risk_class: EchoRiskClass::Low,
                network_stability_class: NetworkStabilityClass::Stable,
                recoverability_class: RecoverabilityClass::Fast,
            },
        ));

        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 120,
            transcript_text: "set the reminder for tomorrow".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.88,
            low_confidence_ratio: 0.05,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn at_c_5h_step7_provider_payload_includes_lexicon_hints() {
        let req = req_with_duration(900);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2301, 3301, "tenant_a".to_string());
        live.enforce_provider_disagreement_clarify = false;
        live.tenant_vocabulary_pack_id = Some("tenant_pack_001".to_string());
        live.user_vocabulary_pack_id = Some("user_pack_jd".to_string());
        live.tenant_lexicon_terms = vec!["Selene Vault".to_string(), "bcast_id".to_string()];
        live.domain_lexicon_terms = vec!["Missed STT".to_string()];
        let adapter = ScriptedAdapter::new(vec![AdapterAction::OkStt {
            text: "selene vault bcast id report",
            language: "en",
            confidence_bp: 9_600,
            stable: true,
            latency_ms: 75,
        }]);
        let out = rt.run_via_live_provider_adapter(&req, &live, &adapter);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));

        let payloads = adapter.seen_input_payload_inline();
        let first = payloads
            .first()
            .cloned()
            .flatten()
            .expect("payload must exist");
        let json: serde_json::Value =
            serde_json::from_str(&first).expect("payload must be valid json");
        assert_eq!(json["tenant_vocabulary_pack_id"], "tenant_pack_001");
        assert_eq!(json["user_vocabulary_pack_id"], "user_pack_jd");
        assert_eq!(json["tenant_lexicon_terms"][0], "bcast id");
        assert_eq!(json["tenant_lexicon_terms"][1], "selene vault");
        assert_eq!(json["domain_lexicon_terms"][0], "missed stt");
    }

    #[test]
    fn at_c_5h_step7_lexicon_boost_promotes_borderline_transcript() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_retries_per_provider = 0;
        let rt = Ph1cRuntime::new(cfg);

        let mut live_without_lexicon =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2302, 3302, "tenant_a".to_string());
        live_without_lexicon.retry_budget = 0;
        live_without_lexicon.enforce_provider_disagreement_clarify = false;
        let adapter_without = ScriptedAdapter::new(vec![
            AdapterAction::OkStt {
                text: "open selenevault report",
                language: "en",
                confidence_bp: 8_400,
                stable: true,
                latency_ms: 79,
            },
            AdapterAction::ProviderTimeout,
        ]);
        let without =
            rt.run_via_live_provider_adapter(&req, &live_without_lexicon, &adapter_without);
        assert!(
            matches!(without, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_LOW_CONFIDENCE)
        );

        let mut live_with_lexicon =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2303, 3303, "tenant_a".to_string());
        live_with_lexicon.retry_budget = 0;
        live_with_lexicon.enforce_provider_disagreement_clarify = false;
        live_with_lexicon.tenant_lexicon_terms = vec!["selenevault".to_string()];
        let adapter_with = ScriptedAdapter::new(vec![
            AdapterAction::OkStt {
                text: "open selenevault report",
                language: "en",
                confidence_bp: 8_400,
                stable: true,
                latency_ms: 79,
            },
            AdapterAction::ProviderTimeout,
        ]);
        let with = rt.run_via_live_provider_adapter(&req, &live_with_lexicon, &adapter_with);
        assert!(matches!(with, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn at_c_5h_step8_intent_aware_repair_recovers_rambling_transcript() {
        let req = req_with_duration(1200);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 110,
            transcript_text: "um um um um remind me tmr to call john tomorrow".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.92,
            low_confidence_ratio: 0.17,
            stable: true,
        }];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(
                    ok.transcript_text,
                    "remind me tomorrow to call john tomorrow"
                );
                assert_eq!(ok.confidence_bucket, ConfidenceBucket::High);
            }
            other => panic!("expected repaired transcript_ok, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_5h_step8_multilingual_scramble_marker_triggers_repair_candidate() {
        let att = SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 115,
            transcript_text: "ok euh rappelle moi demain pour la facture".to_string(),
            language_tag: LanguageTag::new("fr-FR").unwrap(),
            avg_word_confidence: 0.93,
            low_confidence_ratio: 0.10,
            stable: true,
        };
        assert!(is_intent_repair_candidate(&att, &att.transcript_text));
    }

    #[test]
    fn at_c_5h_step8_multilingual_fillers_are_recognized() {
        assert!(is_filler_repair_token("euh"));
        assert!(is_filler_repair_token("يعني"));
        assert!(is_filler_repair_token("那个"));
        assert!(!is_filler_repair_token("rappelle"));
    }

    #[test]
    fn at_c_5h_step8_reference_ambiguity_fails_closed_without_repair_guess() {
        let req = req_with_duration(1100);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 105,
            transcript_text: "um um um um remind me about that".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.91,
            low_confidence_ratio: 0.18,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_GARBLED)
        );
    }

    #[test]
    fn at_c_5h_step11_inhouse_shadow_route_eligible_only_with_governed_gate() {
        let req = req_with_duration(1000);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let provider_truth = SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 100,
            transcript_text: "show missed stt report for june".to_string(),
            language_tag: LanguageTag::new("en-US").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.03,
            stable: true,
        };
        let inhouse_shadow = SttAttempt {
            provider: ProviderSlot::Tertiary,
            latency_ms: 140,
            transcript_text: "show missed stt report for june".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.94,
            low_confidence_ratio: 0.03,
            stable: true,
        };
        let slice_key = Ph1cShadowSliceKey {
            locale: "en-US".to_string(),
            device_route: "desktop_builtin".to_string(),
            tenant_id: "tenant_a".to_string(),
        };

        let hold = rt
            .evaluate_inhouse_shadow_route(
                &req,
                slice_key.clone(),
                &provider_truth,
                &inhouse_shadow,
                false,
            )
            .expect("shadow evaluation should succeed");
        assert_eq!(hold.decision, Ph1cShadowRouteDecision::HoldShadow);
        assert_eq!(
            hold.block_reason_code,
            Some(reason_codes::STT_FAIL_SHADOW_PROMOTION_BLOCKED)
        );

        let promoted = rt
            .evaluate_inhouse_shadow_route(&req, slice_key, &provider_truth, &inhouse_shadow, true)
            .expect("shadow evaluation should succeed");
        assert_eq!(
            promoted.decision,
            Ph1cShadowRouteDecision::EligibleForPromotion
        );
        assert_eq!(promoted.block_reason_code, None);
        assert_eq!(promoted.transcript_overlap_bp, 10_000);
        assert_eq!(promoted.confidence_delta_bp, -100);
    }

    #[test]
    fn at_c_5h_step11_inhouse_shadow_route_rejects_invalid_provider_truth() {
        let req = req_with_duration(1000);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let provider_truth = SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 100,
            transcript_text: " ".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.20,
            low_confidence_ratio: 0.95,
            stable: false,
        };
        let inhouse_shadow = SttAttempt {
            provider: ProviderSlot::Tertiary,
            latency_ms: 120,
            transcript_text: "show missed stt report".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.94,
            low_confidence_ratio: 0.03,
            stable: true,
        };
        let slice_key = Ph1cShadowSliceKey {
            locale: "en".to_string(),
            device_route: "desktop_builtin".to_string(),
            tenant_id: "tenant_a".to_string(),
        };

        let err = rt
            .evaluate_inhouse_shadow_route(&req, slice_key, &provider_truth, &inhouse_shadow, true)
            .expect_err("provider truth must pass PH1.C gate in shadow compare");
        assert_eq!(err, reason_codes::STT_FAIL_SHADOW_PROVIDER_TRUTH_INVALID);
    }

    #[test]
    fn at_c_step13_openai_primary_success_prefers_primary_slot() {
        let req = req_with_duration(800);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 1;
        let rt = Ph1cRuntime::new(cfg);

        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 80,
                transcript_text: "openai primary accepted".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.97,
                low_confidence_ratio: 0.02,
                stable: true,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 70,
                transcript_text: "google secondary should not be selected".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.98,
                low_confidence_ratio: 0.01,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "openai primary accepted");
                let audit_meta = ok.audit_meta.expect("audit metadata must be present");
                assert_eq!(audit_meta.selected_slot, SelectedSlot::Primary);
                assert!(!audit_meta.second_pass_used);
            }
            other => panic!("expected primary-slot transcript_ok, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_step13_google_fallback_on_openai_fail_uses_secondary_slot() {
        let req = req_with_duration(800);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 3;
        let rt = Ph1cRuntime::new(cfg);

        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 120,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.10,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 140,
                transcript_text: "google fallback accepted".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.96,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "google fallback accepted");
                let audit_meta = ok.audit_meta.expect("audit metadata must be present");
                assert_eq!(audit_meta.selected_slot, SelectedSlot::Secondary);
                assert!(audit_meta.second_pass_used);
            }
            other => panic!("expected secondary-slot fallback transcript_ok, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_step13_terminal_fail_closed_when_openai_and_google_fail() {
        let req = req_with_duration(800);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 4;
        let rt = Ph1cRuntime::new(cfg);

        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 90,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.1,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 95,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.1,
                low_confidence_ratio: 1.0,
                stable: false,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptReject(reject) => {
                assert_eq!(reject.reason_code, reason_codes::STT_FAIL_EMPTY);
                assert_eq!(reject.retry_advice, RetryAdvice::Repeat);
                let audit_meta = reject.audit_meta.expect("audit metadata must be present");
                assert_eq!(audit_meta.selected_slot, SelectedSlot::None);
                assert!(audit_meta.second_pass_used);
            }
            other => panic!(
                "expected fail-closed transcript_reject when both slots fail, got: {other:?}"
            ),
        }
    }

    #[test]
    fn stutter_is_rejected_as_garbled() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "I I I I want that".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_GARBLED)
        );
    }

    #[test]
    fn budget_exceeded_fails_closed() {
        let req = req_with_duration(800);
        let rt = Ph1cRuntime::new(Ph1cConfig {
            max_attempts_per_turn: 1,
            max_total_latency_budget_ms: 100,
            ..Ph1cConfig::mvp_desktop_v1()
        });

        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 200,
            transcript_text: "set meeting".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.95,
            low_confidence_ratio: 0.02,
            stable: true,
        }];

        let out = rt.run(&req, &attempts);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.reason_code == reason_codes::STT_FAIL_BUDGET_EXCEEDED)
        );
    }

    #[test]
    fn ph1k_handoff_noise_robust_strategy_still_uses_primary_then_secondary_ladder() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Medium,
            VadDecisionConfidenceBand::Medium,
            12.0,
            6.0,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Guarded,
                echo_risk_class: EchoRiskClass::Elevated,
                network_stability_class: NetworkStabilityClass::Flaky,
                recoverability_class: RecoverabilityClass::Guarded,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_total_latency_budget_ms = 1_000;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 200,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.10,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 180,
                transcript_text: "set meeting tomorrow fallback".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.95,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];
        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set meeting tomorrow fallback")
            }
            other => panic!("expected transcript_ok from secondary fallback, got: {other:?}"),
        }
    }

    #[test]
    fn ph1k_handoff_critical_degradation_forces_clarify_only() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Low,
            VadDecisionConfidenceBand::Low,
            8.0,
            20.0,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Critical,
                echo_risk_class: EchoRiskClass::High,
                network_stability_class: NetworkStabilityClass::Unstable,
                recoverability_class: RecoverabilityClass::FailoverRequired,
            },
        ));
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let out = rt.run(&req, &[]);
        assert!(
            matches!(out, Ph1cResponse::TranscriptReject(r) if r.retry_advice == RetryAdvice::SwitchToText)
        );
    }

    #[test]
    fn ph1k_handoff_cloud_assist_strategy_still_uses_primary_then_secondary_ladder() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::Low,
            VadDecisionConfidenceBand::Low,
            26.0,
            0.5,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Guarded,
                echo_risk_class: EchoRiskClass::Low,
                network_stability_class: NetworkStabilityClass::Stable,
                recoverability_class: RecoverabilityClass::Guarded,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 2;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 90,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.10,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 95,
                transcript_text: "use secondary cloud assist fallback".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.96,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "use secondary cloud assist fallback")
            }
            other => panic!("expected transcript_ok from secondary fallback, got: {other:?}"),
        }
    }

    #[test]
    fn ph1k_handoff_standard_strategy_prefers_primary_when_quality_is_clean() {
        let mut req = req_with_duration(800);
        req.ph1k_handoff = Some(handoff(
            InterruptCandidateConfidenceBand::High,
            VadDecisionConfidenceBand::High,
            30.0,
            0.2,
            DegradationClassBundle {
                capture_quality_class: CaptureQualityClass::Clear,
                echo_risk_class: EchoRiskClass::Low,
                network_stability_class: NetworkStabilityClass::Stable,
                recoverability_class: RecoverabilityClass::Fast,
            },
        ));

        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 1;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 80,
                transcript_text: "primary strategy path".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.97,
                low_confidence_ratio: 0.02,
                stable: true,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 70,
                transcript_text: "secondary strategy path".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.98,
                low_confidence_ratio: 0.01,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "primary strategy path")
            }
            other => {
                panic!("expected transcript_ok from primary-first standard path, got: {other:?}")
            }
        }
    }

    #[test]
    fn retries_per_provider_are_bounded_before_fallback() {
        let req = req_with_duration(800);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.max_attempts_per_turn = 4;
        cfg.max_retries_per_provider = 1;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 50,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.1,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 50,
                transcript_text: "".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.1,
                low_confidence_ratio: 1.0,
                stable: false,
            },
            SttAttempt {
                provider: ProviderSlot::Primary,
                latency_ms: 50,
                transcript_text: "should_be_ignored_due_to_retry_cap".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.99,
                low_confidence_ratio: 0.01,
                stable: true,
            },
            SttAttempt {
                provider: ProviderSlot::Secondary,
                latency_ms: 60,
                transcript_text: "secondary success".to_string(),
                language_tag: LanguageTag::new("en").unwrap(),
                avg_word_confidence: 0.95,
                low_confidence_ratio: 0.02,
                stable: true,
            },
        ];

        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptOk(ok) => assert_eq!(ok.transcript_text, "secondary success"),
            other => {
                panic!("expected secondary success after bounded primary retries, got: {other:?}")
            }
        }
    }

    #[test]
    fn partials_are_ordered_and_deduped_deterministically() {
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![
            SttPartialAttempt {
                text_chunk: "hello there".to_string(),
                confidence: 0.81,
                stable: false,
                revision_id: 2,
            },
            SttPartialAttempt {
                text_chunk: "hello".to_string(),
                confidence: 0.70,
                stable: false,
                revision_id: 1,
            },
            SttPartialAttempt {
                text_chunk: "hello there".to_string(),
                confidence: 0.83,
                stable: true,
                revision_id: 2,
            },
            SttPartialAttempt {
                text_chunk: "hello there jd".to_string(),
                confidence: 0.89,
                stable: true,
                revision_id: 3,
            },
        ];
        let out = rt
            .canonicalize_partial_transcripts(&attempts, true)
            .expect("partials should canonicalize");
        assert_eq!(out.partials.len(), 3);
        assert_eq!(out.partials[0].revision_id, 1);
        assert_eq!(out.partials[1].revision_id, 2);
        assert!(out.partials[1].stable);
        assert_eq!(out.partials[2].revision_id, 3);
        assert_eq!(out.partials[2].text_chunk, "hello there jd");
        assert!(out.finalized);
    }

    #[test]
    fn partials_fail_closed_when_revision_order_has_gap() {
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![
            SttPartialAttempt {
                text_chunk: "hello".to_string(),
                confidence: 0.77,
                stable: false,
                revision_id: 1,
            },
            SttPartialAttempt {
                text_chunk: "hello jd".to_string(),
                confidence: 0.84,
                stable: true,
                revision_id: 3,
            },
        ];
        let err = rt
            .canonicalize_partial_transcripts(&attempts, true)
            .expect_err("gapped revisions must fail closed");
        assert_eq!(err.reason_code, reason_codes::STT_FAIL_PARTIAL_ORDER);
    }

    #[test]
    fn partials_fail_closed_when_finalized_without_stable_last() {
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![
            SttPartialAttempt {
                text_chunk: "hello".to_string(),
                confidence: 0.77,
                stable: false,
                revision_id: 1,
            },
            SttPartialAttempt {
                text_chunk: "hello jd".to_string(),
                confidence: 0.84,
                stable: false,
                revision_id: 2,
            },
        ];
        let err = rt
            .canonicalize_partial_transcripts(&attempts, true)
            .expect_err("finalized stream requires stable last revision");
        assert_eq!(err.reason_code, reason_codes::STT_FAIL_PARTIAL_INVALID);
    }

    #[test]
    fn partials_allow_unstable_last_when_not_finalized() {
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![
            SttPartialAttempt {
                text_chunk: "hello".to_string(),
                confidence: 0.77,
                stable: false,
                revision_id: 1,
            },
            SttPartialAttempt {
                text_chunk: "hello jd".to_string(),
                confidence: 0.84,
                stable: false,
                revision_id: 2,
            },
        ];
        let out = rt
            .canonicalize_partial_transcripts(&attempts, false)
            .expect("non-finalized partial stream should allow unstable tail");
        assert_eq!(out.partials.len(), 2);
        assert!(!out.finalized);
    }

    #[test]
    fn at_c_5i_step4_overlap_ambiguity_fails_closed() {
        let mut req = req_with_duration(900);
        req = req
            .with_speaker_overlap_hint(Some(
                SpeakerOverlapHint::v1(
                    SpeakerOverlapClass::MultiSpeaker,
                    Confidence::new(0.92).unwrap(),
                )
                .unwrap(),
            ))
            .unwrap();
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 110,
            transcript_text: "speaker 1: send payment speaker 2: cancel payment".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.97,
            low_confidence_ratio: 0.01,
            stable: true,
        }];
        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptReject(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::STT_FAIL_SPEAKER_OVERLAP_AMBIGUOUS
                );
            }
            other => panic!("expected overlap fail-closed reject, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_5i_step6_provider_disagreement_forces_clarify() {
        let req = req_with_duration(900);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2501, 3501, "tenant_a".to_string());
        live.retry_budget = 0;
        live.enforce_provider_disagreement_clarify = true;
        live.provider_disagreement_divergence_bp_threshold = 2_000;
        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::OkStt {
                text: "set reminder for john",
                language: "en",
                confidence_bp: 9_700,
                stable: true,
                latency_ms: 80,
            },
            AdapterAction::OkStt {
                text: "set reminder for joan next week",
                language: "en",
                confidence_bp: 9_650,
                stable: true,
                latency_ms: 90,
            },
        ]);

        match rt.run_via_live_provider_adapter(&req, &live, &adapter) {
            Ph1cResponse::TranscriptReject(r) => {
                assert_eq!(r.reason_code, reason_codes::STT_FAIL_PROVIDER_DISAGREEMENT);
            }
            other => panic!("expected provider disagreement reject, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_5i_step11_cost_quality_prefers_cheaper_secondary_within_tolerance() {
        let req = req_with_duration(900);
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let mut live =
            Ph1cLiveProviderContext::mvp_openai_google_v1(2502, 3502, "tenant_a".to_string());
        live.retry_budget = 0;
        live.enforce_provider_disagreement_clarify = true;
        live.provider_disagreement_divergence_bp_threshold = 9_500;
        live.enable_cost_quality_routing = true;
        live.primary_cost_microunits = 5_000;
        live.secondary_cost_microunits = 2_000;
        let adapter = ScriptedAdapter::new(vec![
            AdapterAction::OkStt {
                text: "set reminder for john",
                language: "en",
                confidence_bp: 9_800,
                stable: true,
                latency_ms: 80,
            },
            AdapterAction::OkStt {
                text: "set reminder for john",
                language: "en",
                confidence_bp: 9_700,
                stable: true,
                latency_ms: 86,
            },
        ]);

        match rt.run_via_live_provider_adapter(&req, &live, &adapter) {
            Ph1cResponse::TranscriptOk(ok) => {
                assert_eq!(ok.transcript_text, "set reminder for john");
                let audit = ok.audit_meta.expect("audit meta must be present");
                assert_eq!(audit.selected_slot, SelectedSlot::Secondary);
                assert!(audit.second_pass_used);
            }
            other => {
                panic!("expected transcript_ok on cost-quality secondary route, got: {other:?}")
            }
        }
    }

    #[test]
    fn at_c_5i_step2_semantic_gate_blocks_low_intent_quality() {
        let req = req_with_duration(900);
        let mut cfg = Ph1cConfig::mvp_desktop_v1();
        cfg.semantic_gate_enabled = true;
        cfg.min_semantic_quality = 3;
        let rt = Ph1cRuntime::new(cfg);
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 90,
            transcript_text: "hello there yes maybe".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.98,
            low_confidence_ratio: 0.01,
            stable: true,
        }];
        match rt.run(&req, &attempts) {
            Ph1cResponse::TranscriptReject(r) => {
                assert_eq!(
                    r.reason_code,
                    reason_codes::STT_FAIL_LOW_SEMANTIC_CONFIDENCE
                );
            }
            other => panic!("expected semantic-confidence reject, got: {other:?}"),
        }
    }
}
