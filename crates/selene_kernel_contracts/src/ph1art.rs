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
    EmoAffectPack,
    EmoPolicyPack,
    SttVocabPack,
    SttRoutingPolicyPack,
    SttAdaptationProfile,
    TtsPronunciationPack,
    TtsRoutingPolicyPack,
    VoiceIdThresholdPack,
    VoiceIdConfusionPairPack,
    VoiceIdSpoofPolicyPack,
    VoiceIdProfileDeltaPack,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtifactTrustRootVersion(pub u32);

impl Validate for ArtifactTrustRootVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactTrustRootKind {
    RootAuthority,
    DomainAuthority,
    ArtifactClassAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactTrustRootState {
    Draft,
    Active,
    Rotating,
    Revoked,
    Expired,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustRootRegistryRowInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub trust_root_id: String,
    pub trust_root_version: ArtifactTrustRootVersion,
    pub trust_root_kind: ArtifactTrustRootKind,
    pub signer_identity_id: String,
    pub state: ArtifactTrustRootState,
    pub parent_trust_root_id: Option<String>,
    pub lineage_root_trust_root_id: String,
    pub crypto_suite_version: String,
    pub revocation_ref: Option<String>,
    pub idempotency_key: Option<String>,
}

impl ArtifactTrustRootRegistryRowInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        trust_root_id: String,
        trust_root_version: ArtifactTrustRootVersion,
        trust_root_kind: ArtifactTrustRootKind,
        signer_identity_id: String,
        state: ArtifactTrustRootState,
        parent_trust_root_id: Option<String>,
        lineage_root_trust_root_id: String,
        crypto_suite_version: String,
        revocation_ref: Option<String>,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            created_at,
            trust_root_id,
            trust_root_version,
            trust_root_kind,
            signer_identity_id,
            state,
            parent_trust_root_id,
            lineage_root_trust_root_id,
            crypto_suite_version,
            revocation_ref,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ArtifactTrustRootRegistryRowInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root_registry_row_input.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root_registry_row_input.created_at",
                reason: "must be > 0",
            });
        }
        validate_id(
            "artifact_trust_root_registry_row_input.trust_root_id",
            &self.trust_root_id,
            128,
        )?;
        self.trust_root_version.validate()?;
        validate_id(
            "artifact_trust_root_registry_row_input.signer_identity_id",
            &self.signer_identity_id,
            128,
        )?;
        validate_opt_id(
            "artifact_trust_root_registry_row_input.parent_trust_root_id",
            &self.parent_trust_root_id,
            128,
        )?;
        validate_id(
            "artifact_trust_root_registry_row_input.lineage_root_trust_root_id",
            &self.lineage_root_trust_root_id,
            128,
        )?;
        validate_id(
            "artifact_trust_root_registry_row_input.crypto_suite_version",
            &self.crypto_suite_version,
            64,
        )?;
        validate_opt_id(
            "artifact_trust_root_registry_row_input.revocation_ref",
            &self.revocation_ref,
            128,
        )?;
        validate_opt_id(
            "artifact_trust_root_registry_row_input.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustRootRegistryRow {
    pub schema_version: SchemaVersion,
    pub trust_root_registry_row_id: u64,
    pub created_at: MonotonicTimeNs,
    pub trust_root_id: String,
    pub trust_root_version: ArtifactTrustRootVersion,
    pub trust_root_kind: ArtifactTrustRootKind,
    pub signer_identity_id: String,
    pub state: ArtifactTrustRootState,
    pub parent_trust_root_id: Option<String>,
    pub lineage_root_trust_root_id: String,
    pub crypto_suite_version: String,
    pub revocation_ref: Option<String>,
    pub idempotency_key: Option<String>,
}

impl ArtifactTrustRootRegistryRow {
    pub fn from_input_v1(
        trust_root_registry_row_id: u64,
        input: ArtifactTrustRootRegistryRowInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let r = Self {
            schema_version: PH1ART_CONTRACT_VERSION,
            trust_root_registry_row_id,
            created_at: input.created_at,
            trust_root_id: input.trust_root_id,
            trust_root_version: input.trust_root_version,
            trust_root_kind: input.trust_root_kind,
            signer_identity_id: input.signer_identity_id,
            state: input.state,
            parent_trust_root_id: input.parent_trust_root_id,
            lineage_root_trust_root_id: input.lineage_root_trust_root_id,
            crypto_suite_version: input.crypto_suite_version,
            revocation_ref: input.revocation_ref,
            idempotency_key: input.idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ArtifactTrustRootRegistryRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1ART_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root_registry_row.schema_version",
                reason: "must match PH1ART_CONTRACT_VERSION",
            });
        }
        if self.trust_root_registry_row_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root_registry_row.trust_root_registry_row_id",
                reason: "must be > 0",
            });
        }
        ArtifactTrustRootRegistryRowInput {
            schema_version: self.schema_version,
            created_at: self.created_at,
            trust_root_id: self.trust_root_id.clone(),
            trust_root_version: self.trust_root_version,
            trust_root_kind: self.trust_root_kind,
            signer_identity_id: self.signer_identity_id.clone(),
            state: self.state,
            parent_trust_root_id: self.parent_trust_root_id.clone(),
            lineage_root_trust_root_id: self.lineage_root_trust_root_id.clone(),
            crypto_suite_version: self.crypto_suite_version.clone(),
            revocation_ref: self.revocation_ref.clone(),
            idempotency_key: self.idempotency_key.clone(),
        }
        .validate()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn artifact_trust_root_registry_input_rejects_zero_version() {
        let out = ArtifactTrustRootRegistryRowInput::v1(
            MonotonicTimeNs(1),
            "root.selene".to_string(),
            ArtifactTrustRootVersion(0),
            ArtifactTrustRootKind::RootAuthority,
            "SELENE_ROOT_CA".to_string(),
            ArtifactTrustRootState::Active,
            None,
            "root.selene".to_string(),
            "ed25519-sha256-v1".to_string(),
            None,
            Some("idem_root".to_string()),
        );
        assert!(out.is_err());
    }

    #[test]
    fn artifact_trust_root_registry_row_round_trips_from_input() {
        let row = ArtifactTrustRootRegistryRow::from_input_v1(
            7,
            ArtifactTrustRootRegistryRowInput::v1(
                MonotonicTimeNs(10),
                "root.selene".to_string(),
                ArtifactTrustRootVersion(1),
                ArtifactTrustRootKind::RootAuthority,
                "SELENE_ROOT_CA".to_string(),
                ArtifactTrustRootState::Active,
                None,
                "root.selene".to_string(),
                "ed25519-sha256-v1".to_string(),
                None,
                Some("idem_root".to_string()),
            )
            .unwrap(),
        )
        .unwrap();

        assert_eq!(row.trust_root_registry_row_id, 7);
        assert_eq!(row.trust_root_id, "root.selene");
        assert_eq!(row.lineage_root_trust_root_id, "root.selene");
        assert_eq!(row.state, ArtifactTrustRootState::Active);
    }
}
