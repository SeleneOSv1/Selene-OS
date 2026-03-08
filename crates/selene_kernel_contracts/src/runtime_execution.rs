#![forbid(unsafe_code)]

use crate::ph1_voice_id::{IdentityTierV2, SpoofLivenessStatus, UserId};
use crate::ph1d::PolicyContextRef;
use crate::ph1j::{
    DeviceId, ProofChainStatus, ProofFailureClass, ProofVerificationPosture, ProofWriteOutcome,
    TimestampTrustPosture, TurnId,
};
use crate::ph1l::SessionId;
use crate::ph1link::AppPlatform;
use crate::ph1m::MemoryConfidence;
use crate::runtime_governance::GovernanceExecutionState;
use crate::{ContractViolation, Validate};
use std::collections::BTreeSet;

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

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceClass {
    Phone,
    Tablet,
    Desktop,
}

impl DeviceClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            DeviceClass::Phone => "PHONE",
            DeviceClass::Tablet => "TABLET",
            DeviceClass::Desktop => "DESKTOP",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NetworkProfile {
    Unknown,
    Offline,
    Degraded,
    Standard,
    HighThroughput,
}

impl NetworkProfile {
    pub const fn as_str(self) -> &'static str {
        match self {
            NetworkProfile::Unknown => "UNKNOWN",
            NetworkProfile::Offline => "OFFLINE",
            NetworkProfile::Degraded => "DEGRADED",
            NetworkProfile::Standard => "STANDARD",
            NetworkProfile::HighThroughput => "HIGH_THROUGHPUT",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceCapability {
    Microphone,
    Camera,
    SpeakerOutput,
    FileSystemAccess,
    SensorAvailability,
    HardwareAcceleration,
    WakeWord,
}

impl DeviceCapability {
    pub const fn as_str(self) -> &'static str {
        match self {
            DeviceCapability::Microphone => "MICROPHONE",
            DeviceCapability::Camera => "CAMERA",
            DeviceCapability::SpeakerOutput => "SPEAKER_OUTPUT",
            DeviceCapability::FileSystemAccess => "FILE_SYSTEM_ACCESS",
            DeviceCapability::SensorAvailability => "SENSOR_AVAILABILITY",
            DeviceCapability::HardwareAcceleration => "HARDWARE_ACCELERATION",
            DeviceCapability::WakeWord => "WAKE_WORD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeviceTrustClass {
    TrustedDevice,
    StandardDevice,
    RestrictedDevice,
    UntrustedDevice,
}

impl DeviceTrustClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            DeviceTrustClass::TrustedDevice => "TRUSTED_DEVICE",
            DeviceTrustClass::StandardDevice => "STANDARD_DEVICE",
            DeviceTrustClass::RestrictedDevice => "RESTRICTED_DEVICE",
            DeviceTrustClass::UntrustedDevice => "UNTRUSTED_DEVICE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientIntegrityStatus {
    Unknown,
    IntegrityUnavailable,
    IntegrityVerified,
    Attested,
    IntegrityFailed,
}

impl ClientIntegrityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            ClientIntegrityStatus::Unknown => "UNKNOWN",
            ClientIntegrityStatus::IntegrityUnavailable => "INTEGRITY_UNAVAILABLE",
            ClientIntegrityStatus::IntegrityVerified => "INTEGRITY_VERIFIED",
            ClientIntegrityStatus::Attested => "ATTESTED",
            ClientIntegrityStatus::IntegrityFailed => "INTEGRITY_FAILED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientCompatibilityStatus {
    Unknown,
    Compatible,
    UpgradeRequired,
    UnsupportedClient,
}

impl ClientCompatibilityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            ClientCompatibilityStatus::Unknown => "UNKNOWN",
            ClientCompatibilityStatus::Compatible => "COMPATIBLE",
            ClientCompatibilityStatus::UpgradeRequired => "UPGRADE_REQUIRED",
            ClientCompatibilityStatus::UnsupportedClient => "UNSUPPORTED_CLIENT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuntimeEntryTrigger {
    Explicit,
    WakeWord,
}

impl RuntimeEntryTrigger {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeEntryTrigger::Explicit => "EXPLICIT",
            RuntimeEntryTrigger::WakeWord => "WAKE_WORD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PlatformTriggerPolicy {
    ExplicitOnly,
    WakeOrExplicit,
}

impl PlatformTriggerPolicy {
    pub const fn as_str(self) -> &'static str {
        match self {
            PlatformTriggerPolicy::ExplicitOnly => "EXPLICIT_ONLY",
            PlatformTriggerPolicy::WakeOrExplicit => "WAKE_OR_EXPLICIT",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PlatformRuntimeContext {
    pub platform_type: AppPlatform,
    pub platform_version: String,
    pub device_class: DeviceClass,
    pub runtime_client_version: String,
    pub hardware_capability_profile: String,
    pub network_profile: NetworkProfile,
    pub claimed_capabilities: Vec<DeviceCapability>,
    pub negotiated_capabilities: Vec<DeviceCapability>,
    pub device_trust_class: DeviceTrustClass,
    pub integrity_status: ClientIntegrityStatus,
    pub compatibility_status: ClientCompatibilityStatus,
    pub minimum_supported_client_version: Option<String>,
    pub attestation_ref: Option<String>,
    pub requested_trigger: RuntimeEntryTrigger,
    pub trigger_policy: PlatformTriggerPolicy,
    pub trigger_allowed: bool,
}

impl PlatformRuntimeContext {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        platform_type: AppPlatform,
        platform_version: String,
        device_class: DeviceClass,
        runtime_client_version: String,
        hardware_capability_profile: String,
        network_profile: NetworkProfile,
        claimed_capabilities: Vec<DeviceCapability>,
        negotiated_capabilities: Vec<DeviceCapability>,
        device_trust_class: DeviceTrustClass,
        integrity_status: ClientIntegrityStatus,
        compatibility_status: ClientCompatibilityStatus,
        minimum_supported_client_version: Option<String>,
        attestation_ref: Option<String>,
        requested_trigger: RuntimeEntryTrigger,
        trigger_policy: PlatformTriggerPolicy,
        trigger_allowed: bool,
    ) -> Result<Self, ContractViolation> {
        let context = Self {
            platform_type,
            platform_version,
            device_class,
            runtime_client_version,
            hardware_capability_profile,
            network_profile,
            claimed_capabilities,
            negotiated_capabilities,
            device_trust_class,
            integrity_status,
            compatibility_status,
            minimum_supported_client_version,
            attestation_ref,
            requested_trigger,
            trigger_policy,
            trigger_allowed,
        };
        context.validate()?;
        Ok(context)
    }

    pub fn default_for_platform(platform_type: AppPlatform) -> Result<Self, ContractViolation> {
        Self::default_for_platform_and_trigger(platform_type, RuntimeEntryTrigger::Explicit)
    }

    pub fn default_for_platform_and_trigger(
        platform_type: AppPlatform,
        requested_trigger: RuntimeEntryTrigger,
    ) -> Result<Self, ContractViolation> {
        let device_class = default_device_class_for_platform(platform_type);
        let negotiated_capabilities = supported_capabilities_for_platform(platform_type);
        Self::v1(
            platform_type,
            "UNKNOWN".to_string(),
            device_class,
            "UNKNOWN".to_string(),
            default_hardware_capability_profile(platform_type).to_string(),
            NetworkProfile::Unknown,
            negotiated_capabilities.clone(),
            negotiated_capabilities,
            DeviceTrustClass::StandardDevice,
            ClientIntegrityStatus::Unknown,
            ClientCompatibilityStatus::Unknown,
            None,
            None,
            requested_trigger,
            default_trigger_policy_for_platform(platform_type),
            trigger_allowed_for_platform(platform_type, requested_trigger),
        )
    }
}

impl Validate for PlatformRuntimeContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.platform_type.validate()?;
        validate_ascii_token(
            "platform_runtime_context.platform_version",
            &self.platform_version,
            64,
        )?;
        validate_ascii_token(
            "platform_runtime_context.runtime_client_version",
            &self.runtime_client_version,
            64,
        )?;
        validate_ascii_token(
            "platform_runtime_context.hardware_capability_profile",
            &self.hardware_capability_profile,
            64,
        )?;
        validate_optional_ascii_token(
            "platform_runtime_context.minimum_supported_client_version",
            &self.minimum_supported_client_version,
            64,
        )?;
        validate_optional_ascii_token(
            "platform_runtime_context.attestation_ref",
            &self.attestation_ref,
            128,
        )?;

        let expected_device_class = default_device_class_for_platform(self.platform_type);
        if self.device_class != expected_device_class {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.device_class",
                reason: "must match canonical platform device class",
            });
        }

        validate_capability_list(
            "platform_runtime_context.claimed_capabilities",
            &self.claimed_capabilities,
        )?;
        validate_capability_list(
            "platform_runtime_context.negotiated_capabilities",
            &self.negotiated_capabilities,
        )?;

        let supported: BTreeSet<DeviceCapability> =
            supported_capabilities_for_platform(self.platform_type)
                .into_iter()
                .collect();
        for capability in &self.claimed_capabilities {
            if !supported.contains(capability) {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.claimed_capabilities",
                    reason: "contains capability unsupported by platform",
                });
            }
        }
        for capability in &self.negotiated_capabilities {
            if !supported.contains(capability) {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.negotiated_capabilities",
                    reason: "contains capability unsupported by platform",
                });
            }
        }
        if !self
            .negotiated_capabilities
            .iter()
            .all(|capability| supported.contains(capability))
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.negotiated_capabilities",
                reason: "must be a subset of supported platform capabilities",
            });
        }
        if !self
            .claimed_capabilities
            .iter()
            .all(|capability| self.negotiated_capabilities.contains(capability))
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.claimed_capabilities",
                reason: "claimed capabilities must be present in negotiated capabilities",
            });
        }

        let expected_policy = default_trigger_policy_for_platform(self.platform_type);
        if self.trigger_policy != expected_policy {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.trigger_policy",
                reason: "must match canonical platform trigger policy",
            });
        }

        let expected_trigger_allowed =
            trigger_allowed_for_platform(self.platform_type, self.requested_trigger);
        let wake_capability_present = self
            .negotiated_capabilities
            .contains(&DeviceCapability::WakeWord);
        if self.requested_trigger == RuntimeEntryTrigger::WakeWord
            && expected_trigger_allowed
            && !wake_capability_present
            && self.trigger_allowed
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.trigger_allowed",
                reason: "wake trigger requires negotiated WAKE_WORD capability",
            });
        }
        if self.trigger_allowed
            != (expected_trigger_allowed
                && (self.requested_trigger != RuntimeEntryTrigger::WakeWord
                    || wake_capability_present))
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.trigger_allowed",
                reason: "must match canonical platform trigger governance decision",
            });
        }

        Ok(())
    }
}

pub fn default_device_class_for_platform(platform_type: AppPlatform) -> DeviceClass {
    match platform_type {
        AppPlatform::Ios | AppPlatform::Android => DeviceClass::Phone,
        AppPlatform::Tablet => DeviceClass::Tablet,
        AppPlatform::Desktop => DeviceClass::Desktop,
    }
}

pub fn default_trigger_policy_for_platform(platform_type: AppPlatform) -> PlatformTriggerPolicy {
    match platform_type {
        AppPlatform::Ios => PlatformTriggerPolicy::ExplicitOnly,
        AppPlatform::Android | AppPlatform::Tablet | AppPlatform::Desktop => {
            PlatformTriggerPolicy::WakeOrExplicit
        }
    }
}

pub fn trigger_allowed_for_platform(
    platform_type: AppPlatform,
    requested_trigger: RuntimeEntryTrigger,
) -> bool {
    match default_trigger_policy_for_platform(platform_type) {
        PlatformTriggerPolicy::ExplicitOnly => requested_trigger == RuntimeEntryTrigger::Explicit,
        PlatformTriggerPolicy::WakeOrExplicit => true,
    }
}

pub fn default_hardware_capability_profile(platform_type: AppPlatform) -> &'static str {
    match platform_type {
        AppPlatform::Ios | AppPlatform::Android => "PHONE_STANDARD",
        AppPlatform::Tablet => "TABLET_STANDARD",
        AppPlatform::Desktop => "DESKTOP_STANDARD",
    }
}

pub fn supported_capabilities_for_platform(platform_type: AppPlatform) -> Vec<DeviceCapability> {
    let mut capabilities = vec![
        DeviceCapability::Microphone,
        DeviceCapability::Camera,
        DeviceCapability::SpeakerOutput,
        DeviceCapability::FileSystemAccess,
        DeviceCapability::HardwareAcceleration,
    ];
    match platform_type {
        AppPlatform::Ios => {}
        AppPlatform::Android | AppPlatform::Tablet => {
            capabilities.push(DeviceCapability::SensorAvailability);
            capabilities.push(DeviceCapability::WakeWord);
        }
        AppPlatform::Desktop => {
            capabilities.push(DeviceCapability::WakeWord);
        }
    }
    capabilities
}

fn validate_capability_list(
    field: &'static str,
    values: &[DeviceCapability],
) -> Result<(), ContractViolation> {
    if values.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain <= 16 capabilities",
        });
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(*value) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain duplicates",
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdmissionState {
    IngressValidated,
    SessionResolved,
    ExecutionAdmitted,
    Rejected,
}

impl AdmissionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            AdmissionState::IngressValidated => "INGRESS_VALIDATED",
            AdmissionState::SessionResolved => "SESSION_RESOLVED",
            AdmissionState::ExecutionAdmitted => "EXECUTION_ADMITTED",
            AdmissionState::Rejected => "REJECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FailureClass {
    AuthenticationFailure,
    AuthorizationFailure,
    InvalidPayload,
    ReplayRejected,
    SessionConflict,
    PolicyViolation,
    ExecutionFailure,
    RetryableRuntime,
}

impl FailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            FailureClass::AuthenticationFailure => "AUTHENTICATION_FAILURE",
            FailureClass::AuthorizationFailure => "AUTHORIZATION_FAILURE",
            FailureClass::InvalidPayload => "INVALID_PAYLOAD",
            FailureClass::ReplayRejected => "REPLAY_REJECTED",
            FailureClass::SessionConflict => "SESSION_CONFLICT",
            FailureClass::PolicyViolation => "POLICY_VIOLATION",
            FailureClass::ExecutionFailure => "EXECUTION_FAILURE",
            FailureClass::RetryableRuntime => "RETRYABLE_RUNTIME",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SessionAttachOutcome {
    NewSessionCreated,
    ExistingSessionReused,
    ExistingSessionAttached,
    RetryReusedResult,
}

impl SessionAttachOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            SessionAttachOutcome::NewSessionCreated => "NEW_SESSION_CREATED",
            SessionAttachOutcome::ExistingSessionReused => "EXISTING_SESSION_REUSED",
            SessionAttachOutcome::ExistingSessionAttached => "EXISTING_SESSION_ATTACHED",
            SessionAttachOutcome::RetryReusedResult => "RETRY_REUSED_RESULT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceRecoveryMode {
    Normal,
    Recovering,
    DegradedRecovery,
    QuarantinedLocalState,
}

impl PersistenceRecoveryMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            PersistenceRecoveryMode::Normal => "NORMAL",
            PersistenceRecoveryMode::Recovering => "RECOVERING",
            PersistenceRecoveryMode::DegradedRecovery => "DEGRADED_RECOVERY",
            PersistenceRecoveryMode::QuarantinedLocalState => "QUARANTINED_LOCAL_STATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceAcknowledgementState {
    PendingCloudAcknowledgement,
    AuthoritativelyAcknowledged,
    StaleRejected,
    QuarantinedLocalState,
}

impl PersistenceAcknowledgementState {
    pub const fn as_str(self) -> &'static str {
        match self {
            PersistenceAcknowledgementState::PendingCloudAcknowledgement => {
                "PENDING_CLOUD_ACKNOWLEDGEMENT"
            }
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged => {
                "AUTHORITATIVELY_ACKNOWLEDGED"
            }
            PersistenceAcknowledgementState::StaleRejected => "STALE_REJECTED",
            PersistenceAcknowledgementState::QuarantinedLocalState => "QUARANTINED_LOCAL_STATE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceConflictSeverity {
    Info,
    Retryable,
    StaleRejected,
    QuarantineRequired,
}

impl PersistenceConflictSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            PersistenceConflictSeverity::Info => "INFO",
            PersistenceConflictSeverity::Retryable => "RETRYABLE",
            PersistenceConflictSeverity::StaleRejected => "STALE_REJECTED",
            PersistenceConflictSeverity::QuarantineRequired => "QUARANTINE_REQUIRED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReconciliationDecision {
    RetrySameOperation,
    ReusePriorAuthoritativeOutcome,
    RejectStaleOperation,
    RequestFreshSessionState,
    QuarantineLocalState,
}

impl ReconciliationDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            ReconciliationDecision::RetrySameOperation => "RETRY_SAME_OPERATION",
            ReconciliationDecision::ReusePriorAuthoritativeOutcome => {
                "REUSE_PRIOR_AUTHORITATIVE_OUTCOME"
            }
            ReconciliationDecision::RejectStaleOperation => "REJECT_STALE_OPERATION",
            ReconciliationDecision::RequestFreshSessionState => "REQUEST_FRESH_SESSION_STATE",
            ReconciliationDecision::QuarantineLocalState => "QUARANTINE_LOCAL_STATE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PersistenceExecutionState {
    pub recovery_mode: PersistenceRecoveryMode,
    pub acknowledgement_state: PersistenceAcknowledgementState,
    pub reconciliation_decision: Option<ReconciliationDecision>,
    pub conflict_severity: Option<PersistenceConflictSeverity>,
    pub cross_node_dedupe_applied: bool,
    pub audit_ref: Option<String>,
}

impl PersistenceExecutionState {
    pub fn v1(
        recovery_mode: PersistenceRecoveryMode,
        acknowledgement_state: PersistenceAcknowledgementState,
        reconciliation_decision: Option<ReconciliationDecision>,
        conflict_severity: Option<PersistenceConflictSeverity>,
        cross_node_dedupe_applied: bool,
        audit_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            recovery_mode,
            acknowledgement_state,
            reconciliation_decision,
            conflict_severity,
            cross_node_dedupe_applied,
            audit_ref,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for PersistenceExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_ascii_token(
            "persistence_execution_state.audit_ref",
            &self.audit_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityVerificationConsistencyLevel {
    StrictVerified,
    HighConfidenceVerified,
    DegradedVerification,
    RecoveryRestricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityTrustTier {
    Verified,
    HighConfidence,
    Conditional,
    Restricted,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityRecoveryState {
    None,
    ReauthRequired,
    ReEnrollmentRequired,
    RecoveryRestricted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityExecutionState {
    pub consistency_level: IdentityVerificationConsistencyLevel,
    pub trust_tier: IdentityTrustTier,
    pub identity_tier_v2: IdentityTierV2,
    pub spoof_liveness_status: SpoofLivenessStatus,
    pub step_up_required: bool,
    pub recovery_state: IdentityRecoveryState,
    pub cluster_drift_detected: bool,
    pub reason_code: Option<u64>,
}

impl IdentityExecutionState {
    pub fn v1(
        consistency_level: IdentityVerificationConsistencyLevel,
        trust_tier: IdentityTrustTier,
        identity_tier_v2: IdentityTierV2,
        spoof_liveness_status: SpoofLivenessStatus,
        step_up_required: bool,
        recovery_state: IdentityRecoveryState,
        cluster_drift_detected: bool,
        reason_code: Option<u64>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            consistency_level,
            trust_tier,
            identity_tier_v2,
            spoof_liveness_status,
            step_up_required,
            recovery_state,
            cluster_drift_detected,
            reason_code,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for IdentityExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryConsistencyLevel {
    StrictLedger,
    EventualView,
    RecoveryRebuild,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTrustLevel {
    Verified,
    HighConfidence,
    LowConfidence,
    Unverified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryEligibilityDecision {
    Eligible,
    IdentityScopeBlocked,
    PolicyBlocked,
    GovernedBlocked,
    NoEligibleCandidates,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryExecutionState {
    pub cloud_authoritative: bool,
    pub consistency_level: MemoryConsistencyLevel,
    pub trust_level: MemoryTrustLevel,
    pub eligibility_decision: MemoryEligibilityDecision,
    pub confidence_floor: Option<MemoryConfidence>,
    pub candidate_count: u16,
    pub ledger_write_accepted: bool,
    pub ledger_event_count: u16,
    pub governance_blocked: bool,
    pub reason_code: Option<u64>,
}

impl MemoryExecutionState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        cloud_authoritative: bool,
        consistency_level: MemoryConsistencyLevel,
        trust_level: MemoryTrustLevel,
        eligibility_decision: MemoryEligibilityDecision,
        confidence_floor: Option<MemoryConfidence>,
        candidate_count: u16,
        ledger_write_accepted: bool,
        ledger_event_count: u16,
        governance_blocked: bool,
        reason_code: Option<u64>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            cloud_authoritative,
            consistency_level,
            trust_level,
            eligibility_decision,
            confidence_floor,
            candidate_count,
            ledger_write_accepted,
            ledger_event_count,
            governance_blocked,
            reason_code,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for MemoryExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SimulationCertificationState {
    NotRequested,
    CertifiedActive,
    MissingSimulationPath,
    InactiveSimulation,
    StepUpRequired,
}

impl SimulationCertificationState {
    pub const fn as_str(self) -> &'static str {
        match self {
            SimulationCertificationState::NotRequested => "NOT_REQUESTED",
            SimulationCertificationState::CertifiedActive => "CERTIFIED_ACTIVE",
            SimulationCertificationState::MissingSimulationPath => "MISSING_SIMULATION_PATH",
            SimulationCertificationState::InactiveSimulation => "INACTIVE_SIMULATION",
            SimulationCertificationState::StepUpRequired => "STEP_UP_REQUIRED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OnboardingReadinessState {
    NotApplicable,
    Ready,
    Incomplete,
    Blocked,
}

impl OnboardingReadinessState {
    pub const fn as_str(self) -> &'static str {
        match self {
            OnboardingReadinessState::NotApplicable => "NOT_APPLICABLE",
            OnboardingReadinessState::Ready => "READY",
            OnboardingReadinessState::Incomplete => "INCOMPLETE",
            OnboardingReadinessState::Blocked => "BLOCKED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuthorityPolicyDecision {
    NotRequested,
    Allowed,
    Denied,
    StepUpRequired,
    PendingConfirmation,
}

impl AuthorityPolicyDecision {
    pub const fn as_str(self) -> &'static str {
        match self {
            AuthorityPolicyDecision::NotRequested => "NOT_REQUESTED",
            AuthorityPolicyDecision::Allowed => "ALLOWED",
            AuthorityPolicyDecision::Denied => "DENIED",
            AuthorityPolicyDecision::StepUpRequired => "STEP_UP_REQUIRED",
            AuthorityPolicyDecision::PendingConfirmation => "PENDING_CONFIRMATION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityExecutionState {
    pub policy_context_ref: Option<PolicyContextRef>,
    pub simulation_certification_state: SimulationCertificationState,
    pub onboarding_readiness_state: OnboardingReadinessState,
    pub policy_decision: AuthorityPolicyDecision,
    pub identity_scope_required: bool,
    pub identity_scope_satisfied: bool,
    pub memory_scope_allowed: bool,
    pub reason_code: Option<u64>,
}

impl AuthorityExecutionState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        policy_context_ref: Option<PolicyContextRef>,
        simulation_certification_state: SimulationCertificationState,
        onboarding_readiness_state: OnboardingReadinessState,
        policy_decision: AuthorityPolicyDecision,
        identity_scope_required: bool,
        identity_scope_satisfied: bool,
        memory_scope_allowed: bool,
        reason_code: Option<u64>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            policy_context_ref,
            simulation_certification_state,
            onboarding_readiness_state,
            policy_decision,
            identity_scope_required,
            identity_scope_satisfied,
            memory_scope_allowed,
            reason_code,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for AuthorityExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(policy_context_ref) = self.policy_context_ref.as_ref() {
            policy_context_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofExecutionState {
    pub proof_record_ref: Option<String>,
    pub proof_write_outcome: ProofWriteOutcome,
    pub proof_failure_class: Option<ProofFailureClass>,
    pub proof_chain_status: ProofChainStatus,
    pub proof_verification_posture: ProofVerificationPosture,
    pub timestamp_trust_posture: TimestampTrustPosture,
    pub verifier_metadata_ref: Option<String>,
}

impl ProofExecutionState {
    pub fn v1(
        proof_record_ref: Option<String>,
        proof_write_outcome: ProofWriteOutcome,
        proof_failure_class: Option<ProofFailureClass>,
        proof_chain_status: ProofChainStatus,
        proof_verification_posture: ProofVerificationPosture,
        timestamp_trust_posture: TimestampTrustPosture,
        verifier_metadata_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            proof_record_ref,
            proof_write_outcome,
            proof_failure_class,
            proof_chain_status,
            proof_verification_posture,
            timestamp_trust_posture,
            verifier_metadata_ref,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for ProofExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_ascii_token(
            "proof_execution_state.proof_record_ref",
            &self.proof_record_ref,
            128,
        )?;
        validate_optional_ascii_token(
            "proof_execution_state.verifier_metadata_ref",
            &self.verifier_metadata_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeExecutionEnvelope {
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub actor_identity: UserId,
    pub device_identity: DeviceId,
    pub platform: AppPlatform,
    pub platform_context: PlatformRuntimeContext,
    pub session_id: Option<SessionId>,
    pub turn_id: TurnId,
    pub device_turn_sequence: Option<u64>,
    pub admission_state: AdmissionState,
    pub session_attach_outcome: Option<SessionAttachOutcome>,
    pub persistence_state: Option<PersistenceExecutionState>,
    pub governance_state: Option<GovernanceExecutionState>,
    pub proof_state: Option<ProofExecutionState>,
    pub identity_state: Option<IdentityExecutionState>,
    pub memory_state: Option<MemoryExecutionState>,
    pub authority_state: Option<AuthorityExecutionState>,
}

impl RuntimeExecutionEnvelope {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            PlatformRuntimeContext::default_for_platform(platform)?,
            session_id,
            turn_id,
            None,
            admission_state,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_device_turn_sequence(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            PlatformRuntimeContext::default_for_platform(platform)?,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_device_turn_sequence_and_attach_outcome(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            PlatformRuntimeContext::default_for_platform(platform)?,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_platform_context_device_turn_sequence_and_attach_outcome(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        platform_context: PlatformRuntimeContext,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            platform_context,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
            None,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_platform_context_device_turn_sequence_attach_outcome_and_persistence_state(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        platform_context: PlatformRuntimeContext,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
        session_attach_outcome: Option<SessionAttachOutcome>,
        persistence_state: Option<PersistenceExecutionState>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state(
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            platform_context,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
            persistence_state,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_platform_context_device_turn_sequence_attach_outcome_persistence_and_governance_state(
        request_id: String,
        trace_id: String,
        idempotency_key: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform: AppPlatform,
        platform_context: PlatformRuntimeContext,
        session_id: Option<SessionId>,
        turn_id: TurnId,
        device_turn_sequence: Option<u64>,
        admission_state: AdmissionState,
        session_attach_outcome: Option<SessionAttachOutcome>,
        persistence_state: Option<PersistenceExecutionState>,
        governance_state: Option<GovernanceExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            request_id,
            trace_id,
            idempotency_key,
            actor_identity,
            device_identity,
            platform,
            platform_context,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
            persistence_state,
            governance_state,
            proof_state: None,
            identity_state: None,
            memory_state: None,
            authority_state: None,
        };
        envelope.validate()?;
        Ok(envelope)
    }

    pub fn with_admission_state(
        &self,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.admission_state = admission_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_session_and_admission_state(
        &self,
        session_id: Option<SessionId>,
        admission_state: AdmissionState,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.session_id = session_id;
        next.admission_state = admission_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_session_device_turn_and_attach_outcome(
        &self,
        session_id: Option<SessionId>,
        admission_state: AdmissionState,
        device_turn_sequence: Option<u64>,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.session_id = session_id;
        next.admission_state = admission_state;
        next.device_turn_sequence = device_turn_sequence;
        next.session_attach_outcome = session_attach_outcome;
        next.validate()?;
        Ok(next)
    }

    pub fn with_attach_outcome(
        &self,
        session_attach_outcome: Option<SessionAttachOutcome>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.session_attach_outcome = session_attach_outcome;
        next.validate()?;
        Ok(next)
    }

    pub fn with_persistence_state(
        &self,
        persistence_state: Option<PersistenceExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.persistence_state = persistence_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_governance_state(
        &self,
        governance_state: Option<GovernanceExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.governance_state = governance_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_identity_state(
        &self,
        identity_state: Option<IdentityExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.identity_state = identity_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_proof_state(
        &self,
        proof_state: Option<ProofExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.proof_state = proof_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_memory_state(
        &self,
        memory_state: Option<MemoryExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.memory_state = memory_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_authority_state(
        &self,
        authority_state: Option<AuthorityExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.authority_state = authority_state;
        next.validate()?;
        Ok(next)
    }
}

impl Validate for RuntimeExecutionEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "runtime_execution_envelope.request_id",
            &self.request_id,
            256,
        )?;
        validate_ascii_token("runtime_execution_envelope.trace_id", &self.trace_id, 256)?;
        validate_ascii_token(
            "runtime_execution_envelope.idempotency_key",
            &self.idempotency_key,
            256,
        )?;
        if self.actor_identity.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.actor_identity",
                reason: "must not be empty",
            });
        }
        if self.actor_identity.as_str().len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.actor_identity",
                reason: "must be <= 128 chars",
            });
        }
        self.device_identity.validate()?;
        self.platform.validate()?;
        self.platform_context.validate()?;
        if self.platform_context.platform_type != self.platform {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.platform_context.platform_type",
                reason: "must match runtime_execution_envelope.platform",
            });
        }
        self.turn_id.validate()?;
        if matches!(self.session_id, Some(SessionId(0))) {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.session_id",
                reason: "must be > 0 when provided",
            });
        }
        if matches!(self.device_turn_sequence, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_execution_envelope.device_turn_sequence",
                reason: "must be > 0 when provided",
            });
        }
        if let Some(state) = self.persistence_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.governance_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.proof_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.identity_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.memory_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.authority_state.as_ref() {
            state.validate()?;
        }
        Ok(())
    }
}
