#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1OS_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const OS_CLARIFY_OWNER_ENGINE_ID: &str = "PH1.NLP";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsCapabilityId {
    OsPolicyEvaluate,
    OsDecisionCompute,
}

impl OsCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            OsCapabilityId::OsPolicyEvaluate => "OS_POLICY_EVALUATE",
            OsCapabilityId::OsDecisionCompute => "OS_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsNextMove {
    Respond,
    Clarify,
    Confirm,
    DispatchTool,
    DispatchSimulation,
    Explain,
    Wait,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsOutcomeActionClass {
    ActNow,
    QueueLearn,
    AuditOnly,
    Drop,
}

impl OsOutcomeActionClass {
    pub fn as_str(self) -> &'static str {
        match self {
            OsOutcomeActionClass::ActNow => "ACT_NOW",
            OsOutcomeActionClass::QueueLearn => "QUEUE_LEARN",
            OsOutcomeActionClass::AuditOnly => "AUDIT_ONLY",
            OsOutcomeActionClass::Drop => "DROP",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OsGateDecision {
    Allow,
    Deny,
    Escalate,
    Invalid,
}

impl OsGateDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            OsGateDecision::Allow => "ALLOW",
            OsGateDecision::Deny => "DENY",
            OsGateDecision::Escalate => "ESCALATE",
            OsGateDecision::Invalid => "INVALID",
        }
    }

    pub fn is_allow(self) -> bool {
        matches!(self, OsGateDecision::Allow)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsOutcomeUtilizationEntry {
    pub schema_version: SchemaVersion,
    pub engine_id: String,
    pub outcome_type: String,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub action_class: OsOutcomeActionClass,
    pub consumed_by: String,
    pub latency_cost_ms: u32,
    pub decision_delta: bool,
    pub reason_code: ReasonCodeId,
}

impl OsOutcomeUtilizationEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        engine_id: String,
        outcome_type: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        action_class: OsOutcomeActionClass,
        consumed_by: String,
        latency_cost_ms: u32,
        decision_delta: bool,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let entry = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            engine_id,
            outcome_type,
            correlation_id,
            turn_id,
            action_class,
            consumed_by,
            latency_cost_ms,
            decision_delta,
            reason_code,
        };
        entry.validate()?;
        Ok(entry)
    }
}

impl Validate for OsOutcomeUtilizationEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_outcome_utilization_entry.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }

        validate_token_ascii(
            "os_outcome_utilization_entry.engine_id",
            &self.engine_id,
            64,
        )?;
        validate_token_ascii(
            "os_outcome_utilization_entry.outcome_type",
            &self.outcome_type,
            64,
        )?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token_ascii(
            "os_outcome_utilization_entry.consumed_by",
            &self.consumed_by,
            64,
        )?;

        if matches!(
            self.action_class,
            OsOutcomeActionClass::ActNow | OsOutcomeActionClass::QueueLearn
        ) && self.consumed_by == "NONE"
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_outcome_utilization_entry.consumed_by",
                reason: "ACT_NOW and QUEUE_LEARN outcomes require consumed_by owner (not NONE)",
            });
        }

        if self.latency_cost_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_outcome_utilization_entry.latency_cost_ms",
                reason: "must be <= 60000",
            });
        }

        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "os_outcome_utilization_entry.reason_code",
                reason: "must be non-zero",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_guard_failures: u8,
    pub max_diagnostics: u8,
    pub max_outcome_entries: u16,
}

impl OsRequestEnvelope {
    pub const DEFAULT_MAX_OUTCOME_ENTRIES: u16 = 128;

    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_guard_failures: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_outcome_budget(
            correlation_id,
            turn_id,
            max_guard_failures,
            max_diagnostics,
            Self::DEFAULT_MAX_OUTCOME_ENTRIES,
        )
    }

    pub fn v1_with_outcome_budget(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_guard_failures: u8,
        max_diagnostics: u8,
        max_outcome_entries: u16,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_guard_failures,
            max_diagnostics,
            max_outcome_entries,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for OsRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_request_envelope.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_guard_failures == 0 || self.max_guard_failures > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "os_request_envelope.max_guard_failures",
                reason: "must be within 1..=16",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "os_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if self.max_outcome_entries == 0 || self.max_outcome_entries > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "os_request_envelope.max_outcome_entries",
                reason: "must be within 1..=1024",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: OsRequestEnvelope,
    pub session_active: bool,
    pub transcript_ok: bool,
    pub nlp_confidence_high: bool,
    pub requires_confirmation: bool,
    pub confirmation_received: bool,
    pub prompt_policy_required: bool,
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
    pub no_engine_to_engine_calls: bool,
    pub no_simulation_no_execution: bool,
    pub one_turn_one_move: bool,
    pub optional_budget_enforced: bool,
    pub optional_invocations_requested: u16,
    pub optional_invocations_budget: u16,
    pub optional_invocations_skipped_budget: u16,
    pub optional_latency_budget_ms: u32,
    pub optional_latency_estimated_ms: u32,
    pub outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
}

impl OsPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_and_outcomes(
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            Vec::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_governance(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_and_outcomes(
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            Vec::new(),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_outcomes(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_and_outcomes(
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            outcome_utilization_entries,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_governance_and_outcomes(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_governance_outcomes_and_optional_budget(
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
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
    pub fn v1_with_governance_outcomes_and_optional_budget(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        optional_budget_enforced: bool,
        optional_invocations_requested: u16,
        optional_invocations_budget: u16,
        optional_invocations_skipped_budget: u16,
        optional_latency_budget_ms: u32,
        optional_latency_estimated_ms: u32,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_outcomes_internal(
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            optional_budget_enforced,
            optional_invocations_requested,
            optional_invocations_budget,
            optional_invocations_skipped_budget,
            optional_latency_budget_ms,
            optional_latency_estimated_ms,
            outcome_utilization_entries,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn v1_with_outcomes_internal(
        envelope: OsRequestEnvelope,
        session_active: bool,
        transcript_ok: bool,
        nlp_confidence_high: bool,
        requires_confirmation: bool,
        confirmation_received: bool,
        prompt_policy_required: bool,
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
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        optional_budget_enforced: bool,
        optional_invocations_requested: u16,
        optional_invocations_budget: u16,
        optional_invocations_skipped_budget: u16,
        optional_latency_budget_ms: u32,
        optional_latency_estimated_ms: u32,
        outcome_utilization_entries: Vec<OsOutcomeUtilizationEntry>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            envelope,
            session_active,
            transcript_ok,
            nlp_confidence_high,
            requires_confirmation,
            confirmation_received,
            prompt_policy_required,
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
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            optional_budget_enforced,
            optional_invocations_requested,
            optional_invocations_budget,
            optional_invocations_skipped_budget,
            optional_latency_budget_ms,
            optional_latency_estimated_ms,
            outcome_utilization_entries,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for OsPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;

        if self.tool_requested && self.simulation_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.tool_requested",
                reason: "tool_requested and simulation_requested cannot both be true",
            });
        }
        if !self.prompt_policy_required && !self.prompt_policy_gate_ok {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.prompt_policy_gate_ok",
                reason: "must be true when prompt_policy_required=false",
            });
        }

        if !self.no_engine_to_engine_calls {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.no_engine_to_engine_calls",
                reason: "must be true",
            });
        }
        if !self.no_simulation_no_execution {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.no_simulation_no_execution",
                reason: "must be true",
            });
        }
        if !self.one_turn_one_move {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.one_turn_one_move",
                reason: "must be true",
            });
        }
        if !self.optional_budget_enforced {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_budget_enforced",
                reason: "must be true (fail-closed on optional-budget policy drift)",
            });
        }
        if self.optional_invocations_requested > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_invocations_requested",
                reason: "must be within 0..=64",
            });
        }
        if self.optional_invocations_budget > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_invocations_budget",
                reason: "must be within 0..=64",
            });
        }
        if self.optional_invocations_skipped_budget > self.optional_invocations_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_invocations_skipped_budget",
                reason: "cannot exceed optional_invocations_requested",
            });
        }
        if self.optional_invocations_skipped_budget
            != self
                .optional_invocations_requested
                .saturating_sub(self.optional_invocations_budget)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_invocations_skipped_budget",
                reason: "must equal requested.saturating_sub(optional_invocations_budget)",
            });
        }
        if self.optional_latency_budget_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_latency_budget_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_latency_estimated_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > self.optional_latency_budget_ms {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.optional_latency_estimated_ms",
                reason: "must be <= optional_latency_budget_ms",
            });
        }

        if self.outcome_utilization_entries.len() > self.envelope.max_outcome_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_request.outcome_utilization_entries",
                reason: "exceeds envelope max_outcome_entries",
            });
        }
        for entry in &self.outcome_utilization_entries {
            entry.validate()?;
            if entry.correlation_id != self.envelope.correlation_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_policy_evaluate_request.outcome_utilization_entries.correlation_id",
                    reason: "must match envelope correlation_id",
                });
            }
            if entry.turn_id != self.envelope.turn_id {
                return Err(ContractViolation::InvalidValue {
                    field: "os_policy_evaluate_request.outcome_utilization_entries.turn_id",
                    reason: "must match envelope turn_id",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: OsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub session_gate_ok: bool,
    pub understanding_gate_ok: bool,
    pub confirmation_gate_ok: bool,
    pub prompt_policy_gate_ok: bool,
    pub policy_gate_ok: bool,
    pub tenant_gate_ok: bool,
    pub gov_gate_ok: bool,
    pub quota_gate_ok: bool,
    pub work_gate_ok: bool,
    pub capreq_gate_ok: bool,
    pub governance_contradiction_detected: bool,
    pub governance_decision_trace: Vec<String>,
    pub access_gate_ok: bool,
    pub blueprint_gate_ok: bool,
    pub simulation_gate_ok: bool,
    pub idempotency_gate_ok: bool,
    pub lease_gate_ok: bool,
    pub tool_dispatch_allowed: bool,
    pub simulation_dispatch_allowed: bool,
    pub execution_allowed: bool,
    pub guard_failures: Vec<String>,
    pub no_engine_to_engine_calls: bool,
    pub no_simulation_no_execution: bool,
    pub one_turn_one_move: bool,
    pub outcome_entries_evaluated: u16,
    pub classification_coverage_pct: u8,
    pub unresolved_outcomes: u16,
    pub gate_u1_classification_complete: bool,
    pub gate_u2_unresolved_outcomes_zero: bool,
    pub gate_u3_optional_budget_enforced: bool,
    pub optional_invocations_requested: u16,
    pub optional_invocations_budget: u16,
    pub optional_invocations_skipped_budget: u16,
    pub optional_latency_budget_ms: u32,
    pub optional_latency_estimated_ms: u32,
}

impl OsPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        session_gate_ok: bool,
        understanding_gate_ok: bool,
        confirmation_gate_ok: bool,
        prompt_policy_gate_ok: bool,
        access_gate_ok: bool,
        blueprint_gate_ok: bool,
        simulation_gate_ok: bool,
        idempotency_gate_ok: bool,
        lease_gate_ok: bool,
        tool_dispatch_allowed: bool,
        simulation_dispatch_allowed: bool,
        execution_allowed: bool,
        guard_failures: Vec<String>,
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_outcome_gates(
            reason_code,
            session_gate_ok,
            understanding_gate_ok,
            confirmation_gate_ok,
            prompt_policy_gate_ok,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            vec![
                "POLICY:ALLOW".to_string(),
                "TENANT:ALLOW".to_string(),
                "GOV:ALLOW".to_string(),
                "QUOTA:ALLOW".to_string(),
                "WORK:ALLOW".to_string(),
                "CAPREQ:ALLOW".to_string(),
            ],
            access_gate_ok,
            blueprint_gate_ok,
            simulation_gate_ok,
            idempotency_gate_ok,
            lease_gate_ok,
            tool_dispatch_allowed,
            simulation_dispatch_allowed,
            execution_allowed,
            guard_failures,
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            0,
            100,
            0,
            true,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_outcome_gates(
        reason_code: ReasonCodeId,
        session_gate_ok: bool,
        understanding_gate_ok: bool,
        confirmation_gate_ok: bool,
        prompt_policy_gate_ok: bool,
        policy_gate_ok: bool,
        tenant_gate_ok: bool,
        gov_gate_ok: bool,
        quota_gate_ok: bool,
        work_gate_ok: bool,
        capreq_gate_ok: bool,
        governance_contradiction_detected: bool,
        governance_decision_trace: Vec<String>,
        access_gate_ok: bool,
        blueprint_gate_ok: bool,
        simulation_gate_ok: bool,
        idempotency_gate_ok: bool,
        lease_gate_ok: bool,
        tool_dispatch_allowed: bool,
        simulation_dispatch_allowed: bool,
        execution_allowed: bool,
        guard_failures: Vec<String>,
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        outcome_entries_evaluated: u16,
        classification_coverage_pct: u8,
        unresolved_outcomes: u16,
        gate_u1_classification_complete: bool,
        gate_u2_unresolved_outcomes_zero: bool,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_outcome_gates_and_optional_budget(
            reason_code,
            session_gate_ok,
            understanding_gate_ok,
            confirmation_gate_ok,
            prompt_policy_gate_ok,
            policy_gate_ok,
            tenant_gate_ok,
            gov_gate_ok,
            quota_gate_ok,
            work_gate_ok,
            capreq_gate_ok,
            governance_contradiction_detected,
            governance_decision_trace,
            access_gate_ok,
            blueprint_gate_ok,
            simulation_gate_ok,
            idempotency_gate_ok,
            lease_gate_ok,
            tool_dispatch_allowed,
            simulation_dispatch_allowed,
            execution_allowed,
            guard_failures,
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            outcome_entries_evaluated,
            classification_coverage_pct,
            unresolved_outcomes,
            gate_u1_classification_complete,
            gate_u2_unresolved_outcomes_zero,
            true,
            0,
            0,
            0,
            0,
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_outcome_gates_and_optional_budget(
        reason_code: ReasonCodeId,
        session_gate_ok: bool,
        understanding_gate_ok: bool,
        confirmation_gate_ok: bool,
        prompt_policy_gate_ok: bool,
        policy_gate_ok: bool,
        tenant_gate_ok: bool,
        gov_gate_ok: bool,
        quota_gate_ok: bool,
        work_gate_ok: bool,
        capreq_gate_ok: bool,
        governance_contradiction_detected: bool,
        governance_decision_trace: Vec<String>,
        access_gate_ok: bool,
        blueprint_gate_ok: bool,
        simulation_gate_ok: bool,
        idempotency_gate_ok: bool,
        lease_gate_ok: bool,
        tool_dispatch_allowed: bool,
        simulation_dispatch_allowed: bool,
        execution_allowed: bool,
        guard_failures: Vec<String>,
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
        outcome_entries_evaluated: u16,
        classification_coverage_pct: u8,
        unresolved_outcomes: u16,
        gate_u1_classification_complete: bool,
        gate_u2_unresolved_outcomes_zero: bool,
        gate_u3_optional_budget_enforced: bool,
        optional_invocations_requested: u16,
        optional_invocations_budget: u16,
        optional_invocations_skipped_budget: u16,
        optional_latency_budget_ms: u32,
        optional_latency_estimated_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            capability_id: OsCapabilityId::OsPolicyEvaluate,
            reason_code,
            session_gate_ok,
            understanding_gate_ok,
            confirmation_gate_ok,
            prompt_policy_gate_ok,
            policy_gate_ok,
            tenant_gate_ok,
            gov_gate_ok,
            quota_gate_ok,
            work_gate_ok,
            capreq_gate_ok,
            governance_contradiction_detected,
            governance_decision_trace,
            access_gate_ok,
            blueprint_gate_ok,
            simulation_gate_ok,
            idempotency_gate_ok,
            lease_gate_ok,
            tool_dispatch_allowed,
            simulation_dispatch_allowed,
            execution_allowed,
            guard_failures,
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
            outcome_entries_evaluated,
            classification_coverage_pct,
            unresolved_outcomes,
            gate_u1_classification_complete,
            gate_u2_unresolved_outcomes_zero,
            gate_u3_optional_budget_enforced,
            optional_invocations_requested,
            optional_invocations_budget,
            optional_invocations_skipped_budget,
            optional_latency_budget_ms,
            optional_latency_estimated_ms,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for OsPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        if self.capability_id != OsCapabilityId::OsPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.capability_id",
                reason: "must be OS_POLICY_EVALUATE",
            });
        }

        if self.guard_failures.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.guard_failures",
                reason: "must contain <= 16 guard failures",
            });
        }
        for failure in &self.guard_failures {
            validate_token_ascii("os_policy_evaluate_ok.guard_failures", failure, 64)?;
        }

        if !self.no_engine_to_engine_calls {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.no_engine_to_engine_calls",
                reason: "must be true",
            });
        }
        if !self.no_simulation_no_execution {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.no_simulation_no_execution",
                reason: "must be true",
            });
        }
        if !self.one_turn_one_move {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.one_turn_one_move",
                reason: "must be true",
            });
        }

        if self.execution_allowed
            && (!self.session_gate_ok
                || !self.understanding_gate_ok
                || !self.confirmation_gate_ok
                || !self.prompt_policy_gate_ok
                || !self.policy_gate_ok
                || !self.tenant_gate_ok
                || !self.gov_gate_ok
                || !self.quota_gate_ok
                || !self.work_gate_ok
                || !self.capreq_gate_ok
                || self.governance_contradiction_detected
                || !self.access_gate_ok
                || !self.blueprint_gate_ok
                || !self.simulation_gate_ok
                || !self.idempotency_gate_ok
                || !self.lease_gate_ok)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.execution_allowed",
                reason: "cannot be true when a required gate is false",
            });
        }
        if self.simulation_dispatch_allowed && !self.execution_allowed {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.simulation_dispatch_allowed",
                reason: "requires execution_allowed=true",
            });
        }
        if self.tool_dispatch_allowed && self.simulation_dispatch_allowed {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.tool_dispatch_allowed",
                reason: "tool and simulation dispatch cannot both be allowed in one turn",
            });
        }
        if self.governance_decision_trace.is_empty() || self.governance_decision_trace.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.governance_decision_trace",
                reason: "must contain 1..=16 entries",
            });
        }
        for item in &self.governance_decision_trace {
            validate_token_ascii("os_policy_evaluate_ok.governance_decision_trace", item, 64)?;
        }
        if self.classification_coverage_pct > 100 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.classification_coverage_pct",
                reason: "must be within 0..=100",
            });
        }
        if self.unresolved_outcomes > self.outcome_entries_evaluated {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.unresolved_outcomes",
                reason: "cannot exceed outcome_entries_evaluated",
            });
        }
        if self.gate_u1_classification_complete && self.classification_coverage_pct != 100 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.gate_u1_classification_complete",
                reason: "requires classification_coverage_pct=100",
            });
        }
        if self.gate_u2_unresolved_outcomes_zero && self.unresolved_outcomes != 0 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.gate_u2_unresolved_outcomes_zero",
                reason: "requires unresolved_outcomes=0",
            });
        }
        if self.execution_allowed
            && (!self.gate_u1_classification_complete || !self.gate_u2_unresolved_outcomes_zero)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.execution_allowed",
                reason: "cannot be true when outcome utilization gates fail",
            });
        }
        if self.optional_invocations_requested > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_invocations_requested",
                reason: "must be within 0..=64",
            });
        }
        if self.optional_invocations_budget > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_invocations_budget",
                reason: "must be within 0..=64",
            });
        }
        if self.optional_invocations_skipped_budget > self.optional_invocations_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_invocations_skipped_budget",
                reason: "cannot exceed optional_invocations_requested",
            });
        }
        if self.optional_invocations_skipped_budget
            != self
                .optional_invocations_requested
                .saturating_sub(self.optional_invocations_budget)
        {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_invocations_skipped_budget",
                reason: "must equal requested.saturating_sub(optional_invocations_budget)",
            });
        }
        if self.optional_latency_budget_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_latency_budget_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_latency_estimated_ms",
                reason: "must be <= 60000",
            });
        }
        if self.optional_latency_estimated_ms > self.optional_latency_budget_ms {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.optional_latency_estimated_ms",
                reason: "must be <= optional_latency_budget_ms",
            });
        }
        if self.execution_allowed && !self.gate_u3_optional_budget_enforced {
            return Err(ContractViolation::InvalidValue {
                field: "os_policy_evaluate_ok.execution_allowed",
                reason: "cannot be true when GATE-U3 optional budget enforcement fails",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: OsRequestEnvelope,
    pub policy_evaluate: OsPolicyEvaluateOk,
    pub chat_requested: bool,
    pub clarify_required: bool,
    pub clarify_owner_engine_id: Option<String>,
    pub confirm_required: bool,
    pub explain_requested: bool,
    pub wait_required: bool,
    pub tool_requested: bool,
    pub simulation_requested: bool,
}

impl OsDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: OsRequestEnvelope,
        policy_evaluate: OsPolicyEvaluateOk,
        chat_requested: bool,
        clarify_required: bool,
        clarify_owner_engine_id: Option<String>,
        confirm_required: bool,
        explain_requested: bool,
        wait_required: bool,
        tool_requested: bool,
        simulation_requested: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            envelope,
            policy_evaluate,
            chat_requested,
            clarify_required,
            clarify_owner_engine_id,
            confirm_required,
            explain_requested,
            wait_required,
            tool_requested,
            simulation_requested,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for OsDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_request.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.policy_evaluate.validate()?;

        if self.tool_requested && self.simulation_requested {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_request.tool_requested",
                reason: "tool_requested and simulation_requested cannot both be true",
            });
        }
        if let Some(owner) = &self.clarify_owner_engine_id {
            validate_token_ascii("os_decision_compute_request.clarify_owner_engine_id", owner, 64)?;
        }
        if self.clarify_required {
            if self.clarify_owner_engine_id.as_deref() != Some(OS_CLARIFY_OWNER_ENGINE_ID) {
                return Err(ContractViolation::InvalidValue {
                    field: "os_decision_compute_request.clarify_owner_engine_id",
                    reason: "must be PH1.NLP when clarify_required=true",
                });
            }
        } else if self.clarify_owner_engine_id.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_request.clarify_owner_engine_id",
                reason: "must be omitted when clarify_required=false",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: OsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub next_move: OsNextMove,
    pub fail_closed: bool,
    pub dispatch_allowed: bool,
    pub execution_allowed: bool,
    pub no_engine_to_engine_calls: bool,
    pub no_simulation_no_execution: bool,
    pub one_turn_one_move: bool,
}

impl OsDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        next_move: OsNextMove,
        fail_closed: bool,
        dispatch_allowed: bool,
        execution_allowed: bool,
        no_engine_to_engine_calls: bool,
        no_simulation_no_execution: bool,
        one_turn_one_move: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            capability_id: OsCapabilityId::OsDecisionCompute,
            reason_code,
            next_move,
            fail_closed,
            dispatch_allowed,
            execution_allowed,
            no_engine_to_engine_calls,
            no_simulation_no_execution,
            one_turn_one_move,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for OsDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_ok.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        if self.capability_id != OsCapabilityId::OsDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_ok.capability_id",
                reason: "must be OS_DECISION_COMPUTE",
            });
        }

        if !self.no_engine_to_engine_calls {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_ok.no_engine_to_engine_calls",
                reason: "must be true",
            });
        }
        if !self.no_simulation_no_execution {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_ok.no_simulation_no_execution",
                reason: "must be true",
            });
        }
        if !self.one_turn_one_move {
            return Err(ContractViolation::InvalidValue {
                field: "os_decision_compute_ok.one_turn_one_move",
                reason: "must be true",
            });
        }

        match self.next_move {
            OsNextMove::DispatchTool => {
                if !self.dispatch_allowed {
                    return Err(ContractViolation::InvalidValue {
                        field: "os_decision_compute_ok.dispatch_allowed",
                        reason: "DispatchTool requires dispatch_allowed=true",
                    });
                }
            }
            OsNextMove::DispatchSimulation => {
                if !self.dispatch_allowed || !self.execution_allowed {
                    return Err(ContractViolation::InvalidValue {
                        field: "os_decision_compute_ok.execution_allowed",
                        reason:
                            "DispatchSimulation requires dispatch_allowed=true and execution_allowed=true",
                    });
                }
            }
            _ => {
                if self.fail_closed && (self.next_move == OsNextMove::Respond) {
                    return Err(ContractViolation::InvalidValue {
                        field: "os_decision_compute_ok.fail_closed",
                        reason: "fail_closed=true cannot emit Respond",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OsRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: OsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl OsRefuse {
    pub fn v1(
        capability_id: OsCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1OS_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for OsRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1OS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "os_refuse.schema_version",
                reason: "must match PH1OS_CONTRACT_VERSION",
            });
        }
        validate_text_ascii("os_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1OsRequest {
    OsPolicyEvaluate(OsPolicyEvaluateRequest),
    OsDecisionCompute(OsDecisionComputeRequest),
}

impl Validate for Ph1OsRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1OsRequest::OsPolicyEvaluate(req) => req.validate(),
            Ph1OsRequest::OsDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1OsResponse {
    OsPolicyEvaluateOk(OsPolicyEvaluateOk),
    OsDecisionComputeOk(OsDecisionComputeOk),
    Refuse(OsRefuse),
}

impl Validate for Ph1OsResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1OsResponse::OsPolicyEvaluateOk(out) => out.validate(),
            Ph1OsResponse::OsDecisionComputeOk(out) => out.validate(),
            Ph1OsResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token_ascii(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

fn validate_text_ascii(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> OsRequestEnvelope {
        OsRequestEnvelope::v1_with_outcome_budget(CorrelationId(7801), TurnId(8801), 8, 8, 8)
            .unwrap()
    }

    fn valid_outcome(
        action_class: OsOutcomeActionClass,
        consumed_by: &str,
    ) -> OsOutcomeUtilizationEntry {
        OsOutcomeUtilizationEntry::v1(
            "PH1.NLP".to_string(),
            "INTENT_DRAFT".to_string(),
            CorrelationId(7801),
            TurnId(8801),
            action_class,
            consumed_by.to_string(),
            4,
            true,
            ReasonCodeId(11),
        )
        .unwrap()
    }

    #[test]
    fn at_os_contract_01_tool_and_simulation_requested_together_fail() {
        let req = OsPolicyEvaluateRequest::v1(
            envelope(),
            true,
            true,
            true,
            false,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_os_contract_02_execution_allowed_requires_all_gates() {
        let out = OsPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            true,
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
            false,
            true,
            true,
            vec![],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_os_contract_03_dispatch_simulation_requires_execution_allowed() {
        let out = OsDecisionComputeOk::v1(
            ReasonCodeId(2),
            OsNextMove::DispatchSimulation,
            false,
            true,
            false,
            true,
            true,
            true,
        );
        assert!(out.is_err());

        let valid = OsDecisionComputeOk::v1(
            ReasonCodeId(3),
            OsNextMove::DispatchSimulation,
            false,
            true,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        assert_eq!(valid.next_move, OsNextMove::DispatchSimulation);
    }

    #[test]
    fn at_os_contract_03a_clarify_owner_must_be_ph1_nlp() {
        let policy = OsPolicyEvaluateOk::v1(
            ReasonCodeId(4),
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
        let env = OsRequestEnvelope::v1(CorrelationId(7802), TurnId(8802), 8, 8).unwrap();

        let bad = OsDecisionComputeRequest::v1(
            env.clone(),
            policy.clone(),
            false,
            true,
            Some("PH1.DIAG".to_string()),
            false,
            false,
            false,
            false,
            false,
        );
        assert!(bad.is_err());

        let bad_when_not_required = OsDecisionComputeRequest::v1(
            env.clone(),
            policy.clone(),
            false,
            false,
            Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string()),
            false,
            false,
            false,
            false,
            false,
        );
        assert!(bad_when_not_required.is_err());

        let ok = OsDecisionComputeRequest::v1(
            env,
            policy,
            false,
            true,
            Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string()),
            false,
            false,
            false,
            false,
            false,
        );
        assert!(ok.is_ok());
    }

    #[test]
    fn at_os_contract_04_gate_u2_requires_consumed_by_for_act_now() {
        let entry = OsOutcomeUtilizationEntry::v1(
            "PH1.X".to_string(),
            "X_DISPATCH".to_string(),
            CorrelationId(7801),
            TurnId(8801),
            OsOutcomeActionClass::ActNow,
            "NONE".to_string(),
            3,
            true,
            ReasonCodeId(1),
        );
        assert!(entry.is_err());
    }

    #[test]
    fn at_os_contract_05_outcome_entry_must_match_envelope_correlation_turn() {
        let mut entry = valid_outcome(OsOutcomeActionClass::AuditOnly, "NONE");
        entry.turn_id = TurnId(9900);
        let req = OsPolicyEvaluateRequest::v1_with_outcomes(
            envelope(),
            true,
            true,
            true,
            false,
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
            false,
            true,
            true,
            true,
            vec![entry],
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_os_contract_06_execution_not_allowed_when_u1_u2_fail() {
        let out = OsPolicyEvaluateOk::v1_with_outcome_gates(
            ReasonCodeId(4),
            true,
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
            vec![
                "POLICY:ALLOW".to_string(),
                "TENANT:ALLOW".to_string(),
                "GOV:ALLOW".to_string(),
                "QUOTA:ALLOW".to_string(),
                "WORK:ALLOW".to_string(),
                "CAPREQ:ALLOW".to_string(),
            ],
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
            4,
            75,
            1,
            false,
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_os_contract_07_governance_trace_required() {
        let out = OsPolicyEvaluateOk::v1_with_outcome_gates(
            ReasonCodeId(5),
            true,
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
            vec![],
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
            0,
            100,
            0,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_os_contract_08_optional_latency_budget_must_bound_estimate() {
        let req = OsPolicyEvaluateRequest::v1_with_governance_outcomes_and_optional_budget(
            envelope(),
            true,
            true,
            true,
            false,
            false,
            false,
            true,
            false,
            false,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            true,
            true,
            true,
            false,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
            4,
            2,
            2,
            20,
            40,
            vec![],
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_os_contract_09_execution_not_allowed_when_u3_fails() {
        let out = OsPolicyEvaluateOk::v1_with_outcome_gates_and_optional_budget(
            ReasonCodeId(6),
            true,
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
            vec![
                "POLICY:ALLOW".to_string(),
                "TENANT:ALLOW".to_string(),
                "GOV:ALLOW".to_string(),
                "QUOTA:ALLOW".to_string(),
                "WORK:ALLOW".to_string(),
                "CAPREQ:ALLOW".to_string(),
            ],
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
            3,
            100,
            0,
            true,
            true,
            false,
            3,
            2,
            1,
            20,
            20,
        );
        assert!(out.is_err());
    }
}
