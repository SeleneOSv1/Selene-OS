#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::ph1work::WorkOrderId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SCHED_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedCapabilityId {
    SchedPolicyEvaluate,
    SchedDecisionCompute,
}

impl SchedCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            SchedCapabilityId::SchedPolicyEvaluate => "SCHED_POLICY_EVALUATE",
            SchedCapabilityId::SchedDecisionCompute => "SCHED_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedDecisionAction {
    RetryAt,
    Fail,
    Wait,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_retryable_reason_codes: u8,
    pub max_diagnostics: u8,
    pub max_backoff_ms: u32,
}

impl SchedRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_retryable_reason_codes: u8,
        max_diagnostics: u8,
        max_backoff_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_retryable_reason_codes,
            max_diagnostics,
            max_backoff_ms,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for SchedRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_request_envelope.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_retryable_reason_codes == 0 || self.max_retryable_reason_codes > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_request_envelope.max_retryable_reason_codes",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if self.max_backoff_ms == 0 || self.max_backoff_ms > 86_400_000 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_request_envelope.max_backoff_ms",
                reason: "must be within 1..=86_400_000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SchedRequestEnvelope,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub step_id: String,
    pub now_ns: MonotonicTimeNs,
    pub step_started_at_ns: MonotonicTimeNs,
    pub timeout_ms: u32,
    pub max_retries: u16,
    pub retry_backoff_ms: u32,
    pub attempt_index: u16,
    pub last_failure_reason_code: Option<ReasonCodeId>,
    pub retryable_reason_codes: Vec<ReasonCodeId>,
    pub wait_is_pause_only: bool,
}

impl SchedPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: SchedRequestEnvelope,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        step_id: String,
        now_ns: MonotonicTimeNs,
        step_started_at_ns: MonotonicTimeNs,
        timeout_ms: u32,
        max_retries: u16,
        retry_backoff_ms: u32,
        attempt_index: u16,
        last_failure_reason_code: Option<ReasonCodeId>,
        retryable_reason_codes: Vec<ReasonCodeId>,
        wait_is_pause_only: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            step_id,
            now_ns,
            step_started_at_ns,
            timeout_ms,
            max_retries,
            retry_backoff_ms,
            attempt_index,
            last_failure_reason_code,
            retryable_reason_codes,
            wait_is_pause_only,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for SchedPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token("sched_policy_evaluate_request.step_id", &self.step_id, 96)?;

        if self.now_ns.0 == 0 || self.step_started_at_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.now_ns",
                reason: "time values must be > 0",
            });
        }
        if self.now_ns.0 < self.step_started_at_ns.0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.now_ns",
                reason: "must be >= step_started_at_ns",
            });
        }
        if self.timeout_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.timeout_ms",
                reason: "must be > 0",
            });
        }
        if self.max_retries > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.max_retries",
                reason: "must be <= 1024",
            });
        }
        if self.retry_backoff_ms == 0 || self.retry_backoff_ms > self.envelope.max_backoff_ms {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.retry_backoff_ms",
                reason: "must be within 1..=envelope.max_backoff_ms",
            });
        }
        if self.retryable_reason_codes.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.retryable_reason_codes",
                reason: "must not be empty",
            });
        }
        if self.retryable_reason_codes.len() > self.envelope.max_retryable_reason_codes as usize {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.retryable_reason_codes",
                reason: "exceeds envelope max_retryable_reason_codes",
            });
        }
        let mut reason_set = BTreeSet::new();
        for reason in &self.retryable_reason_codes {
            if reason.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_policy_evaluate_request.retryable_reason_codes",
                    reason: "must contain non-zero reason codes",
                });
            }
            if !reason_set.insert(reason.0) {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_policy_evaluate_request.retryable_reason_codes",
                    reason: "must be unique",
                });
            }
        }
        if let Some(reason) = self.last_failure_reason_code {
            if reason.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_policy_evaluate_request.last_failure_reason_code",
                    reason: "must be non-zero when present",
                });
            }
        }
        if !self.wait_is_pause_only {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_request.wait_is_pause_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SchedCapabilityId,
    pub reason_code: ReasonCodeId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub step_id: String,
    pub attempt_index: u16,
    pub next_attempt_index: u16,
    pub timeout_exceeded: bool,
    pub max_retries_reached: bool,
    pub retry_allowed: bool,
    pub retry_backoff_ms: u32,
    pub wait_is_pause_only: bool,
    pub deterministic: bool,
}

impl SchedPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        step_id: String,
        attempt_index: u16,
        next_attempt_index: u16,
        timeout_exceeded: bool,
        max_retries_reached: bool,
        retry_allowed: bool,
        retry_backoff_ms: u32,
        wait_is_pause_only: bool,
        deterministic: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            capability_id: SchedCapabilityId::SchedPolicyEvaluate,
            reason_code,
            tenant_id,
            work_order_id,
            step_id,
            attempt_index,
            next_attempt_index,
            timeout_exceeded,
            max_retries_reached,
            retry_allowed,
            retry_backoff_ms,
            wait_is_pause_only,
            deterministic,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SchedPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SchedCapabilityId::SchedPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.capability_id",
                reason: "must be SCHED_POLICY_EVALUATE",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token("sched_policy_evaluate_ok.step_id", &self.step_id, 96)?;
        if self.retry_backoff_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.retry_backoff_ms",
                reason: "must be > 0",
            });
        }
        if !self.wait_is_pause_only {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.wait_is_pause_only",
                reason: "must be true",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.deterministic",
                reason: "must be true",
            });
        }

        if self.retry_allowed {
            if self.timeout_exceeded || self.max_retries_reached {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_policy_evaluate_ok.retry_allowed",
                    reason: "cannot be true when timeout/max-retries guard fired",
                });
            }
            if self.next_attempt_index != self.attempt_index.saturating_add(1) {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_policy_evaluate_ok.next_attempt_index",
                    reason: "must advance by exactly 1 when retry_allowed=true",
                });
            }
        } else if self.next_attempt_index != self.attempt_index {
            return Err(ContractViolation::InvalidValue {
                field: "sched_policy_evaluate_ok.next_attempt_index",
                reason: "must not advance when retry_allowed=false",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SchedRequestEnvelope,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub step_id: String,
    pub now_ns: MonotonicTimeNs,
    pub attempt_index: u16,
    pub next_attempt_index: u16,
    pub retry_backoff_ms: u32,
    pub timeout_exceeded: bool,
    pub max_retries_reached: bool,
    pub retry_allowed: bool,
    pub wait_is_pause_only: bool,
}

impl SchedDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: SchedRequestEnvelope,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        step_id: String,
        now_ns: MonotonicTimeNs,
        attempt_index: u16,
        next_attempt_index: u16,
        retry_backoff_ms: u32,
        timeout_exceeded: bool,
        max_retries_reached: bool,
        retry_allowed: bool,
        wait_is_pause_only: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            step_id,
            now_ns,
            attempt_index,
            next_attempt_index,
            retry_backoff_ms,
            timeout_exceeded,
            max_retries_reached,
            retry_allowed,
            wait_is_pause_only,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for SchedDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_request.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token("sched_decision_compute_request.step_id", &self.step_id, 96)?;
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_request.now_ns",
                reason: "must be > 0",
            });
        }
        if self.retry_backoff_ms == 0 || self.retry_backoff_ms > self.envelope.max_backoff_ms {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_request.retry_backoff_ms",
                reason: "must be within 1..=envelope.max_backoff_ms",
            });
        }
        if !self.wait_is_pause_only {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_request.wait_is_pause_only",
                reason: "must be true",
            });
        }

        if self.retry_allowed {
            if self.timeout_exceeded || self.max_retries_reached {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_decision_compute_request.retry_allowed",
                    reason: "cannot be true when timeout/max-retries guard fired",
                });
            }
            if self.next_attempt_index != self.attempt_index.saturating_add(1) {
                return Err(ContractViolation::InvalidValue {
                    field: "sched_decision_compute_request.next_attempt_index",
                    reason: "must advance by exactly 1 when retry_allowed=true",
                });
            }
        } else if self.next_attempt_index != self.attempt_index {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_request.next_attempt_index",
                reason: "must not advance when retry_allowed=false",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SchedCapabilityId,
    pub reason_code: ReasonCodeId,
    pub action: SchedDecisionAction,
    pub attempt_index: u16,
    pub attempt_next_index: u16,
    pub next_due_at_ns: Option<MonotonicTimeNs>,
    pub wait_is_pause_only: bool,
    pub deterministic: bool,
}

impl SchedDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        action: SchedDecisionAction,
        attempt_index: u16,
        attempt_next_index: u16,
        next_due_at_ns: Option<MonotonicTimeNs>,
        wait_is_pause_only: bool,
        deterministic: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            capability_id: SchedCapabilityId::SchedDecisionCompute,
            reason_code,
            action,
            attempt_index,
            attempt_next_index,
            next_due_at_ns,
            wait_is_pause_only,
            deterministic,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SchedDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_ok.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SchedCapabilityId::SchedDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_ok.capability_id",
                reason: "must be SCHED_DECISION_COMPUTE",
            });
        }
        if !self.wait_is_pause_only {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_ok.wait_is_pause_only",
                reason: "must be true",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "sched_decision_compute_ok.deterministic",
                reason: "must be true",
            });
        }

        match self.action {
            SchedDecisionAction::RetryAt => {
                let next_due_at_ns =
                    self.next_due_at_ns.ok_or(ContractViolation::InvalidValue {
                        field: "sched_decision_compute_ok.next_due_at_ns",
                        reason: "must be present when action=RETRY_AT",
                    })?;
                if next_due_at_ns.0 == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "sched_decision_compute_ok.next_due_at_ns",
                        reason: "must be > 0 when action=RETRY_AT",
                    });
                }
                if self.attempt_next_index != self.attempt_index.saturating_add(1) {
                    return Err(ContractViolation::InvalidValue {
                        field: "sched_decision_compute_ok.attempt_next_index",
                        reason: "must advance by 1 when action=RETRY_AT",
                    });
                }
            }
            SchedDecisionAction::Fail | SchedDecisionAction::Wait => {
                if self.next_due_at_ns.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "sched_decision_compute_ok.next_due_at_ns",
                        reason: "must be absent when action is FAIL or WAIT",
                    });
                }
                if self.attempt_next_index != self.attempt_index {
                    return Err(ContractViolation::InvalidValue {
                        field: "sched_decision_compute_ok.attempt_next_index",
                        reason: "must not advance when action is FAIL or WAIT",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchedRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: SchedCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl SchedRefuse {
    pub fn v1(
        capability_id: SchedCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1SCHED_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SchedRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SCHED_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "sched_refuse.schema_version",
                reason: "must match PH1SCHED_CONTRACT_VERSION",
            });
        }
        validate_text("sched_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SchedRequest {
    SchedPolicyEvaluate(SchedPolicyEvaluateRequest),
    SchedDecisionCompute(SchedDecisionComputeRequest),
}

impl Validate for Ph1SchedRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SchedRequest::SchedPolicyEvaluate(req) => req.validate(),
            Ph1SchedRequest::SchedDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SchedResponse {
    SchedPolicyEvaluateOk(SchedPolicyEvaluateOk),
    SchedDecisionComputeOk(SchedDecisionComputeOk),
    Refuse(SchedRefuse),
}

impl Validate for Ph1SchedResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SchedResponse::SchedPolicyEvaluateOk(out) => out.validate(),
            Ph1SchedResponse::SchedDecisionComputeOk(out) => out.validate(),
            Ph1SchedResponse::Refuse(out) => out.validate(),
        }
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

fn validate_text(
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> SchedRequestEnvelope {
        SchedRequestEnvelope::v1(CorrelationId(4101), TurnId(5101), 8, 8, 300_000).unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        WorkOrderId::new("wo_123").unwrap()
    }

    #[test]
    fn at_sched_01_retry_action_requires_next_due() {
        let out = SchedDecisionComputeOk::v1(
            ReasonCodeId(1),
            SchedDecisionAction::RetryAt,
            1,
            2,
            None,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_sched_02_wait_action_must_not_advance_attempt() {
        let out = SchedDecisionComputeOk::v1(
            ReasonCodeId(1),
            SchedDecisionAction::Wait,
            2,
            3,
            None,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_sched_03_retryable_reason_codes_must_be_unique() {
        let req = SchedPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            work_order_id(),
            "step_1".to_string(),
            MonotonicTimeNs(2_000_000_000),
            MonotonicTimeNs(1_000_000_000),
            5000,
            3,
            1000,
            1,
            Some(ReasonCodeId(9)),
            vec![ReasonCodeId(9), ReasonCodeId(9)],
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_sched_04_wait_pause_rule_is_mandatory() {
        let req = SchedPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            work_order_id(),
            "step_1".to_string(),
            MonotonicTimeNs(2_000_000_000),
            MonotonicTimeNs(1_000_000_000),
            5000,
            3,
            1000,
            1,
            Some(ReasonCodeId(9)),
            vec![ReasonCodeId(9)],
            false,
        );
        assert!(req.is_err());
    }
}
