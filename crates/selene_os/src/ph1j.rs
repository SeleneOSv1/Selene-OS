#![forbid(unsafe_code)]

use std::cell::RefCell;

use selene_kernel_contracts::ph1art::{
    ArtifactTrustDecisionRecord, ArtifactTrustExecutionState,
};
use selene_kernel_contracts::ph1d::SafetyTier;
use selene_kernel_contracts::ph1j::{
    artifact_trust_proof_entry_ref_for_event_id_and_ordinal,
    artifact_trust_proof_record_ref_for_event_id, ArtifactTrustProofEntryInput,
    CanonicalProofRecord, CanonicalProofRecordInput, ProofChainStatus, ProofFailureClass,
    ProofProtectedActionClass, ProofRetentionClass, ProofSignerIdentityMetadata,
    ProofVerificationPosture, ProofVerificationResult, ProofWriteOutcome, ProofWriteReceipt,
    TimestampTrustPosture,
};
use selene_kernel_contracts::ph1simcat::{SimulationId, SimulationVersion};
use selene_kernel_contracts::runtime_execution::{ProofExecutionState, RuntimeExecutionEnvelope};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{Ph1fStore, StorageError};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    pub const PH1_J_PROOF_WRITE_FAILED: ReasonCodeId = ReasonCodeId(0x4A10_0001);
    pub const PH1_J_PROOF_CHAIN_INTEGRITY_FAILED: ReasonCodeId = ReasonCodeId(0x4A10_0002);
    pub const PH1_J_PROOF_SIGNATURE_FAILED: ReasonCodeId = ReasonCodeId(0x4A10_0003);
    pub const PH1_J_PROOF_CANONICALIZATION_FAILED: ReasonCodeId = ReasonCodeId(0x4A10_0004);
    pub const PH1_J_PROOF_STORAGE_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4A10_0005);
    pub const PH1_J_PROOF_VERIFICATION_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x4A10_0006);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtectedProofWriteRequest {
    pub runtime_execution_envelope: RuntimeExecutionEnvelope,
    pub action_class: ProofProtectedActionClass,
    pub authority_decision_reference: Option<String>,
    pub policy_rule_identifiers: Vec<String>,
    pub policy_version: Option<String>,
    pub simulation_id: Option<SimulationId>,
    pub simulation_version: Option<SimulationVersion>,
    pub simulation_certification_state: Option<String>,
    pub execution_outcome: String,
    pub failure_class: Option<String>,
    pub reason_codes: Vec<ReasonCodeId>,
    pub received_at: MonotonicTimeNs,
    pub executed_at: MonotonicTimeNs,
    pub proof_retention_class: ProofRetentionClass,
    pub verifier_metadata_ref: Option<String>,
}

impl ProtectedProofWriteRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        runtime_execution_envelope: RuntimeExecutionEnvelope,
        action_class: ProofProtectedActionClass,
        authority_decision_reference: Option<String>,
        policy_rule_identifiers: Vec<String>,
        policy_version: Option<String>,
        simulation_id: Option<SimulationId>,
        simulation_version: Option<SimulationVersion>,
        simulation_certification_state: Option<String>,
        execution_outcome: String,
        failure_class: Option<String>,
        reason_codes: Vec<ReasonCodeId>,
        received_at: MonotonicTimeNs,
        executed_at: MonotonicTimeNs,
        proof_retention_class: ProofRetentionClass,
        verifier_metadata_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            runtime_execution_envelope,
            action_class,
            authority_decision_reference,
            policy_rule_identifiers,
            policy_version,
            simulation_id,
            simulation_version,
            simulation_certification_state,
            execution_outcome,
            failure_class,
            reason_codes,
            received_at,
            executed_at,
            proof_retention_class,
            verifier_metadata_ref,
        };
        request.validate()?;
        Ok(request)
    }
}

impl Validate for ProtectedProofWriteRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.runtime_execution_envelope.validate()?;
        if self.execution_outcome.trim().is_empty()
            || self.execution_outcome.len() > 64
            || !self.execution_outcome.is_ascii()
        {
            return Err(ContractViolation::InvalidValue {
                field: "protected_proof_write_request.execution_outcome",
                reason: "must be ASCII and <= 64 chars",
            });
        }
        if let Some(failure_class) = self.failure_class.as_ref() {
            if failure_class.trim().is_empty()
                || failure_class.len() > 64
                || !failure_class.is_ascii()
            {
                return Err(ContractViolation::InvalidValue {
                    field: "protected_proof_write_request.failure_class",
                    reason: "must be ASCII and <= 64 chars when provided",
                });
            }
        }
        if self.received_at.0 == 0
            || self.executed_at.0 == 0
            || self.executed_at.0 < self.received_at.0
        {
            return Err(ContractViolation::InvalidValue {
                field: "protected_proof_write_request.executed_at",
                reason: "must be >= received_at",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1jRuntimeConfig {
    pub proof_capture_enabled: bool,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub signer_identity: String,
    pub signer_key_id: String,
    pub signature_algorithm: String,
}

impl Ph1jRuntimeConfig {
    pub fn mvp_v1() -> Self {
        Self {
            proof_capture_enabled: true,
            node_id: env_or_default("SELENE_RUNTIME_NODE_ID", "selene_os_node_v1"),
            runtime_instance_identity: env_or_default(
                "SELENE_RUNTIME_INSTANCE_ID",
                "selene_runtime_instance_v1",
            ),
            environment_identity: env_or_default("SELENE_ENVIRONMENT_IDENTITY", "local-dev"),
            build_version: env_or_default("SELENE_BUILD_VERSION", "dev-build"),
            git_commit: env_or_default("SELENE_GIT_COMMIT", "unknown_commit"),
            signer_identity: env_or_default("SELENE_PH1J_SIGNER_ID", "selene_ph1j_signer_v1"),
            signer_key_id: env_or_default("SELENE_PH1J_SIGNER_KEY_ID", "ph1j_key_v1"),
            signature_algorithm: "SHA256_KEYED_DIGEST".to_string(),
        }
    }

    pub fn with_proof_capture_enabled(mut self, proof_capture_enabled: bool) -> Self {
        self.proof_capture_enabled = proof_capture_enabled;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Ph1jRuntime {
    config: Ph1jRuntimeConfig,
    forced_failure_for_tests: RefCell<Option<ProofFailureClass>>,
}

impl Default for Ph1jRuntime {
    fn default() -> Self {
        Self::new(Ph1jRuntimeConfig::mvp_v1())
    }
}

impl Ph1jRuntime {
    pub fn new(config: Ph1jRuntimeConfig) -> Self {
        Self {
            config,
            forced_failure_for_tests: RefCell::new(None),
        }
    }

    pub fn config(&self) -> &Ph1jRuntimeConfig {
        &self.config
    }

    pub fn force_failure_for_tests(&self, failure_class: Option<ProofFailureClass>) {
        *self.forced_failure_for_tests.borrow_mut() = failure_class;
    }

    pub fn emit_protected_proof(
        &self,
        store: &mut Ph1fStore,
        request: ProtectedProofWriteRequest,
    ) -> Result<ProofWriteReceipt, StorageError> {
        request
            .validate()
            .map_err(StorageError::ContractViolation)?;
        if !self.config.proof_capture_enabled {
            return Err(StorageError::ProofFailure {
                class: ProofFailureClass::ProofWriteFailure,
                detail: "proof capture is disabled for a protected action".to_string(),
            });
        }
        if let Some(class) = self.forced_failure_for_tests.borrow_mut().take() {
            return Err(StorageError::ProofFailure {
                class,
                detail: "forced PH1.J proof failure for test path".to_string(),
            });
        }

        let signer_identity_metadata = ProofSignerIdentityMetadata::v1(
            self.config.signer_identity.clone(),
            self.config.signer_key_id.clone(),
            self.config.signature_algorithm.clone(),
        )
        .map_err(StorageError::ContractViolation)?;
        let artifact_trust_entries = artifact_trust_proof_entry_inputs(
            &request.runtime_execution_envelope,
        )
        .map_err(StorageError::ContractViolation)?;
        let input = CanonicalProofRecordInput::v1_with_artifact_trust_entries(
            request.runtime_execution_envelope.request_id.clone(),
            request.runtime_execution_envelope.trace_id.clone(),
            request.runtime_execution_envelope.session_id,
            Some(request.runtime_execution_envelope.turn_id),
            Some(
                request
                    .runtime_execution_envelope
                    .actor_identity
                    .as_str()
                    .to_string(),
            ),
            Some(request.runtime_execution_envelope.device_identity.clone()),
            self.config.node_id.clone(),
            self.config.runtime_instance_identity.clone(),
            self.config.environment_identity.clone(),
            self.config.build_version.clone(),
            self.config.git_commit.clone(),
            request.action_class,
            request.authority_decision_reference.clone(),
            request.policy_rule_identifiers.clone(),
            request.policy_version.clone(),
            request.simulation_id.clone(),
            request.simulation_version,
            request.simulation_certification_state.clone(),
            request.execution_outcome.clone(),
            request.failure_class.clone(),
            request.reason_codes.clone(),
            request.received_at,
            request.executed_at,
            signer_identity_metadata,
            request.proof_retention_class,
            ProofVerificationPosture::VerificationReady,
            TimestampTrustPosture::RuntimeMonotonic,
            request.verifier_metadata_ref.clone(),
            artifact_trust_entries,
        )
        .map_err(StorageError::ContractViolation)?;
        selene_storage::ph1j::Ph1jRuntime::emit_proof(
            store,
            input,
            Some(request.runtime_execution_envelope.idempotency_key.clone()),
        )
    }

    pub fn reconstruct_by_request_id(
        &self,
        store: &Ph1fStore,
        request_id: &str,
        limit: usize,
    ) -> Result<Vec<CanonicalProofRecord>, StorageError> {
        selene_storage::ph1j::Ph1jRuntime::replay_by_request_id(store, request_id, limit)
    }

    pub fn reconstruct_by_session_turn(
        &self,
        store: &Ph1fStore,
        session_id: selene_kernel_contracts::ph1l::SessionId,
        turn_id: selene_kernel_contracts::ph1j::TurnId,
        limit: usize,
    ) -> Result<Vec<CanonicalProofRecord>, StorageError> {
        selene_storage::ph1j::Ph1jRuntime::replay_by_session_turn(store, session_id, turn_id, limit)
    }

    pub fn verify_by_request_id(
        &self,
        store: &Ph1fStore,
        request_id: &str,
        limit: usize,
    ) -> Result<Vec<ProofVerificationResult>, StorageError> {
        selene_storage::ph1j::Ph1jRuntime::verify_by_request_id(store, request_id, limit)
    }
}

pub fn proof_failure_class_for_storage_error(error: &StorageError) -> ProofFailureClass {
    match error {
        StorageError::ProofFailure { class, .. } => *class,
        StorageError::ContractViolation(_) => ProofFailureClass::ProofCanonicalizationFailure,
        StorageError::AppendOnlyViolation { .. } => ProofFailureClass::ProofChainIntegrityFailure,
        StorageError::ForeignKeyViolation { .. } | StorageError::DuplicateKey { .. } => {
            ProofFailureClass::ProofWriteFailure
        }
    }
}

pub fn proof_execution_state_from_error(
    error: &StorageError,
) -> Result<ProofExecutionState, StorageError> {
    let failure_class = proof_failure_class_for_storage_error(error);
    ProofExecutionState::v1(
        None,
        ProofWriteOutcome::Failed,
        Some(failure_class),
        match failure_class {
            ProofFailureClass::ProofChainIntegrityFailure
            | ProofFailureClass::ProofSignatureFailure => ProofChainStatus::ChainBreakDetected,
            _ => ProofChainStatus::NotChecked,
        },
        match failure_class {
            ProofFailureClass::ProofVerificationUnavailable => {
                ProofVerificationPosture::VerificationUnavailable
            }
            _ => ProofVerificationPosture::NotRequested,
        },
        TimestampTrustPosture::RuntimeMonotonic,
        None,
    )
    .map_err(StorageError::ContractViolation)
}

pub fn proof_execution_state_from_receipt(
    receipt: ProofWriteReceipt,
) -> Result<ProofExecutionState, StorageError> {
    ProofExecutionState::v1(
        Some(receipt.proof_record_ref),
        receipt.proof_write_outcome,
        None,
        receipt.proof_chain_status,
        receipt.proof_verification_posture,
        receipt.timestamp_trust_posture,
        receipt.verifier_metadata_ref,
    )
    .map_err(StorageError::ContractViolation)
}

fn artifact_trust_proof_entry_inputs(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Result<Vec<ArtifactTrustProofEntryInput>, ContractViolation> {
    runtime_execution_envelope
        .artifact_trust_state
        .as_ref()
        .map(|state| {
            state
                .decision_records
                .iter()
                .map(artifact_trust_proof_entry_input_from_decision_record)
                .collect()
        })
        .unwrap_or_else(|| Ok(Vec::new()))
}

fn artifact_trust_proof_entry_input_from_decision_record(
    decision_record: &ArtifactTrustDecisionRecord,
) -> Result<ArtifactTrustProofEntryInput, ContractViolation> {
    let entry = ArtifactTrustProofEntryInput {
        authority_decision_id: decision_record.authority_decision_id.clone(),
        artifact_identity_ref: decision_record.artifact_identity_ref.clone(),
        artifact_trust_binding_ref: decision_record.artifact_trust_binding_ref.clone(),
        trust_policy_snapshot_ref: decision_record.trust_policy_snapshot_ref.clone(),
        trust_set_snapshot_ref: decision_record.trust_set_snapshot_ref.clone(),
        verification_basis_fingerprint: decision_record.verification_basis_fingerprint.clone(),
        artifact_verification_outcome: decision_record
            .artifact_verification_result
            .artifact_verification_outcome,
        artifact_verification_failure_class: decision_record
            .artifact_verification_result
            .artifact_verification_failure_class,
        negative_verification_result_ref: decision_record.negative_verification_result_ref.clone(),
        historical_snapshot_ref: decision_record.provenance.historical_snapshot_ref.clone(),
        provenance_verifier_owner: decision_record.provenance.verifier_owner.clone(),
        provenance_verifier_version: decision_record.provenance.verifier_version.clone(),
        provenance_evidence_refs: decision_record.provenance.evidence_refs.clone(),
    };
    entry.validate()?;
    Ok(entry)
}

pub fn artifact_trust_state_from_receipt(
    artifact_trust_state: Option<&ArtifactTrustExecutionState>,
    receipt: &ProofWriteReceipt,
) -> Result<Option<ArtifactTrustExecutionState>, StorageError> {
    let Some(state) = artifact_trust_state else {
        return Ok(None);
    };
    let proof_record_ref = artifact_trust_proof_record_ref_for_event_id(receipt.proof_event_id)
        .map_err(StorageError::ContractViolation)?;
    if receipt.proof_record_ref != proof_record_ref.0 {
        return Err(StorageError::ProofFailure {
            class: ProofFailureClass::ProofCanonicalizationFailure,
            detail: "proof receipt record ref does not match canonical trust proof record ref"
                .to_string(),
        });
    }
    let mut next = state.clone();
    next.proof_record_ref = Some(proof_record_ref);
    for (index, decision_record) in next.decision_records.iter_mut().enumerate() {
        decision_record.proof_entry_ref = Some(
            artifact_trust_proof_entry_ref_for_event_id_and_ordinal(receipt.proof_event_id, index)
                .map_err(StorageError::ContractViolation)?,
        );
    }
    next.validate().map_err(StorageError::ContractViolation)?;
    Ok(Some(next))
}

pub fn proof_policy_rule_identifiers(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Vec<String> {
    runtime_execution_envelope
        .governance_state
        .as_ref()
        .and_then(|state| state.last_rule_id.clone())
        .into_iter()
        .collect()
}

pub fn proof_policy_version(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Option<String> {
    runtime_execution_envelope
        .governance_state
        .as_ref()
        .map(|state| state.governance_policy_version.clone())
}

pub fn proof_authority_decision_reference(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Option<String> {
    runtime_execution_envelope.authority_state.as_ref().map(|state| {
        let policy_context_ref = state
            .policy_context_ref
            .map(|value| {
                format!(
                    "privacy_mode={};do_not_disturb={};safety_tier={}",
                    value.privacy_mode,
                    value.do_not_disturb,
                    match value.safety_tier {
                        SafetyTier::Standard => "STANDARD",
                        SafetyTier::Strict => "STRICT",
                    }
                )
            })
            .unwrap_or_else(|| "-".to_string());
        format!(
            "policy_decision={};policy_context={};identity_scope_required={};identity_scope_satisfied={};memory_scope_allowed={};reason_code={}",
            state.policy_decision.as_str(),
            policy_context_ref,
            state.identity_scope_required,
            state.identity_scope_satisfied,
            state.memory_scope_allowed,
            state.reason_code.unwrap_or_default(),
        )
    })
}

pub fn proof_verifier_metadata_ref(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> String {
    if let Some(session_id) = runtime_execution_envelope.session_id {
        format!(
            "session:{}:turn:{}:request:{}",
            session_id.0,
            runtime_execution_envelope.turn_id.0,
            runtime_execution_envelope.request_id
        )
    } else {
        format!("request:{}", runtime_execution_envelope.request_id)
    }
}

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| default.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1art::{
        ArtifactIdentityRef, ArtifactTrustBindingRef, ArtifactTrustControlHints,
        ArtifactTrustDecisionId, ArtifactTrustDecisionProvenance,
        ArtifactVerificationFailureClass, ArtifactVerificationOutcome,
        ArtifactVerificationResult, NegativeVerificationResultRef,
        TrustPolicySnapshotRef, TrustSetSnapshotRef, VerificationBasisFingerprint,
    };
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::runtime_execution::{
        AdmissionState, PlatformRuntimeContext, RuntimeExecutionEnvelope,
    };
    use selene_kernel_contracts::SessionState;
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, SessionRecord};

    fn store_with_identity_device_session() -> Ph1fStore {
        let mut store = Ph1fStore::new_in_memory();
        let user_id = UserId::new("proof_user").unwrap();
        let device_id = DeviceId::new("proof_device").unwrap();
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "desktop".to_string(),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        store
            .insert_session(
                SessionRecord::v1(
                    SessionId(1),
                    user_id,
                    device_id,
                    SessionState::Active,
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
        store
    }

    fn sample_envelope() -> RuntimeExecutionEnvelope {
        RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state(
            "req_1".to_string(),
            "trace_1".to_string(),
            "idem_1".to_string(),
            UserId::new("proof_user").unwrap(),
            DeviceId::new("proof_device").unwrap(),
            AppPlatform::Desktop,
            PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop).unwrap(),
            Some(SessionId(1)),
            selene_kernel_contracts::ph1j::TurnId(1),
            Some(1),
            AdmissionState::ExecutionAdmitted,
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn sample_artifact_trust_state() -> ArtifactTrustExecutionState {
        ArtifactTrustExecutionState {
            decision_records: vec![
                ArtifactTrustDecisionRecord {
                    authority_decision_id: ArtifactTrustDecisionId(
                        "authority.decision.1".to_string(),
                    ),
                    artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
                    artifact_trust_binding_ref: ArtifactTrustBindingRef(
                        "artifact.trust.binding.1".to_string(),
                    ),
                    trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                        "policy.snap.1".to_string(),
                    ),
                    trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                    artifact_verification_result: ArtifactVerificationResult {
                        artifact_identity_ref: ArtifactIdentityRef(
                            "artifact.identity.1".to_string(),
                        ),
                        artifact_trust_binding_ref: ArtifactTrustBindingRef(
                            "artifact.trust.binding.1".to_string(),
                        ),
                        trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                            "policy.snap.1".to_string(),
                        ),
                        trust_set_snapshot_ref: TrustSetSnapshotRef(
                            "trust.set.snap.1".to_string(),
                        ),
                        verification_basis_fingerprint: VerificationBasisFingerprint(
                            "basis.fp.1".to_string(),
                        ),
                        artifact_verification_outcome: ArtifactVerificationOutcome::VerifiedFresh,
                        artifact_verification_failure_class: None,
                        negative_verification_result_ref: None,
                        verification_timestamp: MonotonicTimeNs(20),
                        verification_cache_used: false,
                        historical_snapshot_ref: None,
                    },
                    verification_basis_fingerprint: VerificationBasisFingerprint(
                        "basis.fp.1".to_string(),
                    ),
                    negative_verification_result_ref: None,
                    provenance: ArtifactTrustDecisionProvenance {
                        verifier_owner: "section04.authority".to_string(),
                        verifier_version: "v1".to_string(),
                        trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                            "policy.snap.1".to_string(),
                        ),
                        trust_set_snapshot_ref: TrustSetSnapshotRef(
                            "trust.set.snap.1".to_string(),
                        ),
                        evidence_refs: vec!["evidence.1".to_string()],
                        historical_snapshot_ref: None,
                        replay_reconstructable: true,
                    },
                    control_hints: ArtifactTrustControlHints {
                        blast_radius_scope: "ARTIFACT_LOCAL".to_string(),
                        proof_required_for_completion: true,
                        rollback_readiness: true,
                        safe_mode_eligibility: false,
                        quarantine_eligibility: true,
                    },
                    proof_entry_ref: None,
                },
                ArtifactTrustDecisionRecord {
                    authority_decision_id: ArtifactTrustDecisionId(
                        "authority.decision.2".to_string(),
                    ),
                    artifact_identity_ref: ArtifactIdentityRef("artifact.identity.2".to_string()),
                    artifact_trust_binding_ref: ArtifactTrustBindingRef(
                        "artifact.trust.binding.2".to_string(),
                    ),
                    trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                        "policy.snap.1".to_string(),
                    ),
                    trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                    artifact_verification_result: ArtifactVerificationResult {
                        artifact_identity_ref: ArtifactIdentityRef(
                            "artifact.identity.2".to_string(),
                        ),
                        artifact_trust_binding_ref: ArtifactTrustBindingRef(
                            "artifact.trust.binding.2".to_string(),
                        ),
                        trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                            "policy.snap.1".to_string(),
                        ),
                        trust_set_snapshot_ref: TrustSetSnapshotRef(
                            "trust.set.snap.1".to_string(),
                        ),
                        verification_basis_fingerprint: VerificationBasisFingerprint(
                            "basis.fp.2".to_string(),
                        ),
                        artifact_verification_outcome: ArtifactVerificationOutcome::Failed,
                        artifact_verification_failure_class: Some(
                            ArtifactVerificationFailureClass::SignatureInvalid,
                        ),
                        negative_verification_result_ref: Some(NegativeVerificationResultRef(
                            "neg.verify.2".to_string(),
                        )),
                        verification_timestamp: MonotonicTimeNs(21),
                        verification_cache_used: false,
                        historical_snapshot_ref: None,
                    },
                    verification_basis_fingerprint: VerificationBasisFingerprint(
                        "basis.fp.2".to_string(),
                    ),
                    negative_verification_result_ref: Some(NegativeVerificationResultRef(
                        "neg.verify.2".to_string(),
                    )),
                    provenance: ArtifactTrustDecisionProvenance {
                        verifier_owner: "section04.authority".to_string(),
                        verifier_version: "v1".to_string(),
                        trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                            "policy.snap.1".to_string(),
                        ),
                        trust_set_snapshot_ref: TrustSetSnapshotRef(
                            "trust.set.snap.1".to_string(),
                        ),
                        evidence_refs: vec!["evidence.2".to_string()],
                        historical_snapshot_ref: None,
                        replay_reconstructable: true,
                    },
                    control_hints: ArtifactTrustControlHints {
                        blast_radius_scope: "TENANT".to_string(),
                        proof_required_for_completion: true,
                        rollback_readiness: false,
                        safe_mode_eligibility: true,
                        quarantine_eligibility: true,
                    },
                    proof_entry_ref: None,
                },
            ],
            primary_artifact_identity_ref: Some(ArtifactIdentityRef(
                "artifact.identity.1".to_string(),
            )),
            proof_record_ref: None,
        }
    }

    #[test]
    fn at_j_runtime_01_protected_action_writes_canonical_proof_record() {
        let runtime = Ph1jRuntime::default();
        let mut store = store_with_identity_device_session();
        let receipt = runtime
            .emit_protected_proof(
                &mut store,
                ProtectedProofWriteRequest::v1(
                    sample_envelope(),
                    ProofProtectedActionClass::VoiceTurnExecution,
                    Some("authority:allowed".to_string()),
                    vec!["RG-PROOF-001".to_string()],
                    Some("2026.03.08.v1".to_string()),
                    None,
                    None,
                    Some("CERTIFIED_ACTIVE".to_string()),
                    "DISPATCH".to_string(),
                    None,
                    vec![ReasonCodeId(1)],
                    MonotonicTimeNs(10),
                    MonotonicTimeNs(11),
                    ProofRetentionClass::ComplianceRetention,
                    Some("request:req_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        assert_eq!(receipt.proof_write_outcome.as_str(), "WRITTEN");
        assert_eq!(store.proof_records().len(), 1);
    }

    #[test]
    fn at_j_runtime_02_forced_failure_is_structured() {
        let runtime = Ph1jRuntime::default();
        runtime.force_failure_for_tests(Some(ProofFailureClass::ProofStorageUnavailable));
        let mut store = store_with_identity_device_session();
        let err = runtime
            .emit_protected_proof(
                &mut store,
                ProtectedProofWriteRequest::v1(
                    sample_envelope(),
                    ProofProtectedActionClass::PrimaryDeviceConfirmation,
                    Some("authority:allowed".to_string()),
                    vec!["RG-PROOF-001".to_string()],
                    Some("2026.03.08.v1".to_string()),
                    None,
                    None,
                    Some("CERTIFIED_ACTIVE".to_string()),
                    "ALLOW".to_string(),
                    None,
                    vec![ReasonCodeId(2)],
                    MonotonicTimeNs(10),
                    MonotonicTimeNs(11),
                    ProofRetentionClass::ComplianceRetention,
                    Some("request:req_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap_err();
        assert!(matches!(
            err,
            StorageError::ProofFailure {
                class: ProofFailureClass::ProofStorageUnavailable,
                ..
            }
        ));
    }

    #[test]
    fn at_j_runtime_03_artifact_trust_entries_follow_decision_order() {
        let runtime = Ph1jRuntime::default();
        let mut store = store_with_identity_device_session();
        let envelope = sample_envelope()
            .with_artifact_trust_state(Some(sample_artifact_trust_state()))
            .expect("artifact trust state transport must be valid");
        let receipt = runtime
            .emit_protected_proof(
                &mut store,
                ProtectedProofWriteRequest::v1(
                    envelope,
                    ProofProtectedActionClass::VoiceTurnExecution,
                    Some("authority:allowed".to_string()),
                    vec!["RG-PROOF-001".to_string()],
                    Some("2026.03.08.v1".to_string()),
                    None,
                    None,
                    Some("CERTIFIED_ACTIVE".to_string()),
                    "DISPATCH".to_string(),
                    None,
                    vec![ReasonCodeId(3)],
                    MonotonicTimeNs(10),
                    MonotonicTimeNs(11),
                    ProofRetentionClass::ComplianceRetention,
                    Some("request:req_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        let record = store.proof_records().first().expect("proof record must exist");
        assert_eq!(record.artifact_trust_entries.len(), 2);
        assert_eq!(
            record.artifact_trust_entries[0].linkage.proof_record_ref.0,
            receipt.proof_record_ref
        );
        assert_eq!(
            record.artifact_trust_entries[1].linkage.proof_record_ref.0,
            receipt.proof_record_ref
        );
        assert_eq!(
            record.artifact_trust_entries[0]
                .linkage
                .authority_decision_id
                .0,
            "authority.decision.1"
        );
        assert_eq!(
            record.artifact_trust_entries[1]
                .linkage
                .authority_decision_id
                .0,
            "authority.decision.2"
        );
        assert_eq!(
            record.artifact_trust_entries[1].artifact_verification_outcome,
            ArtifactVerificationOutcome::Failed
        );
        assert_eq!(
            record.artifact_trust_entries[1]
                .linkage
                .negative_verification_result_ref
                .as_ref()
                .map(|value| value.0.as_str()),
            Some("neg.verify.2")
        );
    }

    #[test]
    fn at_j_runtime_04_receipt_updates_artifact_trust_state_with_proof_linkage() {
        let runtime = Ph1jRuntime::default();
        let mut store = store_with_identity_device_session();
        let state = sample_artifact_trust_state();
        let envelope = sample_envelope()
            .with_artifact_trust_state(Some(state.clone()))
            .expect("artifact trust state transport must be valid");
        let receipt = runtime
            .emit_protected_proof(
                &mut store,
                ProtectedProofWriteRequest::v1(
                    envelope,
                    ProofProtectedActionClass::VoiceTurnExecution,
                    Some("authority:allowed".to_string()),
                    vec!["RG-PROOF-001".to_string()],
                    Some("2026.03.08.v1".to_string()),
                    None,
                    None,
                    Some("CERTIFIED_ACTIVE".to_string()),
                    "DISPATCH".to_string(),
                    None,
                    vec![ReasonCodeId(4)],
                    MonotonicTimeNs(10),
                    MonotonicTimeNs(11),
                    ProofRetentionClass::ComplianceRetention,
                    Some("request:req_1".to_string()),
                )
                .unwrap(),
            )
            .unwrap();
        let linked_state = artifact_trust_state_from_receipt(Some(&state), &receipt)
            .expect("proof linkage update must succeed")
            .expect("linked state must exist");
        assert!(linked_state.proof_record_ref.is_some());
        assert_eq!(linked_state.decision_records.len(), 2);
        assert!(
            linked_state.decision_records[0].proof_entry_ref.is_some(),
            "first decision must receive proof entry linkage"
        );
        assert!(
            linked_state.decision_records[1].proof_entry_ref.is_some(),
            "second decision must receive proof entry linkage"
        );
        assert_ne!(
            linked_state.decision_records[0].proof_entry_ref,
            linked_state.decision_records[1].proof_entry_ref
        );
    }
}
