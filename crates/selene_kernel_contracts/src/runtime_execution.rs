#![forbid(unsafe_code)]

use crate::ph1_voice_id::{IdentityTierV2, Ph1VoiceIdResponse, SpoofLivenessStatus, UserId};
use crate::ph1art::{
    ArtifactIdentityRef, ArtifactTrustExecutionState, TrustPolicySnapshotRef, TrustSetSnapshotRef,
};
use crate::ph1comp::ComputationExecutionState;
use crate::ph1d::PolicyContextRef;
use crate::ph1j::{
    DeviceId, ProofChainStatus, ProofFailureClass, ProofVerificationPosture, ProofWriteOutcome,
    TimestampTrustPosture, TurnId,
};
use crate::ph1l::SessionId;
use crate::ph1link::AppPlatform;
use crate::ph1m::MemoryConfidence;
use crate::runtime_governance::{
    GovernanceClusterConsistency, GovernanceDriftSignal, GovernanceExecutionState,
};
use crate::runtime_law::{RuntimeLawExecutionState, RuntimeLawResponseClass};
use crate::{ContractViolation, Validate};
use std::collections::BTreeSet;

const SUBSYSTEM_RUNTIME_GOVERNANCE: &str = "RUNTIME_GOVERNANCE";

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

fn validate_unique_ascii_tokens(
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
        validate_ascii_token(field, value, max_len)?;
        if !seen.insert(value.clone()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "contains duplicate value",
            });
        }
    }
    Ok(())
}

fn governance_cluster_drift_detected(governance_state: &GovernanceExecutionState) -> bool {
    governance_state.cluster_consistency != GovernanceClusterConsistency::Consistent
}

fn governance_policy_version_drift_detected(governance_state: &GovernanceExecutionState) -> bool {
    governance_state
        .drift_signals
        .contains(&GovernanceDriftSignal::PolicyVersionDrift)
}

fn law_state_mentions_runtime_governance(law_state: &RuntimeLawExecutionState) -> bool {
    law_state
        .subsystem_inputs
        .iter()
        .any(|candidate| candidate == SUBSYSTEM_RUNTIME_GOVERNANCE)
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
    /// Non-authoritative upstream capture posture only.
    /// This must never be treated as a Section 04 artifact trust decision.
    pub capture_artifact_trust_verified: bool,
    /// Capture posture observation metadata paired with the non-authoritative capture posture.
    pub capture_artifact_observed_at_ns: Option<u64>,
    /// Capture posture retention metadata paired with the non-authoritative capture posture.
    pub capture_artifact_retention_deadline_ns: Option<u64>,
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
            capture_artifact_trust_verified: false,
            capture_artifact_observed_at_ns: None,
            capture_artifact_retention_deadline_ns: None,
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
        if self.integrity_status == ClientIntegrityStatus::Attested
            && self.attestation_ref.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.attestation_ref",
                reason: "must be present when integrity_status=ATTESTED",
            });
        }
        if self.attestation_ref.is_some()
            && self.integrity_status != ClientIntegrityStatus::Attested
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.integrity_status",
                reason: "must be ATTESTED when attestation_ref is present",
            });
        }
        if self.capture_artifact_observed_at_ns.is_some()
            != self.capture_artifact_retention_deadline_ns.is_some()
        {
            return Err(ContractViolation::InvalidValue {
                field: "platform_runtime_context.capture_artifact_retention_deadline_ns",
                reason: "must be present whenever capture_artifact_observed_at_ns is present",
            });
        }
        if let (Some(observed_at_ns), Some(retention_deadline_ns)) = (
            self.capture_artifact_observed_at_ns,
            self.capture_artifact_retention_deadline_ns,
        ) {
            if self.attestation_ref.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.attestation_ref",
                    reason: "must be present when capture artifact lifecycle metadata is recorded",
                });
            }
            if observed_at_ns > retention_deadline_ns {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.capture_artifact_retention_deadline_ns",
                    reason: "must be >= capture_artifact_observed_at_ns",
                });
            }
        }
        if self.capture_artifact_trust_verified {
            if self.integrity_status != ClientIntegrityStatus::Attested {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.integrity_status",
                    reason: "must be ATTESTED when capture_artifact_trust_verified=true",
                });
            }
            if self.attestation_ref.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.attestation_ref",
                    reason: "must be present when capture_artifact_trust_verified=true",
                });
            }
            if self.capture_artifact_observed_at_ns.is_none()
                || self.capture_artifact_retention_deadline_ns.is_none()
            {
                return Err(ContractViolation::InvalidValue {
                    field: "platform_runtime_context.capture_artifact_retention_deadline_ns",
                    reason: "must be present when capture_artifact_trust_verified=true",
                });
            }
        }

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTrustPreVerificationInput {
    pub artifact_identity_refs: Vec<ArtifactIdentityRef>,
    pub trust_policy_snapshot_ref: Option<TrustPolicySnapshotRef>,
    pub trust_set_snapshot_ref: Option<TrustSetSnapshotRef>,
    pub upstream_attestation_ref: Option<String>,
    pub upstream_receipt_refs: Vec<String>,
    pub upstream_posture_refs: Vec<String>,
}

impl Validate for ArtifactTrustPreVerificationInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.artifact_identity_refs.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_pre_verification_input.artifact_identity_refs",
                reason: "must not be empty",
            });
        }
        if self.artifact_identity_refs.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "artifact_trust_pre_verification_input.artifact_identity_refs",
                reason: "exceeds max items",
            });
        }
        let mut identity_refs = BTreeSet::new();
        for artifact_identity_ref in &self.artifact_identity_refs {
            artifact_identity_ref.validate()?;
            if !identity_refs.insert(artifact_identity_ref.clone()) {
                return Err(ContractViolation::InvalidValue {
                    field: "artifact_trust_pre_verification_input.artifact_identity_refs",
                    reason: "contains duplicate artifact_identity_ref",
                });
            }
        }
        if let Some(trust_policy_snapshot_ref) = &self.trust_policy_snapshot_ref {
            trust_policy_snapshot_ref.validate()?;
        }
        if let Some(trust_set_snapshot_ref) = &self.trust_set_snapshot_ref {
            trust_set_snapshot_ref.validate()?;
        }
        validate_optional_ascii_token(
            "artifact_trust_pre_verification_input.upstream_attestation_ref",
            &self.upstream_attestation_ref,
            128,
        )?;
        validate_unique_ascii_tokens(
            "artifact_trust_pre_verification_input.upstream_receipt_refs",
            &self.upstream_receipt_refs,
            32,
            128,
        )?;
        validate_unique_ascii_tokens(
            "artifact_trust_pre_verification_input.upstream_posture_refs",
            &self.upstream_posture_refs,
            32,
            128,
        )?;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PersistenceConvergenceState {
    PendingCloudTruth,
    ConvergedToCloudTruth,
    CloudTruthPreserved,
    QuarantinedLocalState,
}

impl Default for PersistenceConvergenceState {
    fn default() -> Self {
        Self::PendingCloudTruth
    }
}

impl PersistenceConvergenceState {
    pub const fn as_str(self) -> &'static str {
        match self {
            PersistenceConvergenceState::PendingCloudTruth => "PENDING_CLOUD_TRUTH",
            PersistenceConvergenceState::ConvergedToCloudTruth => "CONVERGED_TO_CLOUD_TRUTH",
            PersistenceConvergenceState::CloudTruthPreserved => "CLOUD_TRUTH_PRESERVED",
            PersistenceConvergenceState::QuarantinedLocalState => "QUARANTINED_LOCAL_STATE",
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
    #[serde(default)]
    pub convergence_state: PersistenceConvergenceState,
    pub audit_ref: Option<String>,
}

impl PersistenceExecutionState {
    pub fn v1(
        recovery_mode: PersistenceRecoveryMode,
        acknowledgement_state: PersistenceAcknowledgementState,
        reconciliation_decision: Option<ReconciliationDecision>,
        conflict_severity: Option<PersistenceConflictSeverity>,
        cross_node_dedupe_applied: bool,
        convergence_state: PersistenceConvergenceState,
        audit_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            recovery_mode,
            acknowledgement_state,
            reconciliation_decision,
            conflict_severity,
            cross_node_dedupe_applied,
            convergence_state,
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
        if self.cross_node_dedupe_applied
            && self.reconciliation_decision
                != Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome)
        {
            return Err(ContractViolation::InvalidValue {
                field: "persistence_execution_state.cross_node_dedupe_applied",
                reason: "requires REUSE_PRIOR_AUTHORITATIVE_OUTCOME",
            });
        }
        match self.reconciliation_decision {
            Some(ReconciliationDecision::RetrySameOperation) => {
                if self.acknowledgement_state
                    != PersistenceAcknowledgementState::PendingCloudAcknowledgement
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.acknowledgement_state",
                        reason: "RETRY_SAME_OPERATION requires PENDING_CLOUD_ACKNOWLEDGEMENT",
                    });
                }
                if self.conflict_severity != Some(PersistenceConflictSeverity::Retryable) {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.conflict_severity",
                        reason: "RETRY_SAME_OPERATION requires RETRYABLE conflict_severity",
                    });
                }
                if self.convergence_state != PersistenceConvergenceState::PendingCloudTruth {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.convergence_state",
                        reason: "RETRY_SAME_OPERATION requires PENDING_CLOUD_TRUTH",
                    });
                }
            }
            Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome) => {
                if self.acknowledgement_state
                    != PersistenceAcknowledgementState::AuthoritativelyAcknowledged
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.acknowledgement_state",
                        reason: "REUSE_PRIOR_AUTHORITATIVE_OUTCOME requires AUTHORITATIVELY_ACKNOWLEDGED",
                    });
                }
                if self.conflict_severity != Some(PersistenceConflictSeverity::Info) {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.conflict_severity",
                        reason: "REUSE_PRIOR_AUTHORITATIVE_OUTCOME requires INFO conflict_severity",
                    });
                }
                if self.convergence_state != PersistenceConvergenceState::ConvergedToCloudTruth {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.convergence_state",
                        reason:
                            "REUSE_PRIOR_AUTHORITATIVE_OUTCOME requires CONVERGED_TO_CLOUD_TRUTH",
                    });
                }
            }
            Some(ReconciliationDecision::RejectStaleOperation) => {
                if self.acknowledgement_state != PersistenceAcknowledgementState::StaleRejected {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.acknowledgement_state",
                        reason: "REJECT_STALE_OPERATION requires STALE_REJECTED",
                    });
                }
                if self.conflict_severity != Some(PersistenceConflictSeverity::StaleRejected) {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.conflict_severity",
                        reason: "REJECT_STALE_OPERATION requires STALE_REJECTED conflict_severity",
                    });
                }
                if self.convergence_state != PersistenceConvergenceState::CloudTruthPreserved {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.convergence_state",
                        reason: "REJECT_STALE_OPERATION requires CLOUD_TRUTH_PRESERVED",
                    });
                }
            }
            Some(ReconciliationDecision::RequestFreshSessionState) => {
                if self.acknowledgement_state
                    != PersistenceAcknowledgementState::PendingCloudAcknowledgement
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.acknowledgement_state",
                        reason:
                            "REQUEST_FRESH_SESSION_STATE requires PENDING_CLOUD_ACKNOWLEDGEMENT",
                    });
                }
                if self.conflict_severity.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.conflict_severity",
                        reason: "REQUEST_FRESH_SESSION_STATE requires empty conflict_severity",
                    });
                }
                if self.convergence_state != PersistenceConvergenceState::PendingCloudTruth {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.convergence_state",
                        reason: "REQUEST_FRESH_SESSION_STATE requires PENDING_CLOUD_TRUTH",
                    });
                }
            }
            Some(ReconciliationDecision::QuarantineLocalState) => {
                if self.acknowledgement_state
                    != PersistenceAcknowledgementState::QuarantinedLocalState
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.acknowledgement_state",
                        reason: "QUARANTINE_LOCAL_STATE requires QUARANTINED_LOCAL_STATE",
                    });
                }
                if self.conflict_severity != Some(PersistenceConflictSeverity::QuarantineRequired) {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.conflict_severity",
                        reason:
                            "QUARANTINE_LOCAL_STATE requires QUARANTINE_REQUIRED conflict_severity",
                    });
                }
                if self.convergence_state != PersistenceConvergenceState::QuarantinedLocalState {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.convergence_state",
                        reason: "QUARANTINE_LOCAL_STATE requires QUARANTINED_LOCAL_STATE",
                    });
                }
            }
            None => match self.acknowledgement_state {
                PersistenceAcknowledgementState::PendingCloudAcknowledgement => {
                    if self.conflict_severity.is_some() {
                        return Err(ContractViolation::InvalidValue {
                            field: "persistence_execution_state.conflict_severity",
                            reason: "PENDING_CLOUD_ACKNOWLEDGEMENT without reconciliation decision requires empty conflict_severity",
                        });
                    }
                    if self.convergence_state != PersistenceConvergenceState::PendingCloudTruth {
                        return Err(ContractViolation::InvalidValue {
                            field: "persistence_execution_state.convergence_state",
                            reason: "PENDING_CLOUD_ACKNOWLEDGEMENT without reconciliation decision requires PENDING_CLOUD_TRUTH",
                        });
                    }
                }
                PersistenceAcknowledgementState::AuthoritativelyAcknowledged => {
                    if self.convergence_state != PersistenceConvergenceState::ConvergedToCloudTruth
                    {
                        return Err(ContractViolation::InvalidValue {
                            field: "persistence_execution_state.convergence_state",
                            reason:
                                "AUTHORITATIVELY_ACKNOWLEDGED requires CONVERGED_TO_CLOUD_TRUTH",
                        });
                    }
                }
                PersistenceAcknowledgementState::StaleRejected => {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.reconciliation_decision",
                        reason: "STALE_REJECTED requires REJECT_STALE_OPERATION",
                    });
                }
                PersistenceAcknowledgementState::QuarantinedLocalState => {
                    return Err(ContractViolation::InvalidValue {
                        field: "persistence_execution_state.reconciliation_decision",
                        reason: "QUARANTINED_LOCAL_STATE requires QUARANTINE_LOCAL_STATE",
                    });
                }
            },
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentityExecutionStateInput {
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
    pub fn v1(input: IdentityExecutionStateInput) -> Result<Self, ContractViolation> {
        let state = Self {
            consistency_level: input.consistency_level,
            trust_tier: input.trust_tier,
            identity_tier_v2: input.identity_tier_v2,
            spoof_liveness_status: input.spoof_liveness_status,
            step_up_required: input.step_up_required,
            recovery_state: input.recovery_state,
            cluster_drift_detected: input.cluster_drift_detected,
            reason_code: input.reason_code,
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

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeExecutionEnvelope {
    pub request_id: String,
    pub trace_id: String,
    pub idempotency_key: String,
    pub actor_identity: UserId,
    pub device_identity: DeviceId,
    pub platform: AppPlatform,
    pub platform_context: PlatformRuntimeContext,
    /// Non-authoritative ingress-carried inputs for the Section 04 verification seam.
    pub artifact_trust_inputs: Option<ArtifactTrustPreVerificationInput>,
    pub session_id: Option<SessionId>,
    pub turn_id: TurnId,
    pub device_turn_sequence: Option<u64>,
    pub admission_state: AdmissionState,
    pub session_attach_outcome: Option<SessionAttachOutcome>,
    pub persistence_state: Option<PersistenceExecutionState>,
    pub governance_state: Option<GovernanceExecutionState>,
    pub proof_state: Option<ProofExecutionState>,
    pub computation_state: Option<ComputationExecutionState>,
    pub identity_state: Option<IdentityExecutionState>,
    pub memory_state: Option<MemoryExecutionState>,
    pub authority_state: Option<AuthorityExecutionState>,
    /// Authoritative Section 04 trust output carried read-only for downstream consumers.
    pub artifact_trust_state: Option<ArtifactTrustExecutionState>,
    pub law_state: Option<RuntimeLawExecutionState>,
    /// Canonical non-app PH1.VOICE.ID response carried read-only for later runtime adoption.
    pub voice_identity_assertion: Option<Ph1VoiceIdResponse>,
}

impl Eq for RuntimeExecutionEnvelope {}

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
            artifact_trust_inputs: None,
            session_id,
            turn_id,
            device_turn_sequence,
            admission_state,
            session_attach_outcome,
            persistence_state,
            governance_state,
            proof_state: None,
            computation_state: None,
            identity_state: None,
            memory_state: None,
            authority_state: None,
            artifact_trust_state: None,
            law_state: None,
            voice_identity_assertion: None,
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

    pub fn with_voice_identity_assertion(
        &self,
        voice_identity_assertion: Option<Ph1VoiceIdResponse>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.voice_identity_assertion = voice_identity_assertion;
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

    pub fn with_computation_state(
        &self,
        computation_state: Option<ComputationExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.computation_state = computation_state;
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

    pub fn with_artifact_trust_inputs(
        &self,
        artifact_trust_inputs: Option<ArtifactTrustPreVerificationInput>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.artifact_trust_inputs = artifact_trust_inputs;
        next.validate()?;
        Ok(next)
    }

    pub fn with_artifact_trust_state(
        &self,
        artifact_trust_state: Option<ArtifactTrustExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.artifact_trust_state = artifact_trust_state;
        next.validate()?;
        Ok(next)
    }

    pub fn with_law_state(
        &self,
        law_state: Option<RuntimeLawExecutionState>,
    ) -> Result<Self, ContractViolation> {
        let mut next = self.clone();
        next.law_state = law_state;
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
        if let Some(artifact_trust_inputs) = self.artifact_trust_inputs.as_ref() {
            artifact_trust_inputs.validate()?;
        }
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
        if let Some(state) = self.computation_state.as_ref() {
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
        if let Some(state) = self.artifact_trust_state.as_ref() {
            state.validate()?;
        }
        if let Some(state) = self.law_state.as_ref() {
            state.validate()?;
            if state.independent_verification_support.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.law_state.independent_verification_support",
                    reason: "must be present when law_state is present",
                });
            }
        }
        if let Some(assertion) = self.voice_identity_assertion.as_ref() {
            assertion.validate()?;
        }
        if let (Some(governance_state), Some(identity_state)) =
            (self.governance_state.as_ref(), self.identity_state.as_ref())
        {
            if identity_state.cluster_drift_detected
                != governance_cluster_drift_detected(governance_state)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.identity_state.cluster_drift_detected",
                    reason: "must match governance_state.cluster_consistency",
                });
            }
        }
        if let (Some(governance_state), Some(law_state)) =
            (self.governance_state.as_ref(), self.law_state.as_ref())
        {
            let governance_cluster_drift = governance_cluster_drift_detected(governance_state);
            let governance_policy_drift =
                governance_policy_version_drift_detected(governance_state);
            let governance_requires_runtime_law_visibility = governance_state.safe_mode_active
                || governance_cluster_drift
                || governance_policy_drift;

            if governance_state.safe_mode_active
                && law_state.final_law_response_class != RuntimeLawResponseClass::SafeMode
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.law_state.final_law_response_class",
                    reason: "must be SAFE_MODE when governance_state.safe_mode_active=true",
                });
            }

            if governance_cluster_drift
                && matches!(
                    law_state.final_law_response_class,
                    RuntimeLawResponseClass::Allow | RuntimeLawResponseClass::AllowWithWarning
                )
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.law_state.final_law_response_class",
                    reason: "must degrade or stronger when governance_state.cluster_consistency is not CONSISTENT",
                });
            }

            if governance_policy_drift
                && matches!(
                    law_state.final_law_response_class,
                    RuntimeLawResponseClass::Allow | RuntimeLawResponseClass::AllowWithWarning
                )
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.law_state.final_law_response_class",
                    reason: "must degrade or stronger when governance_state.drift_signals includes POLICY_VERSION_DRIFT",
                });
            }

            if governance_requires_runtime_law_visibility
                && !law_state_mentions_runtime_governance(law_state)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_execution_envelope.law_state.subsystem_inputs",
                    reason: "must include RUNTIME_GOVERNANCE when governance_state reports safe mode, cluster drift, or policy-version drift",
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionUnknown,
        SpeakerLabel,
    };
    use crate::ph1art::{
        ArtifactIdentityRef, ArtifactTrustBindingRef, ArtifactTrustControlHints,
        ArtifactTrustDecisionId, ArtifactTrustDecisionProvenance, ArtifactTrustDecisionRecord,
        ArtifactTrustExecutionState, ArtifactVerificationOutcome, ArtifactVerificationResult,
        TrustPolicySnapshotRef, TrustSetSnapshotRef, VerificationBasisFingerprint,
    };
    use crate::runtime_governance::{
        GovernanceClusterConsistency, GovernanceDriftSignal, GovernanceExecutionState,
        GovernanceResponseClass, GovernanceSeverity,
    };
    use crate::runtime_law::{
        RuntimeLawBlastRadiusScope, RuntimeLawExecutionState,
        RuntimeLawIndependentVerificationSupport, RuntimeLawPolicyWindow, RuntimeLawResponseClass,
        RuntimeLawRollbackReadinessState, RuntimeLawRuleCategory, RuntimeLawRuleDescriptor,
        RuntimeProtectedActionClass,
    };
    use crate::{MonotonicTimeNs, ReasonCodeId};

    fn sample_platform_context() -> PlatformRuntimeContext {
        PlatformRuntimeContext::default_for_platform(AppPlatform::Android)
            .expect("default android platform context must be valid")
    }

    fn sample_artifact_trust_state() -> ArtifactTrustExecutionState {
        ArtifactTrustExecutionState {
            decision_records: vec![ArtifactTrustDecisionRecord {
                authority_decision_id: ArtifactTrustDecisionId(
                    "authority.decision.runtime.1".to_string(),
                ),
                artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
                artifact_trust_binding_ref: ArtifactTrustBindingRef(
                    "artifact.trust.binding.1".to_string(),
                ),
                trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
                trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                artifact_verification_result: ArtifactVerificationResult {
                    artifact_identity_ref: ArtifactIdentityRef("artifact.identity.1".to_string()),
                    artifact_trust_binding_ref: ArtifactTrustBindingRef(
                        "artifact.trust.binding.1".to_string(),
                    ),
                    trust_policy_snapshot_ref: TrustPolicySnapshotRef("policy.snap.1".to_string()),
                    trust_set_snapshot_ref: TrustSetSnapshotRef("trust.set.snap.1".to_string()),
                    verification_basis_fingerprint: VerificationBasisFingerprint(
                        "fingerprint.runtime.1".to_string(),
                    ),
                    artifact_verification_outcome: ArtifactVerificationOutcome::VerifiedFresh,
                    artifact_verification_failure_class: None,
                    negative_verification_result_ref: None,
                    verification_timestamp: MonotonicTimeNs(20),
                    verification_cache_used: false,
                    historical_snapshot_ref: None,
                },
                verification_basis_fingerprint: VerificationBasisFingerprint(
                    "fingerprint.runtime.1".to_string(),
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
            }],
            primary_artifact_identity_ref: Some(ArtifactIdentityRef(
                "artifact.identity.1".to_string(),
            )),
            proof_record_ref: None,
        }
    }

    fn sample_voice_identity_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1(
                IdentityConfidence::Medium,
                ReasonCodeId(1),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(1),
                    MonotonicTimeNs(2),
                    Some(SpeakerLabel::speaker_a()),
                )
                .expect("diarization segment must validate")],
            )
            .expect("voice assertion must validate"),
        )
    }

    fn sample_governance_state(
        cluster_consistency: GovernanceClusterConsistency,
        drift_signals: Vec<GovernanceDriftSignal>,
        safe_mode_active: bool,
    ) -> GovernanceExecutionState {
        GovernanceExecutionState::v1(
            "2026.03.08.gov.v1".to_string(),
            cluster_consistency,
            safe_mode_active,
            vec![],
            vec![],
            drift_signals,
            Some("RG-RULE-0001".to_string()),
            Some(if safe_mode_active {
                GovernanceSeverity::Critical
            } else {
                GovernanceSeverity::Warning
            }),
            Some(if safe_mode_active {
                GovernanceResponseClass::SafeMode
            } else {
                GovernanceResponseClass::Degrade
            }),
            Some("GOV-DEC-0000000001".to_string()),
        )
        .expect("governance execution state must validate")
    }

    fn sample_identity_state(cluster_drift_detected: bool) -> IdentityExecutionState {
        IdentityExecutionState::v1(IdentityExecutionStateInput {
            consistency_level: IdentityVerificationConsistencyLevel::StrictVerified,
            trust_tier: IdentityTrustTier::Verified,
            identity_tier_v2: IdentityTierV2::Confirmed,
            spoof_liveness_status: SpoofLivenessStatus::Live,
            step_up_required: false,
            recovery_state: IdentityRecoveryState::None,
            cluster_drift_detected,
            reason_code: None,
        })
        .expect("identity execution state must validate")
    }

    fn sample_law_state(
        final_law_response_class: RuntimeLawResponseClass,
        subsystem_inputs: Vec<String>,
    ) -> RuntimeLawExecutionState {
        let (final_law_severity, law_reason_codes, triggered_rule_ids) =
            match final_law_response_class {
                RuntimeLawResponseClass::Allow => (
                    crate::runtime_law::RuntimeLawSeverity::Info,
                    vec!["LAW_ALLOW_BASELINE".to_string()],
                    vec!["RL-BASE-001".to_string()],
                ),
                RuntimeLawResponseClass::AllowWithWarning | RuntimeLawResponseClass::Degrade => (
                    crate::runtime_law::RuntimeLawSeverity::Warning,
                    vec!["LAW_GOVERNANCE_POLICY_DRIFT".to_string()],
                    vec!["RL-GOV-003".to_string()],
                ),
                RuntimeLawResponseClass::Block => (
                    crate::runtime_law::RuntimeLawSeverity::Blocking,
                    vec!["LAW_GOVERNANCE_DIVERGENCE".to_string()],
                    vec!["RL-GOV-002".to_string()],
                ),
                RuntimeLawResponseClass::Quarantine => (
                    crate::runtime_law::RuntimeLawSeverity::QuarantineRequired,
                    vec!["LAW_GOVERNANCE_DIVERGENCE".to_string()],
                    vec!["RL-GOV-002".to_string()],
                ),
                RuntimeLawResponseClass::SafeMode => (
                    crate::runtime_law::RuntimeLawSeverity::Critical,
                    vec!["LAW_GOVERNANCE_SAFE_MODE".to_string()],
                    vec!["RL-GOV-001".to_string()],
                ),
            };
        let law_policy_version = "2026.03.08.law.v1".to_string();
        let safe_mode_active = final_law_response_class == RuntimeLawResponseClass::SafeMode;
        let quarantined_scopes = if final_law_response_class == RuntimeLawResponseClass::Quarantine
        {
            vec![RuntimeLawBlastRadiusScope::ClusterScope
                .as_str()
                .to_string()]
        } else {
            Vec::new()
        };

        RuntimeLawExecutionState::v1(
            RuntimeProtectedActionClass::ArtifactAuthority,
            final_law_response_class,
            final_law_severity,
            law_reason_codes,
            law_policy_version.clone(),
            None,
            RuntimeLawRollbackReadinessState::NotRequired,
            RuntimeLawBlastRadiusScope::ClusterScope,
            None,
            triggered_rule_ids.clone(),
            subsystem_inputs,
            "LAW-DEC-0000000001".to_string(),
        )
        .expect("runtime law state must validate")
        .with_independent_verification_support(Some(sample_law_verification_support(
            law_policy_version,
            triggered_rule_ids,
            safe_mode_active,
            quarantined_scopes,
        )))
        .expect("independent verification support must attach")
    }

    fn sample_law_verification_support(
        law_policy_version: String,
        triggered_rule_ids: Vec<String>,
        safe_mode_active: bool,
        quarantined_scopes: Vec<String>,
    ) -> RuntimeLawIndependentVerificationSupport {
        let support_rule_ids = ["RL-BASE-001", "RL-GOV-001", "RL-GOV-002", "RL-GOV-003"];
        let mut rule_registry = Vec::new();
        for rule_id in support_rule_ids {
            let category = if rule_id == "RL-BASE-001" {
                RuntimeLawRuleCategory::Envelope
            } else {
                RuntimeLawRuleCategory::Governance
            };
            let description = match rule_id {
                "RL-BASE-001" => "baseline allow rule",
                "RL-GOV-001" => "governance safe mode rule",
                "RL-GOV-002" => "governance cluster divergence rule",
                "RL-GOV-003" => "governance policy version drift rule",
                _ => "runtime law verification rule",
            };
            rule_registry.push(
                RuntimeLawRuleDescriptor::v1(
                    rule_id.to_string(),
                    category,
                    true,
                    law_policy_version.clone(),
                    if category == RuntimeLawRuleCategory::Envelope {
                        "RUNTIME_LAW".to_string()
                    } else {
                        SUBSYSTEM_RUNTIME_GOVERNANCE.to_string()
                    },
                    description.to_string(),
                )
                .expect("runtime law rule descriptor must validate"),
            );
        }

        for triggered_rule_id in triggered_rule_ids {
            if rule_registry
                .iter()
                .all(|descriptor| descriptor.rule_id != triggered_rule_id)
            {
                rule_registry.push(
                    RuntimeLawRuleDescriptor::v1(
                        triggered_rule_id,
                        RuntimeLawRuleCategory::Governance,
                        true,
                        law_policy_version.clone(),
                        SUBSYSTEM_RUNTIME_GOVERNANCE.to_string(),
                        "runtime law triggered rule".to_string(),
                    )
                    .expect("triggered runtime law rule descriptor must validate"),
                );
            }
        }

        RuntimeLawIndependentVerificationSupport::v1(
            "runtime-law-node-a".to_string(),
            RuntimeLawPolicyWindow::v1(
                law_policy_version.clone(),
                law_policy_version.clone(),
                law_policy_version,
            )
            .expect("runtime law policy window must validate"),
            rule_registry,
            safe_mode_active,
            quarantined_scopes,
        )
        .expect("runtime law independent verification support must validate")
    }

    #[test]
    fn at_runtime_execution_01_attested_integrity_requires_attestation_ref() {
        let mut context = sample_platform_context();
        context.integrity_status = ClientIntegrityStatus::Attested;

        let err = context
            .validate()
            .expect_err("ATTESTED integrity posture must require attestation_ref");
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "platform_runtime_context.attestation_ref");
                assert_eq!(reason, "must be present when integrity_status=ATTESTED");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_02_attestation_ref_requires_attested_integrity() {
        let mut context = sample_platform_context();
        context.integrity_status = ClientIntegrityStatus::IntegrityVerified;
        context.attestation_ref = Some("attestation:android:1".to_string());

        let err = context
            .validate()
            .expect_err("attestation_ref must require ATTESTED integrity posture");
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "platform_runtime_context.integrity_status");
                assert_eq!(reason, "must be ATTESTED when attestation_ref is present");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_03_attested_integrity_with_attestation_ref_is_valid() {
        let mut context = sample_platform_context();
        context.integrity_status = ClientIntegrityStatus::Attested;
        context.attestation_ref = Some("attestation:android:2".to_string());

        context
            .validate()
            .expect("ATTESTED integrity posture with attestation_ref must validate");
    }

    #[test]
    fn at_runtime_execution_04_capture_artifact_trust_requires_lifecycle_metadata() {
        let mut context = sample_platform_context();
        context.integrity_status = ClientIntegrityStatus::Attested;
        context.attestation_ref = Some("attestation:android:3".to_string());
        context.capture_artifact_trust_verified = true;

        let err = context
            .validate()
            .expect_err("trusted capture artifact posture must require lifecycle metadata");
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "platform_runtime_context.capture_artifact_retention_deadline_ns"
                );
                assert_eq!(
                    reason,
                    "must be present when capture_artifact_trust_verified=true"
                );
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_05_capture_artifact_retention_deadline_must_follow_observed_at() {
        let mut context = sample_platform_context();
        context.integrity_status = ClientIntegrityStatus::Attested;
        context.attestation_ref = Some("attestation:android:4".to_string());
        context.capture_artifact_observed_at_ns = Some(10);
        context.capture_artifact_retention_deadline_ns = Some(9);

        let err = context
            .validate()
            .expect_err("capture artifact lifecycle deadline must not precede observed_at");
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "platform_runtime_context.capture_artifact_retention_deadline_ns"
                );
                assert_eq!(reason, "must be >= capture_artifact_observed_at_ns");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_06_artifact_trust_inputs_require_artifact_refs() {
        let err = ArtifactTrustPreVerificationInput {
            artifact_identity_refs: Vec::new(),
            trust_policy_snapshot_ref: None,
            trust_set_snapshot_ref: None,
            upstream_attestation_ref: None,
            upstream_receipt_refs: Vec::new(),
            upstream_posture_refs: Vec::new(),
        }
        .validate()
        .expect_err("artifact trust inputs must carry at least one artifact ref");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "artifact_trust_pre_verification_input.artifact_identity_refs"
                );
                assert_eq!(reason, "must not be empty");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_07_envelope_accepts_artifact_trust_state_transport() {
        let envelope = RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request:1".to_string(),
            "trace:1".to_string(),
            "idem:1".to_string(),
            UserId::new("user_runtime_1").expect("valid actor user id"),
            DeviceId::new("device_runtime_1").expect("valid device id"),
            AppPlatform::Android,
            sample_platform_context(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::SessionResolved,
            None,
        )
        .expect("baseline envelope must validate")
        .with_artifact_trust_inputs(Some(ArtifactTrustPreVerificationInput {
            artifact_identity_refs: vec![ArtifactIdentityRef("artifact.identity.1".to_string())],
            trust_policy_snapshot_ref: Some(TrustPolicySnapshotRef("policy.snap.1".to_string())),
            trust_set_snapshot_ref: Some(TrustSetSnapshotRef("trust.set.snap.1".to_string())),
            upstream_attestation_ref: Some("attestation:android:runtime".to_string()),
            upstream_receipt_refs: vec!["receipt.runtime.1".to_string()],
            upstream_posture_refs: vec!["capture-posture.attested".to_string()],
        }))
        .expect("artifact trust inputs must validate")
        .with_artifact_trust_state(Some(sample_artifact_trust_state()))
        .expect("artifact trust state must validate");

        assert!(envelope.artifact_trust_inputs.is_some());
        assert!(envelope.artifact_trust_state.is_some());
    }

    #[test]
    fn at_runtime_execution_08_envelope_accepts_voice_identity_assertion_transport() {
        let assertion = sample_voice_identity_assertion();
        let envelope = RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request:voice:1".to_string(),
            "trace:voice:1".to_string(),
            "idem:voice:1".to_string(),
            UserId::new("user_runtime_voice_1").expect("valid actor user id"),
            DeviceId::new("device_runtime_voice_1").expect("valid device id"),
            AppPlatform::Android,
            sample_platform_context(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::SessionResolved,
            None,
        )
        .expect("baseline envelope must validate")
        .with_voice_identity_assertion(Some(assertion.clone()))
        .expect("voice assertion transport must validate");

        assert_eq!(envelope.voice_identity_assertion, Some(assertion));
    }

    #[test]
    fn at_runtime_execution_09_envelope_accepts_persistence_state_transport() {
        let persistence_state = PersistenceExecutionState::v1(
            PersistenceRecoveryMode::Recovering,
            PersistenceAcknowledgementState::AuthoritativelyAcknowledged,
            Some(ReconciliationDecision::ReusePriorAuthoritativeOutcome),
            Some(PersistenceConflictSeverity::Info),
            true,
            PersistenceConvergenceState::ConvergedToCloudTruth,
            Some("audit.persistence.runtime.1".to_string()),
        )
        .expect("persistence execution state must validate");
        let envelope = RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request:persistence:1".to_string(),
            "trace:persistence:1".to_string(),
            "idem:persistence:1".to_string(),
            UserId::new("user_runtime_persistence_1").expect("valid actor user id"),
            DeviceId::new("device_runtime_persistence_1").expect("valid device id"),
            AppPlatform::Android,
            sample_platform_context(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::SessionResolved,
            None,
        )
        .expect("baseline envelope must validate")
        .with_persistence_state(Some(persistence_state.clone()))
        .expect("persistence state transport must validate");
        let transported = envelope
            .persistence_state
            .expect("persistence state must be present after transport");

        assert_eq!(transported.recovery_mode, persistence_state.recovery_mode);
        assert_eq!(
            transported.acknowledgement_state,
            persistence_state.acknowledgement_state
        );
        assert_eq!(
            transported.reconciliation_decision,
            persistence_state.reconciliation_decision
        );
        assert_eq!(
            transported.conflict_severity,
            persistence_state.conflict_severity
        );
        assert_eq!(
            transported.cross_node_dedupe_applied,
            persistence_state.cross_node_dedupe_applied
        );
        assert_eq!(
            transported.convergence_state,
            persistence_state.convergence_state
        );
        assert_eq!(transported.audit_ref, persistence_state.audit_ref);
    }

    #[test]
    fn at_runtime_execution_10_persistence_state_transport_rejects_non_ascii_audit_ref() {
        let invalid_state = PersistenceExecutionState {
            recovery_mode: PersistenceRecoveryMode::Recovering,
            acknowledgement_state: PersistenceAcknowledgementState::StaleRejected,
            reconciliation_decision: Some(ReconciliationDecision::RejectStaleOperation),
            conflict_severity: Some(PersistenceConflictSeverity::StaleRejected),
            cross_node_dedupe_applied: false,
            convergence_state: PersistenceConvergenceState::CloudTruthPreserved,
            audit_ref: Some("审计.persistence.runtime.1".to_string()),
        };
        let err = RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request:persistence:invalid:1".to_string(),
            "trace:persistence:invalid:1".to_string(),
            "idem:persistence:invalid:1".to_string(),
            UserId::new("user_runtime_persistence_invalid_1").expect("valid actor user id"),
            DeviceId::new("device_runtime_persistence_invalid_1").expect("valid device id"),
            AppPlatform::Android,
            sample_platform_context(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::SessionResolved,
            None,
        )
        .expect("baseline envelope must validate")
        .with_persistence_state(Some(invalid_state))
        .expect_err("non-ASCII persistence audit_ref must be rejected");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "persistence_execution_state.audit_ref");
                assert_eq!(reason, "must be ASCII");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_11_cluster_drift_requires_identity_alignment() {
        let envelope =
            RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                "request:cluster:1".to_string(),
                "trace:cluster:1".to_string(),
                "idem:cluster:1".to_string(),
                UserId::new("user_runtime_cluster_1").expect("valid actor user id"),
                DeviceId::new("device_runtime_cluster_1").expect("valid device id"),
                AppPlatform::Android,
                sample_platform_context(),
                Some(SessionId(1)),
                TurnId(1),
                Some(1),
                AdmissionState::SessionResolved,
                None,
            )
            .expect("baseline envelope must validate")
            .with_governance_state(Some(sample_governance_state(
                GovernanceClusterConsistency::Diverged,
                vec![],
                false,
            )))
            .expect("governance state must validate");

        let err = envelope
            .with_identity_state(Some(sample_identity_state(false)))
            .expect_err("identity cluster drift must align with governance cluster consistency");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "runtime_execution_envelope.identity_state.cluster_drift_detected"
                );
                assert_eq!(reason, "must match governance_state.cluster_consistency");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_12_policy_version_drift_requires_governed_law_posture() {
        let envelope =
            RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                "request:law:1".to_string(),
                "trace:law:1".to_string(),
                "idem:law:1".to_string(),
                UserId::new("user_runtime_law_1").expect("valid actor user id"),
                DeviceId::new("device_runtime_law_1").expect("valid device id"),
                AppPlatform::Android,
                sample_platform_context(),
                Some(SessionId(1)),
                TurnId(1),
                Some(1),
                AdmissionState::SessionResolved,
                None,
            )
            .expect("baseline envelope must validate")
            .with_governance_state(Some(sample_governance_state(
                GovernanceClusterConsistency::Consistent,
                vec![GovernanceDriftSignal::PolicyVersionDrift],
                false,
            )))
            .expect("governance state must validate");

        let err = envelope
            .with_law_state(Some(sample_law_state(
                RuntimeLawResponseClass::Allow,
                vec!["RUNTIME_LAW".to_string()],
            )))
            .expect_err("policy-version drift must require a governed non-allow law posture");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "runtime_execution_envelope.law_state.final_law_response_class"
                );
                assert_eq!(
                    reason,
                    "must degrade or stronger when governance_state.drift_signals includes POLICY_VERSION_DRIFT"
                );
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_13_cluster_and_policy_drift_accept_governed_law_visibility_transport() {
        let envelope =
            RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                "request:law:2".to_string(),
                "trace:law:2".to_string(),
                "idem:law:2".to_string(),
                UserId::new("user_runtime_law_2").expect("valid actor user id"),
                DeviceId::new("device_runtime_law_2").expect("valid device id"),
                AppPlatform::Android,
                sample_platform_context(),
                Some(SessionId(1)),
                TurnId(1),
                Some(1),
                AdmissionState::SessionResolved,
                None,
            )
            .expect("baseline envelope must validate")
            .with_governance_state(Some(sample_governance_state(
                GovernanceClusterConsistency::CompatibilityWindow,
                vec![GovernanceDriftSignal::PolicyVersionDrift],
                false,
            )))
            .expect("governance state must validate")
            .with_identity_state(Some(sample_identity_state(true)))
            .expect("identity state must validate when cluster drift is aligned")
            .with_law_state(Some(sample_law_state(
                RuntimeLawResponseClass::Degrade,
                vec![SUBSYSTEM_RUNTIME_GOVERNANCE.to_string()],
            )))
            .expect("governed law-state transport must validate");

        assert_eq!(
            envelope
                .identity_state
                .expect("identity state must be present")
                .cluster_drift_detected,
            true
        );
        assert_eq!(
            envelope
                .law_state
                .expect("law state must be present")
                .final_law_response_class,
            RuntimeLawResponseClass::Degrade
        );
    }

    #[test]
    fn at_runtime_execution_14_law_state_transport_requires_independent_verification_support() {
        let law_state = RuntimeLawExecutionState::v1(
            RuntimeProtectedActionClass::ArtifactAuthority,
            RuntimeLawResponseClass::Allow,
            crate::runtime_law::RuntimeLawSeverity::Info,
            vec!["LAW_ALLOW_BASELINE".to_string()],
            "2026.03.08.law.v1".to_string(),
            None,
            RuntimeLawRollbackReadinessState::NotRequired,
            RuntimeLawBlastRadiusScope::ClusterScope,
            None,
            vec!["RL-BASE-001".to_string()],
            vec!["RUNTIME_LAW".to_string()],
            "LAW-DEC-0000000002".to_string(),
        )
        .expect("runtime law state must validate without support before envelope transport");

        let envelope =
            RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                "request:law:3".to_string(),
                "trace:law:3".to_string(),
                "idem:law:3".to_string(),
                UserId::new("user_runtime_law_3").expect("valid actor user id"),
                DeviceId::new("device_runtime_law_3").expect("valid device id"),
                AppPlatform::Android,
                sample_platform_context(),
                Some(SessionId(1)),
                TurnId(1),
                Some(1),
                AdmissionState::SessionResolved,
                None,
            )
            .expect("baseline envelope must validate");

        let err = envelope
            .with_law_state(Some(law_state))
            .expect_err("law-state transport must require independent verification support");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "runtime_execution_envelope.law_state.independent_verification_support"
                );
                assert_eq!(reason, "must be present when law_state is present");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_15_law_state_transport_accepts_independent_verification_support() {
        let envelope =
            RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
                "request:law:4".to_string(),
                "trace:law:4".to_string(),
                "idem:law:4".to_string(),
                UserId::new("user_runtime_law_4").expect("valid actor user id"),
                DeviceId::new("device_runtime_law_4").expect("valid device id"),
                AppPlatform::Android,
                sample_platform_context(),
                Some(SessionId(1)),
                TurnId(1),
                Some(1),
                AdmissionState::SessionResolved,
                None,
            )
            .expect("baseline envelope must validate")
            .with_law_state(Some(sample_law_state(
                RuntimeLawResponseClass::AllowWithWarning,
                vec![SUBSYSTEM_RUNTIME_GOVERNANCE.to_string()],
            )))
            .expect("law-state transport with verification support must validate");

        assert!(envelope
            .law_state
            .and_then(|law_state| law_state.independent_verification_support)
            .is_some());
    }

    #[test]
    fn at_runtime_execution_16_persistence_state_requires_lawful_retry_policy_shape() {
        let err = PersistenceExecutionState::v1(
            PersistenceRecoveryMode::DegradedRecovery,
            PersistenceAcknowledgementState::PendingCloudAcknowledgement,
            Some(ReconciliationDecision::RetrySameOperation),
            None,
            false,
            PersistenceConvergenceState::PendingCloudTruth,
            Some("audit.persistence.retry.1".to_string()),
        )
        .expect_err("retry policy without retryable conflict_severity must be rejected");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "persistence_execution_state.conflict_severity");
                assert_eq!(
                    reason,
                    "RETRY_SAME_OPERATION requires RETRYABLE conflict_severity"
                );
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn at_runtime_execution_17_persistence_state_accepts_fresh_session_reconciliation_shape() {
        let state = PersistenceExecutionState::v1(
            PersistenceRecoveryMode::Recovering,
            PersistenceAcknowledgementState::PendingCloudAcknowledgement,
            Some(ReconciliationDecision::RequestFreshSessionState),
            None,
            false,
            PersistenceConvergenceState::PendingCloudTruth,
            Some("audit.persistence.reconcile.1".to_string()),
        )
        .expect("fresh-session reconciliation state must validate");

        assert_eq!(
            state.reconciliation_decision,
            Some(ReconciliationDecision::RequestFreshSessionState)
        );
        assert_eq!(
            state.convergence_state,
            PersistenceConvergenceState::PendingCloudTruth
        );
    }
}
