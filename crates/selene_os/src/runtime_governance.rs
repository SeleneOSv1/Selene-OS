#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use selene_engines::ph1_voice_id::reason_codes as voice_id_reason_codes;
use selene_kernel_contracts::ph1_voice_id::{
    IdentityTierV2, Ph1VoiceIdResponse, SpoofLivenessStatus,
};
use selene_kernel_contracts::ph1art::{
    ArtifactTrustExecutionState, ArtifactVerificationFailureClass, ArtifactVerificationOutcome,
};
use selene_kernel_contracts::ph1j::ProofFailureClass;
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, FailureClass, IdentityExecutionState, IdentityExecutionStateInput,
    IdentityRecoveryState, IdentityTrustTier, IdentityVerificationConsistencyLevel,
    PersistenceAcknowledgementState, PersistenceConflictSeverity, PersistenceRecoveryMode,
    ProofExecutionState, RuntimeExecutionEnvelope,
};
use selene_kernel_contracts::runtime_governance::{
    GovernanceCertificationStatus, GovernanceClusterConsistency, GovernanceDecisionLogEntry,
    GovernanceDecisionOutcome, GovernanceDriftSignal, GovernanceExecutionState,
    GovernancePolicyWindow, GovernanceProtectedActionClass, GovernanceResponseClass,
    GovernanceRuleCategory, GovernanceRuleDescriptor, GovernanceSeverity,
    GovernanceSubsystemCertification,
};
use selene_kernel_contracts::{ContractViolation, SessionState, Validate};

pub mod reason_codes {
    pub const GOV_ENVELOPE_SESSION_REQUIRED: &str = "GOV_ENVELOPE_SESSION_REQUIRED";
    pub const GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED: &str = "GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED";
    pub const GOV_ENVELOPE_ADMISSION_REQUIRED: &str = "GOV_ENVELOPE_ADMISSION_REQUIRED";
    pub const GOV_PERSISTENCE_DEGRADED: &str = "GOV_PERSISTENCE_DEGRADED";
    pub const GOV_PERSISTENCE_STALE_REJECTED: &str = "GOV_PERSISTENCE_STALE_REJECTED";
    pub const GOV_PERSISTENCE_QUARANTINE_REQUIRED: &str = "GOV_PERSISTENCE_QUARANTINE_REQUIRED";
    pub const GOV_PROOF_REQUIRED: &str = "GOV_PROOF_REQUIRED";
    pub const GOV_GOVERNANCE_INTEGRITY_UNCERTAIN: &str = "GOV_GOVERNANCE_INTEGRITY_UNCERTAIN";
    pub const GOV_POLICY_VERSION_DRIFT: &str = "GOV_POLICY_VERSION_DRIFT";
    pub const GOV_SUBSYSTEM_CERTIFICATION_REGRESSED: &str = "GOV_SUBSYSTEM_CERTIFICATION_REGRESSED";
    pub const GOV_SAFE_MODE_ACTIVE: &str = "GOV_SAFE_MODE_ACTIVE";
    pub const GOV_ARTIFACT_TRUST_REQUIRED: &str = "GOV_ARTIFACT_TRUST_REQUIRED";
    pub const GOV_ARTIFACT_TRUST_EVIDENCE_INCOMPLETE: &str =
        "GOV_ARTIFACT_TRUST_EVIDENCE_INCOMPLETE";
    pub const GOV_ARTIFACT_TRUST_FAILED: &str = "GOV_ARTIFACT_TRUST_FAILED";
    pub const GOV_ARTIFACT_TRUST_DEGRADED: &str = "GOV_ARTIFACT_TRUST_DEGRADED";
}

const SUBSYSTEM_RUNTIME_GOVERNANCE: &str = "RUNTIME_GOVERNANCE";
const SUBSYSTEM_INGRESS_PIPELINE: &str = "INGRESS_PIPELINE";
const SUBSYSTEM_SESSION_ENGINE: &str = "SESSION_ENGINE";
const SUBSYSTEM_PERSISTENCE_SYNC: &str = "PERSISTENCE_SYNC";
const SUBSYSTEM_PROOF_CAPTURE: &str = "PROOF_CAPTURE";
const SUBSYSTEM_CLUSTER_GOVERNANCE: &str = "CLUSTER_GOVERNANCE";
const SUBSYSTEM_ARTIFACT_AUTHORITY: &str = "ARTIFACT_AUTHORITY";
const SUBSYSTEM_IDENTITY_VOICE_ENGINE: &str = "IDENTITY_VOICE_ENGINE";

const RULE_ENV_SESSION_REQUIRED: &str = "RG-SESSION-001";
const RULE_ENV_DEVICE_SEQUENCE_REQUIRED: &str = "RG-ENV-001";
const RULE_ENV_ADMISSION_REQUIRED: &str = "RG-ENV-002";
const RULE_PERSISTENCE_DEGRADED: &str = "RG-PERSIST-001";
const RULE_PERSISTENCE_STALE_REJECTED: &str = "RG-PERSIST-002";
const RULE_PERSISTENCE_QUARANTINE: &str = "RG-PERSIST-003";
const RULE_PROOF_REQUIRED: &str = "RG-PROOF-001";
const RULE_POLICY_VERSION_DRIFT: &str = "RG-CLUSTER-001";
const RULE_SUBSYSTEM_CERT_REGRESSED: &str = "RG-CERT-001";
const RULE_GOVERNANCE_INTEGRITY: &str = "RG-GOV-001";
const RULE_ARTIFACT_TRUST_REQUIRED: &str = "RG-ART-001";
const RULE_ARTIFACT_TRUST_EVIDENCE: &str = "RG-ART-002";
const RULE_ARTIFACT_TRUST_FAILED: &str = "RG-ART-003";
const RULE_ARTIFACT_TRUST_DEGRADED: &str = "RG-ART-004";

#[derive(Debug, Clone)]
pub struct RuntimeGovernanceConfig {
    pub policy_window: GovernancePolicyWindow,
    pub runtime_node_id: String,
    pub repeated_violation_threshold: u32,
    pub force_integrity_failure: bool,
}

impl RuntimeGovernanceConfig {
    pub fn mvp_v1() -> Self {
        Self {
            policy_window: GovernancePolicyWindow::v1(
                "2026.03.08.v1".to_string(),
                "2026.03.08.v1".to_string(),
                "2026.03.08.v1".to_string(),
            )
            .expect("governance policy window must validate"),
            runtime_node_id: default_runtime_node_id(),
            repeated_violation_threshold: 3,
            force_integrity_failure: false,
        }
    }

    pub fn with_force_integrity_failure(mut self, force_integrity_failure: bool) -> Self {
        self.force_integrity_failure = force_integrity_failure;
        self
    }

    pub fn with_policy_window(mut self, policy_window: GovernancePolicyWindow) -> Self {
        self.policy_window = policy_window;
        self
    }

    pub fn with_runtime_node_id(mut self, runtime_node_id: String) -> Self {
        self.runtime_node_id = runtime_node_id;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeGovernanceDecision {
    pub rule_id: String,
    pub subsystem_id: String,
    pub outcome: GovernanceDecisionOutcome,
    pub severity: GovernanceSeverity,
    pub response_class: GovernanceResponseClass,
    pub reason_code: String,
    pub session_id: Option<u128>,
    pub turn_id: Option<u64>,
    pub governance_state: GovernanceExecutionState,
}

#[derive(Debug, Clone)]
pub struct RuntimeGovernanceSnapshot {
    pub policy_window: GovernancePolicyWindow,
    pub rule_registry: Vec<GovernanceRuleDescriptor>,
    pub decision_log: Vec<GovernanceDecisionLogEntry>,
    pub safe_mode_active: bool,
    pub quarantined_subsystems: Vec<String>,
    pub subsystem_certifications: Vec<GovernanceSubsystemCertification>,
    pub cluster_consistency: GovernanceClusterConsistency,
    pub drift_signals: Vec<GovernanceDriftSignal>,
}

#[derive(Debug, Clone, Default)]
struct ArtifactTrustGovernanceLinkage {
    decision_ids: Vec<String>,
    proof_entry_refs: Vec<String>,
    proof_record_ref: Option<String>,
    policy_snapshot_refs: Vec<String>,
    trust_set_snapshot_refs: Vec<String>,
    basis_fingerprints: Vec<String>,
    negative_result_refs: Vec<String>,
}

#[derive(Debug)]
struct RuntimeGovernanceStateStore {
    decision_log: Vec<GovernanceDecisionLogEntry>,
    next_sequence: u64,
    safe_mode_active: bool,
    quarantined_subsystems: BTreeSet<String>,
    subsystem_certifications: BTreeMap<String, GovernanceCertificationStatus>,
    observed_node_policy_versions: BTreeMap<String, String>,
    violation_counts: BTreeMap<String, u32>,
    drift_signals: BTreeSet<GovernanceDriftSignal>,
    cluster_consistency: GovernanceClusterConsistency,
}

impl RuntimeGovernanceStateStore {
    fn new() -> Self {
        let subsystem_certifications = [
            (
                SUBSYSTEM_RUNTIME_GOVERNANCE.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_INGRESS_PIPELINE.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_SESSION_ENGINE.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_PERSISTENCE_SYNC.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_PROOF_CAPTURE.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_CLUSTER_GOVERNANCE.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
            (
                SUBSYSTEM_ARTIFACT_AUTHORITY.to_string(),
                GovernanceCertificationStatus::Certified,
            ),
        ]
        .into_iter()
        .collect();
        Self {
            decision_log: Vec::new(),
            next_sequence: 1,
            safe_mode_active: false,
            quarantined_subsystems: BTreeSet::new(),
            subsystem_certifications,
            observed_node_policy_versions: BTreeMap::new(),
            violation_counts: BTreeMap::new(),
            drift_signals: BTreeSet::new(),
            cluster_consistency: GovernanceClusterConsistency::Consistent,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct GovernanceDecisionBuildSpec<'a> {
    rule_id: &'a str,
    subsystem_id: &'a str,
    outcome: GovernanceDecisionOutcome,
    severity: GovernanceSeverity,
    response_class: GovernanceResponseClass,
    reason_code: &'a str,
    session_id: Option<u128>,
    turn_id: Option<u64>,
}

#[derive(Debug)]
struct GovernanceViolationSpec<'a> {
    decision: GovernanceDecisionBuildSpec<'a>,
    drift_signal: Option<GovernanceDriftSignal>,
    note: Option<String>,
    certification_status: Option<GovernanceCertificationStatus>,
    certification_subsystem: Option<String>,
}

macro_rules! governance_decision_spec {
    (
        $rule_id:expr,
        $subsystem_id:expr,
        $outcome:expr,
        $severity:expr,
        $response_class:expr,
        $reason_code:expr,
        $session_id:expr,
        $turn_id:expr $(,)?
    ) => {
        GovernanceDecisionBuildSpec {
            rule_id: $rule_id,
            subsystem_id: $subsystem_id,
            outcome: $outcome,
            severity: $severity,
            response_class: $response_class,
            reason_code: $reason_code,
            session_id: $session_id,
            turn_id: $turn_id,
        }
    };
}

macro_rules! governance_violation_spec {
    (
        $rule_id:expr,
        $subsystem_id:expr,
        $outcome:expr,
        $severity:expr,
        $response_class:expr,
        $reason_code:expr,
        $session_id:expr,
        $turn_id:expr,
        $drift_signal:expr,
        $note:expr,
        $certification_status:expr,
        $certification_subsystem:expr $(,)?
    ) => {
        GovernanceViolationSpec {
            decision: governance_decision_spec!(
                $rule_id,
                $subsystem_id,
                $outcome,
                $severity,
                $response_class,
                $reason_code,
                $session_id,
                $turn_id,
            ),
            drift_signal: $drift_signal,
            note: $note,
            certification_status: $certification_status,
            certification_subsystem: $certification_subsystem,
        }
    };
}

#[derive(Debug, Clone)]
pub struct RuntimeGovernanceRuntime {
    config: RuntimeGovernanceConfig,
    rule_registry: Vec<GovernanceRuleDescriptor>,
    state: Arc<Mutex<RuntimeGovernanceStateStore>>,
}

impl Default for RuntimeGovernanceRuntime {
    fn default() -> Self {
        Self::new(RuntimeGovernanceConfig::mvp_v1())
    }
}

impl RuntimeGovernanceRuntime {
    pub fn new(config: RuntimeGovernanceConfig) -> Self {
        Self {
            config,
            rule_registry: default_rule_registry(),
            state: Arc::new(Mutex::new(RuntimeGovernanceStateStore::new())),
        }
    }

    pub fn snapshot(&self) -> RuntimeGovernanceSnapshot {
        let guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        RuntimeGovernanceSnapshot {
            policy_window: self.config.policy_window.clone(),
            rule_registry: self.rule_registry.clone(),
            decision_log: guard.decision_log.clone(),
            safe_mode_active: guard.safe_mode_active,
            quarantined_subsystems: guard.quarantined_subsystems.iter().cloned().collect(),
            subsystem_certifications: subsystem_certification_snapshot(&guard),
            cluster_consistency: guard.cluster_consistency,
            drift_signals: guard.drift_signals.iter().copied().collect(),
        }
    }

    pub fn decision_log_snapshot(&self) -> Vec<GovernanceDecisionLogEntry> {
        self.snapshot().decision_log
    }

    pub fn rule_registry_snapshot(&self) -> Vec<GovernanceRuleDescriptor> {
        self.rule_registry.clone()
    }

    pub fn runtime_node_id(&self) -> &str {
        &self.config.runtime_node_id
    }

    pub fn policy_version(&self) -> &str {
        &self.config.policy_window.governance_policy_version
    }

    pub fn exit_safe_mode(&self, note: &str) -> Result<(), ContractViolation> {
        let mut guard = self
            .state
            .lock()
            .map_err(|_| ContractViolation::InvalidValue {
                field: "runtime_governance.state",
                reason: "lock poisoned",
            })?;
        guard.safe_mode_active = false;
        guard
            .quarantined_subsystems
            .remove(SUBSYSTEM_RUNTIME_GOVERNANCE);
        guard.subsystem_certifications.insert(
            SUBSYSTEM_RUNTIME_GOVERNANCE.to_string(),
            GovernanceCertificationStatus::Certified,
        );
        let decision = self.build_decision_from_locked(
            &guard,
            governance_decision_spec!(
                RULE_GOVERNANCE_INTEGRITY,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Passed,
                GovernanceSeverity::Info,
                GovernanceResponseClass::Allow,
                reason_codes::GOV_SAFE_MODE_ACTIVE,
                None,
                None,
            ),
        );
        let _ = self.record_existing_decision_locked(
            &mut guard,
            decision,
            Some(format!("safe mode exit: {note}")),
        )?;
        Ok(())
    }

    pub fn govern_voice_turn_execution(
        &self,
        envelope: &RuntimeExecutionEnvelope,
    ) -> Result<RuntimeExecutionEnvelope, Box<RuntimeGovernanceDecision>> {
        if self.config.force_integrity_failure {
            return Err(Box::new(self.enter_safe_mode(
                reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some("forced governance integrity failure".to_string()),
            )));
        }

        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    governance_decision_spec!(
                        RULE_GOVERNANCE_INTEGRITY,
                        SUBSYSTEM_RUNTIME_GOVERNANCE,
                        GovernanceDecisionOutcome::SafeModeActive,
                        GovernanceSeverity::Critical,
                        GovernanceResponseClass::SafeMode,
                        reason_codes::GOV_SAFE_MODE_ACTIVE,
                        envelope.session_id.map(|value| value.0),
                        Some(envelope.turn_id.0),
                    ),
                );
                drop(guard);
                return Err(Box::new(self.record_governance_decision(
                    decision,
                    Some("safe mode blocks protected voice execution".to_string()),
                    None,
                    None,
                    None,
                )));
            }
        }

        if envelope.admission_state != AdmissionState::IngressValidated
            && envelope.session_id.is_none()
        {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_SESSION_ENGINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("session-first execution requires canonical session_id".to_string()),
                None,
                None,
            ))));
        }
        if envelope.admission_state != AdmissionState::ExecutionAdmitted {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "governance-first protected execution requires the admitted Section 03 handoff"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.device_turn_sequence.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_DEVICE_SEQUENCE_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("device turn sequence is mandatory for governed ordering".to_string()),
                None,
                None,
            ))));
        }
        if voice_turn_execution_has_deferred_state(envelope) {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "governance-first protected execution only accepts the clean Section 03 handoff"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }

        if let Some(persistence_state) = envelope.persistence_state.as_ref() {
            if persistence_state.recovery_mode == PersistenceRecoveryMode::QuarantinedLocalState
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::QuarantineRequired)
            {
                return Err(Box::new(self.apply_violation(governance_violation_spec!(
                    RULE_PERSISTENCE_QUARANTINE,
                    SUBSYSTEM_PERSISTENCE_SYNC,
                    GovernanceDecisionOutcome::Quarantined,
                    GovernanceSeverity::QuarantineRequired,
                    GovernanceResponseClass::Quarantine,
                    reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED,
                    envelope.session_id.map(|value| value.0),
                    Some(envelope.turn_id.0),
                    Some(GovernanceDriftSignal::PersistenceReplayViolation),
                    Some("persistence quarantine blocks protected execution".to_string()),
                    Some(GovernanceCertificationStatus::Quarantined),
                    Some(SUBSYSTEM_PERSISTENCE_SYNC.to_string()),
                ))));
            }
            if persistence_state.acknowledgement_state
                == PersistenceAcknowledgementState::StaleRejected
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::StaleRejected)
            {
                return Err(Box::new(self.apply_violation(governance_violation_spec!(
                    RULE_PERSISTENCE_STALE_REJECTED,
                    SUBSYSTEM_PERSISTENCE_SYNC,
                    GovernanceDecisionOutcome::Failed,
                    GovernanceSeverity::Blocking,
                    GovernanceResponseClass::Block,
                    reason_codes::GOV_PERSISTENCE_STALE_REJECTED,
                    envelope.session_id.map(|value| value.0),
                    Some(envelope.turn_id.0),
                    Some(GovernanceDriftSignal::PersistenceReplayViolation),
                    Some("stale persistence replay rejected by runtime governance".to_string()),
                    Some(GovernanceCertificationStatus::Degraded),
                    Some(SUBSYSTEM_PERSISTENCE_SYNC.to_string()),
                ))));
            }
            if persistence_state.recovery_mode == PersistenceRecoveryMode::DegradedRecovery
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::Retryable)
            {
                let decision = self.apply_violation(governance_violation_spec!(
                    RULE_PERSISTENCE_DEGRADED,
                    SUBSYSTEM_PERSISTENCE_SYNC,
                    GovernanceDecisionOutcome::Degraded,
                    GovernanceSeverity::Warning,
                    GovernanceResponseClass::Degrade,
                    reason_codes::GOV_PERSISTENCE_DEGRADED,
                    envelope.session_id.map(|value| value.0),
                    Some(envelope.turn_id.0),
                    Some(GovernanceDriftSignal::PersistenceReplayViolation),
                    Some("degraded persistence posture recorded".to_string()),
                    Some(GovernanceCertificationStatus::Degraded),
                    Some(SUBSYSTEM_PERSISTENCE_SYNC.to_string()),
                ));
                let envelope = envelope
                    .with_governance_state(Some(decision.governance_state.clone()))
                    .expect("governance state must validate");
                return Ok(envelope);
            }
        }

        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let decision = self.build_decision_from_locked(
            &guard,
            governance_decision_spec!(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Passed,
                GovernanceSeverity::Info,
                GovernanceResponseClass::Allow,
                reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
            ),
        );
        let decision = self
            .record_existing_decision_locked(
                &mut guard,
                decision,
                Some("runtime governance cleared canonical Section 04 voice execution".to_string()),
            )
            .expect("runtime governance decision must record");
        let envelope = envelope
            .with_governance_state(Some(decision.governance_state.clone()))
            .expect("governance state must validate");
        Ok(envelope)
    }

    pub fn govern_persistence_signal(
        &self,
        envelope: Option<&RuntimeExecutionEnvelope>,
        action_class: GovernanceProtectedActionClass,
        signal_reason: &str,
        note: Option<String>,
    ) -> RuntimeGovernanceDecision {
        let normalized = signal_reason.to_ascii_uppercase();
        let (rule_id, outcome, severity, response_class, reason_code, certification_status) =
            if normalized.contains("QUARANTINE") {
                (
                    RULE_PERSISTENCE_QUARANTINE,
                    GovernanceDecisionOutcome::Quarantined,
                    GovernanceSeverity::QuarantineRequired,
                    GovernanceResponseClass::Quarantine,
                    reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED,
                    Some(GovernanceCertificationStatus::Quarantined),
                )
            } else if normalized.contains("STALE") {
                (
                    RULE_PERSISTENCE_STALE_REJECTED,
                    GovernanceDecisionOutcome::Failed,
                    GovernanceSeverity::Blocking,
                    GovernanceResponseClass::Block,
                    reason_codes::GOV_PERSISTENCE_STALE_REJECTED,
                    Some(GovernanceCertificationStatus::Degraded),
                )
            } else {
                (
                    RULE_PERSISTENCE_DEGRADED,
                    GovernanceDecisionOutcome::Degraded,
                    GovernanceSeverity::Warning,
                    GovernanceResponseClass::Degrade,
                    reason_codes::GOV_PERSISTENCE_DEGRADED,
                    Some(GovernanceCertificationStatus::Degraded),
                )
            };
        self.apply_violation(governance_violation_spec!(
            rule_id,
            SUBSYSTEM_PERSISTENCE_SYNC,
            outcome,
            severity,
            response_class,
            reason_code,
            envelope.and_then(|value| value.session_id.map(|session_id| session_id.0)),
            envelope.map(|value| value.turn_id.0),
            Some(GovernanceDriftSignal::PersistenceReplayViolation),
            Some(note.unwrap_or_else(|| {
                format!(
                    "persistence governance signal action={} detail={signal_reason}",
                    action_class.as_str()
                )
            })),
            certification_status,
            Some(SUBSYSTEM_PERSISTENCE_SYNC.to_string()),
        ))
    }

    pub fn govern_protected_action_proof(
        &self,
        action_class: GovernanceProtectedActionClass,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        proof_available: bool,
    ) -> Result<(), Box<RuntimeGovernanceDecision>> {
        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    governance_decision_spec!(
                        RULE_GOVERNANCE_INTEGRITY,
                        SUBSYSTEM_RUNTIME_GOVERNANCE,
                        GovernanceDecisionOutcome::SafeModeActive,
                        GovernanceSeverity::Critical,
                        GovernanceResponseClass::SafeMode,
                        reason_codes::GOV_SAFE_MODE_ACTIVE,
                        session_id,
                        turn_id,
                    ),
                );
                drop(guard);
                return Err(Box::new(self.record_governance_decision(
                    decision,
                    Some(format!(
                        "safe mode blocks protected action {}",
                        action_class.as_str()
                    )),
                    None,
                    None,
                    None,
                )));
            }
        }
        if !proof_available {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_PROOF_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(format!(
                    "proof-critical protected action {} refused without proof",
                    action_class.as_str()
                )),
                Some(GovernanceCertificationStatus::Warning),
                Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
            ))));
        }

        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let decision = self.build_decision_from_locked(
            &guard,
            governance_decision_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Passed,
                GovernanceSeverity::Info,
                GovernanceResponseClass::Allow,
                reason_codes::GOV_PROOF_REQUIRED,
                session_id,
                turn_id,
            ),
        );
        let _ = self
            .record_existing_decision_locked(
                &mut guard,
                decision,
                Some(format!(
                    "proof-critical protected action {} cleared governance",
                    action_class.as_str()
                )),
            )
            .expect("runtime governance decision must record");
        Ok(())
    }

    pub fn govern_protected_action_proof_state(
        &self,
        action_class: GovernanceProtectedActionClass,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        proof_state: &ProofExecutionState,
    ) -> Result<(), Box<RuntimeGovernanceDecision>> {
        let proof_available = matches!(
            proof_state.proof_write_outcome,
            selene_kernel_contracts::ph1j::ProofWriteOutcome::Written
                | selene_kernel_contracts::ph1j::ProofWriteOutcome::ReusedExisting
        ) && proof_state.proof_record_ref.is_some();
        if proof_available {
            return self.govern_protected_action_proof(action_class, session_id, turn_id, true);
        }
        let failure_note = match proof_state.proof_failure_class {
            Some(class) => format!(
                "proof-critical protected action {} refused due to {}",
                action_class.as_str(),
                class.as_str()
            ),
            None => format!(
                "proof-critical protected action {} refused due to missing proof state",
                action_class.as_str()
            ),
        };
        Err(Box::new(self.apply_violation(governance_violation_spec!(
            RULE_PROOF_REQUIRED,
            SUBSYSTEM_PROOF_CAPTURE,
            GovernanceDecisionOutcome::Failed,
            match proof_state.proof_failure_class {
                Some(ProofFailureClass::ProofChainIntegrityFailure)
                | Some(ProofFailureClass::ProofSignatureFailure) => {
                    GovernanceSeverity::QuarantineRequired
                }
                _ => GovernanceSeverity::Blocking,
            },
            match proof_state.proof_failure_class {
                Some(ProofFailureClass::ProofChainIntegrityFailure)
                | Some(ProofFailureClass::ProofSignatureFailure) => {
                    GovernanceResponseClass::Quarantine
                }
                _ => GovernanceResponseClass::Block,
            },
            reason_codes::GOV_PROOF_REQUIRED,
            session_id,
            turn_id,
            Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
            Some(failure_note),
            Some(GovernanceCertificationStatus::Warning),
            Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
        ))))
    }

    pub fn govern_protected_action_proof_state_execution(
        &self,
        envelope: &RuntimeExecutionEnvelope,
        action_class: GovernanceProtectedActionClass,
    ) -> Result<RuntimeExecutionEnvelope, Box<RuntimeGovernanceDecision>> {
        if envelope.session_id.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_SESSION_ENGINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution requires the canonical runtime session".to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.admission_state != AdmissionState::ExecutionAdmitted {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution requires the admitted Section 03 handoff"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.device_turn_sequence.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_DEVICE_SEQUENCE_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution requires canonical device turn ordering"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        let Some(governance_state) = envelope.governance_state.as_ref() else {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution requires the accepted H11 governance_state"
                        .to_string(),
                ),
                None,
                None,
            ))));
        };
        if governance_state.decision_log_ref.is_none()
            || governance_state_has_deferred_artifact_trust_linkage(governance_state)
        {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution only accepts the canonical H11 governance-first posture"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if proof_state_execution_has_deferred_state(envelope) {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution only accepts governance_state plus proof_state"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        let Some(proof_state) = envelope.proof_state.as_ref() else {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_PROOF_REQUIRED,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "proof-governance execution requires canonical proof_state transport"
                        .to_string(),
                ),
                Some(GovernanceCertificationStatus::Warning),
                Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
            ))));
        };
        self.govern_protected_action_proof_state(
            action_class,
            envelope.session_id.map(|value| value.0),
            Some(envelope.turn_id.0),
            proof_state,
        )?;
        Ok(envelope.clone())
    }

    pub fn govern_artifact_activation_execution(
        &self,
        envelope: &RuntimeExecutionEnvelope,
    ) -> Result<RuntimeExecutionEnvelope, Box<RuntimeGovernanceDecision>> {
        let session_id = envelope.session_id.map(|value| value.0);
        let turn_id = Some(envelope.turn_id.0);
        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    governance_decision_spec!(
                        RULE_GOVERNANCE_INTEGRITY,
                        SUBSYSTEM_RUNTIME_GOVERNANCE,
                        GovernanceDecisionOutcome::SafeModeActive,
                        GovernanceSeverity::Critical,
                        GovernanceResponseClass::SafeMode,
                        reason_codes::GOV_SAFE_MODE_ACTIVE,
                        session_id,
                        turn_id,
                    ),
                );
                drop(guard);
                return Err(Box::new(self.record_governance_decision(
                    decision,
                    Some("safe mode blocks artifact-trust protected execution".to_string()),
                    None,
                    None,
                    None,
                )));
            }
        }
        if envelope.session_id.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_SESSION_ENGINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("artifact activation requires the canonical runtime session".to_string()),
                None,
                None,
            ))));
        }
        if envelope.admission_state != AdmissionState::ExecutionAdmitted {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("artifact activation requires the admitted Section 03 handoff".to_string()),
                None,
                None,
            ))));
        }
        if envelope.device_turn_sequence.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_DEVICE_SEQUENCE_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("artifact activation requires canonical device turn ordering".to_string()),
                None,
                None,
            ))));
        }
        let Some(governance_state) = envelope.governance_state.as_ref() else {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some("artifact activation requires the accepted H11 governance_state".to_string()),
                None,
                None,
            ))));
        };
        if governance_state.decision_log_ref.is_none()
            || governance_state_has_deferred_artifact_trust_linkage(governance_state)
        {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "artifact activation only accepts the pre-artifact-trust H11 governance posture"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if artifact_activation_execution_has_deferred_state(envelope) {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "artifact activation only accepts governance_state, proof_state, artifact_trust_state, and optional voice_identity_assertion"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        let Some(proof_state) = envelope.proof_state.as_ref() else {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_PROOF_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "artifact activation requires the accepted H12 proof-governance posture"
                        .to_string(),
                ),
                Some(GovernanceCertificationStatus::Warning),
                Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
            ))));
        };
        let proof_available = matches!(
            proof_state.proof_write_outcome,
            selene_kernel_contracts::ph1j::ProofWriteOutcome::Written
                | selene_kernel_contracts::ph1j::ProofWriteOutcome::ReusedExisting
        ) && proof_state.proof_record_ref.is_some();
        if !proof_available {
            let (severity, response_class) = match proof_state.proof_failure_class {
                Some(ProofFailureClass::ProofChainIntegrityFailure)
                | Some(ProofFailureClass::ProofSignatureFailure) => (
                    GovernanceSeverity::QuarantineRequired,
                    GovernanceResponseClass::Quarantine,
                ),
                _ => (GovernanceSeverity::Blocking, GovernanceResponseClass::Block),
            };
            let note = match proof_state.proof_failure_class {
                Some(class) => format!(
                    "artifact activation requires H12 proof-governance posture; current proof posture failed with {}",
                    class.as_str()
                ),
                None => {
                    "artifact activation requires H12 proof-governance posture with a proof record"
                        .to_string()
                }
            };
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Failed,
                severity,
                response_class,
                reason_codes::GOV_PROOF_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(note),
                Some(GovernanceCertificationStatus::Warning),
                Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
            ))));
        }
        let Some(artifact_trust_state) = envelope.artifact_trust_state.as_ref() else {
            return Err(Box::new(self.apply_violation_with_artifact_trust(
                governance_violation_spec!(
                        RULE_ARTIFACT_TRUST_REQUIRED,
                        SUBSYSTEM_ARTIFACT_AUTHORITY,
                        GovernanceDecisionOutcome::Failed,
                        GovernanceSeverity::Blocking,
                        GovernanceResponseClass::Block,
                        reason_codes::GOV_ARTIFACT_TRUST_REQUIRED,
                        session_id,
                        turn_id,
                        Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                        Some(
                            "artifact activation requires canonical artifact_trust_state transport"
                                .to_string(),
                        ),
                        Some(GovernanceCertificationStatus::Warning),
                        Some(SUBSYSTEM_ARTIFACT_AUTHORITY.to_string()),
                    ),
                None,
            )));
        };

        let linkage = artifact_trust_governance_linkage(artifact_trust_state);
        if !artifact_trust_evidence_complete(artifact_trust_state) {
            return Err(Box::new(self.apply_violation_with_artifact_trust(
                governance_violation_spec!(
                        RULE_ARTIFACT_TRUST_EVIDENCE,
                        SUBSYSTEM_ARTIFACT_AUTHORITY,
                        GovernanceDecisionOutcome::Failed,
                        GovernanceSeverity::Blocking,
                        GovernanceResponseClass::Block,
                        reason_codes::GOV_ARTIFACT_TRUST_EVIDENCE_INCOMPLETE,
                        session_id,
                        turn_id,
                        Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                        Some(
                            "artifact activation requires complete trust decision and proof linkage"
                                .to_string(),
                        ),
                        Some(GovernanceCertificationStatus::Warning),
                        Some(SUBSYSTEM_ARTIFACT_AUTHORITY.to_string()),
                    ),
                Some(&linkage),
            )));
        }

        if let Some(failure_class) = strongest_artifact_trust_failure(artifact_trust_state) {
            let (severity, response_class, outcome, certification_status, drift_signal) =
                artifact_trust_governance_failure_posture(failure_class);
            return Err(Box::new(self.apply_violation_with_artifact_trust(
                governance_violation_spec!(
                    RULE_ARTIFACT_TRUST_FAILED,
                    SUBSYSTEM_ARTIFACT_AUTHORITY,
                    outcome,
                    severity,
                    response_class,
                    reason_codes::GOV_ARTIFACT_TRUST_FAILED,
                    session_id,
                    turn_id,
                    drift_signal,
                    Some(format!(
                        "artifact activation blocked by canonical trust failure {failure_class:?}"
                    )),
                    Some(certification_status),
                    Some(SUBSYSTEM_ARTIFACT_AUTHORITY.to_string()),
                ),
                Some(&linkage),
            )));
        }

        let (outcome, severity, response_class, reason_code, certification_status) =
            if artifact_trust_is_degraded(artifact_trust_state) {
                (
                    GovernanceDecisionOutcome::Degraded,
                    GovernanceSeverity::Warning,
                    GovernanceResponseClass::Degrade,
                    reason_codes::GOV_ARTIFACT_TRUST_DEGRADED,
                    GovernanceCertificationStatus::Warning,
                )
            } else {
                (
                    GovernanceDecisionOutcome::Passed,
                    GovernanceSeverity::Info,
                    GovernanceResponseClass::Allow,
                    reason_codes::GOV_ARTIFACT_TRUST_REQUIRED,
                    GovernanceCertificationStatus::Certified,
                )
            };
        let note = if response_class == GovernanceResponseClass::Degrade {
            Some(
                "artifact activation allowed with warning from canonical degraded trust state"
                    .to_string(),
            )
        } else {
            Some("artifact activation cleared canonical trust governance".to_string())
        };
        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        self.update_certification_locked(
            &mut guard,
            SUBSYSTEM_ARTIFACT_AUTHORITY,
            certification_status,
        );
        let decision = self.build_decision_from_locked(
            &guard,
            governance_decision_spec!(
                if response_class == GovernanceResponseClass::Degrade {
                    RULE_ARTIFACT_TRUST_DEGRADED
                } else {
                    RULE_ARTIFACT_TRUST_REQUIRED
                },
                SUBSYSTEM_ARTIFACT_AUTHORITY,
                outcome,
                severity,
                response_class,
                reason_code,
                session_id,
                turn_id,
            ),
        );
        let decision = self
            .record_existing_decision_with_artifact_trust_locked(
                &mut guard,
                decision,
                note,
                Some(&linkage),
            )
            .expect("artifact trust governance decision must record");
        drop(guard);
        let envelope = envelope
            .with_governance_state(Some(decision.governance_state.clone()))
            .expect("governance state must validate");
        if let Some(assertion) = envelope.voice_identity_assertion.as_ref() {
            return self.govern_artifact_activation_identity_state_execution(&envelope, assertion);
        }
        Ok(envelope)
    }

    pub fn govern_artifact_activation_identity_state_execution(
        &self,
        envelope: &RuntimeExecutionEnvelope,
        assertion: &Ph1VoiceIdResponse,
    ) -> Result<RuntimeExecutionEnvelope, Box<RuntimeGovernanceDecision>> {
        let session_id = envelope.session_id.map(|value| value.0);
        let turn_id = Some(envelope.turn_id.0);
        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    governance_decision_spec!(
                        RULE_GOVERNANCE_INTEGRITY,
                        SUBSYSTEM_RUNTIME_GOVERNANCE,
                        GovernanceDecisionOutcome::SafeModeActive,
                        GovernanceSeverity::Critical,
                        GovernanceResponseClass::SafeMode,
                        reason_codes::GOV_SAFE_MODE_ACTIVE,
                        session_id,
                        turn_id,
                    ),
                );
                drop(guard);
                return Err(Box::new(
                    self.record_governance_decision(
                        decision,
                        Some(
                            "safe mode blocks canonical non-app identity-state construction"
                                .to_string(),
                        ),
                        None,
                        None,
                        None,
                    ),
                ));
            }
        }
        if envelope.session_id.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_SESSION_ENGINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires the canonical runtime session"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.admission_state != AdmissionState::ExecutionAdmitted {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires the admitted artifact-governed handoff"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.device_turn_sequence.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_DEVICE_SEQUENCE_REQUIRED,
                SUBSYSTEM_INGRESS_PIPELINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires canonical device turn ordering"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        let Some(governance_state) = envelope.governance_state.as_ref() else {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires the accepted artifact-governed governance_state"
                        .to_string(),
                ),
                None,
                None,
            ))));
        };
        if governance_state.decision_log_ref.is_none()
            || !governance_state_has_artifact_trust_linkage(governance_state)
        {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ARTIFACT_TRUST_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ARTIFACT_TRUST_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires the accepted artifact-trust governance linkage"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if envelope.proof_state.is_none() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_PROOF_REQUIRED,
                SUBSYSTEM_PROOF_CAPTURE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_PROOF_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires the accepted proof-governed posture"
                        .to_string(),
                ),
                Some(GovernanceCertificationStatus::Warning),
                Some(SUBSYSTEM_PROOF_CAPTURE.to_string()),
            ))));
        }
        let Some(artifact_trust_state) = envelope.artifact_trust_state.as_ref() else {
            return Err(Box::new(self.apply_violation_with_artifact_trust(
                governance_violation_spec!(
                    RULE_ARTIFACT_TRUST_REQUIRED,
                    SUBSYSTEM_ARTIFACT_AUTHORITY,
                    GovernanceDecisionOutcome::Failed,
                    GovernanceSeverity::Blocking,
                    GovernanceResponseClass::Block,
                    reason_codes::GOV_ARTIFACT_TRUST_REQUIRED,
                    session_id,
                    turn_id,
                    Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                    Some(
                        "identity-state construction requires canonical artifact_trust_state transport"
                            .to_string(),
                    ),
                    Some(GovernanceCertificationStatus::Warning),
                    Some(SUBSYSTEM_ARTIFACT_AUTHORITY.to_string()),
                ),
                None,
            )));
        };
        let linkage = artifact_trust_governance_linkage(artifact_trust_state);
        if !governance_state_matches_artifact_trust_linkage(governance_state, &linkage) {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_GOVERNANCE_INTEGRITY,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires governance-state linkage to match artifact-trust evidence"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if artifact_activation_identity_state_execution_has_deferred_state(envelope) {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_GOVERNANCE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction only accepts governance_state, proof_state, and artifact_trust_state"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        if assertion.validate().is_err() {
            return Err(Box::new(self.apply_violation(governance_violation_spec!(
                RULE_GOVERNANCE_INTEGRITY,
                SUBSYSTEM_IDENTITY_VOICE_ENGINE,
                GovernanceDecisionOutcome::Failed,
                GovernanceSeverity::Blocking,
                GovernanceResponseClass::Block,
                reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                session_id,
                turn_id,
                Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                Some(
                    "identity-state construction requires a valid PH1.VOICE.ID response"
                        .to_string(),
                ),
                None,
                None,
            ))));
        }
        let identity_state = identity_execution_state_from_voice_assertion(assertion, envelope)
            .map_err(|_| {
                Box::new(self.apply_violation(governance_violation_spec!(
                    RULE_GOVERNANCE_INTEGRITY,
                    SUBSYSTEM_IDENTITY_VOICE_ENGINE,
                    GovernanceDecisionOutcome::Failed,
                    GovernanceSeverity::Blocking,
                    GovernanceResponseClass::Block,
                    reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                    session_id,
                    turn_id,
                    Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                    Some(
                        "identity-state construction failed for the canonical PH1.VOICE.ID response"
                            .to_string(),
                    ),
                    None,
                    None,
                )))
            })?;
        envelope
            .clone()
            .with_identity_state(Some(identity_state))
            .map_err(|_| {
                Box::new(self.apply_violation(governance_violation_spec!(
                    RULE_GOVERNANCE_INTEGRITY,
                    SUBSYSTEM_RUNTIME_GOVERNANCE,
                    GovernanceDecisionOutcome::Failed,
                    GovernanceSeverity::Blocking,
                    GovernanceResponseClass::Block,
                    reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                    session_id,
                    turn_id,
                    Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
                    Some(
                        "identity-state attachment failed for the canonical artifact-governed envelope"
                            .to_string(),
                    ),
                    None,
                    None,
                )))
            })
    }

    pub fn observe_node_policy_version(
        &self,
        node_id: &str,
        observed_policy_version: Option<&str>,
    ) -> RuntimeGovernanceDecision {
        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let (cluster_consistency, outcome, severity, response_class, drift_signal, status) =
            match observed_policy_version {
                None => (
                    GovernanceClusterConsistency::Unknown,
                    GovernanceDecisionOutcome::Degraded,
                    GovernanceSeverity::Warning,
                    GovernanceResponseClass::Degrade,
                    None,
                    GovernanceCertificationStatus::Warning,
                ),
                Some(version) if version == self.policy_version() => (
                    GovernanceClusterConsistency::Consistent,
                    GovernanceDecisionOutcome::Passed,
                    GovernanceSeverity::Info,
                    GovernanceResponseClass::Allow,
                    None,
                    GovernanceCertificationStatus::Certified,
                ),
                Some(version) if self.policy_version_compatible(version) => (
                    GovernanceClusterConsistency::CompatibilityWindow,
                    GovernanceDecisionOutcome::Passed,
                    GovernanceSeverity::Warning,
                    GovernanceResponseClass::AllowWithWarning,
                    None,
                    GovernanceCertificationStatus::Warning,
                ),
                Some(_) => (
                    GovernanceClusterConsistency::Diverged,
                    GovernanceDecisionOutcome::Degraded,
                    GovernanceSeverity::Critical,
                    GovernanceResponseClass::Degrade,
                    Some(GovernanceDriftSignal::PolicyVersionDrift),
                    GovernanceCertificationStatus::Degraded,
                ),
            };
        guard.cluster_consistency = cluster_consistency;
        if let Some(version) = observed_policy_version {
            guard
                .observed_node_policy_versions
                .insert(node_id.to_string(), version.to_string());
        }
        if let Some(signal) = drift_signal {
            guard.drift_signals.insert(signal);
        }
        self.update_certification_locked(&mut guard, SUBSYSTEM_CLUSTER_GOVERNANCE, status);
        let decision = self.build_decision_from_locked(
            &guard,
            governance_decision_spec!(
                RULE_POLICY_VERSION_DRIFT,
                SUBSYSTEM_CLUSTER_GOVERNANCE,
                outcome,
                severity,
                response_class,
                reason_codes::GOV_POLICY_VERSION_DRIFT,
                None,
                None,
            ),
        );
        self.record_existing_decision_locked(
            &mut guard,
            decision,
            Some(match observed_policy_version {
                Some(version) => {
                    format!("observed remote node {node_id} governance policy version {version}")
                }
                None => format!("observed remote node {node_id} without governance policy version"),
            }),
        )
        .expect("runtime governance decision must record")
    }

    fn policy_version_compatible(&self, policy_version: &str) -> bool {
        policy_version >= self.config.policy_window.compatibility_floor.as_str()
            && policy_version <= self.config.policy_window.compatibility_ceiling.as_str()
    }

    fn enter_safe_mode(
        &self,
        reason_code: &str,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        note: Option<String>,
    ) -> RuntimeGovernanceDecision {
        self.apply_violation(GovernanceViolationSpec {
            decision: GovernanceDecisionBuildSpec {
                rule_id: RULE_GOVERNANCE_INTEGRITY,
                subsystem_id: SUBSYSTEM_RUNTIME_GOVERNANCE,
                outcome: GovernanceDecisionOutcome::SafeModeActive,
                severity: GovernanceSeverity::Critical,
                response_class: GovernanceResponseClass::SafeMode,
                reason_code,
                session_id,
                turn_id,
            },
            drift_signal: Some(GovernanceDriftSignal::SubsystemCertificationRegression),
            note,
            certification_status: Some(GovernanceCertificationStatus::Quarantined),
            certification_subsystem: Some(SUBSYSTEM_RUNTIME_GOVERNANCE.to_string()),
        })
    }

    fn apply_violation(&self, spec: GovernanceViolationSpec<'_>) -> RuntimeGovernanceDecision {
        self.apply_violation_with_artifact_trust(spec, None)
    }

    fn apply_violation_with_artifact_trust(
        &self,
        spec: GovernanceViolationSpec<'_>,
        artifact_linkage: Option<&ArtifactTrustGovernanceLinkage>,
    ) -> RuntimeGovernanceDecision {
        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let count = guard
            .violation_counts
            .entry(spec.decision.rule_id.to_string())
            .or_insert(0);
        *count = count.saturating_add(1);
        if *count >= self.config.repeated_violation_threshold {
            guard
                .drift_signals
                .insert(GovernanceDriftSignal::RepeatedArchitectureViolations);
        }
        if let Some(signal) = spec.drift_signal {
            guard.drift_signals.insert(signal);
        }
        if let (Some(subsystem), Some(status)) = (
            spec.certification_subsystem.as_deref(),
            spec.certification_status,
        ) {
            self.update_certification_locked(&mut guard, subsystem, status);
        }
        match spec.decision.response_class {
            GovernanceResponseClass::Quarantine => {
                guard
                    .quarantined_subsystems
                    .insert(spec.decision.subsystem_id.to_string());
            }
            GovernanceResponseClass::SafeMode => {
                guard.safe_mode_active = true;
                guard
                    .quarantined_subsystems
                    .insert(SUBSYSTEM_RUNTIME_GOVERNANCE.to_string());
            }
            GovernanceResponseClass::Degrade => {
                if guard.cluster_consistency == GovernanceClusterConsistency::Consistent {
                    guard.cluster_consistency = GovernanceClusterConsistency::CompatibilityWindow;
                }
            }
            GovernanceResponseClass::Allow
            | GovernanceResponseClass::AllowWithWarning
            | GovernanceResponseClass::Block => {}
        }
        let decision = self.build_decision_from_locked(&guard, spec.decision);
        self.record_existing_decision_with_artifact_trust_locked(
            &mut guard,
            decision,
            spec.note,
            artifact_linkage,
        )
        .expect("runtime governance decision must record")
    }

    fn record_governance_decision(
        &self,
        decision: RuntimeGovernanceDecision,
        note: Option<String>,
        drift_signal: Option<GovernanceDriftSignal>,
        certification_subsystem: Option<&str>,
        certification_status: Option<GovernanceCertificationStatus>,
    ) -> RuntimeGovernanceDecision {
        self.record_governance_decision_with_artifact_trust(
            decision,
            note,
            drift_signal,
            certification_subsystem,
            certification_status,
            None,
        )
    }

    fn record_governance_decision_with_artifact_trust(
        &self,
        decision: RuntimeGovernanceDecision,
        note: Option<String>,
        drift_signal: Option<GovernanceDriftSignal>,
        certification_subsystem: Option<&str>,
        certification_status: Option<GovernanceCertificationStatus>,
        artifact_linkage: Option<&ArtifactTrustGovernanceLinkage>,
    ) -> RuntimeGovernanceDecision {
        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        if let Some(signal) = drift_signal {
            guard.drift_signals.insert(signal);
        }
        if let (Some(subsystem), Some(status)) = (certification_subsystem, certification_status) {
            self.update_certification_locked(&mut guard, subsystem, status);
        }
        self.record_existing_decision_with_artifact_trust_locked(
            &mut guard,
            decision,
            note,
            artifact_linkage,
        )
        .expect("runtime governance decision must record")
    }

    fn update_certification_locked(
        &self,
        guard: &mut RuntimeGovernanceStateStore,
        subsystem_id: &str,
        status: GovernanceCertificationStatus,
    ) {
        let previous = guard
            .subsystem_certifications
            .insert(subsystem_id.to_string(), status)
            .unwrap_or(GovernanceCertificationStatus::Certified);
        if certification_rank(status) > certification_rank(previous) {
            guard
                .drift_signals
                .insert(GovernanceDriftSignal::SubsystemCertificationRegression);
        }
        if status == GovernanceCertificationStatus::Quarantined {
            guard
                .quarantined_subsystems
                .insert(subsystem_id.to_string());
        }
    }

    fn build_decision_from_locked(
        &self,
        guard: &RuntimeGovernanceStateStore,
        spec: GovernanceDecisionBuildSpec<'_>,
    ) -> RuntimeGovernanceDecision {
        RuntimeGovernanceDecision {
            rule_id: spec.rule_id.to_string(),
            subsystem_id: spec.subsystem_id.to_string(),
            outcome: spec.outcome,
            severity: spec.severity,
            response_class: spec.response_class,
            reason_code: spec.reason_code.to_string(),
            session_id: spec.session_id,
            turn_id: spec.turn_id,
            governance_state: governance_execution_state_from_locked(
                &self.config.policy_window,
                guard,
                Some(spec.rule_id.to_string()),
                Some(spec.severity),
                Some(spec.response_class),
                Some(format!("gov_decision_{}", guard.next_sequence)),
                None,
            ),
        }
    }

    fn record_existing_decision_locked(
        &self,
        guard: &mut RuntimeGovernanceStateStore,
        decision: RuntimeGovernanceDecision,
        note: Option<String>,
    ) -> Result<RuntimeGovernanceDecision, ContractViolation> {
        self.record_existing_decision_with_artifact_trust_locked(guard, decision, note, None)
    }

    fn record_existing_decision_with_artifact_trust_locked(
        &self,
        guard: &mut RuntimeGovernanceStateStore,
        mut decision: RuntimeGovernanceDecision,
        note: Option<String>,
        artifact_linkage: Option<&ArtifactTrustGovernanceLinkage>,
    ) -> Result<RuntimeGovernanceDecision, ContractViolation> {
        let sequence = guard.next_sequence;
        guard.next_sequence = guard.next_sequence.saturating_add(1);
        let entry = GovernanceDecisionLogEntry::v1(
            sequence,
            decision.rule_id.clone(),
            decision.subsystem_id.clone(),
            self.config.policy_window.governance_policy_version.clone(),
            decision.outcome,
            decision.severity,
            decision.response_class,
            decision.reason_code.clone(),
            decision.session_id,
            decision.turn_id,
            self.config.runtime_node_id.clone(),
            note,
        )?;
        let entry = if let Some(linkage) = artifact_linkage {
            entry.with_artifact_trust_linkage(
                linkage.decision_ids.clone(),
                linkage.proof_entry_refs.clone(),
                linkage.proof_record_ref.clone(),
                linkage.policy_snapshot_refs.clone(),
                linkage.trust_set_snapshot_refs.clone(),
                linkage.basis_fingerprints.clone(),
                linkage.negative_result_refs.clone(),
            )?
        } else {
            entry
        };
        guard.decision_log.push(entry);
        decision.governance_state = governance_execution_state_from_locked(
            &self.config.policy_window,
            guard,
            Some(decision.rule_id.clone()),
            Some(decision.severity),
            Some(decision.response_class),
            Some(format!("gov_decision_{sequence}")),
            artifact_linkage,
        );
        Ok(decision)
    }
}

fn governance_execution_state_from_locked(
    policy_window: &GovernancePolicyWindow,
    guard: &RuntimeGovernanceStateStore,
    last_rule_id: Option<String>,
    last_severity: Option<GovernanceSeverity>,
    last_response_class: Option<GovernanceResponseClass>,
    decision_log_ref: Option<String>,
    artifact_linkage: Option<&ArtifactTrustGovernanceLinkage>,
) -> GovernanceExecutionState {
    let state = GovernanceExecutionState::v1(
        policy_window.governance_policy_version.clone(),
        guard.cluster_consistency,
        guard.safe_mode_active,
        guard.quarantined_subsystems.iter().cloned().collect(),
        subsystem_certification_snapshot(guard),
        guard.drift_signals.iter().copied().collect(),
        last_rule_id,
        last_severity,
        last_response_class,
        decision_log_ref,
    )
    .expect("governance execution state must validate");
    if let Some(linkage) = artifact_linkage {
        state
            .with_artifact_trust_linkage(
                linkage.decision_ids.clone(),
                linkage.proof_entry_refs.clone(),
                linkage.proof_record_ref.clone(),
                linkage.policy_snapshot_refs.clone(),
                linkage.trust_set_snapshot_refs.clone(),
                linkage.basis_fingerprints.clone(),
                linkage.negative_result_refs.clone(),
            )
            .expect("governance execution artifact trust linkage must validate")
    } else {
        state
    }
}

fn subsystem_certification_snapshot(
    guard: &RuntimeGovernanceStateStore,
) -> Vec<GovernanceSubsystemCertification> {
    guard
        .subsystem_certifications
        .iter()
        .map(|(subsystem_id, status)| {
            GovernanceSubsystemCertification::v1(subsystem_id.clone(), *status)
                .expect("governance subsystem certification must validate")
        })
        .collect()
}

fn voice_turn_execution_has_deferred_state(envelope: &RuntimeExecutionEnvelope) -> bool {
    envelope.governance_state.is_some()
        || envelope.proof_state.is_some()
        || envelope.computation_state.is_some()
        || envelope.identity_state.is_some()
        || envelope.memory_state.is_some()
        || envelope.authority_state.is_some()
        || envelope.artifact_trust_state.is_some()
        || envelope.law_state.is_some()
}

fn proof_state_execution_has_deferred_state(envelope: &RuntimeExecutionEnvelope) -> bool {
    envelope.persistence_state.is_some()
        || envelope.computation_state.is_some()
        || envelope.identity_state.is_some()
        || envelope.memory_state.is_some()
        || envelope.authority_state.is_some()
        || envelope.artifact_trust_state.is_some()
        || envelope.law_state.is_some()
}

fn artifact_activation_execution_has_deferred_state(envelope: &RuntimeExecutionEnvelope) -> bool {
    envelope.persistence_state.is_some()
        || envelope.computation_state.is_some()
        || envelope.identity_state.is_some()
        || envelope.memory_state.is_some()
        || envelope.authority_state.is_some()
        || envelope.law_state.is_some()
}

fn artifact_activation_identity_state_execution_has_deferred_state(
    envelope: &RuntimeExecutionEnvelope,
) -> bool {
    envelope.persistence_state.is_some()
        || envelope.computation_state.is_some()
        || envelope.identity_state.is_some()
        || envelope.memory_state.is_some()
        || envelope.authority_state.is_some()
        || envelope.law_state.is_some()
}

fn governance_state_has_deferred_artifact_trust_linkage(state: &GovernanceExecutionState) -> bool {
    !state.artifact_trust_decision_ids.is_empty()
        || !state.artifact_trust_proof_entry_refs.is_empty()
        || state.artifact_trust_proof_record_ref.is_some()
}

fn governance_state_has_artifact_trust_linkage(state: &GovernanceExecutionState) -> bool {
    !state.artifact_trust_decision_ids.is_empty()
}

fn governance_state_matches_artifact_trust_linkage(
    state: &GovernanceExecutionState,
    linkage: &ArtifactTrustGovernanceLinkage,
) -> bool {
    state.artifact_trust_decision_ids == linkage.decision_ids
        && state.artifact_trust_proof_entry_refs == linkage.proof_entry_refs
        && state.artifact_trust_proof_record_ref == linkage.proof_record_ref
        && state.artifact_trust_policy_snapshot_refs == linkage.policy_snapshot_refs
        && state.artifact_trust_set_snapshot_refs == linkage.trust_set_snapshot_refs
        && state.artifact_trust_basis_fingerprints == linkage.basis_fingerprints
        && state.artifact_trust_negative_result_refs == linkage.negative_result_refs
}

fn governance_quarantines_subsystem(
    governance_state: &GovernanceExecutionState,
    subsystem_id: &str,
) -> bool {
    governance_state
        .quarantined_subsystems
        .iter()
        .any(|candidate| candidate == subsystem_id)
}

pub fn attach_identity_state_for_governed_voice_turn(
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
    assertion: &Ph1VoiceIdResponse,
) -> Result<RuntimeExecutionEnvelope, ContractViolation> {
    if runtime_execution_envelope.session_id.is_none() {
        return Err(ContractViolation::InvalidValue {
            field: "runtime_execution_envelope.session_id",
            reason: "governed_voice_identity_state_requires_session",
        });
    }
    if runtime_execution_envelope.admission_state != AdmissionState::ExecutionAdmitted {
        return Err(ContractViolation::InvalidValue {
            field: "runtime_execution_envelope.admission_state",
            reason: "governed_voice_identity_state_requires_execution_admission",
        });
    }
    if runtime_execution_envelope.device_turn_sequence.is_none() {
        return Err(ContractViolation::InvalidValue {
            field: "runtime_execution_envelope.device_turn_sequence",
            reason: "governed_voice_identity_state_requires_device_turn_sequence",
        });
    }
    let Some(governance_state) = runtime_execution_envelope.governance_state.as_ref() else {
        return Err(ContractViolation::InvalidValue {
            field: "runtime_execution_envelope.governance_state",
            reason: "governed_voice_identity_state_requires_governance_state",
        });
    };
    if governance_state.decision_log_ref.is_none() {
        return Err(ContractViolation::InvalidValue {
            field: "runtime_execution_envelope.governance_state.decision_log_ref",
            reason: "governed_voice_identity_state_requires_governance_decision_log",
        });
    }
    assertion.validate()?;
    let identity_state =
        identity_execution_state_from_voice_assertion(assertion, runtime_execution_envelope)?;
    runtime_execution_envelope.with_identity_state(Some(identity_state))
}

fn identity_execution_state_from_voice_assertion(
    assertion: &Ph1VoiceIdResponse,
    runtime_execution_envelope: &RuntimeExecutionEnvelope,
) -> Result<IdentityExecutionState, ContractViolation> {
    let governance_identity_quarantined = runtime_execution_envelope
        .governance_state
        .as_ref()
        .map(|state| {
            state.safe_mode_active
                || governance_quarantines_subsystem(state, SUBSYSTEM_IDENTITY_VOICE_ENGINE)
        })
        .unwrap_or(false);
    let cluster_drift_detected = runtime_execution_envelope
        .governance_state
        .as_ref()
        .map(|state| state.cluster_consistency != GovernanceClusterConsistency::Consistent)
        .unwrap_or(false);

    match assertion {
        Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => {
            let mut consistency_level = match ok.identity_v2.identity_tier_v2 {
                IdentityTierV2::Confirmed => IdentityVerificationConsistencyLevel::StrictVerified,
                IdentityTierV2::Probable => {
                    IdentityVerificationConsistencyLevel::HighConfidenceVerified
                }
                IdentityTierV2::Unknown => {
                    IdentityVerificationConsistencyLevel::DegradedVerification
                }
            };
            let mut trust_tier = match ok.identity_v2.identity_tier_v2 {
                IdentityTierV2::Confirmed => IdentityTrustTier::Verified,
                IdentityTierV2::Probable => IdentityTrustTier::HighConfidence,
                IdentityTierV2::Unknown => IdentityTrustTier::Conditional,
            };
            let mut step_up_required = false;
            let mut recovery_state = IdentityRecoveryState::None;
            if cluster_drift_detected
                && consistency_level == IdentityVerificationConsistencyLevel::StrictVerified
            {
                consistency_level = IdentityVerificationConsistencyLevel::HighConfidenceVerified;
            }
            if ok.spoof_liveness_status == SpoofLivenessStatus::SuspectedSpoof {
                consistency_level = IdentityVerificationConsistencyLevel::RecoveryRestricted;
                trust_tier = IdentityTrustTier::Rejected;
                step_up_required = true;
                recovery_state = IdentityRecoveryState::RecoveryRestricted;
            } else if governance_identity_quarantined {
                consistency_level = IdentityVerificationConsistencyLevel::RecoveryRestricted;
                trust_tier = IdentityTrustTier::Restricted;
                step_up_required = true;
                recovery_state = IdentityRecoveryState::RecoveryRestricted;
            }
            IdentityExecutionState::v1(IdentityExecutionStateInput {
                consistency_level,
                trust_tier,
                identity_tier_v2: ok.identity_v2.identity_tier_v2,
                spoof_liveness_status: ok.spoof_liveness_status,
                step_up_required,
                recovery_state,
                cluster_drift_detected,
                reason_code: ok.reason_code.map(|code| u64::from(code.0)),
            })
        }
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(unknown) => {
            let (mut consistency_level, mut trust_tier, step_up_required, recovery_state) =
                match unknown.reason_code {
                    code if code == voice_id_reason_codes::VID_REAUTH_REQUIRED
                        || code == voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED =>
                    {
                        (
                            IdentityVerificationConsistencyLevel::RecoveryRestricted,
                            IdentityTrustTier::Restricted,
                            true,
                            IdentityRecoveryState::ReauthRequired,
                        )
                    }
                    code if code == voice_id_reason_codes::VID_ENROLLMENT_REQUIRED
                        || code == voice_id_reason_codes::VID_FAIL_PROFILE_NOT_ENROLLED =>
                    {
                        (
                            IdentityVerificationConsistencyLevel::RecoveryRestricted,
                            IdentityTrustTier::Restricted,
                            false,
                            IdentityRecoveryState::ReEnrollmentRequired,
                        )
                    }
                    code if code == voice_id_reason_codes::VID_SPOOF_RISK => (
                        IdentityVerificationConsistencyLevel::RecoveryRestricted,
                        IdentityTrustTier::Rejected,
                        true,
                        IdentityRecoveryState::RecoveryRestricted,
                    ),
                    _ if unknown.candidate_user_id.is_some() => (
                        IdentityVerificationConsistencyLevel::DegradedVerification,
                        IdentityTrustTier::Conditional,
                        false,
                        IdentityRecoveryState::None,
                    ),
                    _ => (
                        IdentityVerificationConsistencyLevel::DegradedVerification,
                        IdentityTrustTier::Restricted,
                        false,
                        IdentityRecoveryState::None,
                    ),
                };
            if cluster_drift_detected
                && consistency_level == IdentityVerificationConsistencyLevel::HighConfidenceVerified
            {
                consistency_level = IdentityVerificationConsistencyLevel::DegradedVerification;
            }
            if unknown.spoof_liveness_status == SpoofLivenessStatus::SuspectedSpoof {
                consistency_level = IdentityVerificationConsistencyLevel::RecoveryRestricted;
                trust_tier = IdentityTrustTier::Rejected;
            } else if governance_identity_quarantined {
                consistency_level = IdentityVerificationConsistencyLevel::RecoveryRestricted;
                trust_tier = IdentityTrustTier::Restricted;
            }
            IdentityExecutionState::v1(IdentityExecutionStateInput {
                consistency_level,
                trust_tier,
                identity_tier_v2: unknown.identity_v2.identity_tier_v2,
                spoof_liveness_status: unknown.spoof_liveness_status,
                step_up_required,
                recovery_state,
                cluster_drift_detected,
                reason_code: Some(u64::from(unknown.reason_code.0)),
            })
        }
    }
}

fn artifact_trust_governance_linkage(
    state: &ArtifactTrustExecutionState,
) -> ArtifactTrustGovernanceLinkage {
    let mut linkage = ArtifactTrustGovernanceLinkage::default();
    let mut policy_snapshot_refs_seen = BTreeSet::new();
    let mut trust_set_snapshot_refs_seen = BTreeSet::new();
    let mut negative_result_refs_seen = BTreeSet::new();
    for decision in &state.decision_records {
        linkage
            .decision_ids
            .push(decision.authority_decision_id.0.clone());
        linkage
            .basis_fingerprints
            .push(decision.verification_basis_fingerprint.0.clone());
        if let Some(proof_entry_ref) = decision.proof_entry_ref.as_ref() {
            linkage.proof_entry_refs.push(proof_entry_ref.0.clone());
        }
        if policy_snapshot_refs_seen.insert(decision.trust_policy_snapshot_ref.0.clone()) {
            linkage
                .policy_snapshot_refs
                .push(decision.trust_policy_snapshot_ref.0.clone());
        }
        if trust_set_snapshot_refs_seen.insert(decision.trust_set_snapshot_ref.0.clone()) {
            linkage
                .trust_set_snapshot_refs
                .push(decision.trust_set_snapshot_ref.0.clone());
        }
        if let Some(negative_verification_result_ref) =
            decision.negative_verification_result_ref.as_ref()
        {
            if negative_result_refs_seen.insert(negative_verification_result_ref.0.clone()) {
                linkage
                    .negative_result_refs
                    .push(negative_verification_result_ref.0.clone());
            }
        }
    }
    linkage.proof_record_ref = state.proof_record_ref.as_ref().map(|value| value.0.clone());
    linkage
}

fn artifact_trust_evidence_complete(state: &ArtifactTrustExecutionState) -> bool {
    let proof_required = state
        .decision_records
        .iter()
        .any(|decision| decision.control_hints.proof_required_for_completion);
    if !proof_required {
        return true;
    }
    if state.proof_record_ref.is_none() {
        return false;
    }
    state.decision_records.iter().all(|decision| {
        !decision.control_hints.proof_required_for_completion || decision.proof_entry_ref.is_some()
    })
}

fn strongest_artifact_trust_failure(
    state: &ArtifactTrustExecutionState,
) -> Option<ArtifactVerificationFailureClass> {
    state.decision_records.iter().find_map(|decision| {
        if decision
            .artifact_verification_result
            .artifact_verification_outcome
            == ArtifactVerificationOutcome::Failed
        {
            decision
                .artifact_verification_result
                .artifact_verification_failure_class
        } else {
            None
        }
    })
}

fn artifact_trust_is_degraded(state: &ArtifactTrustExecutionState) -> bool {
    state.decision_records.iter().any(|decision| {
        decision
            .artifact_verification_result
            .artifact_verification_outcome
            == ArtifactVerificationOutcome::DegradedVerified
    })
}

fn artifact_trust_governance_failure_posture(
    failure_class: ArtifactVerificationFailureClass,
) -> (
    GovernanceSeverity,
    GovernanceResponseClass,
    GovernanceDecisionOutcome,
    GovernanceCertificationStatus,
    Option<GovernanceDriftSignal>,
) {
    match failure_class {
        ArtifactVerificationFailureClass::ClusterTrustDivergence => (
            GovernanceSeverity::QuarantineRequired,
            GovernanceResponseClass::Quarantine,
            GovernanceDecisionOutcome::Quarantined,
            GovernanceCertificationStatus::Quarantined,
            Some(GovernanceDriftSignal::PolicyVersionDrift),
        ),
        ArtifactVerificationFailureClass::TrustRootRevoked => (
            GovernanceSeverity::QuarantineRequired,
            GovernanceResponseClass::Quarantine,
            GovernanceDecisionOutcome::Quarantined,
            GovernanceCertificationStatus::Quarantined,
            Some(GovernanceDriftSignal::SubsystemCertificationRegression),
        ),
        _ => (
            GovernanceSeverity::Blocking,
            GovernanceResponseClass::Block,
            GovernanceDecisionOutcome::Failed,
            GovernanceCertificationStatus::Degraded,
            Some(GovernanceDriftSignal::EnvelopeIntegrityDrift),
        ),
    }
}

fn certification_rank(status: GovernanceCertificationStatus) -> u8 {
    match status {
        GovernanceCertificationStatus::Certified => 0,
        GovernanceCertificationStatus::Warning => 1,
        GovernanceCertificationStatus::Degraded => 2,
        GovernanceCertificationStatus::Quarantined => 3,
        GovernanceCertificationStatus::Uncertified => 4,
    }
}

fn default_runtime_node_id() -> String {
    std::env::var("SELENE_RUNTIME_NODE_ID")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "selene_os_node_v1".to_string())
}

fn default_rule_registry() -> Vec<GovernanceRuleDescriptor> {
    vec![
        GovernanceRuleDescriptor::v1(
            RULE_ENV_SESSION_REQUIRED.to_string(),
            GovernanceRuleCategory::SessionFirst,
            SUBSYSTEM_SESSION_ENGINE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ENV_DEVICE_SEQUENCE_REQUIRED.to_string(),
            GovernanceRuleCategory::EnvelopeDiscipline,
            SUBSYSTEM_INGRESS_PIPELINE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ENV_ADMISSION_REQUIRED.to_string(),
            GovernanceRuleCategory::EnvelopeDiscipline,
            SUBSYSTEM_INGRESS_PIPELINE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_PERSISTENCE_DEGRADED.to_string(),
            GovernanceRuleCategory::PersistenceSync,
            SUBSYSTEM_PERSISTENCE_SYNC.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_PERSISTENCE_STALE_REJECTED.to_string(),
            GovernanceRuleCategory::PersistenceSync,
            SUBSYSTEM_PERSISTENCE_SYNC.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_PERSISTENCE_QUARANTINE.to_string(),
            GovernanceRuleCategory::PersistenceSync,
            SUBSYSTEM_PERSISTENCE_SYNC.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_PROOF_REQUIRED.to_string(),
            GovernanceRuleCategory::ProofCapture,
            SUBSYSTEM_PROOF_CAPTURE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_POLICY_VERSION_DRIFT.to_string(),
            GovernanceRuleCategory::CrossNodeConsensus,
            SUBSYSTEM_CLUSTER_GOVERNANCE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_SUBSYSTEM_CERT_REGRESSED.to_string(),
            GovernanceRuleCategory::SubsystemCertification,
            SUBSYSTEM_RUNTIME_GOVERNANCE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_GOVERNANCE_INTEGRITY.to_string(),
            GovernanceRuleCategory::GovernanceIntegrity,
            SUBSYSTEM_RUNTIME_GOVERNANCE.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ARTIFACT_TRUST_REQUIRED.to_string(),
            GovernanceRuleCategory::ArtifactTrust,
            SUBSYSTEM_ARTIFACT_AUTHORITY.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ARTIFACT_TRUST_EVIDENCE.to_string(),
            GovernanceRuleCategory::ArtifactTrust,
            SUBSYSTEM_ARTIFACT_AUTHORITY.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ARTIFACT_TRUST_FAILED.to_string(),
            GovernanceRuleCategory::ArtifactTrust,
            SUBSYSTEM_ARTIFACT_AUTHORITY.to_string(),
            "1".to_string(),
            false,
        )
        .expect("governance rule must validate"),
        GovernanceRuleDescriptor::v1(
            RULE_ARTIFACT_TRUST_DEGRADED.to_string(),
            GovernanceRuleCategory::ArtifactTrust,
            SUBSYSTEM_ARTIFACT_AUTHORITY.to_string(),
            "1".to_string(),
            true,
        )
        .expect("governance rule must validate"),
    ]
}

pub fn governance_failure_class_for_response(
    response_class: GovernanceResponseClass,
) -> FailureClass {
    match response_class {
        GovernanceResponseClass::Allow | GovernanceResponseClass::AllowWithWarning => {
            FailureClass::ExecutionFailure
        }
        GovernanceResponseClass::Degrade | GovernanceResponseClass::SafeMode => {
            FailureClass::RetryableRuntime
        }
        GovernanceResponseClass::Block => FailureClass::PolicyViolation,
        GovernanceResponseClass::Quarantine => FailureClass::SessionConflict,
    }
}

pub fn governance_reason_to_session_state(
    _response_class: GovernanceResponseClass,
) -> Option<SessionState> {
    None
}

pub fn governance_runtime_reason(decision: &RuntimeGovernanceDecision) -> String {
    match decision.response_class {
        GovernanceResponseClass::Quarantine => {
            format!("session_conflict {}", decision.reason_code)
        }
        GovernanceResponseClass::Block => {
            format!("governance_policy_block {}", decision.reason_code)
        }
        GovernanceResponseClass::SafeMode => {
            format!("governance_safe_mode {}", decision.reason_code)
        }
        GovernanceResponseClass::Degrade => {
            format!("governance_degrade {}", decision.reason_code)
        }
        GovernanceResponseClass::Allow | GovernanceResponseClass::AllowWithWarning => {
            format!("runtime_governance {}", decision.reason_code)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_law::RuntimeLawRuntime;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionOk,
        SpeakerAssertionUnknown, SpeakerId, SpeakerLabel, UserId,
    };
    use selene_kernel_contracts::ph1art::{
        ArtifactIdentityRef, ArtifactTrustBindingRef, ArtifactTrustControlHints,
        ArtifactTrustDecisionId, ArtifactTrustDecisionProvenance, ArtifactTrustDecisionRecord,
        ArtifactTrustExecutionState, ArtifactTrustProofEntryRef, ArtifactTrustProofRecordRef,
        ArtifactVerificationFailureClass, ArtifactVerificationOutcome, ArtifactVerificationResult,
        TrustPolicySnapshotRef, TrustSetSnapshotRef, VerificationBasisFingerprint,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1j::{DeviceId, TurnId};
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::runtime_execution::{
        AuthorityExecutionState, AuthorityPolicyDecision, IdentityExecutionState,
        IdentityExecutionStateInput, IdentityRecoveryState, IdentityTrustTier,
        IdentityVerificationConsistencyLevel, OnboardingReadinessState, PlatformRuntimeContext,
        ProofExecutionState, RuntimeEntryTrigger, RuntimeExecutionEnvelope,
        SimulationCertificationState,
    };
    use selene_kernel_contracts::runtime_law::{
        RuntimeLawEvaluationContext, RuntimeLawResponseClass, RuntimeProtectedActionClass,
    };
    use selene_kernel_contracts::MonotonicTimeNs;

    fn base_envelope() -> RuntimeExecutionEnvelope {
        RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request_1".to_string(),
            "trace_1".to_string(),
            "idem_1".to_string(),
            UserId::new("tenant_a:user_gov_test".to_string()).unwrap(),
            DeviceId::new("device_gov_1".to_string()).unwrap(),
            AppPlatform::Ios,
            PlatformRuntimeContext::default_for_platform_and_trigger(
                AppPlatform::Ios,
                RuntimeEntryTrigger::Explicit,
            )
            .unwrap(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::ExecutionAdmitted,
            None,
        )
        .unwrap()
    }

    fn verified_artifact_trust_state() -> ArtifactTrustExecutionState {
        ArtifactTrustExecutionState {
            decision_records: vec![ArtifactTrustDecisionRecord {
                authority_decision_id: ArtifactTrustDecisionId(
                    "authority.decision.gov.1".to_string(),
                ),
                artifact_identity_ref: ArtifactIdentityRef("artifact.identity.gov.1".to_string()),
                artifact_trust_binding_ref: ArtifactTrustBindingRef(
                    "artifact.trust.binding.gov.1".to_string(),
                ),
                trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.gov.1".to_string()),
                trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.gov.1".to_string()),
                artifact_verification_result: ArtifactVerificationResult {
                    artifact_identity_ref: ArtifactIdentityRef(
                        "artifact.identity.gov.1".to_string(),
                    ),
                    artifact_trust_binding_ref: ArtifactTrustBindingRef(
                        "artifact.trust.binding.gov.1".to_string(),
                    ),
                    trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                        "policy.snap.gov.1".to_string(),
                    ),
                    trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.gov.1".to_string()),
                    verification_basis_fingerprint: VerificationBasisFingerprint(
                        "basis.fp.gov.1".to_string(),
                    ),
                    artifact_verification_outcome: ArtifactVerificationOutcome::VerifiedFresh,
                    artifact_verification_failure_class: None,
                    negative_verification_result_ref: None,
                    verification_timestamp: MonotonicTimeNs(100),
                    verification_cache_used: false,
                    historical_snapshot_ref: None,
                },
                verification_basis_fingerprint: VerificationBasisFingerprint(
                    "basis.fp.gov.1".to_string(),
                ),
                negative_verification_result_ref: None,
                provenance: ArtifactTrustDecisionProvenance {
                    verifier_owner: "SECTION_04_AUTHORITY".to_string(),
                    verifier_version: "v1".to_string(),
                    trust_policy_snapshot_ref: TrustPolicySnapshotRef(
                        "policy.snap.gov.1".to_string(),
                    ),
                    trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.gov.1".to_string()),
                    evidence_refs: vec!["evidence.gov.1".to_string()],
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
                proof_entry_ref: Some(ArtifactTrustProofEntryRef(
                    "artifact.trust.proof.entry.gov.1".to_string(),
                )),
            }],
            primary_artifact_identity_ref: Some(ArtifactIdentityRef(
                "artifact.identity.gov.1".to_string(),
            )),
            proof_record_ref: Some(ArtifactTrustProofRecordRef(
                "artifact.trust.proof.record.gov.1".to_string(),
            )),
        }
    }

    fn available_proof_state() -> ProofExecutionState {
        ProofExecutionState::v1(
            Some("proof.record.gov.1".to_string()),
            selene_kernel_contracts::ph1j::ProofWriteOutcome::Written,
            None,
            selene_kernel_contracts::ph1j::ProofChainStatus::ChainLinked,
            selene_kernel_contracts::ph1j::ProofVerificationPosture::VerificationReady,
            selene_kernel_contracts::ph1j::TimestampTrustPosture::RuntimeMonotonic,
            Some("proof.meta.gov.1".to_string()),
        )
        .expect("proof state must validate")
    }

    fn failed_proof_state() -> ProofExecutionState {
        ProofExecutionState::v1(
            None,
            selene_kernel_contracts::ph1j::ProofWriteOutcome::Failed,
            Some(ProofFailureClass::ProofChainIntegrityFailure),
            selene_kernel_contracts::ph1j::ProofChainStatus::ChainBreakDetected,
            selene_kernel_contracts::ph1j::ProofVerificationPosture::VerificationUnavailable,
            selene_kernel_contracts::ph1j::TimestampTrustPosture::TrustedTimeUnavailable,
            Some("proof.meta.gov.fail.1".to_string()),
        )
        .expect("failed proof state must validate")
    }

    fn governance_linked_envelope() -> RuntimeExecutionEnvelope {
        let runtime = RuntimeGovernanceRuntime::default();
        runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governance-first envelope must be accepted")
    }

    fn proof_governed_envelope(runtime: &RuntimeGovernanceRuntime) -> RuntimeExecutionEnvelope {
        let governed = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governance-first execution must succeed");
        let proof_envelope = governed
            .with_proof_state(Some(available_proof_state()))
            .expect("proof state must attach");
        runtime
            .govern_protected_action_proof_state_execution(
                &proof_envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect("proof-governance execution must succeed")
    }

    fn artifact_governed_envelope(runtime: &RuntimeGovernanceRuntime) -> RuntimeExecutionEnvelope {
        let proof_governed = proof_governed_envelope(runtime);
        let envelope = proof_governed
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .expect("artifact-trust state must attach");
        runtime
            .govern_artifact_activation_execution(&envelope)
            .expect("artifact activation must succeed")
    }

    fn confirmed_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1(
                SpeakerId::new("spk_runtime_gov_confirmed").expect("speaker id must validate"),
                Some(user_id),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                SpeakerLabel::speaker_a(),
            )
            .expect("confirmed voice assertion must validate"),
        )
    }

    fn reauth_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_REAUTH_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                Some(user_id),
                None,
            )
            .expect("reauth voice assertion must validate"),
        )
    }

    fn device_claim_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                Some(user_id),
                None,
            )
            .expect("device-claim voice assertion must validate"),
        )
    }

    fn reenrollment_required_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_ENROLLMENT_REQUIRED,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                Some(user_id),
                None,
            )
            .expect("reenrollment voice assertion must validate"),
        )
    }

    fn spoof_risk_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_metrics_and_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_SPOOF_RISK,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                4500,
                None,
                SpoofLivenessStatus::SuspectedSpoof,
                vec![],
                Some(user_id),
                None,
            )
            .expect("spoof-risk voice assertion must validate"),
        )
    }

    fn low_confidence_voice_assertion(user_id: UserId) -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                Some(user_id),
                None,
            )
            .expect("low-confidence voice assertion must validate"),
        )
    }

    fn echo_unsafe_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Medium,
                voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE,
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("segment must validate")],
                None,
                None,
            )
            .expect("echo-unsafe voice assertion must validate"),
        )
    }

    fn no_speech_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_NO_SPEECH,
                vec![],
                None,
                None,
            )
            .expect("no-speech voice assertion must validate"),
        )
    }

    fn multi_speaker_voice_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1_with_candidate(
                IdentityConfidence::Low,
                voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT,
                vec![
                    DiarizationSegment::v1(
                        MonotonicTimeNs(1),
                        MonotonicTimeNs(2),
                        Some(SpeakerLabel::speaker_a()),
                    )
                    .expect("segment must validate"),
                    DiarizationSegment::v1(
                        MonotonicTimeNs(3),
                        MonotonicTimeNs(4),
                        Some(SpeakerLabel::speaker_b()),
                    )
                    .expect("segment must validate"),
                ],
                None,
                None,
            )
            .expect("multi-speaker voice assertion must validate"),
        )
    }

    fn allowed_authority_state() -> AuthorityExecutionState {
        AuthorityExecutionState::v1(
            Some(PolicyContextRef::v1(false, false, SafetyTier::Standard)),
            SimulationCertificationState::CertifiedActive,
            OnboardingReadinessState::Ready,
            AuthorityPolicyDecision::Allowed,
            true,
            true,
            false,
            Some(1),
        )
        .expect("authority state must validate")
    }

    #[test]
    fn at_runtime_gov_01_blocking_rule_triggers_block() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_session_and_admission_state(None, AdmissionState::ExecutionAdmitted)
            .unwrap();
        let decision = runtime
            .govern_voice_turn_execution(&envelope)
            .expect_err("missing canonical session must block");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(decision.severity, GovernanceSeverity::Blocking);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_SESSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02_critical_rule_can_trigger_quarantine() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_persistence_state(Some(
                selene_kernel_contracts::runtime_execution::PersistenceExecutionState::v1(
                    PersistenceRecoveryMode::QuarantinedLocalState,
                    PersistenceAcknowledgementState::QuarantinedLocalState,
                    Some(selene_kernel_contracts::runtime_execution::ReconciliationDecision::QuarantineLocalState),
                    Some(PersistenceConflictSeverity::QuarantineRequired),
                    false,
                    selene_kernel_contracts::runtime_execution::PersistenceConvergenceState::QuarantinedLocalState,
                    Some("audit_1".to_string()),
                )
                .unwrap(),
            ))
            .unwrap();
        let decision = runtime
            .govern_voice_turn_execution(&envelope)
            .expect_err("quarantined persistence must quarantine execution");
        assert_eq!(decision.response_class, GovernanceResponseClass::Quarantine);
        assert_eq!(decision.severity, GovernanceSeverity::QuarantineRequired);
    }

    #[test]
    fn at_runtime_gov_02b_section03_handoff_populates_only_governance_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope();
        let out = runtime
            .govern_voice_turn_execution(&envelope)
            .expect("accepted Section 03 handoff must enter governance-first execution");
        assert_eq!(out.request_id, envelope.request_id);
        assert_eq!(out.trace_id, envelope.trace_id);
        assert_eq!(out.idempotency_key, envelope.idempotency_key);
        assert_eq!(out.session_id, envelope.session_id);
        assert_eq!(out.turn_id, envelope.turn_id);
        assert_eq!(out.device_turn_sequence, envelope.device_turn_sequence);
        assert_eq!(out.admission_state, AdmissionState::ExecutionAdmitted);
        assert_eq!(out.persistence_state, envelope.persistence_state);
        let governance_state = out
            .governance_state
            .as_ref()
            .expect("governance state must be attached to the admitted envelope");
        assert_eq!(
            governance_state.last_response_class,
            Some(GovernanceResponseClass::Allow)
        );
        assert!(governance_state.decision_log_ref.is_some());
        assert!(governance_state.artifact_trust_decision_ids.is_empty());
        assert!(governance_state.artifact_trust_proof_entry_refs.is_empty());
        assert!(governance_state.artifact_trust_proof_record_ref.is_none());
        assert!(out.proof_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.identity_state.is_none());
        assert!(out.memory_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.artifact_trust_state.is_none());
        assert!(out.law_state.is_none());
        let log = runtime.decision_log_snapshot();
        let last = log
            .last()
            .expect("governance decision log entry must exist");
        assert_eq!(
            last.note.as_deref(),
            Some("runtime governance cleared canonical Section 04 voice execution")
        );
        assert!(last.artifact_trust_decision_ids.is_empty());
        assert!(last.artifact_trust_proof_entry_refs.is_empty());
        assert!(last.artifact_trust_proof_record_ref.is_none());
    }

    #[test]
    fn at_runtime_gov_02c_non_admitted_handoff_blocks_voice_execution() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_session_and_admission_state(Some(SessionId(1)), AdmissionState::IngressValidated)
            .unwrap();
        let decision = runtime
            .govern_voice_turn_execution(&envelope)
            .expect_err("non-admitted envelopes must fail closed");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
        let log = runtime.decision_log_snapshot();
        let last = log
            .last()
            .expect("governance decision log entry must exist");
        assert_eq!(
            last.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02d_later_section04_state_remains_deferred_for_voice_execution() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .unwrap();
        let decision = runtime
            .govern_voice_turn_execution(&envelope)
            .expect_err("later Section 04 artifact-trust posture must remain deferred");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02e_pre_governed_envelope_cannot_reenter_voice_execution() {
        let runtime = RuntimeGovernanceRuntime::default();
        let governed = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("first governance pass must succeed");
        let decision = runtime
            .govern_voice_turn_execution(&governed)
            .expect_err("governance-first execution must reject alternate reentry");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02f_proof_governance_reuses_h11_envelope_and_populates_only_proof_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let governed = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governance-first execution must succeed");
        let proof_envelope = governed
            .with_proof_state(Some(available_proof_state()))
            .expect("proof state must attach");

        let out = runtime
            .govern_protected_action_proof_state_execution(
                &proof_envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect("canonical admitted proof-governance handoff must succeed");

        assert_eq!(out.request_id, proof_envelope.request_id);
        assert_eq!(out.trace_id, proof_envelope.trace_id);
        assert_eq!(out.idempotency_key, proof_envelope.idempotency_key);
        assert_eq!(out.session_id, proof_envelope.session_id);
        assert_eq!(out.turn_id, proof_envelope.turn_id);
        assert_eq!(
            out.device_turn_sequence,
            proof_envelope.device_turn_sequence
        );
        assert_eq!(out.admission_state, AdmissionState::ExecutionAdmitted);
        assert_eq!(out.governance_state, governed.governance_state);
        assert_eq!(out.proof_state, proof_envelope.proof_state);
        assert!(out.persistence_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.identity_state.is_none());
        assert!(out.memory_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.artifact_trust_state.is_none());
        assert!(out.law_state.is_none());

        let log = runtime.decision_log_snapshot();
        let last = log
            .last()
            .expect("proof-governance decision log entry must exist");
        assert_eq!(last.reason_code, reason_codes::GOV_PROOF_REQUIRED);
        assert_eq!(
            last.note.as_deref(),
            Some("proof-critical protected action VOICE_TURN_EXECUTION cleared governance")
        );
        assert!(last.artifact_trust_decision_ids.is_empty());
        assert!(last.artifact_trust_proof_entry_refs.is_empty());
        assert!(last.artifact_trust_proof_record_ref.is_none());
    }

    #[test]
    fn at_runtime_gov_02g_proof_governance_requires_h11_governance_prerequisite() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_proof_state(Some(available_proof_state()))
            .expect("proof state must attach");

        let decision = runtime
            .govern_protected_action_proof_state_execution(
                &envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect_err("proof-governance requires the H11 governance-first output");

        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02h_proof_governance_rejects_non_admitted_handoff() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = governance_linked_envelope()
            .with_proof_state(Some(available_proof_state()))
            .and_then(|value| {
                value.with_session_and_admission_state(
                    value.session_id,
                    AdmissionState::IngressValidated,
                )
            })
            .expect("non-admitted envelope must validate structurally");

        let decision = runtime
            .govern_protected_action_proof_state_execution(
                &envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect_err("non-admitted proof-governance handoff must fail closed");

        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02i_proof_governance_rejects_malformed_proof_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = governance_linked_envelope()
            .with_proof_state(Some(failed_proof_state()))
            .expect("failed proof state must attach");

        let decision = runtime
            .govern_protected_action_proof_state_execution(
                &envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect_err("malformed proof posture must fail closed");

        assert_eq!(decision.response_class, GovernanceResponseClass::Quarantine);
        assert_eq!(decision.reason_code, reason_codes::GOV_PROOF_REQUIRED);
    }

    #[test]
    fn at_runtime_gov_02j_later_section04_state_remains_deferred_for_proof_governance() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = governance_linked_envelope()
            .with_proof_state(Some(available_proof_state()))
            .and_then(|value| {
                value.with_artifact_trust_state(Some(verified_artifact_trust_state()))
            })
            .expect("artifact-trust state must attach");

        let decision = runtime
            .govern_protected_action_proof_state_execution(
                &envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect_err("artifact-trust governance must remain deferred");

        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_02k_proof_governance_rejects_governance_state_with_later_linkage() {
        let runtime = RuntimeGovernanceRuntime::default();
        let governed = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governance-first execution must succeed");
        let mut governance_state = governed
            .governance_state
            .clone()
            .expect("governance-first execution must populate governance_state");
        governance_state.artifact_trust_decision_ids =
            vec!["artifact.decision.later.1".to_string()];
        let envelope = governed
            .with_governance_state(Some(governance_state))
            .and_then(|value| value.with_proof_state(Some(available_proof_state())))
            .expect("later governance linkage must attach");

        let decision = runtime
            .govern_protected_action_proof_state_execution(
                &envelope,
                GovernanceProtectedActionClass::VoiceTurnExecution,
            )
            .expect_err("proof-governance must reject alternate later authority posture");

        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_03_governance_integrity_failure_triggers_safe_mode() {
        let runtime = RuntimeGovernanceRuntime::new(
            RuntimeGovernanceConfig::mvp_v1().with_force_integrity_failure(true),
        );
        let decision = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect_err("integrity failure must trigger safe mode");
        assert_eq!(decision.response_class, GovernanceResponseClass::SafeMode);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN
        );
        assert!(runtime.snapshot().safe_mode_active);
    }

    #[test]
    fn at_runtime_gov_04_persistence_replay_anomaly_produces_governed_response() {
        let runtime = RuntimeGovernanceRuntime::default();
        let decision = runtime.govern_persistence_signal(
            Some(&base_envelope()),
            GovernanceProtectedActionClass::PersistenceReplay,
            "persistence_quarantine_required replay_request_mismatch",
            None,
        );
        assert_eq!(decision.response_class, GovernanceResponseClass::Quarantine);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_PERSISTENCE_QUARANTINE_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_05_governance_decision_log_records_deterministic_outcomes() {
        let runtime = RuntimeGovernanceRuntime::default();
        let first = runtime.govern_persistence_signal(
            Some(&base_envelope()),
            GovernanceProtectedActionClass::PersistenceReplay,
            "stale_replay_rejected",
            None,
        );
        let second = runtime.govern_persistence_signal(
            Some(&base_envelope()),
            GovernanceProtectedActionClass::PersistenceReplay,
            "stale_replay_rejected",
            None,
        );
        assert_eq!(first.rule_id, second.rule_id);
        assert_eq!(first.response_class, second.response_class);
        assert_eq!(first.reason_code, second.reason_code);
        let log = runtime.decision_log_snapshot();
        assert!(log.len() >= 2);
        assert_eq!(log[log.len() - 1].rule_id, RULE_PERSISTENCE_STALE_REJECTED);
    }

    #[test]
    fn at_runtime_gov_06_cross_node_policy_version_drift_is_detected() {
        let runtime = RuntimeGovernanceRuntime::default();
        let decision = runtime.observe_node_policy_version("node_b", Some("2026.04.01.v1"));
        assert_eq!(decision.response_class, GovernanceResponseClass::Degrade);
        assert!(runtime
            .snapshot()
            .drift_signals
            .contains(&GovernanceDriftSignal::PolicyVersionDrift));
    }

    #[test]
    fn at_runtime_gov_07_artifact_activation_missing_trust_state_blocks() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = proof_governed_envelope(&runtime);
        let decision = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect_err("artifact activation must refuse missing canonical trust state");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ARTIFACT_TRUST_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_08_cluster_divergence_quarantines_artifact_activation() {
        let runtime = RuntimeGovernanceRuntime::default();
        let mut state = verified_artifact_trust_state();
        state.decision_records[0]
            .artifact_verification_result
            .artifact_verification_outcome = ArtifactVerificationOutcome::Failed;
        state.decision_records[0]
            .artifact_verification_result
            .artifact_verification_failure_class =
            Some(ArtifactVerificationFailureClass::ClusterTrustDivergence);
        let envelope = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(state))
            .unwrap();
        let decision = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect_err("cluster divergence must quarantine artifact activation");
        assert_eq!(decision.response_class, GovernanceResponseClass::Quarantine);
    }

    #[test]
    fn at_runtime_gov_09_artifact_activation_requires_proof_linkage_when_hint_demands_it() {
        let runtime = RuntimeGovernanceRuntime::default();
        let mut state = verified_artifact_trust_state();
        state.proof_record_ref = None;
        state.decision_records[0].proof_entry_ref = None;
        let envelope = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(state))
            .unwrap();
        let decision = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect_err("proof-required artifact activation must block without proof linkage");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ARTIFACT_TRUST_EVIDENCE_INCOMPLETE
        );
    }

    #[test]
    fn at_runtime_gov_10_turn_level_proof_without_per_artifact_entry_still_blocks() {
        let runtime = RuntimeGovernanceRuntime::default();
        let mut state = verified_artifact_trust_state();
        state.decision_records[0].proof_entry_ref = None;
        let envelope = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(state))
            .unwrap();
        let decision = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect_err(
                "turn-level proof linkage must not substitute for per-artifact proof linkage",
            );
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ARTIFACT_TRUST_EVIDENCE_INCOMPLETE
        );
    }

    #[test]
    fn at_runtime_gov_11_verified_artifact_activation_records_canonical_linkage() {
        let runtime = RuntimeGovernanceRuntime::default();
        let proof_governed = proof_governed_envelope(&runtime);
        let envelope = proof_governed
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .unwrap();
        let out = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect("verified artifact activation must pass governance");
        assert_eq!(out.request_id, envelope.request_id);
        assert_eq!(out.trace_id, envelope.trace_id);
        assert_eq!(out.idempotency_key, envelope.idempotency_key);
        assert_eq!(out.session_id, envelope.session_id);
        assert_eq!(out.turn_id, envelope.turn_id);
        assert_eq!(out.device_turn_sequence, envelope.device_turn_sequence);
        assert_eq!(out.admission_state, AdmissionState::ExecutionAdmitted);
        assert_eq!(out.proof_state, proof_governed.proof_state);
        assert_eq!(out.artifact_trust_state, envelope.artifact_trust_state);
        assert!(out.identity_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.law_state.is_none());
        assert!(out.persistence_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.memory_state.is_none());
        let state = out
            .governance_state
            .expect("governance state must be attached to envelope");
        assert!(state.decision_log_ref.is_some());
        assert_eq!(
            state.artifact_trust_decision_ids,
            vec!["authority.decision.gov.1".to_string()]
        );
        assert_eq!(
            state.artifact_trust_proof_entry_refs,
            vec!["artifact.trust.proof.entry.gov.1".to_string()]
        );
        assert_eq!(
            state.artifact_trust_proof_record_ref.as_deref(),
            Some("artifact.trust.proof.record.gov.1")
        );
        let log = runtime.decision_log_snapshot();
        let last = log.last().expect("decision log entry must exist");
        assert_eq!(last.reason_code, reason_codes::GOV_ARTIFACT_TRUST_REQUIRED);
        assert_eq!(
            last.note.as_deref(),
            Some("artifact activation cleared canonical trust governance")
        );
        assert_eq!(
            last.artifact_trust_policy_snapshot_refs,
            vec!["policy.snap.gov.1".to_string()]
        );
    }

    #[test]
    fn at_runtime_gov_11b_artifact_activation_adopts_voice_identity_into_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let proof_governed = proof_governed_envelope(&runtime);
        let assertion = confirmed_voice_assertion(proof_governed.actor_identity.clone());
        let envelope = proof_governed
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .and_then(|value| value.with_voice_identity_assertion(Some(assertion.clone())))
            .expect("artifact-trust state and voice identity assertion must attach");

        let out = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect("artifact activation must adopt canonical voice identity transport");

        assert_eq!(out.request_id, envelope.request_id);
        assert_eq!(out.trace_id, envelope.trace_id);
        assert_eq!(out.idempotency_key, envelope.idempotency_key);
        assert_eq!(out.session_id, envelope.session_id);
        assert_eq!(out.turn_id, envelope.turn_id);
        assert_eq!(out.device_turn_sequence, envelope.device_turn_sequence);
        assert_eq!(out.admission_state, AdmissionState::ExecutionAdmitted);
        assert_eq!(out.proof_state, proof_governed.proof_state);
        assert_eq!(out.artifact_trust_state, envelope.artifact_trust_state);
        assert!(out.persistence_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.memory_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.law_state.is_none());
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("artifact activation must now attach identity_state when the canonical carrier is present");
        assert_eq!(
            *identity_state,
            IdentityExecutionState::v1(IdentityExecutionStateInput {
                consistency_level: IdentityVerificationConsistencyLevel::StrictVerified,
                trust_tier: IdentityTrustTier::Verified,
                identity_tier_v2: IdentityTierV2::Confirmed,
                spoof_liveness_status: SpoofLivenessStatus::Unknown,
                step_up_required: false,
                recovery_state: IdentityRecoveryState::None,
                cluster_drift_detected: false,
                reason_code: None,
            })
            .expect("expected identity state must validate")
        );
    }

    #[test]
    fn at_runtime_gov_12_artifact_activation_requires_h11_governance_and_h12_proof_prerequisites() {
        let runtime = RuntimeGovernanceRuntime::default();
        let envelope = base_envelope()
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .expect("artifact-trust state must attach");
        let decision = runtime
            .govern_artifact_activation_execution(&envelope)
            .expect_err("artifact activation requires the accepted H11 and H12 outputs");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );

        let governed_only = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governance-first execution must succeed")
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .expect("artifact-trust state must attach");
        let decision = runtime
            .govern_artifact_activation_execution(&governed_only)
            .expect_err("artifact activation requires the accepted H12 proof-governance output");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(decision.reason_code, reason_codes::GOV_PROOF_REQUIRED);
    }

    #[test]
    fn at_runtime_gov_13_artifact_activation_rejects_non_admitted_and_later_state_reentry() {
        let runtime = RuntimeGovernanceRuntime::default();
        let non_admitted = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .and_then(|value| {
                value.with_session_and_admission_state(
                    value.session_id,
                    AdmissionState::IngressValidated,
                )
            })
            .expect("non-admitted artifact envelope must validate structurally");
        let decision = runtime
            .govern_artifact_activation_execution(&non_admitted)
            .expect_err("artifact activation must fail closed on non-admitted handoff");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );

        let later_state = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .and_then(|value| {
                value.with_authority_state(Some(
                    AuthorityExecutionState::v1(
                        Some(PolicyContextRef::v1(false, false, SafetyTier::Standard)),
                        SimulationCertificationState::CertifiedActive,
                        OnboardingReadinessState::Ready,
                        AuthorityPolicyDecision::Allowed,
                        true,
                        true,
                        false,
                        Some(1),
                    )
                    .expect("authority state must validate"),
                ))
            })
            .expect("later authority state must attach");
        let decision = runtime
            .govern_artifact_activation_execution(&later_state)
            .expect_err("later protected state must remain deferred");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_14_artifact_activation_rejects_reentry_after_artifact_linkage() {
        let runtime = RuntimeGovernanceRuntime::default();
        let first_input = proof_governed_envelope(&runtime)
            .with_artifact_trust_state(Some(verified_artifact_trust_state()))
            .expect("artifact-trust state must attach");
        let first = runtime
            .govern_artifact_activation_execution(&first_input)
            .expect("first artifact activation pass must succeed");
        let decision = runtime
            .govern_artifact_activation_execution(&first)
            .expect_err("artifact activation must reject reentry after artifact linkage");
        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_15_identity_state_execution_constructs_canonical_non_app_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let assertion = confirmed_voice_assertion(artifact_governed.actor_identity.clone());

        let out = runtime
            .govern_artifact_activation_identity_state_execution(&artifact_governed, &assertion)
            .expect("identity-state construction must succeed after artifact activation");

        assert_eq!(out.request_id, artifact_governed.request_id);
        assert_eq!(out.trace_id, artifact_governed.trace_id);
        assert_eq!(out.idempotency_key, artifact_governed.idempotency_key);
        assert_eq!(out.session_id, artifact_governed.session_id);
        assert_eq!(out.turn_id, artifact_governed.turn_id);
        assert_eq!(
            out.device_turn_sequence,
            artifact_governed.device_turn_sequence
        );
        assert_eq!(out.governance_state, artifact_governed.governance_state);
        assert_eq!(out.proof_state, artifact_governed.proof_state);
        assert_eq!(
            out.artifact_trust_state,
            artifact_governed.artifact_trust_state
        );
        assert!(out.persistence_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.memory_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.law_state.is_none());
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity-state construction must attach canonical identity state");
        assert_eq!(
            *identity_state,
            IdentityExecutionState::v1(IdentityExecutionStateInput {
                consistency_level: IdentityVerificationConsistencyLevel::StrictVerified,
                trust_tier: IdentityTrustTier::Verified,
                identity_tier_v2: IdentityTierV2::Confirmed,
                spoof_liveness_status: SpoofLivenessStatus::Unknown,
                step_up_required: false,
                recovery_state: IdentityRecoveryState::None,
                cluster_drift_detected: false,
                reason_code: None,
            })
            .expect("expected identity state must validate")
        );
    }

    #[test]
    fn at_runtime_gov_16_identity_state_execution_rejects_prepopulated_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime)
            .with_identity_state(Some(
                IdentityExecutionState::v1(IdentityExecutionStateInput {
                    consistency_level: IdentityVerificationConsistencyLevel::StrictVerified,
                    trust_tier: IdentityTrustTier::Verified,
                    identity_tier_v2: IdentityTierV2::Confirmed,
                    spoof_liveness_status: SpoofLivenessStatus::Live,
                    step_up_required: false,
                    recovery_state: IdentityRecoveryState::None,
                    cluster_drift_detected: false,
                    reason_code: None,
                })
                .expect("identity state must validate"),
            ))
            .expect("identity state must attach");
        let assertion = confirmed_voice_assertion(artifact_governed.actor_identity.clone());

        let decision = runtime
            .govern_artifact_activation_identity_state_execution(&artifact_governed, &assertion)
            .expect_err("pre-populated identity_state must fail closed");

        assert_eq!(decision.response_class, GovernanceResponseClass::Block);
        assert_eq!(
            decision.reason_code,
            reason_codes::GOV_ENVELOPE_ADMISSION_REQUIRED
        );
    }

    #[test]
    fn at_runtime_gov_17_runtime_law_consumes_identity_state_from_canonical_non_app_flow() {
        let runtime = RuntimeGovernanceRuntime::default();
        let law = RuntimeLawRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let authority_state = allowed_authority_state();

        let without_identity = artifact_governed
            .clone()
            .with_authority_state(Some(authority_state.clone()))
            .expect("authority state must attach");
        let without_identity_decision = law.evaluate(
            &without_identity,
            RuntimeProtectedActionClass::IdentitySensitive,
            &RuntimeLawEvaluationContext::default(),
        );
        assert!(without_identity_decision
            .reason_codes
            .iter()
            .any(|code| code == crate::runtime_law::reason_codes::LAW_IDENTITY_POSTURE_REQUIRED));

        let with_identity = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &confirmed_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("identity-state construction must succeed")
            .with_authority_state(Some(authority_state))
            .expect("authority state must attach");
        let with_identity_decision = law.evaluate(
            &with_identity,
            RuntimeProtectedActionClass::IdentitySensitive,
            &RuntimeLawEvaluationContext::default(),
        );
        assert_eq!(
            with_identity_decision.response_class,
            RuntimeLawResponseClass::Allow
        );
        assert!(!with_identity_decision
            .reason_codes
            .iter()
            .any(|code| code == crate::runtime_law::reason_codes::LAW_IDENTITY_POSTURE_REQUIRED));
    }

    #[test]
    fn at_runtime_gov_18_reauth_assertion_maps_to_restricted_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &reauth_required_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("reauth-required assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for reauth-required posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReauthRequired
        );
    }

    #[test]
    fn at_runtime_gov_22_device_claim_required_assertion_maps_to_restricted_step_up_identity_state()
    {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &device_claim_required_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("device-claim assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for device-claim posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReauthRequired
        );
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_DEVICE_CLAIM_REQUIRED.0
            ))
        );
    }

    #[test]
    fn at_runtime_gov_23_low_confidence_assertion_maps_to_degraded_conditional_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &low_confidence_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("low-confidence assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for low-confidence posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Conditional);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_LOW_CONFIDENCE.0))
        );
    }

    #[test]
    fn at_runtime_gov_24_echo_unsafe_assertion_maps_to_degraded_restricted_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &echo_unsafe_voice_assertion(),
            )
            .expect("echo-unsafe assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for echo-unsafe posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_ECHO_UNSAFE.0))
        );
    }

    #[test]
    fn at_runtime_gov_25_no_speech_assertion_maps_to_degraded_restricted_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &no_speech_voice_assertion(),
            )
            .expect("no-speech assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for no-speech posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_FAIL_NO_SPEECH.0))
        );
    }

    #[test]
    fn at_runtime_gov_26_multi_speaker_assertion_maps_to_degraded_restricted_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &multi_speaker_voice_assertion(),
            )
            .expect("multi-speaker assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for multi-speaker posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::DegradedVerification
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(identity_state.recovery_state, IdentityRecoveryState::None);
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(
                voice_id_reason_codes::VID_FAIL_MULTI_SPEAKER_PRESENT.0
            ))
        );
    }

    #[test]
    fn at_runtime_gov_20_reenrollment_assertion_maps_to_restricted_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &reenrollment_required_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("reenrollment-required assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for reenrollment-required posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Restricted);
        assert!(!identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::ReEnrollmentRequired
        );
    }

    #[test]
    fn at_runtime_gov_21_spoof_risk_assertion_maps_to_rejected_identity_state() {
        let runtime = RuntimeGovernanceRuntime::default();
        let artifact_governed = artifact_governed_envelope(&runtime);
        let out = runtime
            .govern_artifact_activation_identity_state_execution(
                &artifact_governed,
                &spoof_risk_voice_assertion(artifact_governed.actor_identity.clone()),
            )
            .expect("spoof-risk assertion must still produce bounded identity state");
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for spoof-risk posture");
        assert_eq!(
            identity_state.consistency_level,
            IdentityVerificationConsistencyLevel::RecoveryRestricted
        );
        assert_eq!(identity_state.trust_tier, IdentityTrustTier::Rejected);
        assert!(identity_state.step_up_required);
        assert_eq!(
            identity_state.recovery_state,
            IdentityRecoveryState::RecoveryRestricted
        );
        assert_eq!(
            identity_state.reason_code,
            Some(u64::from(voice_id_reason_codes::VID_SPOOF_RISK.0))
        );
        assert_eq!(
            identity_state.spoof_liveness_status,
            SpoofLivenessStatus::SuspectedSpoof
        );
    }

    #[test]
    fn at_runtime_gov_19_governed_voice_turn_identity_attachment_constructs_canonical_non_app_identity_state(
    ) {
        let runtime = RuntimeGovernanceRuntime::default();
        let governed_voice_turn = runtime
            .govern_voice_turn_execution(&base_envelope())
            .expect("governed voice turn envelope must validate");
        let assertion = confirmed_voice_assertion(governed_voice_turn.actor_identity.clone());

        let out = attach_identity_state_for_governed_voice_turn(&governed_voice_turn, &assertion)
            .expect("governed voice turn must attach canonical identity state");

        assert_eq!(out.request_id, governed_voice_turn.request_id);
        assert_eq!(out.trace_id, governed_voice_turn.trace_id);
        assert_eq!(out.idempotency_key, governed_voice_turn.idempotency_key);
        assert_eq!(out.session_id, governed_voice_turn.session_id);
        assert_eq!(out.turn_id, governed_voice_turn.turn_id);
        assert_eq!(
            out.device_turn_sequence,
            governed_voice_turn.device_turn_sequence
        );
        assert_eq!(out.governance_state, governed_voice_turn.governance_state);
        assert!(out.proof_state.is_none());
        assert!(out.artifact_trust_state.is_none());
        assert!(out.persistence_state.is_none());
        assert!(out.computation_state.is_none());
        assert!(out.memory_state.is_none());
        assert!(out.authority_state.is_none());
        assert!(out.law_state.is_none());
        let identity_state = out
            .identity_state
            .as_ref()
            .expect("identity state must attach for governed voice turn");
        assert_eq!(
            *identity_state,
            IdentityExecutionState::v1(IdentityExecutionStateInput {
                consistency_level: IdentityVerificationConsistencyLevel::StrictVerified,
                trust_tier: IdentityTrustTier::Verified,
                identity_tier_v2: IdentityTierV2::Confirmed,
                spoof_liveness_status: SpoofLivenessStatus::Unknown,
                step_up_required: false,
                recovery_state: IdentityRecoveryState::None,
                cluster_drift_detected: false,
                reason_code: None,
            })
            .expect("expected identity state must validate")
        );
    }
}
