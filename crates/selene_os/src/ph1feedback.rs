#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1c::{ConfidenceBucket, Ph1cResponse, QualityBucket, SelectedSlot};
use selene_kernel_contracts::ph1d::{
    Ph1dProviderCallResponse, Ph1dProviderStatus, Ph1dProviderTask, Ph1dProviderValidationStatus,
};
use selene_kernel_contracts::ph1feedback::{
    FeedbackCapabilityId, FeedbackConfidenceBucket, FeedbackEventCollectOk,
    FeedbackEventCollectRequest, FeedbackEventRecord, FeedbackEventType, FeedbackGoldStatus,
    FeedbackMetrics, FeedbackPathType, FeedbackRefuse, FeedbackRequestEnvelope,
    FeedbackSignalEmitOk, FeedbackSignalEmitRequest, FeedbackToolStatus, FeedbackValidationStatus,
    Ph1FeedbackRequest, Ph1FeedbackResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1selfheal::{
    stable_card_id, FailureContainmentAction, FailureEvent, FailureProviderContext,
};
use selene_kernel_contracts::ph1tts::Ph1ttsEvent;
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.FEEDBACK OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_FEEDBACK_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4642_0101);
    pub const PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4642_01F1);
    pub const PH1_FEEDBACK_GOLDCASE_STT_LOW_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x4642_0211);
    pub const PH1_FEEDBACK_GOLDCASE_PROVIDER_LOW_CONFIDENCE: ReasonCodeId =
        ReasonCodeId(0x4642_0212);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1FeedbackWiringConfig {
    pub feedback_enabled: bool,
    pub max_events: u8,
    pub max_signals: u8,
}

impl Ph1FeedbackWiringConfig {
    pub fn mvp_v1(feedback_enabled: bool) -> Self {
        Self {
            feedback_enabled,
            max_events: 24,
            max_signals: 12,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub events: Vec<FeedbackEventRecord>,
}

impl FeedbackTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        events: Vec<FeedbackEventRecord>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            events,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for FeedbackTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.events.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_turn_input.events",
                reason: "must be <= 64",
            });
        }
        for event in &self.events {
            event.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub event_collect: FeedbackEventCollectOk,
    pub signal_emit: FeedbackSignalEmitOk,
}

impl FeedbackForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        event_collect: FeedbackEventCollectOk,
        signal_emit: FeedbackSignalEmitOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            event_collect,
            signal_emit,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for FeedbackForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.event_collect.validate()?;
        self.signal_emit.validate()?;
        if self.signal_emit.validation_status != FeedbackValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_forward_bundle.signal_emit.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedbackWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoEvents,
    Refused(FeedbackRefuse),
    Forwarded(FeedbackForwardBundle),
}

pub fn map_feedback_event_to_failure_event(
    event: &FeedbackEventRecord,
    forward_bundle: &FeedbackForwardBundle,
    containment_action: FailureContainmentAction,
    escalation_required: bool,
    unresolved_reason: Option<String>,
    provider_context: Option<FailureProviderContext>,
) -> Result<FailureEvent, ContractViolation> {
    event.validate()?;
    forward_bundle.validate()?;

    if event.correlation_id != forward_bundle.correlation_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_feedback_event_to_failure_event.event.correlation_id",
            reason: "must match feedback_forward_bundle.correlation_id",
        });
    }
    if event.turn_id != forward_bundle.turn_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_feedback_event_to_failure_event.event.turn_id",
            reason: "must match feedback_forward_bundle.turn_id",
        });
    }

    let reason_hex = format!("{:08x}", event.reason_code.0);
    let route_domain = provider_context
        .as_ref()
        .map(|ctx| route_domain_key(ctx.route_domain))
        .unwrap_or("NONE");
    let fingerprint = stable_card_id(
        "fingerprint",
        &[
            feedback_event_type_key(event.event_type),
            feedback_path_type_key(event.path_type),
            reason_hex.as_str(),
            event.evidence_ref.as_str(),
            route_domain,
        ],
    )?;

    FailureEvent::v1(
        event.event_id.clone(),
        event.tenant_id.clone(),
        event.user_id.clone(),
        event.speaker_id.clone(),
        event.session_id.clone(),
        event.device_id.clone(),
        event.correlation_id,
        event.turn_id,
        event.event_type,
        event.reason_code,
        event.path_type,
        event.evidence_ref.clone(),
        event.idempotency_key.clone(),
        event.metrics.confidence_bucket,
        event.metrics.tool_status,
        event.metrics.latency_ms,
        event.metrics.retries,
        event.metrics.missing_fields.clone(),
        fingerprint,
        containment_action,
        escalation_required,
        unresolved_reason,
        provider_context,
    )
}

fn feedback_event_type_key(
    event_type: selene_kernel_contracts::ph1feedback::FeedbackEventType,
) -> &'static str {
    use selene_kernel_contracts::ph1feedback::FeedbackEventType as T;
    match event_type {
        T::SttReject => "STT_REJECT",
        T::SttRetry => "STT_RETRY",
        T::LanguageMismatch => "LANGUAGE_MISMATCH",
        T::UserCorrection => "USER_CORRECTION",
        T::ClarifyLoop => "CLARIFY_LOOP",
        T::ConfirmAbort => "CONFIRM_ABORT",
        T::ToolFail => "TOOL_FAIL",
        T::MemoryOverride => "MEMORY_OVERRIDE",
        T::DeliverySwitch => "DELIVERY_SWITCH",
        T::BargeIn => "BARGE_IN",
        T::VoiceIdFalseReject => "VOICE_ID_FALSE_REJECT",
        T::VoiceIdFalseAccept => "VOICE_ID_FALSE_ACCEPT",
        T::VoiceIdSpoofRisk => "VOICE_ID_SPOOF_RISK",
        T::VoiceIdMultiSpeaker => "VOICE_ID_MULTI_SPEAKER",
        T::VoiceIdDriftAlert => "VOICE_ID_DRIFT_ALERT",
        T::VoiceIdReauthFriction => "VOICE_ID_REAUTH_FRICTION",
        T::VoiceIdConfusionPair => "VOICE_ID_CONFUSION_PAIR",
        T::VoiceIdDrift => "VOICE_ID_DRIFT",
        T::VoiceIdLowQuality => "VOICE_ID_LOW_QUALITY",
    }
}

fn feedback_path_type_key(
    path_type: selene_kernel_contracts::ph1feedback::FeedbackPathType,
) -> &'static str {
    use selene_kernel_contracts::ph1feedback::FeedbackPathType as P;
    match path_type {
        P::Defect => "DEFECT",
        P::Improvement => "IMPROVEMENT",
    }
}

fn route_domain_key(route_domain: selene_kernel_contracts::ph1pae::PaeRouteDomain) -> &'static str {
    use selene_kernel_contracts::ph1pae::PaeRouteDomain as D;
    match route_domain {
        D::Stt => "STT",
        D::Tts => "TTS",
        D::Llm => "LLM",
        D::Tooling => "TOOLING",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldCaseOwnerEngine {
    Ph1c,
    Ph1d,
    Ph1tts,
}

impl GoldCaseOwnerEngine {
    pub const fn as_str(self) -> &'static str {
        match self {
            GoldCaseOwnerEngine::Ph1c => "PH1.C",
            GoldCaseOwnerEngine::Ph1d => "PH1.D",
            GoldCaseOwnerEngine::Ph1tts => "PH1.TTS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldCaseCaptureContext {
    pub tenant_id: String,
    pub user_id: String,
    pub speaker_id: String,
    pub session_id: String,
    pub device_id: String,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub idempotency_root: String,
}

impl GoldCaseCaptureContext {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: String,
        user_id: String,
        speaker_id: String,
        session_id: String,
        device_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        idempotency_root: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            tenant_id,
            user_id,
            speaker_id,
            session_id,
            device_id,
            correlation_id,
            turn_id,
            idempotency_root,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for GoldCaseCaptureContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token_like("gold_case_capture_context.tenant_id", &self.tenant_id, 64)?;
        validate_token_like("gold_case_capture_context.user_id", &self.user_id, 96)?;
        validate_token_like("gold_case_capture_context.speaker_id", &self.speaker_id, 96)?;
        validate_token_like("gold_case_capture_context.session_id", &self.session_id, 96)?;
        validate_token_like("gold_case_capture_context.device_id", &self.device_id, 96)?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token_like(
            "gold_case_capture_context.idempotency_root",
            &self.idempotency_root,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoldCaseCapture {
    pub owner_engine: GoldCaseOwnerEngine,
    pub feedback_event: FeedbackEventRecord,
    pub gold_case_id: String,
    pub primary_failure_fingerprint: String,
    pub secondary_failure_fingerprint: String,
    pub reason_code_chain: Vec<ReasonCodeId>,
    pub final_accepted_transcript: Option<String>,
    pub language_locale: Option<String>,
}

impl GoldCaseCapture {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        owner_engine: GoldCaseOwnerEngine,
        feedback_event: FeedbackEventRecord,
        gold_case_id: String,
        primary_failure_fingerprint: String,
        secondary_failure_fingerprint: String,
        reason_code_chain: Vec<ReasonCodeId>,
        final_accepted_transcript: Option<String>,
        language_locale: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            owner_engine,
            feedback_event,
            gold_case_id,
            primary_failure_fingerprint,
            secondary_failure_fingerprint,
            reason_code_chain,
            final_accepted_transcript,
            language_locale,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for GoldCaseCapture {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.feedback_event.validate()?;
        if self.feedback_event.path_type != FeedbackPathType::Improvement {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.feedback_event.path_type",
                reason: "must be IMPROVEMENT",
            });
        }
        if self.feedback_event.gold_status != FeedbackGoldStatus::Pending {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.feedback_event.gold_status",
                reason: "must be PENDING",
            });
        }
        validate_token_like("gold_case_capture.gold_case_id", &self.gold_case_id, 96)?;
        if self.feedback_event.gold_case_id.as_deref() != Some(self.gold_case_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.feedback_event.gold_case_id",
                reason: "must match gold_case_capture.gold_case_id",
            });
        }
        validate_token_like(
            "gold_case_capture.primary_failure_fingerprint",
            &self.primary_failure_fingerprint,
            96,
        )?;
        validate_token_like(
            "gold_case_capture.secondary_failure_fingerprint",
            &self.secondary_failure_fingerprint,
            96,
        )?;
        if self.reason_code_chain.is_empty() || self.reason_code_chain.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.reason_code_chain",
                reason: "must contain 1..=8 reason codes",
            });
        }
        if self
            .reason_code_chain
            .iter()
            .any(|reason_code| reason_code.0 == 0)
        {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.reason_code_chain",
                reason: "reason codes must be > 0",
            });
        }
        if !self
            .reason_code_chain
            .iter()
            .any(|reason_code| *reason_code == self.feedback_event.reason_code)
        {
            return Err(ContractViolation::InvalidValue {
                field: "gold_case_capture.reason_code_chain",
                reason: "must include feedback_event.reason_code",
            });
        }
        if let Some(transcript) = &self.final_accepted_transcript {
            if transcript.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "gold_case_capture.final_accepted_transcript",
                    reason: "must not be empty when present",
                });
            }
            if transcript.len() > 32_768 {
                return Err(ContractViolation::InvalidValue {
                    field: "gold_case_capture.final_accepted_transcript",
                    reason: "must be <= 32768 chars",
                });
            }
        }
        if let Some(language_locale) = &self.language_locale {
            validate_token_like("gold_case_capture.language_locale", language_locale, 32)?;
        }
        Ok(())
    }
}

pub fn build_gold_case_capture_from_ph1c_response(
    context: &GoldCaseCaptureContext,
    response: &Ph1cResponse,
    evidence_ref: Option<&str>,
    final_accepted_transcript: Option<String>,
) -> Result<Option<GoldCaseCapture>, ContractViolation> {
    context.validate()?;
    match response {
        Ph1cResponse::TranscriptReject(reject) => {
            reject.validate()?;
            let selected_slot = reject
                .audit_meta
                .as_ref()
                .map(|meta| selected_slot_key(meta.selected_slot))
                .unwrap_or("NONE");
            let evidence_ref = resolve_evidence_ref(
                GoldCaseOwnerEngine::Ph1c,
                context,
                evidence_ref,
                &format!("transcript_reject:{selected_slot}"),
            )?;
            let reason_code_chain = vec![reject.reason_code];
            let capture = build_gold_case_capture(
                GoldCaseOwnerEngine::Ph1c,
                context,
                FeedbackEventType::SttReject,
                reject.reason_code,
                evidence_ref,
                reject
                    .audit_meta
                    .as_ref()
                    .map(|meta| map_quality_bucket(meta.quality_confidence_bucket))
                    .unwrap_or(FeedbackConfidenceBucket::Unknown),
                FeedbackToolStatus::Fail,
                reject
                    .audit_meta
                    .as_ref()
                    .map(|meta| meta.total_latency_ms)
                    .unwrap_or(0),
                reject
                    .audit_meta
                    .as_ref()
                    .map(|meta| meta.attempt_count.saturating_sub(1))
                    .unwrap_or(0),
                vec!["transcript_rejected".to_string()],
                reason_code_chain,
                None,
                final_accepted_transcript,
                &format!("transcript_reject:{selected_slot}"),
            )?;
            Ok(Some(capture))
        }
        Ph1cResponse::TranscriptOk(ok) => {
            ok.validate()?;
            let low_confidence = ok.confidence_bucket != ConfidenceBucket::High;
            let has_uncertain_spans = !ok.uncertain_spans.is_empty();
            if !low_confidence && !has_uncertain_spans {
                return Ok(None);
            }
            let confidence_bucket = map_ph1c_confidence_bucket(ok.confidence_bucket);
            let mut missing_fields = Vec::new();
            if low_confidence {
                missing_fields.push("transcript_confidence".to_string());
            }
            if has_uncertain_spans {
                missing_fields.push("uncertain_span".to_string());
            }
            let selected_slot = ok
                .audit_meta
                .as_ref()
                .map(|meta| selected_slot_key(meta.selected_slot))
                .unwrap_or("NONE");
            let evidence_ref = resolve_evidence_ref(
                GoldCaseOwnerEngine::Ph1c,
                context,
                evidence_ref,
                &format!(
                    "transcript_low_conf:{}:{}",
                    selected_slot,
                    feedback_confidence_bucket_key(confidence_bucket)
                ),
            )?;
            let reason_code_chain = vec![reason_codes::PH1_FEEDBACK_GOLDCASE_STT_LOW_CONFIDENCE];
            let capture = build_gold_case_capture(
                GoldCaseOwnerEngine::Ph1c,
                context,
                FeedbackEventType::SttRetry,
                reason_codes::PH1_FEEDBACK_GOLDCASE_STT_LOW_CONFIDENCE,
                evidence_ref,
                confidence_bucket,
                FeedbackToolStatus::Conflict,
                ok.audit_meta
                    .as_ref()
                    .map(|meta| meta.total_latency_ms)
                    .unwrap_or(0),
                ok.audit_meta
                    .as_ref()
                    .map(|meta| meta.attempt_count.saturating_sub(1))
                    .unwrap_or(0),
                missing_fields,
                reason_code_chain,
                Some(ok.language_tag.as_str().to_string()),
                final_accepted_transcript.or_else(|| Some(ok.transcript_text.clone())),
                &format!(
                    "transcript_low_conf:{}:{}",
                    selected_slot,
                    feedback_confidence_bucket_key(confidence_bucket)
                ),
            )?;
            Ok(Some(capture))
        }
    }
}

pub fn build_gold_case_capture_from_ph1d_response(
    context: &GoldCaseCaptureContext,
    response: &Ph1dProviderCallResponse,
    evidence_ref: Option<&str>,
    final_accepted_transcript: Option<String>,
    language_locale: Option<String>,
) -> Result<Option<GoldCaseCapture>, ContractViolation> {
    context.validate()?;
    response.validate()?;
    let confidence_bucket = provider_confidence_to_feedback_bucket(response.provider_confidence_bp);
    let low_confidence_stt = response.provider_task == Ph1dProviderTask::SttTranscribe
        && confidence_bucket == FeedbackConfidenceBucket::Low;
    let provider_failure = response.provider_status == Ph1dProviderStatus::Error
        || response.validation_status == Ph1dProviderValidationStatus::SchemaFail;
    if !provider_failure && !low_confidence_stt {
        return Ok(None);
    }
    let event_type = if response.provider_task == Ph1dProviderTask::SttTranscribe {
        FeedbackEventType::SttReject
    } else {
        FeedbackEventType::ToolFail
    };
    let event_reason_code = if low_confidence_stt && !provider_failure {
        reason_codes::PH1_FEEDBACK_GOLDCASE_PROVIDER_LOW_CONFIDENCE
    } else {
        response.reason_code
    };
    let mut reason_code_chain = vec![response.reason_code];
    if event_reason_code != response.reason_code {
        reason_code_chain.push(event_reason_code);
    }

    let tool_status = if response.validation_status == Ph1dProviderValidationStatus::SchemaFail {
        FeedbackToolStatus::Conflict
    } else if response.provider_status == Ph1dProviderStatus::Error {
        FeedbackToolStatus::Fail
    } else {
        FeedbackToolStatus::Ok
    };
    let mut missing_fields = Vec::new();
    if response.validation_status == Ph1dProviderValidationStatus::SchemaFail {
        missing_fields.push("normalized_output_schema".to_string());
        if response.normalized_output_json.is_none() {
            missing_fields.push("normalized_output_json".to_string());
        }
    }
    if low_confidence_stt {
        missing_fields.push("provider_confidence".to_string());
    }

    let secondary_hint = format!(
        "{}:{}:{}",
        response.provider_task.as_str(),
        response.provider_status.as_str(),
        response.validation_status.as_str()
    );
    let evidence_ref = resolve_evidence_ref(
        GoldCaseOwnerEngine::Ph1d,
        context,
        evidence_ref.or(response.provider_call_id.as_deref()),
        secondary_hint.as_str(),
    )?;
    let capture = build_gold_case_capture(
        GoldCaseOwnerEngine::Ph1d,
        context,
        event_type,
        event_reason_code,
        evidence_ref,
        confidence_bucket,
        tool_status,
        response.provider_latency_ms,
        0,
        missing_fields,
        reason_code_chain,
        language_locale,
        final_accepted_transcript,
        secondary_hint.as_str(),
    )?;
    Ok(Some(capture))
}

pub fn build_gold_case_capture_from_ph1tts_event(
    context: &GoldCaseCaptureContext,
    event: &Ph1ttsEvent,
    evidence_ref: Option<&str>,
    final_accepted_transcript: Option<String>,
    language_locale: Option<String>,
) -> Result<Option<GoldCaseCapture>, ContractViolation> {
    context.validate()?;
    event.validate()?;
    let Ph1ttsEvent::Failed(failed) = event else {
        return Ok(None);
    };
    let secondary_hint = format!("tts_failed:{}", failed.answer_id.0);
    let evidence_ref = resolve_evidence_ref(
        GoldCaseOwnerEngine::Ph1tts,
        context,
        evidence_ref,
        secondary_hint.as_str(),
    )?;
    let capture = build_gold_case_capture(
        GoldCaseOwnerEngine::Ph1tts,
        context,
        FeedbackEventType::ToolFail,
        failed.reason_code,
        evidence_ref,
        FeedbackConfidenceBucket::Unknown,
        FeedbackToolStatus::Fail,
        0,
        0,
        vec!["tts_audio_output".to_string()],
        vec![failed.reason_code],
        language_locale,
        final_accepted_transcript,
        secondary_hint.as_str(),
    )?;
    Ok(Some(capture))
}

#[allow(clippy::too_many_arguments)]
fn build_gold_case_capture(
    owner_engine: GoldCaseOwnerEngine,
    context: &GoldCaseCaptureContext,
    event_type: FeedbackEventType,
    reason_code: ReasonCodeId,
    evidence_ref: String,
    confidence_bucket: FeedbackConfidenceBucket,
    tool_status: FeedbackToolStatus,
    latency_ms: u32,
    retries: u8,
    missing_fields: Vec<String>,
    reason_code_chain: Vec<ReasonCodeId>,
    language_locale: Option<String>,
    final_accepted_transcript: Option<String>,
    secondary_cluster_hint: &str,
) -> Result<GoldCaseCapture, ContractViolation> {
    context.validate()?;
    let correlation_id = context.correlation_id.0.to_string();
    let turn_id = context.turn_id.0.to_string();
    let event_type_key = feedback_event_type_key(event_type);
    let reason_hex = format!("{:08x}", reason_code.0);
    let confidence_key = feedback_confidence_bucket_key(confidence_bucket);
    let language_key = language_locale.as_deref().unwrap_or("unknown");

    let gold_case_id = stable_card_id(
        "gold_case",
        &[
            owner_engine.as_str(),
            event_type_key,
            correlation_id.as_str(),
            turn_id.as_str(),
            reason_hex.as_str(),
            secondary_cluster_hint,
        ],
    )?;
    let event_id = stable_card_id("fb_event", &[gold_case_id.as_str(), evidence_ref.as_str()])?;
    let idempotency_key = stable_card_id(
        "fb_idem",
        &[context.idempotency_root.as_str(), gold_case_id.as_str()],
    )?;
    let primary_failure_fingerprint = stable_card_id(
        "ffp_primary",
        &[
            owner_engine.as_str(),
            event_type_key,
            reason_hex.as_str(),
            confidence_key,
            language_key,
        ],
    )?;
    let secondary_failure_fingerprint = stable_card_id(
        "ffp_second",
        &[primary_failure_fingerprint.as_str(), secondary_cluster_hint],
    )?;
    let metrics = FeedbackMetrics::v1(
        latency_ms,
        retries,
        confidence_bucket,
        missing_fields,
        tool_status,
    )?;
    let feedback_event = FeedbackEventRecord::v2(
        event_id,
        context.tenant_id.clone(),
        context.user_id.clone(),
        context.speaker_id.clone(),
        context.session_id.clone(),
        context.device_id.clone(),
        context.correlation_id,
        context.turn_id,
        event_type,
        FeedbackPathType::Improvement,
        Some(gold_case_id.clone()),
        None,
        FeedbackGoldStatus::Pending,
        reason_code,
        evidence_ref,
        idempotency_key,
        metrics,
    )?;
    GoldCaseCapture::v1(
        owner_engine,
        feedback_event,
        gold_case_id,
        primary_failure_fingerprint,
        secondary_failure_fingerprint,
        reason_code_chain,
        final_accepted_transcript,
        language_locale,
    )
}

fn resolve_evidence_ref(
    owner_engine: GoldCaseOwnerEngine,
    context: &GoldCaseCaptureContext,
    evidence_ref: Option<&str>,
    secondary_hint: &str,
) -> Result<String, ContractViolation> {
    if let Some(evidence_ref) = evidence_ref {
        if !evidence_ref.trim().is_empty() {
            return Ok(evidence_ref.to_string());
        }
    }
    let correlation_id = context.correlation_id.0.to_string();
    let turn_id = context.turn_id.0.to_string();
    stable_card_id(
        "evidence",
        &[
            owner_engine.as_str(),
            correlation_id.as_str(),
            turn_id.as_str(),
            secondary_hint,
        ],
    )
}

fn map_ph1c_confidence_bucket(bucket: ConfidenceBucket) -> FeedbackConfidenceBucket {
    match bucket {
        ConfidenceBucket::High => FeedbackConfidenceBucket::High,
        ConfidenceBucket::Med => FeedbackConfidenceBucket::Med,
        ConfidenceBucket::Low => FeedbackConfidenceBucket::Low,
    }
}

fn map_quality_bucket(bucket: QualityBucket) -> FeedbackConfidenceBucket {
    match bucket {
        QualityBucket::High => FeedbackConfidenceBucket::High,
        QualityBucket::Med => FeedbackConfidenceBucket::Med,
        QualityBucket::Low => FeedbackConfidenceBucket::Low,
    }
}

fn provider_confidence_to_feedback_bucket(
    provider_confidence_bp: Option<u16>,
) -> FeedbackConfidenceBucket {
    match provider_confidence_bp {
        Some(value) if value >= 8_500 => FeedbackConfidenceBucket::High,
        Some(value) if value >= 6_500 => FeedbackConfidenceBucket::Med,
        Some(_) => FeedbackConfidenceBucket::Low,
        None => FeedbackConfidenceBucket::Unknown,
    }
}

fn feedback_confidence_bucket_key(confidence_bucket: FeedbackConfidenceBucket) -> &'static str {
    match confidence_bucket {
        FeedbackConfidenceBucket::High => "HIGH",
        FeedbackConfidenceBucket::Med => "MED",
        FeedbackConfidenceBucket::Low => "LOW",
        FeedbackConfidenceBucket::Unknown => "UNKNOWN",
    }
}

fn selected_slot_key(selected_slot: SelectedSlot) -> &'static str {
    match selected_slot {
        SelectedSlot::Primary => "PRIMARY",
        SelectedSlot::Secondary => "SECONDARY",
        SelectedSlot::Tertiary => "TERTIARY",
        SelectedSlot::None => "NONE",
    }
}

fn validate_token_like(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|ch| ch.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

pub trait Ph1FeedbackEngine {
    fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1FeedbackWiring<E>
where
    E: Ph1FeedbackEngine,
{
    config: Ph1FeedbackWiringConfig,
    engine: E,
}

impl<E> Ph1FeedbackWiring<E>
where
    E: Ph1FeedbackEngine,
{
    pub fn new(config: Ph1FeedbackWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_events == 0 || config.max_events > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1feedback_wiring_config.max_events",
                reason: "must be within 1..=64",
            });
        }
        if config.max_signals == 0 || config.max_signals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1feedback_wiring_config.max_signals",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &FeedbackTurnInput,
    ) -> Result<FeedbackWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.feedback_enabled {
            return Ok(FeedbackWiringOutcome::NotInvokedDisabled);
        }
        if input.events.is_empty() {
            return Ok(FeedbackWiringOutcome::NotInvokedNoEvents);
        }

        let envelope = FeedbackRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_events, 64),
            min(self.config.max_signals, 32),
        )?;

        let collect_req = Ph1FeedbackRequest::FeedbackEventCollect(
            FeedbackEventCollectRequest::v1(envelope.clone(), input.events.clone())?,
        );
        let collect_resp = self.engine.run(&collect_req);
        collect_resp.validate()?;

        let collect_ok = match collect_resp {
            Ph1FeedbackResponse::Refuse(refuse) => {
                return Ok(FeedbackWiringOutcome::Refused(refuse))
            }
            Ph1FeedbackResponse::FeedbackEventCollectOk(ok) => ok,
            Ph1FeedbackResponse::FeedbackSignalEmitOk(_) => {
                return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                    FeedbackCapabilityId::FeedbackEventCollect,
                    reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-emit response for event-collect request".to_string(),
                )?));
            }
        };

        let emit_req = Ph1FeedbackRequest::FeedbackSignalEmit(FeedbackSignalEmitRequest::v1(
            envelope,
            collect_ok.selected_candidate_id.clone(),
            collect_ok.ordered_signal_candidates.clone(),
        )?);
        let emit_resp = self.engine.run(&emit_req);
        emit_resp.validate()?;

        let emit_ok = match emit_resp {
            Ph1FeedbackResponse::Refuse(refuse) => {
                return Ok(FeedbackWiringOutcome::Refused(refuse))
            }
            Ph1FeedbackResponse::FeedbackSignalEmitOk(ok) => ok,
            Ph1FeedbackResponse::FeedbackEventCollectOk(_) => {
                return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                    FeedbackCapabilityId::FeedbackSignalEmit,
                    reason_codes::PH1_FEEDBACK_INTERNAL_PIPELINE_ERROR,
                    "unexpected event-collect response for signal-emit request".to_string(),
                )?));
            }
        };

        if emit_ok.validation_status != FeedbackValidationStatus::Ok {
            return Ok(FeedbackWiringOutcome::Refused(FeedbackRefuse::v1(
                FeedbackCapabilityId::FeedbackSignalEmit,
                reason_codes::PH1_FEEDBACK_VALIDATION_FAILED,
                "feedback signal-emit validation failed".to_string(),
            )?));
        }

        let bundle =
            FeedbackForwardBundle::v1(input.correlation_id, input.turn_id, collect_ok, emit_ok)?;
        Ok(FeedbackWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{LanguageTag, RetryAdvice, TranscriptOk, TranscriptReject};
    use selene_kernel_contracts::ph1d::{
        Ph1dProviderCallResponse, Ph1dProviderStatus, Ph1dProviderTask,
        Ph1dProviderValidationStatus, RequestId,
    };
    use selene_kernel_contracts::ph1feedback::{
        FeedbackConfidenceBucket, FeedbackEventType, FeedbackMetrics, FeedbackToolStatus,
    };
    use selene_kernel_contracts::ph1pae::{PaeProviderSlot, PaeRouteDomain};
    use selene_kernel_contracts::ph1selfheal::FailureContainmentAction;
    use selene_kernel_contracts::ph1tts::{AnswerId, TtsFailed, PH1TTS_CONTRACT_VERSION};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicFeedbackEngine;

    impl Ph1FeedbackEngine for DeterministicFeedbackEngine {
        fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
            match req {
                Ph1FeedbackRequest::FeedbackEventCollect(r) => {
                    let mut candidates = r
                        .events
                        .iter()
                        .enumerate()
                        .map(|(idx, event)| {
                            selene_kernel_contracts::ph1feedback::FeedbackSignalCandidate::v1(
                                format!("signal_{}", event.event_id),
                                event.event_type,
                                "correction_rate_by_intent".to_string(),
                                selene_kernel_contracts::ph1feedback::FeedbackSignalTarget::LearnPackage,
                                1200 - (idx as i16 * 100),
                                1,
                                event.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    candidates.sort_by(|a, b| {
                        b.signal_value_bp
                            .cmp(&a.signal_value_bp)
                            .then(a.candidate_id.cmp(&b.candidate_id))
                    });
                    Ph1FeedbackResponse::FeedbackEventCollectOk(
                        FeedbackEventCollectOk::v1(
                            ReasonCodeId(101),
                            candidates[0].candidate_id.clone(),
                            candidates,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1FeedbackRequest::FeedbackSignalEmit(_r) => {
                    Ph1FeedbackResponse::FeedbackSignalEmitOk(
                        FeedbackSignalEmitOk::v1(
                            ReasonCodeId(102),
                            FeedbackValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftFeedbackEngine;

    impl Ph1FeedbackEngine for DriftFeedbackEngine {
        fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
            match req {
                Ph1FeedbackRequest::FeedbackEventCollect(r) => {
                    let candidate = selene_kernel_contracts::ph1feedback::FeedbackSignalCandidate::v1(
                        format!("signal_{}", r.events[0].event_id),
                        r.events[0].event_type,
                        "correction_rate_by_intent".to_string(),
                        selene_kernel_contracts::ph1feedback::FeedbackSignalTarget::LearnPackage,
                        900,
                        1,
                        r.events[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1FeedbackResponse::FeedbackEventCollectOk(
                        FeedbackEventCollectOk::v1(
                            ReasonCodeId(111),
                            candidate.candidate_id.clone(),
                            vec![candidate],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1FeedbackRequest::FeedbackSignalEmit(_r) => {
                    Ph1FeedbackResponse::FeedbackSignalEmitOk(
                        FeedbackSignalEmitOk::v1(
                            ReasonCodeId(112),
                            FeedbackValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_candidates".to_string()],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn event(event_id: &str, event_type: FeedbackEventType) -> FeedbackEventRecord {
        FeedbackEventRecord::v1(
            event_id.to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3301),
            TurnId(301),
            event_type,
            ReasonCodeId(10),
            "evidence:feedback:1".to_string(),
            "idem:feedback:1".to_string(),
            FeedbackMetrics::v1(
                400,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn gold_context() -> GoldCaseCaptureContext {
        GoldCaseCaptureContext::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3410),
            TurnId(410),
            "idem_gold_1".to_string(),
        )
        .unwrap()
    }

    fn ph1d_provider_failure_response() -> Ph1dProviderCallResponse {
        Ph1dProviderCallResponse::v1(
            3410,
            410,
            RequestId(88),
            "idem:provider:88".to_string(),
            Some("call_88".to_string()),
            "openai".to_string(),
            Ph1dProviderTask::SttTranscribe,
            "gpt_4o_mini_transcribe".to_string(),
            Ph1dProviderStatus::Error,
            780,
            2_100,
            Some(3_600),
            None,
            None,
            Ph1dProviderValidationStatus::SchemaFail,
            ReasonCodeId(0x4400_0202),
        )
        .unwrap()
    }

    #[test]
    fn at_feedback_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3301),
            TurnId(301),
            vec![
                event("event_1", FeedbackEventType::SttReject),
                event("event_2", FeedbackEventType::UserCorrection),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FeedbackWiringOutcome::Forwarded(bundle) => assert!(bundle.validate().is_ok()),
            _ => panic!("expected Forwarded outcome"),
        }
    }

    #[test]
    fn at_feedback_02_os_output_is_deterministic() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3302),
            TurnId(302),
            vec![
                event("event_1", FeedbackEventType::SttReject),
                event("event_2", FeedbackEventType::UserCorrection),
            ],
        )
        .unwrap();

        let out_1 = wiring.run_turn(&input).unwrap();
        let out_2 = wiring.run_turn(&input).unwrap();

        let selected_1 = match out_1 {
            FeedbackWiringOutcome::Forwarded(bundle) => bundle.event_collect.selected_candidate_id,
            _ => panic!("expected Forwarded outcome"),
        };
        let selected_2 = match out_2 {
            FeedbackWiringOutcome::Forwarded(bundle) => bundle.event_collect.selected_candidate_id,
            _ => panic!("expected Forwarded outcome"),
        };
        assert_eq!(selected_1, selected_2);
    }

    #[test]
    fn at_feedback_03_os_does_not_invoke_when_feedback_is_disabled() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(false),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3303),
            TurnId(303),
            vec![event("event_1", FeedbackEventType::SttReject)],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert!(matches!(out, FeedbackWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_feedback_04_os_fails_closed_on_signal_emit_validation_drift() {
        let wiring =
            Ph1FeedbackWiring::new(Ph1FeedbackWiringConfig::mvp_v1(true), DriftFeedbackEngine)
                .unwrap();
        let input = FeedbackTurnInput::v1(
            CorrelationId(3304),
            TurnId(304),
            vec![event("event_1", FeedbackEventType::SttReject)],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            FeedbackWiringOutcome::Refused(refuse) => assert_eq!(
                refuse.reason_code,
                reason_codes::PH1_FEEDBACK_VALIDATION_FAILED
            ),
            _ => panic!("expected Refused outcome"),
        }
    }

    #[test]
    fn at_feedback_05_mapper_builds_deterministic_failure_event() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let mut event = event("event_55", FeedbackEventType::ToolFail);
        event.correlation_id = CorrelationId(3355);
        event.turn_id = TurnId(355);
        let input =
            FeedbackTurnInput::v1(CorrelationId(3355), TurnId(355), vec![event.clone()]).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let FeedbackWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded feedback bundle");
        };
        let provider_context = FailureProviderContext::v1(
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            Ph1dProviderTask::OcrTextExtract,
            12_000,
            77,
            false,
        )
        .unwrap();

        let failure_a = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            true,
            Some("provider timeout".to_string()),
            Some(provider_context.clone()),
        )
        .unwrap();
        let failure_b = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            true,
            Some("provider timeout".to_string()),
            Some(provider_context),
        )
        .unwrap();
        assert_eq!(failure_a.fingerprint, failure_b.fingerprint);
        assert_eq!(failure_a.failure_id, event.event_id);
        assert!(failure_a.escalation_required);
        assert_eq!(
            failure_a.unresolved_reason.as_deref(),
            Some("provider timeout")
        );
    }

    #[test]
    fn at_feedback_06_mapper_fails_closed_on_correlation_mismatch() {
        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let mut event = event("event_56", FeedbackEventType::SttReject);
        event.correlation_id = CorrelationId(3356);
        event.turn_id = TurnId(356);
        let input =
            FeedbackTurnInput::v1(CorrelationId(3356), TurnId(356), vec![event.clone()]).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let FeedbackWiringOutcome::Forwarded(mut bundle) = out else {
            panic!("expected forwarded feedback bundle");
        };
        bundle.correlation_id = CorrelationId(9_999);

        let err = map_feedback_event_to_failure_event(
            &event,
            &bundle,
            FailureContainmentAction::FailClosedRefuse,
            false,
            None,
            None,
        )
        .expect_err("mapper should fail closed on correlation mismatch");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "map_feedback_event_to_failure_event.event.correlation_id"
                );
            }
            _ => panic!("expected invalid-value violation"),
        }
    }

    #[test]
    fn at_feedback_07_gold_case_capture_from_ph1c_reject_emits_pending_gold_with_fingerprints() {
        let context = gold_context();
        let response = Ph1cResponse::TranscriptReject(TranscriptReject::v1(
            ReasonCodeId(0x4300_0002),
            RetryAdvice::SpeakSlower,
        ));

        let capture = build_gold_case_capture_from_ph1c_response(
            &context,
            &response,
            Some("evidence:ph1c:reject"),
            None,
        )
        .unwrap()
        .expect("reject should emit gold-case capture");
        assert_eq!(capture.owner_engine, GoldCaseOwnerEngine::Ph1c);
        assert_eq!(
            capture.feedback_event.event_type,
            FeedbackEventType::SttReject
        );
        assert_eq!(
            capture.feedback_event.path_type,
            FeedbackPathType::Improvement
        );
        assert_eq!(
            capture.feedback_event.gold_status,
            FeedbackGoldStatus::Pending
        );
        assert_eq!(
            capture.feedback_event.gold_case_id.as_deref(),
            Some(capture.gold_case_id.as_str())
        );
        assert_eq!(capture.reason_code_chain, vec![ReasonCodeId(0x4300_0002)]);
        assert!(!capture.primary_failure_fingerprint.is_empty());
        assert!(!capture.secondary_failure_fingerprint.is_empty());
    }

    #[test]
    fn at_feedback_08_gold_case_capture_from_low_conf_transcript_ok_is_deterministic() {
        let context = gold_context();
        let response = Ph1cResponse::TranscriptOk(
            TranscriptOk::v1(
                "invoice total is one twenty three".to_string(),
                LanguageTag::new("en").unwrap(),
                ConfidenceBucket::Low,
            )
            .unwrap(),
        );

        let capture_a = build_gold_case_capture_from_ph1c_response(
            &context,
            &response,
            Some("evidence:ph1c:low_conf"),
            None,
        )
        .unwrap()
        .expect("low-confidence transcript should emit gold-case capture");
        let capture_b = build_gold_case_capture_from_ph1c_response(
            &context,
            &response,
            Some("evidence:ph1c:low_conf"),
            None,
        )
        .unwrap()
        .expect("low-confidence transcript should emit gold-case capture");

        assert_eq!(capture_a.gold_case_id, capture_b.gold_case_id);
        assert_eq!(
            capture_a.primary_failure_fingerprint,
            capture_b.primary_failure_fingerprint
        );
        assert_eq!(
            capture_a.secondary_failure_fingerprint,
            capture_b.secondary_failure_fingerprint
        );
        assert_eq!(
            capture_a.feedback_event.reason_code,
            reason_codes::PH1_FEEDBACK_GOLDCASE_STT_LOW_CONFIDENCE
        );
        assert_eq!(capture_a.language_locale.as_deref(), Some("en"));
        assert_eq!(
            capture_a.final_accepted_transcript.as_deref(),
            Some("invoice total is one twenty three")
        );
    }

    #[test]
    fn at_feedback_09_gold_case_capture_from_ph1d_failure_is_deterministic() {
        let context = gold_context();
        let response = ph1d_provider_failure_response();

        let capture_a = build_gold_case_capture_from_ph1d_response(
            &context,
            &response,
            Some("evidence:ph1d:provider_fail"),
            None,
            Some("en".to_string()),
        )
        .unwrap()
        .expect("provider failure should emit gold-case capture");
        let capture_b = build_gold_case_capture_from_ph1d_response(
            &context,
            &response,
            Some("evidence:ph1d:provider_fail"),
            None,
            Some("en".to_string()),
        )
        .unwrap()
        .expect("provider failure should emit gold-case capture");

        assert_eq!(capture_a.gold_case_id, capture_b.gold_case_id);
        assert_eq!(
            capture_a.feedback_event.event_type,
            FeedbackEventType::SttReject
        );
        assert_eq!(
            capture_a.feedback_event.reason_code,
            ReasonCodeId(0x4400_0202)
        );
        assert_eq!(capture_a.reason_code_chain, vec![ReasonCodeId(0x4400_0202)]);
        assert_eq!(capture_a.language_locale.as_deref(), Some("en"));
    }

    #[test]
    fn at_feedback_10_gold_case_capture_from_tts_failed_emits_tool_fail_event() {
        let context = gold_context();
        let event = Ph1ttsEvent::Failed(TtsFailed {
            schema_version: PH1TTS_CONTRACT_VERSION,
            answer_id: AnswerId(77),
            reason_code: ReasonCodeId(0x5454_0006),
        });

        let capture = build_gold_case_capture_from_ph1tts_event(
            &context,
            &event,
            Some("evidence:ph1tts:failed"),
            None,
            Some("en".to_string()),
        )
        .unwrap()
        .expect("tts_failed should emit gold-case capture");
        assert_eq!(capture.owner_engine, GoldCaseOwnerEngine::Ph1tts);
        assert_eq!(
            capture.feedback_event.event_type,
            FeedbackEventType::ToolFail
        );
        assert_eq!(
            capture.feedback_event.reason_code,
            ReasonCodeId(0x5454_0006)
        );
        assert_eq!(capture.reason_code_chain, vec![ReasonCodeId(0x5454_0006)]);
    }

    #[test]
    fn at_feedback_11_gold_case_capture_skips_high_confidence_ph1c_ok_without_uncertain_spans() {
        let context = gold_context();
        let response = Ph1cResponse::TranscriptOk(
            TranscriptOk::v1(
                "set a meeting for tomorrow".to_string(),
                LanguageTag::new("en").unwrap(),
                ConfidenceBucket::High,
            )
            .unwrap(),
        );

        let out = build_gold_case_capture_from_ph1c_response(
            &context,
            &response,
            Some("evidence:ph1c:high_conf"),
            None,
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_feedback_12_gold_case_creation_on_correction_and_escalation() {
        let context = gold_context();
        let corrected_transcript = "corrected invoice total is 123.45".to_string();
        let response = Ph1cResponse::TranscriptOk(
            TranscriptOk::v1(
                "invoice total is one twenty three".to_string(),
                LanguageTag::new("en").unwrap(),
                ConfidenceBucket::Low,
            )
            .unwrap(),
        );

        let capture = build_gold_case_capture_from_ph1c_response(
            &context,
            &response,
            Some("evidence:ph1c:correction_case"),
            Some(corrected_transcript.clone()),
        )
        .unwrap()
        .expect("correction path must emit gold-case capture");
        assert_eq!(capture.owner_engine, GoldCaseOwnerEngine::Ph1c);
        assert_eq!(
            capture.feedback_event.event_type,
            FeedbackEventType::SttRetry
        );
        assert_eq!(
            capture.final_accepted_transcript.as_deref(),
            Some(corrected_transcript.as_str())
        );

        let wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            DeterministicFeedbackEngine,
        )
        .unwrap();
        let feedback_input = FeedbackTurnInput::v1(
            capture.feedback_event.correlation_id,
            capture.feedback_event.turn_id,
            vec![capture.feedback_event.clone()],
        )
        .unwrap();
        let feedback_outcome = wiring.run_turn(&feedback_input).unwrap();
        let FeedbackWiringOutcome::Forwarded(feedback_bundle) = feedback_outcome else {
            panic!("expected forwarded feedback bundle for correction gold-case event");
        };

        let failure = map_feedback_event_to_failure_event(
            &capture.feedback_event,
            &feedback_bundle,
            FailureContainmentAction::Escalated,
            true,
            Some("human verification pending for correction-backed gold case".to_string()),
            None,
        )
        .unwrap();
        assert!(failure.escalation_required);
        assert_eq!(
            failure.unresolved_reason.as_deref(),
            Some("human verification pending for correction-backed gold case")
        );
    }
}
