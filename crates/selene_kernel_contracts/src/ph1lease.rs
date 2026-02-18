#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::ph1work::WorkOrderId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1LEASE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaseCapabilityId {
    LeasePolicyEvaluate,
    LeaseDecisionCompute,
}

impl LeaseCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            LeaseCapabilityId::LeasePolicyEvaluate => "LEASE_POLICY_EVALUATE",
            LeaseCapabilityId::LeaseDecisionCompute => "LEASE_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaseOperation {
    Acquire,
    Renew,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LeaseDecisionAction {
    LeaseGranted,
    LeaseDenied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_diagnostics: u8,
    pub max_ttl_ms: u32,
}

impl LeaseRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_diagnostics: u8,
        max_ttl_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_diagnostics,
            max_ttl_ms,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for LeaseRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_request_envelope.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if self.max_ttl_ms == 0 || self.max_ttl_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_request_envelope.max_ttl_ms",
                reason: "must be within 1..=3_600_000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeasePolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LeaseRequestEnvelope,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub lease_owner_id: String,
    pub operation: LeaseOperation,
    pub requested_ttl_ms: u32,
    pub now_ns: MonotonicTimeNs,
    pub lease_token: Option<String>,
    pub active_lease_owner_id: Option<String>,
    pub active_lease_token: Option<String>,
    pub active_lease_expires_at_ns: Option<MonotonicTimeNs>,
    pub idempotency_key: Option<String>,
    pub deterministic_takeover_from_ledger: bool,
    pub one_active_lease_per_work_order: bool,
    pub token_owner_required: bool,
}

impl LeasePolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: LeaseRequestEnvelope,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        lease_owner_id: String,
        operation: LeaseOperation,
        requested_ttl_ms: u32,
        now_ns: MonotonicTimeNs,
        lease_token: Option<String>,
        active_lease_owner_id: Option<String>,
        active_lease_token: Option<String>,
        active_lease_expires_at_ns: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
        deterministic_takeover_from_ledger: bool,
        one_active_lease_per_work_order: bool,
        token_owner_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            lease_owner_id,
            operation,
            requested_ttl_ms,
            now_ns,
            lease_token,
            active_lease_owner_id,
            active_lease_token,
            active_lease_expires_at_ns,
            idempotency_key,
            deterministic_takeover_from_ledger,
            one_active_lease_per_work_order,
            token_owner_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for LeasePolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;

        validate_token_ascii(
            "lease_policy_evaluate_request.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;
        validate_opt_token_ascii(
            "lease_policy_evaluate_request.lease_token",
            &self.lease_token,
            192,
        )?;
        validate_opt_token_ascii(
            "lease_policy_evaluate_request.active_lease_owner_id",
            &self.active_lease_owner_id,
            128,
        )?;
        validate_opt_token_ascii(
            "lease_policy_evaluate_request.active_lease_token",
            &self.active_lease_token,
            192,
        )?;
        validate_opt_token_ascii(
            "lease_policy_evaluate_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;

        if self.requested_ttl_ms == 0 || self.requested_ttl_ms > self.envelope.max_ttl_ms {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.requested_ttl_ms",
                reason: "must be within 1..=envelope.max_ttl_ms",
            });
        }
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.now_ns",
                reason: "must be > 0",
            });
        }

        if matches!(
            self.operation,
            LeaseOperation::Renew | LeaseOperation::Release
        ) && self.lease_token.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.lease_token",
                reason: "must be present when operation is RENEW or RELEASE",
            });
        }

        let active_fields_count = usize::from(self.active_lease_owner_id.is_some())
            + usize::from(self.active_lease_token.is_some())
            + usize::from(self.active_lease_expires_at_ns.is_some());
        if active_fields_count != 0 && active_fields_count != 3 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.active_lease_owner_id",
                reason: "active lease snapshot must include owner/token/expires together",
            });
        }
        if let Some(expires_at_ns) = self.active_lease_expires_at_ns {
            if expires_at_ns.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "lease_policy_evaluate_request.active_lease_expires_at_ns",
                    reason: "must be > 0 when present",
                });
            }
        }

        if !self.deterministic_takeover_from_ledger {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.deterministic_takeover_from_ledger",
                reason: "must be true",
            });
        }
        if !self.one_active_lease_per_work_order {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.one_active_lease_per_work_order",
                reason: "must be true",
            });
        }
        if !self.token_owner_required {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_request.token_owner_required",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeasePolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LeaseCapabilityId,
    pub reason_code: ReasonCodeId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub lease_owner_id: String,
    pub operation: LeaseOperation,
    pub lease_exists: bool,
    pub lease_expired: bool,
    pub owner_match: bool,
    pub token_match: bool,
    pub ttl_in_bounds: bool,
    pub grant_eligible: bool,
    pub deterministic_takeover_from_ledger: bool,
    pub one_active_lease_per_work_order: bool,
    pub token_owner_required: bool,
}

impl LeasePolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        lease_owner_id: String,
        operation: LeaseOperation,
        lease_exists: bool,
        lease_expired: bool,
        owner_match: bool,
        token_match: bool,
        ttl_in_bounds: bool,
        grant_eligible: bool,
        deterministic_takeover_from_ledger: bool,
        one_active_lease_per_work_order: bool,
        token_owner_required: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            capability_id: LeaseCapabilityId::LeasePolicyEvaluate,
            reason_code,
            tenant_id,
            work_order_id,
            lease_owner_id,
            operation,
            lease_exists,
            lease_expired,
            owner_match,
            token_match,
            ttl_in_bounds,
            grant_eligible,
            deterministic_takeover_from_ledger,
            one_active_lease_per_work_order,
            token_owner_required,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LeasePolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LeaseCapabilityId::LeasePolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.capability_id",
                reason: "must be LEASE_POLICY_EVALUATE",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token_ascii(
            "lease_policy_evaluate_ok.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;

        if self.grant_eligible && !self.ttl_in_bounds {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.ttl_in_bounds",
                reason: "must be true when grant_eligible=true",
            });
        }
        if self.lease_expired && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.lease_expired",
                reason: "lease_expired requires lease_exists=true",
            });
        }
        if self.owner_match && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.owner_match",
                reason: "owner_match requires lease_exists=true",
            });
        }
        if self.token_match && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.token_match",
                reason: "token_match requires lease_exists=true",
            });
        }
        if !self.deterministic_takeover_from_ledger {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.deterministic_takeover_from_ledger",
                reason: "must be true",
            });
        }
        if !self.one_active_lease_per_work_order {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.one_active_lease_per_work_order",
                reason: "must be true",
            });
        }
        if !self.token_owner_required {
            return Err(ContractViolation::InvalidValue {
                field: "lease_policy_evaluate_ok.token_owner_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LeaseRequestEnvelope,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub lease_owner_id: String,
    pub operation: LeaseOperation,
    pub requested_ttl_ms: u32,
    pub now_ns: MonotonicTimeNs,
    pub lease_token: Option<String>,
    pub proposed_lease_token: Option<String>,
    pub lease_exists: bool,
    pub lease_expired: bool,
    pub owner_match: bool,
    pub token_match: bool,
    pub ttl_in_bounds: bool,
    pub grant_eligible: bool,
    pub active_lease_owner_id: Option<String>,
    pub active_lease_expires_at_ns: Option<MonotonicTimeNs>,
    pub deterministic_takeover_from_ledger: bool,
    pub one_active_lease_per_work_order: bool,
    pub token_owner_required: bool,
}

impl LeaseDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: LeaseRequestEnvelope,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        lease_owner_id: String,
        operation: LeaseOperation,
        requested_ttl_ms: u32,
        now_ns: MonotonicTimeNs,
        lease_token: Option<String>,
        proposed_lease_token: Option<String>,
        lease_exists: bool,
        lease_expired: bool,
        owner_match: bool,
        token_match: bool,
        ttl_in_bounds: bool,
        grant_eligible: bool,
        active_lease_owner_id: Option<String>,
        active_lease_expires_at_ns: Option<MonotonicTimeNs>,
        deterministic_takeover_from_ledger: bool,
        one_active_lease_per_work_order: bool,
        token_owner_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            lease_owner_id,
            operation,
            requested_ttl_ms,
            now_ns,
            lease_token,
            proposed_lease_token,
            lease_exists,
            lease_expired,
            owner_match,
            token_match,
            ttl_in_bounds,
            grant_eligible,
            active_lease_owner_id,
            active_lease_expires_at_ns,
            deterministic_takeover_from_ledger,
            one_active_lease_per_work_order,
            token_owner_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for LeaseDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;

        validate_token_ascii(
            "lease_decision_compute_request.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;
        validate_opt_token_ascii(
            "lease_decision_compute_request.lease_token",
            &self.lease_token,
            192,
        )?;
        validate_opt_token_ascii(
            "lease_decision_compute_request.proposed_lease_token",
            &self.proposed_lease_token,
            192,
        )?;
        validate_opt_token_ascii(
            "lease_decision_compute_request.active_lease_owner_id",
            &self.active_lease_owner_id,
            128,
        )?;

        if self.requested_ttl_ms == 0 || self.requested_ttl_ms > self.envelope.max_ttl_ms {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.requested_ttl_ms",
                reason: "must be within 1..=envelope.max_ttl_ms",
            });
        }
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.now_ns",
                reason: "must be > 0",
            });
        }
        if matches!(
            self.operation,
            LeaseOperation::Renew | LeaseOperation::Release
        ) && self.lease_token.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.lease_token",
                reason: "must be present when operation is RENEW or RELEASE",
            });
        }

        if self.lease_exists {
            if self.active_lease_owner_id.is_none() || self.active_lease_expires_at_ns.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "lease_decision_compute_request.active_lease_owner_id",
                    reason: "active lease details are required when lease_exists=true",
                });
            }
        } else if self.active_lease_owner_id.is_some() || self.active_lease_expires_at_ns.is_some()
        {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.active_lease_owner_id",
                reason: "active lease details must be absent when lease_exists=false",
            });
        }

        if self.lease_expired && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.lease_expired",
                reason: "lease_expired requires lease_exists=true",
            });
        }
        if self.owner_match && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.owner_match",
                reason: "owner_match requires lease_exists=true",
            });
        }
        if self.token_match && !self.lease_exists {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.token_match",
                reason: "token_match requires lease_exists=true",
            });
        }
        if self.grant_eligible && !self.ttl_in_bounds {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.ttl_in_bounds",
                reason: "must be true when grant_eligible=true",
            });
        }

        if !self.deterministic_takeover_from_ledger {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.deterministic_takeover_from_ledger",
                reason: "must be true",
            });
        }
        if !self.one_active_lease_per_work_order {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.one_active_lease_per_work_order",
                reason: "must be true",
            });
        }
        if !self.token_owner_required {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_request.token_owner_required",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LeaseCapabilityId,
    pub reason_code: ReasonCodeId,
    pub operation: LeaseOperation,
    pub action: LeaseDecisionAction,
    pub lease_active_after_decision: bool,
    pub lease_token: Option<String>,
    pub lease_expires_at_ns: Option<MonotonicTimeNs>,
    pub held_by_owner_id: Option<String>,
    pub held_until_ns: Option<MonotonicTimeNs>,
    pub resume_from_ledger_required: bool,
    pub deterministic_takeover_from_ledger: bool,
    pub one_active_lease_per_work_order: bool,
    pub token_owner_required: bool,
}

impl LeaseDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        operation: LeaseOperation,
        action: LeaseDecisionAction,
        lease_active_after_decision: bool,
        lease_token: Option<String>,
        lease_expires_at_ns: Option<MonotonicTimeNs>,
        held_by_owner_id: Option<String>,
        held_until_ns: Option<MonotonicTimeNs>,
        resume_from_ledger_required: bool,
        deterministic_takeover_from_ledger: bool,
        one_active_lease_per_work_order: bool,
        token_owner_required: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            capability_id: LeaseCapabilityId::LeaseDecisionCompute,
            reason_code,
            operation,
            action,
            lease_active_after_decision,
            lease_token,
            lease_expires_at_ns,
            held_by_owner_id,
            held_until_ns,
            resume_from_ledger_required,
            deterministic_takeover_from_ledger,
            one_active_lease_per_work_order,
            token_owner_required,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LeaseDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_ok.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LeaseCapabilityId::LeaseDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_ok.capability_id",
                reason: "must be LEASE_DECISION_COMPUTE",
            });
        }
        validate_opt_token_ascii(
            "lease_decision_compute_ok.lease_token",
            &self.lease_token,
            192,
        )?;
        validate_opt_token_ascii(
            "lease_decision_compute_ok.held_by_owner_id",
            &self.held_by_owner_id,
            128,
        )?;

        match self.action {
            LeaseDecisionAction::LeaseGranted => {
                if self.lease_active_after_decision {
                    if self.lease_token.is_none() || self.lease_expires_at_ns.is_none() {
                        return Err(ContractViolation::InvalidValue {
                            field: "lease_decision_compute_ok.lease_token",
                            reason: "active grant requires lease_token and lease_expires_at_ns",
                        });
                    }
                } else {
                    if self.operation != LeaseOperation::Release {
                        return Err(ContractViolation::InvalidValue {
                            field: "lease_decision_compute_ok.lease_active_after_decision",
                            reason: "only RELEASE may grant with inactive post-state",
                        });
                    }
                    if self.lease_token.is_some() || self.lease_expires_at_ns.is_some() {
                        return Err(ContractViolation::InvalidValue {
                            field: "lease_decision_compute_ok.lease_token",
                            reason: "release grant must not return active lease token/expires",
                        });
                    }
                }
                if self.held_by_owner_id.is_some() || self.held_until_ns.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "lease_decision_compute_ok.held_by_owner_id",
                        reason: "grant path must not include held-by details",
                    });
                }
            }
            LeaseDecisionAction::LeaseDenied => {
                if self.lease_active_after_decision {
                    return Err(ContractViolation::InvalidValue {
                        field: "lease_decision_compute_ok.lease_active_after_decision",
                        reason: "denied path must set lease_active_after_decision=false",
                    });
                }
                if self.lease_token.is_some() || self.lease_expires_at_ns.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "lease_decision_compute_ok.lease_token",
                        reason: "denied path must not include lease token/expires",
                    });
                }
                if self.held_until_ns.is_some() && self.held_by_owner_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "lease_decision_compute_ok.held_by_owner_id",
                        reason: "held_until_ns requires held_by_owner_id",
                    });
                }
            }
        }

        if let Some(expires_at_ns) = self.lease_expires_at_ns {
            if expires_at_ns.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "lease_decision_compute_ok.lease_expires_at_ns",
                    reason: "must be > 0 when present",
                });
            }
        }
        if let Some(held_until_ns) = self.held_until_ns {
            if held_until_ns.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "lease_decision_compute_ok.held_until_ns",
                    reason: "must be > 0 when present",
                });
            }
        }

        if !self.deterministic_takeover_from_ledger {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_ok.deterministic_takeover_from_ledger",
                reason: "must be true",
            });
        }
        if !self.one_active_lease_per_work_order {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_ok.one_active_lease_per_work_order",
                reason: "must be true",
            });
        }
        if !self.token_owner_required {
            return Err(ContractViolation::InvalidValue {
                field: "lease_decision_compute_ok.token_owner_required",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaseRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: LeaseCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl LeaseRefuse {
    pub fn v1(
        capability_id: LeaseCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEASE_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LeaseRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEASE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "lease_refuse.schema_version",
                reason: "must match PH1LEASE_CONTRACT_VERSION",
            });
        }
        validate_text_ascii("lease_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LeaseRequest {
    LeasePolicyEvaluate(LeasePolicyEvaluateRequest),
    LeaseDecisionCompute(LeaseDecisionComputeRequest),
}

impl Validate for Ph1LeaseRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LeaseRequest::LeasePolicyEvaluate(req) => req.validate(),
            Ph1LeaseRequest::LeaseDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LeaseResponse {
    LeasePolicyEvaluateOk(LeasePolicyEvaluateOk),
    LeaseDecisionComputeOk(LeaseDecisionComputeOk),
    Refuse(LeaseRefuse),
}

impl Validate for Ph1LeaseResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LeaseResponse::LeasePolicyEvaluateOk(out) => out.validate(),
            Ph1LeaseResponse::LeaseDecisionComputeOk(out) => out.validate(),
            Ph1LeaseResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token_ascii(
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

fn validate_opt_token_ascii(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_token_ascii(field, v, max_len)?;
    }
    Ok(())
}

fn validate_text_ascii(
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

    fn envelope() -> LeaseRequestEnvelope {
        LeaseRequestEnvelope::v1(CorrelationId(7101), TurnId(8101), 8, 300_000).unwrap()
    }

    fn tenant_id() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        WorkOrderId::new("wo_demo").unwrap()
    }

    #[test]
    fn at_lease_contract_01_token_required_for_renew_and_release() {
        let renew_missing_token = LeasePolicyEvaluateRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            "owner_a".to_string(),
            LeaseOperation::Renew,
            30_000,
            MonotonicTimeNs(1_000_000),
            None,
            Some("owner_a".to_string()),
            Some("tok_a".to_string()),
            Some(MonotonicTimeNs(2_000_000)),
            Some("idem_1".to_string()),
            true,
            true,
            true,
        );
        assert!(renew_missing_token.is_err());

        let release_missing_token = LeaseDecisionComputeRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            "owner_a".to_string(),
            LeaseOperation::Release,
            30_000,
            MonotonicTimeNs(1_000_000),
            None,
            None,
            true,
            false,
            true,
            false,
            true,
            false,
            Some("owner_a".to_string()),
            Some(MonotonicTimeNs(2_000_000)),
            true,
            true,
            true,
        );
        assert!(release_missing_token.is_err());
    }

    #[test]
    fn at_lease_contract_02_decision_grant_inactive_allowed_only_for_release() {
        let invalid = LeaseDecisionComputeOk::v1(
            ReasonCodeId(1),
            LeaseOperation::Acquire,
            LeaseDecisionAction::LeaseGranted,
            false,
            None,
            None,
            None,
            None,
            false,
            true,
            true,
            true,
        );
        assert!(invalid.is_err());

        let valid_release = LeaseDecisionComputeOk::v1(
            ReasonCodeId(2),
            LeaseOperation::Release,
            LeaseDecisionAction::LeaseGranted,
            false,
            None,
            None,
            None,
            None,
            false,
            true,
            true,
            true,
        )
        .unwrap();
        assert_eq!(valid_release.action, LeaseDecisionAction::LeaseGranted);
    }

    #[test]
    fn at_lease_contract_03_expired_takeover_requires_existing_lease_snapshot() {
        let invalid = LeaseDecisionComputeRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            "owner_b".to_string(),
            LeaseOperation::Acquire,
            60_000,
            MonotonicTimeNs(5_000_000),
            None,
            Some("tok_new".to_string()),
            false,
            true,
            false,
            false,
            true,
            true,
            None,
            None,
            true,
            true,
            true,
        );
        assert!(invalid.is_err());
    }
}
