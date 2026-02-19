#![forbid(unsafe_code)]

use std::{cmp::min, collections::BTreeSet};

use selene_engines::ph1_voice_id::{
    EnrolledSpeaker as EngineEnrolledSpeaker, VoiceIdObservation as EngineVoiceIdObservation,
};
use selene_kernel_contracts::ph1_voice_id::{Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1os::{
    OsCapabilityId, OsDecisionComputeOk, OsDecisionComputeRequest, OsGateDecision, OsNextMove,
    OsOutcomeUtilizationEntry, OsPolicyEvaluateOk, OsPolicyEvaluateRequest, OsRefuse,
    OsRequestEnvelope, Ph1OsRequest, Ph1OsResponse, OS_CLARIFY_OWNER_ENGINE_ID,
};
use selene_kernel_contracts::{ContractViolation, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

use crate::device_artifact_sync::{self, DeviceArtifactSyncSenderRuntime};
use crate::ph1_voice_id::{
    Ph1VoiceIdLiveRuntime, VoiceIdentityChannel, VoiceIdentityPlatform,
    VoiceIdentityRuntimeContext, VoiceIdentitySignalScope,
};

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
pub struct OsVoiceLiveTurnInput {
    pub top_level_turn_input: OsTopLevelTurnInput,
    pub voice_id_request: Ph1VoiceIdRequest,
    pub actor_user_id: UserId,
    pub tenant_id: Option<String>,
    pub device_id: Option<selene_kernel_contracts::ph1j::DeviceId>,
    pub enrolled_speakers: Vec<EngineEnrolledSpeaker>,
    pub observation: EngineVoiceIdObservation,
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
        })
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
    use selene_engines::ph1_voice_id::VoiceIdObservation as EngineVoiceIdObservation;
    use selene_kernel_contracts::ph1_voice_id::{
        DeviceTrustLevel, Ph1VoiceIdRequest, Ph1VoiceIdResponse, UserId,
    };
    use selene_kernel_contracts::ph1j::{AuditEngine, DeviceId, PayloadKey};
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, AudioStreamRef, ChannelCount,
        Confidence, FrameDurationMs, SampleFormat, SampleRateHz, SpeechLikeness, VadEvent,
    };
    use selene_kernel_contracts::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use selene_kernel_contracts::ph1os::{OsOutcomeActionClass, OsOutcomeUtilizationEntry};
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
}
