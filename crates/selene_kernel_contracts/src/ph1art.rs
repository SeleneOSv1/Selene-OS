#![forbid(unsafe_code)]

use crate::{ContractViolation, MonotonicTimeNs, SchemaVersion, Validate};
use std::collections::BTreeSet;

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

fn validate_string_list_unique(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max items",
        });
    }
    let mut seen = BTreeSet::new();
    for value in values {
        validate_id(field, value, max_len)?;
        if !seen.insert(value.clone()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "contains duplicate value",
            });
        }
    }
    Ok(())
}

macro_rules! string_contract_type {
    ($name:ident, $field:literal, $max_len:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub String);

        impl Validate for $name {
            fn validate(&self) -> Result<(), ContractViolation> {
                validate_id($field, &self.0, $max_len)
            }
        }
    };
}

string_contract_type!(ArtifactIdentityRef, "artifact_identity_ref", 128);
string_contract_type!(ArtifactTrustBindingRef, "artifact_trust_binding_ref", 128);
string_contract_type!(
    ArtifactSignerIdentityRef,
    "artifact_signer_identity_ref",
    128
);
string_contract_type!(ArtifactDependencySetRef, "artifact_dependency_set_ref", 128);
string_contract_type!(
    ArtifactLifecycleStateRef,
    "artifact_lifecycle_state_ref",
    128
);
string_contract_type!(TrustPolicySnapshotRef, "trust_policy_snapshot_ref", 128);
string_contract_type!(TrustSetSnapshotRef, "trust_set_snapshot_ref", 128);
string_contract_type!(
    NegativeVerificationResultRef,
    "negative_verification_result_ref",
    128
);
string_contract_type!(ArtifactTrustDecisionId, "artifact_trust_decision_id", 128);
string_contract_type!(
    ArtifactTrustProofEntryRef,
    "artifact_trust_proof_entry_ref",
    128
);
string_contract_type!(
    ArtifactTrustProofRecordRef,
    "artifact_trust_proof_record_ref",
    128
);
string_contract_type!(ArtifactScopeFingerprint, "artifact_scope_fingerprint", 128);
string_contract_type!(
    VerificationBasisFingerprint,
    "verification_basis_fingerprint",
    128
);
string_contract_type!(
    HistoricalTrustSnapshotRef,
    "historical_trust_snapshot_ref",
    128
);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactClass {
    SimulationDefinition,
    SimulationWorkflow,
    RuntimePolicyBundle,
    WakeArtifact,
    VoiceIdentityArtifact,
    SttAdaptationArtifact,
    TtsPolicyArtifact,
    EmoPolicyArtifact,
    BuilderOutput,
    DeploymentPackage,
    LearningPromotionArtifact,
    PaePolicyArtifact,
    SelfHealArtifact,
    MemoryHintArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactDigestAlgorithm {
    Sha256,
    Sha512,
    Blake3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactDigestEncoding {
    Hex,
    Base64Url,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactDigestBoundary {
    WholePayload,
    ManifestOnly,
    ManifestAndPayload,
    ComponentTree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDigest {
    pub artifact_digest: String,
    pub artifact_digest_algorithm: ArtifactDigestAlgorithm,
    pub artifact_digest_encoding: ArtifactDigestEncoding,
    pub artifact_digest_boundary: ArtifactDigestBoundary,
}

impl ArtifactDigest {
    pub fn new(
        artifact_digest: String,
        artifact_digest_algorithm: ArtifactDigestAlgorithm,
        artifact_digest_encoding: ArtifactDigestEncoding,
        artifact_digest_boundary: ArtifactDigestBoundary,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            artifact_digest,
            artifact_digest_algorithm,
            artifact_digest_encoding,
            artifact_digest_boundary,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ArtifactDigest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_digest.artifact_digest",
            &self.artifact_digest,
            256,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPayloadComponentDigest {
    pub component_ref: String,
    pub digest: ArtifactDigest,
}

impl Validate for ArtifactPayloadComponentDigest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_payload_component_digest.component_ref",
            &self.component_ref,
            128,
        )?;
        self.digest.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactBundleIntegrityMode {
    ManifestOnly,
    PayloadOnly,
    ManifestAndPayload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactBundleOrderingMode {
    DeclaredOrder,
    LexicographicPath,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBundleEntry {
    pub entry_ref: String,
    pub entry_order_key: String,
    pub payload_ref: String,
    pub payload_digest: ArtifactDigest,
}

impl Validate for ArtifactBundleEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("artifact_bundle_entry.entry_ref", &self.entry_ref, 128)?;
        validate_id(
            "artifact_bundle_entry.entry_order_key",
            &self.entry_order_key,
            128,
        )?;
        validate_id("artifact_bundle_entry.payload_ref", &self.payload_ref, 256)?;
        self.payload_digest.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactBundleManifest {
    pub bundle_manifest_ref: String,
    pub integrity_mode: ArtifactBundleIntegrityMode,
    pub ordering_mode: ArtifactBundleOrderingMode,
    pub manifest_absent_for_single_payload: bool,
    pub manifest_digest: Option<ArtifactDigest>,
    pub bundle_summary_digest: Option<ArtifactDigest>,
    pub entries: Vec<ArtifactBundleEntry>,
    pub component_digests: Vec<ArtifactPayloadComponentDigest>,
}

impl Validate for ArtifactBundleManifest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_bundle_manifest.bundle_manifest_ref",
            &self.bundle_manifest_ref,
            128,
        )?;
        if self.entries.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_bundle_manifest.entries",
                reason: "must not be empty",
            });
        }
        if matches!(
            self.integrity_mode,
            ArtifactBundleIntegrityMode::ManifestOnly
                | ArtifactBundleIntegrityMode::ManifestAndPayload
        ) && self.manifest_digest.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_bundle_manifest.manifest_digest",
                reason: "required for selected integrity mode",
            });
        }
        if self.manifest_absent_for_single_payload && self.entries.len() != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_bundle_manifest.manifest_absent_for_single_payload",
                reason: "only valid for single-payload bundles",
            });
        }
        let mut entry_refs = BTreeSet::new();
        let mut order_keys = BTreeSet::new();
        for entry in &self.entries {
            entry.validate()?;
            if !entry_refs.insert(entry.entry_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_bundle_manifest.entries",
                    reason: "contains duplicate entry_ref",
                });
            }
            if !order_keys.insert(entry.entry_order_key.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_bundle_manifest.entries",
                    reason: "contains duplicate entry_order_key",
                });
            }
        }
        if let Some(manifest_digest) = &self.manifest_digest {
            manifest_digest.validate()?;
        }
        if let Some(bundle_summary_digest) = &self.bundle_summary_digest {
            bundle_summary_digest.validate()?;
        }
        let mut component_refs = BTreeSet::new();
        for component_digest in &self.component_digests {
            component_digest.validate()?;
            if !component_refs.insert(component_digest.component_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_bundle_manifest.component_digests",
                    reason: "contains duplicate component_ref",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactDependencyValidationPosture {
    Strict,
    GovernedCompatibilityWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactDependencyConflictClass {
    MissingRequiredDependency,
    IncompatibleDependency,
    DependencyCycleDetected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactDependencySet {
    pub dependency_set_ref: ArtifactDependencySetRef,
    pub required_artifact_refs: Vec<ArtifactIdentityRef>,
    pub optional_artifact_refs: Vec<ArtifactIdentityRef>,
    pub incompatible_artifact_refs: Vec<ArtifactIdentityRef>,
    pub minimum_runtime_contract_version: String,
    pub compatible_platform_classes: Vec<String>,
    pub compatible_environment_classes: Vec<String>,
    pub dependency_validation_posture: ArtifactDependencyValidationPosture,
}

impl Validate for ArtifactDependencySet {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.dependency_set_ref.validate()?;
        validate_id(
            "artifact_dependency_set.minimum_runtime_contract_version",
            &self.minimum_runtime_contract_version,
            64,
        )?;

        let mut required = BTreeSet::new();
        for artifact_ref in &self.required_artifact_refs {
            artifact_ref.validate()?;
            if !required.insert(artifact_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_dependency_set.required_artifact_refs",
                    reason: "contains duplicate ref",
                });
            }
        }

        let mut optional = BTreeSet::new();
        for artifact_ref in &self.optional_artifact_refs {
            artifact_ref.validate()?;
            if !optional.insert(artifact_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_dependency_set.optional_artifact_refs",
                    reason: "contains duplicate ref",
                });
            }
        }

        let mut incompatible = BTreeSet::new();
        for artifact_ref in &self.incompatible_artifact_refs {
            artifact_ref.validate()?;
            if !incompatible.insert(artifact_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_dependency_set.incompatible_artifact_refs",
                    reason: "contains duplicate ref",
                });
            }
        }

        validate_string_list_unique(
            "artifact_dependency_set.compatible_platform_classes",
            &self.compatible_platform_classes,
            32,
            64,
        )?;
        validate_string_list_unique(
            "artifact_dependency_set.compatible_environment_classes",
            &self.compatible_environment_classes,
            32,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactCertificationStateValue {
    Draft,
    TestCertified,
    RuntimeCertified,
    Quarantined,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactCertificationState {
    pub state: ArtifactCertificationStateValue,
    pub certified_by_identity_ref: Option<ArtifactSignerIdentityRef>,
    pub certified_at: Option<MonotonicTimeNs>,
    pub certification_ref: Option<String>,
    pub certification_policy_version: Option<String>,
}

impl Validate for ArtifactCertificationState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(certified_by_identity_ref) = &self.certified_by_identity_ref {
            certified_by_identity_ref.validate()?;
        }
        if let Some(certified_at) = self.certified_at {
            if certified_at.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_certification_state.certified_at",
                    reason: "must be > 0",
                });
            }
        }
        validate_opt_id(
            "artifact_certification_state.certification_ref",
            &self.certification_ref,
            128,
        )?;
        validate_opt_id(
            "artifact_certification_state.certification_policy_version",
            &self.certification_policy_version,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactRevocationStateValue {
    NotRevoked,
    Revoked,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRevocationState {
    pub state: ArtifactRevocationStateValue,
    pub revocation_ref: Option<String>,
}

impl Validate for ArtifactRevocationState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_opt_id(
            "artifact_revocation_state.revocation_ref",
            &self.revocation_ref,
            128,
        )?;
        if self.state == ArtifactRevocationStateValue::Revoked && self.revocation_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_revocation_state.revocation_ref",
                reason: "required when revoked",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactLifecycleStage {
    Publication,
    Distribution,
    Delivery,
    Installation,
    ApplyRequest,
    ApplyReceipt,
    Activation,
    RuntimeUse,
    Replacement,
    Rollback,
    Revocation,
    Retirement,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLifecycleState {
    pub lifecycle_state_ref: ArtifactLifecycleStateRef,
    pub stage: ArtifactLifecycleStage,
    pub entered_at: MonotonicTimeNs,
    pub previous_stage: Option<ArtifactLifecycleStage>,
}

impl Validate for ArtifactLifecycleState {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.lifecycle_state_ref.validate()?;
        if self.entered_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_lifecycle_state.entered_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactScopeDimension {
    Tenant,
    Environment,
    Runtime,
    Platform,
    Rollout,
    FeatureFlag,
    Identity,
    Device,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactScopeWildcardPosture {
    NoWildcard,
    ExplicitWildcardOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactScopeInheritancePosture {
    NoInheritance,
    ExplicitInheritanceOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactScope {
    pub tenant_scope_id: Option<String>,
    pub environment_scope_id: String,
    pub runtime_scope_id: String,
    pub platform_scope_id: Option<String>,
    pub rollout_scope_id: Option<String>,
    pub feature_flag_scope_ids: Vec<String>,
    pub identity_scope_id: Option<String>,
    pub device_scope_id: Option<String>,
}

impl Validate for ArtifactScope {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_opt_id("artifact_scope.tenant_scope_id", &self.tenant_scope_id, 128)?;
        validate_id(
            "artifact_scope.environment_scope_id",
            &self.environment_scope_id,
            128,
        )?;
        validate_id(
            "artifact_scope.runtime_scope_id",
            &self.runtime_scope_id,
            128,
        )?;
        validate_opt_id(
            "artifact_scope.platform_scope_id",
            &self.platform_scope_id,
            128,
        )?;
        validate_opt_id(
            "artifact_scope.rollout_scope_id",
            &self.rollout_scope_id,
            128,
        )?;
        validate_string_list_unique(
            "artifact_scope.feature_flag_scope_ids",
            &self.feature_flag_scope_ids,
            32,
            128,
        )?;
        validate_opt_id(
            "artifact_scope.identity_scope_id",
            &self.identity_scope_id,
            128,
        )?;
        validate_opt_id("artifact_scope.device_scope_id", &self.device_scope_id, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactScopeEvaluationPolicy {
    pub precedence_order: Vec<ArtifactScopeDimension>,
    pub wildcard_posture: ArtifactScopeWildcardPosture,
    pub inheritance_posture: ArtifactScopeInheritancePosture,
    pub narrowing_allowed: bool,
    pub widening_requires_governance: bool,
}

impl Validate for ArtifactScopeEvaluationPolicy {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.precedence_order.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_scope_evaluation_policy.precedence_order",
                reason: "must not be empty",
            });
        }
        let mut seen = BTreeSet::new();
        for dimension in &self.precedence_order {
            if !seen.insert(*dimension) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_scope_evaluation_policy.precedence_order",
                    reason: "contains duplicate dimension",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactLineage {
    pub lineage_parent_artifact_ref: Option<ArtifactIdentityRef>,
    pub lineage_root_artifact_ref: ArtifactIdentityRef,
    pub replacement_of_artifact_ref: Option<ArtifactIdentityRef>,
    pub rollback_target_artifact_ref: Option<ArtifactIdentityRef>,
}

impl Validate for ArtifactLineage {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(lineage_parent_artifact_ref) = &self.lineage_parent_artifact_ref {
            lineage_parent_artifact_ref.validate()?;
        }
        self.lineage_root_artifact_ref.validate()?;
        if let Some(replacement_of_artifact_ref) = &self.replacement_of_artifact_ref {
            replacement_of_artifact_ref.validate()?;
        }
        if let Some(rollback_target_artifact_ref) = &self.rollback_target_artifact_ref {
            rollback_target_artifact_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub artifact_class: ArtifactClass,
    pub artifact_type_name: String,
    pub artifact_version: ArtifactVersion,
    pub artifact_schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub payload_ref: Option<String>,
    pub scope: ArtifactScope,
    pub lineage: ArtifactLineage,
    pub lifecycle_state_ref: Option<ArtifactLifecycleStateRef>,
    pub dependency_set_ref: Option<ArtifactDependencySetRef>,
}

impl Validate for ArtifactIdentity {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.artifact_identity_ref.validate()?;
        validate_id(
            "artifact_identity.artifact_type_name",
            &self.artifact_type_name,
            128,
        )?;
        self.artifact_version.validate()?;
        if self.artifact_schema_version.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_identity.artifact_schema_version",
                reason: "must be > 0",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_identity.created_at",
                reason: "must be > 0",
            });
        }
        validate_opt_id("artifact_identity.payload_ref", &self.payload_ref, 256)?;
        self.scope.validate()?;
        self.lineage.validate()?;
        if let Some(lifecycle_state_ref) = &self.lifecycle_state_ref {
            lifecycle_state_ref.validate()?;
        }
        if let Some(dependency_set_ref) = &self.dependency_set_ref {
            dependency_set_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactAuthorityDomains {
    pub signing_authority_domain: String,
    pub certification_authority_domain: String,
    pub revocation_authority_domain: String,
    pub verification_authority_domain: String,
}

impl Validate for ArtifactAuthorityDomains {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_authority_domains.signing_authority_domain",
            &self.signing_authority_domain,
            128,
        )?;
        validate_id(
            "artifact_authority_domains.certification_authority_domain",
            &self.certification_authority_domain,
            128,
        )?;
        validate_id(
            "artifact_authority_domains.revocation_authority_domain",
            &self.revocation_authority_domain,
            128,
        )?;
        validate_id(
            "artifact_authority_domains.verification_authority_domain",
            &self.verification_authority_domain,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustRoot {
    pub trust_root_id: String,
    pub trust_root_version: ArtifactTrustRootVersion,
    pub trust_root_kind: ArtifactTrustRootKind,
    pub state: ArtifactTrustRootState,
    pub signer_identity_ref: ArtifactSignerIdentityRef,
    pub parent_trust_root_id: Option<String>,
    pub lineage_root_trust_root_id: String,
    pub crypto_suite_version: String,
    pub allowed_artifact_classes: Vec<ArtifactClass>,
    pub authority_domains: ArtifactAuthorityDomains,
}

impl Validate for ArtifactTrustRoot {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_trust_root.trust_root_id",
            &self.trust_root_id,
            128,
        )?;
        self.trust_root_version.validate()?;
        self.signer_identity_ref.validate()?;
        validate_opt_id(
            "artifact_trust_root.parent_trust_root_id",
            &self.parent_trust_root_id,
            128,
        )?;
        validate_id(
            "artifact_trust_root.lineage_root_trust_root_id",
            &self.lineage_root_trust_root_id,
            128,
        )?;
        validate_id(
            "artifact_trust_root.crypto_suite_version",
            &self.crypto_suite_version,
            64,
        )?;
        if self.allowed_artifact_classes.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_root.allowed_artifact_classes",
                reason: "must not be empty",
            });
        }
        self.authority_domains.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactSignerState {
    Active,
    Rotating,
    Revoked,
    Expired,
    Retired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactSignerIdentity {
    pub artifact_signer_identity_ref: ArtifactSignerIdentityRef,
    pub key_id: String,
    pub trust_root_id: String,
    pub trust_root_version: ArtifactTrustRootVersion,
    pub state: ArtifactSignerState,
    pub crypto_suite_version: String,
    pub allowed_artifact_classes: Vec<ArtifactClass>,
}

impl Validate for ArtifactSignerIdentity {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.artifact_signer_identity_ref.validate()?;
        validate_id("artifact_signer_identity.key_id", &self.key_id, 128)?;
        validate_id(
            "artifact_signer_identity.trust_root_id",
            &self.trust_root_id,
            128,
        )?;
        self.trust_root_version.validate()?;
        validate_id(
            "artifact_signer_identity.crypto_suite_version",
            &self.crypto_suite_version,
            64,
        )?;
        if self.allowed_artifact_classes.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_signer_identity.allowed_artifact_classes",
                reason: "must not be empty",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustBinding {
    pub artifact_trust_binding_ref: ArtifactTrustBindingRef,
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub digest: ArtifactDigest,
    pub bundle_manifest: Option<ArtifactBundleManifest>,
    pub artifact_signer_identity_ref: ArtifactSignerIdentityRef,
    pub trust_root_id: String,
    pub trust_root_version: ArtifactTrustRootVersion,
    pub certification_state: ArtifactCertificationState,
    pub revocation_state: ArtifactRevocationState,
    pub dependency_set_ref: Option<ArtifactDependencySetRef>,
}

impl Validate for ArtifactTrustBinding {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.artifact_trust_binding_ref.validate()?;
        self.artifact_identity_ref.validate()?;
        self.digest.validate()?;
        if let Some(bundle_manifest) = &self.bundle_manifest {
            bundle_manifest.validate()?;
        }
        self.artifact_signer_identity_ref.validate()?;
        validate_id(
            "artifact_trust_binding.trust_root_id",
            &self.trust_root_id,
            128,
        )?;
        self.trust_root_version.validate()?;
        self.certification_state.validate()?;
        self.revocation_state.validate()?;
        if let Some(dependency_set_ref) = &self.dependency_set_ref {
            dependency_set_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum LegacyCompatibilityClass {
    StrictNoLegacy,
    GovernedCompatibilityWindow,
    LegacyBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactVerificationCachePolicyClass {
    NoCache,
    PositiveOnly,
    PositiveAndNegativeBounded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustPolicySnapshot {
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_policy_version: String,
    pub trust_set_version: String,
    pub crypto_suite_version: String,
    pub time_authority_basis: String,
    pub policy_matrix_hash: String,
    pub generated_at: MonotonicTimeNs,
    pub legacy_compatibility_class: LegacyCompatibilityClass,
    pub verification_cache_policy_class: ArtifactVerificationCachePolicyClass,
}

impl Validate for TrustPolicySnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.trust_policy_snapshot_ref.validate()?;
        validate_id(
            "trust_policy_snapshot.trust_policy_version",
            &self.trust_policy_version,
            64,
        )?;
        validate_id(
            "trust_policy_snapshot.trust_set_version",
            &self.trust_set_version,
            64,
        )?;
        validate_id(
            "trust_policy_snapshot.crypto_suite_version",
            &self.crypto_suite_version,
            64,
        )?;
        validate_id(
            "trust_policy_snapshot.time_authority_basis",
            &self.time_authority_basis,
            128,
        )?;
        validate_id(
            "trust_policy_snapshot.policy_matrix_hash",
            &self.policy_matrix_hash,
            128,
        )?;
        if self.generated_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "trust_policy_snapshot.generated_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrustSetClusterConsistencyPosture {
    Consistent,
    Degraded,
    Diverged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum TrustSetMemberKind {
    TrustRoot,
    SignerIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustSetMember {
    pub member_ref: String,
    pub member_kind: TrustSetMemberKind,
    pub trust_root_id: Option<String>,
    pub artifact_signer_identity_ref: Option<ArtifactSignerIdentityRef>,
}

impl Validate for TrustSetMember {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("trust_set_member.member_ref", &self.member_ref, 128)?;
        validate_opt_id("trust_set_member.trust_root_id", &self.trust_root_id, 128)?;
        if let Some(artifact_signer_identity_ref) = &self.artifact_signer_identity_ref {
            artifact_signer_identity_ref.validate()?;
        }
        match self.member_kind {
            TrustSetMemberKind::TrustRoot if self.trust_root_id.is_none() => {
                Err(ContractViolation::InvalidValue {
                    field: "trust_set_member.trust_root_id",
                    reason: "required for trust root member",
                })
            }
            TrustSetMemberKind::SignerIdentity if self.artifact_signer_identity_ref.is_none() => {
                Err(ContractViolation::InvalidValue {
                    field: "trust_set_member.artifact_signer_identity_ref",
                    reason: "required for signer identity member",
                })
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrustSetSnapshot {
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub trust_set_version: String,
    pub snapshot_epoch: u64,
    pub archived_snapshot_ref: Option<HistoricalTrustSnapshotRef>,
    pub cluster_consistency_posture: TrustSetClusterConsistencyPosture,
    pub members: Vec<TrustSetMember>,
}

impl Validate for TrustSetSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.trust_set_snapshot_ref.validate()?;
        validate_id(
            "trust_set_snapshot.trust_set_version",
            &self.trust_set_version,
            64,
        )?;
        if self.snapshot_epoch == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "trust_set_snapshot.snapshot_epoch",
                reason: "must be > 0",
            });
        }
        if let Some(archived_snapshot_ref) = &self.archived_snapshot_ref {
            archived_snapshot_ref.validate()?;
        }
        if self.members.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "trust_set_snapshot.members",
                reason: "must not be empty",
            });
        }
        let mut member_refs = BTreeSet::new();
        for member in &self.members {
            member.validate()?;
            if !member_refs.insert(member.member_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "trust_set_snapshot.members",
                    reason: "contains duplicate member_ref",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactVerificationOutcome {
    VerifiedFresh,
    VerifiedCached,
    DegradedVerified,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactVerificationFailureClass {
    HashMismatch,
    SignatureInvalid,
    TrustRootUnknown,
    TrustRootRevoked,
    ArtifactRevoked,
    ArtifactExpired,
    CertificationInvalid,
    LineageInvalid,
    ScopeInvalid,
    CryptoSuiteUnsupported,
    TimeAuthorityUnavailable,
    VerificationUnavailable,
    CacheBasisInvalid,
    LegacyBlocked,
    ClusterTrustDivergence,
    HistoricalSnapshotMissing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactVerificationResult {
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub artifact_trust_binding_ref: ArtifactTrustBindingRef,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub verification_basis_fingerprint: VerificationBasisFingerprint,
    pub artifact_verification_outcome: ArtifactVerificationOutcome,
    pub artifact_verification_failure_class: Option<ArtifactVerificationFailureClass>,
    pub negative_verification_result_ref: Option<NegativeVerificationResultRef>,
    pub verification_timestamp: MonotonicTimeNs,
    pub verification_cache_used: bool,
    pub historical_snapshot_ref: Option<HistoricalTrustSnapshotRef>,
}

impl Validate for ArtifactVerificationResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.artifact_identity_ref.validate()?;
        self.artifact_trust_binding_ref.validate()?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        self.verification_basis_fingerprint.validate()?;
        if let Some(negative_verification_result_ref) = &self.negative_verification_result_ref {
            negative_verification_result_ref.validate()?;
        }
        if self.verification_timestamp.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_verification_result.verification_timestamp",
                reason: "must be > 0",
            });
        }
        if let Some(historical_snapshot_ref) = &self.historical_snapshot_ref {
            historical_snapshot_ref.validate()?;
        }
        match self.artifact_verification_outcome {
            ArtifactVerificationOutcome::Failed
                if self.artifact_verification_failure_class.is_none() =>
            {
                Err(ContractViolation::InvalidValue {
                    field: "artifact_verification_result.artifact_verification_failure_class",
                    reason: "required for failed verification outcome",
                })
            }
            ArtifactVerificationOutcome::VerifiedFresh
            | ArtifactVerificationOutcome::VerifiedCached
            | ArtifactVerificationOutcome::DegradedVerified
                if self.artifact_verification_failure_class.is_some() =>
            {
                Err(ContractViolation::InvalidValue {
                    field: "artifact_verification_result.artifact_verification_failure_class",
                    reason: "must be absent for non-failed verification outcome",
                })
            }
            _ => Ok(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NegativeArtifactVerificationResult {
    pub negative_verification_result_ref: NegativeVerificationResultRef,
    pub authority_decision_id: ArtifactTrustDecisionId,
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub verification_failure_class: ArtifactVerificationFailureClass,
    pub reason_codes: Vec<String>,
    pub captured_at: MonotonicTimeNs,
    pub durable: bool,
    pub cacheable: bool,
}

impl Validate for NegativeArtifactVerificationResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.negative_verification_result_ref.validate()?;
        self.authority_decision_id.validate()?;
        self.artifact_identity_ref.validate()?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        validate_string_list_unique(
            "negative_artifact_verification_result.reason_codes",
            &self.reason_codes,
            32,
            128,
        )?;
        if self.reason_codes.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "negative_artifact_verification_result.reason_codes",
                reason: "must not be empty",
            });
        }
        if self.captured_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "negative_artifact_verification_result.captured_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ArtifactVerificationInvalidationTrigger {
    TrustRootRotation,
    TrustRootRevocation,
    ArtifactRevocation,
    PolicySnapshotChanged,
    TrustSetSnapshotChanged,
    ScopeChanged,
    FreshnessWindowExpired,
    NegativeResultRecorded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationCacheBasis {
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub artifact_trust_binding_ref: ArtifactTrustBindingRef,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub verification_basis_fingerprint: VerificationBasisFingerprint,
    pub artifact_scope_fingerprint: ArtifactScopeFingerprint,
    pub verified_at: MonotonicTimeNs,
    pub fresh_until: MonotonicTimeNs,
    pub cache_policy_class: ArtifactVerificationCachePolicyClass,
    pub invalidation_triggers: Vec<ArtifactVerificationInvalidationTrigger>,
    pub linked_negative_result_refs: Vec<NegativeVerificationResultRef>,
}

impl Validate for VerificationCacheBasis {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.artifact_identity_ref.validate()?;
        self.artifact_trust_binding_ref.validate()?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        self.verification_basis_fingerprint.validate()?;
        self.artifact_scope_fingerprint.validate()?;
        if self.verified_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "verification_cache_basis.verified_at",
                reason: "must be > 0",
            });
        }
        if self.fresh_until.0 <= self.verified_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "verification_cache_basis.fresh_until",
                reason: "must be > verified_at",
            });
        }
        let mut negative_refs = BTreeSet::new();
        for linked_negative_result_ref in &self.linked_negative_result_refs {
            linked_negative_result_ref.validate()?;
            if !negative_refs.insert(linked_negative_result_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "verification_cache_basis.linked_negative_result_refs",
                    reason: "contains duplicate ref",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustDecisionProvenance {
    pub verifier_owner: String,
    pub verifier_version: String,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub evidence_refs: Vec<String>,
    pub historical_snapshot_ref: Option<HistoricalTrustSnapshotRef>,
    pub replay_reconstructable: bool,
}

impl Validate for ArtifactTrustDecisionProvenance {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_trust_decision_provenance.verifier_owner",
            &self.verifier_owner,
            128,
        )?;
        validate_id(
            "artifact_trust_decision_provenance.verifier_version",
            &self.verifier_version,
            64,
        )?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        validate_string_list_unique(
            "artifact_trust_decision_provenance.evidence_refs",
            &self.evidence_refs,
            32,
            128,
        )?;
        if let Some(historical_snapshot_ref) = &self.historical_snapshot_ref {
            historical_snapshot_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustControlHints {
    pub blast_radius_scope: String,
    pub proof_required_for_completion: bool,
    pub rollback_readiness: bool,
    pub safe_mode_eligibility: bool,
    pub quarantine_eligibility: bool,
}

impl Validate for ArtifactTrustControlHints {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "artifact_trust_control_hints.blast_radius_scope",
            &self.blast_radius_scope,
            64,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustDecisionRecord {
    pub authority_decision_id: ArtifactTrustDecisionId,
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub artifact_trust_binding_ref: ArtifactTrustBindingRef,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub artifact_verification_result: ArtifactVerificationResult,
    pub verification_basis_fingerprint: VerificationBasisFingerprint,
    pub negative_verification_result_ref: Option<NegativeVerificationResultRef>,
    pub provenance: ArtifactTrustDecisionProvenance,
    pub control_hints: ArtifactTrustControlHints,
    pub proof_entry_ref: Option<ArtifactTrustProofEntryRef>,
}

impl Validate for ArtifactTrustDecisionRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.authority_decision_id.validate()?;
        self.artifact_identity_ref.validate()?;
        self.artifact_trust_binding_ref.validate()?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        self.artifact_verification_result.validate()?;
        self.verification_basis_fingerprint.validate()?;
        if let Some(negative_verification_result_ref) = &self.negative_verification_result_ref {
            negative_verification_result_ref.validate()?;
        }
        self.provenance.validate()?;
        self.control_hints.validate()?;
        if let Some(proof_entry_ref) = &self.proof_entry_ref {
            proof_entry_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustProofEntry {
    pub proof_entry_ref: ArtifactTrustProofEntryRef,
    pub proof_record_ref: ArtifactTrustProofRecordRef,
    pub authority_decision_id: ArtifactTrustDecisionId,
    pub artifact_identity_ref: ArtifactIdentityRef,
    pub artifact_trust_binding_ref: ArtifactTrustBindingRef,
    pub trust_policy_snapshot_ref: TrustPolicySnapshotRef,
    pub trust_set_snapshot_ref: TrustSetSnapshotRef,
    pub verification_basis_fingerprint: VerificationBasisFingerprint,
    pub negative_verification_result_ref: Option<NegativeVerificationResultRef>,
    pub historical_snapshot_ref: Option<HistoricalTrustSnapshotRef>,
}

impl Validate for ArtifactTrustProofEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.proof_entry_ref.validate()?;
        self.proof_record_ref.validate()?;
        self.authority_decision_id.validate()?;
        self.artifact_identity_ref.validate()?;
        self.artifact_trust_binding_ref.validate()?;
        self.trust_policy_snapshot_ref.validate()?;
        self.trust_set_snapshot_ref.validate()?;
        self.verification_basis_fingerprint.validate()?;
        if let Some(negative_verification_result_ref) = &self.negative_verification_result_ref {
            negative_verification_result_ref.validate()?;
        }
        if let Some(historical_snapshot_ref) = &self.historical_snapshot_ref {
            historical_snapshot_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustExecutionState {
    pub decision_records: Vec<ArtifactTrustDecisionRecord>,
    pub primary_artifact_identity_ref: Option<ArtifactIdentityRef>,
    pub proof_record_ref: Option<ArtifactTrustProofRecordRef>,
}

impl Validate for ArtifactTrustExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.decision_records.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_execution_state.decision_records",
                reason: "must not be empty",
            });
        }
        let mut decision_ids = BTreeSet::new();
        for decision_record in &self.decision_records {
            decision_record.validate()?;
            if !decision_ids.insert(decision_record.authority_decision_id.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_trust_execution_state.decision_records",
                    reason: "contains duplicate authority_decision_id",
                });
            }
        }
        if let Some(primary_artifact_identity_ref) = &self.primary_artifact_identity_ref {
            primary_artifact_identity_ref.validate()?;
        }
        if let Some(proof_record_ref) = &self.proof_record_ref {
            proof_record_ref.validate()?;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_scope() -> ArtifactScope {
        ArtifactScope {
            tenant_scope_id: Some("tenant.alpha".to_string()),
            environment_scope_id: "env.prod".to_string(),
            runtime_scope_id: "runtime.selene".to_string(),
            platform_scope_id: Some("ios".to_string()),
            rollout_scope_id: Some("rollout.a".to_string()),
            feature_flag_scope_ids: vec!["feature.trust".to_string()],
            identity_scope_id: Some("user.jd".to_string()),
            device_scope_id: Some("device.1".to_string()),
        }
    }

    fn sample_lineage() -> ArtifactLineage {
        ArtifactLineage {
            lineage_parent_artifact_ref: None,
            lineage_root_artifact_ref: ArtifactIdentityRef("artifact.root".to_string()),
            replacement_of_artifact_ref: None,
            rollback_target_artifact_ref: None,
        }
    }

    fn sample_digest() -> ArtifactDigest {
        ArtifactDigest::new(
            "sha256_artifact_digest".to_string(),
            ArtifactDigestAlgorithm::Sha256,
            ArtifactDigestEncoding::Hex,
            ArtifactDigestBoundary::WholePayload,
        )
        .unwrap()
    }

    fn sample_policy_snapshot() -> TrustPolicySnapshot {
        TrustPolicySnapshot {
            trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
            trust_policy_version: "policy_v1".to_string(),
            trust_set_version: "trust_set_v1".to_string(),
            crypto_suite_version: "ed25519-sha256-v1".to_string(),
            time_authority_basis: "monotonic_ns".to_string(),
            policy_matrix_hash: "policy_matrix_hash_v1".to_string(),
            generated_at: MonotonicTimeNs(10),
            legacy_compatibility_class: LegacyCompatibilityClass::StrictNoLegacy,
            verification_cache_policy_class: ArtifactVerificationCachePolicyClass::PositiveOnly,
        }
    }

    fn sample_trust_set_snapshot() -> TrustSetSnapshot {
        TrustSetSnapshot {
            trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
            trust_set_version: "trust_set_v1".to_string(),
            snapshot_epoch: 1,
            archived_snapshot_ref: None,
            cluster_consistency_posture: TrustSetClusterConsistencyPosture::Consistent,
            members: vec![TrustSetMember {
                member_ref: "member.root.1".to_string(),
                member_kind: TrustSetMemberKind::TrustRoot,
                trust_root_id: Some("root.selene".to_string()),
                artifact_signer_identity_ref: None,
            }],
        }
    }

    fn sample_verification_result() -> ArtifactVerificationResult {
        ArtifactVerificationResult {
            artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
            artifact_trust_binding_ref: ArtifactTrustBindingRef(
                "artifact.trust.binding.1".to_string(),
            ),
            trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
            trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
            verification_basis_fingerprint: VerificationBasisFingerprint(
                "fingerprint.1".to_string(),
            ),
            artifact_verification_outcome: ArtifactVerificationOutcome::VerifiedFresh,
            artifact_verification_failure_class: None,
            negative_verification_result_ref: None,
            verification_timestamp: MonotonicTimeNs(20),
            verification_cache_used: false,
            historical_snapshot_ref: None,
        }
    }

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

    #[test]
    fn artifact_identity_rejects_empty_ref() {
        let identity = ArtifactIdentity {
            artifact_identity_ref: ArtifactIdentityRef(String::new()),
            artifact_class: ArtifactClass::WakeArtifact,
            artifact_type_name: "WAKE_MODEL".to_string(),
            artifact_version: ArtifactVersion(1),
            artifact_schema_version: SchemaVersion(1),
            created_at: MonotonicTimeNs(1),
            payload_ref: Some("blob://wake".to_string()),
            scope: sample_scope(),
            lineage: sample_lineage(),
            lifecycle_state_ref: None,
            dependency_set_ref: None,
        };
        assert!(identity.validate().is_err());
    }

    #[test]
    fn trust_set_snapshot_requires_member() {
        let snapshot = TrustSetSnapshot {
            trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
            trust_set_version: "trust_set_v1".to_string(),
            snapshot_epoch: 1,
            archived_snapshot_ref: None,
            cluster_consistency_posture: TrustSetClusterConsistencyPosture::Consistent,
            members: Vec::new(),
        };

        assert!(snapshot.validate().is_err());
    }

    #[test]
    fn artifact_trust_decision_record_round_trips_minimal_valid() {
        let record = ArtifactTrustDecisionRecord {
            authority_decision_id: ArtifactTrustDecisionId("authority.decision.1".to_string()),
            artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
            artifact_trust_binding_ref: ArtifactTrustBindingRef(
                "artifact.trust.binding.1".to_string(),
            ),
            trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
            trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
            artifact_verification_result: sample_verification_result(),
            verification_basis_fingerprint: VerificationBasisFingerprint(
                "fingerprint.1".to_string(),
            ),
            negative_verification_result_ref: None,
            provenance: ArtifactTrustDecisionProvenance {
                verifier_owner: "SECTION_04_AUTHORITY".to_string(),
                verifier_version: "v1".to_string(),
                trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
                trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                evidence_refs: vec!["artifact.ref.1".to_string()],
                historical_snapshot_ref: None,
                replay_reconstructable: true,
            },
            control_hints: ArtifactTrustControlHints {
                blast_radius_scope: "artifact-local".to_string(),
                proof_required_for_completion: true,
                rollback_readiness: false,
                safe_mode_eligibility: false,
                quarantine_eligibility: true,
            },
            proof_entry_ref: None,
        };

        assert!(record.validate().is_ok());
    }

    #[test]
    fn artifact_trust_binding_uses_digest_not_hash_alias() {
        let binding = ArtifactTrustBinding {
            artifact_trust_binding_ref: ArtifactTrustBindingRef(
                "artifact.trust.binding.1".to_string(),
            ),
            artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
            digest: sample_digest(),
            bundle_manifest: None,
            artifact_signer_identity_ref: ArtifactSignerIdentityRef(
                "signer.identity.1".to_string(),
            ),
            trust_root_id: "root.selene".to_string(),
            trust_root_version: ArtifactTrustRootVersion(1),
            certification_state: ArtifactCertificationState {
                state: ArtifactCertificationStateValue::RuntimeCertified,
                certified_by_identity_ref: Some(ArtifactSignerIdentityRef(
                    "signer.identity.1".to_string(),
                )),
                certified_at: Some(MonotonicTimeNs(5)),
                certification_ref: Some("cert.ref.1".to_string()),
                certification_policy_version: Some("policy_v1".to_string()),
            },
            revocation_state: ArtifactRevocationState {
                state: ArtifactRevocationStateValue::NotRevoked,
                revocation_ref: None,
            },
            dependency_set_ref: None,
        };

        assert!(binding.validate().is_ok());
    }

    #[test]
    fn negative_verification_result_requires_reason_codes() {
        let negative = NegativeArtifactVerificationResult {
            negative_verification_result_ref: NegativeVerificationResultRef(
                "negative.result.1".to_string(),
            ),
            authority_decision_id: ArtifactTrustDecisionId("authority.decision.1".to_string()),
            artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
            trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
            trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
            verification_failure_class: ArtifactVerificationFailureClass::SignatureInvalid,
            reason_codes: Vec::new(),
            captured_at: MonotonicTimeNs(30),
            durable: true,
            cacheable: false,
        };

        assert!(negative.validate().is_err());
    }

    #[test]
    fn artifact_trust_execution_state_requires_unique_decision_ids() {
        let record = ArtifactTrustDecisionRecord {
            authority_decision_id: ArtifactTrustDecisionId("authority.decision.1".to_string()),
            artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
            artifact_trust_binding_ref: ArtifactTrustBindingRef(
                "artifact.trust.binding.1".to_string(),
            ),
            trust_policy_snapshot_ref: sample_policy_snapshot().trust_policy_snapshot_ref.clone(),
            trust_set_snapshot_ref: sample_trust_set_snapshot().trust_set_snapshot_ref.clone(),
            artifact_verification_result: sample_verification_result(),
            verification_basis_fingerprint: VerificationBasisFingerprint(
                "fingerprint.1".to_string(),
            ),
            negative_verification_result_ref: None,
            provenance: ArtifactTrustDecisionProvenance {
                verifier_owner: "SECTION_04_AUTHORITY".to_string(),
                verifier_version: "v1".to_string(),
                trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
                trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                evidence_refs: vec!["artifact.ref.1".to_string()],
                historical_snapshot_ref: None,
                replay_reconstructable: true,
            },
            control_hints: ArtifactTrustControlHints {
                blast_radius_scope: "artifact-local".to_string(),
                proof_required_for_completion: true,
                rollback_readiness: false,
                safe_mode_eligibility: false,
                quarantine_eligibility: true,
            },
            proof_entry_ref: None,
        };
        let execution_state = ArtifactTrustExecutionState {
            decision_records: vec![record.clone(), record],
            primary_artifact_identity_ref: Some(ArtifactIdentityRef(
                "artifact.identity.1".to_string(),
            )),
            proof_record_ref: None,
        };

        assert!(execution_state.validate().is_err());
    }
}
