#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1sched::{
    Ph1SchedRequest, Ph1SchedResponse, SchedCapabilityId, SchedDecisionAction,
    SchedDecisionComputeOk, SchedDecisionComputeRequest, SchedPolicyEvaluateOk,
    SchedPolicyEvaluateRequest, SchedRefuse,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SCHED reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_SCHED_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x5343_0001);
    pub const PH1_SCHED_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x5343_0002);

    pub const SCHED_RETRY_SCHEDULED: ReasonCodeId = ReasonCodeId(0x5343_0010);
    pub const SCHED_MAX_RETRIES_REACHED: ReasonCodeId = ReasonCodeId(0x5343_0011);
    pub const SCHED_TIMEOUT: ReasonCodeId = ReasonCodeId(0x5343_0012);
    pub const SCHED_NOT_RETRYABLE: ReasonCodeId = ReasonCodeId(0x5343_0013);

    pub const PH1_SCHED_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5343_00F1);
    pub const PH1_SCHED_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5343_00F2);
    pub const PH1_SCHED_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5343_00F3);
    pub const PH1_SCHED_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5343_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SchedConfig {
    pub max_retryable_reason_codes: u8,
    pub max_backoff_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1SchedConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_retryable_reason_codes: 16,
            max_backoff_ms: 300_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1SchedRuntime {
    config: Ph1SchedConfig,
}

impl Ph1SchedRuntime {
    pub fn new(config: Ph1SchedConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1SchedRequest) -> Ph1SchedResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_SCHED_INPUT_SCHEMA_INVALID,
                "sched request failed contract validation",
            );
        }

        match req {
            Ph1SchedRequest::SchedPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1SchedRequest::SchedDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &SchedPolicyEvaluateRequest) -> Ph1SchedResponse {
        if req.step_id.is_empty() {
            return self.refuse(
                SchedCapabilityId::SchedPolicyEvaluate,
                reason_codes::PH1_SCHED_UPSTREAM_INPUT_MISSING,
                "step_id is missing",
            );
        }
        if req.retryable_reason_codes.len() > self.config.max_retryable_reason_codes as usize {
            return self.refuse(
                SchedCapabilityId::SchedPolicyEvaluate,
                reason_codes::PH1_SCHED_BUDGET_EXCEEDED,
                "retryable reason-code budget exceeded",
            );
        }
        if req.retry_backoff_ms > self.config.max_backoff_ms {
            return self.refuse(
                SchedCapabilityId::SchedPolicyEvaluate,
                reason_codes::PH1_SCHED_BUDGET_EXCEEDED,
                "retry_backoff_ms exceeds runtime budget",
            );
        }

        let elapsed_ns = req.now_ns.0.saturating_sub(req.step_started_at_ns.0);
        let timeout_ns = u64::from(req.timeout_ms).saturating_mul(1_000_000);
        let timeout_exceeded = elapsed_ns >= timeout_ns;
        let max_retries_reached = req.attempt_index >= req.max_retries;
        let last_failure_retryable = match req.last_failure_reason_code {
            Some(code) => req.retryable_reason_codes.iter().any(|rc| rc.0 == code.0),
            None => false,
        };

        let retry_allowed = !timeout_exceeded && !max_retries_reached && last_failure_retryable;
        let next_attempt_index = if retry_allowed {
            req.attempt_index.saturating_add(1)
        } else {
            req.attempt_index
        };

        let reason_code = if timeout_exceeded {
            reason_codes::SCHED_TIMEOUT
        } else if max_retries_reached {
            reason_codes::SCHED_MAX_RETRIES_REACHED
        } else if !last_failure_retryable {
            reason_codes::SCHED_NOT_RETRYABLE
        } else {
            reason_codes::PH1_SCHED_OK_POLICY_EVALUATE
        };

        match SchedPolicyEvaluateOk::v1(
            reason_code,
            req.tenant_id.clone(),
            req.work_order_id.clone(),
            req.step_id.clone(),
            req.attempt_index,
            next_attempt_index,
            timeout_exceeded,
            max_retries_reached,
            retry_allowed,
            req.retry_backoff_ms,
            true,
            true,
        ) {
            Ok(ok) => Ph1SchedResponse::SchedPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                SchedCapabilityId::SchedPolicyEvaluate,
                reason_codes::PH1_SCHED_INTERNAL_PIPELINE_ERROR,
                "failed to construct sched policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &SchedDecisionComputeRequest) -> Ph1SchedResponse {
        if req.retry_backoff_ms > self.config.max_backoff_ms {
            return self.refuse(
                SchedCapabilityId::SchedDecisionCompute,
                reason_codes::PH1_SCHED_BUDGET_EXCEEDED,
                "retry_backoff_ms exceeds runtime budget",
            );
        }

        let (action, reason_code, next_due_at_ns, attempt_next_index) = if req.timeout_exceeded {
            (
                SchedDecisionAction::Fail,
                reason_codes::SCHED_TIMEOUT,
                None,
                req.attempt_index,
            )
        } else if req.max_retries_reached {
            (
                SchedDecisionAction::Fail,
                reason_codes::SCHED_MAX_RETRIES_REACHED,
                None,
                req.attempt_index,
            )
        } else if req.retry_allowed {
            let next_due_at_ns = MonotonicTimeNs(
                req.now_ns
                    .0
                    .saturating_add(u64::from(req.retry_backoff_ms).saturating_mul(1_000_000)),
            );
            (
                SchedDecisionAction::RetryAt,
                reason_codes::SCHED_RETRY_SCHEDULED,
                Some(next_due_at_ns),
                req.next_attempt_index,
            )
        } else {
            (
                SchedDecisionAction::Wait,
                reason_codes::SCHED_NOT_RETRYABLE,
                None,
                req.attempt_index,
            )
        };

        match SchedDecisionComputeOk::v1(
            reason_code,
            action,
            req.attempt_index,
            attempt_next_index,
            next_due_at_ns,
            true,
            true,
        ) {
            Ok(ok) => Ph1SchedResponse::SchedDecisionComputeOk(ok),
            Err(_) => self.refuse(
                SchedCapabilityId::SchedDecisionCompute,
                reason_codes::PH1_SCHED_INTERNAL_PIPELINE_ERROR,
                "failed to construct sched decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: SchedCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1SchedResponse {
        let out = SchedRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("SchedRefuse::v1 must construct for static messages");
        Ph1SchedResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1SchedRequest) -> SchedCapabilityId {
    match req {
        Ph1SchedRequest::SchedPolicyEvaluate(_) => SchedCapabilityId::SchedPolicyEvaluate,
        Ph1SchedRequest::SchedDecisionCompute(_) => SchedCapabilityId::SchedDecisionCompute,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1sched::{
        SchedDecisionComputeRequest, SchedPolicyEvaluateRequest, SchedRequestEnvelope,
    };
    use selene_kernel_contracts::ph1work::WorkOrderId;

    fn envelope() -> SchedRequestEnvelope {
        SchedRequestEnvelope::v1(CorrelationId(6601), TurnId(7701), 8, 8, 30_000).unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        WorkOrderId::new("wo_123").unwrap()
    }

    fn policy_request(
        attempt_index: u16,
        max_retries: u16,
        now_ns: u64,
        start_ns: u64,
        last_failure_reason_code: Option<ReasonCodeId>,
    ) -> SchedPolicyEvaluateRequest {
        SchedPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            work_order_id(),
            "step_1".to_string(),
            MonotonicTimeNs(now_ns),
            MonotonicTimeNs(start_ns),
            2_000,
            max_retries,
            1_000,
            attempt_index,
            last_failure_reason_code,
            vec![ReasonCodeId(11), ReasonCodeId(12)],
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_sched_01_retry_schedule_is_deterministic() {
        let runtime = Ph1SchedRuntime::new(Ph1SchedConfig::mvp_v1());
        let policy_req = policy_request(0, 3, 2_000_000_000, 1_999_000_000, Some(ReasonCodeId(11)));
        let policy_resp = runtime.run(&Ph1SchedRequest::SchedPolicyEvaluate(policy_req));
        let Ph1SchedResponse::SchedPolicyEvaluateOk(policy_ok) = policy_resp else {
            panic!("expected sched policy ok");
        };
        assert!(policy_ok.retry_allowed);

        let decision_req = SchedDecisionComputeRequest::v1(
            envelope(),
            tenant(),
            work_order_id(),
            "step_1".to_string(),
            MonotonicTimeNs(2_000_000_000),
            policy_ok.attempt_index,
            policy_ok.next_attempt_index,
            policy_ok.retry_backoff_ms,
            policy_ok.timeout_exceeded,
            policy_ok.max_retries_reached,
            policy_ok.retry_allowed,
            true,
        )
        .unwrap();

        let out1 = runtime.run(&Ph1SchedRequest::SchedDecisionCompute(decision_req.clone()));
        let out2 = runtime.run(&Ph1SchedRequest::SchedDecisionCompute(decision_req));

        let next_due_1 = match out1 {
            Ph1SchedResponse::SchedDecisionComputeOk(ok) => {
                assert_eq!(ok.reason_code, reason_codes::SCHED_RETRY_SCHEDULED);
                ok.next_due_at_ns
            }
            _ => panic!("expected decision ok #1"),
        };
        let next_due_2 = match out2 {
            Ph1SchedResponse::SchedDecisionComputeOk(ok) => ok.next_due_at_ns,
            _ => panic!("expected decision ok #2"),
        };
        assert_eq!(next_due_1, next_due_2);
    }

    #[test]
    fn at_sched_02_max_retries_enforced() {
        let runtime = Ph1SchedRuntime::new(Ph1SchedConfig::mvp_v1());
        let policy_req = policy_request(3, 3, 2_000_000_000, 1_999_000_000, Some(ReasonCodeId(11)));
        let policy_resp = runtime.run(&Ph1SchedRequest::SchedPolicyEvaluate(policy_req));
        let Ph1SchedResponse::SchedPolicyEvaluateOk(policy_ok) = policy_resp else {
            panic!("expected sched policy ok");
        };
        assert!(policy_ok.max_retries_reached);
        assert!(!policy_ok.retry_allowed);
    }

    #[test]
    fn at_sched_03_wait_does_not_advance_plan() {
        let runtime = Ph1SchedRuntime::new(Ph1SchedConfig::mvp_v1());
        let policy_req = policy_request(1, 3, 2_000_000_000, 1_999_000_000, Some(ReasonCodeId(99)));
        let policy_resp = runtime.run(&Ph1SchedRequest::SchedPolicyEvaluate(policy_req));
        let Ph1SchedResponse::SchedPolicyEvaluateOk(policy_ok) = policy_resp else {
            panic!("expected sched policy ok");
        };
        assert!(!policy_ok.retry_allowed);
        assert_eq!(policy_ok.next_attempt_index, policy_ok.attempt_index);

        let decision_req = SchedDecisionComputeRequest::v1(
            envelope(),
            tenant(),
            work_order_id(),
            "step_1".to_string(),
            MonotonicTimeNs(2_000_000_000),
            policy_ok.attempt_index,
            policy_ok.next_attempt_index,
            policy_ok.retry_backoff_ms,
            policy_ok.timeout_exceeded,
            policy_ok.max_retries_reached,
            policy_ok.retry_allowed,
            true,
        )
        .unwrap();
        let decision_resp = runtime.run(&Ph1SchedRequest::SchedDecisionCompute(decision_req));
        let Ph1SchedResponse::SchedDecisionComputeOk(decision_ok) = decision_resp else {
            panic!("expected decision compute ok");
        };
        assert_eq!(decision_ok.action, SchedDecisionAction::Wait);
        assert_eq!(decision_ok.attempt_next_index, decision_ok.attempt_index);
    }

    #[test]
    fn at_sched_04_timeout_enforced() {
        let runtime = Ph1SchedRuntime::new(Ph1SchedConfig::mvp_v1());
        let policy_req = policy_request(1, 3, 3_500_000_000, 1_000_000_000, Some(ReasonCodeId(11)));
        let policy_resp = runtime.run(&Ph1SchedRequest::SchedPolicyEvaluate(policy_req));
        let Ph1SchedResponse::SchedPolicyEvaluateOk(policy_ok) = policy_resp else {
            panic!("expected sched policy ok");
        };
        assert!(policy_ok.timeout_exceeded);
        assert!(!policy_ok.retry_allowed);
        assert_eq!(policy_ok.reason_code, reason_codes::SCHED_TIMEOUT);
    }
}
