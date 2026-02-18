#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1cost::{
    CostBudgetPlanBuildOk, CostBudgetPlanBuildRequest, CostCapabilityId, CostRefuse,
    CostResponseLengthHint, CostRouteBudget, CostRouteGuardValidateOk,
    CostRouteGuardValidateRequest, CostRouteGuardrail, CostRouteTierHint, CostRoutingDecision,
    CostValidationStatus, Ph1CostRequest, Ph1CostResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.COST reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_COST_OK_PLAN_BUILD: ReasonCodeId = ReasonCodeId(0x434F_0001);
    pub const PH1_COST_OK_GUARD_VALIDATE: ReasonCodeId = ReasonCodeId(0x434F_0002);

    pub const PH1_COST_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x434F_00F1);
    pub const PH1_COST_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x434F_00F2);
    pub const PH1_COST_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x434F_00F3);
    pub const PH1_COST_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x434F_00F4);
    pub const PH1_COST_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x434F_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1CostConfig {
    pub max_route_budgets: u8,
    pub max_diagnostics: u8,
}

impl Ph1CostConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_route_budgets: 4,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1CostRuntime {
    config: Ph1CostConfig,
}

impl Ph1CostRuntime {
    pub fn new(config: Ph1CostConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1CostRequest) -> Ph1CostResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_COST_INPUT_SCHEMA_INVALID,
                "cost request failed contract validation",
            );
        }

        match req {
            Ph1CostRequest::CostBudgetPlanBuild(r) => self.run_plan_build(r),
            Ph1CostRequest::CostRouteGuardValidate(r) => self.run_guard_validate(r),
        }
    }

    fn run_plan_build(&self, req: &CostBudgetPlanBuildRequest) -> Ph1CostResponse {
        if req.route_budgets.is_empty() {
            return self.refuse(
                CostCapabilityId::CostBudgetPlanBuild,
                reason_codes::PH1_COST_UPSTREAM_INPUT_MISSING,
                "route_budgets is empty",
            );
        }
        if req.route_budgets.len() > self.config.max_route_budgets as usize {
            return self.refuse(
                CostCapabilityId::CostBudgetPlanBuild,
                reason_codes::PH1_COST_BUDGET_EXCEEDED,
                "route_budgets exceeds runtime budget",
            );
        }

        let guardrails = match build_guardrails(&req.route_budgets) {
            Ok(g) => g,
            Err(_) => {
                return self.refuse(
                    CostCapabilityId::CostBudgetPlanBuild,
                    reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                    "failed to build route guardrails",
                )
            }
        };

        match CostBudgetPlanBuildOk::v1(
            reason_codes::PH1_COST_OK_PLAN_BUILD,
            req.budget_scope.clone(),
            guardrails,
            true,
        ) {
            Ok(ok) => Ph1CostResponse::CostBudgetPlanBuildOk(ok),
            Err(_) => self.refuse(
                CostCapabilityId::CostBudgetPlanBuild,
                reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                "failed to construct cost plan build output",
            ),
        }
    }

    fn run_guard_validate(&self, req: &CostRouteGuardValidateRequest) -> Ph1CostResponse {
        if req.route_budgets.is_empty() || req.route_guardrails.is_empty() {
            return self.refuse(
                CostCapabilityId::CostRouteGuardValidate,
                reason_codes::PH1_COST_UPSTREAM_INPUT_MISSING,
                "route budgets or guardrails missing",
            );
        }
        if req.route_budgets.len() > self.config.max_route_budgets as usize {
            return self.refuse(
                CostCapabilityId::CostRouteGuardValidate,
                reason_codes::PH1_COST_BUDGET_EXCEEDED,
                "route_budgets exceeds runtime budget",
            );
        }

        let expected = match build_guardrails(&req.route_budgets) {
            Ok(g) => g,
            Err(_) => {
                return self.refuse(
                    CostCapabilityId::CostRouteGuardValidate,
                    reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                    "failed to rebuild expected route guardrails",
                )
            }
        };

        let mut diagnostics: Vec<String> = Vec::new();
        if req.route_guardrails.len() != expected.len() {
            diagnostics.push("route_guardrails_len_mismatch".to_string());
        }

        for expected_guard in &expected {
            match req
                .route_guardrails
                .iter()
                .find(|g| g.route_kind == expected_guard.route_kind)
            {
                Some(actual) => {
                    collect_guardrail_drift_diagnostics(actual, expected_guard, &mut diagnostics)
                }
                None => diagnostics.push(format!(
                    "{}_guardrail_missing",
                    expected_guard.route_kind.as_str().to_ascii_lowercase()
                )),
            }
            if diagnostics.len() >= self.config.max_diagnostics as usize {
                break;
            }
        }

        for actual in &req.route_guardrails {
            if expected.iter().all(|g| g.route_kind != actual.route_kind) {
                diagnostics.push(format!(
                    "{}_guardrail_unexpected",
                    actual.route_kind.as_str().to_ascii_lowercase()
                ));
                if diagnostics.len() >= self.config.max_diagnostics as usize {
                    break;
                }
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (status, reason_code) = if diagnostics.is_empty() {
            (
                CostValidationStatus::Ok,
                reason_codes::PH1_COST_OK_GUARD_VALIDATE,
            )
        } else {
            (
                CostValidationStatus::Fail,
                reason_codes::PH1_COST_VALIDATION_FAILED,
            )
        };

        match CostRouteGuardValidateOk::v1(reason_code, status, diagnostics, true) {
            Ok(ok) => Ph1CostResponse::CostRouteGuardValidateOk(ok),
            Err(_) => self.refuse(
                CostCapabilityId::CostRouteGuardValidate,
                reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                "failed to construct cost guard validate output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: CostCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1CostResponse {
        let r = CostRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("CostRefuse::v1 must construct for static message");
        Ph1CostResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1CostRequest) -> CostCapabilityId {
    match req {
        Ph1CostRequest::CostBudgetPlanBuild(_) => CostCapabilityId::CostBudgetPlanBuild,
        Ph1CostRequest::CostRouteGuardValidate(_) => CostCapabilityId::CostRouteGuardValidate,
    }
}

fn build_guardrails(
    budgets: &[CostRouteBudget],
) -> Result<Vec<CostRouteGuardrail>, selene_kernel_contracts::ContractViolation> {
    budgets
        .iter()
        .map(evaluate_guardrail)
        .collect::<Result<Vec<_>, _>>()
}

fn evaluate_guardrail(
    budget: &CostRouteBudget,
) -> Result<CostRouteGuardrail, selene_kernel_contracts::ContractViolation> {
    let utilization_bp = utilization_bp(budget.daily_used_units, budget.daily_budget_units);
    let remaining_units = budget
        .daily_budget_units
        .saturating_sub(budget.daily_used_units);

    let (routing_decision, route_tier_hint, response_length_hint, suggested_retry_limit) =
        if budget.daily_used_units >= budget.daily_budget_units {
            (
                CostRoutingDecision::Refuse,
                CostRouteTierHint::Budget,
                CostResponseLengthHint::Short,
                0,
            )
        } else if utilization_bp >= 9000 {
            (
                CostRoutingDecision::Degrade,
                CostRouteTierHint::Budget,
                CostResponseLengthHint::Short,
                budget.retry_cap.min(1),
            )
        } else if utilization_bp >= 7500 {
            (
                CostRoutingDecision::Degrade,
                CostRouteTierHint::Budget,
                CostResponseLengthHint::Standard,
                budget.retry_cap.min(2),
            )
        } else {
            (
                CostRoutingDecision::Allow,
                CostRouteTierHint::Standard,
                CostResponseLengthHint::Standard,
                budget.retry_cap,
            )
        };

    CostRouteGuardrail::v1(
        budget.route_kind,
        routing_decision,
        route_tier_hint,
        response_length_hint,
        suggested_retry_limit,
        remaining_units,
        utilization_bp,
    )
}

fn utilization_bp(used: u32, budget: u32) -> u16 {
    if budget == 0 {
        return 10_000;
    }
    let bp = (used as u128 * 10_000u128) / budget as u128;
    bp.min(10_000) as u16
}

fn collect_guardrail_drift_diagnostics(
    actual: &CostRouteGuardrail,
    expected: &CostRouteGuardrail,
    diagnostics: &mut Vec<String>,
) {
    let route_prefix = actual.route_kind.as_str().to_ascii_lowercase();
    if actual.routing_decision != expected.routing_decision {
        diagnostics.push(format!("{route_prefix}_routing_decision_mismatch"));
    }
    if actual.route_tier_hint != expected.route_tier_hint {
        diagnostics.push(format!("{route_prefix}_route_tier_hint_mismatch"));
    }
    if actual.response_length_hint != expected.response_length_hint {
        diagnostics.push(format!("{route_prefix}_response_length_hint_mismatch"));
    }
    if actual.suggested_retry_limit != expected.suggested_retry_limit {
        diagnostics.push(format!("{route_prefix}_suggested_retry_limit_mismatch"));
    }
    if actual.remaining_units != expected.remaining_units {
        diagnostics.push(format!("{route_prefix}_remaining_units_mismatch"));
    }
    if actual.utilization_bp != expected.utilization_bp {
        diagnostics.push(format!("{route_prefix}_utilization_bp_mismatch"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1cost::{
        CostBudgetPlanBuildRequest, CostBudgetScope, CostRequestEnvelope,
        CostRouteGuardValidateRequest, CostRouteKind,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1CostRuntime {
        Ph1CostRuntime::new(Ph1CostConfig::mvp_v1())
    }

    fn envelope(max_route_budgets: u8) -> CostRequestEnvelope {
        CostRequestEnvelope::v1(CorrelationId(1401), TurnId(101), max_route_budgets).unwrap()
    }

    fn scope() -> CostBudgetScope {
        CostBudgetScope::v1("user_hash_abc".to_string(), "2026-02-16".to_string()).unwrap()
    }

    fn budgets() -> Vec<CostRouteBudget> {
        vec![
            CostRouteBudget::v1(CostRouteKind::Stt, 100, 50, 3).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Llm, 100, 95, 3).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Tts, 100, 60, 2).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Tool, 100, 99, 1).unwrap(),
        ]
    }

    #[test]
    fn at_cost_01_budget_plan_build_output_is_schema_valid() {
        let req = Ph1CostRequest::CostBudgetPlanBuild(
            CostBudgetPlanBuildRequest::v1(envelope(4), scope(), budgets()).unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1CostResponse::CostBudgetPlanBuildOk(ok) => {
                assert_eq!(ok.route_guardrails.len(), 4);
                assert!(ok.no_truth_mutation);
            }
            _ => panic!("expected CostBudgetPlanBuildOk"),
        }
    }

    #[test]
    fn at_cost_02_degrade_modes_trigger_on_high_utilization() {
        let req = Ph1CostRequest::CostBudgetPlanBuild(
            CostBudgetPlanBuildRequest::v1(envelope(4), scope(), budgets()).unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1CostResponse::CostBudgetPlanBuildOk(ok) => {
                let llm = ok
                    .route_guardrails
                    .iter()
                    .find(|g| g.route_kind == CostRouteKind::Llm)
                    .expect("llm guardrail");
                assert_eq!(llm.routing_decision, CostRoutingDecision::Degrade);
                assert_eq!(llm.route_tier_hint, CostRouteTierHint::Budget);
                assert_eq!(llm.response_length_hint, CostResponseLengthHint::Short);
                assert!(llm.suggested_retry_limit <= 1);
            }
            _ => panic!("expected CostBudgetPlanBuildOk"),
        }
    }

    #[test]
    fn at_cost_03_budget_exhaustion_refuses_route() {
        let exhausted = vec![CostRouteBudget::v1(CostRouteKind::Tool, 100, 100, 3).unwrap()];
        let req = Ph1CostRequest::CostBudgetPlanBuild(
            CostBudgetPlanBuildRequest::v1(envelope(4), scope(), exhausted).unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1CostResponse::CostBudgetPlanBuildOk(ok) => {
                assert_eq!(ok.route_guardrails.len(), 1);
                assert_eq!(
                    ok.route_guardrails[0].routing_decision,
                    CostRoutingDecision::Refuse
                );
                assert_eq!(ok.route_guardrails[0].suggested_retry_limit, 0);
            }
            _ => panic!("expected CostBudgetPlanBuildOk"),
        }
    }

    #[test]
    fn at_cost_04_validate_fails_for_drifted_guardrails() {
        let route_budgets = vec![CostRouteBudget::v1(CostRouteKind::Stt, 100, 95, 3).unwrap()];
        let drifted = vec![CostRouteGuardrail::v1(
            CostRouteKind::Stt,
            CostRoutingDecision::Allow,
            CostRouteTierHint::Standard,
            CostResponseLengthHint::Standard,
            3,
            80,
            2000,
        )
        .unwrap()];

        let req = Ph1CostRequest::CostRouteGuardValidate(
            CostRouteGuardValidateRequest::v1(envelope(4), scope(), route_budgets, drifted)
                .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1CostResponse::CostRouteGuardValidateOk(ok) => {
                assert_eq!(ok.validation_status, CostValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_COST_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "stt_routing_decision_mismatch"));
            }
            _ => panic!("expected CostRouteGuardValidateOk"),
        }
    }
}
