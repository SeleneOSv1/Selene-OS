#![forbid(unsafe_code)]

use std::{cmp::min, collections::BTreeSet};

use selene_engines::ph1_voice_id::{
    EnrolledSpeaker as EngineEnrolledSpeaker, VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_engines::ph1d::{Ph1dProviderAdapter, Ph1dProviderAdapterError};
use selene_kernel_contracts::ph1_voice_id::{Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId};
use selene_kernel_contracts::ph1c::TranscriptOk;
use selene_kernel_contracts::ph1d::{
    Ph1dProviderCallRequest, Ph1dProviderCallResponse, Ph1dProviderInputPayloadKind,
    Ph1dProviderRouteClass, Ph1dProviderStatus, Ph1dProviderTask, Ph1dProviderValidationStatus,
    RequestId, SafetyTier, SchemaHash, PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
};
use selene_kernel_contracts::ph1doc::{DocValidationStatus, DocumentSourceKind};
use selene_kernel_contracts::ph1feedback::FeedbackEventRecord;
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1k::{
    AudioStreamRef, DeviceHealth as Ph1kDeviceHealth, DeviceState as Ph1kDeviceState, DuplexFrame,
    InterruptCandidate as Ph1kInterruptCandidate, InterruptCandidateConfidenceBand,
    InterruptRiskContextClass, PreRollBufferRef, TimingStats as Ph1kTimingStats,
    TtsPlaybackActiveEvent, VadEvent,
};
use selene_kernel_contracts::ph1n::{Ph1nRequest, Ph1nResponse};
use selene_kernel_contracts::ph1os::{
    OsCapabilityId, OsDecisionComputeOk, OsDecisionComputeRequest, OsGateDecision, OsNextMove,
    OsOutcomeActionClass, OsOutcomeUtilizationEntry, OsPolicyEvaluateOk, OsPolicyEvaluateRequest,
    OsRefuse, OsRequestEnvelope, Ph1OsRequest, Ph1OsResponse, OS_CLARIFY_OWNER_ENGINE_ID,
};
use selene_kernel_contracts::ph1selfheal::{
    stable_card_id, FailureContainmentAction, FailureProviderContext, ProblemCardState,
    PromotionDecisionAction, SelfHealCardChain,
};
use selene_kernel_contracts::ph1vision::VisualSourceKind;
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate,
};
use selene_storage::ph1f::{Ph1fStore, StorageError};
use serde_json::{json, Value};

use crate::device_artifact_sync::{self, DeviceArtifactSyncSenderRuntime};
use crate::ph1_voice_id::{
    Ph1VoiceIdLiveRuntime, VoiceIdentityChannel, VoiceIdentityPlatform,
    VoiceIdentityRuntimeContext, VoiceIdentitySignalScope,
};
use crate::ph1builder::BuilderOfflineInput;
use crate::ph1c_superiority::{
    evaluate_ph1c_superiority_pack, SuperiorityEvalPack, SuperiorityGateReport, SuperiorityLane,
};
use crate::ph1context::{
    ContextForwardBundle, ContextTurnInput, ContextWiringOutcome, Ph1ContextEngine,
    Ph1ContextWiring,
};
use crate::ph1doc::DocForwardBundle;
use crate::ph1feedback::{map_feedback_event_to_failure_event, FeedbackForwardBundle};
use crate::ph1learn::{
    map_learn_bundle_to_fix_card, map_learn_bundle_to_problem_card, LearnForwardBundle,
    LearnTurnInput, ProblemCardEscalationState,
};
use crate::ph1n::{Ph1nEngine, Ph1nWiring, Ph1nWiringOutcome};
use crate::ph1pae::{map_pae_bundle_to_promotion_decision, PaeForwardBundle, PaeTurnInput};
use crate::ph1vision::VisionForwardBundle;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.OS OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_OS_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4F53_0101);
    pub const PH1_OS_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4F53_01F1);
    pub const PH1_OS_TOPLEVEL_SEQUENCE_INVALID: ReasonCodeId = ReasonCodeId(0x4F53_0201);
    pub const PH1_OS_TOPLEVEL_UNKNOWN_OPTIONAL_ENGINE: ReasonCodeId = ReasonCodeId(0x4F53_0202);
    pub const PH1_OS_TOPLEVEL_OPTIONAL_BUDGET_INVALID: ReasonCodeId = ReasonCodeId(0x4F53_0203);
    pub const PH1_OS_TOPLEVEL_RUNTIME_BOUNDARY_VIOLATION: ReasonCodeId = ReasonCodeId(0x4F53_0204);
    pub const PH1_OS_TOPLEVEL_CLARIFY_OWNER_INVALID: ReasonCodeId = ReasonCodeId(0x4F53_0205);
    pub const PH1_OS_TOPLEVEL_OPTIONAL_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4F53_0206);
    pub const PH1_OS_VOICE_PH1K_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4F53_0207);
    pub const PH1_OS_VOICE_PH1K_DEVICE_FAILED: ReasonCodeId = ReasonCodeId(0x4F53_0208);
    pub const PH1_OS_VOICE_PH1K_EVIDENCE_REQUIRED: ReasonCodeId = ReasonCodeId(0x4F53_0209);
    pub const PH1_OS_OCR_ROUTE_INVALID_INPUT: ReasonCodeId = ReasonCodeId(0x4F53_0301);
    pub const PH1_OS_OCR_ROUTE_PROVIDER_ERROR: ReasonCodeId = ReasonCodeId(0x4F53_0302);
    pub const PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4F53_0303);
    pub const PH1_OS_OCR_ROUTE_PROVIDER_LOW_CONFIDENCE: ReasonCodeId = ReasonCodeId(0x4F53_0304);
    pub const PH1_OS_OCR_ROUTE_CONTEXT_REFUSED: ReasonCodeId = ReasonCodeId(0x4F53_0305);
    pub const PH1_OS_OCR_ROUTE_NLP_REFUSED: ReasonCodeId = ReasonCodeId(0x4F53_0306);
    pub const PH1_OS_OCR_ROUTE_NLP_INPUT_INVALID: ReasonCodeId = ReasonCodeId(0x4F53_0307);
    pub const PH1_OS_OCR_ROUTE_CLARIFY_POLICY_BLOCK: ReasonCodeId = ReasonCodeId(0x4F53_0308);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1OsWiringConfig {
    pub os_enabled: bool,
    pub max_guard_failures: u8,
    pub max_diagnostics: u8,
    pub max_outcome_entries: u16,
}

impl Ph1OsWiringConfig {
    pub fn mvp_v1(os_enabled: bool) -> Self {
        Self {
            os_enabled,
            max_guard_failures: 8,
            max_diagnostics: 8,
            max_outcome_entries: 128,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_active: bool,
    pub transcript_ok: bool,
    pub nlp_confidence_high: bool,
    pub requires_confirmation: bool,
    pub confirmation_received: bool,
    pub prompt_policy_gate_ok: bool,
    pub tool_requested: bool,
    pub simulation_requested: bool,
    pub policy_gate_decision: OsGateDecision,
    pub tenant_gate_decision: OsGateDecision,
    pub gov_gate_decision: OsGateDecision,
    pub quota_gate_decision: OsGateDecision,
    pub work_gate_decision: OsGateDecision,
    pub capreq_gate_decision: OsGateDecision,
    pub access_allowed: bool,
    pub blueprint_active: bool,
    pub simulation_active: bool,
    pub idempotency_required: bool,
    pub idempotency_key_present: bool,
    pub lease_required: bool,
    pub lease_valid: bool,
    pub chat_requested: bool,
    pub clarify_required: bool,
    pub clarify_owner_engine_id: Option<String>,
    pub confirm_required: bool,
    pub explain_requested: bool,
    pub wait_required: bool,
    pub optional_budget_enforced: bool,
    pub optional_invocations_requested: u16,
    pub optional_invocations_budget: u16,
    pub optional_invocations_skipped_budget: u16,
    pub optional_latency_budget_ms: u32,
    pub optional_latency_estimated_ms: u32,
    pub outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
}

impl OsTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_gate_ok: bool,
        tool_requested: bool,
        simulation_requested: bool,
        access_allowed: bool,
        blueprint_active: bool,
        simulation_active: bool,
        idempotency_required: bool,
        idempotency_key_present: bool,
        lease_required: bool,
        lease_valid: bool,
        chat_requested: bool,
        clarify_required: bool,
        confirm_required: bool,
        explain_requested: bool,
        wait_required: bool,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_and_outcomes(
            correlation_id,
            turn_id,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_gate_ok,
            tool_requested,
            simulation_requested,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            access_allowed,
            blueprint_active,
            simulation_active,
            idempotency_required,
            idempotency_key_present,
            lease_required,
            lease_valid,
            chat_requested,
            clarify_required,
            confirm_required,
            explain_requested,
            wait_required,
            true,
            0,
            0,
            0,
            0,
            0,
            Vec::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_outcomes(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_gate_ok: bool,
        tool_requested: bool,
        simulation_requested: bool,
        access_allowed: bool,
        blueprint_active: bool,
        simulation_active: bool,
        idempotency_required: bool,
        idempotency_key_present: bool,
        lease_required: bool,
        lease_valid: bool,
        chat_requested: bool,
        clarify_required: bool,
        confirm_required: bool,
        explain_requested: bool,
        wait_required: bool,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_and_outcomes(
            correlation_id,
            turn_id,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_gate_ok,
            tool_requested,
            simulation_requested,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            access_allowed,
            blueprint_active,
            simulation_active,
            idempotency_required,
            idempotency_key_present,
            lease_required,
            lease_valid,
            chat_requested,
            clarify_required,
            confirm_required,
            explain_requested,
            wait_required,
            true,
            0,
            0,
            0,
            0,
            0,
            outcome_utilization_entries,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_governance_and_outcomes(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_gate_ok: bool,
        tool_requested: bool,
        simulation_requested: bool,
        policy_gate_decision: OsGateDecision,
        tenant_gate_decision: OsGateDecision,
        gov_gate_decision: OsGateDecision,
        quota_gate_decision: OsGateDecision,
        work_gate_decision: OsGateDecision,
        capreq_gate_decision: OsGateDecision,
        access_allowed: bool,
        blueprint_active: bool,
        simulation_active: bool,
        idempotency_required: bool,
        idempotency_key_present: bool,
        lease_required: bool,
        lease_valid: bool,
        chat_requested: bool,
        clarify_required: bool,
        confirm_required: bool,
        explain_requested: bool,
        wait_required: bool,
        optional_budget_enforced: bool,
        optional_invocations_requested: u16,
        optional_invocations_budget: u16,
        optional_invocations_skipped_budget: u16,
        optional_latency_budget_ms: u32,
        optional_latency_estimated_ms: u32,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_gate_ok,
            tool_requested,
            simulation_requested,
            policy_gate_decision,
            tenant_gate_decision,
            gov_gate_decision,
            quota_gate_decision,
            work_gate_decision,
            capreq_gate_decision,
            access_allowed,
            blueprint_active,
            simulation_active,
            idempotency_required,
            idempotency_key_present,
            lease_required,
            lease_valid,
            chat_requested,
            clarify_required,
            clarify_owner_engine_id: if clarify_required {
                Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string())
            } else {
                None
            },
            confirm_required,
            explain_requested,
            wait_required,
            optional_budget_enforced,
            optional_invocations_requested,
            optional_invocations_budget,
            optional_invocations_skipped_budget,
            optional_latency_budget_ms,
            optional_latency_estimated_ms,
            outcome_utilization_entries,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for OsTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.tool_requested && self.simulation_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.tool_requested",
                reason: "tool_requested and simulation_requested cannot both be true",
            });
        }
        if !self.clarify_required && !self.confirm_required && !self.prompt_policy_gate_ok {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.prompt_policy_gate_ok",
                reason: "must be true when clarify_required=false and confirm_required=false",
            });
        }
        if let Some(owner) = &self.clarify_owner_engine_id {
            if !is_engine_id_token(owner) {
                return Err(ContractViolation::InvalidValue {
                    field: "os_turn_input.clarify_owner_engine_id",
                    reason: "must be ASCII [A-Z0-9._] and <= 64 chars",
                });
            }
        }
        if !self.optional_budget_enforced {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_budget_enforced",
                reason: "must be true",
            });
        }
        if self.optional_invocations_requested > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_invocations_requested",
                reason: "must be <= 64",
            });
        }
        if self.optional_invocations_budget > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_invocations_budget",
                reason: "must be <= 64",
            });
        }
        if self.optional_invocations_skipped_budget > self.optional_invocations_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_invocations_skipped_budget",
                reason: "cannot exceed optional_invocations_requested",
            });
        }
        if self.optional_invocations_skipped_budget
            != self
                .optional_invocations_requested
                .saturating_sub(self.optional_invocations_budget)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_invocations_skipped_budget",
                reason: "must equal requested.saturating_sub(optional_invocations_budget)",
            });
        }
        if self.optional_latency_budget_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_latency_budget_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_latency_estimated_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > self.optional_latency_budget_ms {
            return Err(ContractViolation::InvalidValue {
                field: "os_turn_input.optional_latency_estimated_ms",
                reason: "must be <= optional_latency_budget_ms",
            });
        }
        for entry in &self.outcome_utilization_entries {
            entry.validate()?;
            if entry.correlation_id != self.correlation_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_turn_input.outcome_utilization_entries.correlation_id",
                    reason: "must match os_turn_input.correlation_id",
                });
            }
            if entry.turn_id != self.turn_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_turn_input.outcome_utilization_entries.turn_id",
                    reason: "must match os_turn_input.turn_id",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: OsPolicyEvaluateOk,
    pub decision_compute: OsDecisionComputeOk,
}

impl OsForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: OsPolicyEvaluateOk,
        decision_compute: OsDecisionComputeOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            policy_evaluate,
            decision_compute,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for OsForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;

        if self.decision_compute.next_move == OsNextMove::DispatchSimulation
            && !self.policy_evaluate.simulation_dispatch_allowed
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_forward_bundle.decision_compute.next_move",
                reason: "DispatchSimulation requires simulation_dispatch_allowed=true in policy",
            });
        }
        if self.decision_compute.next_move == OsNextMove::DispatchTool
            && !self.policy_evaluate.tool_dispatch_allowed
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_forward_bundle.decision_compute.next_move",
                reason: "DispatchTool requires tool_dispatch_allowed=true in policy",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsWiringOutcome {
    NotInvokedDisabled,
    Refused(OsRefuse),
    Forwarded(OsForwardBundle),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsTopLevelTurnPath {
    Voice,
    Text,
}

impl OsTopLevelTurnPath {
    pub fn as_str(self) -> &'static str {
        match self {
            OsTopLevelTurnPath::Voice => "VOICE",
            OsTopLevelTurnPath::Text => "TEXT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsVoicePlatform {
    Ios,
    Android,
    Desktop,
}

impl OsVoicePlatform {
    pub fn as_str(self) -> &'static str {
        match self {
            OsVoicePlatform::Ios => "IOS",
            OsVoicePlatform::Android => "ANDROID",
            OsVoicePlatform::Desktop => "DESKTOP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsVoiceTrigger {
    WakeWord,
    Explicit,
}

impl OsVoiceTrigger {
    pub fn as_str(self) -> &'static str {
        match self {
            OsVoiceTrigger::WakeWord => "WAKE_WORD",
            OsVoiceTrigger::Explicit => "EXPLICIT",
        }
    }

    pub fn wake_stage_required(self) -> bool {
        matches!(self, OsVoiceTrigger::WakeWord)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsVoiceTurnContext {
    pub platform: OsVoicePlatform,
    pub trigger: OsVoiceTrigger,
}

impl OsVoiceTurnContext {
    pub fn v1(platform: OsVoicePlatform, trigger: OsVoiceTrigger) -> Self {
        Self { platform, trigger }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsTopLevelTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub path: OsTopLevelTurnPath,
    pub voice_context: Option<OsVoiceTurnContext>,
    pub always_on_completed_sequence: Vec<String>,
    pub optional_requested: Vec<String>,
    pub max_optional_invocations: u8,
    pub os_turn_input: OsTurnInput,
}

impl OsTopLevelTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        path: OsTopLevelTurnPath,
        voice_context: Option<OsVoiceTurnContext>,
        always_on_completed_sequence: Vec<String>,
        optional_requested: Vec<String>,
        max_optional_invocations: u8,
        os_turn_input: OsTurnInput,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            path,
            voice_context,
            always_on_completed_sequence,
            optional_requested,
            max_optional_invocations,
            os_turn_input,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for OsTopLevelTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.os_turn_input.validate()?;

        if self.os_turn_input.correlation_id != self.correlation_id {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_turn_input.os_turn_input.correlation_id",
                reason: "must match os_top_level_turn_input.correlation_id",
            });
        }
        if self.os_turn_input.turn_id != self.turn_id {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_turn_input.os_turn_input.turn_id",
                reason: "must match os_top_level_turn_input.turn_id",
            });
        }
        match (self.path, self.voice_context) {
            (OsTopLevelTurnPath::Voice, None) => {
                return Err(ContractViolation::InvalidValue {
                    field: "os_top_level_turn_input.voice_context",
                    reason: "VOICE path requires voice_context",
                });
            }
            (OsTopLevelTurnPath::Text, Some(_)) => {
                return Err(ContractViolation::InvalidValue {
                    field: "os_top_level_turn_input.voice_context",
                    reason: "TEXT path must not carry voice_context",
                });
            }
            _ => {}
        }
        if self.max_optional_invocations > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_turn_input.max_optional_invocations",
                reason: "must be <= 64",
            });
        }
        if self.always_on_completed_sequence.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_turn_input.always_on_completed_sequence",
                reason: "must contain <= 16 engine ids",
            });
        }
        if self.optional_requested.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_turn_input.optional_requested",
                reason: "must contain <= 64 engine ids",
            });
        }
        ensure_engine_id_list_valid(
            "os_top_level_turn_input.always_on_completed_sequence",
            &self.always_on_completed_sequence,
        )?;
        ensure_engine_id_list_valid(
            "os_top_level_turn_input.optional_requested",
            &self.optional_requested,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1OsTopLevelConfig {
    pub orchestrator_enabled: bool,
    pub max_optional_invocations: u8,
    pub max_optional_latency_ms: u32,
}

impl Ph1OsTopLevelConfig {
    pub fn mvp_v1(orchestrator_enabled: bool) -> Self {
        Self {
            orchestrator_enabled,
            max_optional_invocations: 8,
            max_optional_latency_ms: 120,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsTopLevelForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub path: OsTopLevelTurnPath,
    pub always_on_sequence: Vec<String>,
    pub optional_sequence_invoked: Vec<String>,
    pub optional_sequence_skipped_budget: Vec<String>,
    pub os_bundle: OsForwardBundle,
}

impl OsTopLevelForwardBundle {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        path: OsTopLevelTurnPath,
        always_on_sequence: Vec<String>,
        optional_sequence_invoked: Vec<String>,
        optional_sequence_skipped_budget: Vec<String>,
        os_bundle: OsForwardBundle,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            path,
            always_on_sequence,
            optional_sequence_invoked,
            optional_sequence_skipped_budget,
            os_bundle,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for OsTopLevelForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.os_bundle.validate()?;

        if self.os_bundle.correlation_id != self.correlation_id {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_forward_bundle.os_bundle.correlation_id",
                reason: "must match os_top_level_forward_bundle.correlation_id",
            });
        }
        if self.os_bundle.turn_id != self.turn_id {
            return Err(ContractViolation::InvalidValue {
                field: "os_top_level_forward_bundle.os_bundle.turn_id",
                reason: "must match os_top_level_forward_bundle.turn_id",
            });
        }

        ensure_engine_id_list_valid(
            "os_top_level_forward_bundle.always_on_sequence",
            &self.always_on_sequence,
        )?;
        ensure_engine_id_list_valid(
            "os_top_level_forward_bundle.optional_sequence_invoked",
            &self.optional_sequence_invoked,
        )?;
        ensure_engine_id_list_valid(
            "os_top_level_forward_bundle.optional_sequence_skipped_budget",
            &self.optional_sequence_skipped_budget,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsTopLevelWiringOutcome {
    NotInvokedDisabled,
    Refused(OsRefuse),
    Forwarded(OsTopLevelForwardBundle),
}

pub trait Ph1OsEngine {
    fn run(&self, req: &Ph1OsRequest) -> Ph1OsResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1OsWiring<E>
where
    E: Ph1OsEngine,
{
    config: Ph1OsWiringConfig,
    engine: E,
}

impl<E> Ph1OsWiring<E>
where
    E: Ph1OsEngine,
{
    pub fn new(config: Ph1OsWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_guard_failures == 0 || config.max_guard_failures > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_wiring_config.max_guard_failures",
                reason: "must be within 1..=16",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if config.max_outcome_entries == 0 || config.max_outcome_entries > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_wiring_config.max_outcome_entries",
                reason: "must be within 1..=1024",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &OsTurnInput) -> Result<OsWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.os_enabled {
            return Ok(OsWiringOutcome::NotInvokedDisabled);
        }

        if input.clarify_required {
            if input.clarify_owner_engine_id.as_deref() != Some(OS_CLARIFY_OWNER_ENGINE_ID) {
                return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_VALIDATION_FAILED,
                    "clarify owner must be PH1.NLP when clarify_required=true".to_string(),
                )?));
            }
        } else if input.clarify_owner_engine_id.is_some() {
            return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_VALIDATION_FAILED,
                "clarify owner must be omitted when clarify_required=false".to_string(),
            )?));
        }

        let envelope = OsRequestEnvelope::v1_with_outcome_budget(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_guard_failures, 16),
            min(self.config.max_diagnostics, 16),
            min(self.config.max_outcome_entries, 1024u16),
        )?;

        let policy_req = Ph1OsRequest::OsPolicyEvaluate(
            OsPolicyEvaluateRequest::v1_with_governance_outcomes_and_optional_budget(
                envelope.clone(),
                input.session_active,
                input.transcript_ok,
                input.nlp_confidence_high,
                input.requires_confirmation,
                input.confirmation_received,
                input.clarify_required || input.confirm_required,
                input.prompt_policy_gate_ok,
                input.tool_requested,
                input.simulation_requested,
                input.policy_gate_decision,
                input.tenant_gate_decision,
                input.gov_gate_decision,
                input.quota_gate_decision,
                input.work_gate_decision,
                input.capreq_gate_decision,
                input.access_allowed,
                input.blueprint_active,
                input.simulation_active,
                input.idempotency_required,
                input.idempotency_key_present,
                input.lease_required,
                input.lease_valid,
                true,
                true,
                true,
                input.optional_budget_enforced,
                input.optional_invocations_requested,
                input.optional_invocations_budget,
                input.optional_invocations_skipped_budget,
                input.optional_latency_budget_ms,
                input.optional_latency_estimated_ms,
                input.outcome_utilization_entries.clone(),
            )?,
        );
        let policy_resp = self.engine.run(&policy_req);
        if policy_resp.validate().is_err() {
            return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsPolicyEvaluate,
                reason_codes::PH1_OS_VALIDATION_FAILED,
                "invalid os policy response contract".to_string(),
            )?));
        }

        let policy_ok = match policy_resp {
            Ph1OsResponse::Refuse(refuse) => return Ok(OsWiringOutcome::Refused(refuse)),
            Ph1OsResponse::OsPolicyEvaluateOk(ok) => ok,
            Ph1OsResponse::OsDecisionComputeOk(_) => {
                return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsPolicyEvaluate,
                    reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy-evaluate request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1OsRequest::OsDecisionCompute(OsDecisionComputeRequest::v1(
            envelope,
            policy_ok.clone(),
            input.chat_requested,
            input.clarify_required,
            input.clarify_owner_engine_id.clone(),
            input.confirm_required,
            input.explain_requested,
            input.wait_required,
            input.tool_requested,
            input.simulation_requested,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        if decision_resp.validate().is_err() {
            return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_VALIDATION_FAILED,
                "invalid os decision response contract".to_string(),
            )?));
        }

        let decision_ok = match decision_resp {
            Ph1OsResponse::Refuse(refuse) => return Ok(OsWiringOutcome::Refused(refuse)),
            Ph1OsResponse::OsDecisionComputeOk(ok) => ok,
            Ph1OsResponse::OsPolicyEvaluateOk(_) => {
                return Ok(OsWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision-compute request".to_string(),
                )?));
            }
        };

        let bundle =
            OsForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(OsWiringOutcome::Forwarded(bundle))
    }
}

#[derive(Debug, Clone)]
pub struct Ph1OsTopLevelWiring<E>
where
    E: Ph1OsEngine,
{
    config: Ph1OsTopLevelConfig,
    os_wiring: Ph1OsWiring<E>,
}

impl<E> Ph1OsTopLevelWiring<E>
where
    E: Ph1OsEngine,
{
    pub fn new(
        config: Ph1OsTopLevelConfig,
        os_wiring: Ph1OsWiring<E>,
    ) -> Result<Self, ContractViolation> {
        if config.max_optional_invocations == 0 || config.max_optional_invocations > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_top_level_config.max_optional_invocations",
                reason: "must be within 1..=64",
            });
        }
        if config.max_optional_latency_ms == 0 || config.max_optional_latency_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_top_level_config.max_optional_latency_ms",
                reason: "must be within 1..=60000",
            });
        }
        Ok(Self { config, os_wiring })
    }

    pub fn run_turn(
        &self,
        input: &OsTopLevelTurnInput,
    ) -> Result<OsTopLevelWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.orchestrator_enabled {
            return Ok(OsTopLevelWiringOutcome::NotInvokedDisabled);
        }

        if input.max_optional_invocations > self.config.max_optional_invocations {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_OPTIONAL_BUDGET_INVALID,
                "requested optional invocation budget exceeds configured max".to_string(),
            )?));
        }

        if input.os_turn_input.clarify_required {
            if input.os_turn_input.clarify_owner_engine_id.as_deref()
                != Some(OS_CLARIFY_OWNER_ENGINE_ID)
            {
                return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_TOPLEVEL_CLARIFY_OWNER_INVALID,
                    "clarify owner must be PH1.NLP when clarify_required=true".to_string(),
                )?));
            }
        } else if input.os_turn_input.clarify_owner_engine_id.is_some() {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_CLARIFY_OWNER_INVALID,
                "clarify owner must be omitted when clarify_required=false".to_string(),
            )?));
        }

        if let Some(engine_id) =
            first_runtime_forbidden_engine_id(&input.always_on_completed_sequence)
        {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_RUNTIME_BOUNDARY_VIOLATION,
                format!(
                    "runtime boundary violation: {} is OFFLINE_ONLY or control-plane only",
                    engine_id
                ),
            )?));
        }
        if let Some(engine_id) = first_runtime_forbidden_engine_id(&input.optional_requested) {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_RUNTIME_BOUNDARY_VIOLATION,
                format!(
                    "runtime boundary violation: {} is OFFLINE_ONLY or control-plane only",
                    engine_id
                ),
            )?));
        }

        let expected_always_on = expected_always_on_sequence(input.path, input.voice_context);
        if !matches_engine_order(&input.always_on_completed_sequence, expected_always_on) {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_SEQUENCE_INVALID,
                format!(
                    "always_on sequence mismatch for path {}; expected {:?}",
                    input.path.as_str(),
                    expected_always_on
                ),
            )?));
        }

        let requested_optional: BTreeSet<&str> = input
            .optional_requested
            .iter()
            .map(String::as_str)
            .collect();
        for engine_id in &requested_optional {
            if !turn_optional_sequence().contains(engine_id) {
                return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_TOPLEVEL_UNKNOWN_OPTIONAL_ENGINE,
                    format!("unknown turn-optional engine id {}", engine_id),
                )?));
            }
            if !optional_engine_allowed_by_policy(engine_id, &input.os_turn_input) {
                return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_TOPLEVEL_OPTIONAL_POLICY_BLOCK,
                    format!(
                        "optional engine {} is not allowed under current clarify policy posture",
                        engine_id
                    ),
                )?));
            }
        }

        let mut optional_sequence_invoked = Vec::new();
        let mut optional_sequence_skipped_budget = Vec::new();
        for engine_id in turn_optional_sequence() {
            if !requested_optional.contains(engine_id) {
                continue;
            }
            if optional_sequence_invoked.len() < input.max_optional_invocations as usize {
                optional_sequence_invoked.push((*engine_id).to_string());
            } else {
                optional_sequence_skipped_budget.push((*engine_id).to_string());
            }
        }

        const OPTIONAL_ENGINE_ESTIMATED_COST_MS: u32 = 20;
        let optional_invocations_requested = requested_optional.len() as u16;
        let optional_invocations_budget = input.max_optional_invocations as u16;
        let optional_invocations_skipped_budget = optional_sequence_skipped_budget.len() as u16;
        let optional_latency_budget_ms = self.config.max_optional_latency_ms;
        let optional_latency_estimated_ms =
            (optional_sequence_invoked.len() as u32) * OPTIONAL_ENGINE_ESTIMATED_COST_MS;
        if optional_latency_estimated_ms > optional_latency_budget_ms {
            return Ok(OsTopLevelWiringOutcome::Refused(OsRefuse::v1(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_TOPLEVEL_OPTIONAL_BUDGET_INVALID,
                "optional latency estimate exceeds configured budget".to_string(),
            )?));
        }

        let mut os_turn_input = input.os_turn_input.clone();
        os_turn_input.optional_budget_enforced = true;
        os_turn_input.optional_invocations_requested = optional_invocations_requested;
        os_turn_input.optional_invocations_budget = optional_invocations_budget;
        os_turn_input.optional_invocations_skipped_budget = optional_invocations_skipped_budget;
        os_turn_input.optional_latency_budget_ms = optional_latency_budget_ms;
        os_turn_input.optional_latency_estimated_ms = optional_latency_estimated_ms;

        let os_outcome = self.os_wiring.run_turn(&os_turn_input)?;
        let outcome = match os_outcome {
            OsWiringOutcome::NotInvokedDisabled => OsTopLevelWiringOutcome::NotInvokedDisabled,
            OsWiringOutcome::Refused(refuse) => OsTopLevelWiringOutcome::Refused(refuse),
            OsWiringOutcome::Forwarded(bundle) => {
                OsTopLevelWiringOutcome::Forwarded(OsTopLevelForwardBundle::v1(
                    input.correlation_id,
                    input.turn_id,
                    input.path,
                    input.always_on_completed_sequence.clone(),
                    optional_sequence_invoked,
                    optional_sequence_skipped_budget,
                    bundle,
                )?)
            }
        };

        Ok(outcome)
    }
}

#[derive(Debug, Clone)]
pub struct OsPh1kLiveEvidence {
    pub processed_stream_ref: AudioStreamRef,
    pub pre_roll_buffer_ref: PreRollBufferRef,
    pub vad_events: Vec<VadEvent>,
    pub device_state: Ph1kDeviceState,
    pub timing_stats: Option<Ph1kTimingStats>,
    pub tts_playback: Option<TtsPlaybackActiveEvent>,
    pub interrupt_candidate: Option<Ph1kInterruptCandidate>,
    pub duplex_frame: Option<DuplexFrame>,
}

impl Validate for OsPh1kLiveEvidence {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.processed_stream_ref.validate()?;
        self.pre_roll_buffer_ref.validate()?;
        if self.pre_roll_buffer_ref.stream_id != self.processed_stream_ref.stream_id {
            return Err(ContractViolation::InvalidValue {
                field: "os_ph1k_live_evidence.pre_roll_buffer_ref.stream_id",
                reason: "must match processed_stream_ref.stream_id",
            });
        }
        if self.vad_events.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "os_ph1k_live_evidence.vad_events",
                reason: "must not be empty",
            });
        }
        for event in &self.vad_events {
            event.validate()?;
            if event.stream_id != self.processed_stream_ref.stream_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_ph1k_live_evidence.vad_events[].stream_id",
                    reason: "must match processed_stream_ref.stream_id",
                });
            }
        }
        self.device_state.validate()?;
        if let Some(timing_stats) = self.timing_stats {
            timing_stats.validate()?;
        }
        if let Some(interrupt) = &self.interrupt_candidate {
            interrupt.timing_markers.validate()?;
            interrupt.speech_window_metrics.validate()?;
        }
        if let Some(duplex) = self.duplex_frame {
            duplex.validate()?;
            if duplex.stream_id != self.processed_stream_ref.stream_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_ph1k_live_evidence.duplex_frame.stream_id",
                    reason: "must match processed_stream_ref.stream_id",
                });
            }
            if duplex.pre_roll_buffer_id != self.pre_roll_buffer_ref.buffer_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_ph1k_live_evidence.duplex_frame.pre_roll_buffer_id",
                    reason: "must match pre_roll_buffer_ref.buffer_id",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct OsVoiceLiveTurnInput {
    pub top_level_turn_input: OsTopLevelTurnInput,
    pub voice_id_request: Ph1VoiceIdRequest,
    pub actor_user_id: UserId,
    pub tenant_id: Option<String>,
    pub device_id: Option<selene_kernel_contracts::ph1j::DeviceId>,
    pub enrolled_speakers: Vec<EngineEnrolledSpeaker>,
    pub observation: EngineVoiceIdObservation,
    pub ph1k_live_evidence: Option<OsPh1kLiveEvidence>,
}

impl OsVoiceLiveTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        top_level_turn_input: OsTopLevelTurnInput,
        voice_id_request: Ph1VoiceIdRequest,
        actor_user_id: UserId,
        tenant_id: Option<String>,
        device_id: Option<selene_kernel_contracts::ph1j::DeviceId>,
        enrolled_speakers: Vec<EngineEnrolledSpeaker>,
        observation: EngineVoiceIdObservation,
    ) -> Result<Self, ContractViolation> {
        top_level_turn_input.validate()?;
        voice_id_request.validate()?;
        if top_level_turn_input.path != OsTopLevelTurnPath::Voice {
            return Err(ContractViolation::InvalidValue {
                field: "os_voice_live_turn_input.top_level_turn_input.path",
                reason: "must be VOICE",
            });
        }
        if top_level_turn_input.voice_context.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "os_voice_live_turn_input.top_level_turn_input.voice_context",
                reason: "VOICE path requires voice_context",
            });
        }
        Ok(Self {
            top_level_turn_input,
            voice_id_request,
            actor_user_id,
            tenant_id,
            device_id,
            enrolled_speakers,
            observation,
            ph1k_live_evidence: None,
        })
    }

    pub fn with_ph1k_live_evidence(
        mut self,
        evidence: OsPh1kLiveEvidence,
    ) -> Result<Self, ContractViolation> {
        evidence.validate()?;
        self.ph1k_live_evidence = Some(evidence);
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OsVoiceLiveForwardBundle {
    pub top_level_bundle: OsTopLevelForwardBundle,
    pub voice_identity_assertion: Ph1VoiceIdResponse,
    pub identity_prompt_scope_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum OsVoiceLiveTurnOutcome {
    NotInvokedDisabled,
    Refused(OsRefuse),
    Forwarded(OsVoiceLiveForwardBundle),
}

#[derive(Debug, Clone)]
pub struct Ph1OsVoiceLiveRuntime<E>
where
    E: Ph1OsEngine,
{
    top_level_wiring: Ph1OsTopLevelWiring<E>,
    voice_id_live: Ph1VoiceIdLiveRuntime,
    device_sync_sender: DeviceArtifactSyncSenderRuntime,
}

fn ph1k_live_policy_block(evidence: &OsPh1kLiveEvidence) -> Option<(ReasonCodeId, String)> {
    if evidence.device_state.health == Ph1kDeviceHealth::Failed {
        return Some((
            reason_codes::PH1_OS_VOICE_PH1K_DEVICE_FAILED,
            "PH1.K reported failed device health; fail-closed until capture route recovers"
                .to_string(),
        ));
    }
    let candidate = evidence.interrupt_candidate.as_ref()?;
    if matches!(
        candidate.candidate_confidence_band,
        InterruptCandidateConfidenceBand::Low
    ) {
        return Some((
            reason_codes::PH1_OS_VOICE_PH1K_POLICY_BLOCK,
            "PH1.K interrupt candidate confidence LOW; clarify required before proceeding"
                .to_string(),
        ));
    }
    if matches!(
        candidate.risk_context_class,
        InterruptRiskContextClass::High
    ) {
        return Some((
            reason_codes::PH1_OS_VOICE_PH1K_POLICY_BLOCK,
            "PH1.K interrupt candidate risk_context HIGH; fail-closed for policy safety"
                .to_string(),
        ));
    }
    None
}

impl<E> Ph1OsVoiceLiveRuntime<E>
where
    E: Ph1OsEngine,
{
    pub fn new(
        top_level_wiring: Ph1OsTopLevelWiring<E>,
        voice_id_live: Ph1VoiceIdLiveRuntime,
    ) -> Self {
        Self {
            top_level_wiring,
            voice_id_live,
            device_sync_sender: DeviceArtifactSyncSenderRuntime::from_env_or_loopback(),
        }
    }

    pub fn with_device_sync_sender(mut self, sender: DeviceArtifactSyncSenderRuntime) -> Self {
        self.device_sync_sender = sender;
        self
    }

    pub fn run_turn(
        &self,
        store: &mut Ph1fStore,
        input: OsVoiceLiveTurnInput,
    ) -> Result<OsVoiceLiveTurnOutcome, StorageError> {
        let now = input.voice_id_request.now;
        let correlation_id = input.top_level_turn_input.correlation_id;
        let turn_id = input.top_level_turn_input.turn_id;
        let Some(ph1k_evidence) = input.ph1k_live_evidence.as_ref() else {
            self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)?;
            return Ok(OsVoiceLiveTurnOutcome::Refused(
                OsRefuse::v1(
                    OsCapabilityId::OsDecisionCompute,
                    reason_codes::PH1_OS_VOICE_PH1K_EVIDENCE_REQUIRED,
                    "PH1.K live evidence is required for voice runtime path".to_string(),
                )
                .map_err(StorageError::ContractViolation)?,
            ));
        };
        ph1k_evidence
            .validate()
            .map_err(StorageError::ContractViolation)?;
        if ph1k_evidence.processed_stream_ref.stream_id
            != input.voice_id_request.processed_audio_stream_ref.stream_id
        {
            return Err(StorageError::ContractViolation(
                ContractViolation::InvalidValue {
                    field:
                        "os_voice_live_turn_input.ph1k_live_evidence.processed_stream_ref.stream_id",
                    reason: "must match voice_id_request.processed_audio_stream_ref.stream_id",
                },
            ));
        }
        if let Some((reason_code, message)) = ph1k_live_policy_block(ph1k_evidence) {
            self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)?;
            return Ok(OsVoiceLiveTurnOutcome::Refused(
                OsRefuse::v1(OsCapabilityId::OsDecisionCompute, reason_code, message)
                    .map_err(StorageError::ContractViolation)?,
            ));
        }
        let top_level_outcome = self
            .top_level_wiring
            .run_turn(&input.top_level_turn_input)
            .map_err(StorageError::ContractViolation)?;
        let top_level_bundle = match top_level_outcome {
            OsTopLevelWiringOutcome::NotInvokedDisabled => {
                self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)?;
                return Ok(OsVoiceLiveTurnOutcome::NotInvokedDisabled);
            }
            OsTopLevelWiringOutcome::Refused(refuse) => {
                self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)?;
                return Ok(OsVoiceLiveTurnOutcome::Refused(refuse));
            }
            OsTopLevelWiringOutcome::Forwarded(bundle) => bundle,
        };

        let voice_context = input
            .top_level_turn_input
            .voice_context
            .expect("validated in OsVoiceLiveTurnInput::v1");
        let identity_prompt_scope_key = Some(voice_identity_prompt_scope_key(
            input.tenant_id.as_deref(),
            &input.actor_user_id,
            input.device_id.as_ref(),
            voice_context,
        ));
        let signal_scope = VoiceIdentitySignalScope::v1(
            input.voice_id_request.now,
            input.top_level_turn_input.correlation_id,
            input.top_level_turn_input.turn_id,
            input.actor_user_id,
            input.tenant_id.clone(),
            input.device_id,
        );
        let voice_context = voice_identity_runtime_context(voice_context, input.tenant_id);
        let governed_voice_runtime = self
            .voice_id_live
            .with_governed_threshold_pack_overrides(store);
        let voice_identity_assertion = governed_voice_runtime
            .run_identity_assertion_with_signal_emission(
                store,
                &input.voice_id_request,
                voice_context,
                input.enrolled_speakers,
                input.observation,
                signal_scope,
            )?;
        self.run_device_artifact_sync_worker_pass(store, now, correlation_id, turn_id)?;

        Ok(OsVoiceLiveTurnOutcome::Forwarded(
            OsVoiceLiveForwardBundle {
                top_level_bundle,
                voice_identity_assertion,
                identity_prompt_scope_key,
            },
        ))
    }

    fn run_device_artifact_sync_worker_pass(
        &self,
        store: &mut Ph1fStore,
        now: selene_kernel_contracts::MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> Result<(), StorageError> {
        let worker_id = format!("os_device_sync_worker_{}_{}", correlation_id.0, turn_id.0);
        device_artifact_sync::run_device_artifact_sync_worker_pass(
            store,
            now,
            worker_id,
            &self.device_sync_sender,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OsSelfHealChainInput {
    pub feedback_event: FeedbackEventRecord,
    pub feedback_bundle: FeedbackForwardBundle,
    pub learn_turn_input: LearnTurnInput,
    pub learn_bundle: LearnForwardBundle,
    pub pae_turn_input: PaeTurnInput,
    pub pae_bundle: PaeForwardBundle,
    pub owner_engine: String,
    pub first_seen_at: MonotonicTimeNs,
    pub last_seen_at: MonotonicTimeNs,
    pub containment_action: FailureContainmentAction,
    pub escalation_required: bool,
    pub unresolved_reason: Option<String>,
    pub bcast_id: Option<String>,
    pub provider_context: Option<FailureProviderContext>,
    pub governance_required: bool,
    pub governance_ticket_ref: Option<String>,
    pub approved_by: Option<String>,
    pub evaluated_at: MonotonicTimeNs,
    pub ph1c_superiority_pack: Option<SuperiorityEvalPack>,
}

pub fn build_self_heal_chain_from_engine_outputs(
    input: &OsSelfHealChainInput,
) -> Result<SelfHealCardChain, ContractViolation> {
    if !is_engine_id_token(&input.owner_engine) {
        return Err(ContractViolation::InvalidValue {
            field: "os_self_heal_chain_input.owner_engine",
            reason: "must be ASCII [A-Z0-9._] and <= 64 chars",
        });
    }

    let failure_event = map_feedback_event_to_failure_event(
        &input.feedback_event,
        &input.feedback_bundle,
        input.containment_action,
        input.escalation_required,
        input.unresolved_reason.clone(),
        input.provider_context.clone(),
    )?;

    let escalation_state = ProblemCardEscalationState {
        state: if input.escalation_required {
            ProblemCardState::EscalatedOpen
        } else {
            ProblemCardState::Open
        },
        requires_human: input.escalation_required,
        bcast_id: input.bcast_id.clone(),
        unresolved_reason: input.unresolved_reason.clone(),
    };

    let problem_card = map_learn_bundle_to_problem_card(
        &failure_event,
        &input.learn_turn_input,
        &input.learn_bundle,
        input.owner_engine.clone(),
        input.first_seen_at,
        input.last_seen_at,
        escalation_state,
    )?;

    let fix_card = map_learn_bundle_to_fix_card(&problem_card, &input.learn_bundle)?;
    let ph1c_superiority_report = evaluate_ph1c_superiority_gate_for_chain(input)?;

    let promotion_decision = map_pae_bundle_to_promotion_decision(
        &fix_card,
        &input.pae_turn_input,
        &input.pae_bundle,
        input.governance_required,
        input.governance_ticket_ref.clone(),
        input.approved_by.clone(),
        input.evaluated_at,
    )?;
    if let Some(report) = ph1c_superiority_report.as_ref() {
        ensure_ph1c_lane_alignment_with_promotion_decision(
            &promotion_decision.decision_action,
            report.recommended_runtime_lane,
        )?;
    }

    SelfHealCardChain::v1(failure_event, problem_card, fix_card, promotion_decision)
}

fn evaluate_ph1c_superiority_gate_for_chain(
    input: &OsSelfHealChainInput,
) -> Result<Option<SuperiorityGateReport>, ContractViolation> {
    if input.owner_engine != "PH1.C" {
        return Ok(None);
    }
    let pack = input
        .ph1c_superiority_pack
        .as_ref()
        .ok_or(ContractViolation::InvalidValue {
            field: "os_self_heal_chain_input.ph1c_superiority_pack",
            reason: "must be present when owner_engine=PH1.C",
        })?;
    let report = evaluate_ph1c_superiority_pack(pack)?;
    if !report.overall_pass || report.rollback_required {
        return Err(ContractViolation::InvalidValue {
            field: "build_self_heal_chain_from_engine_outputs.ph1c_superiority_gate",
            reason: "ph1c superiority gate must pass with rollback_required=false",
        });
    }
    Ok(Some(report))
}

fn ensure_ph1c_lane_alignment_with_promotion_decision(
    decision_action: &PromotionDecisionAction,
    recommended_lane: SuperiorityLane,
) -> Result<(), ContractViolation> {
    match recommended_lane {
        SuperiorityLane::SeleneBaseline => {
            if matches!(decision_action, PromotionDecisionAction::Promote) {
                return Err(ContractViolation::InvalidValue {
                    field: "build_self_heal_chain_from_engine_outputs.promotion_decision.decision_action",
                    reason: "PH1.C baseline recommendation forbids promote action",
                });
            }
        }
        SuperiorityLane::SeleneChallenger => {
            // Challenger recommendation is advisory for lane preference;
            // demotion/rollback may still be required by independent PAE safety signals.
        }
        SuperiorityLane::ChatgptAb => {
            return Err(ContractViolation::InvalidValue {
                field: "build_self_heal_chain_from_engine_outputs.ph1c_superiority_gate",
                reason: "runtime recommendation lane must be SELENE_BASELINE or SELENE_CHALLENGER",
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsSelfHealReleaseGateConfig {
    pub max_problem_age_ns: u64,
    pub max_decision_age_ns: u64,
}

impl OsSelfHealReleaseGateConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_problem_age_ns: 7 * 24 * 60 * 60 * 1_000_000_000,
            max_decision_age_ns: 24 * 60 * 60 * 1_000_000_000,
        }
    }
}

pub fn check_self_heal_release_gate(
    chain: &SelfHealCardChain,
    now: MonotonicTimeNs,
    config: OsSelfHealReleaseGateConfig,
) -> Result<(), ContractViolation> {
    chain.validate()?;
    if now.0 < chain.problem_card.last_seen_at.0 {
        return Err(ContractViolation::InvalidValue {
            field: "check_self_heal_release_gate.now",
            reason: "must be >= problem_card.last_seen_at",
        });
    }
    if now.0 < chain.promotion_decision.evaluated_at.0 {
        return Err(ContractViolation::InvalidValue {
            field: "check_self_heal_release_gate.now",
            reason: "must be >= promotion_decision.evaluated_at",
        });
    }

    let problem_age_ns = now.0.saturating_sub(chain.problem_card.last_seen_at.0);
    if problem_age_ns > config.max_problem_age_ns {
        return Err(ContractViolation::InvalidValue {
            field: "check_self_heal_release_gate.problem_card.last_seen_at",
            reason: "stale problem_card evidence blocks promotion",
        });
    }

    let decision_age_ns = now
        .0
        .saturating_sub(chain.promotion_decision.evaluated_at.0);
    if decision_age_ns > config.max_decision_age_ns {
        return Err(ContractViolation::InvalidValue {
            field: "check_self_heal_release_gate.promotion_decision.evaluated_at",
            reason: "stale promotion decision blocks promotion",
        });
    }

    if chain.failure_event.escalation_required && chain.problem_card.bcast_id.is_none() {
        return Err(ContractViolation::InvalidValue {
            field: "check_self_heal_release_gate.problem_card.bcast_id",
            reason: "escalation-required chain must include bcast_id proof link",
        });
    }

    if matches!(
        chain.promotion_decision.decision_action,
        PromotionDecisionAction::Promote
    ) {
        if !chain.promotion_decision.promotion_eligible || !chain.promotion_decision.rollback_ready
        {
            return Err(ContractViolation::InvalidValue {
                field: "check_self_heal_release_gate.promotion_decision",
                reason: "PROMOTE requires promotion_eligible=true and rollback_ready=true",
            });
        }
        if chain.problem_card.requires_human
            || matches!(chain.problem_card.state, ProblemCardState::EscalatedOpen)
        {
            return Err(ContractViolation::InvalidValue {
                field: "check_self_heal_release_gate.problem_card.state",
                reason: "PROMOTE blocked while problem requires human escalation",
            });
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsBuilderRemediationConfig {
    pub min_recurrence_count: u32,
}

impl OsBuilderRemediationConfig {
    pub fn mvp_v1() -> Self {
        Self {
            min_recurrence_count: 3,
        }
    }

    fn validate(self) -> Result<(), ContractViolation> {
        if self.min_recurrence_count < 2 || self.min_recurrence_count > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_builder_remediation_config.min_recurrence_count",
                reason: "must be within 2..=10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsBuilderPromotionGateEvidence {
    pub code_permission_gate_passed: bool,
    pub launch_permission_gate_passed: bool,
    pub release_hard_gate_passed: bool,
}

impl OsBuilderPromotionGateEvidence {
    pub fn strict_required_pass() -> Self {
        Self {
            code_permission_gate_passed: true,
            launch_permission_gate_passed: true,
            release_hard_gate_passed: true,
        }
    }
}

pub fn check_builder_remediation_promotion_gate(
    chain: &SelfHealCardChain,
    gate_evidence: OsBuilderPromotionGateEvidence,
) -> Result<(), ContractViolation> {
    chain.validate()?;

    if chain.promotion_decision.decision_action != PromotionDecisionAction::Promote {
        return Ok(());
    }

    if !gate_evidence.code_permission_gate_passed {
        return Err(ContractViolation::InvalidValue {
            field: "check_builder_remediation_promotion_gate.code_permission_gate_passed",
            reason: "promotion requires code permission gate evidence",
        });
    }
    if !gate_evidence.launch_permission_gate_passed {
        return Err(ContractViolation::InvalidValue {
            field: "check_builder_remediation_promotion_gate.launch_permission_gate_passed",
            reason: "promotion requires launch permission gate evidence",
        });
    }
    if !gate_evidence.release_hard_gate_passed {
        return Err(ContractViolation::InvalidValue {
            field: "check_builder_remediation_promotion_gate.release_hard_gate_passed",
            reason: "promotion requires release hard-gate evidence",
        });
    }

    Ok(())
}

pub fn map_recurring_failure_cluster_to_builder_offline_input(
    chain: &SelfHealCardChain,
    now: MonotonicTimeNs,
    config: OsBuilderRemediationConfig,
    gate_evidence: OsBuilderPromotionGateEvidence,
) -> Result<Option<BuilderOfflineInput>, ContractViolation> {
    chain.validate()?;
    config.validate()?;

    if chain.problem_card.recurrence_count < config.min_recurrence_count {
        return Ok(None);
    }
    if matches!(chain.problem_card.state, ProblemCardState::Resolved) {
        return Ok(None);
    }

    if now.0 < chain.problem_card.last_seen_at.0 {
        return Err(ContractViolation::InvalidValue {
            field: "map_recurring_failure_cluster_to_builder_offline_input.now",
            reason: "must be >= problem_card.last_seen_at",
        });
    }

    check_builder_remediation_promotion_gate(chain, gate_evidence)?;

    let correlation_id = chain.failure_event.correlation_id;
    let turn_id = chain.failure_event.turn_id;

    let feedback_entry = OsOutcomeUtilizationEntry::v1(
        "PH1.FEEDBACK".to_string(),
        "FAILURE_CLUSTER_RECURRING".to_string(),
        correlation_id,
        turn_id,
        action_class_for_containment(chain.failure_event.containment_action),
        "PH1.LEARN".to_string(),
        chain.failure_event.latency_ms.min(60_000),
        false,
        chain.failure_event.reason_code,
    )?;

    let learn_entry = OsOutcomeUtilizationEntry::v1(
        "PH1.LEARN".to_string(),
        "FIX_CARD_CLUSTER_CANDIDATE".to_string(),
        correlation_id,
        turn_id,
        OsOutcomeActionClass::QueueLearn,
        "PH1.BUILDER".to_string(),
        quality_impact_proxy_latency_ms(chain.problem_card.quality_impact_bp),
        true,
        chain.failure_event.reason_code,
    )?;

    let pae_entry = OsOutcomeUtilizationEntry::v1(
        "PH1.PAE".to_string(),
        format!(
            "PROMOTION_DECISION_{}",
            promotion_action_key(chain.promotion_decision.decision_action)
        ),
        correlation_id,
        turn_id,
        OsOutcomeActionClass::QueueLearn,
        "PH1.BUILDER".to_string(),
        1,
        true,
        chain.promotion_decision.reason_code,
    )?;

    let os_entry = OsOutcomeUtilizationEntry::v1(
        "PH1.OS".to_string(),
        "BUILDER_REMEDIATION_ENQUEUE".to_string(),
        correlation_id,
        turn_id,
        OsOutcomeActionClass::QueueLearn,
        "PH1.BUILDER".to_string(),
        1,
        true,
        chain.promotion_decision.reason_code,
    )?;

    let recurrence_count_token = chain.problem_card.recurrence_count.to_string();
    let source_signal_hash = stable_card_id(
        "cluster_hash",
        &[
            chain.problem_card.problem_id.as_str(),
            chain.problem_card.fingerprint.as_str(),
            chain.problem_card.latest_failure_id.as_str(),
            recurrence_count_token.as_str(),
        ],
    )?;
    let proposal_idempotency_key = stable_card_id(
        "builder_prop_idem",
        &[
            chain.problem_card.problem_id.as_str(),
            chain.fix_card.fix_id.as_str(),
            chain.promotion_decision.decision_id.as_str(),
        ],
    )?;
    let validation_run_idempotency_key = stable_card_id(
        "builder_run_idem",
        &[
            chain.fix_card.fix_id.as_str(),
            chain.promotion_decision.decision_id.as_str(),
        ],
    )?;

    let builder_input = BuilderOfflineInput::v1(
        correlation_id,
        turn_id,
        chain.problem_card.first_seen_at,
        chain.problem_card.last_seen_at,
        now,
        vec![feedback_entry, learn_entry, pae_entry, os_entry],
        Some(source_signal_hash),
        Some(proposal_idempotency_key),
        Some(validation_run_idempotency_key),
        None,
        None,
        None,
        true,
    )?;

    Ok(Some(builder_input))
}

fn action_class_for_containment(action: FailureContainmentAction) -> OsOutcomeActionClass {
    match action {
        FailureContainmentAction::ObservedOnly => OsOutcomeActionClass::AuditOnly,
        FailureContainmentAction::FailClosedRefuse
        | FailureContainmentAction::ClarifyRequired
        | FailureContainmentAction::RetryScheduled
        | FailureContainmentAction::Escalated => OsOutcomeActionClass::QueueLearn,
    }
}

fn promotion_action_key(action: PromotionDecisionAction) -> &'static str {
    match action {
        PromotionDecisionAction::Promote => "PROMOTE",
        PromotionDecisionAction::Demote => "DEMOTE",
        PromotionDecisionAction::Hold => "HOLD",
        PromotionDecisionAction::Rollback => "ROLLBACK",
    }
}

fn quality_impact_proxy_latency_ms(quality_impact_bp: i16) -> u32 {
    let quality_abs_bp = i32::from(quality_impact_bp).unsigned_abs().min(20_000);
    (100 + quality_abs_bp).min(60_000)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsOcrSourceEngine {
    Vision,
    Doc,
}

impl OsOcrSourceEngine {
    pub fn as_engine_id(self) -> &'static str {
        match self {
            OsOcrSourceEngine::Vision => "PH1.VISION",
            OsOcrSourceEngine::Doc => "PH1.DOC",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsOcrAnalyzerForwardBundle {
    Vision(VisionForwardBundle),
    Doc(DocForwardBundle),
}

impl OsOcrAnalyzerForwardBundle {
    pub fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(bundle) => bundle.validate(),
            OsOcrAnalyzerForwardBundle::Doc(bundle) => bundle.validate(),
        }
    }

    pub fn correlation_id(&self) -> CorrelationId {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(bundle) => bundle.correlation_id,
            OsOcrAnalyzerForwardBundle::Doc(bundle) => bundle.correlation_id,
        }
    }

    pub fn turn_id(&self) -> TurnId {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(bundle) => bundle.turn_id,
            OsOcrAnalyzerForwardBundle::Doc(bundle) => bundle.turn_id,
        }
    }

    fn source_engine(&self) -> OsOcrSourceEngine {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(_) => OsOcrSourceEngine::Vision,
            OsOcrAnalyzerForwardBundle::Doc(_) => OsOcrSourceEngine::Doc,
        }
    }

    fn input_payload_kind(&self) -> Ph1dProviderInputPayloadKind {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(_) => Ph1dProviderInputPayloadKind::Image,
            OsOcrAnalyzerForwardBundle::Doc(_) => Ph1dProviderInputPayloadKind::Document,
        }
    }

    fn input_payload_ref(&self) -> String {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(bundle) => {
                format!("ocr:vision:{}", bundle.source_ref.source_id.as_str())
            }
            OsOcrAnalyzerForwardBundle::Doc(bundle) => {
                format!("ocr:doc:{}", bundle.source_ref.source_id.as_str())
            }
        }
    }

    fn input_payload_inline(&self) -> String {
        match self {
            OsOcrAnalyzerForwardBundle::Vision(bundle) => vision_ocr_payload(bundle).to_string(),
            OsOcrAnalyzerForwardBundle::Doc(bundle) => doc_ocr_payload(bundle).to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1OsOcrRouteConfig {
    pub route_enabled: bool,
    pub tenant_id: String,
    pub provider_route_class: Ph1dProviderRouteClass,
    pub provider_id: String,
    pub model_id: String,
    pub timeout_ms: u32,
    pub retry_budget: u8,
    pub min_provider_confidence_bp: u16,
    pub safety_tier: SafetyTier,
    pub privacy_mode: bool,
    pub do_not_disturb: bool,
}

impl Ph1OsOcrRouteConfig {
    pub fn openai_default() -> Self {
        Self {
            route_enabled: true,
            tenant_id: "tenant_default".to_string(),
            provider_route_class: Ph1dProviderRouteClass::Primary,
            provider_id: "openai".to_string(),
            model_id: std::env::var("PH1D_OPENAI_MODEL")
                .unwrap_or_else(|_| "gpt-4o-mini".to_string()),
            timeout_ms: 15_000,
            retry_budget: 1,
            min_provider_confidence_bp: 0,
            safety_tier: SafetyTier::Standard,
            privacy_mode: false,
            do_not_disturb: false,
        }
    }

    pub fn validate(&self) -> Result<(), ContractViolation> {
        if self.tenant_id.trim().is_empty()
            || self.tenant_id.len() > 128
            || !is_provider_token(&self.tenant_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.tenant_id",
                reason: "must be non-empty provider token and <= 128 chars",
            });
        }
        if self.provider_id.trim().is_empty()
            || self.provider_id.len() > 64
            || !is_provider_token(&self.provider_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.provider_id",
                reason: "must be non-empty provider token and <= 64 chars",
            });
        }
        if self.model_id.trim().is_empty()
            || self.model_id.len() > 128
            || !is_provider_token(&self.model_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.model_id",
                reason: "must be non-empty provider token and <= 128 chars",
            });
        }
        if !(100..=60_000).contains(&self.timeout_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.timeout_ms",
                reason: "must be within 100..=60000",
            });
        }
        if self.retry_budget > 10 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.retry_budget",
                reason: "must be <= 10",
            });
        }
        if self.min_provider_confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_route_config.min_provider_confidence_bp",
                reason: "must be <= 10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsOcrProviderForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_engine: OsOcrSourceEngine,
    pub provider_call: Ph1dProviderCallResponse,
    pub extracted_text: String,
}

impl OsOcrProviderForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_engine: OsOcrSourceEngine,
        provider_call: Ph1dProviderCallResponse,
        extracted_text: String,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            source_engine,
            provider_call,
            extracted_text,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for OsOcrProviderForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.provider_call.validate()?;
        if u128::from(self.provider_call.correlation_id) != self.correlation_id.0 {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.provider_call.correlation_id",
                reason: "must match os_ocr_provider_forward_bundle.correlation_id",
            });
        }
        if self.provider_call.turn_id != self.turn_id.0 {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.provider_call.turn_id",
                reason: "must match os_ocr_provider_forward_bundle.turn_id",
            });
        }
        if !matches!(
            self.provider_call.provider_task,
            Ph1dProviderTask::OcrTextExtract
        ) {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.provider_call.provider_task",
                reason: "must be OCR_TEXT_EXTRACT",
            });
        }
        if self.provider_call.provider_status != Ph1dProviderStatus::Ok
            || self.provider_call.validation_status != Ph1dProviderValidationStatus::SchemaOk
            || self.provider_call.normalized_output_schema_hash
                != Some(PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.provider_call.validation_status",
                reason: "provider call must be provider_status=OK, validation_status=SCHEMA_OK, and normalized_output_schema_hash=v1",
            });
        }
        if self.extracted_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.extracted_text",
                reason: "must not be empty",
            });
        }
        if self.extracted_text.len() > 65_536 {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_provider_forward_bundle.extracted_text",
                reason: "must be <= 65536 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OsOcrRouteOutcome {
    NotInvokedDisabled,
    Refused(OsRefuse),
    Forwarded(OsOcrProviderForwardBundle),
}

#[derive(Debug, Clone)]
pub struct Ph1OsOcrRouteWiring<A>
where
    A: Ph1dProviderAdapter,
{
    config: Ph1OsOcrRouteConfig,
    adapter: A,
}

impl<A> Ph1OsOcrRouteWiring<A>
where
    A: Ph1dProviderAdapter,
{
    pub fn new(config: Ph1OsOcrRouteConfig, adapter: A) -> Result<Self, ContractViolation> {
        config.validate()?;
        Ok(Self { config, adapter })
    }

    pub fn run_handoff(
        &self,
        analyzer_bundle: &OsOcrAnalyzerForwardBundle,
    ) -> Result<OsOcrRouteOutcome, ContractViolation> {
        if !self.config.route_enabled {
            return Ok(OsOcrRouteOutcome::NotInvokedDisabled);
        }

        if analyzer_bundle.validate().is_err() {
            return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_INVALID_INPUT,
                "ocr route input failed analyzer contract validation".to_string(),
            )?));
        }

        let source_engine = analyzer_bundle.source_engine();
        let provider_req = build_ocr_provider_request(&self.config, analyzer_bundle)?;
        let provider_resp = match self.adapter.execute(&provider_req) {
            Ok(resp) => resp,
            Err(Ph1dProviderAdapterError { .. }) => {
                return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_ERROR,
                    "ocr provider transport/adapter error".to_string(),
                )?))
            }
        };

        if provider_resp.validate().is_err()
            || provider_resp.correlation_id != provider_req.correlation_id
            || provider_resp.turn_id != provider_req.turn_id
            || provider_resp.request_id != provider_req.request_id
            || provider_resp.idempotency_key != provider_req.idempotency_key
            || provider_resp.provider_task != provider_req.provider_task
            || provider_resp.provider_id != provider_req.provider_id
            || provider_resp.model_id != provider_req.model_id
        {
            return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED,
                "ocr provider response failed contract/idempotency validation".to_string(),
            )?));
        }

        if provider_resp.provider_status != Ph1dProviderStatus::Ok
            || provider_resp.validation_status != Ph1dProviderValidationStatus::SchemaOk
            || provider_resp.normalized_output_schema_hash != Some(provider_req.output_schema_hash)
        {
            return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED,
                "ocr provider response must be provider_status=OK, validation_status=SCHEMA_OK, and normalized schema hash must match request output schema hash".to_string(),
            )?));
        }

        if self.config.min_provider_confidence_bp > 0 {
            let Some(confidence_bp) = provider_resp.provider_confidence_bp else {
                return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_LOW_CONFIDENCE,
                    "ocr provider confidence missing under active confidence floor".to_string(),
                )?));
            };
            if confidence_bp < self.config.min_provider_confidence_bp {
                return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_LOW_CONFIDENCE,
                    "ocr provider confidence below policy floor".to_string(),
                )?));
            }
        }

        let extracted_text = provider_resp
            .normalized_output_json
            .as_deref()
            .and_then(extract_ocr_text_from_normalized_json);
        let Some(extracted_text) = extracted_text else {
            return Ok(OsOcrRouteOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED,
                "ocr provider normalized output missing extractable text".to_string(),
            )?));
        };

        let bundle = OsOcrProviderForwardBundle::v1(
            analyzer_bundle.correlation_id(),
            analyzer_bundle.turn_id(),
            source_engine,
            provider_resp,
            extracted_text,
        )?;
        Ok(OsOcrRouteOutcome::Forwarded(bundle))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1OsOcrContextNlpConfig {
    pub bridge_enabled: bool,
    pub context_intent_type: String,
    pub context_source_engine_id: String,
    pub context_rank_score_bp: i16,
    pub context_sensitivity_private: bool,
    pub max_ocr_append_chars: u16,
    pub high_confidence_min_bp: u16,
    pub medium_confidence_min_bp: u16,
    pub missing_provider_confidence_band: OsOcrConfidenceBand,
    pub medium_confidence_requires_clarify: bool,
    pub low_confidence_requires_clarify: bool,
}

impl Ph1OsOcrContextNlpConfig {
    pub fn mvp_v1() -> Self {
        Self {
            bridge_enabled: true,
            context_intent_type: "OCR_TEXT_EXTRACT".to_string(),
            context_source_engine_id: "PH1.D".to_string(),
            context_rank_score_bp: 1500,
            context_sensitivity_private: true,
            max_ocr_append_chars: 2048,
            high_confidence_min_bp: 8500,
            medium_confidence_min_bp: 5500,
            missing_provider_confidence_band: OsOcrConfidenceBand::Medium,
            medium_confidence_requires_clarify: false,
            low_confidence_requires_clarify: true,
        }
    }

    pub fn validate(&self) -> Result<(), ContractViolation> {
        if self.context_intent_type.trim().is_empty()
            || self.context_intent_type.len() > 96
            || self.context_intent_type.chars().any(|c| c.is_control())
        {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.context_intent_type",
                reason: "must be non-empty, <= 96 chars, and contain no control chars",
            });
        }
        if !is_engine_id_token(&self.context_source_engine_id) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.context_source_engine_id",
                reason: "must be ASCII [A-Z0-9._] and <= 64 chars",
            });
        }
        if !(-20_000..=20_000).contains(&self.context_rank_score_bp) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.context_rank_score_bp",
                reason: "must be within -20000..=20000",
            });
        }
        if self.max_ocr_append_chars == 0 || self.max_ocr_append_chars > 8192 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.max_ocr_append_chars",
                reason: "must be within 1..=8192",
            });
        }
        if self.high_confidence_min_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.high_confidence_min_bp",
                reason: "must be <= 10000",
            });
        }
        if self.medium_confidence_min_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.medium_confidence_min_bp",
                reason: "must be <= 10000",
            });
        }
        if self.medium_confidence_min_bp > self.high_confidence_min_bp {
            return Err(ContractViolation::InvalidValue {
                field: "ph1os_ocr_context_nlp_config.medium_confidence_min_bp",
                reason: "must be <= high_confidence_min_bp",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsOcrConfidenceBand {
    High,
    Medium,
    Low,
}

impl OsOcrConfidenceBand {
    pub fn as_str(self) -> &'static str {
        match self {
            OsOcrConfidenceBand::High => "HIGH",
            OsOcrConfidenceBand::Medium => "MEDIUM",
            OsOcrConfidenceBand::Low => "LOW",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct OsOcrContextNlpForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub ocr_provider_bundle: OsOcrProviderForwardBundle,
    pub confidence_band: OsOcrConfidenceBand,
    pub clarify_policy_required: bool,
    pub context_bundle: ContextForwardBundle,
    pub nlp_output: Ph1nResponse,
    pub nlp_fail_closed: bool,
}

impl OsOcrContextNlpForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        ocr_provider_bundle: OsOcrProviderForwardBundle,
        confidence_band: OsOcrConfidenceBand,
        clarify_policy_required: bool,
        context_bundle: ContextForwardBundle,
        nlp_output: Ph1nResponse,
        nlp_fail_closed: bool,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            ocr_provider_bundle,
            confidence_band,
            clarify_policy_required,
            context_bundle,
            nlp_output,
            nlp_fail_closed,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for OsOcrContextNlpForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.ocr_provider_bundle.validate()?;
        self.context_bundle.validate()?;
        if self.ocr_provider_bundle.correlation_id != self.correlation_id
            || self.ocr_provider_bundle.turn_id != self.turn_id
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_context_nlp_forward_bundle.ocr_provider_bundle",
                reason: "correlation/turn must match top-level bundle",
            });
        }
        if self.context_bundle.correlation_id != self.correlation_id
            || self.context_bundle.turn_id != self.turn_id
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_context_nlp_forward_bundle.context_bundle",
                reason: "correlation/turn must match top-level bundle",
            });
        }
        validate_nlp_response(&self.nlp_output)?;
        if self.clarify_policy_required && !matches!(self.nlp_output, Ph1nResponse::Clarify(_)) {
            return Err(ContractViolation::InvalidValue {
                field: "os_ocr_context_nlp_forward_bundle.nlp_output",
                reason: "must be clarify when clarify_policy_required=true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum OsOcrContextNlpOutcome {
    NotInvokedDisabled,
    Refused(OsRefuse),
    Forwarded(OsOcrContextNlpForwardBundle),
}

#[derive(Debug, Clone)]
pub struct Ph1OsOcrContextNlpWiring<CE, NE>
where
    CE: Ph1ContextEngine,
    NE: Ph1nEngine,
{
    config: Ph1OsOcrContextNlpConfig,
    context_wiring: Ph1ContextWiring<CE>,
    nlp_wiring: Ph1nWiring<NE>,
}

impl<CE, NE> Ph1OsOcrContextNlpWiring<CE, NE>
where
    CE: Ph1ContextEngine,
    NE: Ph1nEngine,
{
    pub fn new(
        config: Ph1OsOcrContextNlpConfig,
        context_wiring: Ph1ContextWiring<CE>,
        nlp_wiring: Ph1nWiring<NE>,
    ) -> Result<Self, ContractViolation> {
        config.validate()?;
        Ok(Self {
            config,
            context_wiring,
            nlp_wiring,
        })
    }

    pub fn run_handoff(
        &self,
        ocr_bundle: &OsOcrProviderForwardBundle,
        base_nlp_request: &Ph1nRequest,
    ) -> Result<OsOcrContextNlpOutcome, ContractViolation> {
        base_nlp_request.validate()?;

        if !self.config.bridge_enabled {
            return Ok(OsOcrContextNlpOutcome::NotInvokedDisabled);
        }

        if ocr_bundle.validate().is_err() {
            return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_INVALID_INPUT,
                "ocr->context/nlp handoff requires a validated OCR provider bundle".to_string(),
            )?));
        }
        let confidence_band = classify_ocr_confidence_band(&self.config, ocr_bundle);
        let clarify_policy_required =
            confidence_band_requires_clarify(&self.config, confidence_band);

        let context_input = match build_context_turn_input_from_ocr(&self.config, ocr_bundle) {
            Ok(v) => v,
            Err(_) => {
                return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_INVALID_INPUT,
                    "failed to construct context input from OCR bundle".to_string(),
                )?))
            }
        };

        let context_bundle = match self.context_wiring.run_turn(&context_input)? {
            ContextWiringOutcome::NotInvokedDisabled => {
                return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_CONTEXT_REFUSED,
                    "context wiring disabled during OCR handoff".to_string(),
                )?))
            }
            ContextWiringOutcome::NotInvokedNoContextInput => {
                return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_CONTEXT_REFUSED,
                    "context wiring produced no context input during OCR handoff".to_string(),
                )?))
            }
            ContextWiringOutcome::Refused(refuse) => {
                return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_CONTEXT_REFUSED,
                    format!(
                        "context wiring refused OCR handoff (reason_code={:?})",
                        refuse.reason_code
                    ),
                )?))
            }
            ContextWiringOutcome::Forwarded(bundle) => bundle,
        };

        let nlp_request =
            match build_nlp_request_from_ocr(&self.config, base_nlp_request, ocr_bundle) {
                Ok(v) => v,
                Err(_) => {
                    return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                        reason_codes::PH1_OS_OCR_ROUTE_NLP_INPUT_INVALID,
                        "failed to construct NLP request from OCR bundle".to_string(),
                    )?))
                }
            };

        let (nlp_output, nlp_fail_closed) = match self.nlp_wiring.run_turn(&nlp_request)? {
            Ph1nWiringOutcome::NotInvokedDisabled => {
                return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                    reason_codes::PH1_OS_OCR_ROUTE_NLP_REFUSED,
                    "NLP wiring disabled during OCR handoff".to_string(),
                )?))
            }
            Ph1nWiringOutcome::Refused(resp) => (resp, true),
            Ph1nWiringOutcome::Forwarded(resp) => (resp, false),
        };
        if clarify_policy_required && !matches!(nlp_output, Ph1nResponse::Clarify(_)) {
            return Ok(OsOcrContextNlpOutcome::Refused(os_ocr_refuse(
                reason_codes::PH1_OS_OCR_ROUTE_CLARIFY_POLICY_BLOCK,
                format!(
                    "OCR confidence band {} requires PH1.NLP clarify output",
                    confidence_band.as_str()
                ),
            )?));
        }

        let bundle = OsOcrContextNlpForwardBundle::v1(
            ocr_bundle.correlation_id,
            ocr_bundle.turn_id,
            ocr_bundle.clone(),
            confidence_band,
            clarify_policy_required,
            context_bundle,
            nlp_output,
            nlp_fail_closed,
        )?;
        Ok(OsOcrContextNlpOutcome::Forwarded(bundle))
    }
}

fn classify_ocr_confidence_band(
    config: &Ph1OsOcrContextNlpConfig,
    ocr_bundle: &OsOcrProviderForwardBundle,
) -> OsOcrConfidenceBand {
    match ocr_bundle.provider_call.provider_confidence_bp {
        Some(confidence_bp) if confidence_bp >= config.high_confidence_min_bp => {
            OsOcrConfidenceBand::High
        }
        Some(confidence_bp) if confidence_bp >= config.medium_confidence_min_bp => {
            OsOcrConfidenceBand::Medium
        }
        Some(_) => OsOcrConfidenceBand::Low,
        None => config.missing_provider_confidence_band,
    }
}

fn confidence_band_requires_clarify(
    config: &Ph1OsOcrContextNlpConfig,
    confidence_band: OsOcrConfidenceBand,
) -> bool {
    match confidence_band {
        OsOcrConfidenceBand::High => false,
        OsOcrConfidenceBand::Medium => config.medium_confidence_requires_clarify,
        OsOcrConfidenceBand::Low => config.low_confidence_requires_clarify,
    }
}

fn build_context_turn_input_from_ocr(
    config: &Ph1OsOcrContextNlpConfig,
    ocr_bundle: &OsOcrProviderForwardBundle,
) -> Result<ContextTurnInput, ContractViolation> {
    let source_kind = match ocr_bundle.source_engine {
        OsOcrSourceEngine::Vision => {
            selene_kernel_contracts::ph1context::ContextSourceKind::VisionEvidence
        }
        OsOcrSourceEngine::Doc => {
            selene_kernel_contracts::ph1context::ContextSourceKind::DocEvidence
        }
    };
    let item_hash_seed = format!(
        "ocr_ctx:{}:{}:{}:{}",
        ocr_bundle.correlation_id.0,
        ocr_bundle.turn_id.0,
        ocr_bundle.source_engine.as_engine_id(),
        ocr_bundle.extracted_text
    );
    let item_hash = stable_nonzero_hash_u64(item_hash_seed.as_bytes());
    let item_id = format!("ocr_ctx_{item_hash:016x}");
    let content_ref = format!("ocr_content:{item_hash:016x}");
    let provider_call_ref = ocr_bundle
        .provider_call
        .provider_call_id
        .as_deref()
        .unwrap_or("provider_call_unknown");
    let evidence_ref = format!("ocr_evidence:{provider_call_ref}");
    let source_item = selene_kernel_contracts::ph1context::ContextSourceItem::v1(
        item_id,
        ocr_bundle.source_engine.as_engine_id().to_string(),
        source_kind,
        config.context_rank_score_bp,
        content_ref,
        evidence_ref,
        config.context_sensitivity_private,
    )?;
    ContextTurnInput::v1(
        ocr_bundle.correlation_id,
        ocr_bundle.turn_id,
        config.context_intent_type.clone(),
        config.context_sensitivity_private,
        vec![source_item],
        true,
        true,
    )
}

fn build_nlp_request_from_ocr(
    config: &Ph1OsOcrContextNlpConfig,
    base_nlp_request: &Ph1nRequest,
    ocr_bundle: &OsOcrProviderForwardBundle,
) -> Result<Ph1nRequest, ContractViolation> {
    let ocr_text = ocr_bundle.extracted_text.trim();
    if ocr_text.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "ph1os_ocr_context_nlp.nlp_input.ocr_text",
            reason: "must not be empty",
        });
    }
    if ocr_text.chars().count() > config.max_ocr_append_chars as usize {
        return Err(ContractViolation::InvalidValue {
            field: "ph1os_ocr_context_nlp.nlp_input.ocr_text",
            reason: "exceeds max_ocr_append_chars",
        });
    }

    let source_label = match ocr_bundle.source_engine {
        OsOcrSourceEngine::Vision => "VISION",
        OsOcrSourceEngine::Doc => "DOC",
    };
    let mut merged_transcript = base_nlp_request.transcript_ok.transcript_text.clone();
    if !merged_transcript
        .chars()
        .last()
        .map(char::is_whitespace)
        .unwrap_or(false)
    {
        merged_transcript.push(' ');
    }
    merged_transcript.push_str("[OCR:");
    merged_transcript.push_str(source_label);
    merged_transcript.push_str("] ");
    merged_transcript.push_str(ocr_text);

    let merged_transcript_ok = TranscriptOk::v1_with_metadata(
        merged_transcript,
        base_nlp_request.transcript_ok.language_tag.clone(),
        base_nlp_request.transcript_ok.confidence_bucket,
        base_nlp_request.transcript_ok.uncertain_spans.clone(),
        base_nlp_request.transcript_ok.audit_meta.clone(),
    )?;

    let mut nlp_request = base_nlp_request.clone();
    nlp_request.transcript_ok = merged_transcript_ok;
    nlp_request.validate()?;
    Ok(nlp_request)
}

fn validate_nlp_response(resp: &Ph1nResponse) -> Result<(), ContractViolation> {
    match resp {
        Ph1nResponse::IntentDraft(v) => v.validate(),
        Ph1nResponse::Clarify(v) => v.validate(),
        Ph1nResponse::Chat(v) => v.validate(),
    }
}

fn build_ocr_provider_request(
    config: &Ph1OsOcrRouteConfig,
    analyzer_bundle: &OsOcrAnalyzerForwardBundle,
) -> Result<Ph1dProviderCallRequest, ContractViolation> {
    let correlation_id_u64 = u64::try_from(analyzer_bundle.correlation_id().0).map_err(|_| {
        ContractViolation::InvalidValue {
            field: "ph1os_ocr_route.correlation_id",
            reason: "must fit in u64 for PH1.D provider call envelope",
        }
    })?;
    let payload_ref = analyzer_bundle.input_payload_ref();
    let payload_inline = analyzer_bundle.input_payload_inline();
    let payload_hash = stable_schema_hash(payload_inline.as_bytes());
    let request_hash_seed = format!(
        "ph1os_ocr:req:{}:{}:{}:{:016x}",
        analyzer_bundle.source_engine().as_engine_id(),
        analyzer_bundle.correlation_id().0,
        analyzer_bundle.turn_id().0,
        payload_hash.0
    );
    let request_id = RequestId(stable_nonzero_hash_u64(request_hash_seed.as_bytes()));
    let idem_seed = format!(
        "ph1os_ocr:idem:{}:{}:{}:{:016x}",
        analyzer_bundle.source_engine().as_engine_id(),
        analyzer_bundle.correlation_id().0,
        analyzer_bundle.turn_id().0,
        payload_hash.0
    );
    let idempotency_key = format!(
        "ph1os_ocr:{:016x}",
        stable_nonzero_hash_u64(idem_seed.as_bytes())
    );

    let policy_seed = format!(
        "{}:{}:{}",
        config.safety_tier as u8, config.privacy_mode, config.do_not_disturb
    );

    Ph1dProviderCallRequest::v1(
        correlation_id_u64,
        analyzer_bundle.turn_id().0,
        config.tenant_id.clone(),
        request_id,
        idempotency_key,
        Ph1dProviderTask::OcrTextExtract,
        config.provider_route_class,
        config.provider_id.clone(),
        config.model_id.clone(),
        config.timeout_ms,
        config.retry_budget,
        None,
        None,
        SchemaVersion(1),
        PH1D_PROVIDER_NORMALIZED_OUTPUT_SCHEMA_HASH_V1,
        stable_schema_hash(b"ph1os_ocr_tool_catalog_none"),
        stable_schema_hash(policy_seed.as_bytes()),
        None,
        payload_ref,
        analyzer_bundle.input_payload_kind(),
        payload_hash,
        Some(payload_inline),
        Some("application/json".to_string()),
        config.safety_tier,
        config.privacy_mode,
        config.do_not_disturb,
    )
}

fn os_ocr_refuse(
    reason_code: selene_kernel_contracts::ReasonCodeId,
    reason: String,
) -> Result<OsRefuse, ContractViolation> {
    OsRefuse::v1(OsCapabilityId::OsDecisionCompute, reason_code, reason)
}

fn extract_ocr_text_from_normalized_json(json_text: &str) -> Option<String> {
    let value: Value = serde_json::from_str(json_text).ok()?;

    if let Some(text) = value.get("ocr_text").and_then(Value::as_str) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(text) = value.get("text_output").and_then(Value::as_str) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(text) = value.get("text").and_then(Value::as_str) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(text) = value.get("short_analysis").and_then(Value::as_str) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Some(lines) = value.get("lines").and_then(Value::as_array) {
        let joined = lines
            .iter()
            .filter_map(Value::as_str)
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if !joined.is_empty() {
            return Some(joined);
        }
    }
    None
}

fn vision_ocr_payload(bundle: &VisionForwardBundle) -> Value {
    json!({
        "source_engine": "PH1.VISION",
        "source_ref": {
            "source_id": bundle.source_ref.source_id.as_str(),
            "source_kind": visual_source_kind_label(bundle.source_ref.source_kind),
        },
        "visible_content_only": bundle.visible_content_only,
        "evidence_items": bundle.evidence_items.iter().map(|item| {
            let bbox = item.bbox.map(|b| {
                json!({
                    "x": b.x,
                    "y": b.y,
                    "w": b.w,
                    "h": b.h
                })
            });
            json!({
                "text": item.text,
                "bbox": bbox
            })
        }).collect::<Vec<_>>()
    })
}

fn doc_ocr_payload(bundle: &DocForwardBundle) -> Value {
    json!({
        "source_engine": "PH1.DOC",
        "source_ref": {
            "source_id": bundle.source_ref.source_id.as_str(),
            "source_kind": document_source_kind_label(bundle.source_ref.source_kind),
        },
        "citation_validation_status": match bundle.citation_map.validation_status {
            DocValidationStatus::Ok => "OK",
            DocValidationStatus::Fail => "FAIL",
        },
        "evidence_backed_only": bundle.extract.evidence_backed_only && bundle.citation_map.evidence_backed_only,
        "evidence_items": bundle.extract.evidence_items.iter().map(|item| {
            json!({
                "evidence_id": item.evidence_id.as_str(),
                "segment_id": item.segment_id.as_str(),
                "page_index": item.page_index,
                "text": item.text
            })
        }).collect::<Vec<_>>()
    })
}

fn visual_source_kind_label(kind: VisualSourceKind) -> &'static str {
    match kind {
        VisualSourceKind::Image => "IMAGE",
        VisualSourceKind::Screenshot => "SCREENSHOT",
        VisualSourceKind::Diagram => "DIAGRAM",
    }
}

fn document_source_kind_label(kind: DocumentSourceKind) -> &'static str {
    match kind {
        DocumentSourceKind::Pdf => "PDF",
        DocumentSourceKind::Word => "WORD",
        DocumentSourceKind::Html => "HTML",
        DocumentSourceKind::Scan => "SCAN",
    }
}

fn stable_schema_hash(bytes: &[u8]) -> SchemaHash {
    SchemaHash(stable_nonzero_hash_u64(bytes))
}

fn stable_nonzero_hash_u64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn is_provider_token(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | ':' | '/'))
}

fn voice_identity_runtime_context(
    voice_context: OsVoiceTurnContext,
    tenant_id: Option<String>,
) -> VoiceIdentityRuntimeContext {
    let platform = match voice_context.platform {
        OsVoicePlatform::Ios => VoiceIdentityPlatform::Ios,
        OsVoicePlatform::Android => VoiceIdentityPlatform::Android,
        OsVoicePlatform::Desktop => VoiceIdentityPlatform::Desktop,
    };
    let channel = match voice_context.trigger {
        OsVoiceTrigger::WakeWord => VoiceIdentityChannel::WakeWord,
        OsVoiceTrigger::Explicit => VoiceIdentityChannel::Explicit,
    };
    VoiceIdentityRuntimeContext::for_tenant(tenant_id, platform, channel)
}

fn voice_identity_prompt_scope_key(
    tenant_id: Option<&str>,
    actor_user_id: &UserId,
    device_id: Option<&selene_kernel_contracts::ph1j::DeviceId>,
    voice_context: OsVoiceTurnContext,
) -> String {
    let tenant_component = tenant_id.unwrap_or("none");
    let device_component = device_id.map(|d| d.as_str()).unwrap_or("none");
    let branch_component = format!(
        "{}:{}",
        voice_context.platform.as_str(),
        voice_context.trigger.as_str()
    );
    format!(
        "vidscope:v1:t{:016x}:u{:016x}:d{:016x}:b{:016x}",
        stable_scope_hash_u64(tenant_component),
        stable_scope_hash_u64(actor_user_id.as_str()),
        stable_scope_hash_u64(device_component),
        stable_scope_hash_u64(&branch_component),
    )
}

fn stable_scope_hash_u64(value: &str) -> u64 {
    const FNV64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV64_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV64_OFFSET_BASIS;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV64_PRIME);
    }
    hash
}

fn expected_always_on_sequence(
    path: OsTopLevelTurnPath,
    voice_context: Option<OsVoiceTurnContext>,
) -> &'static [&'static str] {
    match path {
        OsTopLevelTurnPath::Voice => {
            let context = voice_context.expect("voice context must be present when path is VOICE");
            if context.trigger.wake_stage_required() {
                &[
                    "PH1.K",
                    "PH1.W",
                    "PH1.VOICE.ID",
                    "PH1.C",
                    "PH1.SRL",
                    "PH1.NLP",
                    "PH1.CONTEXT",
                    "PH1.POLICY",
                    "PH1.X",
                ]
            } else {
                &[
                    "PH1.K",
                    "PH1.VOICE.ID",
                    "PH1.C",
                    "PH1.SRL",
                    "PH1.NLP",
                    "PH1.CONTEXT",
                    "PH1.POLICY",
                    "PH1.X",
                ]
            }
        }
        OsTopLevelTurnPath::Text => &["PH1.NLP", "PH1.CONTEXT", "PH1.POLICY", "PH1.X"],
    }
}

fn turn_optional_sequence() -> &'static [&'static str] {
    &[
        "PH1.ENDPOINT",
        "PH1.LANG",
        "PH1.PRON",
        "PH1.DOC",
        "PH1.SUMMARY",
        "PH1.VISION",
        "PH1.PRUNE",
        "PH1.DIAG",
        "PH1.SEARCH",
        "PH1.COST",
        "PH1.PREFETCH",
        "PH1.EXPLAIN",
        "PH1.LISTEN",
        "PH1.EMO.GUIDE",
        "PH1.EMO.CORE",
        "PH1.PERSONA",
        "PH1.FEEDBACK",
        "PH1.LEARN",
        "PH1.PAE",
        "PH1.CACHE",
        "PH1.KNOW",
        "PH1.MULTI",
        "PH1.KG",
        "PH1.BCAST",
        "PH1.DELIVERY",
    ]
}

fn runtime_forbidden_engine_ids() -> &'static [&'static str] {
    &["PH1.PATTERN", "PH1.RLL", "PH1.GOV", "PH1.EXPORT", "PH1.KMS"]
}

fn first_runtime_forbidden_engine_id(engine_ids: &[String]) -> Option<&str> {
    engine_ids
        .iter()
        .map(String::as_str)
        .find(|engine_id| runtime_forbidden_engine_ids().contains(engine_id))
}

fn optional_engine_allowed_by_policy(engine_id: &str, input: &OsTurnInput) -> bool {
    match engine_id {
        "PH1.PRUNE" => input.clarify_required,
        "PH1.DIAG" => {
            input.clarify_required
                || input.confirm_required
                || input.simulation_requested
                || input.tool_requested
        }
        _ => true,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsOptionalEngineTier {
    Strict,
    Balanced,
    Rich,
}

impl OsOptionalEngineTier {
    pub fn as_str(self) -> &'static str {
        match self {
            OsOptionalEngineTier::Strict => "STRICT",
            OsOptionalEngineTier::Balanced => "BALANCED",
            OsOptionalEngineTier::Rich => "RICH",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsOptionalUtilityAction {
    Keep,
    Degrade,
    DisableCandidate,
}

impl OsOptionalUtilityAction {
    pub fn as_str(self) -> &'static str {
        match self {
            OsOptionalUtilityAction::Keep => "KEEP",
            OsOptionalUtilityAction::Degrade => "DEGRADE",
            OsOptionalUtilityAction::DisableCandidate => "DISABLE_CANDIDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OsOptionalUtilityThresholds {
    pub decision_delta_min_bps: u16,
    pub queue_learn_conversion_min_bps: u16,
    pub no_value_max_bps: u16,
    pub latency_p95_max_ms: u32,
    pub latency_p99_max_ms: u32,
    pub sustained_fail_streak_days: u16,
}

impl OsOptionalUtilityThresholds {
    pub fn mvp_v1() -> Self {
        Self {
            decision_delta_min_bps: 800,
            queue_learn_conversion_min_bps: 2000,
            no_value_max_bps: 6000,
            latency_p95_max_ms: 20,
            latency_p99_max_ms: 40,
            sustained_fail_streak_days: 7,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsOptionalEngineUtilityReview {
    pub engine_id: String,
    pub tier: OsOptionalEngineTier,
    pub outcome_count: u32,
    pub decision_delta_rate_bps: u16,
    pub queue_learn_conversion_rate_bps: u16,
    pub no_value_rate_bps: u16,
    pub latency_p95_ms: u32,
    pub latency_p99_ms: u32,
    pub gate_u4_pass: bool,
    pub gate_u5_triggered: bool,
    pub action: OsOptionalUtilityAction,
}

pub fn review_optional_engine_utility(
    engine_id: &str,
    outcome_entries: &[OsOutcomeUtilizationEntry],
    fail_streak_days: u16,
    thresholds: OsOptionalUtilityThresholds,
) -> Result<OsOptionalEngineUtilityReview, ContractViolation> {
    if !is_engine_id_token(engine_id) {
        return Err(ContractViolation::InvalidValue {
            field: "os_optional_engine_utility.engine_id",
            reason: "engine id must be ASCII [A-Z0-9._] and <= 64 chars",
        });
    }
    if !turn_optional_sequence().contains(&engine_id) {
        return Err(ContractViolation::InvalidValue {
            field: "os_optional_engine_utility.engine_id",
            reason: "must be a TURN_OPTIONAL engine id",
        });
    }

    let mut outcome_count = 0u32;
    let mut decision_delta_count = 0u32;
    let mut queue_learn_total = 0u32;
    let mut queue_learn_delta = 0u32;
    let mut drop_count = 0u32;
    let mut latencies = Vec::new();

    for entry in outcome_entries
        .iter()
        .filter(|entry| entry.engine_id == engine_id)
    {
        outcome_count = outcome_count.saturating_add(1);
        if entry.decision_delta {
            decision_delta_count = decision_delta_count.saturating_add(1);
        }
        if entry.action_class == selene_kernel_contracts::ph1os::OsOutcomeActionClass::QueueLearn {
            queue_learn_total = queue_learn_total.saturating_add(1);
            if entry.decision_delta {
                queue_learn_delta = queue_learn_delta.saturating_add(1);
            }
        }
        if entry.action_class == selene_kernel_contracts::ph1os::OsOutcomeActionClass::Drop {
            drop_count = drop_count.saturating_add(1);
        }
        latencies.push(entry.latency_cost_ms);
    }

    if outcome_count == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "os_optional_engine_utility.outcome_entries",
            reason: "must include at least one outcome entry for the selected engine",
        });
    }

    latencies.sort_unstable();

    let decision_delta_rate_bps = ratio_bps(decision_delta_count, outcome_count);
    let queue_learn_conversion_rate_bps = ratio_bps(queue_learn_delta, queue_learn_total);
    let no_value_rate_bps = ratio_bps(drop_count, outcome_count);
    let latency_p95_ms = nearest_rank_percentile_ms(&latencies, 95);
    let latency_p99_ms = nearest_rank_percentile_ms(&latencies, 99);

    let gate_u4_pass = (decision_delta_rate_bps >= thresholds.decision_delta_min_bps
        || queue_learn_conversion_rate_bps >= thresholds.queue_learn_conversion_min_bps)
        && no_value_rate_bps <= thresholds.no_value_max_bps
        && latency_p95_ms <= thresholds.latency_p95_max_ms
        && latency_p99_ms <= thresholds.latency_p99_max_ms;
    let gate_u5_triggered =
        !gate_u4_pass && fail_streak_days >= thresholds.sustained_fail_streak_days;

    let action = if gate_u4_pass {
        OsOptionalUtilityAction::Keep
    } else if gate_u5_triggered {
        OsOptionalUtilityAction::DisableCandidate
    } else {
        OsOptionalUtilityAction::Degrade
    };

    Ok(OsOptionalEngineUtilityReview {
        engine_id: engine_id.to_string(),
        tier: optional_engine_tier(engine_id),
        outcome_count,
        decision_delta_rate_bps,
        queue_learn_conversion_rate_bps,
        no_value_rate_bps,
        latency_p95_ms,
        latency_p99_ms,
        gate_u4_pass,
        gate_u5_triggered,
        action,
    })
}

fn optional_engine_tier(engine_id: &str) -> OsOptionalEngineTier {
    match engine_id {
        "PH1.ENDPOINT" | "PH1.LANG" | "PH1.PRON" | "PH1.PRUNE" | "PH1.DIAG" | "PH1.SEARCH"
        | "PH1.PREFETCH" => OsOptionalEngineTier::Strict,
        "PH1.DOC" | "PH1.SUMMARY" | "PH1.VISION" | "PH1.COST" | "PH1.EXPLAIN" | "PH1.LISTEN"
        | "PH1.EMO.GUIDE" | "PH1.PERSONA" | "PH1.FEEDBACK" | "PH1.CACHE" | "PH1.KNOW" => {
            OsOptionalEngineTier::Balanced
        }
        _ => OsOptionalEngineTier::Rich,
    }
}

fn ratio_bps(numerator: u32, denominator: u32) -> u16 {
    if denominator == 0 {
        return 0;
    }
    let scaled = (numerator as u64)
        .saturating_mul(10_000)
        .saturating_div(denominator as u64);
    scaled.min(10_000) as u16
}

fn nearest_rank_percentile_ms(sorted_values: &[u32], percentile: u8) -> u32 {
    if sorted_values.is_empty() {
        return 0;
    }
    let clamped = percentile.clamp(1, 100) as u128;
    let n = sorted_values.len() as u128;
    let rank = (clamped.saturating_mul(n).saturating_add(99)).saturating_div(100);
    let index = (rank.saturating_sub(1) as usize).min(sorted_values.len() - 1);
    sorted_values[index]
}

fn matches_engine_order(actual: &[String], expected: &[&str]) -> bool {
    actual.len() == expected.len()
        && actual
            .iter()
            .zip(expected.iter())
            .all(|(left, right)| left == right)
}

fn ensure_engine_id_list_valid(
    field: &'static str,
    list: &[String],
) -> Result<(), ContractViolation> {
    let mut uniq = BTreeSet::new();
    for engine_id in list {
        if !is_engine_id_token(engine_id) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "engine id must be ASCII [A-Z0-9._] and <= 64 chars",
            });
        }
        if !uniq.insert(engine_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "duplicate engine id in list",
            });
        }
    }
    Ok(())
}

fn is_engine_id_token(value: &str) -> bool {
    if value.is_empty() || value.len() > 64 {
        return false;
    }
    value
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'.' || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1c_superiority::{
        SuperiorityEvalPack, SuperiorityLane, SuperiorityMetricRow, SuperioritySliceKey,
    };
    use crate::ph1doc::DocForwardBundle;
    use crate::ph1feedback::{
        build_gold_case_capture_from_ph1c_response, FeedbackTurnInput, FeedbackWiringOutcome,
        GoldCaseCaptureContext, Ph1FeedbackEngine, Ph1FeedbackWiring, Ph1FeedbackWiringConfig,
    };
    use crate::ph1learn::{
        route_feedback_into_learn_wiring, FeedbackLearnRouteConfig, LearnWiringOutcome,
        Ph1LearnEngine, Ph1LearnWiring, Ph1LearnWiringConfig,
    };
    use crate::ph1pae::{PaeWiringOutcome, Ph1PaeEngine, Ph1PaeWiring, Ph1PaeWiringConfig};
    use crate::ph1vision::VisionForwardBundle;
    use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
    use selene_engines::ph1d::{Ph1dProviderAdapter, Ph1dProviderAdapterError};
    use selene_engines::ph1feedback::{Ph1FeedbackConfig, Ph1FeedbackRuntime};
    use selene_engines::ph1learn::{Ph1LearnConfig, Ph1LearnRuntime};
    use selene_engines::ph1pae::{Ph1PaeConfig, Ph1PaeRuntime};
    use selene_kernel_contracts::ph1_voice_id::{
        DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId,
    };
    use selene_kernel_contracts::ph1c::{
        ConfidenceBucket, LanguageTag, Ph1cAuditMeta, Ph1cResponse, QualityBucket, RetryAdvice,
        RouteClassUsed, RoutingModeUsed, SelectedSlot, SessionStateRef,
        TranscriptOk as NlpTranscriptOk, TranscriptReject,
    };
    use selene_kernel_contracts::ph1context::{
        ContextBundleBuildOk, ContextBundleItem, ContextBundleTrimOk, ContextSourceKind,
        ContextValidationStatus, Ph1ContextRequest, Ph1ContextResponse,
    };
    use selene_kernel_contracts::ph1d::{
        Ph1dProviderCallRequest, Ph1dProviderCallResponse, Ph1dProviderInputPayloadKind,
        Ph1dProviderStatus, Ph1dProviderTask, Ph1dProviderValidationStatus,
    };
    use selene_kernel_contracts::ph1doc::{
        DocCitationMapBuildOk, DocEvidenceExtractOk, DocEvidenceId, DocEvidenceItem,
        DocValidationStatus, DocumentSegmentId, DocumentSourceId, DocumentSourceKind,
        DocumentSourceRef,
    };
    use selene_kernel_contracts::ph1feedback::{
        FeedbackConfidenceBucket, FeedbackEventCollectOk, FeedbackEventRecord, FeedbackEventType,
        FeedbackGoldProvenanceMethod, FeedbackGoldStatus, FeedbackMetrics, FeedbackSignalCandidate,
        FeedbackSignalEmitOk, FeedbackSignalTarget, FeedbackToolStatus, FeedbackValidationStatus,
        Ph1FeedbackRequest, Ph1FeedbackResponse,
    };
    use selene_kernel_contracts::ph1j::{AuditEngine, DeviceId, PayloadKey};
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, DeviceHealth, DeviceRoute, DeviceState, FrameDurationMs, PreRollBufferId,
        PreRollBufferRef, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1learn::{
        LearnArtifactCandidate, LearnArtifactPackageBuildOk, LearnArtifactTarget, LearnScope,
        LearnSignal, LearnSignalAggregateOk, LearnSignalType, LearnTargetEngine,
        LearnValidationStatus, Ph1LearnRequest, Ph1LearnResponse,
    };
    use selene_kernel_contracts::ph1n::{Chat, Ph1nRequest, Ph1nResponse};
    use selene_kernel_contracts::ph1os::{OsOutcomeActionClass, OsOutcomeUtilizationEntry};
    use selene_kernel_contracts::ph1pae::{
        PaeAdaptationHint, PaeAdaptationHintEmitOk, PaeMode, PaePolicyCandidate,
        PaePolicyScoreBuildOk, PaeProviderSlot, PaeRouteDomain, PaeScoreEntry, PaeSignalSource,
        PaeSignalVector, PaeTargetEngine, PaeValidationStatus, Ph1PaeRequest, Ph1PaeResponse,
    };
    use selene_kernel_contracts::ph1vision::{
        VisionEvidenceItem, VisualSourceId, VisualSourceKind, VisualSourceRef,
    };
    use selene_kernel_contracts::ReasonCodeId;
    use selene_kernel_contracts::{MonotonicTimeNs, SessionState};
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore};
    use std::cell::RefCell;
    use std::rc::Rc;

    #[derive(Debug, Clone)]
    struct MockOsEngine {
        policy_response: Ph1OsResponse,
        decision_response: Ph1OsResponse,
    }

    impl Ph1OsEngine for MockOsEngine {
        fn run(&self, req: &Ph1OsRequest) -> Ph1OsResponse {
            match req {
                Ph1OsRequest::OsPolicyEvaluate(_) => self.policy_response.clone(),
                Ph1OsRequest::OsDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct InspectingMockOsEngine {
        policy_response: Ph1OsResponse,
        decision_response: Ph1OsResponse,
        seen_policy_requests: Rc<RefCell<Vec<OsPolicyEvaluateRequest>>>,
    }

    impl InspectingMockOsEngine {
        fn new(policy_response: Ph1OsResponse, decision_response: Ph1OsResponse) -> Self {
            Self {
                policy_response,
                decision_response,
                seen_policy_requests: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl Ph1OsEngine for InspectingMockOsEngine {
        fn run(&self, req: &Ph1OsRequest) -> Ph1OsResponse {
            match req {
                Ph1OsRequest::OsPolicyEvaluate(r) => {
                    self.seen_policy_requests.borrow_mut().push(r.clone());
                    self.policy_response.clone()
                }
                Ph1OsRequest::OsDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    #[derive(Debug, Clone)]
    struct FeedbackRuntimeEngineAdapter {
        runtime: Ph1FeedbackRuntime,
    }

    impl FeedbackRuntimeEngineAdapter {
        fn mvp_v1() -> Self {
            Self {
                runtime: Ph1FeedbackRuntime::new(Ph1FeedbackConfig::mvp_v1()),
            }
        }
    }

    impl Ph1FeedbackEngine for FeedbackRuntimeEngineAdapter {
        fn run(&self, req: &Ph1FeedbackRequest) -> Ph1FeedbackResponse {
            self.runtime.run(req)
        }
    }

    #[derive(Debug, Clone)]
    struct LearnRuntimeEngineAdapter {
        runtime: Ph1LearnRuntime,
    }

    impl LearnRuntimeEngineAdapter {
        fn mvp_v1() -> Self {
            Self {
                runtime: Ph1LearnRuntime::new(Ph1LearnConfig::mvp_v1()),
            }
        }
    }

    impl Ph1LearnEngine for LearnRuntimeEngineAdapter {
        fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
            self.runtime.run(req)
        }
    }

    #[derive(Debug, Clone)]
    struct PaeRuntimeEngineAdapter {
        runtime: Ph1PaeRuntime,
    }

    impl PaeRuntimeEngineAdapter {
        fn mvp_v1() -> Self {
            Self {
                runtime: Ph1PaeRuntime::new(Ph1PaeConfig::mvp_v1()),
            }
        }
    }

    impl Ph1PaeEngine for PaeRuntimeEngineAdapter {
        fn run(&self, req: &Ph1PaeRequest) -> Ph1PaeResponse {
            self.runtime.run(req)
        }
    }

    fn gold_loop_context() -> GoldCaseCaptureContext {
        GoldCaseCaptureContext::v1(
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(49_101),
            TurnId(6_101),
            "idem_gold_loop_1".to_string(),
        )
        .unwrap()
    }

    fn build_ph1c_gold_loop_feedback_turn_input(
    ) -> (FeedbackEventRecord, FeedbackEventRecord, FeedbackTurnInput) {
        let context = gold_loop_context();
        let miss_capture = build_gold_case_capture_from_ph1c_response(
            &context,
            &Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                ReasonCodeId(0x4300_0002),
                RetryAdvice::SpeakSlower,
            )),
            Some("evidence:ph1c:miss_case"),
            None,
        )
        .unwrap()
        .expect("TranscriptReject must emit gold-case miss capture");
        let correction_capture = build_gold_case_capture_from_ph1c_response(
            &context,
            &Ph1cResponse::TranscriptOk(
                NlpTranscriptOk::v1(
                    "invoice total one twenty three".to_string(),
                    LanguageTag::new("en-US").unwrap(),
                    ConfidenceBucket::Low,
                )
                .unwrap(),
            ),
            Some("evidence:ph1c:correction_case"),
            Some("invoice total 123.45".to_string()),
        )
        .unwrap()
        .expect("low-confidence TranscriptOk with correction must emit gold-case correction");
        let miss_event = miss_capture.feedback_event.clone();
        let correction_event = correction_capture.feedback_event.clone();
        let feedback_input = FeedbackTurnInput::v1(
            context.correlation_id,
            context.turn_id,
            vec![miss_event.clone(), correction_event.clone()],
        )
        .unwrap();
        (miss_event, correction_event, feedback_input)
    }

    fn build_pae_turn_input_from_learn(
        learn_turn_input: &LearnTurnInput,
        learn_bundle: &LearnForwardBundle,
    ) -> PaeTurnInput {
        let selected_artifact = learn_bundle
            .signal_aggregate
            .ordered_artifacts
            .iter()
            .find(|artifact| artifact.target == LearnArtifactTarget::PaeRoutingWeights)
            .unwrap_or_else(|| &learn_bundle.signal_aggregate.ordered_artifacts[0]);
        let sample_size = learn_turn_input
            .signals
            .iter()
            .map(|signal| u32::from(signal.occurrence_count))
            .sum::<u32>()
            .clamp(10, u32::from(u16::MAX)) as u16;
        let first_signal = &learn_turn_input.signals[0];
        let expected_quality_bp = selected_artifact.expected_effect_bp.clamp(-9_500, 9_500);
        let regression_risk_bp = expected_quality_bp.unsigned_abs().min(9_500);
        let candidate_id = format!("pae_candidate_{}", selected_artifact.artifact_id);
        let signal = PaeSignalVector::v1(
            format!("pae_signal_{}", first_signal.signal_id),
            PaeSignalSource::Feedback,
            PaeRouteDomain::Stt,
            first_signal.metric_key.clone(),
            first_signal.metric_value_bp,
            sample_size,
            true,
            first_signal.evidence_ref.clone(),
        )
        .unwrap();
        let candidate = PaePolicyCandidate::v1(
            candidate_id,
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            expected_quality_bp,
            240,
            120,
            regression_risk_bp,
            sample_size,
            Some(selected_artifact.artifact_id.clone()),
            selected_artifact.rollback_to.clone(),
        )
        .unwrap();
        PaeTurnInput::v1(
            learn_turn_input.correlation_id,
            learn_turn_input.turn_id,
            learn_turn_input.tenant_id.clone(),
            "desktop_profile_round2".to_string(),
            PaeMode::Assist,
            vec![signal],
            vec![candidate],
            vec![PaeTargetEngine::Ph1C],
            true,
            10,
            800,
            3,
            0,
            true,
        )
        .unwrap()
    }

    fn build_gold_loop_chain_input(
        feedback_event: FeedbackEventRecord,
        feedback_bundle: FeedbackForwardBundle,
        learn_turn_input: LearnTurnInput,
        learn_bundle: LearnForwardBundle,
        pae_turn_input: PaeTurnInput,
        pae_bundle: PaeForwardBundle,
    ) -> OsSelfHealChainInput {
        OsSelfHealChainInput {
            feedback_event,
            feedback_bundle,
            learn_turn_input,
            learn_bundle,
            pae_turn_input,
            pae_bundle,
            owner_engine: "PH1.C".to_string(),
            first_seen_at: MonotonicTimeNs(3_000),
            last_seen_at: MonotonicTimeNs(3_500),
            containment_action: FailureContainmentAction::FailClosedRefuse,
            escalation_required: false,
            unresolved_reason: None,
            bcast_id: None,
            provider_context: Some(
                FailureProviderContext::v1(
                    PaeRouteDomain::Stt,
                    PaeProviderSlot::Primary,
                    Ph1dProviderTask::SttTranscribe,
                    4_200,
                    88,
                    false,
                )
                .unwrap(),
            ),
            governance_required: false,
            governance_ticket_ref: None,
            approved_by: None,
            evaluated_at: MonotonicTimeNs(3_700),
            ph1c_superiority_pack: Some(sample_ph1c_superiority_pack()),
        }
    }

    fn sample_ph1c_superiority_pack() -> SuperiorityEvalPack {
        let slice = SuperioritySliceKey::v1(
            "en-US".to_string(),
            "desktop_mic".to_string(),
            "tenant_1".to_string(),
        )
        .unwrap();
        let baseline = SuperiorityMetricRow::v1(
            "2026-02-26T12:00:00Z".to_string(),
            "abc1234".to_string(),
            slice.clone(),
            SuperiorityLane::SeleneBaseline,
            320,
            9_620,
            9_540,
            9_500,
            9_520,
            240,
            295,
            9_000,
            10_000,
            10_000,
            9_550,
            9_480,
            2_600,
            5_600,
            9_850,
            900,
            9_100,
            8_000,
            500,
            9_000,
            9_000,
            9_200,
            9_000,
            9_850,
            10_000,
            true,
            false,
            PaeMode::Shadow,
        )
        .unwrap();
        let chatgpt = SuperiorityMetricRow::v1(
            "2026-02-26T12:00:00Z".to_string(),
            "abc1234".to_string(),
            slice.clone(),
            SuperiorityLane::ChatgptAb,
            320,
            9_700,
            9_650,
            9_680,
            9_710,
            225,
            280,
            9_150,
            10_000,
            10_000,
            9_600,
            9_500,
            2_450,
            0,
            0,
            1_200,
            9_200,
            0,
            0,
            0,
            0,
            0,
            0,
            9_900,
            10_000,
            true,
            false,
            PaeMode::Shadow,
        )
        .unwrap();
        let challenger = SuperiorityMetricRow::v1(
            "2026-02-26T12:00:00Z".to_string(),
            "abc1234".to_string(),
            slice,
            SuperiorityLane::SeleneChallenger,
            320,
            9_780,
            9_710,
            9_735,
            9_760,
            230,
            285,
            9_250,
            10_000,
            10_000,
            9_660,
            9_540,
            2_100,
            6_200,
            9_900,
            1_000,
            9_400,
            9_850,
            150,
            9_300,
            9_400,
            9_300,
            9_200,
            9_900,
            10_000,
            true,
            true,
            PaeMode::Assist,
        )
        .unwrap();
        SuperiorityEvalPack::v1(vec![baseline, chatgpt, challenger]).unwrap()
    }

    fn base_input() -> OsTurnInput {
        OsTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            true,
            true,
            true,
            false,
            false,
            true,
            false,
            false,
            true,
            true,
            true,
            false,
            false,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
        )
        .unwrap()
    }

    fn policy_ok() -> OsPolicyEvaluateOk {
        OsPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            true,
            vec![],
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok(next_move: OsNextMove) -> OsDecisionComputeOk {
        let (dispatch_allowed, execution_allowed) = match next_move {
            OsNextMove::DispatchSimulation => (true, true),
            OsNextMove::DispatchTool => (true, false),
            _ => (false, false),
        };
        OsDecisionComputeOk::v1(
            ReasonCodeId(2),
            next_move,
            false,
            dispatch_allowed,
            execution_allowed,
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn sample_live_voice_id_request(now: MonotonicTimeNs) -> Ph1VoiceIdRequest {
        let stream_id = AudioStreamId(1);
        let processed_stream_ref = AudioStreamRef::v1(
            stream_id,
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
        );
        let vad_events = vec![VadEvent::v1(
            stream_id,
            MonotonicTimeNs(now.0.saturating_sub(1_000_000)),
            now,
            Confidence::new(0.95).unwrap(),
            SpeechLikeness::new(0.95).unwrap(),
        )];
        let session_snapshot = SessionSnapshot {
            schema_version: selene_kernel_contracts::SchemaVersion(1),
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        };
        Ph1VoiceIdRequest::v1(
            now,
            processed_stream_ref,
            vad_events,
            AudioDeviceId::new("os_live_mic_1").unwrap(),
            session_snapshot,
            None,
            false,
            DeviceTrustLevel::Trusted,
            None,
        )
        .unwrap()
    }

    fn sample_ph1k_live_evidence_from_request(
        request: &Ph1VoiceIdRequest,
        health: DeviceHealth,
    ) -> OsPh1kLiveEvidence {
        let start = request
            .vad_events
            .first()
            .map(|event| event.t_start)
            .unwrap_or(request.now);
        let end = request
            .vad_events
            .last()
            .map(|event| event.t_end)
            .unwrap_or(request.now);
        OsPh1kLiveEvidence {
            processed_stream_ref: request.processed_audio_stream_ref,
            pre_roll_buffer_ref: PreRollBufferRef::v1(
                PreRollBufferId(9_001),
                request.processed_audio_stream_ref.stream_id,
                start,
                end,
            ),
            vad_events: request.vad_events.clone(),
            device_state: DeviceState::v1_with_route(
                AudioDeviceId::new("os_live_mic_k").unwrap(),
                AudioDeviceId::new("os_live_spk_k").unwrap(),
                DeviceRoute::BuiltIn,
                health,
                Vec::new(),
            ),
            timing_stats: None,
            tts_playback: None,
            interrupt_candidate: None,
            duplex_frame: None,
        }
    }

    #[test]
    fn at_os_05_wiring_disabled() {
        let wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(false),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        assert_eq!(outcome, OsWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_os_06_wiring_forwarded() {
        let mut input = base_input();
        input.simulation_requested = true;
        input.chat_requested = false;

        let policy = OsPolicyEvaluateOk::v1(
            ReasonCodeId(11),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            vec![],
            true,
            true,
            true,
        )
        .unwrap();
        let decision = decision_ok(OsNextMove::DispatchSimulation);
        let wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy.clone()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision.clone()),
            },
        )
        .unwrap();

        let outcome = wiring.run_turn(&input).unwrap();
        let OsWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(bundle.policy_evaluate, policy);
        assert_eq!(bundle.decision_compute, decision);
    }

    #[test]
    fn at_os_07_invalid_engine_response_fails_closed() {
        let wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        let OsWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refused outcome");
        };
        assert_eq!(refuse.capability_id, OsCapabilityId::OsPolicyEvaluate);
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR
        );
    }

    #[test]
    fn at_os_08_turn_input_rejects_unresolved_outcome_consumed_by() {
        let outcome = OsOutcomeUtilizationEntry::v1(
            "PH1.NLP".to_string(),
            "INTENT_DRAFT".to_string(),
            CorrelationId(7801),
            TurnId(8801),
            OsOutcomeActionClass::ActNow,
            "NONE".to_string(),
            2,
            true,
            ReasonCodeId(9),
        );
        assert!(outcome.is_err());
    }

    fn voice_context_android_wake() -> Option<OsVoiceTurnContext> {
        Some(OsVoiceTurnContext::v1(
            OsVoicePlatform::Android,
            OsVoiceTrigger::WakeWord,
        ))
    }

    fn voice_context_ios_explicit() -> Option<OsVoiceTurnContext> {
        Some(OsVoiceTurnContext::v1(
            OsVoicePlatform::Ios,
            OsVoiceTrigger::Explicit,
        ))
    }

    fn always_on_voice_sequence_wake() -> Vec<String> {
        vec![
            "PH1.K".to_string(),
            "PH1.W".to_string(),
            "PH1.VOICE.ID".to_string(),
            "PH1.C".to_string(),
            "PH1.SRL".to_string(),
            "PH1.NLP".to_string(),
            "PH1.CONTEXT".to_string(),
            "PH1.POLICY".to_string(),
            "PH1.X".to_string(),
        ]
    }

    fn always_on_voice_sequence_explicit() -> Vec<String> {
        vec![
            "PH1.K".to_string(),
            "PH1.VOICE.ID".to_string(),
            "PH1.C".to_string(),
            "PH1.SRL".to_string(),
            "PH1.NLP".to_string(),
            "PH1.CONTEXT".to_string(),
            "PH1.POLICY".to_string(),
            "PH1.X".to_string(),
        ]
    }

    fn always_on_text_sequence() -> Vec<String> {
        vec![
            "PH1.NLP".to_string(),
            "PH1.CONTEXT".to_string(),
            "PH1.POLICY".to_string(),
            "PH1.X".to_string(),
        ]
    }

    #[test]
    fn at_os_09_top_level_voice_path_orders_optional_and_forwards() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let mut turn_input = base_input();
        turn_input.nlp_confidence_high = false;
        turn_input.clarify_required = true;
        turn_input.clarify_owner_engine_id = Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string());

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            always_on_voice_sequence_wake(),
            vec![
                "PH1.DIAG".to_string(),
                "PH1.PREFETCH".to_string(),
                "PH1.PRUNE".to_string(),
            ],
            2,
            turn_input,
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded top-level outcome");
        };

        assert_eq!(forwarded.path, OsTopLevelTurnPath::Voice);
        assert_eq!(
            forwarded.always_on_sequence,
            always_on_voice_sequence_wake()
        );
        assert_eq!(
            forwarded.optional_sequence_invoked,
            vec!["PH1.PRUNE".to_string(), "PH1.DIAG".to_string()]
        );
        assert_eq!(
            forwarded.optional_sequence_skipped_budget,
            vec!["PH1.PREFETCH".to_string()]
        );
    }

    #[test]
    fn at_os_09_voice_explicit_trigger_skips_wake_stage() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_ios_explicit(),
            always_on_voice_sequence_explicit(),
            vec![],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded top-level outcome");
        };

        assert_eq!(
            forwarded.always_on_sequence,
            always_on_voice_sequence_explicit()
        );
        assert!(!forwarded.always_on_sequence.contains(&"PH1.W".to_string()));
    }

    #[test]
    fn at_os_09a_top_level_fails_closed_on_clarify_owner_not_ph1_nlp() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let mut turn_input = base_input();
        turn_input.clarify_required = true;
        turn_input.clarify_owner_engine_id = Some("PH1.DIAG".to_string());

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Text,
            None,
            always_on_text_sequence(),
            vec![],
            1,
            turn_input,
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_CLARIFY_OWNER_INVALID
        );
    }

    #[test]
    fn at_os_09a_voice_path_requires_voice_context() {
        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            None,
            always_on_voice_sequence_wake(),
            vec![],
            1,
            base_input(),
        );
        assert!(input.is_err());
    }

    #[test]
    fn at_os_09a_text_path_rejects_voice_context() {
        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Text,
            voice_context_android_wake(),
            always_on_text_sequence(),
            vec![],
            1,
            base_input(),
        );
        assert!(input.is_err());
    }

    #[test]
    fn at_os_09b_top_level_fails_closed_when_optional_assist_not_allowed_by_policy() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Text,
            None,
            always_on_text_sequence(),
            vec!["PH1.PRUNE".to_string()],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_OPTIONAL_POLICY_BLOCK
        );
    }

    #[test]
    fn at_os_10_top_level_text_path_forwards() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Text,
            None,
            always_on_text_sequence(),
            vec![],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded top-level outcome");
        };
        assert_eq!(forwarded.path, OsTopLevelTurnPath::Text);
        assert_eq!(forwarded.always_on_sequence, always_on_text_sequence());
        assert!(forwarded.optional_sequence_invoked.is_empty());
        assert!(forwarded.optional_sequence_skipped_budget.is_empty());
    }

    #[test]
    fn at_os_11_top_level_fails_closed_on_always_on_sequence_mismatch() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let bad_sequence = vec![
            "PH1.K".to_string(),
            "PH1.W".to_string(),
            "PH1.VOICE.ID".to_string(),
            "PH1.C".to_string(),
            "PH1.SRL".to_string(),
            "PH1.NLP".to_string(),
            "PH1.X".to_string(),
            "PH1.CONTEXT".to_string(),
            "PH1.POLICY".to_string(),
        ];
        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            bad_sequence,
            vec![],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_SEQUENCE_INVALID
        );
    }

    #[test]
    fn at_os_12_top_level_fails_closed_on_unknown_optional_engine() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Text,
            None,
            always_on_text_sequence(),
            vec!["PH1.UNKNOWN".to_string()],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_UNKNOWN_OPTIONAL_ENGINE
        );
    }

    #[test]
    fn at_os_13_top_level_propagates_os_gate_refusal() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::Refuse(
                    OsRefuse::v1(
                        OsCapabilityId::OsPolicyEvaluate,
                        ReasonCodeId(0x4F53_8811),
                        "policy gate refused".to_string(),
                    )
                    .unwrap(),
                ),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            always_on_voice_sequence_wake(),
            vec!["PH1.PREFETCH".to_string()],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected propagated refusal");
        };
        assert_eq!(refuse.reason_code, ReasonCodeId(0x4F53_8811));
    }

    #[test]
    fn at_os_14_top_level_fails_closed_on_optional_latency_budget_breach() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring = Ph1OsTopLevelWiring::new(
            Ph1OsTopLevelConfig {
                orchestrator_enabled: true,
                max_optional_invocations: 8,
                max_optional_latency_ms: 20,
            },
            os_wiring,
        )
        .unwrap();

        let mut turn_input = base_input();
        turn_input.clarify_required = true;
        turn_input.clarify_owner_engine_id = Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string());

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            always_on_voice_sequence_wake(),
            vec!["PH1.PRUNE".to_string(), "PH1.DIAG".to_string()],
            2,
            turn_input,
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_OPTIONAL_BUDGET_INVALID
        );
    }

    fn utility_entry(
        engine_id: &str,
        action_class: OsOutcomeActionClass,
        latency_cost_ms: u32,
        decision_delta: bool,
    ) -> OsOutcomeUtilizationEntry {
        OsOutcomeUtilizationEntry::v1(
            engine_id.to_string(),
            "UTILITY_OUTCOME".to_string(),
            CorrelationId(7801),
            TurnId(8801),
            action_class,
            "PH1.OS".to_string(),
            latency_cost_ms,
            decision_delta,
            ReasonCodeId(77),
        )
        .unwrap()
    }

    fn memory_outcome_entry(
        correlation_id: CorrelationId,
        turn_id: TurnId,
    ) -> OsOutcomeUtilizationEntry {
        OsOutcomeUtilizationEntry::v1(
            "PH1.M".to_string(),
            "MEMORY_CANDIDATE".to_string(),
            correlation_id,
            turn_id,
            OsOutcomeActionClass::ActNow,
            "PH1.X".to_string(),
            6,
            true,
            ReasonCodeId(0x4D00_0001),
        )
        .unwrap()
    }

    #[test]
    fn at_os_15_optional_utility_keep_when_u4_passes() {
        let entries = vec![
            utility_entry("PH1.PRUNE", OsOutcomeActionClass::ActNow, 12, true),
            utility_entry("PH1.PRUNE", OsOutcomeActionClass::ActNow, 14, false),
            utility_entry("PH1.PRUNE", OsOutcomeActionClass::QueueLearn, 10, true),
            utility_entry("PH1.PRUNE", OsOutcomeActionClass::AuditOnly, 16, false),
            utility_entry("PH1.PRUNE", OsOutcomeActionClass::Drop, 18, false),
        ];

        let review = review_optional_engine_utility(
            "PH1.PRUNE",
            &entries,
            0,
            OsOptionalUtilityThresholds::mvp_v1(),
        )
        .unwrap();

        assert_eq!(review.tier, OsOptionalEngineTier::Strict);
        assert_eq!(review.outcome_count, 5);
        assert!(review.gate_u4_pass);
        assert!(!review.gate_u5_triggered);
        assert_eq!(review.action, OsOptionalUtilityAction::Keep);
    }

    #[test]
    fn at_os_16_optional_utility_disables_candidate_on_sustained_u4_failure() {
        let entries = vec![
            utility_entry("PH1.DELIVERY", OsOutcomeActionClass::Drop, 50, false),
            utility_entry("PH1.DELIVERY", OsOutcomeActionClass::Drop, 60, false),
            utility_entry("PH1.DELIVERY", OsOutcomeActionClass::Drop, 70, false),
            utility_entry("PH1.DELIVERY", OsOutcomeActionClass::AuditOnly, 55, false),
        ];

        let review = review_optional_engine_utility(
            "PH1.DELIVERY",
            &entries,
            7,
            OsOptionalUtilityThresholds::mvp_v1(),
        )
        .unwrap();

        assert_eq!(review.tier, OsOptionalEngineTier::Rich);
        assert!(!review.gate_u4_pass);
        assert!(review.gate_u5_triggered);
        assert_eq!(review.action, OsOptionalUtilityAction::DisableCandidate);
    }

    #[test]
    fn at_os_17_optional_utility_degrades_before_sustained_fail_streak() {
        let entries = vec![
            utility_entry("PH1.SEARCH", OsOutcomeActionClass::Drop, 30, false),
            utility_entry("PH1.SEARCH", OsOutcomeActionClass::Drop, 28, false),
            utility_entry("PH1.SEARCH", OsOutcomeActionClass::AuditOnly, 25, false),
            utility_entry("PH1.SEARCH", OsOutcomeActionClass::QueueLearn, 26, false),
        ];

        let review = review_optional_engine_utility(
            "PH1.SEARCH",
            &entries,
            3,
            OsOptionalUtilityThresholds::mvp_v1(),
        )
        .unwrap();

        assert_eq!(review.tier, OsOptionalEngineTier::Strict);
        assert!(!review.gate_u4_pass);
        assert!(!review.gate_u5_triggered);
        assert_eq!(review.action, OsOptionalUtilityAction::Degrade);
    }

    #[test]
    fn at_os_18_top_level_fails_closed_on_offline_optional_engine_runtime_leak() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            always_on_voice_sequence_wake(),
            vec!["PH1.PATTERN".to_string()],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_RUNTIME_BOUNDARY_VIOLATION
        );
    }

    #[test]
    fn at_os_19_top_level_fails_closed_on_control_plane_always_on_runtime_leak() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();

        let input = OsTopLevelTurnInput::v1(
            CorrelationId(7801),
            TurnId(8801),
            OsTopLevelTurnPath::Voice,
            voice_context_android_wake(),
            vec![
                "PH1.K".to_string(),
                "PH1.W".to_string(),
                "PH1.VOICE.ID".to_string(),
                "PH1.C".to_string(),
                "PH1.SRL".to_string(),
                "PH1.NLP".to_string(),
                "PH1.CONTEXT".to_string(),
                "PH1.POLICY".to_string(),
                "PH1.GOV".to_string(),
            ],
            vec![],
            1,
            base_input(),
        )
        .unwrap();

        let outcome = top_level_wiring.run_turn(&input).unwrap();
        let OsTopLevelWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected top-level refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_TOPLEVEL_RUNTIME_BOUNDARY_VIOLATION
        );
    }

    #[test]
    fn at_os_20_wiring_forwards_memory_outcome_entries_into_policy_request() {
        let mut input = base_input();
        input.simulation_requested = true;
        input.outcome_utilization_entries =
            vec![memory_outcome_entry(input.correlation_id, input.turn_id)];

        let policy = OsPolicyEvaluateOk::v1(
            ReasonCodeId(22),
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            vec![],
            true,
            true,
            true,
        )
        .unwrap();

        let inspect_engine = InspectingMockOsEngine::new(
            Ph1OsResponse::OsPolicyEvaluateOk(policy),
            Ph1OsResponse::OsDecisionComputeOk(decision_ok(OsNextMove::DispatchSimulation)),
        );
        let wiring =
            Ph1OsWiring::new(Ph1OsWiringConfig::mvp_v1(true), inspect_engine.clone()).unwrap();

        let outcome = wiring.run_turn(&input).unwrap();
        let OsWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded outcome");
        };
        assert_eq!(
            bundle.decision_compute.next_move,
            OsNextMove::DispatchSimulation
        );

        let seen = inspect_engine.seen_policy_requests.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(
            seen[0].outcome_utilization_entries,
            vec![memory_outcome_entry(input.correlation_id, input.turn_id)]
        );
    }

    #[test]
    fn at_os_21_wiring_fails_closed_on_memory_outcome_correlation_mismatch() {
        let mut input = base_input();
        input.outcome_utilization_entries =
            vec![memory_outcome_entry(CorrelationId(999_999), input.turn_id)];

        let wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();

        let err = wiring
            .run_turn(&input)
            .expect_err("memory outcome correlation mismatch must fail closed");
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "os_turn_input.outcome_utilization_entries.correlation_id"
                );
                assert_eq!(reason, "must match os_turn_input.correlation_id");
            }
            _ => panic!("expected ContractViolation::InvalidValue"),
        }
    }

    #[test]
    fn at_os_22_voice_live_entrypoint_forwards_and_emits_feedback_learn() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();
        let runtime =
            Ph1OsVoiceLiveRuntime::new(top_level_wiring, Ph1VoiceIdLiveRuntime::default());

        let actor_user_id = UserId::new("tenant_1:os_live_voice_user").unwrap();
        let device_id = DeviceId::new("os_live_voice_device_1").unwrap();
        let expected_scope_key = voice_identity_prompt_scope_key(
            Some("tenant_1"),
            &actor_user_id,
            Some(&device_id),
            voice_context_ios_explicit().expect("voice context must exist for voice path"),
        );
        let mut store = Ph1fStore::new_in_memory();
        store
            .insert_identity(IdentityRecord::v1(
                actor_user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor_user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let input = OsVoiceLiveTurnInput::v1(
            OsTopLevelTurnInput::v1(
                CorrelationId(7801),
                TurnId(8801),
                OsTopLevelTurnPath::Voice,
                voice_context_ios_explicit(),
                always_on_voice_sequence_explicit(),
                vec![],
                1,
                base_input(),
            )
            .unwrap(),
            sample_live_voice_id_request(MonotonicTimeNs(3)),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            EngineVoiceIdObservation {
                primary_fingerprint: None,
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        )
        .unwrap();
        let ph1k_evidence =
            sample_ph1k_live_evidence_from_request(&input.voice_id_request, DeviceHealth::Healthy);
        let input = input.with_ph1k_live_evidence(ph1k_evidence).unwrap();

        let outcome = runtime.run_turn(&mut store, input).unwrap();
        let OsVoiceLiveTurnOutcome::Forwarded(forwarded) = outcome else {
            panic!("expected forwarded live voice outcome");
        };
        assert_eq!(forwarded.top_level_bundle.path, OsTopLevelTurnPath::Voice);
        assert!(matches!(
            forwarded.voice_identity_assertion,
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)
        ));
        assert_eq!(
            forwarded.identity_prompt_scope_key,
            Some(expected_scope_key)
        );

        let feedback_rows = store.ph1feedback_audit_rows(CorrelationId(7801));
        assert_eq!(feedback_rows.len(), 1);
        assert!(feedback_rows[0]
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("feedback_event_type").unwrap()));

        let learn_rows = store
            .audit_events_by_correlation(CorrelationId(7801))
            .into_iter()
            .filter(|row| {
                matches!(&row.engine, AuditEngine::Other(engine_id) if engine_id == "PH1.LEARN")
            })
            .collect::<Vec<_>>();
        assert_eq!(learn_rows.len(), 1);
        assert!(learn_rows[0]
            .payload_min
            .entries
            .contains_key(&PayloadKey::new("learn_signal_type").unwrap()));
    }

    #[test]
    fn at_os_22a_voice_live_entrypoint_rejects_text_path_input() {
        let input = OsVoiceLiveTurnInput::v1(
            OsTopLevelTurnInput::v1(
                CorrelationId(7801),
                TurnId(8801),
                OsTopLevelTurnPath::Text,
                None,
                always_on_text_sequence(),
                vec![],
                1,
                base_input(),
            )
            .unwrap(),
            sample_live_voice_id_request(MonotonicTimeNs(3)),
            UserId::new("tenant_1:os_live_voice_user").unwrap(),
            Some("tenant_1".to_string()),
            Some(DeviceId::new("os_live_voice_device_2").unwrap()),
            Vec::new(),
            EngineVoiceIdObservation {
                primary_fingerprint: None,
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        )
        .expect_err("text path must be rejected for live voice runtime input");
        match input {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "os_voice_live_turn_input.top_level_turn_input.path");
                assert_eq!(reason, "must be VOICE");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_os_22c_voice_live_entrypoint_refuses_when_ph1k_device_health_failed() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();
        let runtime =
            Ph1OsVoiceLiveRuntime::new(top_level_wiring, Ph1VoiceIdLiveRuntime::default());

        let actor_user_id = UserId::new("tenant_1:os_live_voice_user").unwrap();
        let device_id = DeviceId::new("os_live_voice_device_health_failed").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        store
            .insert_identity(IdentityRecord::v1(
                actor_user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor_user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let voice_id_request = sample_live_voice_id_request(MonotonicTimeNs(5));
        let ph1k_evidence =
            sample_ph1k_live_evidence_from_request(&voice_id_request, DeviceHealth::Failed);
        let mut os_turn_input = base_input();
        os_turn_input.correlation_id = CorrelationId(7802);
        os_turn_input.turn_id = TurnId(8802);
        let input = OsVoiceLiveTurnInput::v1(
            OsTopLevelTurnInput::v1(
                CorrelationId(7802),
                TurnId(8802),
                OsTopLevelTurnPath::Voice,
                voice_context_ios_explicit(),
                always_on_voice_sequence_explicit(),
                vec![],
                1,
                os_turn_input,
            )
            .unwrap(),
            voice_id_request,
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            EngineVoiceIdObservation {
                primary_fingerprint: None,
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        )
        .unwrap()
        .with_ph1k_live_evidence(ph1k_evidence)
        .unwrap();

        let outcome = runtime.run_turn(&mut store, input).unwrap();
        let OsVoiceLiveTurnOutcome::Refused(refused) = outcome else {
            panic!("expected refusal when PH1.K reports failed device health");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_OS_VOICE_PH1K_DEVICE_FAILED
        );
    }

    #[test]
    fn at_os_22d_voice_live_entrypoint_refuses_when_ph1k_evidence_missing() {
        let os_wiring = Ph1OsWiring::new(
            Ph1OsWiringConfig::mvp_v1(true),
            MockOsEngine {
                policy_response: Ph1OsResponse::OsPolicyEvaluateOk(policy_ok()),
                decision_response: Ph1OsResponse::OsDecisionComputeOk(decision_ok(
                    OsNextMove::Respond,
                )),
            },
        )
        .unwrap();
        let top_level_wiring =
            Ph1OsTopLevelWiring::new(Ph1OsTopLevelConfig::mvp_v1(true), os_wiring).unwrap();
        let runtime =
            Ph1OsVoiceLiveRuntime::new(top_level_wiring, Ph1VoiceIdLiveRuntime::default());

        let actor_user_id = UserId::new("tenant_1:os_live_voice_user").unwrap();
        let device_id = DeviceId::new("os_live_voice_device_evidence_missing").unwrap();
        let mut store = Ph1fStore::new_in_memory();
        store
            .insert_identity(IdentityRecord::v1(
                actor_user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    actor_user_id.clone(),
                    "phone".to_string(),
                    MonotonicTimeNs(2),
                    None,
                )
                .unwrap(),
            )
            .unwrap();

        let mut os_turn_input = base_input();
        os_turn_input.correlation_id = CorrelationId(7803);
        os_turn_input.turn_id = TurnId(8803);
        let input = OsVoiceLiveTurnInput::v1(
            OsTopLevelTurnInput::v1(
                CorrelationId(7803),
                TurnId(8803),
                OsTopLevelTurnPath::Voice,
                voice_context_ios_explicit(),
                always_on_voice_sequence_explicit(),
                vec![],
                1,
                os_turn_input,
            )
            .unwrap(),
            sample_live_voice_id_request(MonotonicTimeNs(7)),
            actor_user_id,
            Some("tenant_1".to_string()),
            Some(device_id),
            Vec::new(),
            EngineVoiceIdObservation {
                primary_fingerprint: None,
                secondary_fingerprint: None,
                primary_embedding: None,
                secondary_embedding: None,
                spoof_risk: false,
            },
        )
        .unwrap();

        let outcome = runtime.run_turn(&mut store, input).unwrap();
        let OsVoiceLiveTurnOutcome::Refused(refused) = outcome else {
            panic!("expected refusal when PH1.K evidence is missing");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_OS_VOICE_PH1K_EVIDENCE_REQUIRED
        );
    }

    #[test]
    fn at_os_22b_voice_identity_prompt_scope_key_is_deterministic_and_branch_scoped() {
        let actor_user_id = UserId::new("tenant_1:os_live_voice_user").unwrap();
        let device_id = DeviceId::new("os_live_voice_device_3").unwrap();
        let key_a = voice_identity_prompt_scope_key(
            Some("tenant_1"),
            &actor_user_id,
            Some(&device_id),
            OsVoiceTurnContext::v1(OsVoicePlatform::Ios, OsVoiceTrigger::Explicit),
        );
        let key_b = voice_identity_prompt_scope_key(
            Some("tenant_1"),
            &actor_user_id,
            Some(&device_id),
            OsVoiceTurnContext::v1(OsVoicePlatform::Ios, OsVoiceTrigger::Explicit),
        );
        let key_c = voice_identity_prompt_scope_key(
            Some("tenant_1"),
            &actor_user_id,
            Some(&device_id),
            OsVoiceTurnContext::v1(OsVoicePlatform::Ios, OsVoiceTrigger::WakeWord),
        );
        let key_d = voice_identity_prompt_scope_key(
            Some("tenant_1"),
            &actor_user_id,
            Some(&device_id),
            OsVoiceTurnContext::v1(OsVoicePlatform::Android, OsVoiceTrigger::Explicit),
        );
        assert_eq!(key_a, key_b);
        assert_ne!(key_a, key_c);
        assert_ne!(key_a, key_d);
        assert!(key_a.len() <= 192);
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum OcrAdapterMode {
        SchemaOk,
        SchemaFail,
        MediumConfidence,
        LowConfidence,
        ContractMismatch,
    }

    #[derive(Debug, Clone)]
    struct RecordingOcrAdapter {
        mode: OcrAdapterMode,
        seen_requests: Rc<RefCell<Vec<Ph1dProviderCallRequest>>>,
    }

    impl RecordingOcrAdapter {
        fn new(mode: OcrAdapterMode) -> Self {
            Self {
                mode,
                seen_requests: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl Ph1dProviderAdapter for RecordingOcrAdapter {
        fn execute(
            &self,
            req: &Ph1dProviderCallRequest,
        ) -> Result<Ph1dProviderCallResponse, Ph1dProviderAdapterError> {
            self.seen_requests.borrow_mut().push(req.clone());

            let (validation_status, normalized_output_json, confidence_bp, reason_code) =
                match self.mode {
                    OcrAdapterMode::SchemaOk => (
                        Ph1dProviderValidationStatus::SchemaOk,
                        Some(r#"{"ocr_text":"invoice total due 123.45"}"#.to_string()),
                        Some(9100),
                        ReasonCodeId(0x4F53_3301),
                    ),
                    OcrAdapterMode::SchemaFail => (
                        Ph1dProviderValidationStatus::SchemaFail,
                        None,
                        Some(9100),
                        ReasonCodeId(0x4F53_3302),
                    ),
                    OcrAdapterMode::MediumConfidence => (
                        Ph1dProviderValidationStatus::SchemaOk,
                        Some(r#"{"ocr_text":"line item subtotal 77"}"#.to_string()),
                        Some(6200),
                        ReasonCodeId(0x4F53_3304),
                    ),
                    OcrAdapterMode::LowConfidence => (
                        Ph1dProviderValidationStatus::SchemaOk,
                        Some(r#"{"ocr_text":"possible subtotal 88"}"#.to_string()),
                        Some(1200),
                        ReasonCodeId(0x4F53_3303),
                    ),
                    OcrAdapterMode::ContractMismatch => (
                        Ph1dProviderValidationStatus::SchemaOk,
                        Some(r#"{"ocr_text":"contract mismatch simulation"}"#.to_string()),
                        Some(9100),
                        ReasonCodeId(0x4F53_3305),
                    ),
                };

            let provider_id = if matches!(self.mode, OcrAdapterMode::ContractMismatch) {
                "provider_unexpected".to_string()
            } else {
                req.provider_id.clone()
            };

            Ph1dProviderCallResponse::v1(
                req.correlation_id,
                req.turn_id,
                req.request_id,
                req.idempotency_key.clone(),
                Some("provider_call_ocr_1".to_string()),
                provider_id,
                req.provider_task,
                req.model_id.clone(),
                Ph1dProviderStatus::Ok,
                88,
                0,
                confidence_bp,
                Some(req.output_schema_hash),
                normalized_output_json,
                validation_status,
                reason_code,
            )
            .map_err(|e| Ph1dProviderAdapterError::terminal(format!("{e:?}")))
        }
    }

    fn sample_vision_forward_bundle() -> VisionForwardBundle {
        let source_ref = VisualSourceRef::v1(
            VisualSourceId::new("vision_src_1").unwrap(),
            VisualSourceKind::Screenshot,
        )
        .unwrap();
        VisionForwardBundle::v1(
            CorrelationId(9901),
            TurnId(1201),
            source_ref,
            vec![
                VisionEvidenceItem::v1("Invoice #42".to_string(), None).unwrap(),
                VisionEvidenceItem::v1("Total Due 123.45".to_string(), None).unwrap(),
            ],
            true,
        )
        .unwrap()
    }

    fn sample_doc_forward_bundle() -> DocForwardBundle {
        let source_ref = DocumentSourceRef::v1(
            DocumentSourceId::new("doc_src_1").unwrap(),
            DocumentSourceKind::Scan,
        )
        .unwrap();
        let evidence_items = vec![
            DocEvidenceItem::v1(
                DocEvidenceId::new("doc_ev_001").unwrap(),
                DocumentSegmentId::new("seg_001").unwrap(),
                Some(1),
                "Invoice Number 42".to_string(),
            )
            .unwrap(),
            DocEvidenceItem::v1(
                DocEvidenceId::new("doc_ev_002").unwrap(),
                DocumentSegmentId::new("seg_002").unwrap(),
                Some(1),
                "Subtotal 88".to_string(),
            )
            .unwrap(),
        ];
        let extract = DocEvidenceExtractOk::v1(
            ReasonCodeId(0x444F_1101),
            source_ref.clone(),
            evidence_items,
            true,
        )
        .unwrap();
        let citation_map = DocCitationMapBuildOk::v1(
            ReasonCodeId(0x444F_1102),
            source_ref.clone(),
            DocValidationStatus::Ok,
            vec![],
            true,
        )
        .unwrap();
        DocForwardBundle::v1(
            CorrelationId(9902),
            TurnId(1202),
            source_ref,
            extract,
            citation_map,
        )
        .unwrap()
    }

    #[test]
    fn at_os_23_ocr_handoff_routes_vision_bundle_through_ph1d_provider() {
        let adapter = RecordingOcrAdapter::new(OcrAdapterMode::SchemaOk);
        let wiring =
            Ph1OsOcrRouteWiring::new(Ph1OsOcrRouteConfig::openai_default(), adapter.clone())
                .unwrap();

        let outcome = wiring
            .run_handoff(&OsOcrAnalyzerForwardBundle::Vision(
                sample_vision_forward_bundle(),
            ))
            .unwrap();
        let OsOcrRouteOutcome::Forwarded(bundle) = outcome else {
            panic!("expected ocr route forwarded outcome");
        };
        assert_eq!(bundle.source_engine, OsOcrSourceEngine::Vision);
        assert_eq!(
            bundle.provider_call.provider_task,
            Ph1dProviderTask::OcrTextExtract
        );
        assert_eq!(bundle.extracted_text, "invoice total due 123.45");

        let seen = adapter.seen_requests.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(seen[0].provider_task, Ph1dProviderTask::OcrTextExtract);
        assert_eq!(
            seen[0].input_payload_kind,
            Ph1dProviderInputPayloadKind::Image
        );
        assert_eq!(seen[0].input_payload_ref, "ocr:vision:vision_src_1");
        assert!(seen[0]
            .input_payload_inline
            .as_deref()
            .expect("inline payload must be present")
            .contains("\"source_engine\":\"PH1.VISION\""));
    }

    #[test]
    fn at_os_24_ocr_handoff_routes_doc_bundle_through_ph1d_provider() {
        let adapter = RecordingOcrAdapter::new(OcrAdapterMode::SchemaOk);
        let wiring =
            Ph1OsOcrRouteWiring::new(Ph1OsOcrRouteConfig::openai_default(), adapter.clone())
                .unwrap();

        let outcome = wiring
            .run_handoff(&OsOcrAnalyzerForwardBundle::Doc(sample_doc_forward_bundle()))
            .unwrap();
        let OsOcrRouteOutcome::Forwarded(bundle) = outcome else {
            panic!("expected ocr route forwarded outcome");
        };
        assert_eq!(bundle.source_engine, OsOcrSourceEngine::Doc);
        assert_eq!(
            bundle.provider_call.provider_task,
            Ph1dProviderTask::OcrTextExtract
        );

        let seen = adapter.seen_requests.borrow();
        assert_eq!(seen.len(), 1);
        assert_eq!(
            seen[0].input_payload_kind,
            Ph1dProviderInputPayloadKind::Document
        );
        assert_eq!(seen[0].input_payload_ref, "ocr:doc:doc_src_1");
        assert!(seen[0]
            .input_payload_inline
            .as_deref()
            .expect("inline payload must be present")
            .contains("\"source_engine\":\"PH1.DOC\""));
    }

    #[test]
    fn at_os_25_ocr_handoff_fails_closed_on_provider_schema_drift() {
        let adapter = RecordingOcrAdapter::new(OcrAdapterMode::SchemaFail);
        let wiring =
            Ph1OsOcrRouteWiring::new(Ph1OsOcrRouteConfig::openai_default(), adapter).unwrap();

        let outcome = wiring
            .run_handoff(&OsOcrAnalyzerForwardBundle::Vision(
                sample_vision_forward_bundle(),
            ))
            .unwrap();
        let OsOcrRouteOutcome::Refused(refuse) = outcome else {
            panic!("expected ocr route refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED
        );
    }

    #[test]
    fn at_os_26_ocr_handoff_fails_closed_on_low_confidence() {
        let adapter = RecordingOcrAdapter::new(OcrAdapterMode::LowConfidence);
        let mut config = Ph1OsOcrRouteConfig::openai_default();
        config.min_provider_confidence_bp = 7000;
        let wiring = Ph1OsOcrRouteWiring::new(config, adapter).unwrap();

        let outcome = wiring
            .run_handoff(&OsOcrAnalyzerForwardBundle::Doc(sample_doc_forward_bundle()))
            .unwrap();
        let OsOcrRouteOutcome::Refused(refuse) = outcome else {
            panic!("expected ocr route refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_LOW_CONFIDENCE
        );
    }

    #[test]
    fn at_os_26b_ocr_handoff_fails_closed_on_provider_contract_mismatch() {
        let adapter = RecordingOcrAdapter::new(OcrAdapterMode::ContractMismatch);
        let wiring =
            Ph1OsOcrRouteWiring::new(Ph1OsOcrRouteConfig::openai_default(), adapter).unwrap();

        let outcome = wiring
            .run_handoff(&OsOcrAnalyzerForwardBundle::Vision(
                sample_vision_forward_bundle(),
            ))
            .unwrap();
        let OsOcrRouteOutcome::Refused(refuse) = outcome else {
            panic!("expected ocr route refusal");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_PROVIDER_VALIDATION_FAILED
        );
    }

    #[derive(Debug, Clone)]
    struct OcrBridgeContextEngine;

    impl Ph1ContextEngine for OcrBridgeContextEngine {
        fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse {
            match req {
                Ph1ContextRequest::ContextBundleBuild(r) => {
                    let mut ordered = r
                        .source_items
                        .iter()
                        .enumerate()
                        .map(|(idx, src)| {
                            ContextBundleItem::v1(
                                src.item_id.clone(),
                                src.source_engine.clone(),
                                src.source_kind,
                                (idx + 1) as u8,
                                src.content_ref.clone(),
                                src.evidence_ref.clone(),
                                src.sensitivity_private,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    ordered.sort_by(|a, b| a.item_id.cmp(&b.item_id));
                    for (idx, item) in ordered.iter_mut().enumerate() {
                        item.bundle_rank = (idx + 1) as u8;
                    }
                    let selected_item_ids = vec![ordered[0].item_id.clone()];
                    Ph1ContextResponse::ContextBundleBuildOk(
                        ContextBundleBuildOk::v1(
                            ReasonCodeId(0x4358_2201),
                            selected_item_ids,
                            ordered,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ContextRequest::ContextBundleTrim(_) => Ph1ContextResponse::ContextBundleTrimOk(
                    ContextBundleTrimOk::v1(
                        ReasonCodeId(0x4358_2202),
                        ContextValidationStatus::Ok,
                        vec![],
                        true,
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum OcrBridgeNlpMode {
        Chat,
        EngineErrorFailClosed,
    }

    #[derive(Debug, Clone)]
    struct RecordingNlpEngine {
        mode: OcrBridgeNlpMode,
        seen_requests: Rc<RefCell<Vec<Ph1nRequest>>>,
    }

    impl RecordingNlpEngine {
        fn new(mode: OcrBridgeNlpMode) -> Self {
            Self {
                mode,
                seen_requests: Rc::new(RefCell::new(Vec::new())),
            }
        }
    }

    impl Ph1nEngine for RecordingNlpEngine {
        fn run(&self, req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
            self.seen_requests.borrow_mut().push(req.clone());
            match self.mode {
                OcrBridgeNlpMode::Chat => Ok(Ph1nResponse::Chat(
                    Chat::v1("Acknowledged.".to_string(), ReasonCodeId(0x4E00_2201)).unwrap(),
                )),
                OcrBridgeNlpMode::EngineErrorFailClosed => Err(ContractViolation::InvalidValue {
                    field: "ph1n_runtime",
                    reason: "forced nlp failure for fail-closed validation",
                }),
            }
        }
    }

    fn sample_ocr_provider_bundle_from_route_mode(
        mode: OcrAdapterMode,
        source_engine: OsOcrSourceEngine,
    ) -> OsOcrProviderForwardBundle {
        let adapter = RecordingOcrAdapter::new(mode);
        let route_wiring =
            Ph1OsOcrRouteWiring::new(Ph1OsOcrRouteConfig::openai_default(), adapter).unwrap();
        let analyzer_bundle = match source_engine {
            OsOcrSourceEngine::Vision => OsOcrAnalyzerForwardBundle::Vision(sample_vision_forward_bundle()),
            OsOcrSourceEngine::Doc => OsOcrAnalyzerForwardBundle::Doc(sample_doc_forward_bundle()),
        };
        let out = route_wiring.run_handoff(&analyzer_bundle).unwrap();
        match out {
            OsOcrRouteOutcome::Forwarded(bundle) => bundle,
            _ => panic!("expected routed OCR forwarded bundle"),
        }
    }

    fn sample_ocr_provider_bundle_from_route() -> OsOcrProviderForwardBundle {
        sample_ocr_provider_bundle_from_route_mode(OcrAdapterMode::SchemaOk, OsOcrSourceEngine::Vision)
    }

    fn sample_doc_ocr_provider_bundle_from_route() -> OsOcrProviderForwardBundle {
        sample_ocr_provider_bundle_from_route_mode(OcrAdapterMode::SchemaOk, OsOcrSourceEngine::Doc)
    }

    fn base_nlp_request_for_ocr() -> Ph1nRequest {
        let transcript = NlpTranscriptOk::v1_with_metadata(
            "show me the invoice status".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
            vec![],
            Some(
                Ph1cAuditMeta::v1(
                    RouteClassUsed::OnDevice,
                    1,
                    1,
                    SelectedSlot::Primary,
                    RoutingModeUsed::Lead,
                    false,
                    100,
                    QualityBucket::High,
                    QualityBucket::High,
                    QualityBucket::High,
                    None,
                    None,
                    Some("ph1k_handoff_standard".to_string()),
                    Some("openai_google_clarify_v1".to_string()),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        Ph1nRequest::v1(transcript, SessionStateRef::v1(SessionState::Active, false)).unwrap()
    }

    #[test]
    fn at_os_27_ocr_validated_output_feeds_context_and_nlp() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_engine = RecordingNlpEngine::new(OcrBridgeNlpMode::Chat);
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            nlp_engine.clone(),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let ocr_bundle = sample_ocr_provider_bundle_from_route();
        let base_nlp_request = base_nlp_request_for_ocr();
        let out = bridge.run_handoff(&ocr_bundle, &base_nlp_request).unwrap();
        let OsOcrContextNlpOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded OCR->CONTEXT/NLP outcome");
        };
        assert!(!bundle.nlp_fail_closed);
        assert_eq!(bundle.confidence_band, OsOcrConfidenceBand::High);
        assert!(!bundle.clarify_policy_required);
        assert!(matches!(bundle.nlp_output, Ph1nResponse::Chat(_)));
        assert_eq!(
            bundle
                .context_bundle
                .bundle_build
                .ordered_bundle_items
                .len(),
            1
        );
        let item = &bundle.context_bundle.bundle_build.ordered_bundle_items[0];
        assert_eq!(item.source_engine, "PH1.VISION");
        assert_eq!(item.source_kind, ContextSourceKind::VisionEvidence);

        let seen = nlp_engine.seen_requests.borrow();
        assert_eq!(seen.len(), 1);
        assert!(seen[0]
            .transcript_ok
            .transcript_text
            .contains("[OCR:VISION]"));
        assert!(seen[0]
            .transcript_ok
            .transcript_text
            .contains("invoice total due 123.45"));
    }

    #[test]
    fn at_os_27b_ocr_context_source_engine_preserves_doc_provenance() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_engine = RecordingNlpEngine::new(OcrBridgeNlpMode::Chat);
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            nlp_engine,
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let ocr_bundle = sample_doc_ocr_provider_bundle_from_route();
        let base_nlp_request = base_nlp_request_for_ocr();
        let out = bridge.run_handoff(&ocr_bundle, &base_nlp_request).unwrap();
        let OsOcrContextNlpOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded OCR->CONTEXT/NLP outcome");
        };
        let item = &bundle.context_bundle.bundle_build.ordered_bundle_items[0];
        assert_eq!(item.source_engine, "PH1.DOC");
        assert_eq!(item.source_kind, ContextSourceKind::DocEvidence);
    }

    #[test]
    fn at_os_28_ocr_context_nlp_fails_closed_when_context_disabled() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(false),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            RecordingNlpEngine::new(OcrBridgeNlpMode::Chat),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route(),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Refused(refuse) = out else {
            panic!("expected refusal when context wiring is disabled");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_CONTEXT_REFUSED
        );
    }

    #[test]
    fn at_os_29_ocr_context_nlp_fails_closed_when_nlp_disabled() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(false),
            RecordingNlpEngine::new(OcrBridgeNlpMode::Chat),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route(),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Refused(refuse) = out else {
            panic!("expected refusal when NLP wiring is disabled");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_NLP_REFUSED
        );
    }

    #[test]
    fn at_os_30_ocr_context_nlp_preserves_fail_closed_nlp_output() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            RecordingNlpEngine::new(OcrBridgeNlpMode::EngineErrorFailClosed),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route(),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded outcome with fail-closed NLP output");
        };
        assert!(bundle.nlp_fail_closed);
        assert_eq!(bundle.confidence_band, OsOcrConfidenceBand::High);
        assert!(matches!(bundle.nlp_output, Ph1nResponse::Clarify(_)));
    }

    #[test]
    fn at_os_31_ocr_context_nlp_routes_medium_confidence_without_clarify_when_policy_allows() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            RecordingNlpEngine::new(OcrBridgeNlpMode::Chat),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route_mode(
                    OcrAdapterMode::MediumConfidence,
                    OsOcrSourceEngine::Vision,
                ),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded outcome under medium-confidence policy");
        };
        assert_eq!(bundle.confidence_band, OsOcrConfidenceBand::Medium);
        assert!(!bundle.clarify_policy_required);
        assert!(matches!(bundle.nlp_output, Ph1nResponse::Chat(_)));
    }

    #[test]
    fn at_os_32_ocr_context_nlp_low_confidence_blocks_non_clarify_output() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            RecordingNlpEngine::new(OcrBridgeNlpMode::Chat),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route_mode(
                    OcrAdapterMode::LowConfidence,
                    OsOcrSourceEngine::Vision,
                ),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Refused(refuse) = out else {
            panic!("expected clarify-policy refusal for low-confidence non-clarify output");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_OS_OCR_ROUTE_CLARIFY_POLICY_BLOCK
        );
    }

    #[test]
    fn at_os_33_ocr_context_nlp_low_confidence_accepts_ph1_nlp_clarify() {
        let context_wiring = Ph1ContextWiring::new(
            crate::ph1context::Ph1ContextWiringConfig::mvp_v1(true),
            OcrBridgeContextEngine,
        )
        .unwrap();
        let nlp_wiring = Ph1nWiring::new(
            crate::ph1n::Ph1nWiringConfig::mvp_v1(true),
            RecordingNlpEngine::new(OcrBridgeNlpMode::EngineErrorFailClosed),
        )
        .unwrap();
        let bridge = Ph1OsOcrContextNlpWiring::new(
            Ph1OsOcrContextNlpConfig::mvp_v1(),
            context_wiring,
            nlp_wiring,
        )
        .unwrap();

        let out = bridge
            .run_handoff(
                &sample_ocr_provider_bundle_from_route_mode(
                    OcrAdapterMode::LowConfidence,
                    OsOcrSourceEngine::Vision,
                ),
                &base_nlp_request_for_ocr(),
            )
            .unwrap();
        let OsOcrContextNlpOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded low-confidence outcome when PH1.NLP returns clarify");
        };
        assert_eq!(bundle.confidence_band, OsOcrConfidenceBand::Low);
        assert!(bundle.clarify_policy_required);
        assert!(matches!(bundle.nlp_output, Ph1nResponse::Clarify(_)));
    }

    fn sample_self_heal_chain_input(
        escalation_required: bool,
        bcast_id: Option<&str>,
    ) -> OsSelfHealChainInput {
        let correlation_id = CorrelationId(44_001);
        let turn_id = TurnId(17);

        let feedback_event = FeedbackEventRecord::v1(
            "feedback_event_1".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            correlation_id,
            turn_id,
            FeedbackEventType::ToolFail,
            ReasonCodeId(0x4642_2001),
            "evidence:selfheal:1".to_string(),
            "idem_feedback_1".to_string(),
            FeedbackMetrics::v1(
                420,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["ocr_text".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap();

        let feedback_bundle = FeedbackForwardBundle::v1(
            correlation_id,
            turn_id,
            FeedbackEventCollectOk::v1(
                ReasonCodeId(0x4642_2002),
                "feedback_candidate_1".to_string(),
                vec![FeedbackSignalCandidate::v1(
                    "feedback_candidate_1".to_string(),
                    FeedbackEventType::ToolFail,
                    "tool_fail_rate".to_string(),
                    FeedbackSignalTarget::LearnPackage,
                    -220,
                    12,
                    "evidence:selfheal:1".to_string(),
                )
                .unwrap()],
                true,
                true,
            )
            .unwrap(),
            FeedbackSignalEmitOk::v1(
                ReasonCodeId(0x4642_2003),
                FeedbackValidationStatus::Ok,
                vec!["ok".to_string()],
                true,
                true,
                true,
                true,
            )
            .unwrap(),
        )
        .unwrap();

        let learn_turn_input = LearnTurnInput::v2(
            correlation_id,
            turn_id,
            "tenant_1".to_string(),
            vec![LearnSignal::v1(
                "learn_signal_1".to_string(),
                "tenant_1".to_string(),
                LearnSignalType::ToolFail,
                LearnScope::Tenant,
                Some("tenant_1".to_string()),
                "ocr_fail_rate".to_string(),
                -180,
                3,
                false,
                false,
                false,
                "evidence:selfheal:1".to_string(),
            )
            .unwrap()],
            vec![LearnTargetEngine::Pae],
            true,
            true,
            0,
            0,
            0,
        )
        .unwrap();

        let learn_bundle = LearnForwardBundle::v1(
            correlation_id,
            turn_id,
            LearnSignalAggregateOk::v1(
                ReasonCodeId(0x4C45_2001),
                "artifact_1".to_string(),
                vec![LearnArtifactCandidate::v1(
                    "artifact_1".to_string(),
                    LearnArtifactTarget::PaeRoutingWeights,
                    LearnScope::Tenant,
                    Some("tenant_1".to_string()),
                    2,
                    140,
                    "prov:selfheal:1".to_string(),
                    Some("artifact_prev".to_string()),
                    true,
                )
                .unwrap()],
                true,
                true,
                true,
                true,
            )
            .unwrap(),
            LearnArtifactPackageBuildOk::v1(
                ReasonCodeId(0x4C45_2002),
                LearnValidationStatus::Ok,
                vec!["ok".to_string()],
                vec![LearnTargetEngine::Pae],
                true,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
            Vec::new(),
        )
        .unwrap();

        let pae_turn_input = PaeTurnInput::v1(
            correlation_id,
            turn_id,
            "tenant_1".to_string(),
            "device_profile_1".to_string(),
            PaeMode::Shadow,
            vec![PaeSignalVector::v1(
                "pae_signal_1".to_string(),
                PaeSignalSource::Learn,
                PaeRouteDomain::Tooling,
                "ocr_fail_rate".to_string(),
                -160,
                9_100,
                true,
                "evidence:selfheal:1".to_string(),
            )
            .unwrap()],
            vec![PaePolicyCandidate::v1(
                "candidate_1".to_string(),
                PaeRouteDomain::Tooling,
                PaeProviderSlot::Primary,
                PaeMode::Assist,
                220,
                250,
                120,
                90,
                180,
                Some("artifact_1".to_string()),
                None,
            )
            .unwrap()],
            vec![PaeTargetEngine::Ph1C],
            true,
            100,
            800,
            3,
            0,
            true,
        )
        .unwrap();

        let pae_bundle = PaeForwardBundle::v1(
            correlation_id,
            turn_id,
            PaePolicyScoreBuildOk::v1(
                ReasonCodeId(0x5041_2001),
                "candidate_1".to_string(),
                vec![PaeScoreEntry::v1(
                    "candidate_1".to_string(),
                    PaeRouteDomain::Tooling,
                    PaeProviderSlot::Primary,
                    PaeMode::Assist,
                    1_950,
                    2_300,
                    100,
                    80,
                    70,
                    180,
                )
                .unwrap()],
                PaeMode::Assist,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
            PaeAdaptationHintEmitOk::v1(
                ReasonCodeId(0x5041_2002),
                PaeValidationStatus::Ok,
                vec![],
                vec![PaeTargetEngine::Ph1C],
                vec![PaeAdaptationHint::v1(
                    "hint_1".to_string(),
                    PaeTargetEngine::Ph1C,
                    PaeRouteDomain::Tooling,
                    "routing_weight".to_string(),
                    "shift_primary".to_string(),
                    8_900,
                    "prov:selfheal:1".to_string(),
                )
                .unwrap()],
                true,
                true,
                true,
            )
            .unwrap(),
        )
        .unwrap();

        OsSelfHealChainInput {
            feedback_event,
            feedback_bundle,
            learn_turn_input,
            learn_bundle,
            pae_turn_input,
            pae_bundle,
            owner_engine: "PH1.OS".to_string(),
            first_seen_at: MonotonicTimeNs(1_000),
            last_seen_at: MonotonicTimeNs(2_000),
            containment_action: FailureContainmentAction::FailClosedRefuse,
            escalation_required,
            unresolved_reason: if escalation_required {
                Some("resolution not yet proven".to_string())
            } else {
                None
            },
            bcast_id: bcast_id.map(str::to_string),
            provider_context: Some(
                FailureProviderContext::v1(
                    PaeRouteDomain::Tooling,
                    PaeProviderSlot::Primary,
                    Ph1dProviderTask::OcrTextExtract,
                    12_000,
                    95,
                    false,
                )
                .unwrap(),
            ),
            governance_required: false,
            governance_ticket_ref: None,
            approved_by: None,
            evaluated_at: MonotonicTimeNs(2_100),
            ph1c_superiority_pack: None,
        }
    }

    #[test]
    fn at_os_34_self_heal_chain_completeness_and_release_gate_pass() {
        let input = sample_self_heal_chain_input(false, None);
        let chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        assert_eq!(
            chain.problem_card.latest_failure_id,
            chain.failure_event.failure_id
        );
        assert_eq!(chain.fix_card.problem_id, chain.problem_card.problem_id);
        assert_eq!(chain.promotion_decision.fix_id, chain.fix_card.fix_id);
        check_self_heal_release_gate(
            &chain,
            MonotonicTimeNs(2_300),
            OsSelfHealReleaseGateConfig::mvp_v1(),
        )
        .unwrap();
    }

    #[test]
    fn at_os_35_self_heal_chain_fails_closed_on_feedback_correlation_mismatch() {
        let mut input = sample_self_heal_chain_input(false, None);
        input.feedback_bundle.correlation_id = CorrelationId(99_991);
        let err = build_self_heal_chain_from_engine_outputs(&input)
            .expect_err("correlation mismatch must fail closed");
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
    fn at_os_36_self_heal_chain_replay_is_deterministic() {
        let input = sample_self_heal_chain_input(false, None);
        let chain_a = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        let chain_b = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        assert_eq!(
            chain_a.failure_event.fingerprint,
            chain_b.failure_event.fingerprint
        );
        assert_eq!(
            chain_a.problem_card.problem_id,
            chain_b.problem_card.problem_id
        );
        assert_eq!(chain_a.fix_card.fix_id, chain_b.fix_card.fix_id);
        assert_eq!(
            chain_a.promotion_decision.decision_id,
            chain_b.promotion_decision.decision_id
        );
    }

    #[test]
    fn at_os_37_self_heal_release_gate_blocks_escalation_without_bcast_proof() {
        let input = sample_self_heal_chain_input(true, None);
        let chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        let err = check_self_heal_release_gate(
            &chain,
            MonotonicTimeNs(2_300),
            OsSelfHealReleaseGateConfig::mvp_v1(),
        )
        .expect_err("escalation without bcast_id must be blocked");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "check_self_heal_release_gate.problem_card.bcast_id");
            }
            _ => panic!("expected invalid-value violation"),
        }
    }

    #[test]
    fn at_os_38_self_heal_release_gate_blocks_stale_decision_evidence() {
        let input = sample_self_heal_chain_input(false, None);
        let chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        let err = check_self_heal_release_gate(
            &chain,
            MonotonicTimeNs(2_300),
            OsSelfHealReleaseGateConfig {
                max_problem_age_ns: 500,
                max_decision_age_ns: 50,
            },
        )
        .expect_err("stale decision evidence must block release");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "check_self_heal_release_gate.promotion_decision.evaluated_at"
                );
            }
            _ => panic!("expected invalid-value violation"),
        }
    }

    #[test]
    fn at_os_39_builder_remediation_maps_recurring_cluster_to_offline_input() {
        let input = sample_self_heal_chain_input(false, None);
        let chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        assert_eq!(
            chain.promotion_decision.decision_action,
            PromotionDecisionAction::Promote
        );

        let builder_input = map_recurring_failure_cluster_to_builder_offline_input(
            &chain,
            MonotonicTimeNs(2_300),
            OsBuilderRemediationConfig::mvp_v1(),
            OsBuilderPromotionGateEvidence::strict_required_pass(),
        )
        .unwrap()
        .expect("recurring cluster should map to builder offline input");

        builder_input.validate().unwrap();
        assert_eq!(builder_input.outcome_entries.len(), 4);
        assert!(builder_input
            .outcome_entries
            .iter()
            .any(|entry| entry.engine_id == "PH1.FEEDBACK"));
        assert!(builder_input
            .outcome_entries
            .iter()
            .any(|entry| entry.engine_id == "PH1.LEARN"));
        assert!(builder_input.offline_pipeline_only);
    }

    #[test]
    fn at_os_40_builder_remediation_skips_non_recurring_cluster() {
        let input = sample_self_heal_chain_input(false, None);
        let mut chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        chain.problem_card.recurrence_count = 1;
        chain.validate().unwrap();

        let out = map_recurring_failure_cluster_to_builder_offline_input(
            &chain,
            MonotonicTimeNs(2_300),
            OsBuilderRemediationConfig::mvp_v1(),
            OsBuilderPromotionGateEvidence::strict_required_pass(),
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_os_41_builder_remediation_blocks_promote_without_permission_gate_evidence() {
        let input = sample_self_heal_chain_input(false, None);
        let chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        assert_eq!(
            chain.promotion_decision.decision_action,
            PromotionDecisionAction::Promote
        );

        let err = map_recurring_failure_cluster_to_builder_offline_input(
            &chain,
            MonotonicTimeNs(2_300),
            OsBuilderRemediationConfig::mvp_v1(),
            OsBuilderPromotionGateEvidence {
                code_permission_gate_passed: false,
                launch_permission_gate_passed: true,
                release_hard_gate_passed: true,
            },
        )
        .expect_err("promote path must fail closed without code permission proof");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "check_builder_remediation_promotion_gate.code_permission_gate_passed"
                );
            }
            _ => panic!("expected invalid-value violation"),
        }
    }

    #[test]
    fn at_os_42_builder_remediation_allows_non_promote_without_gate_proofs() {
        let input = sample_self_heal_chain_input(false, None);
        let mut chain = build_self_heal_chain_from_engine_outputs(&input).unwrap();
        chain.promotion_decision.decision_action = PromotionDecisionAction::Hold;
        chain.promotion_decision.to_mode = chain.promotion_decision.from_mode;
        chain.promotion_decision.promotion_eligible = false;
        chain.validate().unwrap();

        let out = map_recurring_failure_cluster_to_builder_offline_input(
            &chain,
            MonotonicTimeNs(2_300),
            OsBuilderRemediationConfig::mvp_v1(),
            OsBuilderPromotionGateEvidence {
                code_permission_gate_passed: false,
                launch_permission_gate_passed: false,
                release_hard_gate_passed: false,
            },
        )
        .unwrap();
        assert!(out.is_some());
    }

    #[test]
    fn at_os_43_ph1c_gold_loop_miss_and_correction_route_to_learn_and_pae_deterministically() {
        let (miss_event, correction_event, feedback_input) =
            build_ph1c_gold_loop_feedback_turn_input();
        assert!(feedback_input
            .events
            .iter()
            .all(|event| event.gold_status == FeedbackGoldStatus::Pending));
        let mut verified_feedback_input = feedback_input.clone();
        for event in &mut verified_feedback_input.events {
            event.gold_status = FeedbackGoldStatus::Verified;
            event.gold_provenance_method =
                Some(FeedbackGoldProvenanceMethod::VerifiedHumanCorrection);
        }

        let feedback_wiring = Ph1FeedbackWiring::new(
            Ph1FeedbackWiringConfig::mvp_v1(true),
            FeedbackRuntimeEngineAdapter::mvp_v1(),
        )
        .unwrap();
        let feedback_outcome_a = feedback_wiring.run_turn(&verified_feedback_input).unwrap();
        let feedback_outcome_b = feedback_wiring.run_turn(&verified_feedback_input).unwrap();
        let FeedbackWiringOutcome::Forwarded(feedback_bundle_a) = feedback_outcome_a else {
            panic!("feedback path must forward for gold-loop miss/correction events");
        };
        let FeedbackWiringOutcome::Forwarded(feedback_bundle_b) = feedback_outcome_b else {
            panic!("feedback path replay must forward for deterministic proof");
        };
        assert_eq!(feedback_bundle_a, feedback_bundle_b);
        assert!(feedback_bundle_a
            .event_collect
            .ordered_signal_candidates
            .iter()
            .all(|candidate| candidate.target == FeedbackSignalTarget::PaeScorecard));
        assert!(feedback_bundle_a.signal_emit.emits_learn);
        assert!(feedback_bundle_a.signal_emit.emits_pae);

        let learn_wiring = Ph1LearnWiring::new(
            Ph1LearnWiringConfig::mvp_v1(true),
            LearnRuntimeEngineAdapter::mvp_v1(),
        )
        .unwrap();
        let (learn_turn_input_a, learn_outcome_a) = route_feedback_into_learn_wiring(
            &learn_wiring,
            &verified_feedback_input,
            &feedback_bundle_a,
            FeedbackLearnRouteConfig::mvp_v1(),
        )
        .unwrap();
        let (learn_turn_input_b, learn_outcome_b) = route_feedback_into_learn_wiring(
            &learn_wiring,
            &verified_feedback_input,
            &feedback_bundle_a,
            FeedbackLearnRouteConfig::mvp_v1(),
        )
        .unwrap();
        assert_eq!(learn_turn_input_a, learn_turn_input_b);
        assert_eq!(
            learn_turn_input_a.requested_target_engines,
            vec![LearnTargetEngine::Pae]
        );
        assert!(learn_turn_input_a
            .signals
            .iter()
            .all(|signal| signal.gold_case_id.is_some()));
        let LearnWiringOutcome::Forwarded(learn_bundle_a) = learn_outcome_a else {
            panic!(
                "learn path must forward for feedback gold-loop signals: {:?}",
                learn_outcome_a
            );
        };
        let LearnWiringOutcome::Forwarded(learn_bundle_b) = learn_outcome_b else {
            panic!(
                "learn path replay must forward for deterministic proof: {:?}",
                learn_outcome_b
            );
        };
        assert_eq!(learn_bundle_a, learn_bundle_b);
        assert!(learn_bundle_a.signal_aggregate.advisory_only);
        assert!(learn_bundle_a.signal_aggregate.no_execution_authority);
        assert!(learn_bundle_a.artifact_package_build.advisory_only);
        assert!(learn_bundle_a.artifact_package_build.no_execution_authority);

        let pae_turn_input = build_pae_turn_input_from_learn(&learn_turn_input_a, &learn_bundle_a);
        let pae_wiring = Ph1PaeWiring::new(
            Ph1PaeWiringConfig::mvp_v1(true),
            PaeRuntimeEngineAdapter::mvp_v1(),
        )
        .unwrap();
        let pae_outcome_a = pae_wiring.run_turn(&pae_turn_input).unwrap();
        let pae_outcome_b = pae_wiring.run_turn(&pae_turn_input).unwrap();
        let PaeWiringOutcome::Forwarded(pae_bundle_a) = pae_outcome_a else {
            panic!("pae path must forward for learn target=PAE signals");
        };
        let PaeWiringOutcome::Forwarded(pae_bundle_b) = pae_outcome_b else {
            panic!("pae replay must forward for deterministic proof");
        };
        assert_eq!(pae_bundle_a, pae_bundle_b);

        let miss_chain_input_a = build_gold_loop_chain_input(
            miss_event.clone(),
            feedback_bundle_a.clone(),
            learn_turn_input_a.clone(),
            learn_bundle_a.clone(),
            pae_turn_input.clone(),
            pae_bundle_a.clone(),
        );
        let miss_chain_input_b = build_gold_loop_chain_input(
            miss_event,
            feedback_bundle_a.clone(),
            learn_turn_input_a.clone(),
            learn_bundle_a.clone(),
            pae_turn_input.clone(),
            pae_bundle_a.clone(),
        );
        let correction_chain_input_a = build_gold_loop_chain_input(
            correction_event.clone(),
            feedback_bundle_a.clone(),
            learn_turn_input_a.clone(),
            learn_bundle_a.clone(),
            pae_turn_input.clone(),
            pae_bundle_a.clone(),
        );
        let correction_chain_input_b = build_gold_loop_chain_input(
            correction_event,
            feedback_bundle_a,
            learn_turn_input_a,
            learn_bundle_a,
            pae_turn_input,
            pae_bundle_a,
        );

        let miss_chain_a = build_self_heal_chain_from_engine_outputs(&miss_chain_input_a).unwrap();
        let miss_chain_b = build_self_heal_chain_from_engine_outputs(&miss_chain_input_b).unwrap();
        assert_eq!(
            miss_chain_a.failure_event.fingerprint,
            miss_chain_b.failure_event.fingerprint
        );
        assert_eq!(
            miss_chain_a.problem_card.problem_id,
            miss_chain_b.problem_card.problem_id
        );
        assert_eq!(miss_chain_a.fix_card.fix_id, miss_chain_b.fix_card.fix_id);
        assert_eq!(
            miss_chain_a.promotion_decision.decision_id,
            miss_chain_b.promotion_decision.decision_id
        );

        let correction_chain_a =
            build_self_heal_chain_from_engine_outputs(&correction_chain_input_a).unwrap();
        let correction_chain_b =
            build_self_heal_chain_from_engine_outputs(&correction_chain_input_b).unwrap();
        assert_eq!(
            correction_chain_a.failure_event.fingerprint,
            correction_chain_b.failure_event.fingerprint
        );
        assert_eq!(
            correction_chain_a.problem_card.problem_id,
            correction_chain_b.problem_card.problem_id
        );
        assert_eq!(
            correction_chain_a.fix_card.fix_id,
            correction_chain_b.fix_card.fix_id
        );
        assert_eq!(
            correction_chain_a.promotion_decision.decision_id,
            correction_chain_b.promotion_decision.decision_id
        );
        assert_ne!(
            miss_chain_a.failure_event.fingerprint,
            correction_chain_a.failure_event.fingerprint
        );
    }

    #[test]
    fn at_os_44_ph1c_self_heal_chain_requires_superiority_pack() {
        let mut input = sample_self_heal_chain_input(false, None);
        input.owner_engine = "PH1.C".to_string();
        input.ph1c_superiority_pack = None;
        let err = build_self_heal_chain_from_engine_outputs(&input)
            .expect_err("ph1c self-heal chain must fail closed without superiority pack");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "os_self_heal_chain_input.ph1c_superiority_pack");
            }
            other => panic!("expected invalid-value violation, got: {other:?}"),
        }
    }

    #[test]
    fn at_os_45_ph1c_self_heal_chain_blocks_failed_superiority_gate() {
        let mut input = sample_self_heal_chain_input(false, None);
        input.owner_engine = "PH1.C".to_string();
        let mut pack = sample_ph1c_superiority_pack();
        pack.rows
            .retain(|row| row.lane != SuperiorityLane::ChatgptAb);
        input.ph1c_superiority_pack = Some(pack);
        let err = build_self_heal_chain_from_engine_outputs(&input)
            .expect_err("ph1c self-heal chain must fail closed when superiority gate fails");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "build_self_heal_chain_from_engine_outputs.ph1c_superiority_gate"
                );
            }
            other => panic!("expected invalid-value violation, got: {other:?}"),
        }
    }
}
