#![forbid(unsafe_code)]

use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PBS_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BlueprintVersion(pub u32);

impl Validate for BlueprintVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "blueprint_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProcessId(String);

impl ProcessId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for ProcessId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("process_id", &self.0, 96)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IntentType(String);

impl IntentType {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for IntentType {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("intent_type", &self.0, 96)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlueprintStatus {
    Draft,
    Active,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessBlueprintEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub process_id: ProcessId,
    pub blueprint_version: BlueprintVersion,
    pub intent_type: IntentType,
    pub status: BlueprintStatus,
    pub ordered_step_count: u16,
    pub confirmation_step_count: u16,
    pub simulation_requirements_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl ProcessBlueprintEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: TenantId,
        process_id: ProcessId,
        blueprint_version: BlueprintVersion,
        intent_type: IntentType,
        status: BlueprintStatus,
        ordered_step_count: u16,
        confirmation_step_count: u16,
        simulation_requirements_hash: String,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PBS_CONTRACT_VERSION,
            created_at,
            tenant_id,
            process_id,
            blueprint_version,
            intent_type,
            status,
            ordered_step_count,
            confirmation_step_count,
            simulation_requirements_hash,
            reason_code,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ProcessBlueprintEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PBS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event_input.schema_version",
                reason: "must match PH1PBS_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event_input.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.process_id.validate()?;
        self.blueprint_version.validate()?;
        self.intent_type.validate()?;
        if self.ordered_step_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event_input.ordered_step_count",
                reason: "must be > 0",
            });
        }
        if self.confirmation_step_count > self.ordered_step_count {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event_input.confirmation_step_count",
                reason: "must be <= ordered_step_count",
            });
        }
        validate_id(
            "process_blueprint_event_input.simulation_requirements_hash",
            &self.simulation_requirements_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "process_blueprint_event_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessBlueprintEvent {
    pub schema_version: SchemaVersion,
    pub process_blueprint_event_id: u64,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub process_id: ProcessId,
    pub blueprint_version: BlueprintVersion,
    pub intent_type: IntentType,
    pub status: BlueprintStatus,
    pub ordered_step_count: u16,
    pub confirmation_step_count: u16,
    pub simulation_requirements_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl ProcessBlueprintEvent {
    pub fn from_input_v1(
        process_blueprint_event_id: u64,
        input: ProcessBlueprintEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1PBS_CONTRACT_VERSION,
            process_blueprint_event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            process_id: input.process_id,
            blueprint_version: input.blueprint_version,
            intent_type: input.intent_type,
            status: input.status,
            ordered_step_count: input.ordered_step_count,
            confirmation_step_count: input.confirmation_step_count,
            simulation_requirements_hash: input.simulation_requirements_hash,
            reason_code: input.reason_code,
            idempotency_key: input.idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ProcessBlueprintEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PBS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.schema_version",
                reason: "must match PH1PBS_CONTRACT_VERSION",
            });
        }
        if self.process_blueprint_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.process_blueprint_event_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.process_id.validate()?;
        self.blueprint_version.validate()?;
        self.intent_type.validate()?;
        if self.ordered_step_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.ordered_step_count",
                reason: "must be > 0",
            });
        }
        if self.confirmation_step_count > self.ordered_step_count {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.confirmation_step_count",
                reason: "must be <= ordered_step_count",
            });
        }
        validate_id(
            "process_blueprint_event.simulation_requirements_hash",
            &self.simulation_requirements_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "process_blueprint_event.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "process_blueprint_event.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlueprintRegistryRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub intent_type: IntentType,
    pub process_id: ProcessId,
    pub blueprint_version: BlueprintVersion,
    pub status: BlueprintStatus,
    pub source_event_id: u64,
    pub updated_at: MonotonicTimeNs,
}

impl BlueprintRegistryRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        intent_type: IntentType,
        process_id: ProcessId,
        blueprint_version: BlueprintVersion,
        status: BlueprintStatus,
        source_event_id: u64,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1PBS_CONTRACT_VERSION,
            tenant_id,
            intent_type,
            process_id,
            blueprint_version,
            status,
            source_event_id,
            updated_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for BlueprintRegistryRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PBS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "blueprint_registry_record.schema_version",
                reason: "must match PH1PBS_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.intent_type.validate()?;
        self.process_id.validate()?;
        self.blueprint_version.validate()?;
        if self.source_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "blueprint_registry_record.source_event_id",
                reason: "must be > 0",
            });
        }
        if self.updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "blueprint_registry_record.updated_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}
