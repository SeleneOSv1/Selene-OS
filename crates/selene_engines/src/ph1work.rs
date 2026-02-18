#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1work::{
    Ph1WorkRequest, Ph1WorkResponse, WorkCapabilityId, WorkDecisionComputeOk,
    WorkDecisionComputeRequest, WorkDecisionStatus, WorkPolicyEvaluateOk,
    WorkPolicyEvaluateRequest, WorkRefuse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.WORK reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_WORK_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x574B_0001);
    pub const PH1_WORK_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x574B_0002);

    pub const WORK_EVENT_INVALID: ReasonCodeId = ReasonCodeId(0x574B_0010);
    pub const WORK_APPEND_ONLY_VIOLATION: ReasonCodeId = ReasonCodeId(0x574B_0011);
    pub const WORK_IDEMPOTENCY_DUP: ReasonCodeId = ReasonCodeId(0x574B_0012);
    pub const WORK_TENANT_MISMATCH: ReasonCodeId = ReasonCodeId(0x574B_0013);

    pub const PH1_WORK_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x574B_00F1);
    pub const PH1_WORK_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x574B_00F2);
    pub const PH1_WORK_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x574B_00F3);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1WorkConfig {
    pub max_payload_bytes: u16,
    pub max_diagnostics: u8,
}

impl Ph1WorkConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_payload_bytes: 2_048,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1WorkRuntime {
    config: Ph1WorkConfig,
}

impl Ph1WorkRuntime {
    pub fn new(config: Ph1WorkConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1WorkRequest) -> Ph1WorkResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_WORK_INPUT_SCHEMA_INVALID,
                "work request failed contract validation",
            );
        }

        match req {
            Ph1WorkRequest::WorkPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1WorkRequest::WorkDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &WorkPolicyEvaluateRequest) -> Ph1WorkResponse {
        if req.payload_min.len() > self.config.max_payload_bytes as usize {
            return self.refuse(
                WorkCapabilityId::WorkPolicyEvaluate,
                reason_codes::PH1_WORK_BUDGET_EXCEEDED,
                "payload_min exceeds runtime max_payload_bytes",
            );
        }

        let event_valid = !req.payload_min.trim().is_empty()
            && (!req.idempotency_required || req.idempotency_key.is_some());
        let append_allowed = event_valid
            && !req.append_only_violation
            && !req.tenant_scope_mismatch
            && !req.idempotency_duplicate;

        let reason_code = if req.tenant_scope_mismatch {
            reason_codes::WORK_TENANT_MISMATCH
        } else if req.append_only_violation {
            reason_codes::WORK_APPEND_ONLY_VIOLATION
        } else if !event_valid {
            reason_codes::WORK_EVENT_INVALID
        } else if req.idempotency_duplicate {
            reason_codes::WORK_IDEMPOTENCY_DUP
        } else {
            reason_codes::PH1_WORK_OK_POLICY_EVALUATE
        };

        match WorkPolicyEvaluateOk::v1(
            reason_code,
            req.tenant_id.clone(),
            req.work_order_id.clone(),
            req.event_type,
            stable_payload_hash_hex(&req.payload_min),
            event_valid,
            append_allowed,
            req.idempotency_duplicate,
            req.append_only_violation,
            req.tenant_scope_mismatch,
            true,
            true,
        ) {
            Ok(ok) => Ph1WorkResponse::WorkPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                WorkCapabilityId::WorkPolicyEvaluate,
                reason_codes::PH1_WORK_INTERNAL_PIPELINE_ERROR,
                "failed to construct work policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &WorkDecisionComputeRequest) -> Ph1WorkResponse {
        let decision = if req.tenant_scope_mismatch {
            (
                WorkDecisionStatus::Refused,
                reason_codes::WORK_TENANT_MISMATCH,
                None,
                false,
            )
        } else if req.append_only_violation {
            (
                WorkDecisionStatus::Refused,
                reason_codes::WORK_APPEND_ONLY_VIOLATION,
                None,
                false,
            )
        } else if !req.event_valid {
            (
                WorkDecisionStatus::Fail,
                reason_codes::WORK_EVENT_INVALID,
                None,
                false,
            )
        } else if req.idempotency_duplicate {
            (
                WorkDecisionStatus::Ok,
                reason_codes::WORK_IDEMPOTENCY_DUP,
                req.existing_event_id_on_duplicate,
                true,
            )
        } else if req.append_allowed {
            (
                WorkDecisionStatus::Ok,
                reason_codes::PH1_WORK_OK_DECISION_COMPUTE,
                req.proposed_event_id,
                false,
            )
        } else {
            (
                WorkDecisionStatus::Refused,
                reason_codes::WORK_EVENT_INVALID,
                None,
                false,
            )
        };

        match WorkDecisionComputeOk::v1(decision.1, decision.0, decision.2, decision.3, true, true)
        {
            Ok(ok) => Ph1WorkResponse::WorkDecisionComputeOk(ok),
            Err(_) => self.refuse(
                WorkCapabilityId::WorkDecisionCompute,
                reason_codes::PH1_WORK_INTERNAL_PIPELINE_ERROR,
                "failed to construct work decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: WorkCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1WorkResponse {
        let out = WorkRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("WorkRefuse::v1 must construct for static messages");
        Ph1WorkResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1WorkRequest) -> WorkCapabilityId {
    match req {
        Ph1WorkRequest::WorkPolicyEvaluate(_) => WorkCapabilityId::WorkPolicyEvaluate,
        Ph1WorkRequest::WorkDecisionCompute(_) => WorkCapabilityId::WorkDecisionCompute,
    }
}

fn stable_payload_hash_hex(payload: &str) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for byte in payload.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1work::{
        WorkDecisionComputeRequest, WorkEventType, WorkPolicyEvaluateRequest, WorkRequestEnvelope,
    };
    use selene_kernel_contracts::MonotonicTimeNs;

    fn envelope() -> WorkRequestEnvelope {
        WorkRequestEnvelope::v1(CorrelationId(7301), TurnId(8301), 2048, 8).unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order() -> selene_kernel_contracts::ph1work::WorkOrderId {
        selene_kernel_contracts::ph1work::WorkOrderId::new("wo_demo").unwrap()
    }

    fn policy_request(
        append_only_violation: bool,
        tenant_scope_mismatch: bool,
        idempotency_duplicate: bool,
    ) -> WorkPolicyEvaluateRequest {
        WorkPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            work_order(),
            WorkEventType::StepStarted,
            "{\"step\":\"dispatch\"}".to_string(),
            MonotonicTimeNs(1_000_000),
            Some("idem_1".to_string()),
            true,
            append_only_violation,
            tenant_scope_mismatch,
            idempotency_duplicate,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_work_01_append_only_enforced() {
        let runtime = Ph1WorkRuntime::new(Ph1WorkConfig::mvp_v1());
        let req = policy_request(true, false, false);
        let out = runtime.run(&Ph1WorkRequest::WorkPolicyEvaluate(req));

        let ok = match out {
            Ph1WorkResponse::WorkPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy ok output"),
        };
        assert_eq!(ok.reason_code, reason_codes::WORK_APPEND_ONLY_VIOLATION);
        assert!(!ok.append_allowed);
    }

    #[test]
    fn at_work_02_current_view_rebuild_matches_deterministic_ordering_flag() {
        let runtime = Ph1WorkRuntime::new(Ph1WorkConfig::mvp_v1());
        let req = policy_request(false, false, false);

        let out1 = runtime.run(&Ph1WorkRequest::WorkPolicyEvaluate(req.clone()));
        let out2 = runtime.run(&Ph1WorkRequest::WorkPolicyEvaluate(req));

        let ok1 = match out1 {
            Ph1WorkResponse::WorkPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy ok #1"),
        };
        let ok2 = match out2 {
            Ph1WorkResponse::WorkPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected policy ok #2"),
        };
        assert!(ok1.deterministic_replay_order);
        assert_eq!(ok1.payload_min_hash, ok2.payload_min_hash);
    }

    #[test]
    fn at_work_03_idempotency_no_op_on_retry() {
        let runtime = Ph1WorkRuntime::new(Ph1WorkConfig::mvp_v1());
        let policy = policy_request(false, false, true);
        let policy_out = runtime.run(&Ph1WorkRequest::WorkPolicyEvaluate(policy));
        let Ph1WorkResponse::WorkPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy ok output");
        };
        assert!(policy_ok.idempotency_duplicate);

        let decision_req = WorkDecisionComputeRequest::v1(
            envelope(),
            policy_ok.tenant_id.clone(),
            policy_ok.work_order_id.clone(),
            policy_ok.event_type,
            policy_ok.event_valid,
            policy_ok.append_allowed,
            policy_ok.idempotency_duplicate,
            policy_ok.append_only_violation,
            policy_ok.tenant_scope_mismatch,
            Some(777),
            None,
            true,
            true,
        )
        .unwrap();
        let decision_out = runtime.run(&Ph1WorkRequest::WorkDecisionCompute(decision_req));

        let ok = match decision_out {
            Ph1WorkResponse::WorkDecisionComputeOk(ok) => ok,
            _ => panic!("expected decision ok output"),
        };
        assert_eq!(ok.reason_code, reason_codes::WORK_IDEMPOTENCY_DUP);
        assert_eq!(ok.work_order_event_id, Some(777));
        assert!(ok.idempotency_no_op);
    }

    #[test]
    fn at_work_04_tenant_mismatch_blocked() {
        let runtime = Ph1WorkRuntime::new(Ph1WorkConfig::mvp_v1());
        let policy = policy_request(false, true, false);
        let policy_out = runtime.run(&Ph1WorkRequest::WorkPolicyEvaluate(policy));
        let Ph1WorkResponse::WorkPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy ok output");
        };

        let decision_req = WorkDecisionComputeRequest::v1(
            envelope(),
            policy_ok.tenant_id,
            policy_ok.work_order_id,
            policy_ok.event_type,
            policy_ok.event_valid,
            policy_ok.append_allowed,
            policy_ok.idempotency_duplicate,
            policy_ok.append_only_violation,
            policy_ok.tenant_scope_mismatch,
            None,
            None,
            true,
            true,
        )
        .unwrap();
        let decision_out = runtime.run(&Ph1WorkRequest::WorkDecisionCompute(decision_req));

        let ok = match decision_out {
            Ph1WorkResponse::WorkDecisionComputeOk(ok) => ok,
            _ => panic!("expected decision ok output"),
        };
        assert_eq!(ok.status, WorkDecisionStatus::Refused);
        assert_eq!(ok.reason_code, reason_codes::WORK_TENANT_MISMATCH);
        assert_eq!(ok.work_order_event_id, None);
    }
}
