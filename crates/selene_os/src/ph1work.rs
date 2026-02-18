#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1work::{
    Ph1WorkRequest, Ph1WorkResponse, WorkCapabilityId, WorkDecisionComputeOk,
    WorkDecisionComputeRequest, WorkDecisionStatus, WorkEventType, WorkOrderId,
    WorkPolicyEvaluateOk, WorkPolicyEvaluateRequest, WorkRefuse, WorkRequestEnvelope,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.WORK OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_WORK_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x574B_0101);
    pub const PH1_WORK_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x574B_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1WorkWiringConfig {
    pub work_enabled: bool,
    pub max_payload_bytes: u16,
    pub max_diagnostics: u8,
}

impl Ph1WorkWiringConfig {
    pub fn mvp_v1(work_enabled: bool) -> Self {
        Self {
            work_enabled,
            max_payload_bytes: 2_048,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub event_type: WorkEventType,
    pub payload_min: String,
    pub created_at: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
    pub idempotency_required: bool,
    pub append_only_violation: bool,
    pub tenant_scope_mismatch: bool,
    pub idempotency_duplicate: bool,
    pub existing_event_id_on_duplicate: Option<u64>,
    pub proposed_event_id: Option<u64>,
}

impl WorkTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        event_type: WorkEventType,
        payload_min: String,
        created_at: MonotonicTimeNs,
        idempotency_key: Option<String>,
        idempotency_required: bool,
        append_only_violation: bool,
        tenant_scope_mismatch: bool,
        idempotency_duplicate: bool,
        existing_event_id_on_duplicate: Option<u64>,
        proposed_event_id: Option<u64>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            work_order_id,
            event_type,
            payload_min,
            created_at,
            idempotency_key,
            idempotency_required,
            append_only_violation,
            tenant_scope_mismatch,
            idempotency_duplicate,
            existing_event_id_on_duplicate,
            proposed_event_id,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for WorkTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        if self.payload_min.is_empty() || !self.payload_min.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.payload_min",
                reason: "must be non-empty ASCII",
            });
        }
        if self.payload_min.len() > 16_384 {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.payload_min",
                reason: "must be <= 16384 bytes",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.created_at",
                reason: "must be > 0",
            });
        }
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token("work_turn_input.idempotency_key", idempotency_key, 128)?;
        }
        if self.idempotency_required && self.idempotency_key.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.idempotency_key",
                reason: "must be present when idempotency_required=true",
            });
        }
        if self.idempotency_duplicate && self.existing_event_id_on_duplicate.unwrap_or(0) == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.existing_event_id_on_duplicate",
                reason: "must be present when idempotency_duplicate=true",
            });
        }
        if self.idempotency_duplicate && self.proposed_event_id.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.proposed_event_id",
                reason: "must be absent when idempotency_duplicate=true",
            });
        }
        if !self.idempotency_duplicate
            && !self.append_only_violation
            && !self.tenant_scope_mismatch
            && self.proposed_event_id.unwrap_or(0) == 0
        {
            return Err(ContractViolation::InvalidValue {
                field: "work_turn_input.proposed_event_id",
                reason: "must be present for append path",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: WorkPolicyEvaluateOk,
    pub decision_compute: WorkDecisionComputeOk,
}

impl WorkForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: WorkPolicyEvaluateOk,
        decision_compute: WorkDecisionComputeOk,
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

impl Validate for WorkForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;
        if self.decision_compute.status == WorkDecisionStatus::Ok
            && self.decision_compute.idempotency_no_op
            && !self.policy_evaluate.idempotency_duplicate
        {
            return Err(ContractViolation::InvalidValue {
                field: "work_forward_bundle.decision_compute.idempotency_no_op",
                reason: "idempotency_no_op requires policy idempotency_duplicate=true",
            });
        }
        if self.decision_compute.status == WorkDecisionStatus::Ok
            && !self.decision_compute.idempotency_no_op
            && !self.policy_evaluate.append_allowed
        {
            return Err(ContractViolation::InvalidValue {
                field: "work_forward_bundle.decision_compute.status",
                reason: "OK append path requires policy append_allowed=true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkWiringOutcome {
    NotInvokedDisabled,
    Refused(WorkRefuse),
    Forwarded(WorkForwardBundle),
}

pub trait Ph1WorkEngine {
    fn run(&self, req: &Ph1WorkRequest) -> Ph1WorkResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1WorkWiring<E>
where
    E: Ph1WorkEngine,
{
    config: Ph1WorkWiringConfig,
    engine: E,
}

impl<E> Ph1WorkWiring<E>
where
    E: Ph1WorkEngine,
{
    pub fn new(config: Ph1WorkWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_payload_bytes == 0 || config.max_payload_bytes > 16_384 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1work_wiring_config.max_payload_bytes",
                reason: "must be within 1..=16384",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1work_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &WorkTurnInput) -> Result<WorkWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.work_enabled {
            return Ok(WorkWiringOutcome::NotInvokedDisabled);
        }

        let envelope = WorkRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_payload_bytes, 16_384),
            min(self.config.max_diagnostics, 16),
        )?;

        let policy_req = Ph1WorkRequest::WorkPolicyEvaluate(WorkPolicyEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.work_order_id.clone(),
            input.event_type,
            input.payload_min.clone(),
            input.created_at,
            input.idempotency_key.clone(),
            input.idempotency_required,
            input.append_only_violation,
            input.tenant_scope_mismatch,
            input.idempotency_duplicate,
            true,
            true,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        if policy_resp.validate().is_err() {
            return Ok(WorkWiringOutcome::Refused(WorkRefuse::v1(
                WorkCapabilityId::WorkPolicyEvaluate,
                reason_codes::PH1_WORK_VALIDATION_FAILED,
                "invalid work policy response contract".to_string(),
            )?));
        }

        let policy_ok = match policy_resp {
            Ph1WorkResponse::Refuse(refuse) => return Ok(WorkWiringOutcome::Refused(refuse)),
            Ph1WorkResponse::WorkPolicyEvaluateOk(ok) => ok,
            Ph1WorkResponse::WorkDecisionComputeOk(_) => {
                return Ok(WorkWiringOutcome::Refused(WorkRefuse::v1(
                    WorkCapabilityId::WorkPolicyEvaluate,
                    reason_codes::PH1_WORK_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1WorkRequest::WorkDecisionCompute(WorkDecisionComputeRequest::v1(
            envelope,
            policy_ok.tenant_id.clone(),
            policy_ok.work_order_id.clone(),
            policy_ok.event_type,
            policy_ok.event_valid,
            policy_ok.append_allowed,
            policy_ok.idempotency_duplicate,
            policy_ok.append_only_violation,
            policy_ok.tenant_scope_mismatch,
            input.existing_event_id_on_duplicate,
            input.proposed_event_id,
            policy_ok.deterministic_replay_order,
            policy_ok.no_silent_conflict_merge,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        if decision_resp.validate().is_err() {
            return Ok(WorkWiringOutcome::Refused(WorkRefuse::v1(
                WorkCapabilityId::WorkDecisionCompute,
                reason_codes::PH1_WORK_VALIDATION_FAILED,
                "invalid work decision response contract".to_string(),
            )?));
        }

        let decision_ok = match decision_resp {
            Ph1WorkResponse::Refuse(refuse) => return Ok(WorkWiringOutcome::Refused(refuse)),
            Ph1WorkResponse::WorkDecisionComputeOk(ok) => ok,
            Ph1WorkResponse::WorkPolicyEvaluateOk(_) => {
                return Ok(WorkWiringOutcome::Refused(WorkRefuse::v1(
                    WorkCapabilityId::WorkDecisionCompute,
                    reason_codes::PH1_WORK_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision request".to_string(),
                )?));
            }
        };

        let bundle =
            WorkForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(WorkWiringOutcome::Forwarded(bundle))
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct MockWorkEngine {
        policy_response: Ph1WorkResponse,
        decision_response: Ph1WorkResponse,
    }

    impl Ph1WorkEngine for MockWorkEngine {
        fn run(&self, req: &Ph1WorkRequest) -> Ph1WorkResponse {
            match req {
                Ph1WorkRequest::WorkPolicyEvaluate(_) => self.policy_response.clone(),
                Ph1WorkRequest::WorkDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    fn base_input() -> WorkTurnInput {
        WorkTurnInput::v1(
            CorrelationId(7401),
            TurnId(8401),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            WorkEventType::StepStarted,
            "{\"step\":\"dispatch\"}".to_string(),
            MonotonicTimeNs(10_000),
            Some("idem_1".to_string()),
            true,
            false,
            false,
            false,
            None,
            Some(901),
        )
        .unwrap()
    }

    fn policy_ok_append() -> WorkPolicyEvaluateOk {
        WorkPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            WorkEventType::StepStarted,
            "payload_hash_1".to_string(),
            true,
            true,
            false,
            false,
            false,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok_append() -> WorkDecisionComputeOk {
        WorkDecisionComputeOk::v1(
            ReasonCodeId(2),
            WorkDecisionStatus::Ok,
            Some(901),
            false,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_work_01_disabled_returns_not_invoked() {
        let engine = MockWorkEngine {
            policy_response: Ph1WorkResponse::WorkPolicyEvaluateOk(policy_ok_append()),
            decision_response: Ph1WorkResponse::WorkDecisionComputeOk(decision_ok_append()),
        };
        let wiring = Ph1WorkWiring::new(Ph1WorkWiringConfig::mvp_v1(false), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        assert!(matches!(out, WorkWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_work_02_policy_refuse_propagates() {
        let refuse = WorkRefuse::v1(
            WorkCapabilityId::WorkPolicyEvaluate,
            ReasonCodeId(100),
            "policy blocked".to_string(),
        )
        .unwrap();
        let engine = MockWorkEngine {
            policy_response: Ph1WorkResponse::Refuse(refuse.clone()),
            decision_response: Ph1WorkResponse::WorkDecisionComputeOk(decision_ok_append()),
        };
        let wiring = Ph1WorkWiring::new(Ph1WorkWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let WorkWiringOutcome::Refused(refused) = out else {
            panic!("expected refuse");
        };
        assert_eq!(refused.reason_code, refuse.reason_code);
    }

    #[test]
    fn at_work_03_forwarded_duplicate_no_op_bundle_is_valid() {
        let policy = WorkPolicyEvaluateOk::v1(
            ReasonCodeId(3),
            TenantId::new("tenant_demo").unwrap(),
            WorkOrderId::new("wo_demo").unwrap(),
            WorkEventType::StepFailed,
            "payload_hash_2".to_string(),
            true,
            false,
            true,
            false,
            false,
            true,
            true,
        )
        .unwrap();
        let decision = WorkDecisionComputeOk::v1(
            ReasonCodeId(4),
            WorkDecisionStatus::Ok,
            Some(777),
            true,
            true,
            true,
        )
        .unwrap();
        let engine = MockWorkEngine {
            policy_response: Ph1WorkResponse::WorkPolicyEvaluateOk(policy),
            decision_response: Ph1WorkResponse::WorkDecisionComputeOk(decision),
        };
        let mut input = base_input();
        input.idempotency_duplicate = true;
        input.existing_event_id_on_duplicate = Some(777);
        input.proposed_event_id = None;

        let wiring = Ph1WorkWiring::new(Ph1WorkWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let WorkWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded");
        };
        assert!(bundle.decision_compute.idempotency_no_op);
        assert_eq!(bundle.decision_compute.work_order_event_id, Some(777));
    }

    #[test]
    fn at_work_04_fail_closed_on_unexpected_response_variant() {
        let engine = MockWorkEngine {
            policy_response: Ph1WorkResponse::WorkPolicyEvaluateOk(policy_ok_append()),
            decision_response: Ph1WorkResponse::WorkPolicyEvaluateOk(policy_ok_append()),
        };
        let wiring = Ph1WorkWiring::new(Ph1WorkWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let WorkWiringOutcome::Refused(refused) = out else {
            panic!("expected refused");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_WORK_INTERNAL_PIPELINE_ERROR
        );
    }
}
