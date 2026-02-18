#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1lease::{
    LeaseCapabilityId, LeaseDecisionAction, LeaseDecisionComputeOk, LeaseDecisionComputeRequest,
    LeaseOperation, LeasePolicyEvaluateOk, LeasePolicyEvaluateRequest, LeaseRefuse,
    Ph1LeaseRequest, Ph1LeaseResponse,
};
use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LEASE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_LEASE_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x4C53_0001);
    pub const PH1_LEASE_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x4C53_0002);

    pub const LEASE_HELD_BY_OTHER: ReasonCodeId = ReasonCodeId(0x4C53_0010);
    pub const LEASE_TOKEN_INVALID: ReasonCodeId = ReasonCodeId(0x4C53_0011);
    pub const LEASE_TTL_OUT_OF_BOUNDS: ReasonCodeId = ReasonCodeId(0x4C53_0012);
    pub const LEASE_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4C53_0013);

    pub const PH1_LEASE_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4C53_00F1);
    pub const PH1_LEASE_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4C53_00F2);
    pub const PH1_LEASE_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4C53_00F3);
    pub const PH1_LEASE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C53_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LeaseConfig {
    pub max_ttl_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1LeaseConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_ttl_ms: 300_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1LeaseRuntime {
    config: Ph1LeaseConfig,
}

impl Ph1LeaseRuntime {
    pub fn new(config: Ph1LeaseConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1LeaseRequest) -> Ph1LeaseResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_LEASE_INPUT_SCHEMA_INVALID,
                "lease request failed contract validation",
            );
        }

        match req {
            Ph1LeaseRequest::LeasePolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1LeaseRequest::LeaseDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &LeasePolicyEvaluateRequest) -> Ph1LeaseResponse {
        if req.requested_ttl_ms > self.config.max_ttl_ms {
            return self.refuse(
                LeaseCapabilityId::LeasePolicyEvaluate,
                reason_codes::PH1_LEASE_BUDGET_EXCEEDED,
                "requested_ttl_ms exceeds runtime max_ttl_ms",
            );
        }
        if req.lease_owner_id.is_empty() {
            return self.refuse(
                LeaseCapabilityId::LeasePolicyEvaluate,
                reason_codes::PH1_LEASE_UPSTREAM_INPUT_MISSING,
                "lease_owner_id is missing",
            );
        }

        let active_exists = req.active_lease_owner_id.is_some()
            && req.active_lease_token.is_some()
            && req.active_lease_expires_at_ns.is_some();
        let active_expires_at_ns = req.active_lease_expires_at_ns.unwrap_or(MonotonicTimeNs(0));

        let lease_expired = active_exists && active_expires_at_ns.0 <= req.now_ns.0;
        let lease_exists = active_exists;
        let owner_match = active_exists
            && req
                .active_lease_owner_id
                .as_deref()
                .map(|owner| owner == req.lease_owner_id)
                .unwrap_or(false);
        let token_match = active_exists
            && req
                .lease_token
                .as_deref()
                .zip(req.active_lease_token.as_deref())
                .map(|(provided, active)| provided == active)
                .unwrap_or(false);
        let ttl_in_bounds =
            req.requested_ttl_ms > 0 && req.requested_ttl_ms <= self.config.max_ttl_ms;

        let grant_eligible = match req.operation {
            LeaseOperation::Acquire => {
                ttl_in_bounds && (!lease_exists || lease_expired || owner_match)
            }
            LeaseOperation::Renew => {
                ttl_in_bounds && lease_exists && !lease_expired && owner_match && token_match
            }
            LeaseOperation::Release => lease_exists && !lease_expired && owner_match && token_match,
        };

        let reason_code = if !ttl_in_bounds {
            reason_codes::LEASE_TTL_OUT_OF_BOUNDS
        } else {
            match req.operation {
                LeaseOperation::Acquire => {
                    if lease_exists && !lease_expired && !owner_match {
                        reason_codes::LEASE_HELD_BY_OTHER
                    } else {
                        reason_codes::PH1_LEASE_OK_POLICY_EVALUATE
                    }
                }
                LeaseOperation::Renew | LeaseOperation::Release => {
                    if !lease_exists {
                        reason_codes::LEASE_NOT_FOUND
                    } else if !owner_match {
                        reason_codes::LEASE_HELD_BY_OTHER
                    } else if !token_match {
                        reason_codes::LEASE_TOKEN_INVALID
                    } else {
                        reason_codes::PH1_LEASE_OK_POLICY_EVALUATE
                    }
                }
            }
        };

        match LeasePolicyEvaluateOk::v1(
            reason_code,
            req.tenant_id.clone(),
            req.work_order_id.clone(),
            req.lease_owner_id.clone(),
            req.operation,
            lease_exists,
            lease_expired,
            owner_match,
            token_match,
            ttl_in_bounds,
            grant_eligible,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1LeaseResponse::LeasePolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                LeaseCapabilityId::LeasePolicyEvaluate,
                reason_codes::PH1_LEASE_INTERNAL_PIPELINE_ERROR,
                "failed to construct lease policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &LeaseDecisionComputeRequest) -> Ph1LeaseResponse {
        if req.requested_ttl_ms > self.config.max_ttl_ms {
            return self.refuse(
                LeaseCapabilityId::LeaseDecisionCompute,
                reason_codes::PH1_LEASE_BUDGET_EXCEEDED,
                "requested_ttl_ms exceeds runtime max_ttl_ms",
            );
        }

        let default_hold_owner = req.active_lease_owner_id.clone();
        let default_hold_until = req.active_lease_expires_at_ns;

        let (
            reason_code,
            action,
            lease_active_after_decision,
            lease_token,
            lease_expires_at_ns,
            held_by_owner_id,
            held_until_ns,
            resume_from_ledger_required,
        ) = if !req.ttl_in_bounds {
            (
                reason_codes::LEASE_TTL_OUT_OF_BOUNDS,
                LeaseDecisionAction::LeaseDenied,
                false,
                None,
                None,
                default_hold_owner,
                default_hold_until,
                false,
            )
        } else {
            match req.operation {
                LeaseOperation::Acquire => {
                    if req.grant_eligible {
                        let token = req
                            .proposed_lease_token
                            .clone()
                            .unwrap_or_else(|| stable_lease_token(req));
                        let expires_at = MonotonicTimeNs(
                            req.now_ns
                                .0
                                .saturating_add(u64::from(req.requested_ttl_ms) * 1_000_000),
                        );
                        (
                            reason_codes::PH1_LEASE_OK_DECISION_COMPUTE,
                            LeaseDecisionAction::LeaseGranted,
                            true,
                            Some(token),
                            Some(expires_at),
                            None,
                            None,
                            req.lease_expired,
                        )
                    } else if req.lease_exists && !req.owner_match {
                        (
                            reason_codes::LEASE_HELD_BY_OTHER,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    } else {
                        (
                            reason_codes::LEASE_NOT_FOUND,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    }
                }
                LeaseOperation::Renew => {
                    if req.grant_eligible {
                        let token = req
                            .lease_token
                            .clone()
                            .unwrap_or_else(|| stable_lease_token(req));
                        let expires_at = MonotonicTimeNs(
                            req.now_ns
                                .0
                                .saturating_add(u64::from(req.requested_ttl_ms) * 1_000_000),
                        );
                        (
                            reason_codes::PH1_LEASE_OK_DECISION_COMPUTE,
                            LeaseDecisionAction::LeaseGranted,
                            true,
                            Some(token),
                            Some(expires_at),
                            None,
                            None,
                            false,
                        )
                    } else if !req.lease_exists {
                        (
                            reason_codes::LEASE_NOT_FOUND,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            None,
                            None,
                            false,
                        )
                    } else if !req.owner_match {
                        (
                            reason_codes::LEASE_HELD_BY_OTHER,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    } else {
                        (
                            reason_codes::LEASE_TOKEN_INVALID,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    }
                }
                LeaseOperation::Release => {
                    if req.grant_eligible {
                        (
                            reason_codes::PH1_LEASE_OK_DECISION_COMPUTE,
                            LeaseDecisionAction::LeaseGranted,
                            false,
                            None,
                            None,
                            None,
                            None,
                            false,
                        )
                    } else if !req.lease_exists {
                        (
                            reason_codes::LEASE_NOT_FOUND,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            None,
                            None,
                            false,
                        )
                    } else if !req.owner_match {
                        (
                            reason_codes::LEASE_HELD_BY_OTHER,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    } else {
                        (
                            reason_codes::LEASE_TOKEN_INVALID,
                            LeaseDecisionAction::LeaseDenied,
                            false,
                            None,
                            None,
                            default_hold_owner,
                            default_hold_until,
                            false,
                        )
                    }
                }
            }
        };

        match LeaseDecisionComputeOk::v1(
            reason_code,
            req.operation,
            action,
            lease_active_after_decision,
            lease_token,
            lease_expires_at_ns,
            held_by_owner_id,
            held_until_ns,
            resume_from_ledger_required,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1LeaseResponse::LeaseDecisionComputeOk(ok),
            Err(_) => self.refuse(
                LeaseCapabilityId::LeaseDecisionCompute,
                reason_codes::PH1_LEASE_INTERNAL_PIPELINE_ERROR,
                "failed to construct lease decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: LeaseCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1LeaseResponse {
        let out = LeaseRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("LeaseRefuse::v1 must construct for static messages");
        Ph1LeaseResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1LeaseRequest) -> LeaseCapabilityId {
    match req {
        Ph1LeaseRequest::LeasePolicyEvaluate(_) => LeaseCapabilityId::LeasePolicyEvaluate,
        Ph1LeaseRequest::LeaseDecisionCompute(_) => LeaseCapabilityId::LeaseDecisionCompute,
    }
}

fn stable_lease_token(req: &LeaseDecisionComputeRequest) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    let key = format!(
        "{}|{}|{}|{}|{}",
        req.tenant_id.as_str(),
        req.work_order_id.as_str(),
        req.lease_owner_id,
        req.now_ns.0,
        req.requested_ttl_ms
    );
    for byte in key.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(PRIME);
    }
    format!("lease_{:016x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1lease::{LeaseDecisionComputeRequest, LeaseRequestEnvelope};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1work::WorkOrderId;

    fn envelope() -> LeaseRequestEnvelope {
        LeaseRequestEnvelope::v1(CorrelationId(7601), TurnId(8601), 8, 300_000).unwrap()
    }

    fn tenant_id() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        WorkOrderId::new("wo_demo").unwrap()
    }

    fn acquire_policy_request(
        owner_id: &str,
        now_ns: u64,
        active_owner: Option<&str>,
        active_token: Option<&str>,
        active_expires_at_ns: Option<u64>,
    ) -> LeasePolicyEvaluateRequest {
        LeasePolicyEvaluateRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            owner_id.to_string(),
            LeaseOperation::Acquire,
            60_000,
            MonotonicTimeNs(now_ns),
            None,
            active_owner.map(str::to_string),
            active_token.map(str::to_string),
            active_expires_at_ns.map(MonotonicTimeNs),
            Some("idem_1".to_string()),
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_lease_01_one_executor_per_work_order() {
        let runtime = Ph1LeaseRuntime::new(Ph1LeaseConfig::mvp_v1());
        let policy_req = acquire_policy_request(
            "owner_b",
            1_000_000,
            Some("owner_a"),
            Some("token_a"),
            Some(2_000_000),
        );
        let policy_out = runtime.run(&Ph1LeaseRequest::LeasePolicyEvaluate(policy_req));

        let policy_ok = match policy_out {
            Ph1LeaseResponse::LeasePolicyEvaluateOk(ok) => ok,
            _ => panic!("expected lease policy output"),
        };
        assert_eq!(policy_ok.reason_code, reason_codes::LEASE_HELD_BY_OTHER);
        assert!(!policy_ok.grant_eligible);

        let decision_req = LeaseDecisionComputeRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            "owner_b".to_string(),
            LeaseOperation::Acquire,
            60_000,
            MonotonicTimeNs(1_000_000),
            None,
            None,
            policy_ok.lease_exists,
            policy_ok.lease_expired,
            policy_ok.owner_match,
            policy_ok.token_match,
            policy_ok.ttl_in_bounds,
            policy_ok.grant_eligible,
            Some("owner_a".to_string()),
            Some(MonotonicTimeNs(2_000_000)),
            true,
            true,
            true,
        )
        .unwrap();

        let decision_out = runtime.run(&Ph1LeaseRequest::LeaseDecisionCompute(decision_req));
        let decision_ok = match decision_out {
            Ph1LeaseResponse::LeaseDecisionComputeOk(ok) => ok,
            _ => panic!("expected lease decision output"),
        };

        assert_eq!(decision_ok.action, LeaseDecisionAction::LeaseDenied);
        assert_eq!(decision_ok.reason_code, reason_codes::LEASE_HELD_BY_OTHER);
        assert_eq!(decision_ok.held_by_owner_id.as_deref(), Some("owner_a"));
    }

    #[test]
    fn at_lease_02_expired_lease_takeover_requires_ledger_resume() {
        let runtime = Ph1LeaseRuntime::new(Ph1LeaseConfig::mvp_v1());
        let policy_req = acquire_policy_request(
            "owner_b",
            5_000_000,
            Some("owner_a"),
            Some("token_a"),
            Some(4_000_000),
        );
        let policy_out = runtime.run(&Ph1LeaseRequest::LeasePolicyEvaluate(policy_req));

        let policy_ok = match policy_out {
            Ph1LeaseResponse::LeasePolicyEvaluateOk(ok) => ok,
            _ => panic!("expected lease policy output"),
        };
        assert!(policy_ok.grant_eligible);
        assert!(policy_ok.lease_expired);

        let decision_req = LeaseDecisionComputeRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            "owner_b".to_string(),
            LeaseOperation::Acquire,
            60_000,
            MonotonicTimeNs(5_000_000),
            None,
            Some("token_new".to_string()),
            policy_ok.lease_exists,
            policy_ok.lease_expired,
            policy_ok.owner_match,
            policy_ok.token_match,
            policy_ok.ttl_in_bounds,
            policy_ok.grant_eligible,
            Some("owner_a".to_string()),
            Some(MonotonicTimeNs(4_000_000)),
            true,
            true,
            true,
        )
        .unwrap();

        let decision_out = runtime.run(&Ph1LeaseRequest::LeaseDecisionCompute(decision_req));
        let decision_ok = match decision_out {
            Ph1LeaseResponse::LeaseDecisionComputeOk(ok) => ok,
            _ => panic!("expected lease decision output"),
        };

        assert_eq!(decision_ok.action, LeaseDecisionAction::LeaseGranted);
        assert!(decision_ok.lease_active_after_decision);
        assert_eq!(decision_ok.lease_token.as_deref(), Some("token_new"));
        assert!(decision_ok.resume_from_ledger_required);
    }

    #[test]
    fn at_lease_03_renew_release_require_token() {
        let runtime = Ph1LeaseRuntime::new(Ph1LeaseConfig::mvp_v1());

        let invalid_renew_req = Ph1LeaseRequest::LeasePolicyEvaluate(LeasePolicyEvaluateRequest {
            schema_version: selene_kernel_contracts::ph1lease::PH1LEASE_CONTRACT_VERSION,
            envelope: envelope(),
            tenant_id: tenant_id(),
            work_order_id: work_order_id(),
            lease_owner_id: "owner_a".to_string(),
            operation: LeaseOperation::Renew,
            requested_ttl_ms: 60_000,
            now_ns: MonotonicTimeNs(1_000_000),
            lease_token: None,
            active_lease_owner_id: Some("owner_a".to_string()),
            active_lease_token: Some("token_a".to_string()),
            active_lease_expires_at_ns: Some(MonotonicTimeNs(2_000_000)),
            idempotency_key: Some("idem_2".to_string()),
            deterministic_takeover_from_ledger: true,
            one_active_lease_per_work_order: true,
            token_owner_required: true,
        });

        let out = runtime.run(&invalid_renew_req);
        let Ph1LeaseResponse::Refuse(refuse) = out else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_LEASE_INPUT_SCHEMA_INVALID
        );
    }
}
