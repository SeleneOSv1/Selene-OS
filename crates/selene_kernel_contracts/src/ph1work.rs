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
