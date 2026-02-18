#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1os::{
    OsCapabilityId, OsDecisionComputeOk, OsDecisionComputeRequest, OsGateDecision, OsNextMove,
    OsOutcomeActionClass, OsPolicyEvaluateOk, OsPolicyEvaluateRequest, OsRefuse, OsRequestEnvelope,
    Ph1OsRequest, Ph1OsResponse, OS_CLARIFY_OWNER_ENGINE_ID,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.OS reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_OS_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x4F53_0001);
    pub const PH1_OS_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x4F53_0002);

    pub const OS_FAIL_SESSION_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0010);
    pub const OS_FAIL_UNDERSTANDING_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0011);
    pub const OS_FAIL_CONFIRMATION_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0012);
    pub const OS_FAIL_PROMPT_POLICY_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0022);
    pub const OS_FAIL_ACCESS_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0013);
    pub const OS_FAIL_BLUEPRINT_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0014);
    pub const OS_FAIL_SIMULATION_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0015);
    pub const OS_FAIL_IDEMPOTENCY_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0016);
    pub const OS_FAIL_LEASE_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0017);
    pub const OS_FAIL_ONE_TURN_ONE_MOVE: ReasonCodeId = ReasonCodeId(0x4F53_0018);
    pub const OS_FAIL_OUTCOME_CLASSIFICATION_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0019);
    pub const OS_FAIL_OUTCOME_UNRESOLVED_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001A);
    pub const OS_FAIL_OPTIONAL_BUDGET_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0023);
    pub const OS_FAIL_CLARIFY_OWNER_PRECEDENCE: ReasonCodeId = ReasonCodeId(0x4F53_0024);
    pub const OS_FAIL_POLICY_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001B);
    pub const OS_FAIL_TENANT_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001C);
    pub const OS_FAIL_GOV_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001D);
    pub const OS_FAIL_QUOTA_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001E);
    pub const OS_FAIL_WORK_GATE: ReasonCodeId = ReasonCodeId(0x4F53_001F);
    pub const OS_FAIL_CAPREQ_GATE: ReasonCodeId = ReasonCodeId(0x4F53_0020);
    pub const OS_FAIL_GOVERNANCE_CONTRADICTION: ReasonCodeId = ReasonCodeId(0x4F53_0021);

    pub const PH1_OS_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4F53_00F1);
    pub const PH1_OS_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4F53_00F2);
    pub const PH1_OS_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4F53_00F3);
    pub const PH1_OS_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4F53_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1OsConfig {
    pub max_guard_failures: u8,
    pub max_diagnostics: u8,
    pub max_outcome_entries: u16,
}

impl Ph1OsConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_guard_failures: 8,
            max_diagnostics: 8,
            max_outcome_entries: 128,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1OsRuntime {
    config: Ph1OsConfig,
}

impl Ph1OsRuntime {
    pub fn new(config: Ph1OsConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1OsRequest) -> Ph1OsResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_OS_INPUT_SCHEMA_INVALID,
                "os request failed contract validation",
            );
        }

        match req {
            Ph1OsRequest::OsPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1OsRequest::OsDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &OsPolicyEvaluateRequest) -> Ph1OsResponse {
        if envelope_budget_exceeded(
            &req.envelope,
            self.config.max_guard_failures,
            self.config.max_diagnostics,
            self.config.max_outcome_entries,
        ) {
            return self.refuse(
                OsCapabilityId::OsPolicyEvaluate,
                reason_codes::PH1_OS_BUDGET_EXCEEDED,
                "request envelope exceeds runtime budgets",
            );
        }

        let session_gate_ok = req.session_active;
        let understanding_gate_ok = req.transcript_ok && req.nlp_confidence_high;
        let confirmation_gate_ok = !req.requires_confirmation || req.confirmation_received;
        let prompt_policy_gate_ok = !req.prompt_policy_required || req.prompt_policy_gate_ok;
        let policy_gate_ok = req.policy_gate_decision.is_allow();
        let tenant_gate_ok = req.tenant_gate_decision.is_allow();
        let gov_gate_ok = req.gov_gate_decision.is_allow();
        let quota_gate_ok = req.quota_gate_decision.is_allow();
        let work_gate_ok = req.work_gate_decision.is_allow();
        let capreq_gate_ok = req.capreq_gate_decision.is_allow();
        let governance_conflict_detected = governance_contradiction_detected(&[
            req.policy_gate_decision,
            req.tenant_gate_decision,
            req.gov_gate_decision,
            req.quota_gate_decision,
            req.work_gate_decision,
            req.capreq_gate_decision,
        ]);
        let governance_decision_trace = governance_decision_trace(&[
            ("POLICY", req.policy_gate_decision),
            ("TENANT", req.tenant_gate_decision),
            ("GOV", req.gov_gate_decision),
            ("QUOTA", req.quota_gate_decision),
            ("WORK", req.work_gate_decision),
            ("CAPREQ", req.capreq_gate_decision),
        ]);
        let access_gate_ok = req.access_allowed;
        let blueprint_gate_ok = !req.simulation_requested || req.blueprint_active;
        let simulation_gate_ok = !req.simulation_requested || req.simulation_active;
        let idempotency_gate_ok = !req.idempotency_required || req.idempotency_key_present;
        let lease_gate_ok = !req.lease_required || req.lease_valid;
        let outcome_entries_evaluated = req.outcome_utilization_entries.len() as u16;
        let classification_coverage_pct = 100;
        let unresolved_outcomes = req
            .outcome_utilization_entries
            .iter()
            .filter(|entry| {
                matches!(
                    entry.action_class,
                    OsOutcomeActionClass::ActNow | OsOutcomeActionClass::QueueLearn
                ) && entry.consumed_by == "NONE"
            })
            .count() as u16;
        let gate_u1_classification_complete = classification_coverage_pct == 100;
        let gate_u2_unresolved_outcomes_zero = unresolved_outcomes == 0;
        let gate_u3_optional_budget_enforced = req.optional_budget_enforced
            && req.optional_invocations_skipped_budget
                == req
                    .optional_invocations_requested
                    .saturating_sub(req.optional_invocations_budget)
            && req.optional_latency_estimated_ms <= req.optional_latency_budget_ms;

        let execution_allowed = session_gate_ok
            && understanding_gate_ok
            && confirmation_gate_ok
            && prompt_policy_gate_ok
            && policy_gate_ok
            && tenant_gate_ok
            && gov_gate_ok
            && quota_gate_ok
            && work_gate_ok
            && capreq_gate_ok
            && !governance_conflict_detected
            && access_gate_ok
            && blueprint_gate_ok
            && simulation_gate_ok
            && idempotency_gate_ok
            && lease_gate_ok
            && gate_u1_classification_complete
            && gate_u2_unresolved_outcomes_zero
            && gate_u3_optional_budget_enforced;

        let tool_dispatch_allowed = req.tool_requested
            && !req.simulation_requested
            && session_gate_ok
            && understanding_gate_ok
            && confirmation_gate_ok
            && prompt_policy_gate_ok
            && policy_gate_ok
            && tenant_gate_ok
            && gov_gate_ok
            && quota_gate_ok
            && work_gate_ok
            && capreq_gate_ok
            && !governance_conflict_detected
            && access_gate_ok;
        let simulation_dispatch_allowed = req.simulation_requested && execution_allowed;

        let mut guard_failures = Vec::new();
        if !session_gate_ok {
            guard_failures.push("SESSION_GATE_FAILED".to_string());
        }
        if !understanding_gate_ok {
            guard_failures.push("UNDERSTANDING_GATE_FAILED".to_string());
        }
        if !confirmation_gate_ok {
            guard_failures.push("CONFIRMATION_GATE_FAILED".to_string());
        }
        if !prompt_policy_gate_ok {
            guard_failures.push("PROMPT_POLICY_GATE_FAILED".to_string());
        }
        if !policy_gate_ok {
            guard_failures.push("POLICY_GATE_FAILED".to_string());
        }
        if !tenant_gate_ok {
            guard_failures.push("TENANT_GATE_FAILED".to_string());
        }
        if !gov_gate_ok {
            guard_failures.push("GOV_GATE_FAILED".to_string());
        }
        if !quota_gate_ok {
            guard_failures.push("QUOTA_GATE_FAILED".to_string());
        }
        if !work_gate_ok {
            guard_failures.push("WORK_GATE_FAILED".to_string());
        }
        if !capreq_gate_ok {
            guard_failures.push("CAPREQ_GATE_FAILED".to_string());
        }
        if governance_conflict_detected {
            guard_failures.push("GOVERNANCE_CONTRADICTION".to_string());
        }
        if !access_gate_ok {
            guard_failures.push("ACCESS_GATE_FAILED".to_string());
        }
        if !blueprint_gate_ok {
            guard_failures.push("BLUEPRINT_GATE_FAILED".to_string());
        }
        if !simulation_gate_ok {
            guard_failures.push("SIMULATION_GATE_FAILED".to_string());
        }
        if !idempotency_gate_ok {
            guard_failures.push("IDEMPOTENCY_GATE_FAILED".to_string());
        }
        if !lease_gate_ok {
            guard_failures.push("LEASE_GATE_FAILED".to_string());
        }
        if !gate_u1_classification_complete {
            guard_failures.push("OUTCOME_CLASSIFICATION_GATE_FAILED".to_string());
        }
        if !gate_u2_unresolved_outcomes_zero {
            guard_failures.push("OUTCOME_UNRESOLVED_GATE_FAILED".to_string());
        }
        if !gate_u3_optional_budget_enforced {
            guard_failures.push("OPTIONAL_BUDGET_GATE_FAILED".to_string());
        }

        if guard_failures.len() > req.envelope.max_guard_failures as usize {
            return self.refuse(
                OsCapabilityId::OsPolicyEvaluate,
                reason_codes::PH1_OS_BUDGET_EXCEEDED,
                "guard failure budget exceeded",
            );
        }

        let reason_code = if execution_allowed {
            reason_codes::PH1_OS_OK_POLICY_EVALUATE
        } else {
            first_policy_failure_reason(
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
                governance_conflict_detected,
                access_gate_ok,
                blueprint_gate_ok,
                simulation_gate_ok,
                idempotency_gate_ok,
                lease_gate_ok,
                gate_u1_classification_complete,
                gate_u2_unresolved_outcomes_zero,
                gate_u3_optional_budget_enforced,
            )
        };

        match OsPolicyEvaluateOk::v1_with_outcome_gates_and_optional_budget(
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
            governance_conflict_detected,
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
            true,
            true,
            true,
            outcome_entries_evaluated,
            classification_coverage_pct,
            unresolved_outcomes,
            gate_u1_classification_complete,
            gate_u2_unresolved_outcomes_zero,
            gate_u3_optional_budget_enforced,
            req.optional_invocations_requested,
            req.optional_invocations_budget,
            req.optional_invocations_skipped_budget,
            req.optional_latency_budget_ms,
            req.optional_latency_estimated_ms,
        ) {
            Ok(ok) => Ph1OsResponse::OsPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                OsCapabilityId::OsPolicyEvaluate,
                reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR,
                "failed to construct os policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &OsDecisionComputeRequest) -> Ph1OsResponse {
        if envelope_budget_exceeded(
            &req.envelope,
            self.config.max_guard_failures,
            self.config.max_diagnostics,
            self.config.max_outcome_entries,
        ) {
            return self.refuse(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_BUDGET_EXCEEDED,
                "request envelope exceeds runtime budgets",
            );
        }

        let requested_move_count = usize::from(req.chat_requested)
            + usize::from(req.clarify_required)
            + usize::from(req.confirm_required)
            + usize::from(req.explain_requested)
            + usize::from(req.wait_required)
            + usize::from(req.tool_requested)
            + usize::from(req.simulation_requested);

        if req.clarify_required {
            if req.clarify_owner_engine_id.as_deref() != Some(OS_CLARIFY_OWNER_ENGINE_ID) {
                return self.decision(
                    reason_codes::OS_FAIL_CLARIFY_OWNER_PRECEDENCE,
                    OsNextMove::Refuse,
                    true,
                    false,
                    false,
                );
            }
        } else if req.clarify_owner_engine_id.is_some() {
            return self.decision(
                reason_codes::OS_FAIL_CLARIFY_OWNER_PRECEDENCE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }

        if requested_move_count > 1 {
            return self.decision(
                reason_codes::OS_FAIL_ONE_TURN_ONE_MOVE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }

        let policy = &req.policy_evaluate;
        if !policy.session_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_SESSION_GATE,
                OsNextMove::Wait,
                true,
                false,
                false,
            );
        }
        if !policy.understanding_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_UNDERSTANDING_GATE,
                OsNextMove::Clarify,
                true,
                false,
                false,
            );
        }
        if !policy.confirmation_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_CONFIRMATION_GATE,
                OsNextMove::Confirm,
                true,
                false,
                false,
            );
        }
        if !policy.prompt_policy_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_PROMPT_POLICY_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if policy.governance_contradiction_detected {
            return self.decision(
                reason_codes::OS_FAIL_GOVERNANCE_CONTRADICTION,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.policy_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_POLICY_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.tenant_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_TENANT_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.gov_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_GOV_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.quota_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_QUOTA_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.work_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_WORK_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.capreq_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_CAPREQ_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.access_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_ACCESS_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.blueprint_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_BLUEPRINT_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.simulation_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_SIMULATION_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.idempotency_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_IDEMPOTENCY_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.lease_gate_ok {
            return self.decision(
                reason_codes::OS_FAIL_LEASE_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.gate_u1_classification_complete {
            return self.decision(
                reason_codes::OS_FAIL_OUTCOME_CLASSIFICATION_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.gate_u2_unresolved_outcomes_zero {
            return self.decision(
                reason_codes::OS_FAIL_OUTCOME_UNRESOLVED_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }
        if !policy.gate_u3_optional_budget_enforced {
            return self.decision(
                reason_codes::OS_FAIL_OPTIONAL_BUDGET_GATE,
                OsNextMove::Refuse,
                true,
                false,
                false,
            );
        }

        if req.simulation_requested {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::DispatchSimulation,
                false,
                true,
                true,
            );
        }
        if req.tool_requested {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::DispatchTool,
                false,
                true,
                false,
            );
        }
        if req.confirm_required {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::Confirm,
                false,
                false,
                false,
            );
        }
        if req.clarify_required {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::Clarify,
                false,
                false,
                false,
            );
        }
        if req.wait_required {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::Wait,
                false,
                false,
                false,
            );
        }
        if req.explain_requested {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::Explain,
                false,
                false,
                false,
            );
        }
        if req.chat_requested || requested_move_count == 0 {
            return self.decision(
                reason_codes::PH1_OS_OK_DECISION_COMPUTE,
                OsNextMove::Respond,
                false,
                false,
                false,
            );
        }

        self.refuse(
            OsCapabilityId::OsDecisionCompute,
            reason_codes::PH1_OS_UPSTREAM_INPUT_MISSING,
            "unable to derive deterministic next move",
        )
    }

    fn decision(
        &self,
        reason_code: ReasonCodeId,
        next_move: OsNextMove,
        fail_closed: bool,
        dispatch_allowed: bool,
        execution_allowed: bool,
    ) -> Ph1OsResponse {
        match OsDecisionComputeOk::v1(
            reason_code,
            next_move,
            fail_closed,
            dispatch_allowed,
            execution_allowed,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1OsResponse::OsDecisionComputeOk(ok),
            Err(_) => self.refuse(
                OsCapabilityId::OsDecisionCompute,
                reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR,
                "failed to construct os decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: OsCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1OsResponse {
        let out = OsRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("OsRefuse::v1 must construct for static messages");
        Ph1OsResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1OsRequest) -> OsCapabilityId {
    match req {
        Ph1OsRequest::OsPolicyEvaluate(_) => OsCapabilityId::OsPolicyEvaluate,
        Ph1OsRequest::OsDecisionCompute(_) => OsCapabilityId::OsDecisionCompute,
    }
}

fn envelope_budget_exceeded(
    envelope: &OsRequestEnvelope,
    max_guard_failures: u8,
    max_diagnostics: u8,
    max_outcome_entries: u16,
) -> bool {
    envelope.max_guard_failures > max_guard_failures
        || envelope.max_diagnostics > max_diagnostics
        || envelope.max_outcome_entries > max_outcome_entries
}

#[allow(clippy::too_many_arguments)]
fn first_policy_failure_reason(
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
    access_gate_ok: bool,
    blueprint_gate_ok: bool,
    simulation_gate_ok: bool,
    idempotency_gate_ok: bool,
    lease_gate_ok: bool,
    gate_u1_classification_complete: bool,
    gate_u2_unresolved_outcomes_zero: bool,
    gate_u3_optional_budget_enforced: bool,
) -> ReasonCodeId {
    if !session_gate_ok {
        return reason_codes::OS_FAIL_SESSION_GATE;
    }
    if !understanding_gate_ok {
        return reason_codes::OS_FAIL_UNDERSTANDING_GATE;
    }
    if !confirmation_gate_ok {
        return reason_codes::OS_FAIL_CONFIRMATION_GATE;
    }
    if !prompt_policy_gate_ok {
        return reason_codes::OS_FAIL_PROMPT_POLICY_GATE;
    }
    if governance_contradiction_detected {
        return reason_codes::OS_FAIL_GOVERNANCE_CONTRADICTION;
    }
    if !policy_gate_ok {
        return reason_codes::OS_FAIL_POLICY_GATE;
    }
    if !tenant_gate_ok {
        return reason_codes::OS_FAIL_TENANT_GATE;
    }
    if !gov_gate_ok {
        return reason_codes::OS_FAIL_GOV_GATE;
    }
    if !quota_gate_ok {
        return reason_codes::OS_FAIL_QUOTA_GATE;
    }
    if !work_gate_ok {
        return reason_codes::OS_FAIL_WORK_GATE;
    }
    if !capreq_gate_ok {
        return reason_codes::OS_FAIL_CAPREQ_GATE;
    }
    if !access_gate_ok {
        return reason_codes::OS_FAIL_ACCESS_GATE;
    }
    if !blueprint_gate_ok {
        return reason_codes::OS_FAIL_BLUEPRINT_GATE;
    }
    if !simulation_gate_ok {
        return reason_codes::OS_FAIL_SIMULATION_GATE;
    }
    if !idempotency_gate_ok {
        return reason_codes::OS_FAIL_IDEMPOTENCY_GATE;
    }
    if !lease_gate_ok {
        return reason_codes::OS_FAIL_LEASE_GATE;
    }
    if !gate_u1_classification_complete {
        return reason_codes::OS_FAIL_OUTCOME_CLASSIFICATION_GATE;
    }
    if !gate_u2_unresolved_outcomes_zero {
        return reason_codes::OS_FAIL_OUTCOME_UNRESOLVED_GATE;
    }
    if !gate_u3_optional_budget_enforced {
        return reason_codes::OS_FAIL_OPTIONAL_BUDGET_GATE;
    }
    reason_codes::PH1_OS_INTERNAL_PIPELINE_ERROR
}

fn governance_contradiction_detected(decisions: &[OsGateDecision]) -> bool {
    let allows = decisions.iter().any(|d| d.is_allow());
    let non_allows = decisions.iter().any(|d| !d.is_allow());
    allows && non_allows
}

fn governance_decision_trace(ordered: &[(&str, OsGateDecision)]) -> Vec<String> {
    ordered
        .iter()
        .map(|(name, decision)| format!("{name}:{}", decision.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn envelope() -> OsRequestEnvelope {
        OsRequestEnvelope::v1(CorrelationId(7701), TurnId(8701), 8, 8).unwrap()
    }

    fn runtime() -> Ph1OsRuntime {
        Ph1OsRuntime::new(Ph1OsConfig::mvp_v1())
    }

    fn utilization_entry() -> selene_kernel_contracts::ph1os::OsOutcomeUtilizationEntry {
        selene_kernel_contracts::ph1os::OsOutcomeUtilizationEntry::v1(
            "PH1.NLP".to_string(),
            "INTENT_DRAFT".to_string(),
            CorrelationId(7701),
            TurnId(8701),
            OsOutcomeActionClass::ActNow,
            "PH1.X".to_string(),
            6,
            true,
            ReasonCodeId(33),
        )
        .unwrap()
    }

    #[test]
    fn at_os_01_no_simulation_no_execution() {
        let req = OsPolicyEvaluateRequest::v1(
            envelope(),
            true,
            true,
            true,
            false,
            false,
            false,
            true,
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
            true,
        )
        .unwrap();

        let out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(req));
        let Ph1OsResponse::OsPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert!(!ok.execution_allowed);
        assert!(!ok.simulation_dispatch_allowed);
        assert_eq!(ok.reason_code, reason_codes::OS_FAIL_SIMULATION_GATE);
    }

    #[test]
    fn at_os_02_one_turn_one_move_enforced() {
        let policy_req = OsPolicyEvaluateRequest::v1(
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
            true,
            true,
            true,
            true,
        )
        .unwrap();
        let policy_out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(policy_req));
        let Ph1OsResponse::OsPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };

        let decision_req = OsDecisionComputeRequest::v1(
            envelope(),
            policy_ok,
            true,
            true,
            Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string()),
            false,
            false,
            false,
            false,
            false,
        )
        .unwrap();
        let out = runtime().run(&Ph1OsRequest::OsDecisionCompute(decision_req));
        let Ph1OsResponse::OsDecisionComputeOk(ok) = out else {
            panic!("expected decision output");
        };
        assert_eq!(ok.next_move, OsNextMove::Refuse);
        assert!(ok.fail_closed);
        assert_eq!(ok.reason_code, reason_codes::OS_FAIL_ONE_TURN_ONE_MOVE);
    }

    #[test]
    fn at_os_03_dispatch_simulation_when_gates_pass() {
        let policy_req = OsPolicyEvaluateRequest::v1(
            envelope(),
            true,
            true,
            true,
            true,
            true,
            false,
            true,
            false,
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
            true,
        )
        .unwrap();
        let policy_out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(policy_req));
        let Ph1OsResponse::OsPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };

        let decision_req = OsDecisionComputeRequest::v1(
            envelope(),
            policy_ok,
            false,
            false,
            None,
            false,
            false,
            false,
            false,
            true,
        )
        .unwrap();
        let out = runtime().run(&Ph1OsRequest::OsDecisionCompute(decision_req));
        let Ph1OsResponse::OsDecisionComputeOk(ok) = out else {
            panic!("expected decision output");
        };
        assert_eq!(ok.next_move, OsNextMove::DispatchSimulation);
        assert!(ok.dispatch_allowed);
        assert!(ok.execution_allowed);
    }

    #[test]
    fn at_os_04_tool_dispatch_without_simulation_path() {
        let policy_req = OsPolicyEvaluateRequest::v1(
            envelope(),
            true,
            true,
            true,
            false,
            false,
            false,
            true,
            true,
            false,
            true,
            false,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        let policy_out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(policy_req));
        let Ph1OsResponse::OsPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };
        assert!(policy_ok.tool_dispatch_allowed);
        assert!(!policy_ok.simulation_dispatch_allowed);

        let decision_req = OsDecisionComputeRequest::v1(
            envelope(),
            policy_ok,
            false,
            false,
            None,
            false,
            false,
            false,
            true,
            false,
        )
        .unwrap();
        let out = runtime().run(&Ph1OsRequest::OsDecisionCompute(decision_req));
        let Ph1OsResponse::OsDecisionComputeOk(ok) = out else {
            panic!("expected decision output");
        };
        assert_eq!(ok.next_move, OsNextMove::DispatchTool);
        assert!(ok.dispatch_allowed);
        assert!(!ok.execution_allowed);
    }

    #[test]
    fn at_os_05_outcome_utilization_gates_reported_on_policy_output() {
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
            true,
            true,
            true,
            true,
            vec![utilization_entry()],
        )
        .unwrap();

        let out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(req));
        let Ph1OsResponse::OsPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert_eq!(ok.outcome_entries_evaluated, 1);
        assert_eq!(ok.classification_coverage_pct, 100);
        assert_eq!(ok.unresolved_outcomes, 0);
        assert!(ok.gate_u1_classification_complete);
        assert!(ok.gate_u2_unresolved_outcomes_zero);
    }

    #[test]
    fn at_os_06_governance_contradiction_fails_closed() {
        let req = OsPolicyEvaluateRequest::v1_with_governance(
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
            OsGateDecision::Deny,
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
        )
        .unwrap();

        let out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(req));
        let Ph1OsResponse::OsPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert!(ok.governance_contradiction_detected);
        assert!(!ok.execution_allowed);
        assert_eq!(
            ok.reason_code,
            reason_codes::OS_FAIL_GOVERNANCE_CONTRADICTION
        );
    }

    #[test]
    fn at_os_07_governance_decision_trace_is_canonical_order() {
        let req = OsPolicyEvaluateRequest::v1_with_governance(
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
            OsGateDecision::Escalate,
            OsGateDecision::Allow,
            OsGateDecision::Allow,
            OsGateDecision::Deny,
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
        )
        .unwrap();

        let out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(req));
        let Ph1OsResponse::OsPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert_eq!(
            ok.governance_decision_trace,
            vec![
                "POLICY:ALLOW",
                "TENANT:ESCALATE",
                "GOV:ALLOW",
                "QUOTA:ALLOW",
                "WORK:DENY",
                "CAPREQ:ALLOW",
            ]
        );
        assert_eq!(
            ok.reason_code,
            reason_codes::OS_FAIL_GOVERNANCE_CONTRADICTION
        );
    }

    #[test]
    fn at_os_08_prompt_policy_gate_fail_closed() {
        let policy_req = OsPolicyEvaluateRequest::v1(
            envelope(),
            true,
            true,
            true,
            false,
            false,
            true,
            false,
            false,
            false,
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
        )
        .unwrap();
        let policy_out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(policy_req));
        let Ph1OsResponse::OsPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };
        assert!(!policy_ok.execution_allowed);
        assert_eq!(
            policy_ok.reason_code,
            reason_codes::OS_FAIL_PROMPT_POLICY_GATE
        );

        let decision_req = OsDecisionComputeRequest::v1(
            envelope(),
            policy_ok,
            false,
            true,
            Some(OS_CLARIFY_OWNER_ENGINE_ID.to_string()),
            false,
            false,
            false,
            false,
            false,
        )
        .unwrap();
        let decision_out = runtime().run(&Ph1OsRequest::OsDecisionCompute(decision_req));
        let Ph1OsResponse::OsDecisionComputeOk(decision_ok) = decision_out else {
            panic!("expected decision output");
        };
        assert_eq!(decision_ok.next_move, OsNextMove::Refuse);
        assert!(decision_ok.fail_closed);
        assert_eq!(
            decision_ok.reason_code,
            reason_codes::OS_FAIL_PROMPT_POLICY_GATE
        );
    }

    #[test]
    fn at_os_09_optional_budget_fields_reported_on_policy_output() {
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
            5,
            3,
            2,
            120,
            60,
            vec![utilization_entry()],
        )
        .unwrap();

        let out = runtime().run(&Ph1OsRequest::OsPolicyEvaluate(req));
        let Ph1OsResponse::OsPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert!(ok.gate_u3_optional_budget_enforced);
        assert_eq!(ok.optional_invocations_requested, 5);
        assert_eq!(ok.optional_invocations_budget, 3);
        assert_eq!(ok.optional_invocations_skipped_budget, 2);
        assert_eq!(ok.optional_latency_budget_ms, 120);
        assert_eq!(ok.optional_latency_estimated_ms, 60);
    }
}
