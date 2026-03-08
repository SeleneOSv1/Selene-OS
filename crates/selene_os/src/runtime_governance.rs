#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use selene_kernel_contracts::ph1j::ProofFailureClass;
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, FailureClass, PersistenceAcknowledgementState, PersistenceConflictSeverity,
    PersistenceRecoveryMode, ProofExecutionState, RuntimeExecutionEnvelope,
};
use selene_kernel_contracts::runtime_governance::{
    GovernanceCertificationStatus, GovernanceClusterConsistency, GovernanceDecisionLogEntry,
    GovernanceDecisionOutcome, GovernanceDriftSignal, GovernanceExecutionState,
    GovernancePolicyWindow, GovernanceProtectedActionClass, GovernanceResponseClass,
    GovernanceRuleCategory, GovernanceRuleDescriptor, GovernanceSeverity,
    GovernanceSubsystemCertification,
};
use selene_kernel_contracts::{ContractViolation, SessionState};

pub mod reason_codes {
    pub const GOV_ENVELOPE_SESSION_REQUIRED: &str = "GOV_ENVELOPE_SESSION_REQUIRED";
    pub const GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED: &str = "GOV_ENVELOPE_DEVICE_SEQUENCE_REQUIRED";
    pub const GOV_PERSISTENCE_DEGRADED: &str = "GOV_PERSISTENCE_DEGRADED";
    pub const GOV_PERSISTENCE_STALE_REJECTED: &str = "GOV_PERSISTENCE_STALE_REJECTED";
    pub const GOV_PERSISTENCE_QUARANTINE_REQUIRED: &str = "GOV_PERSISTENCE_QUARANTINE_REQUIRED";
    pub const GOV_PROOF_REQUIRED: &str = "GOV_PROOF_REQUIRED";
    pub const GOV_GOVERNANCE_INTEGRITY_UNCERTAIN: &str = "GOV_GOVERNANCE_INTEGRITY_UNCERTAIN";
    pub const GOV_POLICY_VERSION_DRIFT: &str = "GOV_POLICY_VERSION_DRIFT";
    pub const GOV_SUBSYSTEM_CERTIFICATION_REGRESSED: &str = "GOV_SUBSYSTEM_CERTIFICATION_REGRESSED";
    pub const GOV_SAFE_MODE_ACTIVE: &str = "GOV_SAFE_MODE_ACTIVE";
}

const SUBSYSTEM_RUNTIME_GOVERNANCE: &str = "RUNTIME_GOVERNANCE";
const SUBSYSTEM_INGRESS_PIPELINE: &str = "INGRESS_PIPELINE";
const SUBSYSTEM_SESSION_ENGINE: &str = "SESSION_ENGINE";
const SUBSYSTEM_PERSISTENCE_SYNC: &str = "PERSISTENCE_SYNC";
const SUBSYSTEM_PROOF_CAPTURE: &str = "PROOF_CAPTURE";
const SUBSYSTEM_CLUSTER_GOVERNANCE: &str = "CLUSTER_GOVERNANCE";

const RULE_ENV_SESSION_REQUIRED: &str = "RG-SESSION-001";
const RULE_ENV_DEVICE_SEQUENCE_REQUIRED: &str = "RG-ENV-001";
const RULE_PERSISTENCE_DEGRADED: &str = "RG-PERSIST-001";
const RULE_PERSISTENCE_STALE_REJECTED: &str = "RG-PERSIST-002";
const RULE_PERSISTENCE_QUARANTINE: &str = "RG-PERSIST-003";
const RULE_PROOF_REQUIRED: &str = "RG-PROOF-001";
const RULE_POLICY_VERSION_DRIFT: &str = "RG-CLUSTER-001";
const RULE_SUBSYSTEM_CERT_REGRESSED: &str = "RG-CERT-001";
const RULE_GOVERNANCE_INTEGRITY: &str = "RG-GOV-001";

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
            RULE_GOVERNANCE_INTEGRITY,
            SUBSYSTEM_RUNTIME_GOVERNANCE,
            GovernanceDecisionOutcome::Passed,
            GovernanceSeverity::Info,
            GovernanceResponseClass::Allow,
            reason_codes::GOV_SAFE_MODE_ACTIVE,
            None,
            None,
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
    ) -> Result<RuntimeExecutionEnvelope, RuntimeGovernanceDecision> {
        if self.config.force_integrity_failure {
            return Err(self.enter_safe_mode(
                reason_codes::GOV_GOVERNANCE_INTEGRITY_UNCERTAIN,
                envelope.session_id.map(|value| value.0),
                Some(envelope.turn_id.0),
                Some("forced governance integrity failure".to_string()),
            ));
        }

        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    RULE_GOVERNANCE_INTEGRITY,
                    SUBSYSTEM_RUNTIME_GOVERNANCE,
                    GovernanceDecisionOutcome::SafeModeActive,
                    GovernanceSeverity::Critical,
                    GovernanceResponseClass::SafeMode,
                    reason_codes::GOV_SAFE_MODE_ACTIVE,
                    envelope.session_id.map(|value| value.0),
                    Some(envelope.turn_id.0),
                );
                drop(guard);
                return Err(self.record_governance_decision(
                    decision,
                    Some("safe mode blocks protected voice execution".to_string()),
                    None,
                    None,
                    None,
                ));
            }
        }

        if envelope.admission_state != AdmissionState::IngressValidated
            && envelope.session_id.is_none()
        {
            return Err(self.apply_violation(
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
            ));
        }
        if envelope.device_turn_sequence.is_none() {
            return Err(self.apply_violation(
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
            ));
        }

        if let Some(persistence_state) = envelope.persistence_state.as_ref() {
            if persistence_state.recovery_mode == PersistenceRecoveryMode::QuarantinedLocalState
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::QuarantineRequired)
            {
                return Err(self.apply_violation(
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
                ));
            }
            if persistence_state.acknowledgement_state
                == PersistenceAcknowledgementState::StaleRejected
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::StaleRejected)
            {
                return Err(self.apply_violation(
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
                ));
            }
            if persistence_state.recovery_mode == PersistenceRecoveryMode::DegradedRecovery
                || persistence_state.conflict_severity
                    == Some(PersistenceConflictSeverity::Retryable)
            {
                let decision = self.apply_violation(
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
                );
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
            RULE_ENV_SESSION_REQUIRED,
            SUBSYSTEM_RUNTIME_GOVERNANCE,
            GovernanceDecisionOutcome::Passed,
            GovernanceSeverity::Info,
            GovernanceResponseClass::Allow,
            reason_codes::GOV_ENVELOPE_SESSION_REQUIRED,
            envelope.session_id.map(|value| value.0),
            Some(envelope.turn_id.0),
        );
        let decision = self
            .record_existing_decision_locked(
                &mut guard,
                decision,
                Some("runtime governance cleared voice execution".to_string()),
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
        self.apply_violation(
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
        )
    }

    pub fn govern_protected_action_proof(
        &self,
        action_class: GovernanceProtectedActionClass,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        proof_available: bool,
    ) -> Result<(), RuntimeGovernanceDecision> {
        {
            let guard = self
                .state
                .lock()
                .expect("runtime governance state lock poisoned");
            if guard.safe_mode_active {
                let decision = self.build_decision_from_locked(
                    &guard,
                    RULE_GOVERNANCE_INTEGRITY,
                    SUBSYSTEM_RUNTIME_GOVERNANCE,
                    GovernanceDecisionOutcome::SafeModeActive,
                    GovernanceSeverity::Critical,
                    GovernanceResponseClass::SafeMode,
                    reason_codes::GOV_SAFE_MODE_ACTIVE,
                    session_id,
                    turn_id,
                );
                drop(guard);
                return Err(self.record_governance_decision(
                    decision,
                    Some(format!(
                        "safe mode blocks protected action {}",
                        action_class.as_str()
                    )),
                    None,
                    None,
                    None,
                ));
            }
        }
        if !proof_available {
            return Err(self.apply_violation(
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
            ));
        }

        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let decision = self.build_decision_from_locked(
            &guard,
            RULE_PROOF_REQUIRED,
            SUBSYSTEM_PROOF_CAPTURE,
            GovernanceDecisionOutcome::Passed,
            GovernanceSeverity::Info,
            GovernanceResponseClass::Allow,
            reason_codes::GOV_PROOF_REQUIRED,
            session_id,
            turn_id,
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
    ) -> Result<(), RuntimeGovernanceDecision> {
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
        Err(self.apply_violation(
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
        ))
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
            RULE_POLICY_VERSION_DRIFT,
            SUBSYSTEM_CLUSTER_GOVERNANCE,
            outcome,
            severity,
            response_class,
            reason_codes::GOV_POLICY_VERSION_DRIFT,
            None,
            None,
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
        self.apply_violation(
            RULE_GOVERNANCE_INTEGRITY,
            SUBSYSTEM_RUNTIME_GOVERNANCE,
            GovernanceDecisionOutcome::SafeModeActive,
            GovernanceSeverity::Critical,
            GovernanceResponseClass::SafeMode,
            reason_code,
            session_id,
            turn_id,
            Some(GovernanceDriftSignal::SubsystemCertificationRegression),
            note,
            Some(GovernanceCertificationStatus::Quarantined),
            Some(SUBSYSTEM_RUNTIME_GOVERNANCE.to_string()),
        )
    }

    fn apply_violation(
        &self,
        rule_id: &str,
        subsystem_id: &str,
        outcome: GovernanceDecisionOutcome,
        severity: GovernanceSeverity,
        response_class: GovernanceResponseClass,
        reason_code: &str,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        drift_signal: Option<GovernanceDriftSignal>,
        note: Option<String>,
        certification_status: Option<GovernanceCertificationStatus>,
        certification_subsystem: Option<String>,
    ) -> RuntimeGovernanceDecision {
        let mut guard = self
            .state
            .lock()
            .expect("runtime governance state lock poisoned");
        let count = guard
            .violation_counts
            .entry(rule_id.to_string())
            .or_insert(0);
        *count = count.saturating_add(1);
        if *count >= self.config.repeated_violation_threshold {
            guard
                .drift_signals
                .insert(GovernanceDriftSignal::RepeatedArchitectureViolations);
        }
        if let Some(signal) = drift_signal {
            guard.drift_signals.insert(signal);
        }
        if let (Some(subsystem), Some(status)) =
            (certification_subsystem.as_deref(), certification_status)
        {
            self.update_certification_locked(&mut guard, subsystem, status);
        }
        match response_class {
            GovernanceResponseClass::Quarantine => {
                guard
                    .quarantined_subsystems
                    .insert(subsystem_id.to_string());
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
        let decision = self.build_decision_from_locked(
            &guard,
            rule_id,
            subsystem_id,
            outcome,
            severity,
            response_class,
            reason_code,
            session_id,
            turn_id,
        );
        self.record_existing_decision_locked(&mut guard, decision, note)
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
        self.record_existing_decision_locked(&mut guard, decision, note)
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
        rule_id: &str,
        subsystem_id: &str,
        outcome: GovernanceDecisionOutcome,
        severity: GovernanceSeverity,
        response_class: GovernanceResponseClass,
        reason_code: &str,
        session_id: Option<u128>,
        turn_id: Option<u64>,
    ) -> RuntimeGovernanceDecision {
        RuntimeGovernanceDecision {
            rule_id: rule_id.to_string(),
            subsystem_id: subsystem_id.to_string(),
            outcome,
            severity,
            response_class,
            reason_code: reason_code.to_string(),
            session_id,
            turn_id,
            governance_state: governance_execution_state_from_locked(
                &self.config.policy_window,
                guard,
                Some(rule_id.to_string()),
                Some(severity),
                Some(response_class),
                Some(format!("gov_decision_{}", guard.next_sequence)),
            ),
        }
    }

    fn record_existing_decision_locked(
        &self,
        guard: &mut RuntimeGovernanceStateStore,
        mut decision: RuntimeGovernanceDecision,
        note: Option<String>,
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
        guard.decision_log.push(entry);
        decision.governance_state = governance_execution_state_from_locked(
            &self.config.policy_window,
            guard,
            Some(decision.rule_id.clone()),
            Some(decision.severity),
            Some(decision.response_class),
            Some(format!("gov_decision_{sequence}")),
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
) -> GovernanceExecutionState {
    GovernanceExecutionState::v1(
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
    .expect("governance execution state must validate")
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
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1j::{DeviceId, TurnId};
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::runtime_execution::{
        PlatformRuntimeContext, RuntimeEntryTrigger, RuntimeExecutionEnvelope,
    };

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
}
