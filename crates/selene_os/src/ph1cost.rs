#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1cost::{
    CostBudgetPlanBuildOk, CostBudgetPlanBuildRequest, CostBudgetScope, CostCapabilityId,
    CostRefuse, CostRequestEnvelope, CostRouteBudget, CostRouteGuardValidateOk,
    CostRouteGuardValidateRequest, CostRouteKind, CostValidationStatus, Ph1CostRequest,
    Ph1CostResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.COST OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_COST_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x434F_0101);
    pub const PH1_COST_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x434F_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1CostWiringConfig {
    pub cost_enabled: bool,
    pub max_route_budgets: u8,
}

impl Ph1CostWiringConfig {
    pub fn mvp_v1(cost_enabled: bool) -> Self {
        Self {
            cost_enabled,
            max_route_budgets: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub budget_scope: CostBudgetScope,
    pub route_budgets: Vec<CostRouteBudget>,
}

impl CostTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        budget_scope: CostBudgetScope,
        route_budgets: Vec<CostRouteBudget>,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            budget_scope,
            route_budgets,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for CostTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.budget_scope.validate()?;

        if self.route_budgets.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_turn_input.route_budgets",
                reason: "must be <= 8",
            });
        }
        let mut kinds: BTreeSet<CostRouteKind> = BTreeSet::new();
        for route_budget in &self.route_budgets {
            route_budget.validate()?;
            if !kinds.insert(route_budget.route_kind) {
                return Err(ContractViolation::InvalidValue {
                    field: "cost_turn_input.route_budgets",
                    reason: "route_kind entries must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub plan_build: CostBudgetPlanBuildOk,
    pub guard_validate: CostRouteGuardValidateOk,
}

impl CostForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        plan_build: CostBudgetPlanBuildOk,
        guard_validate: CostRouteGuardValidateOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            plan_build,
            guard_validate,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for CostForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.plan_build.validate()?;
        self.guard_validate.validate()?;
        if self.guard_validate.validation_status != CostValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "cost_forward_bundle.guard_validate.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CostWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoBudgetInputs,
    Refused(CostRefuse),
    Forwarded(CostForwardBundle),
}

pub trait Ph1CostEngine {
    fn run(&self, req: &Ph1CostRequest) -> Ph1CostResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1CostWiring<E>
where
    E: Ph1CostEngine,
{
    config: Ph1CostWiringConfig,
    engine: E,
}

impl<E> Ph1CostWiring<E>
where
    E: Ph1CostEngine,
{
    pub fn new(config: Ph1CostWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_route_budgets == 0 || config.max_route_budgets > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1cost_wiring_config.max_route_budgets",
                reason: "must be within 1..=8",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &CostTurnInput) -> Result<CostWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.cost_enabled {
            return Ok(CostWiringOutcome::NotInvokedDisabled);
        }

        if input.route_budgets.is_empty() {
            return Ok(CostWiringOutcome::NotInvokedNoBudgetInputs);
        }

        let envelope = CostRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_route_budgets, 8),
        )?;

        let build_req = Ph1CostRequest::CostBudgetPlanBuild(CostBudgetPlanBuildRequest::v1(
            envelope.clone(),
            input.budget_scope.clone(),
            input.route_budgets.clone(),
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1CostResponse::Refuse(r) => return Ok(CostWiringOutcome::Refused(r)),
            Ph1CostResponse::CostBudgetPlanBuildOk(ok) => ok,
            Ph1CostResponse::CostRouteGuardValidateOk(_) => {
                return Ok(CostWiringOutcome::Refused(CostRefuse::v1(
                    CostCapabilityId::CostBudgetPlanBuild,
                    reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                    "unexpected guard-validate response for plan-build request".to_string(),
                )?))
            }
        };

        let validate_req =
            Ph1CostRequest::CostRouteGuardValidate(CostRouteGuardValidateRequest::v1(
                envelope,
                input.budget_scope.clone(),
                input.route_budgets.clone(),
                build_ok.route_guardrails.clone(),
            )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1CostResponse::Refuse(r) => return Ok(CostWiringOutcome::Refused(r)),
            Ph1CostResponse::CostRouteGuardValidateOk(ok) => ok,
            Ph1CostResponse::CostBudgetPlanBuildOk(_) => {
                return Ok(CostWiringOutcome::Refused(CostRefuse::v1(
                    CostCapabilityId::CostRouteGuardValidate,
                    reason_codes::PH1_COST_INTERNAL_PIPELINE_ERROR,
                    "unexpected plan-build response for guard-validate request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != CostValidationStatus::Ok {
            return Ok(CostWiringOutcome::Refused(CostRefuse::v1(
                CostCapabilityId::CostRouteGuardValidate,
                reason_codes::PH1_COST_VALIDATION_FAILED,
                "cost route guard validation failed".to_string(),
            )?));
        }

        let bundle =
            CostForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(CostWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1cost::{
        CostResponseLengthHint, CostRouteGuardrail, CostRouteTierHint, CostRoutingDecision,
    };
    use selene_kernel_contracts::ReasonCodeId;

    fn scope() -> CostBudgetScope {
        CostBudgetScope::v1("user_hash_os".to_string(), "2026-02-16".to_string()).unwrap()
    }

    fn budgets() -> Vec<CostRouteBudget> {
        vec![
            CostRouteBudget::v1(CostRouteKind::Stt, 100, 30, 3).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Llm, 100, 96, 3).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Tts, 100, 25, 2).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Tool, 100, 40, 1).unwrap(),
        ]
    }

    struct DeterministicCostEngine;

    impl Ph1CostEngine for DeterministicCostEngine {
        fn run(&self, req: &Ph1CostRequest) -> Ph1CostResponse {
            match req {
                Ph1CostRequest::CostBudgetPlanBuild(r) => {
                    let guardrails = r
                        .route_budgets
                        .iter()
                        .map(|budget| {
                            let utilization_bp = if budget.daily_budget_units == 0 {
                                10_000
                            } else {
                                ((budget.daily_used_units as u128 * 10_000u128)
                                    / budget.daily_budget_units as u128)
                                    .min(10_000) as u16
                            };
                            let remaining_units = budget
                                .daily_budget_units
                                .saturating_sub(budget.daily_used_units);
                            let (decision, tier, length, retries) = if utilization_bp >= 9000 {
                                (
                                    CostRoutingDecision::Degrade,
                                    CostRouteTierHint::Budget,
                                    CostResponseLengthHint::Short,
                                    budget.retry_cap.min(1),
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
                                decision,
                                tier,
                                length,
                                retries,
                                remaining_units,
                                utilization_bp,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();

                    Ph1CostResponse::CostBudgetPlanBuildOk(
                        CostBudgetPlanBuildOk::v1(
                            ReasonCodeId(1),
                            r.budget_scope.clone(),
                            guardrails,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1CostRequest::CostRouteGuardValidate(_r) => {
                    Ph1CostResponse::CostRouteGuardValidateOk(
                        CostRouteGuardValidateOk::v1(
                            ReasonCodeId(2),
                            CostValidationStatus::Ok,
                            vec![],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftCostEngine;

    impl Ph1CostEngine for DriftCostEngine {
        fn run(&self, req: &Ph1CostRequest) -> Ph1CostResponse {
            match req {
                Ph1CostRequest::CostBudgetPlanBuild(r) => {
                    let guardrails = r
                        .route_budgets
                        .iter()
                        .map(|budget| {
                            CostRouteGuardrail::v1(
                                budget.route_kind,
                                CostRoutingDecision::Allow,
                                CostRouteTierHint::Standard,
                                CostResponseLengthHint::Standard,
                                budget.retry_cap,
                                99,
                                1000,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    Ph1CostResponse::CostBudgetPlanBuildOk(
                        CostBudgetPlanBuildOk::v1(
                            ReasonCodeId(10),
                            r.budget_scope.clone(),
                            guardrails,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1CostRequest::CostRouteGuardValidate(_r) => {
                    Ph1CostResponse::CostRouteGuardValidateOk(
                        CostRouteGuardValidateOk::v1(
                            ReasonCodeId(11),
                            CostValidationStatus::Fail,
                            vec!["llm_routing_decision_mismatch".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    #[test]
    fn at_cost_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1CostWiring::new(Ph1CostWiringConfig::mvp_v1(true), DeterministicCostEngine).unwrap();

        let input =
            CostTurnInput::v1(CorrelationId(1501), TurnId(111), scope(), budgets()).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            CostWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(bundle.plan_build.route_guardrails.len(), 4);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_cost_02_os_preserves_guardrails_for_downstream_routing() {
        let wiring =
            Ph1CostWiring::new(Ph1CostWiringConfig::mvp_v1(true), DeterministicCostEngine).unwrap();

        let input =
            CostTurnInput::v1(CorrelationId(1502), TurnId(112), scope(), budgets()).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            CostWiringOutcome::Forwarded(bundle) => {
                let llm = bundle
                    .plan_build
                    .route_guardrails
                    .iter()
                    .find(|g| g.route_kind == CostRouteKind::Llm)
                    .expect("llm guardrail");
                assert_eq!(llm.routing_decision, CostRoutingDecision::Degrade);
                assert_eq!(llm.route_tier_hint, CostRouteTierHint::Budget);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_cost_03_os_does_not_invoke_when_cost_is_disabled() {
        let wiring =
            Ph1CostWiring::new(Ph1CostWiringConfig::mvp_v1(false), DeterministicCostEngine)
                .unwrap();

        let input =
            CostTurnInput::v1(CorrelationId(1503), TurnId(113), scope(), budgets()).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, CostWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_cost_04_os_fails_closed_on_validation_drift() {
        let wiring =
            Ph1CostWiring::new(Ph1CostWiringConfig::mvp_v1(true), DriftCostEngine).unwrap();

        let input =
            CostTurnInput::v1(CorrelationId(1504), TurnId(114), scope(), budgets()).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            CostWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_COST_VALIDATION_FAILED);
                assert_eq!(r.capability_id, CostCapabilityId::CostRouteGuardValidate);
            }
            _ => panic!("expected Refused"),
        }
    }
}
