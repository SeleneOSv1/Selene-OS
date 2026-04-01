#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::{Ph1VoiceIdResponse, UserId};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1m::{
    MemoryConsent, MemoryLayer, MemoryLedgerEvent, MemoryLedgerEventKind, MemorySensitivityFlag,
    MemoryUsePolicy, Ph1mContextBundleBuildRequest, Ph1mContextBundleBuildResponse,
    Ph1mEmotionalThreadUpdateRequest, Ph1mEmotionalThreadUpdateResponse, Ph1mForgetRequest,
    Ph1mForgetResponse, Ph1mGraphUpdateRequest, Ph1mGraphUpdateResponse,
    Ph1mHintBundleBuildRequest, Ph1mHintBundleBuildResponse, Ph1mMetricsEmitRequest,
    Ph1mMetricsEmitResponse, Ph1mProposeRequest, Ph1mProposeResponse, Ph1mRecallRequest,
    Ph1mRecallResponse, Ph1mResumeSelectRequest, Ph1mResumeSelectResponse,
    Ph1mRetentionModeSetRequest, Ph1mRetentionModeSetResponse, Ph1mSafeSummaryRequest,
    Ph1mSafeSummaryResponse, Ph1mSuppressionSetRequest, Ph1mSuppressionSetResponse,
    Ph1mThreadDigestUpsertRequest, Ph1mThreadDigestUpsertResponse,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{MemoryThreadEventKind, StorageError};
use selene_storage::repo::Ph1MRepo;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.M OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_M_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4D00_0101);
    pub const PH1_M_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4D00_01F1);
    pub const PH1_M_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4D00_01F2);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1mWiringConfig {
    pub memory_enabled: bool,
    pub max_proposals: u8,
    pub max_requested_keys: u8,
}

impl Ph1mWiringConfig {
    pub fn mvp_v1(memory_enabled: bool) -> Self {
        Self {
            memory_enabled,
            max_proposals: 16,
            max_requested_keys: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryOperation {
    Propose(Ph1mProposeRequest),
    Recall(Ph1mRecallRequest),
    Forget(Ph1mForgetRequest),
    HintBundleBuild(Ph1mHintBundleBuildRequest),
    ContextBundleBuild(Ph1mContextBundleBuildRequest),
    SuppressionSet(Ph1mSuppressionSetRequest),
    SafeSummary(Ph1mSafeSummaryRequest),
    EmotionalThreadUpdate(Ph1mEmotionalThreadUpdateRequest),
    MetricsEmit(Ph1mMetricsEmitRequest),
    GraphUpdate(Ph1mGraphUpdateRequest),
    RetentionModeSet(Ph1mRetentionModeSetRequest),
    ThreadDigestUpsert(Ph1mThreadDigestUpsertRequest),
    ResumeSelect(Ph1mResumeSelectRequest),
}

impl Validate for MemoryOperation {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            MemoryOperation::Propose(req) => req.validate(),
            MemoryOperation::Recall(req) => req.validate(),
            MemoryOperation::Forget(req) => req.validate(),
            MemoryOperation::HintBundleBuild(req) => req.validate(),
            MemoryOperation::ContextBundleBuild(req) => req.validate(),
            MemoryOperation::SuppressionSet(req) => req.validate(),
            MemoryOperation::SafeSummary(req) => req.validate(),
            MemoryOperation::EmotionalThreadUpdate(req) => req.validate(),
            MemoryOperation::MetricsEmit(req) => req.validate(),
            MemoryOperation::GraphUpdate(req) => req.validate(),
            MemoryOperation::RetentionModeSet(req) => req.validate(),
            MemoryOperation::ThreadDigestUpsert(req) => req.validate(),
            MemoryOperation::ResumeSelect(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub operation: MemoryOperation,
}

impl MemoryTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        operation: MemoryOperation,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            operation,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for MemoryTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.operation.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryTurnOutput {
    Propose(Ph1mProposeResponse),
    Recall(Ph1mRecallResponse),
    Forget(Ph1mForgetResponse),
    HintBundleBuild(Ph1mHintBundleBuildResponse),
    ContextBundleBuild(Ph1mContextBundleBuildResponse),
    SuppressionSet(Ph1mSuppressionSetResponse),
    SafeSummary(Ph1mSafeSummaryResponse),
    EmotionalThreadUpdate(Ph1mEmotionalThreadUpdateResponse),
    MetricsEmit(Ph1mMetricsEmitResponse),
    GraphUpdate(Ph1mGraphUpdateResponse),
    RetentionModeSet(Ph1mRetentionModeSetResponse),
    ThreadDigestUpsert(Ph1mThreadDigestUpsertResponse),
    ResumeSelect(Ph1mResumeSelectResponse),
}

impl Validate for MemoryTurnOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            MemoryTurnOutput::Propose(resp) => resp.validate(),
            MemoryTurnOutput::Recall(resp) => resp.validate(),
            MemoryTurnOutput::Forget(resp) => resp.validate(),
            MemoryTurnOutput::HintBundleBuild(resp) => resp.validate(),
            MemoryTurnOutput::ContextBundleBuild(resp) => resp.validate(),
            MemoryTurnOutput::SuppressionSet(resp) => resp.validate(),
            MemoryTurnOutput::SafeSummary(resp) => resp.validate(),
            MemoryTurnOutput::EmotionalThreadUpdate(resp) => resp.validate(),
            MemoryTurnOutput::MetricsEmit(resp) => resp.validate(),
            MemoryTurnOutput::GraphUpdate(resp) => resp.validate(),
            MemoryTurnOutput::RetentionModeSet(resp) => resp.validate(),
            MemoryTurnOutput::ThreadDigestUpsert(resp) => resp.validate(),
            MemoryTurnOutput::ResumeSelect(resp) => resp.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub output: MemoryTurnOutput,
}

impl MemoryForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        output: MemoryTurnOutput,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            output,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for MemoryForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.output.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryWiringRefuse {
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl MemoryWiringRefuse {
    pub fn v1(reason_code: ReasonCodeId, message: String) -> Result<Self, ContractViolation> {
        let refuse = Self {
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for MemoryWiringRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.message.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_wiring_refuse.message",
                reason: "must be non-empty",
            });
        }
        if self.message.len() > 192 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_wiring_refuse.message",
                reason: "must be <= 192 chars",
            });
        }
        if !self.message.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_wiring_refuse.message",
                reason: "must be ASCII",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MemoryWiringOutcome {
    NotInvokedDisabled,
    Refused(MemoryWiringRefuse),
    Forwarded(MemoryForwardBundle),
}

fn storage_contract_error(field: &'static str, reason: &'static str) -> StorageError {
    StorageError::ContractViolation(ContractViolation::InvalidValue { field, reason })
}

fn user_id_from_assertion(assertion: &Ph1VoiceIdResponse) -> Result<UserId, StorageError> {
    match assertion {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.user_id.clone().ok_or_else(|| {
            storage_contract_error(
                "ph1m.persistence.user_id",
                "speaker_assertion_ok.user_id must be present",
            )
        }),
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(_) => Err(storage_contract_error(
            "ph1m.persistence.user_id",
            "speaker assertion must be OK for persistence",
        )),
    }
}

fn operation_user_id(operation: &MemoryOperation) -> Result<UserId, StorageError> {
    match operation {
        MemoryOperation::Propose(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::Recall(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::Forget(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::HintBundleBuild(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::ContextBundleBuild(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::SuppressionSet(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::SafeSummary(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::EmotionalThreadUpdate(req) => {
            user_id_from_assertion(&req.speaker_assertion)
        }
        MemoryOperation::MetricsEmit(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::GraphUpdate(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::RetentionModeSet(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::ThreadDigestUpsert(req) => user_id_from_assertion(&req.speaker_assertion),
        MemoryOperation::ResumeSelect(req) => user_id_from_assertion(&req.speaker_assertion),
    }
}

const DEFAULT_MICRO_TTL_MS: u64 = 30 * 24 * 60 * 60 * 1000;

fn derive_use_policy_from_ledger_event(event: &MemoryLedgerEvent) -> MemoryUsePolicy {
    match event.kind {
        MemoryLedgerEventKind::Forgotten => MemoryUsePolicy::UserRequestedOnly,
        MemoryLedgerEventKind::Stored | MemoryLedgerEventKind::Updated => {
            if event.sensitivity_flag == MemorySensitivityFlag::Sensitive {
                return MemoryUsePolicy::ContextRelevantOnly;
            }
            match event.consent {
                MemoryConsent::ExplicitRemember | MemoryConsent::Confirmed => {
                    MemoryUsePolicy::AlwaysUsable
                }
                MemoryConsent::Denied | MemoryConsent::NotRequested => match event.layer {
                    MemoryLayer::Micro => MemoryUsePolicy::RepeatedOrConfirmed,
                    MemoryLayer::Working | MemoryLayer::LongTerm => MemoryUsePolicy::AlwaysUsable,
                },
            }
        }
    }
}

fn derive_expires_at_from_ledger_event(event: &MemoryLedgerEvent) -> Option<MonotonicTimeNs> {
    match event.kind {
        MemoryLedgerEventKind::Forgotten => None,
        MemoryLedgerEventKind::Stored | MemoryLedgerEventKind::Updated => match event.layer {
            MemoryLayer::Micro => {
                Some(MonotonicTimeNs(event.t_event.0.saturating_add(
                    DEFAULT_MICRO_TTL_MS.saturating_mul(1_000_000),
                )))
            }
            MemoryLayer::Working | MemoryLayer::LongTerm => None,
        },
    }
}

/// Persists PH1.M forwarded write outcomes to PH1.F via `Ph1MRepo`.
///
/// Returns:
/// - `Ok(true)` when a write was persisted,
/// - `Ok(false)` when the outcome is non-forwarded or non-writing,
/// - `Err(StorageError)` on contract mismatch or storage failure.
pub fn persist_memory_forwarded_outcome<R: Ph1MRepo>(
    repo: &mut R,
    input: &MemoryTurnInput,
    outcome: &MemoryWiringOutcome,
) -> Result<bool, StorageError> {
    input.validate().map_err(StorageError::ContractViolation)?;
    let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
        return Ok(false);
    };
    bundle.validate().map_err(StorageError::ContractViolation)?;
    if bundle.correlation_id != input.correlation_id || bundle.turn_id != input.turn_id {
        return Err(storage_contract_error(
            "ph1m.persistence.bundle",
            "forwarded bundle correlation/turn must match input",
        ));
    }

    let user_id = operation_user_id(&input.operation)?;

    match (&input.operation, &bundle.output) {
        (MemoryOperation::Propose(_), MemoryTurnOutput::Propose(resp)) => {
            if resp.ledger_events.is_empty() {
                return Ok(false);
            }
            for (idx, event) in resp.ledger_events.iter().enumerate() {
                let use_policy = derive_use_policy_from_ledger_event(event);
                let expires_at = derive_expires_at_from_ledger_event(event);
                let idempotency_key = Some(format!(
                    "ph1m_propose:{}:{}:{idx}",
                    input.correlation_id.0, input.turn_id.0
                ));
                repo.ph1m_append_ledger_row(
                    &user_id,
                    event.clone(),
                    use_policy,
                    expires_at,
                    idempotency_key,
                )?;
            }
            Ok(true)
        }
        (MemoryOperation::Recall(_), MemoryTurnOutput::Recall(_))
        | (MemoryOperation::HintBundleBuild(_), MemoryTurnOutput::HintBundleBuild(_))
        | (MemoryOperation::ContextBundleBuild(_), MemoryTurnOutput::ContextBundleBuild(_))
        | (MemoryOperation::SafeSummary(_), MemoryTurnOutput::SafeSummary(_))
        | (MemoryOperation::ResumeSelect(_), MemoryTurnOutput::ResumeSelect(_)) => Ok(false),
        (MemoryOperation::Forget(_), MemoryTurnOutput::Forget(resp)) => {
            if !resp.forgotten {
                return Ok(false);
            }
            let Some(event) = &resp.ledger_event else {
                return Ok(false);
            };
            let use_policy = derive_use_policy_from_ledger_event(event);
            let expires_at = derive_expires_at_from_ledger_event(event);
            let idempotency_key = Some(format!(
                "ph1m_forget:{}:{}",
                input.correlation_id.0, input.turn_id.0
            ));
            repo.ph1m_append_ledger_row(
                &user_id,
                event.clone(),
                use_policy,
                expires_at,
                idempotency_key,
            )?;
            Ok(true)
        }
        (MemoryOperation::SuppressionSet(req), MemoryTurnOutput::SuppressionSet(resp)) => {
            if !resp.applied {
                return Ok(false);
            }
            repo.ph1m_set_suppression_rule_row(
                &user_id,
                resp.rule.clone(),
                req.now,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        (
            MemoryOperation::EmotionalThreadUpdate(req),
            MemoryTurnOutput::EmotionalThreadUpdate(resp),
        ) => {
            repo.ph1m_emotional_thread_update_commit_row(
                &user_id,
                resp.state.clone(),
                resp.reason_code,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        (MemoryOperation::MetricsEmit(req), MemoryTurnOutput::MetricsEmit(resp)) => {
            if !resp.emitted {
                return Ok(false);
            }
            repo.ph1m_metrics_emit_commit_row(
                &user_id,
                req.payload.clone(),
                resp.reason_code,
                req.now,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        (MemoryOperation::GraphUpdate(req), MemoryTurnOutput::GraphUpdate(_)) => {
            repo.ph1m_graph_upsert_commit_row(
                &user_id,
                req.nodes.clone(),
                req.edges.clone(),
                req.now,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        (MemoryOperation::RetentionModeSet(req), MemoryTurnOutput::RetentionModeSet(resp)) => {
            repo.ph1m_retention_mode_set_commit_row(
                &user_id,
                resp.memory_retention_mode,
                resp.effective_at,
                resp.reason_code,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        (MemoryOperation::ThreadDigestUpsert(req), MemoryTurnOutput::ThreadDigestUpsert(resp)) => {
            if req.thread_digest.thread_id != resp.thread_id {
                return Err(storage_contract_error(
                    "ph1m.persistence.thread_id",
                    "request thread_id must match response thread_id",
                ));
            }
            repo.ph1m_thread_digest_upsert_commit_row(
                &user_id,
                req.memory_retention_mode,
                req.thread_digest.clone(),
                MemoryThreadEventKind::ThreadDigestUpsert,
                resp.reason_code,
                req.idempotency_key.clone(),
            )?;
            Ok(true)
        }
        _ => Err(storage_contract_error(
            "ph1m.persistence.bundle.output",
            "operation and forwarded output variant mismatch",
        )),
    }
}

pub trait Ph1MemoryEngine {
    fn propose(
        &mut self,
        req: &Ph1mProposeRequest,
    ) -> Result<Ph1mProposeResponse, ContractViolation>;
    fn recall(&mut self, req: &Ph1mRecallRequest) -> Result<Ph1mRecallResponse, ContractViolation>;
    fn forget(&mut self, req: &Ph1mForgetRequest) -> Result<Ph1mForgetResponse, ContractViolation>;
    fn hint_bundle_build(
        &mut self,
        req: &Ph1mHintBundleBuildRequest,
    ) -> Result<Ph1mHintBundleBuildResponse, ContractViolation>;
    fn context_bundle_build(
        &mut self,
        req: &Ph1mContextBundleBuildRequest,
    ) -> Result<Ph1mContextBundleBuildResponse, ContractViolation>;
    fn suppression_set(
        &mut self,
        req: &Ph1mSuppressionSetRequest,
    ) -> Result<Ph1mSuppressionSetResponse, ContractViolation>;
    fn safe_summary(
        &mut self,
        req: &Ph1mSafeSummaryRequest,
    ) -> Result<Ph1mSafeSummaryResponse, ContractViolation>;
    fn emotional_thread_update(
        &mut self,
        req: &Ph1mEmotionalThreadUpdateRequest,
    ) -> Result<Ph1mEmotionalThreadUpdateResponse, ContractViolation>;
    fn metrics_emit(
        &mut self,
        req: &Ph1mMetricsEmitRequest,
    ) -> Result<Ph1mMetricsEmitResponse, ContractViolation>;
    fn graph_update(
        &mut self,
        req: &Ph1mGraphUpdateRequest,
    ) -> Result<Ph1mGraphUpdateResponse, ContractViolation>;
    fn retention_mode_set(
        &mut self,
        req: &Ph1mRetentionModeSetRequest,
    ) -> Result<Ph1mRetentionModeSetResponse, ContractViolation>;
    fn thread_digest_upsert(
        &mut self,
        req: &Ph1mThreadDigestUpsertRequest,
    ) -> Result<Ph1mThreadDigestUpsertResponse, ContractViolation>;
    fn resume_select(
        &mut self,
        req: &Ph1mResumeSelectRequest,
    ) -> Result<Ph1mResumeSelectResponse, ContractViolation>;
}

impl Ph1MemoryEngine for selene_engines::ph1m::Ph1mRuntime {
    fn propose(
        &mut self,
        req: &Ph1mProposeRequest,
    ) -> Result<Ph1mProposeResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::propose(self, req)
    }

    fn recall(&mut self, req: &Ph1mRecallRequest) -> Result<Ph1mRecallResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::recall(self, req)
    }

    fn forget(&mut self, req: &Ph1mForgetRequest) -> Result<Ph1mForgetResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::forget(self, req)
    }

    fn hint_bundle_build(
        &mut self,
        req: &Ph1mHintBundleBuildRequest,
    ) -> Result<Ph1mHintBundleBuildResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::hint_bundle_build(self, req)
    }

    fn context_bundle_build(
        &mut self,
        req: &Ph1mContextBundleBuildRequest,
    ) -> Result<Ph1mContextBundleBuildResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::context_bundle_build(self, req)
    }

    fn suppression_set(
        &mut self,
        req: &Ph1mSuppressionSetRequest,
    ) -> Result<Ph1mSuppressionSetResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::suppression_set(self, req)
    }

    fn safe_summary(
        &mut self,
        req: &Ph1mSafeSummaryRequest,
    ) -> Result<Ph1mSafeSummaryResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::safe_summary(self, req)
    }

    fn emotional_thread_update(
        &mut self,
        req: &Ph1mEmotionalThreadUpdateRequest,
    ) -> Result<Ph1mEmotionalThreadUpdateResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::emotional_thread_update(self, req)
    }

    fn metrics_emit(
        &mut self,
        req: &Ph1mMetricsEmitRequest,
    ) -> Result<Ph1mMetricsEmitResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::metrics_emit(self, req)
    }

    fn graph_update(
        &mut self,
        req: &Ph1mGraphUpdateRequest,
    ) -> Result<Ph1mGraphUpdateResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::graph_update(self, req)
    }

    fn retention_mode_set(
        &mut self,
        req: &Ph1mRetentionModeSetRequest,
    ) -> Result<Ph1mRetentionModeSetResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::retention_mode_set(self, req)
    }

    fn thread_digest_upsert(
        &mut self,
        req: &Ph1mThreadDigestUpsertRequest,
    ) -> Result<Ph1mThreadDigestUpsertResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::thread_digest_upsert(self, req)
    }

    fn resume_select(
        &mut self,
        req: &Ph1mResumeSelectRequest,
    ) -> Result<Ph1mResumeSelectResponse, ContractViolation> {
        selene_engines::ph1m::Ph1mRuntime::resume_select(self, req)
    }
}

#[derive(Debug, Clone)]
pub struct Ph1mWiring<E>
where
    E: Ph1MemoryEngine,
{
    config: Ph1mWiringConfig,
    engine: E,
}

impl<E> Ph1mWiring<E>
where
    E: Ph1MemoryEngine,
{
    pub fn new(config: Ph1mWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_proposals == 0 || config.max_proposals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_wiring_config.max_proposals",
                reason: "must be within 1..=32",
            });
        }
        if config.max_requested_keys == 0 || config.max_requested_keys > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_wiring_config.max_requested_keys",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &mut self,
        input: &MemoryTurnInput,
    ) -> Result<MemoryWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.memory_enabled {
            return Ok(MemoryWiringOutcome::NotInvokedDisabled);
        }

        match &input.operation {
            MemoryOperation::Propose(req) => {
                if req.proposals.len() > self.config.max_proposals as usize {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_BUDGET_EXCEEDED,
                        "memory proposal budget exceeded".to_string(),
                    )?));
                }
                let resp = match self.engine.propose(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory propose pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory propose response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::Propose(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::Recall(req) => {
                if req.requested_keys.len() > self.config.max_requested_keys as usize {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_BUDGET_EXCEEDED,
                        "memory recall key budget exceeded".to_string(),
                    )?));
                }
                let resp = match self.engine.recall(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory recall pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory recall response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::Recall(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::Forget(req) => {
                let resp = match self.engine.forget(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory forget pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory forget response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::Forget(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::HintBundleBuild(req) => {
                let resp = match self.engine.hint_bundle_build(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory hint bundle pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory hint bundle response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::HintBundleBuild(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::ContextBundleBuild(req) => {
                let resp = match self.engine.context_bundle_build(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory context bundle pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory context bundle response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::ContextBundleBuild(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::SuppressionSet(req) => {
                let resp = match self.engine.suppression_set(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory suppression set pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory suppression set response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::SuppressionSet(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::SafeSummary(req) => {
                let resp = match self.engine.safe_summary(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory safe summary pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory safe summary response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::SafeSummary(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::EmotionalThreadUpdate(req) => {
                let resp = match self.engine.emotional_thread_update(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory emotional thread update pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory emotional thread update response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::EmotionalThreadUpdate(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::MetricsEmit(req) => {
                let resp = match self.engine.metrics_emit(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory metrics emit pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory metrics emit response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::MetricsEmit(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::GraphUpdate(req) => {
                let resp = match self.engine.graph_update(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory graph update pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory graph update response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::GraphUpdate(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::RetentionModeSet(req) => {
                let resp = match self.engine.retention_mode_set(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory retention mode set pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory retention mode set response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::RetentionModeSet(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::ThreadDigestUpsert(req) => {
                let resp = match self.engine.thread_digest_upsert(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory thread digest upsert pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory thread digest upsert response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::ThreadDigestUpsert(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
            MemoryOperation::ResumeSelect(req) => {
                let resp = match self.engine.resume_select(req) {
                    Ok(resp) => resp,
                    Err(_) => {
                        return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR,
                            "memory resume select pipeline failed".to_string(),
                        )?));
                    }
                };
                if resp.validate().is_err() {
                    return Ok(MemoryWiringOutcome::Refused(MemoryWiringRefuse::v1(
                        reason_codes::PH1_M_VALIDATION_FAILED,
                        "invalid memory resume select response contract".to_string(),
                    )?));
                }
                let bundle = MemoryForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    MemoryTurnOutput::ResumeSelect(resp),
                )?;
                Ok(MemoryWiringOutcome::Forwarded(bundle))
            }
        }
    }

    pub fn run_turn_and_persist<R: Ph1MRepo>(
        &mut self,
        repo: &mut R,
        input: &MemoryTurnInput,
    ) -> Result<MemoryWiringOutcome, StorageError> {
        let outcome = self
            .run_turn(input)
            .map_err(StorageError::ContractViolation)?;
        let _ = persist_memory_forwarded_outcome(repo, input, &outcome)?;
        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_engines::ph1m::{Ph1mConfig, Ph1mRuntime};
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, SpeakerAssertionOk, SpeakerAssertionUnknown,
        SpeakerId, SpeakerLabel, UserId,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1m::{
        MemoryCommitDecision, MemoryCommitStatus, MemoryConfidence, MemoryConsent,
        MemoryContextFact, MemoryEmotionalThreadState, MemoryItemTag, MemoryKey, MemoryLayer,
        MemoryLedgerEvent, MemoryLedgerEventKind, MemoryMetricPayload, MemoryProposedItem,
        MemoryProvenance, MemoryProvenanceTier, MemoryResumeAction, MemoryResumeTier,
        MemoryRetentionMode, MemorySensitivityFlag, MemorySuppressionRule,
        MemorySuppressionRuleKind, MemorySuppressionTargetType, MemoryThreadDigest,
        MemoryUsePolicy, MemoryValue, Ph1mContextBundleBuildRequest,
        Ph1mContextBundleBuildResponse, Ph1mEmotionalThreadUpdateRequest,
        Ph1mEmotionalThreadUpdateResponse, Ph1mGraphUpdateRequest, Ph1mGraphUpdateResponse,
        Ph1mHintBundleBuildRequest, Ph1mHintBundleBuildResponse, Ph1mMetricsEmitRequest,
        Ph1mMetricsEmitResponse, Ph1mRetentionModeSetRequest, Ph1mRetentionModeSetResponse,
        Ph1mSafeSummaryRequest, Ph1mSafeSummaryResponse, Ph1mSuppressionSetRequest,
        Ph1mSuppressionSetResponse,
    };
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};
    use selene_storage::ph1f::{IdentityRecord, IdentityStatus, Ph1fStore};
    use selene_storage::repo::Ph1fFoundationRepo;

    #[derive(Debug, Clone)]
    struct MockMemoryEngine {
        propose_response: Result<Ph1mProposeResponse, ContractViolation>,
        recall_response: Result<Ph1mRecallResponse, ContractViolation>,
        forget_response: Result<Ph1mForgetResponse, ContractViolation>,
        hint_bundle_build_response: Result<Ph1mHintBundleBuildResponse, ContractViolation>,
        context_bundle_build_response: Result<Ph1mContextBundleBuildResponse, ContractViolation>,
        suppression_set_response: Result<Ph1mSuppressionSetResponse, ContractViolation>,
        safe_summary_response: Result<Ph1mSafeSummaryResponse, ContractViolation>,
        emotional_thread_update_response:
            Result<Ph1mEmotionalThreadUpdateResponse, ContractViolation>,
        metrics_emit_response: Result<Ph1mMetricsEmitResponse, ContractViolation>,
        graph_update_response: Result<Ph1mGraphUpdateResponse, ContractViolation>,
        retention_mode_set_response: Result<Ph1mRetentionModeSetResponse, ContractViolation>,
        thread_digest_upsert_response: Result<Ph1mThreadDigestUpsertResponse, ContractViolation>,
        resume_select_response: Result<Ph1mResumeSelectResponse, ContractViolation>,
    }

    impl Ph1MemoryEngine for MockMemoryEngine {
        fn propose(
            &mut self,
            _req: &Ph1mProposeRequest,
        ) -> Result<Ph1mProposeResponse, ContractViolation> {
            self.propose_response.clone()
        }

        fn recall(
            &mut self,
            _req: &Ph1mRecallRequest,
        ) -> Result<Ph1mRecallResponse, ContractViolation> {
            self.recall_response.clone()
        }

        fn forget(
            &mut self,
            _req: &Ph1mForgetRequest,
        ) -> Result<Ph1mForgetResponse, ContractViolation> {
            self.forget_response.clone()
        }

        fn hint_bundle_build(
            &mut self,
            _req: &Ph1mHintBundleBuildRequest,
        ) -> Result<Ph1mHintBundleBuildResponse, ContractViolation> {
            self.hint_bundle_build_response.clone()
        }

        fn context_bundle_build(
            &mut self,
            _req: &Ph1mContextBundleBuildRequest,
        ) -> Result<Ph1mContextBundleBuildResponse, ContractViolation> {
            self.context_bundle_build_response.clone()
        }

        fn suppression_set(
            &mut self,
            _req: &Ph1mSuppressionSetRequest,
        ) -> Result<Ph1mSuppressionSetResponse, ContractViolation> {
            self.suppression_set_response.clone()
        }

        fn safe_summary(
            &mut self,
            _req: &Ph1mSafeSummaryRequest,
        ) -> Result<Ph1mSafeSummaryResponse, ContractViolation> {
            self.safe_summary_response.clone()
        }

        fn emotional_thread_update(
            &mut self,
            _req: &Ph1mEmotionalThreadUpdateRequest,
        ) -> Result<Ph1mEmotionalThreadUpdateResponse, ContractViolation> {
            self.emotional_thread_update_response.clone()
        }

        fn metrics_emit(
            &mut self,
            _req: &Ph1mMetricsEmitRequest,
        ) -> Result<Ph1mMetricsEmitResponse, ContractViolation> {
            self.metrics_emit_response.clone()
        }

        fn graph_update(
            &mut self,
            _req: &Ph1mGraphUpdateRequest,
        ) -> Result<Ph1mGraphUpdateResponse, ContractViolation> {
            self.graph_update_response.clone()
        }

        fn retention_mode_set(
            &mut self,
            _req: &Ph1mRetentionModeSetRequest,
        ) -> Result<Ph1mRetentionModeSetResponse, ContractViolation> {
            self.retention_mode_set_response.clone()
        }

        fn thread_digest_upsert(
            &mut self,
            _req: &Ph1mThreadDigestUpsertRequest,
        ) -> Result<Ph1mThreadDigestUpsertResponse, ContractViolation> {
            self.thread_digest_upsert_response.clone()
        }

        fn resume_select(
            &mut self,
            _req: &Ph1mResumeSelectRequest,
        ) -> Result<Ph1mResumeSelectResponse, ContractViolation> {
            self.resume_select_response.clone()
        }
    }

    fn speaker_ok() -> selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse {
        selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1(
                SpeakerId::new("spk").unwrap(),
                Some(UserId::new("user").unwrap()),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(0),
                    MonotonicTimeNs(1),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                SpeakerLabel::speaker_a(),
            )
            .unwrap(),
        )
    }

    fn speaker_unknown() -> selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse {
        selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1(IdentityConfidence::Medium, ReasonCodeId(1), vec![])
                .unwrap(),
        )
    }

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn policy_privacy_mode() -> PolicyContextRef {
        PolicyContextRef::v1(true, false, SafetyTier::Standard)
    }

    fn base_propose_request() -> Ph1mProposeRequest {
        let item = proposed_item_with_confidence(
            "preferred_name",
            "John",
            MemoryConfidence::High,
            "My name is John",
        );
        Ph1mProposeRequest::v1(MonotonicTimeNs(10), speaker_ok(), policy_ok(), vec![item]).unwrap()
    }

    fn proposed_item_with_confidence(
        key: &str,
        value: &str,
        confidence: MemoryConfidence,
        evidence_quote: &str,
    ) -> MemoryProposedItem {
        let item = MemoryProposedItem::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            confidence,
            MemoryConsent::NotRequested,
            evidence_quote.to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap();
        item
    }

    fn proposed_micro_item_with_confidence(
        key: &str,
        value: &str,
        confidence: MemoryConfidence,
        evidence_quote: &str,
    ) -> MemoryProposedItem {
        MemoryProposedItem::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryLayer::Micro,
            MemorySensitivityFlag::Low,
            confidence,
            MemoryConsent::NotRequested,
            evidence_quote.to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
        )
        .unwrap()
    }

    fn propose_request_at(
        now: MonotonicTimeNs,
        proposals: Vec<MemoryProposedItem>,
    ) -> Ph1mProposeRequest {
        Ph1mProposeRequest::v1(now, speaker_ok(), policy_ok(), proposals).unwrap()
    }

    fn base_recall_request() -> Ph1mRecallRequest {
        Ph1mRecallRequest::v1(
            MonotonicTimeNs(11),
            speaker_ok(),
            policy_ok(),
            vec![MemoryKey::new("preferred_name").unwrap()],
            true,
            8,
        )
        .unwrap()
    }

    fn base_forget_request() -> Ph1mForgetRequest {
        Ph1mForgetRequest::v1(
            MonotonicTimeNs(12),
            speaker_ok(),
            policy_ok(),
            MemoryKey::new("preferred_name").unwrap(),
        )
        .unwrap()
    }

    fn base_thread_digest_upsert_request() -> Ph1mThreadDigestUpsertRequest {
        let digest = MemoryThreadDigest::v1(
            "thread_japan_trip".to_string(),
            "Japan ski trip".to_string(),
            vec![
                "Flights shortlisted".to_string(),
                "Need hotel confirmation".to_string(),
            ],
            false,
            true,
            MonotonicTimeNs(13),
            1,
        )
        .unwrap();
        Ph1mThreadDigestUpsertRequest::v1(
            MonotonicTimeNs(13),
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            digest,
            "idem_thread".to_string(),
        )
        .unwrap()
    }

    fn base_resume_select_request() -> Ph1mResumeSelectRequest {
        Ph1mResumeSelectRequest::v1(
            MonotonicTimeNs(14),
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            true,
            true,
            true,
            false,
            3,
            None,
        )
        .unwrap()
    }

    fn ms_to_ns(ms: u64) -> u64 {
        ms.saturating_mul(1_000_000)
    }

    fn thread_digest_upsert_request_at(
        now: MonotonicTimeNs,
        last_updated_at: MonotonicTimeNs,
        thread_id: &str,
        thread_title: &str,
        summary_bullets: Vec<String>,
        unresolved: bool,
        pinned: bool,
        revision: u32,
        idempotency_key: &str,
    ) -> Ph1mThreadDigestUpsertRequest {
        let digest = MemoryThreadDigest::v1(
            thread_id.to_string(),
            thread_title.to_string(),
            summary_bullets,
            unresolved,
            pinned,
            last_updated_at,
            revision,
        )
        .unwrap();
        Ph1mThreadDigestUpsertRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            digest,
            idempotency_key.to_string(),
        )
        .unwrap()
    }

    fn thread_digest_upsert_request_at_with_contract_flags(
        now: MonotonicTimeNs,
        last_updated_at: MonotonicTimeNs,
        thread_id: &str,
        thread_title: &str,
        summary_bullets: Vec<String>,
        pinned: bool,
        unresolved: bool,
        use_count: u32,
        idempotency_key: &str,
    ) -> Ph1mThreadDigestUpsertRequest {
        let digest = MemoryThreadDigest::v1(
            thread_id.to_string(),
            thread_title.to_string(),
            summary_bullets,
            pinned,
            unresolved,
            last_updated_at,
            use_count,
        )
        .unwrap();
        Ph1mThreadDigestUpsertRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            digest,
            idempotency_key.to_string(),
        )
        .unwrap()
    }

    fn resume_select_request_at(now: MonotonicTimeNs) -> Ph1mResumeSelectRequest {
        Ph1mResumeSelectRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::Default,
            true,
            true,
            true,
            false,
            3,
            None,
        )
        .unwrap()
    }

    fn real_runtime_wiring() -> (Ph1mWiring<Ph1mRuntime>, Ph1mConfig) {
        let cfg = Ph1mConfig::mvp_v1();
        let wiring = Ph1mWiring::new(
            Ph1mWiringConfig::mvp_v1(true),
            Ph1mRuntime::new(Ph1mConfig::mvp_v1()),
        )
        .unwrap();
        (wiring, cfg)
    }

    fn forwarded_resume_select_response(outcome: MemoryWiringOutcome) -> Ph1mResumeSelectResponse {
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::ResumeSelect(resp) => resp,
                _ => panic!("expected resume select output"),
            },
            _ => panic!("expected forwarded resume select"),
        }
    }

    fn forwarded_context_bundle_response(
        outcome: MemoryWiringOutcome,
    ) -> Ph1mContextBundleBuildResponse {
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::ContextBundleBuild(resp) => resp,
                _ => panic!("expected context bundle output"),
            },
            _ => panic!("expected forwarded context bundle"),
        }
    }

    fn base_hint_bundle_request() -> Ph1mHintBundleBuildRequest {
        Ph1mHintBundleBuildRequest::v1(MonotonicTimeNs(18), speaker_ok(), policy_ok(), 8).unwrap()
    }

    fn base_context_bundle_request() -> Ph1mContextBundleBuildRequest {
        Ph1mContextBundleBuildRequest::v1(
            MonotonicTimeNs(19),
            speaker_ok(),
            policy_ok(),
            vec![MemoryKey::new("preferred_name").unwrap()],
            vec![MemoryContextFact::v1(
                MemoryKey::new("preferred_name").unwrap(),
                MemoryValue::v1("John".to_string(), None).unwrap(),
            )
            .unwrap()],
            Some("John".to_string()),
            Some("thread_japan_trip".to_string()),
            Some("wo_1".to_string()),
            true,
            1024,
            8,
            1,
        )
        .unwrap()
    }

    fn context_bundle_request_at(
        now: MonotonicTimeNs,
        requested_keys: Vec<MemoryKey>,
    ) -> Ph1mContextBundleBuildRequest {
        context_bundle_request_with_state_at(now, requested_keys, vec![])
    }

    fn context_bundle_request_with_state_at(
        now: MonotonicTimeNs,
        requested_keys: Vec<MemoryKey>,
        current_state_facts: Vec<MemoryContextFact>,
    ) -> Ph1mContextBundleBuildRequest {
        Ph1mContextBundleBuildRequest::v1(
            now,
            speaker_ok(),
            policy_ok(),
            requested_keys,
            current_state_facts,
            None,
            None,
            None,
            true,
            1024,
            8,
            0,
        )
        .unwrap()
    }

    fn base_suppression_set_request() -> Ph1mSuppressionSetRequest {
        Ph1mSuppressionSetRequest::v1(
            MonotonicTimeNs(20),
            speaker_ok(),
            policy_ok(),
            base_suppression_rule(),
            "idem_sup".to_string(),
        )
        .unwrap()
    }

    fn base_safe_summary_request() -> Ph1mSafeSummaryRequest {
        Ph1mSafeSummaryRequest::v1(MonotonicTimeNs(21), speaker_ok(), policy_ok(), 5, 512).unwrap()
    }

    fn base_emotional_thread_update_request() -> Ph1mEmotionalThreadUpdateRequest {
        Ph1mEmotionalThreadUpdateRequest::v1(
            MonotonicTimeNs(22),
            speaker_ok(),
            policy_ok(),
            MemoryEmotionalThreadState::v1(
                "tone".to_string(),
                vec!["calm".to_string()],
                Some("Tone continuity".to_string()),
                MonotonicTimeNs(22),
            )
            .unwrap(),
            "idem_emo".to_string(),
        )
        .unwrap()
    }

    fn base_metrics_emit_request() -> Ph1mMetricsEmitRequest {
        Ph1mMetricsEmitRequest::v1(
            MonotonicTimeNs(23),
            speaker_ok(),
            policy_ok(),
            base_metric_payload(),
            "idem_metrics".to_string(),
        )
        .unwrap()
    }

    fn base_graph_update_request() -> Ph1mGraphUpdateRequest {
        Ph1mGraphUpdateRequest::v1(
            MonotonicTimeNs(24),
            speaker_ok(),
            policy_ok(),
            vec![],
            vec![],
            "idem_graph".to_string(),
        )
        .unwrap()
    }

    fn base_retention_mode_set_request() -> Ph1mRetentionModeSetRequest {
        Ph1mRetentionModeSetRequest::v1(
            MonotonicTimeNs(25),
            speaker_ok(),
            policy_ok(),
            MemoryRetentionMode::RememberEverything,
            "idem_ret".to_string(),
        )
        .unwrap()
    }

    fn base_suppression_rule() -> MemorySuppressionRule {
        MemorySuppressionRule::v1(
            MemorySuppressionTargetType::TopicKey,
            "preferred_name".to_string(),
            MemorySuppressionRuleKind::DoNotMention,
            true,
            ReasonCodeId(0x4D00_0010),
            MonotonicTimeNs(15),
        )
        .unwrap()
    }

    fn base_metric_payload() -> MemoryMetricPayload {
        MemoryMetricPayload::v1(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).unwrap()
    }

    fn seeded_store_for_known_user() -> Ph1fStore {
        let mut store = Ph1fStore::new_in_memory();
        store
            .insert_identity_row(IdentityRecord::v1(
                UserId::new("user").unwrap(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
    }

    fn wiring(enabled: bool) -> Ph1mWiring<MockMemoryEngine> {
        Ph1mWiring::new(
            Ph1mWiringConfig::mvp_v1(enabled),
            MockMemoryEngine {
                propose_response: Ok(Ph1mProposeResponse::v1(vec![], vec![]).unwrap()),
                recall_response: Ok(Ph1mRecallResponse::v1(vec![], None).unwrap()),
                forget_response: Ok(Ph1mForgetResponse::v1(false, None, None).unwrap()),
                hint_bundle_build_response: Ok(Ph1mHintBundleBuildResponse::v1(
                    vec![],
                    ReasonCodeId(0x4D00_000D),
                )
                .unwrap()),
                context_bundle_build_response: Ok(Ph1mContextBundleBuildResponse::v1(
                    vec![],
                    vec![],
                    vec![],
                    base_metric_payload(),
                    ReasonCodeId(0x4D00_000E),
                )
                .unwrap()),
                suppression_set_response: Ok(Ph1mSuppressionSetResponse::v1(
                    true,
                    base_suppression_rule(),
                    ReasonCodeId(0x4D00_0010),
                )
                .unwrap()),
                safe_summary_response: Ok(Ph1mSafeSummaryResponse::v1(
                    vec![],
                    0,
                    ReasonCodeId(0x4D00_0011),
                )
                .unwrap()),
                emotional_thread_update_response: Ok(Ph1mEmotionalThreadUpdateResponse::v1(
                    MemoryEmotionalThreadState::v1(
                        "tone".to_string(),
                        vec!["calm".to_string()],
                        Some("Tone continuity".to_string()),
                        MonotonicTimeNs(16),
                    )
                    .unwrap(),
                    ReasonCodeId(0x4D00_0012),
                )
                .unwrap()),
                metrics_emit_response: Ok(Ph1mMetricsEmitResponse::v1(
                    true,
                    ReasonCodeId(0x4D00_0013),
                )
                .unwrap()),
                graph_update_response: Ok(Ph1mGraphUpdateResponse::v1(
                    0,
                    ReasonCodeId(0x4D00_0014),
                )
                .unwrap()),
                retention_mode_set_response: Ok(Ph1mRetentionModeSetResponse::v1(
                    MemoryRetentionMode::Default,
                    MonotonicTimeNs(17),
                    ReasonCodeId(0x4D00_0015),
                )
                .unwrap()),
                thread_digest_upsert_response: Ok(Ph1mThreadDigestUpsertResponse::v1(
                    true,
                    "thread_japan_trip".to_string(),
                    ReasonCodeId(0x4D00_0009),
                )
                .unwrap()),
                resume_select_response: Ok(Ph1mResumeSelectResponse::v1(
                    Some("thread_japan_trip".to_string()),
                    Some("Japan ski trip".to_string()),
                    Some(selene_kernel_contracts::ph1m::MemoryResumeTier::Hot),
                    selene_kernel_contracts::ph1m::MemoryResumeAction::AutoLoad,
                    vec!["Flights shortlisted".to_string()],
                    ReasonCodeId(0x4D00_000A),
                )
                .unwrap()),
            },
        )
        .unwrap()
    }

    #[test]
    fn at_m_07_wiring_disabled() {
        let mut w = wiring(false);
        let input = MemoryTurnInput::v1(
            CorrelationId(7901),
            TurnId(8901),
            MemoryOperation::Recall(base_recall_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();
        assert_eq!(outcome, MemoryWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_m_08_propose_forwarded() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7902),
            TurnId(8902),
            MemoryOperation::Propose(base_propose_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(bundle.correlation_id, CorrelationId(7902));
        assert_eq!(bundle.turn_id, TurnId(8902));
        match bundle.output {
            MemoryTurnOutput::Propose(_) => {}
            _ => panic!("expected propose output"),
        }
    }

    #[test]
    fn at_m_09_budget_exceeded_fails_closed() {
        let mut w = wiring(true);

        let mut proposals = Vec::new();
        for i in 0..17 {
            let item = MemoryProposedItem::v1(
                MemoryKey::new(format!("k{i}")).unwrap(),
                MemoryValue::v1("v".to_string(), None).unwrap(),
                MemoryLayer::Micro,
                MemorySensitivityFlag::Low,
                MemoryConfidence::High,
                MemoryConsent::NotRequested,
                "evidence".to_string(),
                MemoryProvenance::v1(None, None).unwrap(),
            )
            .unwrap();
            proposals.push(item);
        }
        let req = Ph1mProposeRequest::v1(MonotonicTimeNs(20), speaker_ok(), policy_ok(), proposals)
            .unwrap();
        let input = MemoryTurnInput::v1(
            CorrelationId(7903),
            TurnId(8903),
            MemoryOperation::Propose(req),
        )
        .unwrap();

        let outcome = w.run_turn(&input).unwrap();
        let MemoryWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refused outcome");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_M_BUDGET_EXCEEDED);
    }

    #[test]
    fn at_m_10_engine_error_fails_closed() {
        let mut w = Ph1mWiring::new(
            Ph1mWiringConfig::mvp_v1(true),
            MockMemoryEngine {
                propose_response: Err(ContractViolation::InvalidValue {
                    field: "mock",
                    reason: "failed",
                }),
                recall_response: Ok(Ph1mRecallResponse::v1(vec![], None).unwrap()),
                forget_response: Ok(Ph1mForgetResponse::v1(false, None, None).unwrap()),
                hint_bundle_build_response: Ok(Ph1mHintBundleBuildResponse::v1(
                    vec![],
                    ReasonCodeId(0x4D00_000D),
                )
                .unwrap()),
                context_bundle_build_response: Ok(Ph1mContextBundleBuildResponse::v1(
                    vec![],
                    vec![],
                    vec![],
                    base_metric_payload(),
                    ReasonCodeId(0x4D00_000E),
                )
                .unwrap()),
                suppression_set_response: Ok(Ph1mSuppressionSetResponse::v1(
                    true,
                    base_suppression_rule(),
                    ReasonCodeId(0x4D00_0010),
                )
                .unwrap()),
                safe_summary_response: Ok(Ph1mSafeSummaryResponse::v1(
                    vec![],
                    0,
                    ReasonCodeId(0x4D00_0011),
                )
                .unwrap()),
                emotional_thread_update_response: Ok(Ph1mEmotionalThreadUpdateResponse::v1(
                    MemoryEmotionalThreadState::v1(
                        "tone".to_string(),
                        vec!["calm".to_string()],
                        Some("Tone continuity".to_string()),
                        MonotonicTimeNs(16),
                    )
                    .unwrap(),
                    ReasonCodeId(0x4D00_0012),
                )
                .unwrap()),
                metrics_emit_response: Ok(Ph1mMetricsEmitResponse::v1(
                    true,
                    ReasonCodeId(0x4D00_0013),
                )
                .unwrap()),
                graph_update_response: Ok(Ph1mGraphUpdateResponse::v1(
                    0,
                    ReasonCodeId(0x4D00_0014),
                )
                .unwrap()),
                retention_mode_set_response: Ok(Ph1mRetentionModeSetResponse::v1(
                    MemoryRetentionMode::Default,
                    MonotonicTimeNs(17),
                    ReasonCodeId(0x4D00_0015),
                )
                .unwrap()),
                thread_digest_upsert_response: Ok(Ph1mThreadDigestUpsertResponse::v1(
                    true,
                    "thread_japan_trip".to_string(),
                    ReasonCodeId(0x4D00_0009),
                )
                .unwrap()),
                resume_select_response: Ok(Ph1mResumeSelectResponse::v1(
                    Some("thread_japan_trip".to_string()),
                    Some("Japan ski trip".to_string()),
                    Some(selene_kernel_contracts::ph1m::MemoryResumeTier::Hot),
                    selene_kernel_contracts::ph1m::MemoryResumeAction::AutoLoad,
                    vec!["Flights shortlisted".to_string()],
                    ReasonCodeId(0x4D00_000A),
                )
                .unwrap()),
            },
        )
        .unwrap();
        let input = MemoryTurnInput::v1(
            CorrelationId(7904),
            TurnId(8904),
            MemoryOperation::Propose(base_propose_request()),
        )
        .unwrap();

        let outcome = w.run_turn(&input).unwrap();
        let MemoryWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refused outcome");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_M_INTERNAL_PIPELINE_ERROR
        );
    }

    #[test]
    fn at_m_11_recall_and_forget_forwarded() {
        let mut w = wiring(true);

        let recall_input = MemoryTurnInput::v1(
            CorrelationId(7905),
            TurnId(8905),
            MemoryOperation::Recall(base_recall_request()),
        )
        .unwrap();
        let recall_outcome = w.run_turn(&recall_input).unwrap();
        match recall_outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::Recall(_) => {}
                _ => panic!("expected recall output"),
            },
            _ => panic!("expected forwarded recall"),
        }

        let forget_input = MemoryTurnInput::v1(
            CorrelationId(7906),
            TurnId(8906),
            MemoryOperation::Forget(base_forget_request()),
        )
        .unwrap();
        let forget_outcome = w.run_turn(&forget_input).unwrap();
        match forget_outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::Forget(_) => {}
                _ => panic!("expected forget output"),
            },
            _ => panic!("expected forwarded forget"),
        }
    }

    #[test]
    fn at_m_12_thread_digest_upsert_forwarded() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7907),
            TurnId(8907),
            MemoryOperation::ThreadDigestUpsert(base_thread_digest_upsert_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::ThreadDigestUpsert(resp) => {
                    assert_eq!(resp.thread_id, "thread_japan_trip");
                }
                _ => panic!("expected thread digest upsert output"),
            },
            _ => panic!("expected forwarded thread digest upsert"),
        }
    }

    #[test]
    fn at_m_13_resume_select_forwarded() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7908),
            TurnId(8908),
            MemoryOperation::ResumeSelect(base_resume_select_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::ResumeSelect(resp) => {
                    assert_eq!(
                        resp.selected_thread_id.as_deref(),
                        Some("thread_japan_trip")
                    );
                }
                _ => panic!("expected resume select output"),
            },
            _ => panic!("expected forwarded resume select"),
        }
    }

    #[test]
    fn at_m_17_real_runtime_resume_hot_surface() {
        let (mut w, cfg) = real_runtime_wiring();
        let hot_delta_ns = ms_to_ns(cfg.resume_hot_window_ms.saturating_sub(1));
        let now = MonotonicTimeNs(hot_delta_ns.saturating_add(5_000_000_000));
        let seed_input = MemoryTurnInput::v1(
            CorrelationId(7926),
            TurnId(8926),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at(
                now,
                MonotonicTimeNs(now.0.saturating_sub(hot_delta_ns)),
                "thread_resume_hot",
                "Japan ski trip",
                vec![
                    "Flights shortlisted".to_string(),
                    "Need hotel confirmation".to_string(),
                ],
                false,
                true,
                5,
                "idem_resume_hot",
            )),
        )
        .unwrap();
        w.run_turn(&seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7927),
            TurnId(8927),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());
        assert_eq!(resp.resume_tier, Some(MemoryResumeTier::Hot));
        assert_eq!(resp.resume_action, MemoryResumeAction::AutoLoad);
    }

    #[test]
    fn at_m_18_real_runtime_resume_warm_surface() {
        let (mut w, cfg) = real_runtime_wiring();
        let warm_delta_ns = ms_to_ns(cfg.resume_warm_window_ms.saturating_sub(1));
        let hot_ns = ms_to_ns(cfg.resume_hot_window_ms);
        let now = MonotonicTimeNs(warm_delta_ns.saturating_add(5_000_000_000));
        let seed_input = MemoryTurnInput::v1(
            CorrelationId(7928),
            TurnId(8928),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at(
                now,
                MonotonicTimeNs(
                    now.0
                        .saturating_sub(warm_delta_ns.max(hot_ns.saturating_add(1))),
                ),
                "thread_resume_warm",
                "Payroll follow-up",
                vec!["Pending approval".to_string()],
                false,
                false,
                2,
                "idem_resume_warm",
            )),
        )
        .unwrap();
        w.run_turn(&seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7929),
            TurnId(8929),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());
        assert_eq!(resp.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(resp.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn at_m_19_real_runtime_resume_cold_without_topic_surface() {
        let (mut w, cfg) = real_runtime_wiring();
        let older_than_warm_ns = ms_to_ns(cfg.resume_warm_window_ms.saturating_add(1));
        let now = MonotonicTimeNs(older_than_warm_ns.saturating_add(5_000_000_000));
        let seed_input = MemoryTurnInput::v1(
            CorrelationId(7930),
            TurnId(8930),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at(
                now,
                MonotonicTimeNs(now.0.saturating_sub(older_than_warm_ns)),
                "thread_resume_cold",
                "Old topic",
                vec!["Old summary".to_string()],
                false,
                false,
                1,
                "idem_resume_cold",
            )),
        )
        .unwrap();
        w.run_turn(&seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7931),
            TurnId(8931),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());
        assert_eq!(resp.resume_tier, None);
        assert_eq!(resp.resume_action, MemoryResumeAction::None);
        assert!(resp.resume_summary_bullets.is_empty());
    }

    #[test]
    fn at_m_27_real_runtime_resume_unresolved_within_decay_window_breaks_warm_tie() {
        let (mut w, cfg) = real_runtime_wiring();
        assert!(cfg.resume_hot_window_ms < cfg.resume_warm_window_ms);
        assert!(cfg.resume_warm_window_ms < cfg.unresolved_decay_window_ms);

        let warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1);
        assert!(warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(warm_age_ms <= cfg.unresolved_decay_window_ms);

        let warm_age_ns = ms_to_ns(warm_age_ms);
        let now = MonotonicTimeNs(warm_age_ns.saturating_add(5_000_000_000));
        let last_updated = MonotonicTimeNs(now.0.saturating_sub(warm_age_ns));
        let shared_title = "Same warm thread";
        let shared_summary = vec!["Same summary".to_string()];

        let resolved_seed_input = MemoryTurnInput::v1(
            CorrelationId(7936),
            TurnId(8936),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                last_updated,
                "thread_alpha_resolved",
                shared_title,
                shared_summary.clone(),
                false,
                false,
                7,
                "idem_resume_alpha_resolved",
            )),
        )
        .unwrap();
        w.run_turn(&resolved_seed_input).unwrap();

        let unresolved_seed_input = MemoryTurnInput::v1(
            CorrelationId(7937),
            TurnId(8937),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                last_updated,
                "thread_unresolved_warm",
                shared_title,
                shared_summary,
                false,
                true,
                7,
                "idem_resume_unresolved_warm",
            )),
        )
        .unwrap();
        w.run_turn(&unresolved_seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7938),
            TurnId(8938),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());

        assert_eq!(
            resp.selected_thread_id,
            Some("thread_unresolved_warm".to_string())
        );
        assert_eq!(resp.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(resp.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn at_m_32_real_runtime_remember_everything_prefers_higher_use_warm_thread() {
        let (mut w, cfg) = real_runtime_wiring();
        let older_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(10_000);
        let newer_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1_000);
        assert!(older_warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(newer_warm_age_ms <= cfg.resume_warm_window_ms);
        let now = MonotonicTimeNs(ms_to_ns(older_warm_age_ms).saturating_add(5_000_000_000));

        let retention_input = MemoryTurnInput::v1(
            CorrelationId(7945),
            TurnId(8945),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    now,
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::RememberEverything,
                    "idem_real_runtime_ret_remember_everything".to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        w.run_turn(&retention_input).unwrap();

        let shared_title = "Shared warm thread";
        let shared_summary = vec!["Shared summary".to_string()];

        let high_use_seed_input = MemoryTurnInput::v1(
            CorrelationId(7946),
            TurnId(8946),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(older_warm_age_ms))),
                "thread_high_use_older",
                shared_title,
                shared_summary.clone(),
                false,
                false,
                9,
                "idem_real_runtime_high_use_older",
            )),
        )
        .unwrap();
        w.run_turn(&high_use_seed_input).unwrap();

        let lower_use_seed_input = MemoryTurnInput::v1(
            CorrelationId(7947),
            TurnId(8947),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(newer_warm_age_ms))),
                "thread_low_use_newer",
                shared_title,
                shared_summary,
                false,
                false,
                1,
                "idem_real_runtime_low_use_newer",
            )),
        )
        .unwrap();
        w.run_turn(&lower_use_seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7948),
            TurnId(8948),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());

        assert_eq!(
            resp.selected_thread_id,
            Some("thread_high_use_older".to_string())
        );
        assert_eq!(resp.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(resp.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn at_m_33_real_runtime_default_prefers_more_recent_warm_thread_over_use_count() {
        let (mut w, cfg) = real_runtime_wiring();
        let older_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(10_000);
        let newer_warm_age_ms = cfg.resume_hot_window_ms.saturating_add(1_000);
        assert!(older_warm_age_ms <= cfg.resume_warm_window_ms);
        assert!(newer_warm_age_ms <= cfg.resume_warm_window_ms);
        let now = MonotonicTimeNs(ms_to_ns(older_warm_age_ms).saturating_add(5_000_000_000));

        let shared_title = "Shared warm thread";
        let shared_summary = vec!["Shared summary".to_string()];

        let high_use_seed_input = MemoryTurnInput::v1(
            CorrelationId(7949),
            TurnId(8949),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(older_warm_age_ms))),
                "thread_high_use_older",
                shared_title,
                shared_summary.clone(),
                false,
                false,
                9,
                "idem_real_runtime_default_high_use_older",
            )),
        )
        .unwrap();
        w.run_turn(&high_use_seed_input).unwrap();

        let lower_use_seed_input = MemoryTurnInput::v1(
            CorrelationId(7950),
            TurnId(8950),
            MemoryOperation::ThreadDigestUpsert(thread_digest_upsert_request_at_with_contract_flags(
                now,
                MonotonicTimeNs(now.0.saturating_sub(ms_to_ns(newer_warm_age_ms))),
                "thread_low_use_newer",
                shared_title,
                shared_summary,
                false,
                false,
                1,
                "idem_real_runtime_default_low_use_newer",
            )),
        )
        .unwrap();
        w.run_turn(&lower_use_seed_input).unwrap();

        let resume_input = MemoryTurnInput::v1(
            CorrelationId(7951),
            TurnId(8951),
            MemoryOperation::ResumeSelect(resume_select_request_at(now)),
        )
        .unwrap();
        let resp = forwarded_resume_select_response(w.run_turn(&resume_input).unwrap());

        assert_eq!(resp.selected_thread_id, Some("thread_low_use_newer".to_string()));
        assert_eq!(resp.resume_tier, Some(MemoryResumeTier::Warm));
        assert_eq!(resp.resume_action, MemoryResumeAction::Suggest);
    }

    #[test]
    fn at_m_28_real_runtime_propose_eligibility_decisions_recorded_in_envelope() {
        let (mut w, _cfg) = real_runtime_wiring();
        let rejected_key = MemoryKey::new("memory:do_not_store:trip_pin").unwrap();
        let needs_consent_key = MemoryKey::new("memory:sensitive:medical_note").unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7944),
            TurnId(8944),
            MemoryOperation::Propose(propose_request_at(
                MonotonicTimeNs(60),
                vec![
                    MemoryProposedItem::v1(
                        rejected_key.clone(),
                        MemoryValue::v1("Do not save this".to_string(), None).unwrap(),
                        MemoryLayer::LongTerm,
                        MemorySensitivityFlag::Low,
                        MemoryConfidence::High,
                        MemoryConsent::Denied,
                        "Do not remember this".to_string(),
                        MemoryProvenance::v1(None, None).unwrap(),
                    )
                    .unwrap(),
                    MemoryProposedItem::v1(
                        needs_consent_key.clone(),
                        MemoryValue::v1("Migraine medication".to_string(), None).unwrap(),
                        MemoryLayer::LongTerm,
                        MemorySensitivityFlag::Sensitive,
                        MemoryConfidence::High,
                        MemoryConsent::NotRequested,
                        "I am taking migraine medication".to_string(),
                        MemoryProvenance::v1(None, None).unwrap(),
                    )
                    .unwrap(),
                ],
            )),
        )
        .unwrap();

        let outcome = w.run_turn(&propose_input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded propose outcome");
        };
        let MemoryTurnOutput::Propose(resp) = bundle.output else {
            panic!("expected propose output");
        };

        assert!(resp.ledger_events.is_empty());
        assert_eq!(resp.decisions.len(), 2);

        let rejected = resp
            .decisions
            .iter()
            .find(|decision| decision.memory_key == rejected_key)
            .expect("expected rejected propose decision");
        assert_eq!(rejected.status, MemoryCommitStatus::Rejected);
        assert_eq!(
            rejected.reason_code,
            selene_engines::ph1m::reason_codes::M_POLICY_BLOCKED
        );
        assert_eq!(rejected.consent_prompt, None);

        let needs_consent = resp
            .decisions
            .iter()
            .find(|decision| decision.memory_key == needs_consent_key)
            .expect("expected needs-consent propose decision");
        assert_eq!(needs_consent.status, MemoryCommitStatus::NeedsConsent);
        assert_eq!(
            needs_consent.reason_code,
            selene_engines::ph1m::reason_codes::M_NEEDS_CONSENT
        );
        assert!(needs_consent.consent_prompt.is_some());
    }

    #[test]
    fn at_m_29_real_runtime_propose_unknown_speaker_rejected_before_loop() {
        let (mut w, _cfg) = real_runtime_wiring();
        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7945),
            TurnId(8945),
            MemoryOperation::Propose(Ph1mProposeRequest::v1(
                MonotonicTimeNs(61),
                speaker_unknown(),
                policy_ok(),
                vec![proposed_item_with_confidence(
                    "preferred_name",
                    "John",
                    MemoryConfidence::High,
                    "My name is John",
                )],
            )
            .unwrap()),
        )
        .unwrap();

        let outcome = w.run_turn(&propose_input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded propose outcome");
        };
        let MemoryTurnOutput::Propose(resp) = bundle.output else {
            panic!("expected propose output");
        };

        assert!(resp.ledger_events.is_empty());
        assert_eq!(resp.decisions.len(), 1);
        let decision = &resp.decisions[0];
        assert_eq!(decision.status, MemoryCommitStatus::Rejected);
        assert_eq!(decision.reason_code, selene_engines::ph1m::reason_codes::M_REJECT_UNKNOWN_SPEAKER);
        assert_eq!(decision.consent_prompt, None);
    }

    #[test]
    fn at_m_30_real_runtime_propose_privacy_mode_rejected_before_loop() {
        let (mut w, _cfg) = real_runtime_wiring();
        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7946),
            TurnId(8946),
            MemoryOperation::Propose(Ph1mProposeRequest::v1(
                MonotonicTimeNs(62),
                speaker_ok(),
                policy_privacy_mode(),
                vec![proposed_item_with_confidence(
                    "preferred_name",
                    "John",
                    MemoryConfidence::High,
                    "My name is John",
                )],
            )
            .unwrap()),
        )
        .unwrap();

        let outcome = w.run_turn(&propose_input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded propose outcome");
        };
        let MemoryTurnOutput::Propose(resp) = bundle.output else {
            panic!("expected propose output");
        };

        assert!(resp.ledger_events.is_empty());
        assert_eq!(resp.decisions.len(), 1);
        let decision = &resp.decisions[0];
        assert_eq!(decision.status, MemoryCommitStatus::Rejected);
        assert_eq!(decision.reason_code, selene_engines::ph1m::reason_codes::M_POLICY_BLOCKED);
        assert_eq!(decision.consent_prompt, None);
    }

    #[test]
    fn at_m_31_real_runtime_propose_do_not_store_rejected_in_loop() {
        let (mut w, _cfg) = real_runtime_wiring();

        let suppression_input = MemoryTurnInput::v1(
            CorrelationId(7947),
            TurnId(8947),
            MemoryOperation::SuppressionSet(Ph1mSuppressionSetRequest::v1(
                MonotonicTimeNs(63),
                speaker_ok(),
                policy_ok(),
                MemorySuppressionRule::v1(
                    MemorySuppressionTargetType::TopicKey,
                    "preferred_name".to_string(),
                    MemorySuppressionRuleKind::DoNotStore,
                    true,
                    ReasonCodeId(101),
                    MonotonicTimeNs(63),
                )
                .unwrap(),
                "idem_sup_store".to_string(),
            )
            .unwrap()),
        )
        .unwrap();
        w.run_turn(&suppression_input).unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7948),
            TurnId(8948),
            MemoryOperation::Propose(Ph1mProposeRequest::v1(
                MonotonicTimeNs(64),
                speaker_ok(),
                policy_ok(),
                vec![proposed_item_with_confidence(
                    "preferred_name",
                    "John",
                    MemoryConfidence::High,
                    "My name is John",
                )],
            )
            .unwrap()),
        )
        .unwrap();

        let outcome = w.run_turn(&propose_input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded propose outcome");
        };
        let MemoryTurnOutput::Propose(resp) = bundle.output else {
            panic!("expected propose output");
        };

        assert!(resp.ledger_events.is_empty());
        assert_eq!(resp.decisions.len(), 1);
        let decision = &resp.decisions[0];
        assert_eq!(decision.status, MemoryCommitStatus::Rejected);
        assert_eq!(decision.reason_code, selene_engines::ph1m::reason_codes::M_POLICY_BLOCKED);
        assert_eq!(decision.consent_prompt, None);
    }

    #[test]
    fn at_m_21_real_runtime_context_bundle_high_confidence_emits_confirmed_tag() {
        let (mut w, _cfg) = real_runtime_wiring();
        let key = MemoryKey::new("project:active:jp_trip").unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7932),
            TurnId(8932),
            MemoryOperation::Propose(propose_request_at(
                MonotonicTimeNs(40),
                vec![proposed_item_with_confidence(
                    key.as_str(),
                    "Japan Trip",
                    MemoryConfidence::High,
                    "I am planning a Japan trip",
                )],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7933),
            TurnId(8933),
            MemoryOperation::ContextBundleBuild(context_bundle_request_at(
                MonotonicTimeNs(41),
                vec![key.clone()],
            )),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 1);
        let item = &resp.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Confirmed);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
    }

    #[test]
    fn at_m_22_real_runtime_context_bundle_confidence_ranking_prefers_high_over_low() {
        let (mut w, _cfg) = real_runtime_wiring();
        let high_key = MemoryKey::new("project:active:travel_priority").unwrap();
        let low_key = MemoryKey::new("project:active:travel_note").unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7934),
            TurnId(8934),
            MemoryOperation::Propose(propose_request_at(
                MonotonicTimeNs(42),
                vec![
                    proposed_item_with_confidence(
                        high_key.as_str(),
                        "Book the flight first",
                        MemoryConfidence::High,
                        "The flight should be booked first",
                    ),
                    proposed_item_with_confidence(
                        low_key.as_str(),
                        "Maybe visit a museum",
                        MemoryConfidence::Low,
                        "A museum might be nice",
                    ),
                ],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7935),
            TurnId(8935),
            MemoryOperation::ContextBundleBuild(context_bundle_request_at(
                MonotonicTimeNs(43),
                vec![high_key.clone(), low_key.clone()],
            )),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 2);

        let high_index = resp
            .pull_items
            .iter()
            .position(|item| item.memory_key == high_key)
            .expect("expected high-confidence pull item");
        let low_index = resp
            .pull_items
            .iter()
            .position(|item| item.memory_key == low_key)
            .expect("expected low-confidence pull item");
        assert!(high_index < low_index);

        let high_item = &resp.pull_items[high_index];
        let low_item = &resp.pull_items[low_index];
        assert_eq!(high_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(high_item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(low_item.tag, MemoryItemTag::Tentative);
        assert_eq!(low_item.provenance_tier, MemoryProvenanceTier::UserStated);
    }

    #[test]
    fn at_m_23_real_runtime_context_bundle_current_state_conflict_emits_conflict_tag() {
        let (mut w, _cfg) = real_runtime_wiring();
        let key = MemoryKey::new("project:active:trip_owner").unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7936),
            TurnId(8936),
            MemoryOperation::Propose(propose_request_at(
                MonotonicTimeNs(44),
                vec![proposed_item_with_confidence(
                    key.as_str(),
                    "Selene",
                    MemoryConfidence::High,
                    "Selene owns the trip plan",
                )],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7937),
            TurnId(8937),
            MemoryOperation::ContextBundleBuild(context_bundle_request_with_state_at(
                MonotonicTimeNs(45),
                vec![key.clone()],
                vec![MemoryContextFact::v1(
                    key.clone(),
                    MemoryValue::v1("Jordan".to_string(), None).unwrap(),
                )
                .unwrap()],
            )),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 1);
        let item = &resp.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Conflict);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(resp.metric_payload.conflict_count, 1);
        assert_eq!(resp.metric_payload.stale_count, 0);
    }

    #[test]
    fn at_m_24_real_runtime_context_bundle_conflict_metric_counts_only_conflicting_entries() {
        let (mut w, _cfg) = real_runtime_wiring();
        let conflict_key = MemoryKey::new("project:active:flight_step").unwrap();
        let aligned_key = MemoryKey::new("project:active:insurance_step").unwrap();

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7938),
            TurnId(8938),
            MemoryOperation::Propose(propose_request_at(
                MonotonicTimeNs(46),
                vec![
                    proposed_item_with_confidence(
                        conflict_key.as_str(),
                        "Book the flight",
                        MemoryConfidence::High,
                        "The next step is booking the flight",
                    ),
                    proposed_item_with_confidence(
                        aligned_key.as_str(),
                        "Buy travel insurance",
                        MemoryConfidence::High,
                        "The travel insurance should be bought",
                    ),
                ],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7939),
            TurnId(8939),
            MemoryOperation::ContextBundleBuild(context_bundle_request_with_state_at(
                MonotonicTimeNs(47),
                vec![conflict_key.clone(), aligned_key.clone()],
                vec![
                    MemoryContextFact::v1(
                        conflict_key.clone(),
                        MemoryValue::v1("Change the flight".to_string(), None).unwrap(),
                    )
                    .unwrap(),
                    MemoryContextFact::v1(
                        aligned_key.clone(),
                        MemoryValue::v1("Buy travel insurance".to_string(), None).unwrap(),
                    )
                    .unwrap(),
                ],
            )),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 2);

        let conflict_item = resp
            .pull_items
            .iter()
            .find(|item| item.memory_key == conflict_key)
            .expect("expected conflicting pull item");
        let aligned_item = resp
            .pull_items
            .iter()
            .find(|item| item.memory_key == aligned_key)
            .expect("expected aligned pull item");
        assert_eq!(conflict_item.tag, MemoryItemTag::Conflict);
        assert_eq!(aligned_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(resp.metric_payload.conflict_count, 1);
        assert_eq!(resp.metric_payload.stale_count, 0);
        assert_eq!(
            conflict_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
        assert_eq!(
            aligned_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
    }

    #[test]
    fn at_m_25_real_runtime_context_bundle_expired_micro_entry_emits_stale_tag() {
        let (mut w, cfg) = real_runtime_wiring();
        let key = MemoryKey::new("project:active:boarding_gate").unwrap();
        let propose_time = MonotonicTimeNs(48);
        let now = MonotonicTimeNs(
            propose_time
                .0
                .saturating_add(ms_to_ns(cfg.micro_ttl_ms))
                .saturating_add(1),
        );

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7940),
            TurnId(8940),
            MemoryOperation::Propose(propose_request_at(
                propose_time,
                vec![proposed_micro_item_with_confidence(
                    key.as_str(),
                    "Gate A12",
                    MemoryConfidence::High,
                    "The boarding gate is A12",
                )],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7941),
            TurnId(8941),
            MemoryOperation::ContextBundleBuild(context_bundle_request_at(now, vec![key.clone()])),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 1);
        let item = &resp.pull_items[0];
        assert_eq!(item.memory_key, key);
        assert_eq!(item.tag, MemoryItemTag::Stale);
        assert_eq!(item.confidence, MemoryConfidence::High);
        assert_eq!(item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(resp.metric_payload.stale_count, 1);
        assert_eq!(resp.metric_payload.conflict_count, 0);
    }

    #[test]
    fn at_m_26_real_runtime_context_bundle_stale_metric_counts_only_expired_entries() {
        let (mut w, cfg) = real_runtime_wiring();
        let stale_key = MemoryKey::new("project:active:boarding_gate").unwrap();
        let confirmed_key = MemoryKey::new("project:active:hotel_booking").unwrap();
        let propose_time = MonotonicTimeNs(50);
        let now = MonotonicTimeNs(
            propose_time
                .0
                .saturating_add(ms_to_ns(cfg.micro_ttl_ms))
                .saturating_add(1),
        );

        let propose_input = MemoryTurnInput::v1(
            CorrelationId(7942),
            TurnId(8942),
            MemoryOperation::Propose(propose_request_at(
                propose_time,
                vec![
                    proposed_micro_item_with_confidence(
                        stale_key.as_str(),
                        "Gate A12",
                        MemoryConfidence::High,
                        "The boarding gate is A12",
                    ),
                    proposed_item_with_confidence(
                        confirmed_key.as_str(),
                        "Hotel booked",
                        MemoryConfidence::High,
                        "The hotel is booked",
                    ),
                ],
            )),
        )
        .unwrap();
        w.run_turn(&propose_input).unwrap();

        let bundle_input = MemoryTurnInput::v1(
            CorrelationId(7943),
            TurnId(8943),
            MemoryOperation::ContextBundleBuild(context_bundle_request_at(
                now,
                vec![stale_key.clone(), confirmed_key.clone()],
            )),
        )
        .unwrap();
        let resp = forwarded_context_bundle_response(w.run_turn(&bundle_input).unwrap());

        assert!(resp.push_items.is_empty());
        assert_eq!(resp.pull_items.len(), 2);

        let stale_item = resp
            .pull_items
            .iter()
            .find(|item| item.memory_key == stale_key)
            .expect("expected expired pull item");
        let confirmed_item = resp
            .pull_items
            .iter()
            .find(|item| item.memory_key == confirmed_key)
            .expect("expected non-expired pull item");
        assert_eq!(stale_item.tag, MemoryItemTag::Stale);
        assert_eq!(confirmed_item.tag, MemoryItemTag::Confirmed);
        assert_eq!(resp.metric_payload.stale_count, 1);
        assert_eq!(resp.metric_payload.conflict_count, 0);
        assert_eq!(stale_item.provenance_tier, MemoryProvenanceTier::UserStated);
        assert_eq!(
            confirmed_item.provenance_tier,
            MemoryProvenanceTier::UserStated
        );
    }

    #[test]
    fn at_m_14_architecture_operations_forwarded() {
        let mut w = wiring(true);

        let ops = vec![
            (
                CorrelationId(7909),
                TurnId(8909),
                MemoryOperation::HintBundleBuild(base_hint_bundle_request()),
            ),
            (
                CorrelationId(7910),
                TurnId(8910),
                MemoryOperation::ContextBundleBuild(base_context_bundle_request()),
            ),
            (
                CorrelationId(7911),
                TurnId(8911),
                MemoryOperation::SuppressionSet(base_suppression_set_request()),
            ),
            (
                CorrelationId(7912),
                TurnId(8912),
                MemoryOperation::SafeSummary(base_safe_summary_request()),
            ),
            (
                CorrelationId(7913),
                TurnId(8913),
                MemoryOperation::EmotionalThreadUpdate(base_emotional_thread_update_request()),
            ),
            (
                CorrelationId(7914),
                TurnId(8914),
                MemoryOperation::MetricsEmit(base_metrics_emit_request()),
            ),
            (
                CorrelationId(7915),
                TurnId(8915),
                MemoryOperation::GraphUpdate(base_graph_update_request()),
            ),
            (
                CorrelationId(7916),
                TurnId(8916),
                MemoryOperation::RetentionModeSet(base_retention_mode_set_request()),
            ),
        ];

        for (cid, tid, op) in ops {
            let input = MemoryTurnInput::v1(cid, tid, op).unwrap();
            let out = w.run_turn(&input).unwrap();
            match out {
                MemoryWiringOutcome::Forwarded(bundle) => {
                    assert_eq!(bundle.correlation_id, cid);
                    assert_eq!(bundle.turn_id, tid);
                }
                _ => panic!("expected forwarded architecture operation"),
            }
        }
    }

    #[test]
    fn at_m_15_persist_forwarded_suppression_to_repo() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7920),
            TurnId(8920),
            MemoryOperation::SuppressionSet(base_suppression_set_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();

        let mut store = seeded_store_for_known_user();
        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(persisted);

        let row = store
            .ph1m_suppression_rule_row(
                &UserId::new("user").unwrap(),
                MemorySuppressionTargetType::TopicKey,
                "preferred_name",
                MemorySuppressionRuleKind::DoNotMention,
            )
            .unwrap();
        assert!(row.rule.active);
    }

    #[test]
    fn at_m_16_persist_forwarded_thread_and_retention_to_repo() {
        let mut w = wiring(true);
        let mut store = seeded_store_for_known_user();

        let thread_input = MemoryTurnInput::v1(
            CorrelationId(7921),
            TurnId(8921),
            MemoryOperation::ThreadDigestUpsert(base_thread_digest_upsert_request()),
        )
        .unwrap();
        let thread_outcome = w.run_turn(&thread_input).unwrap();
        let thread_persisted =
            persist_memory_forwarded_outcome(&mut store, &thread_input, &thread_outcome).unwrap();
        assert!(thread_persisted);
        assert!(store
            .ph1m_thread_current_row(&UserId::new("user").unwrap(), "thread_japan_trip")
            .is_some());

        let retention_input = MemoryTurnInput::v1(
            CorrelationId(7922),
            TurnId(8922),
            MemoryOperation::RetentionModeSet(base_retention_mode_set_request()),
        )
        .unwrap();
        let retention_outcome = w.run_turn(&retention_input).unwrap();
        let retention_persisted =
            persist_memory_forwarded_outcome(&mut store, &retention_input, &retention_outcome)
                .unwrap();
        assert!(retention_persisted);
        assert_eq!(
            store
                .ph1m_retention_preference_row(&UserId::new("user").unwrap())
                .unwrap()
                .memory_retention_mode,
            MemoryRetentionMode::Default
        );
    }

    #[test]
    fn at_m_34_real_runtime_retention_mode_set_persists_remember_everything_to_repo() {
        let (mut w, _cfg) = real_runtime_wiring();
        let mut store = seeded_store_for_known_user();

        let input = MemoryTurnInput::v1(
            CorrelationId(7952),
            TurnId(8952),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    MonotonicTimeNs(70),
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::RememberEverything,
                    "idem_real_runtime_retention_persist_remember_everything".to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let outcome = w.run_turn(&input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = &outcome else {
            panic!("expected forwarded retention mode set outcome");
        };
        let MemoryTurnOutput::RetentionModeSet(resp) = &bundle.output else {
            panic!("expected retention mode set output");
        };
        assert_eq!(resp.memory_retention_mode, MemoryRetentionMode::RememberEverything);

        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(persisted);
        assert_eq!(
            store
                .ph1m_retention_preference_row(&UserId::new("user").unwrap())
                .unwrap()
                .memory_retention_mode,
            MemoryRetentionMode::RememberEverything
        );
    }

    #[test]
    fn at_m_17_persist_non_write_outcome_noop() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7923),
            TurnId(8923),
            MemoryOperation::Recall(base_recall_request()),
        )
        .unwrap();
        let outcome = w.run_turn(&input).unwrap();

        let mut store = seeded_store_for_known_user();
        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(!persisted);
    }

    #[test]
    fn at_m_18_run_turn_and_persist_commits_write_outcome() {
        let mut w = wiring(true);
        let input = MemoryTurnInput::v1(
            CorrelationId(7924),
            TurnId(8924),
            MemoryOperation::ThreadDigestUpsert(base_thread_digest_upsert_request()),
        )
        .unwrap();

        let mut store = seeded_store_for_known_user();
        let outcome = w.run_turn_and_persist(&mut store, &input).unwrap();
        match outcome {
            MemoryWiringOutcome::Forwarded(bundle) => match bundle.output {
                MemoryTurnOutput::ThreadDigestUpsert(resp) => {
                    assert_eq!(resp.thread_id, "thread_japan_trip");
                }
                _ => panic!("expected thread digest upsert output"),
            },
            _ => panic!("expected forwarded outcome"),
        }
        assert!(store
            .ph1m_thread_current_row(&UserId::new("user").unwrap(), "thread_japan_trip")
            .is_some());
    }

    #[test]
    fn at_m_35_real_runtime_run_turn_and_persist_retention_mode_set_commits_remember_everything_to_repo(
    ) {
        let (mut w, _cfg) = real_runtime_wiring();
        let mut store = seeded_store_for_known_user();

        let input = MemoryTurnInput::v1(
            CorrelationId(7953),
            TurnId(8953),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    MonotonicTimeNs(71),
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::RememberEverything,
                    "idem_real_runtime_run_turn_and_persist_retention_remember_everything"
                        .to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let outcome = w.run_turn_and_persist(&mut store, &input).unwrap();
        let MemoryWiringOutcome::Forwarded(bundle) = &outcome else {
            panic!("expected forwarded retention mode set outcome");
        };
        let MemoryTurnOutput::RetentionModeSet(resp) = &bundle.output else {
            panic!("expected retention mode set output");
        };
        assert_eq!(resp.memory_retention_mode, MemoryRetentionMode::RememberEverything);
        assert_eq!(
            store
                .ph1m_retention_preference_row(&UserId::new("user").unwrap())
                .unwrap()
                .memory_retention_mode,
            MemoryRetentionMode::RememberEverything
        );
    }

    #[test]
    fn at_m_36_real_runtime_run_turn_and_persist_retention_mode_idempotent_retry_and_later_default_overwrite(
    ) {
        let (mut w, _cfg) = real_runtime_wiring();
        let mut store = seeded_store_for_known_user();
        let user_id = UserId::new("user").unwrap();

        let first_input = MemoryTurnInput::v1(
            CorrelationId(7954),
            TurnId(8954),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    MonotonicTimeNs(72),
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::RememberEverything,
                    "ret_idem_1".to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let retry_input = MemoryTurnInput::v1(
            CorrelationId(7955),
            TurnId(8955),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    MonotonicTimeNs(73),
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    "ret_idem_1".to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let overwrite_input = MemoryTurnInput::v1(
            CorrelationId(7956),
            TurnId(8956),
            MemoryOperation::RetentionModeSet(
                Ph1mRetentionModeSetRequest::v1(
                    MonotonicTimeNs(74),
                    speaker_ok(),
                    policy_ok(),
                    MemoryRetentionMode::Default,
                    "ret_idem_2".to_string(),
                )
                .unwrap(),
            ),
        )
        .unwrap();

        let first_outcome = w.run_turn_and_persist(&mut store, &first_input).unwrap();
        let MemoryWiringOutcome::Forwarded(first_bundle) = &first_outcome else {
            panic!("expected forwarded retention mode set outcome");
        };
        let MemoryTurnOutput::RetentionModeSet(first_resp) = &first_bundle.output else {
            panic!("expected retention mode set output");
        };
        assert_eq!(
            first_resp.memory_retention_mode,
            MemoryRetentionMode::RememberEverything
        );
        let after_first = store.ph1m_retention_preference_row(&user_id).unwrap();
        assert_eq!(
            after_first.memory_retention_mode,
            MemoryRetentionMode::RememberEverything
        );
        assert_eq!(after_first.updated_at, MonotonicTimeNs(72));
        assert_eq!(after_first.idempotency_key, Some("ret_idem_1".to_string()));

        let retry_outcome = w.run_turn_and_persist(&mut store, &retry_input).unwrap();
        let MemoryWiringOutcome::Forwarded(_) = &retry_outcome else {
            panic!("expected forwarded retention mode set retry outcome");
        };
        let after_retry = store.ph1m_retention_preference_row(&user_id).unwrap();
        assert_eq!(
            after_retry.memory_retention_mode,
            MemoryRetentionMode::RememberEverything
        );
        assert_eq!(after_retry.updated_at, MonotonicTimeNs(72));
        assert_eq!(after_retry.idempotency_key, Some("ret_idem_1".to_string()));

        let overwrite_outcome = w.run_turn_and_persist(&mut store, &overwrite_input).unwrap();
        let MemoryWiringOutcome::Forwarded(_) = &overwrite_outcome else {
            panic!("expected forwarded retention mode set overwrite outcome");
        };
        let after_overwrite = store.ph1m_retention_preference_row(&user_id).unwrap();
        assert_eq!(after_overwrite.memory_retention_mode, MemoryRetentionMode::Default);
        assert_eq!(after_overwrite.updated_at, MonotonicTimeNs(74));
        assert_eq!(
            after_overwrite.idempotency_key,
            Some("ret_idem_2".to_string())
        );
    }

    #[test]
    fn at_m_19_persist_forwarded_propose_commits_memory_rows() {
        let input = MemoryTurnInput::v1(
            CorrelationId(7925),
            TurnId(8925),
            MemoryOperation::Propose(base_propose_request()),
        )
        .unwrap();

        let memory_key = MemoryKey::new("preferred_name").unwrap();
        let event = MemoryLedgerEvent::v1(
            MemoryLedgerEventKind::Stored,
            MonotonicTimeNs(30),
            memory_key.clone(),
            Some(MemoryValue::v1("John".to_string(), None).unwrap()),
            Some("My name is John".to_string()),
            MemoryProvenance::v1(None, None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::ExplicitRemember,
            ReasonCodeId(0x4D00_0001),
        )
        .unwrap();
        let propose_resp = Ph1mProposeResponse::v1(
            vec![MemoryCommitDecision::v1(
                memory_key.clone(),
                MemoryCommitStatus::Stored,
                ReasonCodeId(0x4D00_0001),
                None,
            )
            .unwrap()],
            vec![event],
        )
        .unwrap();
        let outcome = MemoryWiringOutcome::Forwarded(
            MemoryForwardBundle::v1(
                input.correlation_id,
                input.turn_id,
                MemoryTurnOutput::Propose(propose_resp),
            )
            .unwrap(),
        );

        let mut store = seeded_store_for_known_user();
        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(persisted);
        assert_eq!(store.memory_ledger_rows().len(), 1);
        assert_eq!(store.memory_current().len(), 1);
    }

    #[test]
    fn at_m_21_persist_forwarded_updated_propose_commits_memory_rows() {
        let input = MemoryTurnInput::v1(
            CorrelationId(7927),
            TurnId(8927),
            MemoryOperation::Propose(base_propose_request()),
        )
        .unwrap();
        let memory_key = MemoryKey::new("preferred_name").unwrap();

        let mut store = seeded_store_for_known_user();
        store
            .append_memory_row(
                &UserId::new("user").unwrap(),
                MemoryLedgerEvent::v1(
                    MemoryLedgerEventKind::Stored,
                    MonotonicTimeNs(31),
                    memory_key.clone(),
                    Some(MemoryValue::v1("John".to_string(), None).unwrap()),
                    Some("My name is John".to_string()),
                    MemoryProvenance::v1(None, None).unwrap(),
                    MemoryLayer::LongTerm,
                    MemorySensitivityFlag::Low,
                    MemoryConfidence::High,
                    MemoryConsent::ExplicitRemember,
                    selene_engines::ph1m::reason_codes::M_STORED,
                )
                .unwrap(),
                MemoryUsePolicy::AlwaysUsable,
                None,
                Some("seed_preferred_name".to_string()),
            )
            .unwrap();
        assert_eq!(store.memory_current().len(), 1);

        let propose_resp = Ph1mProposeResponse::v1(
            vec![MemoryCommitDecision::v1(
                memory_key.clone(),
                MemoryCommitStatus::Updated,
                selene_engines::ph1m::reason_codes::M_UPDATED,
                None,
            )
            .unwrap()],
            vec![
                MemoryLedgerEvent::v1(
                    MemoryLedgerEventKind::Updated,
                    MonotonicTimeNs(32),
                    memory_key.clone(),
                    Some(MemoryValue::v1("Benji".to_string(), None).unwrap()),
                    Some("Call me Benji".to_string()),
                    MemoryProvenance::v1(None, None).unwrap(),
                    MemoryLayer::LongTerm,
                    MemorySensitivityFlag::Low,
                    MemoryConfidence::High,
                    MemoryConsent::ExplicitRemember,
                    selene_engines::ph1m::reason_codes::M_UPDATED,
                )
                .unwrap(),
            ],
        )
        .unwrap();
        let outcome = MemoryWiringOutcome::Forwarded(
            MemoryForwardBundle::v1(
                input.correlation_id,
                input.turn_id,
                MemoryTurnOutput::Propose(propose_resp),
            )
            .unwrap(),
        );

        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(persisted);
        assert_eq!(store.memory_ledger_rows().len(), 2);
        assert_eq!(store.memory_current().len(), 1);
        let current = store
            .memory_current()
            .get(&(UserId::new("user").unwrap(), memory_key.clone()))
            .expect("updated memory current row should exist");
        assert!(current.active);
        assert_eq!(current.memory_value.as_ref().unwrap().verbatim, "Benji");
    }

    #[test]
    fn at_m_20_persist_forwarded_forget_commits_and_removes_current() {
        let input = MemoryTurnInput::v1(
            CorrelationId(7926),
            TurnId(8926),
            MemoryOperation::Forget(base_forget_request()),
        )
        .unwrap();
        let memory_key = MemoryKey::new("preferred_name").unwrap();

        let mut store = seeded_store_for_known_user();
        store
            .append_memory_row(
                &UserId::new("user").unwrap(),
                MemoryLedgerEvent::v1(
                    MemoryLedgerEventKind::Stored,
                    MonotonicTimeNs(31),
                    memory_key.clone(),
                    Some(MemoryValue::v1("John".to_string(), None).unwrap()),
                    Some("My name is John".to_string()),
                    MemoryProvenance::v1(None, None).unwrap(),
                    MemoryLayer::LongTerm,
                    MemorySensitivityFlag::Low,
                    MemoryConfidence::High,
                    MemoryConsent::ExplicitRemember,
                    ReasonCodeId(0x4D00_0001),
                )
                .unwrap(),
                MemoryUsePolicy::AlwaysUsable,
                None,
                Some("seed_preferred_name".to_string()),
            )
            .unwrap();
        assert_eq!(store.memory_current().len(), 1);

        let forget_resp = Ph1mForgetResponse::v1(
            true,
            Some(
                MemoryLedgerEvent::v1(
                    MemoryLedgerEventKind::Forgotten,
                    MonotonicTimeNs(32),
                    memory_key,
                    None,
                    None,
                    MemoryProvenance::v1(None, None).unwrap(),
                    MemoryLayer::LongTerm,
                    MemorySensitivityFlag::Low,
                    MemoryConfidence::High,
                    MemoryConsent::ExplicitRemember,
                    ReasonCodeId(0x4D00_0006),
                )
                .unwrap(),
            ),
            None,
        )
        .unwrap();
        let outcome = MemoryWiringOutcome::Forwarded(
            MemoryForwardBundle::v1(
                input.correlation_id,
                input.turn_id,
                MemoryTurnOutput::Forget(forget_resp),
            )
            .unwrap(),
        );

        let persisted = persist_memory_forwarded_outcome(&mut store, &input, &outcome).unwrap();
        assert!(persisted);
        assert_eq!(store.memory_ledger_rows().len(), 2);
        assert_eq!(store.memory_current().len(), 1);
        let current = store
            .memory_current()
            .get(&(
                UserId::new("user").unwrap(),
                MemoryKey::new("preferred_name").unwrap(),
            ))
            .expect("memory current row should exist as forgotten tombstone");
        assert!(!current.active);
        assert!(current.memory_value.is_none());
    }
}
