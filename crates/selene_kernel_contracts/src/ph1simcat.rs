#![forbid(unsafe_code)]

use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SIMCAT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

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
pub struct SimulationVersion(pub u32);

impl Validate for SimulationVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SimulationId(String);

impl SimulationId {
    pub fn new(v: impl Into<String>) -> Result<Self, ContractViolation> {
        let v = Self(v.into());
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for SimulationId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("simulation_id", &self.0, 96)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationType {
    Draft,
    Commit,
    Revoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationStatus {
    Draft,
    Active,
    Deprecated,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationCatalogEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub simulation_id: SimulationId,
    pub simulation_version: SimulationVersion,
    pub simulation_type: SimulationType,
    pub status: SimulationStatus,
    pub owning_domain: String,
    pub reads_tables_hash: String,
    pub writes_tables_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl SimulationCatalogEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: TenantId,
        simulation_id: SimulationId,
        simulation_version: SimulationVersion,
        simulation_type: SimulationType,
        status: SimulationStatus,
        owning_domain: String,
        reads_tables_hash: String,
        writes_tables_hash: String,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SIMCAT_CONTRACT_VERSION,
            created_at,
            tenant_id,
            simulation_id,
            simulation_version,
            simulation_type,
            status,
            owning_domain,
            reads_tables_hash,
            writes_tables_hash,
            reason_code,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SimulationCatalogEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SIMCAT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event_input.schema_version",
                reason: "must match PH1SIMCAT_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event_input.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.simulation_id.validate()?;
        self.simulation_version.validate()?;
        validate_id(
            "simulation_catalog_event_input.owning_domain",
            &self.owning_domain,
            64,
        )?;
        validate_id(
            "simulation_catalog_event_input.reads_tables_hash",
            &self.reads_tables_hash,
            128,
        )?;
        validate_id(
            "simulation_catalog_event_input.writes_tables_hash",
            &self.writes_tables_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "simulation_catalog_event_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationCatalogEvent {
    pub schema_version: SchemaVersion,
    pub simulation_catalog_event_id: u64,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: TenantId,
    pub simulation_id: SimulationId,
    pub simulation_version: SimulationVersion,
    pub simulation_type: SimulationType,
    pub status: SimulationStatus,
    pub owning_domain: String,
    pub reads_tables_hash: String,
    pub writes_tables_hash: String,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl SimulationCatalogEvent {
    pub fn from_input_v1(
        simulation_catalog_event_id: u64,
        input: SimulationCatalogEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1SIMCAT_CONTRACT_VERSION,
            simulation_catalog_event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            simulation_id: input.simulation_id,
            simulation_version: input.simulation_version,
            simulation_type: input.simulation_type,
            status: input.status,
            owning_domain: input.owning_domain,
            reads_tables_hash: input.reads_tables_hash,
            writes_tables_hash: input.writes_tables_hash,
            reason_code: input.reason_code,
            idempotency_key: input.idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SimulationCatalogEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SIMCAT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event.schema_version",
                reason: "must match PH1SIMCAT_CONTRACT_VERSION",
            });
        }
        if self.simulation_catalog_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event.simulation_catalog_event_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event.created_at",
                reason: "must be > 0",
            });
        }
        self.tenant_id.validate()?;
        self.simulation_id.validate()?;
        self.simulation_version.validate()?;
        validate_id(
            "simulation_catalog_event.owning_domain",
            &self.owning_domain,
            64,
        )?;
        validate_id(
            "simulation_catalog_event.reads_tables_hash",
            &self.reads_tables_hash,
            128,
        )?;
        validate_id(
            "simulation_catalog_event.writes_tables_hash",
            &self.writes_tables_hash,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_event.reason_code",
                reason: "must be > 0",
            });
        }
        validate_opt_id(
            "simulation_catalog_event.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimulationCatalogCurrentRecord {
    pub schema_version: SchemaVersion,
    pub tenant_id: TenantId,
    pub simulation_id: SimulationId,
    pub simulation_version: SimulationVersion,
    pub simulation_type: SimulationType,
    pub status: SimulationStatus,
    pub owning_domain: String,
    pub source_event_id: u64,
    pub updated_at: MonotonicTimeNs,
}

impl SimulationCatalogCurrentRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        tenant_id: TenantId,
        simulation_id: SimulationId,
        simulation_version: SimulationVersion,
        simulation_type: SimulationType,
        status: SimulationStatus,
        owning_domain: String,
        source_event_id: u64,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SIMCAT_CONTRACT_VERSION,
            tenant_id,
            simulation_id,
            simulation_version,
            simulation_type,
            status,
            owning_domain,
            source_event_id,
            updated_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SimulationCatalogCurrentRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SIMCAT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_current_record.schema_version",
                reason: "must match PH1SIMCAT_CONTRACT_VERSION",
            });
        }
        self.tenant_id.validate()?;
        self.simulation_id.validate()?;
        self.simulation_version.validate()?;
        validate_id(
            "simulation_catalog_current_record.owning_domain",
            &self.owning_domain,
            64,
        )?;
        if self.source_event_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_current_record.source_event_id",
                reason: "must be > 0",
            });
        }
        if self.updated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_catalog_current_record.updated_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}
