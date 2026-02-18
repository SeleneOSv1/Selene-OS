#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1quota::{
    Ph1QuotaRequest, Ph1QuotaResponse, QuotaCapabilityId, QuotaDecisionAction,
    QuotaDecisionComputeOk, QuotaDecisionComputeRequest, QuotaPolicyEvaluateOk,
    QuotaPolicyEvaluateRequest, QuotaRefuse, QuotaThrottleCause,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.QUOTA reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_QUOTA_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x5154_0001);
    pub const PH1_QUOTA_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x5154_0002);

    pub const QUOTA_RATE_LIMIT: ReasonCodeId = ReasonCodeId(0x5154_0010);
    pub const QUOTA_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5154_0011);
    pub const QUOTA_POLICY_BLOCKED: ReasonCodeId = ReasonCodeId(0x5154_0012);

    pub const PH1_QUOTA_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5154_00F1);
    pub const PH1_QUOTA_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5154_00F2);
    pub const PH1_QUOTA_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5154_00F3);
    pub const PH1_QUOTA_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5154_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1QuotaConfig {
    pub max_wait_ms: u32,
    pub default_wait_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1QuotaConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_wait_ms: 120_000,
            default_wait_ms: 5_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1QuotaRuntime {
    config: Ph1QuotaConfig,
}

impl Ph1QuotaRuntime {
    pub fn new(config: Ph1QuotaConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1QuotaRequest) -> Ph1QuotaResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_QUOTA_INPUT_SCHEMA_INVALID,
                "quota request failed contract validation",
            );
        }

        match req {
            Ph1QuotaRequest::QuotaPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1QuotaRequest::QuotaDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &QuotaPolicyEvaluateRequest) -> Ph1QuotaResponse {
        if req.capability_id.is_none() && req.tool_name.is_none() {
            return self.refuse(
                QuotaCapabilityId::QuotaPolicyEvaluate,
                reason_codes::PH1_QUOTA_UPSTREAM_INPUT_MISSING,
                "missing capability/tool reference",
            );
        }

        if let Some(suggested_wait_ms) = req.suggested_wait_ms {
            if suggested_wait_ms > self.config.max_wait_ms {
                return self.refuse(
                    QuotaCapabilityId::QuotaPolicyEvaluate,
                    reason_codes::PH1_QUOTA_BUDGET_EXCEEDED,
                    "suggested wait exceeds runtime max_wait_ms",
                );
            }
        }

        let throttle_cause = if req.policy_blocked {
            QuotaThrottleCause::PolicyBlocked
        } else if req.budget_exceeded {
            QuotaThrottleCause::BudgetExceeded
        } else if req.rate_limit_exceeded {
            QuotaThrottleCause::RateLimit
        } else {
            QuotaThrottleCause::None
        };

        let allow_eligible = throttle_cause == QuotaThrottleCause::None;
        let wait_permitted = !allow_eligible
            && throttle_cause != QuotaThrottleCause::PolicyBlocked
            && req.wait_permitted;
        let wait_ms = if wait_permitted {
            let base_wait = req.suggested_wait_ms.unwrap_or(self.config.default_wait_ms);
            Some(base_wait.min(self.config.max_wait_ms))
        } else {
            None
        };
        let refuse_required = !allow_eligible && !wait_permitted;

        let reason_code = match throttle_cause {
            QuotaThrottleCause::None => reason_codes::PH1_QUOTA_OK_POLICY_EVALUATE,
            QuotaThrottleCause::RateLimit => reason_codes::QUOTA_RATE_LIMIT,
            QuotaThrottleCause::BudgetExceeded => reason_codes::QUOTA_BUDGET_EXCEEDED,
            QuotaThrottleCause::PolicyBlocked => reason_codes::QUOTA_POLICY_BLOCKED,
        };

        let capability_ref = match req.operation_kind {
            selene_kernel_contracts::ph1quota::QuotaOperationKind::Tool => req.tool_name.clone(),
            selene_kernel_contracts::ph1quota::QuotaOperationKind::Stt
            | selene_kernel_contracts::ph1quota::QuotaOperationKind::Tts
            | selene_kernel_contracts::ph1quota::QuotaOperationKind::Simulation
            | selene_kernel_contracts::ph1quota::QuotaOperationKind::Export => {
                req.capability_id.clone()
            }
        };

        match QuotaPolicyEvaluateOk::v1(
            reason_code,
            req.tenant_id.clone(),
            req.operation_kind,
            capability_ref,
            throttle_cause,
            allow_eligible,
            wait_permitted,
            wait_ms,
            refuse_required,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                QuotaCapabilityId::QuotaPolicyEvaluate,
                reason_codes::PH1_QUOTA_INTERNAL_PIPELINE_ERROR,
                "failed to construct quota policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &QuotaDecisionComputeRequest) -> Ph1QuotaResponse {
        if let Some(wait_ms) = req.wait_ms {
            if wait_ms > self.config.max_wait_ms {
                return self.refuse(
                    QuotaCapabilityId::QuotaDecisionCompute,
                    reason_codes::PH1_QUOTA_BUDGET_EXCEEDED,
                    "wait_ms exceeds runtime max_wait_ms",
                );
            }
        }

        let (action, reason_code, wait_ms) = if req.allow_eligible {
            (
                QuotaDecisionAction::Allow,
                reason_codes::PH1_QUOTA_OK_DECISION_COMPUTE,
                None,
            )
        } else if req.throttle_cause == QuotaThrottleCause::PolicyBlocked || req.refuse_required {
            (
                QuotaDecisionAction::Refuse,
                reason_for_cause(req.throttle_cause),
                None,
            )
        } else if req.wait_permitted {
            (
                QuotaDecisionAction::Wait,
                reason_for_cause(req.throttle_cause),
                Some(req.wait_ms.unwrap_or(self.config.default_wait_ms)),
            )
        } else {
            (
                QuotaDecisionAction::Refuse,
                reason_for_cause(req.throttle_cause),
                None,
            )
        };

        match QuotaDecisionComputeOk::v1(reason_code, action, wait_ms, true, true, true) {
            Ok(ok) => Ph1QuotaResponse::QuotaDecisionComputeOk(ok),
            Err(_) => self.refuse(
                QuotaCapabilityId::QuotaDecisionCompute,
                reason_codes::PH1_QUOTA_INTERNAL_PIPELINE_ERROR,
                "failed to construct quota decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: QuotaCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1QuotaResponse {
        let out = QuotaRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("QuotaRefuse::v1 must construct for static messages");
        Ph1QuotaResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1QuotaRequest) -> QuotaCapabilityId {
    match req {
        Ph1QuotaRequest::QuotaPolicyEvaluate(_) => QuotaCapabilityId::QuotaPolicyEvaluate,
        Ph1QuotaRequest::QuotaDecisionCompute(_) => QuotaCapabilityId::QuotaDecisionCompute,
    }
}

fn reason_for_cause(cause: QuotaThrottleCause) -> ReasonCodeId {
    match cause {
        QuotaThrottleCause::None => reason_codes::PH1_QUOTA_OK_DECISION_COMPUTE,
        QuotaThrottleCause::RateLimit => reason_codes::QUOTA_RATE_LIMIT,
        QuotaThrottleCause::BudgetExceeded => reason_codes::QUOTA_BUDGET_EXCEEDED,
        QuotaThrottleCause::PolicyBlocked => reason_codes::QUOTA_POLICY_BLOCKED,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1quota::{
        QuotaDecisionComputeRequest, QuotaOperationKind, QuotaRequestEnvelope,
    };

    fn envelope() -> QuotaRequestEnvelope {
        QuotaRequestEnvelope::v1(CorrelationId(7201), TurnId(8201), 8, 30_000).unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn policy_request(wait_permitted: bool) -> QuotaPolicyEvaluateRequest {
        QuotaPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            Some("user_1".to_string()),
            Some("device_1".to_string()),
            QuotaOperationKind::Stt,
            Some("PH1C_TRANSCRIPT_OK_COMMIT_ROW".to_string()),
            None,
            selene_kernel_contracts::MonotonicTimeNs(5_000_000),
            Some(5000),
            true,
            false,
            false,
            wait_permitted,
            if wait_permitted { Some(2000) } else { None },
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_quota_01_rate_limiting_is_enforced_deterministically() {
        let runtime = Ph1QuotaRuntime::new(Ph1QuotaConfig::mvp_v1());
        let req = policy_request(true);
        let out1 = runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(req.clone()));
        let out2 = runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(req));

        let ok1 = match out1 {
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy-evaluate ok #1"),
        };
        let ok2 = match out2 {
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy-evaluate ok #2"),
        };

        assert_eq!(ok1.throttle_cause, QuotaThrottleCause::RateLimit);
        assert_eq!(ok1.wait_ms, Some(2000));
        assert_eq!(ok1.wait_ms, ok2.wait_ms);
    }

    #[test]
    fn at_quota_02_wait_vs_refuse_follows_policy() {
        let runtime = Ph1QuotaRuntime::new(Ph1QuotaConfig::mvp_v1());

        let wait_policy =
            match runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(policy_request(true))) {
                Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
                _ => panic!("expected policy-evaluate ok"),
            };
        let wait_decision_req = QuotaDecisionComputeRequest::v1(
            envelope(),
            tenant(),
            QuotaOperationKind::Stt,
            wait_policy.throttle_cause,
            wait_policy.allow_eligible,
            wait_policy.wait_permitted,
            wait_policy.wait_ms,
            wait_policy.refuse_required,
            true,
            true,
            true,
        )
        .unwrap();
        let wait_decision = runtime.run(&Ph1QuotaRequest::QuotaDecisionCompute(wait_decision_req));
        let Ph1QuotaResponse::QuotaDecisionComputeOk(wait_ok) = wait_decision else {
            panic!("expected quota decision wait");
        };
        assert_eq!(wait_ok.action, QuotaDecisionAction::Wait);

        let refuse_policy =
            match runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(policy_request(false))) {
                Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
                _ => panic!("expected policy-evaluate ok"),
            };
        let refuse_decision_req = QuotaDecisionComputeRequest::v1(
            envelope(),
            tenant(),
            QuotaOperationKind::Stt,
            refuse_policy.throttle_cause,
            refuse_policy.allow_eligible,
            refuse_policy.wait_permitted,
            refuse_policy.wait_ms,
            refuse_policy.refuse_required,
            true,
            true,
            true,
        )
        .unwrap();
        let refuse_decision =
            runtime.run(&Ph1QuotaRequest::QuotaDecisionCompute(refuse_decision_req));
        let Ph1QuotaResponse::QuotaDecisionComputeOk(refuse_ok) = refuse_decision else {
            panic!("expected quota decision refuse");
        };
        assert_eq!(refuse_ok.action, QuotaDecisionAction::Refuse);
    }

    #[test]
    fn at_quota_03_budget_exceeded_precedence_over_rate_limit() {
        let runtime = Ph1QuotaRuntime::new(Ph1QuotaConfig::mvp_v1());
        let req = QuotaPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            Some("user_1".to_string()),
            Some("device_1".to_string()),
            QuotaOperationKind::Stt,
            Some("PH1C_TRANSCRIPT_OK_COMMIT_ROW".to_string()),
            None,
            selene_kernel_contracts::MonotonicTimeNs(5_000_000),
            Some(5000),
            true,
            true,
            false,
            true,
            Some(2000),
            true,
            true,
            true,
        )
        .unwrap();
        let out = runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(req));
        let Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) = out else {
            panic!("expected policy-evaluate ok");
        };
        assert_eq!(ok.throttle_cause, QuotaThrottleCause::BudgetExceeded);
        assert_eq!(ok.reason_code, reason_codes::QUOTA_BUDGET_EXCEEDED);
    }

    #[test]
    fn at_quota_04_policy_block_forces_refuse() {
        let runtime = Ph1QuotaRuntime::new(Ph1QuotaConfig::mvp_v1());
        let req = QuotaPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            Some("user_1".to_string()),
            Some("device_1".to_string()),
            QuotaOperationKind::Stt,
            Some("PH1C_TRANSCRIPT_OK_COMMIT_ROW".to_string()),
            None,
            selene_kernel_contracts::MonotonicTimeNs(5_000_000),
            Some(5000),
            false,
            false,
            true,
            false,
            None,
            true,
            true,
            true,
        )
        .unwrap();
        let policy = match runtime.run(&Ph1QuotaRequest::QuotaPolicyEvaluate(req)) {
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy-evaluate ok"),
        };
        assert!(policy.refuse_required);

        let decision_req = QuotaDecisionComputeRequest::v1(
            envelope(),
            tenant(),
            QuotaOperationKind::Stt,
            policy.throttle_cause,
            policy.allow_eligible,
            policy.wait_permitted,
            policy.wait_ms,
            policy.refuse_required,
            true,
            true,
            true,
        )
        .unwrap();
        let decision = runtime.run(&Ph1QuotaRequest::QuotaDecisionCompute(decision_req));
        let Ph1QuotaResponse::QuotaDecisionComputeOk(ok) = decision else {
            panic!("expected decision output");
        };
        assert_eq!(ok.action, QuotaDecisionAction::Refuse);
        assert_eq!(ok.reason_code, reason_codes::QUOTA_POLICY_BLOCKED);
    }
}
