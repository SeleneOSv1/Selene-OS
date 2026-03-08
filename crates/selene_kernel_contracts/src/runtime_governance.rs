#![forbid(unsafe_code)]

use crate::{ContractViolation, Validate};

fn validate_ascii_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_optional_ascii_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value.as_ref() {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceRuleCategory {
    EnvelopeDiscipline,
    SessionFirst,
    PersistenceSync,
    ProofCapture,
    SubsystemCertification,
    CrossNodeConsensus,
    GovernanceIntegrity,
}

impl GovernanceRuleCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceRuleCategory::EnvelopeDiscipline => "ENVELOPE_DISCIPLINE",
            GovernanceRuleCategory::SessionFirst => "SESSION_FIRST",
            GovernanceRuleCategory::PersistenceSync => "PERSISTENCE_SYNC",
            GovernanceRuleCategory::ProofCapture => "PROOF_CAPTURE",
            GovernanceRuleCategory::SubsystemCertification => "SUBSYSTEM_CERTIFICATION",
            GovernanceRuleCategory::CrossNodeConsensus => "CROSS_NODE_CONSENSUS",
            GovernanceRuleCategory::GovernanceIntegrity => "GOVERNANCE_INTEGRITY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceSeverity {
    Info,
    Warning,
    Blocking,
    Critical,
    QuarantineRequired,
}

impl GovernanceSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceSeverity::Info => "INFO",
            GovernanceSeverity::Warning => "WARNING",
            GovernanceSeverity::Blocking => "BLOCKING",
            GovernanceSeverity::Critical => "CRITICAL",
            GovernanceSeverity::QuarantineRequired => "QUARANTINE_REQUIRED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceResponseClass {
    Allow,
    AllowWithWarning,
    Degrade,
    Block,
    Quarantine,
    SafeMode,
}

impl GovernanceResponseClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceResponseClass::Allow => "ALLOW",
            GovernanceResponseClass::AllowWithWarning => "ALLOW_WITH_WARNING",
            GovernanceResponseClass::Degrade => "DEGRADE",
            GovernanceResponseClass::Block => "BLOCK",
            GovernanceResponseClass::Quarantine => "QUARANTINE",
            GovernanceResponseClass::SafeMode => "SAFE_MODE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceDecisionOutcome {
    Passed,
    Failed,
    Degraded,
    Quarantined,
    SafeModeActive,
}

impl GovernanceDecisionOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceDecisionOutcome::Passed => "PASSED",
            GovernanceDecisionOutcome::Failed => "FAILED",
            GovernanceDecisionOutcome::Degraded => "DEGRADED",
            GovernanceDecisionOutcome::Quarantined => "QUARANTINED",
            GovernanceDecisionOutcome::SafeModeActive => "SAFE_MODE_ACTIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceCertificationStatus {
    Certified,
    Warning,
    Degraded,
    Quarantined,
    Uncertified,
}

impl GovernanceCertificationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceCertificationStatus::Certified => "CERTIFIED",
            GovernanceCertificationStatus::Warning => "WARNING",
            GovernanceCertificationStatus::Degraded => "DEGRADED",
            GovernanceCertificationStatus::Quarantined => "QUARANTINED",
            GovernanceCertificationStatus::Uncertified => "UNCERTIFIED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceClusterConsistency {
    Consistent,
    CompatibilityWindow,
    Diverged,
    Unknown,
}

impl GovernanceClusterConsistency {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceClusterConsistency::Consistent => "CONSISTENT",
            GovernanceClusterConsistency::CompatibilityWindow => "COMPATIBILITY_WINDOW",
            GovernanceClusterConsistency::Diverged => "DIVERGED",
            GovernanceClusterConsistency::Unknown => "UNKNOWN",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceDriftSignal {
    RepeatedArchitectureViolations,
    SubsystemCertificationRegression,
    PolicyVersionDrift,
    EnvelopeIntegrityDrift,
    PersistenceReplayViolation,
}

impl GovernanceDriftSignal {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceDriftSignal::RepeatedArchitectureViolations => {
                "REPEATED_ARCHITECTURE_VIOLATIONS"
            }
            GovernanceDriftSignal::SubsystemCertificationRegression => {
                "SUBSYSTEM_CERTIFICATION_REGRESSION"
            }
            GovernanceDriftSignal::PolicyVersionDrift => "POLICY_VERSION_DRIFT",
            GovernanceDriftSignal::EnvelopeIntegrityDrift => "ENVELOPE_INTEGRITY_DRIFT",
            GovernanceDriftSignal::PersistenceReplayViolation => "PERSISTENCE_REPLAY_VIOLATION",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GovernanceProtectedActionClass {
    VoiceTurnExecution,
    PersistenceReplay,
    PersistenceRecovery,
    PrimaryDeviceConfirmation,
    AccessProvision,
    ArtifactActivation,
}

impl GovernanceProtectedActionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            GovernanceProtectedActionClass::VoiceTurnExecution => "VOICE_TURN_EXECUTION",
            GovernanceProtectedActionClass::PersistenceReplay => "PERSISTENCE_REPLAY",
            GovernanceProtectedActionClass::PersistenceRecovery => "PERSISTENCE_RECOVERY",
            GovernanceProtectedActionClass::PrimaryDeviceConfirmation => {
                "PRIMARY_DEVICE_CONFIRMATION"
            }
            GovernanceProtectedActionClass::AccessProvision => "ACCESS_PROVISION",
            GovernanceProtectedActionClass::ArtifactActivation => "ARTIFACT_ACTIVATION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernanceRuleDescriptor {
    pub rule_id: String,
    pub category: GovernanceRuleCategory,
    pub owner_subsystem: String,
    pub rule_version: String,
    pub enabled: bool,
}

impl GovernanceRuleDescriptor {
    pub fn v1(
        rule_id: String,
        category: GovernanceRuleCategory,
        owner_subsystem: String,
        rule_version: String,
        enabled: bool,
    ) -> Result<Self, ContractViolation> {
        let descriptor = Self {
            rule_id,
            category,
            owner_subsystem,
            rule_version,
            enabled,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

impl Validate for GovernanceRuleDescriptor {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token("governance_rule_descriptor.rule_id", &self.rule_id, 64)?;
        validate_ascii_token(
            "governance_rule_descriptor.owner_subsystem",
            &self.owner_subsystem,
            64,
        )?;
        validate_ascii_token(
            "governance_rule_descriptor.rule_version",
            &self.rule_version,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernancePolicyWindow {
    pub governance_policy_version: String,
    pub compatibility_floor: String,
    pub compatibility_ceiling: String,
}

impl GovernancePolicyWindow {
    pub fn v1(
        governance_policy_version: String,
        compatibility_floor: String,
        compatibility_ceiling: String,
    ) -> Result<Self, ContractViolation> {
        let window = Self {
            governance_policy_version,
            compatibility_floor,
            compatibility_ceiling,
        };
        window.validate()?;
        Ok(window)
    }
}

impl Validate for GovernancePolicyWindow {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "governance_policy_window.governance_policy_version",
            &self.governance_policy_version,
            64,
        )?;
        validate_ascii_token(
            "governance_policy_window.compatibility_floor",
            &self.compatibility_floor,
            64,
        )?;
        validate_ascii_token(
            "governance_policy_window.compatibility_ceiling",
            &self.compatibility_ceiling,
            64,
        )?;
        if self.compatibility_floor > self.compatibility_ceiling {
            return Err(ContractViolation::InvalidValue {
                field: "governance_policy_window.compatibility_floor",
                reason: "must be <= compatibility_ceiling",
            });
        }
        if self.governance_policy_version < self.compatibility_floor
            || self.governance_policy_version > self.compatibility_ceiling
        {
            return Err(ContractViolation::InvalidValue {
                field: "governance_policy_window.governance_policy_version",
                reason: "must fall within compatibility window",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernanceSubsystemCertification {
    pub subsystem_id: String,
    pub status: GovernanceCertificationStatus,
}

impl GovernanceSubsystemCertification {
    pub fn v1(
        subsystem_id: String,
        status: GovernanceCertificationStatus,
    ) -> Result<Self, ContractViolation> {
        let certification = Self {
            subsystem_id,
            status,
        };
        certification.validate()?;
        Ok(certification)
    }
}

impl Validate for GovernanceSubsystemCertification {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "governance_subsystem_certification.subsystem_id",
            &self.subsystem_id,
            64,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernanceExecutionState {
    pub governance_policy_version: String,
    pub cluster_consistency: GovernanceClusterConsistency,
    pub safe_mode_active: bool,
    pub quarantined_subsystems: Vec<String>,
    pub subsystem_certifications: Vec<GovernanceSubsystemCertification>,
    pub drift_signals: Vec<GovernanceDriftSignal>,
    pub last_rule_id: Option<String>,
    pub last_severity: Option<GovernanceSeverity>,
    pub last_response_class: Option<GovernanceResponseClass>,
    pub decision_log_ref: Option<String>,
}

impl GovernanceExecutionState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        governance_policy_version: String,
        cluster_consistency: GovernanceClusterConsistency,
        safe_mode_active: bool,
        quarantined_subsystems: Vec<String>,
        subsystem_certifications: Vec<GovernanceSubsystemCertification>,
        drift_signals: Vec<GovernanceDriftSignal>,
        last_rule_id: Option<String>,
        last_severity: Option<GovernanceSeverity>,
        last_response_class: Option<GovernanceResponseClass>,
        decision_log_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            governance_policy_version,
            cluster_consistency,
            safe_mode_active,
            quarantined_subsystems,
            subsystem_certifications,
            drift_signals,
            last_rule_id,
            last_severity,
            last_response_class,
            decision_log_ref,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for GovernanceExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "governance_execution_state.governance_policy_version",
            &self.governance_policy_version,
            64,
        )?;
        if self.quarantined_subsystems.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "governance_execution_state.quarantined_subsystems",
                reason: "must contain <= 16 entries",
            });
        }
        for subsystem in &self.quarantined_subsystems {
            validate_ascii_token(
                "governance_execution_state.quarantined_subsystems",
                subsystem,
                64,
            )?;
        }
        if self.subsystem_certifications.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "governance_execution_state.subsystem_certifications",
                reason: "must contain <= 16 entries",
            });
        }
        for certification in &self.subsystem_certifications {
            certification.validate()?;
        }
        if self.drift_signals.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "governance_execution_state.drift_signals",
                reason: "must contain <= 16 entries",
            });
        }
        validate_optional_ascii_token(
            "governance_execution_state.last_rule_id",
            &self.last_rule_id,
            64,
        )?;
        validate_optional_ascii_token(
            "governance_execution_state.decision_log_ref",
            &self.decision_log_ref,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct GovernanceDecisionLogEntry {
    pub sequence: u64,
    pub rule_id: String,
    pub subsystem_id: String,
    pub governance_policy_version: String,
    pub outcome: GovernanceDecisionOutcome,
    pub severity: GovernanceSeverity,
    pub response_class: GovernanceResponseClass,
    pub reason_code: String,
    pub session_id: Option<u128>,
    pub turn_id: Option<u64>,
    pub runtime_node_id: String,
    pub note: Option<String>,
}

impl GovernanceDecisionLogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        sequence: u64,
        rule_id: String,
        subsystem_id: String,
        governance_policy_version: String,
        outcome: GovernanceDecisionOutcome,
        severity: GovernanceSeverity,
        response_class: GovernanceResponseClass,
        reason_code: String,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        runtime_node_id: String,
        note: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let entry = Self {
            sequence,
            rule_id,
            subsystem_id,
            governance_policy_version,
            outcome,
            severity,
            response_class,
            reason_code,
            session_id,
            turn_id,
            runtime_node_id,
            note,
        };
        entry.validate()?;
        Ok(entry)
    }
}

impl Validate for GovernanceDecisionLogEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.sequence == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "governance_decision_log_entry.sequence",
                reason: "must be > 0",
            });
        }
        validate_ascii_token("governance_decision_log_entry.rule_id", &self.rule_id, 64)?;
        validate_ascii_token(
            "governance_decision_log_entry.subsystem_id",
            &self.subsystem_id,
            64,
        )?;
        validate_ascii_token(
            "governance_decision_log_entry.governance_policy_version",
            &self.governance_policy_version,
            64,
        )?;
        validate_ascii_token(
            "governance_decision_log_entry.reason_code",
            &self.reason_code,
            96,
        )?;
        validate_ascii_token(
            "governance_decision_log_entry.runtime_node_id",
            &self.runtime_node_id,
            128,
        )?;
        if matches!(self.session_id, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "governance_decision_log_entry.session_id",
                reason: "must be > 0 when present",
            });
        }
        if matches!(self.turn_id, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "governance_decision_log_entry.turn_id",
                reason: "must be > 0 when present",
            });
        }
        validate_optional_ascii_token("governance_decision_log_entry.note", &self.note, 256)?;
        Ok(())
    }
}
