#![forbid(unsafe_code)]

use std::cell::RefCell;

use selene_kernel_contracts::ph1j::{
    CanonicalProofRecord, CanonicalProofRecordInput, ProofFailureClass, ProofProtectedActionClass,
    ProofRetentionClass, ProofSignerIdentityMetadata, ProofVerificationPosture,
    ProofVerificationResult, ProofWriteReceipt, TimestampTrustPosture,
};
use selene_kernel_contracts::ph1simcat::{SimulationId, SimulationVersion};
use selene_kernel_contracts::runtime_execution::RuntimeExecutionEnvelope;
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
            if failure_class.trim().is_empty() || failure_class.len() > 64 || !failure_class.is_ascii()
            {
                return Err(ContractViolation::InvalidValue {
                    field: "protected_proof_write_request.failure_class",
                    reason: "must be ASCII and <= 64 chars when provided",
                });
            }
        }
        if self.received_at.0 == 0 || self.executed_at.0 == 0 || self.executed_at.0 < self.received_at.0 {
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
        request.validate().map_err(StorageError::ContractViolation)?;
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
        let input = CanonicalProofRecordInput::v1(
            request.runtime_execution_envelope.request_id.clone(),
            request.runtime_execution_envelope.trace_id.clone(),
            request.runtime_execution_envelope.session_id,
            Some(request.runtime_execution_envelope.turn_id),
            Some(request.runtime_execution_envelope.actor_identity.as_str().to_string()),
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
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::runtime_execution::{
        AdmissionState, PlatformRuntimeContext, RuntimeExecutionEnvelope,
    };
    use selene_storage::ph1f::{DeviceRecord, IdentityRecord, IdentityStatus, SessionRecord};
    use selene_kernel_contracts::SessionState;

    fn store_with_identity_device_session() -> Ph1fStore {
        let mut store = Ph1fStore::new_in_memory();
        let user_id = UserId::new("proof_user").unwrap();
        let device_id = DeviceId::new("proof_device").unwrap();
        store
            .insert_identity(
                IdentityRecord::v1(
                    user_id.clone(),
                    None,
                    None,
                    MonotonicTimeNs(1),
                    IdentityStatus::Active,
                ),
            )
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(device_id.clone(), user_id.clone(), "desktop".to_string(), MonotonicTimeNs(1), None)
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
}
