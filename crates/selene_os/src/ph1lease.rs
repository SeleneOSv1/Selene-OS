#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1lease::{
    LeaseCapabilityId, LeaseDecisionAction, LeaseDecisionComputeOk, LeaseDecisionComputeRequest,
    LeaseOperation, LeasePolicyEvaluateOk, LeasePolicyEvaluateRequest, LeaseRefuse,
    LeaseRequestEnvelope, Ph1LeaseRequest, Ph1LeaseResponse,
};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1work::WorkOrderId;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LEASE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LEASE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C53_0101);
    pub const PH1_LEASE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C53_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LeaseWiringConfig {
    pub lease_enabled: bool,
    pub max_ttl_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1LeaseWiringConfig {
    pub fn mvp_v1(lease_enabled: bool) -> Self {
        Self {
            lease_enabled,
            max_ttl_ms: 300_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub lease_owner_id: String,
    pub operation: LeaseOperation,
    pub requested_ttl_ms: u32,
    pub now_ns: MonotonicTimeNs,
    pub lease_token: Option<String>,
    pub proposed_lease_token: Option<String>,
    pub active_lease_owner_id: Option<String>,
    pub active_lease_token: Option<String>,
    pub active_lease_expires_at_ns: Option<MonotonicTimeNs>,
    pub idempotency_key: Option<String>,
}

impl LeaseTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        lease_owner_id: String,
        operation: LeaseOperation,
        requested_ttl_ms: u32,
        now_ns: MonotonicTimeNs,
        lease_token: Option<String>,
        proposed_lease_token: Option<String>,
        active_lease_owner_id: Option<String>,
        active_lease_token: Option<String>,
        active_lease_expires_at_ns: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            work_order_id,
            lease_owner_id,
            operation,
            requested_ttl_ms,
            now_ns,
            lease_token,
            proposed_lease_token,
            active_lease_owner_id,
            active_lease_token,
            active_lease_expires_at_ns,
            idempotency_key,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for LeaseTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;

        validate_token("lease_turn_input.lease_owner_id", &self.lease_owner_id, 128)?;
        validate_opt_token("lease_turn_input.lease_token", &self.lease_token, 192)?;
        validate_opt_token(
            "lease_turn_input.proposed_lease_token",
            &self.proposed_lease_token,
            192,
        )?;
        validate_opt_token(
            "lease_turn_input.active_lease_owner_id",
            &self.active_lease_owner_id,
            128,
        )?;
        validate_opt_token(
            "lease_turn_input.active_lease_token",
            &self.active_lease_token,
            192,
        )?;
        validate_opt_token(
            "lease_turn_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;

        if self.requested_ttl_ms == 0 || self.requested_ttl_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_turn_input.requested_ttl_ms",
                reason: "must be within 1..=3_600_000",
            });
        }
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_turn_input.now_ns",
                reason: "must be > 0",
            });
        }

        if matches!(
            self.operation,
            LeaseOperation::Renew | LeaseOperation::Release
        ) && self.lease_token.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_turn_input.lease_token",
                reason: "must be present when operation is RENEW or RELEASE",
            });
        }

        let active_fields_count = usize::from(self.active_lease_owner_id.is_some())
            + usize::from(self.active_lease_token.is_some())
            + usize::from(self.active_lease_expires_at_ns.is_some());
        if active_fields_count != 0 && active_fields_count != 3 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_turn_input.active_lease_owner_id",
                reason: "active lease snapshot must include owner/token/expires together",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: LeasePolicyEvaluateOk,
    pub decision_compute: LeaseDecisionComputeOk,
}

impl LeaseForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: LeasePolicyEvaluateOk,
        decision_compute: LeaseDecisionComputeOk,
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

impl Validate for LeaseForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;

        if self.decision_compute.action == LeaseDecisionAction::LeaseGranted
            && !self.policy_evaluate.grant_eligible
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_forward_bundle.decision_compute.action",
                reason: "grant decision requires grant_eligible=true in policy",
            });
        }
        if self.decision_compute.resume_from_ledger_required && !self.policy_evaluate.lease_expired
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_forward_bundle.decision_compute.resume_from_ledger_required",
                reason: "resume_from_ledger_required requires policy lease_expired=true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeaseWiringOutcome {
    NotInvokedDisabled,
    Refused(LeaseRefuse),
    Forwarded(LeaseForwardBundle),
}

pub trait Ph1LeaseEngine {
    fn run(&self, req: &Ph1LeaseRequest) -> Ph1LeaseResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1LeaseWiring<E>
where
    E: Ph1LeaseEngine,
{
    config: Ph1LeaseWiringConfig,
    engine: E,
}

impl<E> Ph1LeaseWiring<E>
where
    E: Ph1LeaseEngine,
{
    pub fn new(config: Ph1LeaseWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_ttl_ms == 0 || config.max_ttl_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1lease_wiring_config.max_ttl_ms",
                reason: "must be within 1..=3_600_000",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1lease_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &LeaseTurnInput,
    ) -> Result<LeaseWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.lease_enabled {
            return Ok(LeaseWiringOutcome::NotInvokedDisabled);
        }

        let envelope = LeaseRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_diagnostics, 16),
            min(self.config.max_ttl_ms, 3_600_000),
        )?;

        let policy_req = Ph1LeaseRequest::LeasePolicyEvaluate(LeasePolicyEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.work_order_id.clone(),
            input.lease_owner_id.clone(),
            input.operation,
            input.requested_ttl_ms,
            input.now_ns,
            input.lease_token.clone(),
            input.active_lease_owner_id.clone(),
            input.active_lease_token.clone(),
            input.active_lease_expires_at_ns,
            input.idempotency_key.clone(),
            true,
            true,
            true,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        if policy_resp.validate().is_err() {
            return Ok(LeaseWiringOutcome::Refused(LeaseRefuse::v1(
                LeaseCapabilityId::LeasePolicyEvaluate,
                reason_codes::PH1_LEASE_VALIDATION_FAILED,
                "invalid lease policy response contract".to_string(),
            )?));
        }

        let policy_ok = match policy_resp {
            Ph1LeaseResponse::Refuse(refuse) => return Ok(LeaseWiringOutcome::Refused(refuse)),
            Ph1LeaseResponse::LeasePolicyEvaluateOk(ok) => ok,
            Ph1LeaseResponse::LeaseDecisionComputeOk(_) => {
                return Ok(LeaseWiringOutcome::Refused(LeaseRefuse::v1(
                    LeaseCapabilityId::LeasePolicyEvaluate,
                    reason_codes::PH1_LEASE_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1LeaseRequest::LeaseDecisionCompute(LeaseDecisionComputeRequest::v1(
            envelope,
            input.tenant_id.clone(),
            input.work_order_id.clone(),
            input.lease_owner_id.clone(),
            input.operation,
            input.requested_ttl_ms,
            input.now_ns,
            input.lease_token.clone(),
            input.proposed_lease_token.clone(),
            policy_ok.lease_exists,
            policy_ok.lease_expired,
            policy_ok.owner_match,
            policy_ok.token_match,
            policy_ok.ttl_in_bounds,
            policy_ok.grant_eligible,
            input.active_lease_owner_id.clone(),
            input.active_lease_expires_at_ns,
            policy_ok.deterministic_takeover_from_ledger,
            policy_ok.one_active_lease_per_work_order,
            policy_ok.token_owner_required,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        if decision_resp.validate().is_err() {
            return Ok(LeaseWiringOutcome::Refused(LeaseRefuse::v1(
                LeaseCapabilityId::LeaseDecisionCompute,
                reason_codes::PH1_LEASE_VALIDATION_FAILED,
                "invalid lease decision response contract".to_string(),
            )?));
        }

        let decision_ok = match decision_resp {
            Ph1LeaseResponse::Refuse(refuse) => return Ok(LeaseWiringOutcome::Refused(refuse)),
            Ph1LeaseResponse::LeaseDecisionComputeOk(ok) => ok,
            Ph1LeaseResponse::LeasePolicyEvaluateOk(_) => {
                return Ok(LeaseWiringOutcome::Refused(LeaseRefuse::v1(
                    LeaseCapabilityId::LeaseDecisionCompute,
                    reason_codes::PH1_LEASE_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision request".to_string(),
                )?));
            }
        };

        let bundle =
            LeaseForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(LeaseWiringOutcome::Forwarded(bundle))
    }
}

fn validate_token(
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

fn validate_opt_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_token(field, v, max_len)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct MockLeaseEngine {
        policy_response: Ph1LeaseResponse,
        decision_response: Ph1LeaseResponse,
    }

    impl Ph1LeaseEngine for MockLeaseEngine {
        fn run(&self, req: &Ph1LeaseRequest) -> Ph1LeaseResponse {
            match req {
                Ph1LeaseRequest::LeasePolicyEvaluate(_) => self.policy_response.clone(),
                Ph1LeaseRequest::LeaseDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    fn base_input() -> LeaseTurnInput {
        LeaseTurnInput::v1(
            CorrelationId(7701),
            TurnId(8701),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            "owner_a".to_string(),
            LeaseOperation::Acquire,
            60_000,
            MonotonicTimeNs(10_000),
            None,
            Some("token_new".to_string()),
            None,
            None,
            None,
            Some("idem_1".to_string()),
        )
        .unwrap()
    }

    fn policy_ok_granted() -> LeasePolicyEvaluateOk {
        LeasePolicyEvaluateOk::v1(
            ReasonCodeId(1),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            "owner_a".to_string(),
            LeaseOperation::Acquire,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok_granted() -> LeaseDecisionComputeOk {
        LeaseDecisionComputeOk::v1(
            ReasonCodeId(2),
            LeaseOperation::Acquire,
            LeaseDecisionAction::LeaseGranted,
            true,
            Some("token_new".to_string()),
            Some(MonotonicTimeNs(90_000)),
            None,
            None,
            false,
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_lease_wiring_01_disabled_returns_not_invoked() {
        let engine = MockLeaseEngine {
            policy_response: Ph1LeaseResponse::LeasePolicyEvaluateOk(policy_ok_granted()),
            decision_response: Ph1LeaseResponse::LeaseDecisionComputeOk(decision_ok_granted()),
        };
        let wiring = Ph1LeaseWiring::new(Ph1LeaseWiringConfig::mvp_v1(false), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        assert!(matches!(out, LeaseWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_lease_wiring_02_policy_refuse_propagates() {
        let refuse = LeaseRefuse::v1(
            LeaseCapabilityId::LeasePolicyEvaluate,
            ReasonCodeId(100),
            "policy blocked".to_string(),
        )
        .unwrap();
        let engine = MockLeaseEngine {
            policy_response: Ph1LeaseResponse::Refuse(refuse.clone()),
            decision_response: Ph1LeaseResponse::LeaseDecisionComputeOk(decision_ok_granted()),
        };
        let wiring = Ph1LeaseWiring::new(Ph1LeaseWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let LeaseWiringOutcome::Refused(refused) = out else {
            panic!("expected refuse");
        };
        assert_eq!(refused.reason_code, refuse.reason_code);
    }

    #[test]
    fn at_lease_wiring_03_forwarded_expired_takeover_bundle_is_valid() {
        let policy = LeasePolicyEvaluateOk::v1(
            ReasonCodeId(3),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            "owner_b".to_string(),
            LeaseOperation::Acquire,
            true,
            true,
            false,
            false,
            true,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        let decision = LeaseDecisionComputeOk::v1(
            ReasonCodeId(4),
            LeaseOperation::Acquire,
            LeaseDecisionAction::LeaseGranted,
            true,
            Some("token_takeover".to_string()),
            Some(MonotonicTimeNs(120_000)),
            None,
            None,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        let engine = MockLeaseEngine {
            policy_response: Ph1LeaseResponse::LeasePolicyEvaluateOk(policy),
            decision_response: Ph1LeaseResponse::LeaseDecisionComputeOk(decision),
        };
        let mut input = base_input();
        input.lease_owner_id = "owner_b".to_string();
        input.active_lease_owner_id = Some("owner_a".to_string());
        input.active_lease_token = Some("token_old".to_string());
        input.active_lease_expires_at_ns = Some(MonotonicTimeNs(9_000));

        let wiring = Ph1LeaseWiring::new(Ph1LeaseWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let LeaseWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded");
        };
        assert!(bundle.decision_compute.resume_from_ledger_required);
        assert_eq!(
            bundle.decision_compute.action,
            LeaseDecisionAction::LeaseGranted
        );
    }

    #[test]
    fn at_lease_wiring_04_fail_closed_on_unexpected_response_variant() {
        let engine = MockLeaseEngine {
            policy_response: Ph1LeaseResponse::LeasePolicyEvaluateOk(policy_ok_granted()),
            decision_response: Ph1LeaseResponse::LeasePolicyEvaluateOk(policy_ok_granted()),
        };
        let wiring = Ph1LeaseWiring::new(Ph1LeaseWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let LeaseWiringOutcome::Refused(refused) = out else {
            panic!("expected refused");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_LEASE_INTERNAL_PIPELINE_ERROR
        );
    }
}
