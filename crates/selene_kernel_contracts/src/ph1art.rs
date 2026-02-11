#![forbid(unsafe_code)]

use crate::{ContractViolation, MonotonicTimeNs, SchemaVersion, Validate};

pub const PH1ART_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

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
pub struct ArtifactVersion(pub u32);

impl Validate for ArtifactVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactScopeType {
    Tenant,
    User,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactType {
    WakePack,
    SttVocabPack,
    SttRoutingPolicyPack,
    SttAdaptationProfile,
    TtsPronunciationPack,
    TtsRoutingPolicyPack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactStatus {
    Active,
    RolledBack,
    Deprecated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLedgerRowInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub scope_type: ArtifactScopeType,
    pub scope_id: String,
    pub artifact_type: ArtifactType,
    pub artifact_version: ArtifactVersion,
    pub package_hash: String,
    pub payload_ref: String,
    pub created_by: String,
    pub provenance_ref: String,
    pub status: ArtifactStatus,
    pub idempotency_key: Option<String>,
}

impl ArtifactLedgerRowInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        scope_type: ArtifactScopeType,
        scope_id: String,
        artifact_type: ArtifactType,
        artifact_version: ArtifactVersion,
        package_hash: String,
        payload_ref: String,
        created_by: String,
        provenance_ref: String,
        status: ArtifactStatus,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            created_at,
            scope_type,
            scope_id,
            artifact_type,
            artifact_version,
            package_hash,
            payload_ref,
            created_by,
            provenance_ref,
            status,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ArtifactLedgerRowInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_ledger_row_input.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_ledger_row_input.created_at",
                reason: "must be > 0",
            });
        }
        validate_id("artifact_ledger_row_input.scope_id", &self.scope_id, 128)?;
        self.artifact_version.validate()?;
        validate_id(
            "artifact_ledger_row_input.package_hash",
            &self.package_hash,
            128,
        )?;
        validate_id(
            "artifact_ledger_row_input.payload_ref",
            &self.payload_ref,
            256,
        )?;
        validate_id("artifact_ledger_row_input.created_by", &self.created_by, 96)?;
        validate_id(
            "artifact_ledger_row_input.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        validate_opt_id(
            "artifact_ledger_row_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLedgerRow {
    pub schema_version: SchemaVersion,
    pub artifact_id: u64,
    pub created_at: MonotonicTimeNs,
    pub scope_type: ArtifactScopeType,
    pub scope_id: String,
    pub artifact_type: ArtifactType,
    pub artifact_version: ArtifactVersion,
    pub package_hash: String,
    pub payload_ref: String,
    pub created_by: String,
    pub provenance_ref: String,
    pub status: ArtifactStatus,
    pub idempotency_key: Option<String>,
}

impl ArtifactLedgerRow {
    pub fn from_input_v1(
        artifact_id: u64,
        input: ArtifactLedgerRowInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            artifact_id,
            created_at: input.created_at,
            scope_type: input.scope_type,
            scope_id: input.scope_id,
            artifact_type: input.artifact_type,
            artifact_version: input.artifact_version,
            package_hash: input.package_hash,
            payload_ref: input.payload_ref,
            created_by: input.created_by,
            provenance_ref: input.provenance_ref,
            status: input.status,
            idempotency_key: input.idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ArtifactLedgerRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_ledger_row.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        if self.artifact_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_ledger_row.artifact_id",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_ledger_row.created_at",
                reason: "must be > 0",
            });
        }
        validate_id("artifact_ledger_row.scope_id", &self.scope_id, 128)?;
        self.artifact_version.validate()?;
        validate_id("artifact_ledger_row.package_hash", &self.package_hash, 128)?;
        validate_id("artifact_ledger_row.payload_ref", &self.payload_ref, 256)?;
        validate_id("artifact_ledger_row.created_by", &self.created_by, 96)?;
        validate_id(
            "artifact_ledger_row.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        validate_opt_id(
            "artifact_ledger_row.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCacheRowInput {
    pub schema_version: SchemaVersion,
    pub tool_name: String,
    pub query_hash: String,
    pub locale: String,
    pub result_payload: String,
    pub expires_at: MonotonicTimeNs,
}

impl ToolCacheRowInput {
    pub fn v1(
        tool_name: String,
        query_hash: String,
        locale: String,
        result_payload: String,
        expires_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            tool_name,
            query_hash,
            locale,
            result_payload,
            expires_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ToolCacheRowInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tool_cache_row_input.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        validate_id("tool_cache_row_input.tool_name", &self.tool_name, 64)?;
        validate_id("tool_cache_row_input.query_hash", &self.query_hash, 128)?;
        validate_id("tool_cache_row_input.locale", &self.locale, 32)?;
        validate_id(
            "tool_cache_row_input.result_payload",
            &self.result_payload,
            8192,
        )?;
        if self.expires_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_cache_row_input.expires_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolCacheRow {
    pub schema_version: SchemaVersion,
    pub cache_id: u64,
    pub tool_name: String,
    pub query_hash: String,
    pub locale: String,
    pub result_payload: String,
    pub expires_at: MonotonicTimeNs,
}

impl ToolCacheRow {
    pub fn from_input_v1(
        cache_id: u64,
        input: ToolCacheRowInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            cache_id,
            tool_name: input.tool_name,
            query_hash: input.query_hash,
            locale: input.locale,
            result_payload: input.result_payload,
            expires_at: input.expires_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ToolCacheRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tool_cache_row.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        if self.cache_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tool_cache_row.cache_id",
                reason: "must be > 0",
            });
        }
        ToolCacheRowInput {
            schema_version: self.schema_version,
            tool_name: self.tool_name.clone(),
            query_hash: self.query_hash.clone(),
            locale: self.locale.clone(),
            result_payload: self.result_payload.clone(),
            expires_at: self.expires_at,
        }
        .validate()
    }
}
