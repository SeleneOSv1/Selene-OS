#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1COST_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostCapabilityId {
    CostBudgetPlanBuild,
    CostRouteGuardValidate,
}

impl CostCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            CostCapabilityId::CostBudgetPlanBuild => "COST_BUDGET_PLAN_BUILD",
            CostCapabilityId::CostRouteGuardValidate => "COST_ROUTE_GUARD_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum CostRouteKind {
    Stt,
    Llm,
    Tts,
    Tool,
}

impl CostRouteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            CostRouteKind::Stt => "STT",
            CostRouteKind::Llm => "LLM",
            CostRouteKind::Tts => "TTS",
            CostRouteKind::Tool => "TOOL",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostRoutingDecision {
    Allow,
    Degrade,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostRouteTierHint {
    Standard,
    Budget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostResponseLengthHint {
    Standard,
    Short,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_route_budgets: u8,
}

impl CostRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_route_budgets: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_route_budgets,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for CostRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_request_envelope.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_route_budgets == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_request_envelope.max_route_budgets",
                reason: "must be > 0",
            });
        }
        if self.max_route_budgets > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_request_envelope.max_route_budgets",
                reason: "must be <= 8",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostBudgetScope {
    pub schema_version: SchemaVersion,
    pub user_scope_key: String,
    pub day_bucket_utc: String,
}

impl CostBudgetScope {
    pub fn v1(user_scope_key: String, day_bucket_utc: String) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            user_scope_key,
            day_bucket_utc,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for CostBudgetScope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_budget_scope.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        validate_text(
            "cost_budget_scope.user_scope_key",
            &self.user_scope_key,
            128,
        )?;
        validate_text("cost_budget_scope.day_bucket_utc", &self.day_bucket_utc, 32)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRouteBudget {
    pub schema_version: SchemaVersion,
    pub route_kind: CostRouteKind,
    pub daily_budget_units: u32,
    pub daily_used_units: u32,
    pub retry_cap: u8,
}

impl CostRouteBudget {
    pub fn v1(
        route_kind: CostRouteKind,
        daily_budget_units: u32,
        daily_used_units: u32,
        retry_cap: u8,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            route_kind,
            daily_budget_units,
            daily_used_units,
            retry_cap,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for CostRouteBudget {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_budget.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        if self.daily_budget_units == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_budget.daily_budget_units",
                reason: "must be > 0",
            });
        }
        if self.daily_budget_units > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_budget.daily_budget_units",
                reason: "must be <= 1_000_000",
            });
        }
        if self.daily_used_units > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_budget.daily_used_units",
                reason: "must be <= 1_000_000",
            });
        }
        if self.retry_cap > 5 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_budget.retry_cap",
                reason: "must be <= 5",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostBudgetPlanBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: CostRequestEnvelope,
    pub budget_scope: CostBudgetScope,
    pub route_budgets: Vec<CostRouteBudget>,
}

impl CostBudgetPlanBuildRequest {
    pub fn v1(
        envelope: CostRequestEnvelope,
        budget_scope: CostBudgetScope,
        route_budgets: Vec<CostRouteBudget>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            envelope,
            budget_scope,
            route_budgets,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for CostBudgetPlanBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_budget_plan_build_request.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.budget_scope.validate()?;
        validate_route_budgets(
            "cost_budget_plan_build_request.route_budgets",
            &self.route_budgets,
            self.envelope.max_route_budgets as usize,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRouteGuardValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: CostRequestEnvelope,
    pub budget_scope: CostBudgetScope,
    pub route_budgets: Vec<CostRouteBudget>,
    pub route_guardrails: Vec<CostRouteGuardrail>,
}

impl CostRouteGuardValidateRequest {
    pub fn v1(
        envelope: CostRequestEnvelope,
        budget_scope: CostBudgetScope,
        route_budgets: Vec<CostRouteBudget>,
        route_guardrails: Vec<CostRouteGuardrail>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            envelope,
            budget_scope,
            route_budgets,
            route_guardrails,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for CostRouteGuardValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_request.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.budget_scope.validate()?;
        validate_route_budgets(
            "cost_route_guard_validate_request.route_budgets",
            &self.route_budgets,
            self.envelope.max_route_budgets as usize,
        )?;
        validate_route_guardrails(
            "cost_route_guard_validate_request.route_guardrails",
            &self.route_guardrails,
            self.envelope.max_route_budgets as usize,
        )?;
        if self.route_budgets.len() != self.route_guardrails.len() {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_request.route_guardrails",
                reason: "must align 1:1 with route_budgets",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1CostRequest {
    CostBudgetPlanBuild(CostBudgetPlanBuildRequest),
    CostRouteGuardValidate(CostRouteGuardValidateRequest),
}

impl Validate for Ph1CostRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1CostRequest::CostBudgetPlanBuild(r) => r.validate(),
            Ph1CostRequest::CostRouteGuardValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CostValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRouteGuardrail {
    pub schema_version: SchemaVersion,
    pub route_kind: CostRouteKind,
    pub routing_decision: CostRoutingDecision,
    pub route_tier_hint: CostRouteTierHint,
    pub response_length_hint: CostResponseLengthHint,
    pub suggested_retry_limit: u8,
    pub remaining_units: u32,
    pub utilization_bp: u16,
}

impl CostRouteGuardrail {
    pub fn v1(
        route_kind: CostRouteKind,
        routing_decision: CostRoutingDecision,
        route_tier_hint: CostRouteTierHint,
        response_length_hint: CostResponseLengthHint,
        suggested_retry_limit: u8,
        remaining_units: u32,
        utilization_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let g = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            route_kind,
            routing_decision,
            route_tier_hint,
            response_length_hint,
            suggested_retry_limit,
            remaining_units,
            utilization_bp,
        };
        g.validate()?;
        Ok(g)
    }
}

impl Validate for CostRouteGuardrail {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guardrail.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        if self.suggested_retry_limit > 5 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guardrail.suggested_retry_limit",
                reason: "must be <= 5",
            });
        }
        if self.utilization_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guardrail.utilization_bp",
                reason: "must be <= 10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostBudgetPlanBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: CostCapabilityId,
    pub reason_code: ReasonCodeId,
    pub budget_scope: CostBudgetScope,
    pub route_guardrails: Vec<CostRouteGuardrail>,
    pub no_truth_mutation: bool,
}

impl CostBudgetPlanBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        budget_scope: CostBudgetScope,
        route_guardrails: Vec<CostRouteGuardrail>,
        no_truth_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            capability_id: CostCapabilityId::CostBudgetPlanBuild,
            reason_code,
            budget_scope,
            route_guardrails,
            no_truth_mutation,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for CostBudgetPlanBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_budget_plan_build_ok.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        if self.capability_id != CostCapabilityId::CostBudgetPlanBuild {
            return Err(ContractViolation::InvalidValue {
                field: "cost_budget_plan_build_ok.capability_id",
                reason: "must be COST_BUDGET_PLAN_BUILD",
            });
        }
        self.budget_scope.validate()?;
        validate_route_guardrails(
            "cost_budget_plan_build_ok.route_guardrails",
            &self.route_guardrails,
            8,
        )?;
        if !self.no_truth_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "cost_budget_plan_build_ok.no_truth_mutation",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRouteGuardValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: CostCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: CostValidationStatus,
    pub diagnostics: Vec<String>,
    pub no_truth_mutation: bool,
}

impl CostRouteGuardValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: CostValidationStatus,
        diagnostics: Vec<String>,
        no_truth_mutation: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            capability_id: CostCapabilityId::CostRouteGuardValidate,
            reason_code,
            validation_status,
            diagnostics,
            no_truth_mutation,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for CostRouteGuardValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_ok.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        if self.capability_id != CostCapabilityId::CostRouteGuardValidate {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_ok.capability_id",
                reason: "must be COST_ROUTE_GUARD_VALIDATE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("cost_route_guard_validate_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == CostValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_truth_mutation {
            return Err(ContractViolation::InvalidValue {
                field: "cost_route_guard_validate_ok.no_truth_mutation",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CostRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: CostCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl CostRefuse {
    pub fn v1(
        capability_id: CostCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1COST_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for CostRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1COST_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "cost_refuse.schema_version",
                reason: "must match PH1COST_CONTRACT_VERSION",
            });
        }
        validate_text("cost_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1CostResponse {
    CostBudgetPlanBuildOk(CostBudgetPlanBuildOk),
    CostRouteGuardValidateOk(CostRouteGuardValidateOk),
    Refuse(CostRefuse),
}

impl Validate for Ph1CostResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1CostResponse::CostBudgetPlanBuildOk(o) => o.validate(),
            Ph1CostResponse::CostRouteGuardValidateOk(o) => o.validate(),
            Ph1CostResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_text(field: &'static str, text: &str, max_len: usize) -> Result<(), ContractViolation> {
    if text.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if text.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if text.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_route_budgets(
    field: &'static str,
    route_budgets: &[CostRouteBudget],
    max_entries: usize,
) -> Result<(), ContractViolation> {
    if route_budgets.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if route_budgets.len() > max_entries {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds envelope max entries",
        });
    }
    let mut kinds: BTreeSet<CostRouteKind> = BTreeSet::new();
    for b in route_budgets {
        b.validate()?;
        if !kinds.insert(b.route_kind) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "route_kind entries must be unique",
            });
        }
    }
    Ok(())
}

fn validate_route_guardrails(
    field: &'static str,
    route_guardrails: &[CostRouteGuardrail],
    max_entries: usize,
) -> Result<(), ContractViolation> {
    if route_guardrails.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if route_guardrails.len() > max_entries {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds envelope max entries",
        });
    }
    let mut kinds: BTreeSet<CostRouteKind> = BTreeSet::new();
    for g in route_guardrails {
        g.validate()?;
        if !kinds.insert(g.route_kind) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "route_kind entries must be unique",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_route_budgets: u8) -> CostRequestEnvelope {
        CostRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_route_budgets).unwrap()
    }

    fn scope() -> CostBudgetScope {
        CostBudgetScope::v1("user_scope_hash".to_string(), "2026-02-16".to_string()).unwrap()
    }

    fn budgets() -> Vec<CostRouteBudget> {
        vec![
            CostRouteBudget::v1(CostRouteKind::Stt, 100, 20, 3).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Llm, 100, 30, 2).unwrap(),
        ]
    }

    fn guardrails() -> Vec<CostRouteGuardrail> {
        vec![
            CostRouteGuardrail::v1(
                CostRouteKind::Stt,
                CostRoutingDecision::Allow,
                CostRouteTierHint::Standard,
                CostResponseLengthHint::Standard,
                3,
                80,
                2000,
            )
            .unwrap(),
            CostRouteGuardrail::v1(
                CostRouteKind::Llm,
                CostRoutingDecision::Degrade,
                CostRouteTierHint::Budget,
                CostResponseLengthHint::Short,
                1,
                20,
                8000,
            )
            .unwrap(),
        ]
    }

    #[test]
    fn cost_budget_scope_rejects_empty_user_scope() {
        let scope = CostBudgetScope::v1("".to_string(), "2026-02-16".to_string());
        assert!(scope.is_err());
    }

    #[test]
    fn cost_route_budget_rejects_zero_daily_budget() {
        let budget = CostRouteBudget::v1(CostRouteKind::Tts, 0, 0, 1);
        assert!(budget.is_err());
    }

    #[test]
    fn cost_route_guard_validate_ok_requires_diagnostic_on_fail() {
        let out =
            CostRouteGuardValidateOk::v1(ReasonCodeId(1), CostValidationStatus::Fail, vec![], true);
        assert!(out.is_err());
    }

    #[test]
    fn cost_budget_plan_build_ok_requires_no_truth_mutation_true() {
        let out = CostBudgetPlanBuildOk::v1(ReasonCodeId(1), scope(), guardrails(), false);
        assert!(out.is_err());
    }

    #[test]
    fn cost_budget_plan_build_request_requires_unique_route_kinds() {
        let duplicate = vec![
            CostRouteBudget::v1(CostRouteKind::Tool, 50, 10, 2).unwrap(),
            CostRouteBudget::v1(CostRouteKind::Tool, 50, 15, 2).unwrap(),
        ];
        let req = CostBudgetPlanBuildRequest::v1(envelope(4), scope(), duplicate);
        assert!(req.is_err());
    }

    #[test]
    fn cost_route_guard_validate_request_is_schema_valid_with_matching_lengths() {
        let req = CostRouteGuardValidateRequest::v1(envelope(4), scope(), budgets(), guardrails());
        assert!(req.is_ok());
    }
}
