#![forbid(unsafe_code)]

use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1ECM_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

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
pub struct CapabilityMapVersion(pub u32);

impl Validate for CapabilityMapVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "capability_map_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EngineId(String);

impl EngineId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for EngineId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("engine_id", &self.0, 64)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(String);

impl CapabilityId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for CapabilityId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("capability_id", &self.0, 96)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CapabilityMapStatus {
    Draft,
    Active,
    Deprecated,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AllowedCallers {
    SeleneOsOnly,
    SimulationOnly,
    OsAndSimulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SideEffectsMode {
    None,
    Declared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineCapabilityMapEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub engine_id: EngineId,
    pub capability_id: CapabilityId,
    pub capability_map_version: CapabilityMapVersion,
    pub map_status: CapabilityMapStatus,
    pub owning_domain: String,
    pub capability_name: String,
    pub allowed_callers: AllowedCallers,
    pub side_effects_mode: SideEffectsMode,
    pub reads_tables_hash: String,
    pub writes_tables_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl EngineCapabilityMapEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: TenantId,
        engine_id: EngineId,
        capability_id: CapabilityId,
        capability_map_version: CapabilityMapVersion,
        map_status: CapabilityMapStatus,
        owning_domain: String,
        capability_name: String,
        allowed_callers: AllowedCallers,
        side_effects_mode: SideEffectsMode,
        reads_tables_hash: String,
        writes_tables_hash: String,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ECM_CONTRACT_VERSION,
            created_at,
            tenant_id,
            engine_id,
            capability_id,
            capability_map_version,
            map_status,
            owning_domain,
            capability_name,
            allowed_callers,
            side_effects_mode,
            reads_tables_hash,
            writes_tables_hash,
            reason_code,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for EngineCapabilityMapEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ECM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event_input.schema_version",
                reason: "must match PH1ECM_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event_input.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.engine_id.validate()?;
        self.capability_id.validate()?;
        self.capability_map_version.validate()?;
        validate_id(
            "engine_capability_map_event_input.owning_domain",
            &self.owning_domain,
            64,
        )?;
        validate_id(
            "engine_capability_map_event_input.capability_name",
            &self.capability_name,
            128,
        )?;
        validate_id(
            "engine_capability_map_event_input.reads_tables_hash",
            &self.reads_tables_hash,
            128,
        )?;
        validate_id(
            "engine_capability_map_event_input.writes_tables_hash",
            &self.writes_tables_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "engine_capability_map_event_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineCapabilityMapEvent {
    pub schema_version: SchemaVersion,
    pub engine_capability_map_event_id: u64,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub engine_id: EngineId,
    pub capability_id: CapabilityId,
    pub capability_map_version: CapabilityMapVersion,
    pub map_status: CapabilityMapStatus,
    pub owning_domain: String,
    pub capability_name: String,
    pub allowed_callers: AllowedCallers,
    pub side_effects_mode: SideEffectsMode,
    pub reads_tables_hash: String,
    pub writes_tables_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl EngineCapabilityMapEvent {
    pub fn from_input_v1(
        engine_capability_map_event_id: u64,
        input: EngineCapabilityMapEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1ECM_CONTRACT_VERSION,
            engine_capability_map_event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            engine_id: input.engine_id,
            capability_id: input.capability_id,
            capability_map_version: input.capability_map_version,
            map_status: input.map_status,
            owning_domain: input.owning_domain,
            capability_name: input.capability_name,
            allowed_callers: input.allowed_callers,
            side_effects_mode: input.side_effects_mode,
            reads_tables_hash: input.reads_tables_hash,
            writes_tables_hash: input.writes_tables_hash,
            reason_code: input.reason_code,
            idempotency_key: input.idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for EngineCapabilityMapEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ECM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event.schema_version",
                reason: "must match PH1ECM_CONTRACT_VERSION",
            });
        }
        if self.engine_capability_map_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event.engine_capability_map_event_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.engine_id.validate()?;
        self.capability_id.validate()?;
        self.capability_map_version.validate()?;
        validate_id(
            "engine_capability_map_event.owning_domain",
            &self.owning_domain,
            64,
        )?;
        validate_id(
            "engine_capability_map_event.capability_name",
            &self.capability_name,
            128,
        )?;
        validate_id(
            "engine_capability_map_event.reads_tables_hash",
            &self.reads_tables_hash,
            128,
        )?;
        validate_id(
            "engine_capability_map_event.writes_tables_hash",
            &self.writes_tables_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_event.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "engine_capability_map_event.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EngineCapabilityMapCurrentRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub engine_id: EngineId,
    pub capability_id: CapabilityId,
    pub capability_map_version: CapabilityMapVersion,
    pub map_status: CapabilityMapStatus,
    pub owning_domain: String,
    pub capability_name: String,
    pub allowed_callers: AllowedCallers,
    pub side_effects_mode: SideEffectsMode,
    pub source_event_id: u64,
    pub updated_at: MonotonicTimeNs,
}

impl EngineCapabilityMapCurrentRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        engine_id: EngineId,
        capability_id: CapabilityId,
        capability_map_version: CapabilityMapVersion,
        map_status: CapabilityMapStatus,
        owning_domain: String,
        capability_name: String,
        allowed_callers: AllowedCallers,
        side_effects_mode: SideEffectsMode,
        source_event_id: u64,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ECM_CONTRACT_VERSION,
            tenant_id,
            engine_id,
            capability_id,
            capability_map_version,
            map_status,
            owning_domain,
            capability_name,
            allowed_callers,
            side_effects_mode,
            source_event_id,
            updated_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for EngineCapabilityMapCurrentRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ECM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_current_record.schema_version",
                reason: "must match PH1ECM_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.engine_id.validate()?;
        self.capability_id.validate()?;
        self.capability_map_version.validate()?;
        validate_id(
            "engine_capability_map_current_record.owning_domain",
            &self.owning_domain,
            64,
        )?;
        validate_id(
            "engine_capability_map_current_record.capability_name",
            &self.capability_name,
            128,
        )?;
        if self.source_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_current_record.source_event_id",
                reason: "must be > 0",
            });
        }
        if self.updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "engine_capability_map_current_record.updated_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}
