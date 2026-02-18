#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1QUOTA_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotaCapabilityId {
    QuotaPolicyEvaluate,
    QuotaDecisionCompute,
}

impl QuotaCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            QuotaCapabilityId::QuotaPolicyEvaluate => "QUOTA_POLICY_EVALUATE",
            QuotaCapabilityId::QuotaDecisionCompute => "QUOTA_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotaOperationKind {
    Stt,
    Tts,
    Tool,
    Simulation,
    Export,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotaThrottleCause {
    None,
    RateLimit,
    BudgetExceeded,
    PolicyBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuotaDecisionAction {
    Allow,
    Wait,
    Refuse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_diagnostics: u8,
    pub max_wait_ms: u32,
}

impl QuotaRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_diagnostics: u8,
        max_wait_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_diagnostics,
            max_wait_ms,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for QuotaRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_request_envelope.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "quota_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if self.max_wait_ms == 0 || self.max_wait_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "quota_request_envelope.max_wait_ms",
                reason: "must be within 1..=3_600_000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: QuotaRequestEnvelope,
    pub tenant_id: TenantId,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub operation_kind: QuotaOperationKind,
    pub capability_id: Option<String>,
    pub tool_name: Option<String>,
    pub now_ns: MonotonicTimeNs,
    pub cost_hint_microunits: Option<u64>,
    pub rate_limit_exceeded: bool,
    pub budget_exceeded: bool,
    pub policy_blocked: bool,
    pub wait_permitted: bool,
    pub suggested_wait_ms: Option<u32>,
    pub deterministic: bool,
    pub no_authority_grant: bool,
    pub no_gate_order_change: bool,
}

impl QuotaPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: QuotaRequestEnvelope,
        tenant_id: TenantId,
        user_id: Option<String>,
        device_id: Option<String>,
        operation_kind: QuotaOperationKind,
        capability_id: Option<String>,
        tool_name: Option<String>,
        now_ns: MonotonicTimeNs,
        cost_hint_microunits: Option<u64>,
        rate_limit_exceeded: bool,
        budget_exceeded: bool,
        policy_blocked: bool,
        wait_permitted: bool,
        suggested_wait_ms: Option<u32>,
        deterministic: bool,
        no_authority_grant: bool,
        no_gate_order_change: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            envelope,
            tenant_id,
            user_id,
            device_id,
            operation_kind,
            capability_id,
            tool_name,
            now_ns,
            cost_hint_microunits,
            rate_limit_exceeded,
            budget_exceeded,
            policy_blocked,
            wait_permitted,
            suggested_wait_ms,
            deterministic,
            no_authority_grant,
            no_gate_order_change,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for QuotaPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;

        if let Some(user_id) = &self.user_id {
            validate_token("quota_policy_evaluate_request.user_id", user_id, 96)?;
        }
        if let Some(device_id) = &self.device_id {
            validate_token("quota_policy_evaluate_request.device_id", device_id, 96)?;
        }
        if let Some(capability_id) = &self.capability_id {
            validate_token(
                "quota_policy_evaluate_request.capability_id",
                capability_id,
                128,
            )?;
        }
        if let Some(tool_name) = &self.tool_name {
            validate_token("quota_policy_evaluate_request.tool_name", tool_name, 96)?;
        }

        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.now_ns",
                reason: "must be > 0",
            });
        }
        if let Some(cost_hint_microunits) = self.cost_hint_microunits {
            if cost_hint_microunits == 0 || cost_hint_microunits > 1_000_000_000_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_policy_evaluate_request.cost_hint_microunits",
                    reason: "must be within 1..=1_000_000_000_000 when present",
                });
            }
        }

        match self.operation_kind {
            QuotaOperationKind::Tool => {
                if self.tool_name.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_policy_evaluate_request.tool_name",
                        reason: "must be present when operation_kind=TOOL",
                    });
                }
                if self.capability_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_policy_evaluate_request.capability_id",
                        reason: "must be absent when operation_kind=TOOL",
                    });
                }
            }
            QuotaOperationKind::Stt
            | QuotaOperationKind::Tts
            | QuotaOperationKind::Simulation
            | QuotaOperationKind::Export => {
                if self.capability_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_policy_evaluate_request.capability_id",
                        reason: "must be present when operation_kind is not TOOL",
                    });
                }
                if self.tool_name.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_policy_evaluate_request.tool_name",
                        reason: "must be absent when operation_kind is not TOOL",
                    });
                }
            }
        }

        if self.policy_blocked && self.wait_permitted {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.wait_permitted",
                reason: "must be false when policy_blocked=true",
            });
        }

        if let Some(suggested_wait_ms) = self.suggested_wait_ms {
            if suggested_wait_ms == 0 || suggested_wait_ms > self.envelope.max_wait_ms {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_policy_evaluate_request.suggested_wait_ms",
                    reason: "must be within 1..=envelope.max_wait_ms when present",
                });
            }
        }

        if (self.rate_limit_exceeded || self.budget_exceeded)
            && self.wait_permitted
            && self.suggested_wait_ms.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.suggested_wait_ms",
                reason: "must be present when wait_permitted=true for exceeded quota",
            });
        }

        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_authority_grant {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.no_authority_grant",
                reason: "must be true",
            });
        }
        if !self.no_gate_order_change {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_request.no_gate_order_change",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: QuotaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub tenant_id: TenantId,
    pub operation_kind: QuotaOperationKind,
    pub capability_ref: Option<String>,
    pub throttle_cause: QuotaThrottleCause,
    pub allow_eligible: bool,
    pub wait_permitted: bool,
    pub wait_ms: Option<u32>,
    pub refuse_required: bool,
    pub deterministic: bool,
    pub no_authority_grant: bool,
    pub no_gate_order_change: bool,
}

impl QuotaPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        tenant_id: TenantId,
        operation_kind: QuotaOperationKind,
        capability_ref: Option<String>,
        throttle_cause: QuotaThrottleCause,
        allow_eligible: bool,
        wait_permitted: bool,
        wait_ms: Option<u32>,
        refuse_required: bool,
        deterministic: bool,
        no_authority_grant: bool,
        no_gate_order_change: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            capability_id: QuotaCapabilityId::QuotaPolicyEvaluate,
            reason_code,
            tenant_id,
            operation_kind,
            capability_ref,
            throttle_cause,
            allow_eligible,
            wait_permitted,
            wait_ms,
            refuse_required,
            deterministic,
            no_authority_grant,
            no_gate_order_change,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for QuotaPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_ok.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        if self.capability_id != QuotaCapabilityId::QuotaPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_ok.capability_id",
                reason: "must be QUOTA_POLICY_EVALUATE",
            });
        }
        self.tenant_id.validate()?;
        if let Some(capability_ref) = &self.capability_ref {
            validate_token(
                "quota_policy_evaluate_ok.capability_ref",
                capability_ref,
                128,
            )?;
        }
        if let Some(wait_ms) = self.wait_ms {
            if wait_ms == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_policy_evaluate_ok.wait_ms",
                    reason: "must be > 0 when present",
                });
            }
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_authority_grant {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_ok.no_authority_grant",
                reason: "must be true",
            });
        }
        if !self.no_gate_order_change {
            return Err(ContractViolation::InvalidValue {
                field: "quota_policy_evaluate_ok.no_gate_order_change",
                reason: "must be true",
            });
        }

        if self.allow_eligible {
            if self.throttle_cause != QuotaThrottleCause::None {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_policy_evaluate_ok.throttle_cause",
                    reason: "must be NONE when allow_eligible=true",
                });
            }
            if self.wait_ms.is_some() || self.refuse_required {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_policy_evaluate_ok.wait_ms",
                    reason: "must be absent and refuse_required=false when allow_eligible=true",
                });
            }
        } else {
            match self.throttle_cause {
                QuotaThrottleCause::None => {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_policy_evaluate_ok.throttle_cause",
                        reason: "must not be NONE when allow_eligible=false",
                    });
                }
                QuotaThrottleCause::PolicyBlocked => {
                    if self.wait_permitted || self.wait_ms.is_some() || !self.refuse_required {
                        return Err(ContractViolation::InvalidValue {
                            field: "quota_policy_evaluate_ok.refuse_required",
                            reason: "policy-blocked path must be refuse-only",
                        });
                    }
                }
                QuotaThrottleCause::RateLimit | QuotaThrottleCause::BudgetExceeded => {
                    if self.wait_permitted {
                        if self.wait_ms.is_none() || self.refuse_required {
                            return Err(ContractViolation::InvalidValue {
                                field: "quota_policy_evaluate_ok.wait_ms",
                                reason: "wait path requires wait_ms and refuse_required=false",
                            });
                        }
                    } else if self.wait_ms.is_some() || !self.refuse_required {
                        return Err(ContractViolation::InvalidValue {
                            field: "quota_policy_evaluate_ok.refuse_required",
                            reason: "refuse path requires wait_ms absent and refuse_required=true",
                        });
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: QuotaRequestEnvelope,
    pub tenant_id: TenantId,
    pub operation_kind: QuotaOperationKind,
    pub throttle_cause: QuotaThrottleCause,
    pub allow_eligible: bool,
    pub wait_permitted: bool,
    pub wait_ms: Option<u32>,
    pub refuse_required: bool,
    pub deterministic: bool,
    pub no_authority_grant: bool,
    pub no_gate_order_change: bool,
}

impl QuotaDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: QuotaRequestEnvelope,
        tenant_id: TenantId,
        operation_kind: QuotaOperationKind,
        throttle_cause: QuotaThrottleCause,
        allow_eligible: bool,
        wait_permitted: bool,
        wait_ms: Option<u32>,
        refuse_required: bool,
        deterministic: bool,
        no_authority_grant: bool,
        no_gate_order_change: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            envelope,
            tenant_id,
            operation_kind,
            throttle_cause,
            allow_eligible,
            wait_permitted,
            wait_ms,
            refuse_required,
            deterministic,
            no_authority_grant,
            no_gate_order_change,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for QuotaDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_request.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        if let Some(wait_ms) = self.wait_ms {
            if wait_ms == 0 || wait_ms > self.envelope.max_wait_ms {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_decision_compute_request.wait_ms",
                    reason: "must be within 1..=envelope.max_wait_ms when present",
                });
            }
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_request.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_authority_grant {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_request.no_authority_grant",
                reason: "must be true",
            });
        }
        if !self.no_gate_order_change {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_request.no_gate_order_change",
                reason: "must be true",
            });
        }
        if self.allow_eligible {
            if self.throttle_cause != QuotaThrottleCause::None
                || self.wait_permitted
                || self.wait_ms.is_some()
                || self.refuse_required
            {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_decision_compute_request.allow_eligible",
                    reason: "allow path must have no throttle/wait/refuse flags",
                });
            }
        } else {
            if self.throttle_cause == QuotaThrottleCause::None {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_decision_compute_request.throttle_cause",
                    reason: "must not be NONE when allow_eligible=false",
                });
            }
            if self.throttle_cause == QuotaThrottleCause::PolicyBlocked {
                if self.wait_permitted || self.wait_ms.is_some() || !self.refuse_required {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_decision_compute_request.refuse_required",
                        reason: "policy block must be refuse-only",
                    });
                }
            } else if self.wait_permitted {
                if self.wait_ms.is_none() || self.refuse_required {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_decision_compute_request.wait_ms",
                        reason: "wait path requires wait_ms and refuse_required=false",
                    });
                }
            } else if self.wait_ms.is_some() || !self.refuse_required {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_decision_compute_request.refuse_required",
                    reason: "refuse path requires wait_ms absent and refuse_required=true",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: QuotaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub action: QuotaDecisionAction,
    pub wait_ms: Option<u32>,
    pub deterministic: bool,
    pub no_authority_grant: bool,
    pub no_gate_order_change: bool,
}

impl QuotaDecisionComputeOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        action: QuotaDecisionAction,
        wait_ms: Option<u32>,
        deterministic: bool,
        no_authority_grant: bool,
        no_gate_order_change: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            capability_id: QuotaCapabilityId::QuotaDecisionCompute,
            reason_code,
            action,
            wait_ms,
            deterministic,
            no_authority_grant,
            no_gate_order_change,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for QuotaDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_ok.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        if self.capability_id != QuotaCapabilityId::QuotaDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_ok.capability_id",
                reason: "must be QUOTA_DECISION_COMPUTE",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_authority_grant {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_ok.no_authority_grant",
                reason: "must be true",
            });
        }
        if !self.no_gate_order_change {
            return Err(ContractViolation::InvalidValue {
                field: "quota_decision_compute_ok.no_gate_order_change",
                reason: "must be true",
            });
        }

        match self.action {
            QuotaDecisionAction::Allow | QuotaDecisionAction::Refuse => {
                if self.wait_ms.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_decision_compute_ok.wait_ms",
                        reason: "must be absent when action is ALLOW or REFUSE",
                    });
                }
            }
            QuotaDecisionAction::Wait => {
                let wait_ms = self.wait_ms.ok_or(ContractViolation::InvalidValue {
                    field: "quota_decision_compute_ok.wait_ms",
                    reason: "must be present when action is WAIT",
                })?;
                if wait_ms == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_decision_compute_ok.wait_ms",
                        reason: "must be > 0 when action is WAIT",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: QuotaCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl QuotaRefuse {
    pub fn v1(
        capability_id: QuotaCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1QUOTA_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for QuotaRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1QUOTA_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "quota_refuse.schema_version",
                reason: "must match PH1QUOTA_CONTRACT_VERSION",
            });
        }
        validate_text("quota_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1QuotaRequest {
    QuotaPolicyEvaluate(QuotaPolicyEvaluateRequest),
    QuotaDecisionCompute(QuotaDecisionComputeRequest),
}

impl Validate for Ph1QuotaRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1QuotaRequest::QuotaPolicyEvaluate(req) => req.validate(),
            Ph1QuotaRequest::QuotaDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1QuotaResponse {
    QuotaPolicyEvaluateOk(QuotaPolicyEvaluateOk),
    QuotaDecisionComputeOk(QuotaDecisionComputeOk),
    Refuse(QuotaRefuse),
}

impl Validate for Ph1QuotaResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(out) => out.validate(),
            Ph1QuotaResponse::QuotaDecisionComputeOk(out) => out.validate(),
            Ph1QuotaResponse::Refuse(out) => out.validate(),
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

    fn envelope() -> QuotaRequestEnvelope {
        QuotaRequestEnvelope::v1(CorrelationId(7101), TurnId(8101), 8, 120_000).unwrap()
    }

    fn tenant() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    #[test]
    fn at_quota_01_tool_operation_requires_tool_name() {
        let req = QuotaPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            Some("user_1".to_string()),
            Some("device_1".to_string()),
            QuotaOperationKind::Tool,
            None,
            None,
            MonotonicTimeNs(1000),
            Some(10_000),
            true,
            false,
            false,
            true,
            Some(5000),
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_quota_02_non_tool_operation_requires_capability_id() {
        let req = QuotaPolicyEvaluateRequest::v1(
            envelope(),
            tenant(),
            Some("user_1".to_string()),
            None,
            QuotaOperationKind::Stt,
            None,
            None,
            MonotonicTimeNs(1000),
            Some(5_000),
            true,
            false,
            false,
            true,
            Some(5000),
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_quota_03_wait_decision_requires_wait_ms() {
        let out = QuotaDecisionComputeOk::v1(
            ReasonCodeId(1),
            QuotaDecisionAction::Wait,
            None,
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_quota_04_decision_output_requires_no_gate_order_change() {
        let out = QuotaDecisionComputeOk::v1(
            ReasonCodeId(1),
            QuotaDecisionAction::Allow,
            None,
            true,
            true,
            false,
        );
        assert!(out.is_err());
    }
}
