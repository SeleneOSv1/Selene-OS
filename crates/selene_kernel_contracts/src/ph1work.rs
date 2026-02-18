#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1WORK_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

fn validate_id(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    Ok(())
}

fn validate_opt_id(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        validate_id(field, v, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkOrderId(String);

impl WorkOrderId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = v.into();
        validate_id("work_order_id", &v, 128)?;
        Ok(Self(v))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for WorkOrderId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("work_order_id", &self.0, 128)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkOrderStatus {
    Draft,
    Clarify,
    Confirm,
    Executing,
    Done,
    Refused,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkOrderLedgerEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub work_order_status: WorkOrderStatus,
    pub reason_code: ReasonCodeId,
    pub step_id: Option<String>,
    pub step_input_hash: Option<String>,
    pub lease_owner_id: Option<String>,
    pub lease_token_hash: Option<String>,
    pub lease_expires_at: Option<MonotonicTimeNs>,
    pub idempotency_key: Option<String>,
}

impl WorkOrderLedgerEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        work_order_status: WorkOrderStatus,
        reason_code: ReasonCodeId,
        step_id: Option<String>,
        step_input_hash: Option<String>,
        lease_owner_id: Option<String>,
        lease_token_hash: Option<String>,
        lease_expires_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            created_at,
            tenant_id,
            work_order_id,
            correlation_id,
            turn_id,
            work_order_status,
            reason_code,
            step_id,
            step_input_hash,
            lease_owner_id,
            lease_token_hash,
            lease_expires_at,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WorkOrderLedgerEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event_input.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event_input.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id("work_order_ledger_event_input.step_id", &self.step_id, 96)?;
        validate_opt_id(
            "work_order_ledger_event_input.step_input_hash",
            &self.step_input_hash,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event_input.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event_input.lease_token_hash",
            &self.lease_token_hash,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkOrderLedgerEvent {
    pub schema_version: SchemaVersion,
    pub work_order_event_id: u64,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub work_order_status: WorkOrderStatus,
    pub reason_code: ReasonCodeId,
    pub step_id: Option<String>,
    pub step_input_hash: Option<String>,
    pub lease_owner_id: Option<String>,
    pub lease_token_hash: Option<String>,
    pub lease_expires_at: Option<MonotonicTimeNs>,
    pub idempotency_key: Option<String>,
}

impl WorkOrderLedgerEvent {
    pub fn from_input_v1(
        work_order_event_id: u64,
        input: WorkOrderLedgerEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let row = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            work_order_event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            work_order_id: input.work_order_id,
            correlation_id: input.correlation_id,
            turn_id: input.turn_id,
            work_order_status: input.work_order_status,
            reason_code: input.reason_code,
            step_id: input.step_id,
            step_input_hash: input.step_input_hash,
            lease_owner_id: input.lease_owner_id,
            lease_token_hash: input.lease_token_hash,
            lease_expires_at: input.lease_expires_at,
            idempotency_key: input.idempotency_key,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for WorkOrderLedgerEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        if self.work_order_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event.work_order_event_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_ledger_event.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id("work_order_ledger_event.step_id", &self.step_id, 96)?;
        validate_opt_id(
            "work_order_ledger_event.step_input_hash",
            &self.step_input_hash,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event.lease_token_hash",
            &self.lease_token_hash,
            128,
        )?;
        validate_opt_id(
            "work_order_ledger_event.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkOrderCurrentRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub work_order_status: WorkOrderStatus,
    pub last_event_id: u64,
    pub last_reason_code: ReasonCodeId,
    pub last_updated_at: MonotonicTimeNs,
    pub step_id: Option<String>,
    pub step_input_hash: Option<String>,
    pub lease_owner_id: Option<String>,
    pub lease_token_hash: Option<String>,
    pub lease_expires_at: Option<MonotonicTimeNs>,
}

impl WorkOrderCurrentRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        work_order_status: WorkOrderStatus,
        last_event_id: u64,
        last_reason_code: ReasonCodeId,
        last_updated_at: MonotonicTimeNs,
        step_id: Option<String>,
        step_input_hash: Option<String>,
        lease_owner_id: Option<String>,
        lease_token_hash: Option<String>,
        lease_expires_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            tenant_id,
            work_order_id,
            correlation_id,
            turn_id,
            work_order_status,
            last_event_id,
            last_reason_code,
            last_updated_at,
            step_id,
            step_input_hash,
            lease_owner_id,
            lease_token_hash,
            lease_expires_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for WorkOrderCurrentRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_current_record.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.last_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_current_record.last_event_id",
                reason: "must be > 0",
            });
        }
        if self.last_reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_current_record.last_reason_code",
                reason: "must be > 0",
            });
        }
        if self.last_updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_order_current_record.last_updated_at",
                reason: "must be > 0",
            });
        }
        validate_opt_id("work_order_current_record.step_id", &self.step_id, 96)?;
        validate_opt_id(
            "work_order_current_record.step_input_hash",
            &self.step_input_hash,
            128,
        )?;
        validate_opt_id(
            "work_order_current_record.lease_owner_id",
            &self.lease_owner_id,
            128,
        )?;
        validate_opt_id(
            "work_order_current_record.lease_token_hash",
            &self.lease_token_hash,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkCapabilityId {
    WorkPolicyEvaluate,
    WorkDecisionCompute,
}

impl WorkCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            WorkCapabilityId::WorkPolicyEvaluate => "WORK_POLICY_EVALUATE",
            WorkCapabilityId::WorkDecisionCompute => "WORK_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkEventType {
    WorkOrderCreated,
    FieldSet,
    FieldConflictResolved,
    StatusChanged,
    StepStarted,
    StepFinished,
    StepFailed,
    WorkOrderCanceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorkDecisionStatus {
    Ok,
    Refused,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_payload_bytes: u16,
    pub max_diagnostics: u8,
}

impl WorkRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_payload_bytes: u16,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_payload_bytes,
            max_diagnostics,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for WorkRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_request_envelope.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_payload_bytes == 0 || self.max_payload_bytes > 16_384 {
            return Err(ContractViolation::InvalidValue {
                field: "work_request_envelope.max_payload_bytes",
                reason: "must be within 1..=16384",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "work_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: WorkRequestEnvelope,
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
    pub deterministic_replay_order: bool,
    pub no_silent_conflict_merge: bool,
}

impl WorkPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: WorkRequestEnvelope,
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
        deterministic_replay_order: bool,
        no_silent_conflict_merge: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            envelope,
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
            deterministic_replay_order,
            no_silent_conflict_merge,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for WorkPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.created_at",
                reason: "must be > 0",
            });
        }
        validate_text_ascii(
            "work_policy_evaluate_request.payload_min",
            &self.payload_min,
            self.envelope.max_payload_bytes as usize,
        )?;
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token_ascii(
                "work_policy_evaluate_request.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        if self.idempotency_required && self.idempotency_key.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.idempotency_key",
                reason: "must be present when idempotency_required=true",
            });
        }
        if self.idempotency_duplicate && self.idempotency_key.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.idempotency_key",
                reason: "must be present when idempotency_duplicate=true",
            });
        }
        if !self.deterministic_replay_order {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.deterministic_replay_order",
                reason: "must be true",
            });
        }
        if !self.no_silent_conflict_merge {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_request.no_silent_conflict_merge",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: WorkCapabilityId,
    pub reason_code: ReasonCodeId,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub event_type: WorkEventType,
    pub payload_min_hash: String,
    pub event_valid: bool,
    pub append_allowed: bool,
    pub idempotency_duplicate: bool,
    pub append_only_violation: bool,
    pub tenant_scope_mismatch: bool,
    pub deterministic_replay_order: bool,
    pub no_silent_conflict_merge: bool,
}

impl WorkPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        event_type: WorkEventType,
        payload_min_hash: String,
        event_valid: bool,
        append_allowed: bool,
        idempotency_duplicate: bool,
        append_only_violation: bool,
        tenant_scope_mismatch: bool,
        deterministic_replay_order: bool,
        no_silent_conflict_merge: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            capability_id: WorkCapabilityId::WorkPolicyEvaluate,
            reason_code,
            tenant_id,
            work_order_id,
            event_type,
            payload_min_hash,
            event_valid,
            append_allowed,
            idempotency_duplicate,
            append_only_violation,
            tenant_scope_mismatch,
            deterministic_replay_order,
            no_silent_conflict_merge,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for WorkPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        if self.capability_id != WorkCapabilityId::WorkPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.capability_id",
                reason: "must be WORK_POLICY_EVALUATE",
            });
        }
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        validate_token_ascii(
            "work_policy_evaluate_ok.payload_min_hash",
            &self.payload_min_hash,
            128,
        )?;
        if self.append_allowed
            && (self.append_only_violation
                || self.tenant_scope_mismatch
                || self.idempotency_duplicate)
        {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.append_allowed",
                reason: "must be false when violation/scope-mismatch/duplicate flags are set",
            });
        }
        if !self.event_valid && self.append_allowed {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.event_valid",
                reason: "must be true when append_allowed=true",
            });
        }
        if !self.deterministic_replay_order {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.deterministic_replay_order",
                reason: "must be true",
            });
        }
        if !self.no_silent_conflict_merge {
            return Err(ContractViolation::InvalidValue {
                field: "work_policy_evaluate_ok.no_silent_conflict_merge",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: WorkRequestEnvelope,
    pub tenant_id: TenantId,
    pub work_order_id: WorkOrderId,
    pub event_type: WorkEventType,
    pub event_valid: bool,
    pub append_allowed: bool,
    pub idempotency_duplicate: bool,
    pub append_only_violation: bool,
    pub tenant_scope_mismatch: bool,
    pub existing_event_id_on_duplicate: Option<u64>,
    pub proposed_event_id: Option<u64>,
    pub deterministic_replay_order: bool,
    pub no_silent_conflict_merge: bool,
}

impl WorkDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: WorkRequestEnvelope,
        tenant_id: TenantId,
        work_order_id: WorkOrderId,
        event_type: WorkEventType,
        event_valid: bool,
        append_allowed: bool,
        idempotency_duplicate: bool,
        append_only_violation: bool,
        tenant_scope_mismatch: bool,
        existing_event_id_on_duplicate: Option<u64>,
        proposed_event_id: Option<u64>,
        deterministic_replay_order: bool,
        no_silent_conflict_merge: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            event_type,
            event_valid,
            append_allowed,
            idempotency_duplicate,
            append_only_violation,
            tenant_scope_mismatch,
            existing_event_id_on_duplicate,
            proposed_event_id,
            deterministic_replay_order,
            no_silent_conflict_merge,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for WorkDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_request.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        self.work_order_id.validate()?;
        if self.idempotency_duplicate {
            if self.existing_event_id_on_duplicate.unwrap_or(0) == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "work_decision_compute_request.existing_event_id_on_duplicate",
                    reason: "must be present when idempotency_duplicate=true",
                });
            }
            if self.proposed_event_id.is_some() || self.append_allowed {
                return Err(ContractViolation::InvalidValue {
                    field: "work_decision_compute_request.append_allowed",
                    reason: "duplicate path must not append a new event",
                });
            }
        } else {
            if self.existing_event_id_on_duplicate.is_some() {
                return Err(ContractViolation::InvalidValue {
                    field: "work_decision_compute_request.existing_event_id_on_duplicate",
                    reason: "must be absent when idempotency_duplicate=false",
                });
            }
            if self.append_allowed {
                if self.proposed_event_id.unwrap_or(0) == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "work_decision_compute_request.proposed_event_id",
                        reason: "must be present when append_allowed=true",
                    });
                }
            } else if self.proposed_event_id.is_some() {
                return Err(ContractViolation::InvalidValue {
                    field: "work_decision_compute_request.proposed_event_id",
                    reason: "must be absent when append_allowed=false",
                });
            }
        }
        if (self.append_only_violation || self.tenant_scope_mismatch) && self.append_allowed {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_request.append_allowed",
                reason: "must be false when append-only or tenant scope violation is set",
            });
        }
        if self.append_allowed && !self.event_valid {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_request.event_valid",
                reason: "must be true when append_allowed=true",
            });
        }
        if !self.deterministic_replay_order {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_request.deterministic_replay_order",
                reason: "must be true",
            });
        }
        if !self.no_silent_conflict_merge {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_request.no_silent_conflict_merge",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: WorkCapabilityId,
    pub reason_code: ReasonCodeId,
    pub status: WorkDecisionStatus,
    pub work_order_event_id: Option<u64>,
    pub idempotency_no_op: bool,
    pub deterministic_replay_order: bool,
    pub no_silent_conflict_merge: bool,
}

impl WorkDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        status: WorkDecisionStatus,
        work_order_event_id: Option<u64>,
        idempotency_no_op: bool,
        deterministic_replay_order: bool,
        no_silent_conflict_merge: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            capability_id: WorkCapabilityId::WorkDecisionCompute,
            reason_code,
            status,
            work_order_event_id,
            idempotency_no_op,
            deterministic_replay_order,
            no_silent_conflict_merge,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for WorkDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_ok.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        if self.capability_id != WorkCapabilityId::WorkDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_ok.capability_id",
                reason: "must be WORK_DECISION_COMPUTE",
            });
        }
        match self.status {
            WorkDecisionStatus::Ok => {
                if self.work_order_event_id.unwrap_or(0) == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "work_decision_compute_ok.work_order_event_id",
                        reason: "must be present when status=OK",
                    });
                }
            }
            WorkDecisionStatus::Refused | WorkDecisionStatus::Fail => {
                if self.work_order_event_id.is_some() || self.idempotency_no_op {
                    return Err(ContractViolation::InvalidValue {
                        field: "work_decision_compute_ok.idempotency_no_op",
                        reason: "must be false and event_id absent when status is REFUSED/FAIL",
                    });
                }
            }
        }
        if self.idempotency_no_op && self.status != WorkDecisionStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_ok.idempotency_no_op",
                reason: "must have status=OK when idempotency_no_op=true",
            });
        }
        if !self.deterministic_replay_order {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_ok.deterministic_replay_order",
                reason: "must be true",
            });
        }
        if !self.no_silent_conflict_merge {
            return Err(ContractViolation::InvalidValue {
                field: "work_decision_compute_ok.no_silent_conflict_merge",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: WorkCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl WorkRefuse {
    pub fn v1(
        capability_id: WorkCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1WORK_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for WorkRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1WORK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "work_refuse.schema_version",
                reason: "must match PH1WORK_CONTRACT_VERSION",
            });
        }
        validate_text_ascii("work_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1WorkRequest {
    WorkPolicyEvaluate(WorkPolicyEvaluateRequest),
    WorkDecisionCompute(WorkDecisionComputeRequest),
}

impl Validate for Ph1WorkRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1WorkRequest::WorkPolicyEvaluate(req) => req.validate(),
            Ph1WorkRequest::WorkDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1WorkResponse {
    WorkPolicyEvaluateOk(WorkPolicyEvaluateOk),
    WorkDecisionComputeOk(WorkDecisionComputeOk),
    Refuse(WorkRefuse),
}

impl Validate for Ph1WorkResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1WorkResponse::WorkPolicyEvaluateOk(out) => out.validate(),
            Ph1WorkResponse::WorkDecisionComputeOk(out) => out.validate(),
            Ph1WorkResponse::Refuse(out) => out.validate(),
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

    fn envelope() -> WorkRequestEnvelope {
        WorkRequestEnvelope::v1(CorrelationId(7001), TurnId(8001), 1024, 8).unwrap()
    }

    fn tenant_id() -> TenantId {
        TenantId::new("tenant_demo").unwrap()
    }

    fn work_order_id() -> WorkOrderId {
        WorkOrderId::new("work_order_123").unwrap()
    }

    #[test]
    fn at_work_01_policy_requires_idempotency_key_when_required() {
        let req = WorkPolicyEvaluateRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            WorkEventType::StepStarted,
            "{\"step\":\"dispatch\"}".to_string(),
            MonotonicTimeNs(1_000_000),
            None,
            true,
            false,
            false,
            false,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_work_02_policy_output_rejects_append_on_duplicate() {
        let out = WorkPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            tenant_id(),
            work_order_id(),
            WorkEventType::StepFinished,
            "payload_hash_1".to_string(),
            true,
            true,
            true,
            false,
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_work_03_decision_request_duplicate_requires_existing_event_id() {
        let req = WorkDecisionComputeRequest::v1(
            envelope(),
            tenant_id(),
            work_order_id(),
            WorkEventType::StepFailed,
            true,
            false,
            true,
            false,
            false,
            None,
            None,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_work_04_decision_output_ok_requires_event_id() {
        let out = WorkDecisionComputeOk::v1(
            ReasonCodeId(2),
            WorkDecisionStatus::Ok,
            None,
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }
}
