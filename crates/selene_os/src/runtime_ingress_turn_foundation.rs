#![forbid(unsafe_code)]

use crate::app_ingress::{AppOnboardingContinueAction, AppOnboardingContinueRequest};
use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{CorrelationId, DeviceId, TurnId};
use selene_kernel_contracts::ph1l::SessionId;
use selene_kernel_contracts::ph1link::{AppPlatform, InviteOpenActivateCommitRequest, TokenId};
use selene_kernel_contracts::ph1onb::{OnboardingSessionId, SenderVerifyDecision};
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, FailureClass, PlatformRuntimeContext, RuntimeEntryTrigger,
    RuntimeExecutionEnvelope, SessionAttachOutcome,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, SessionState, Validate};

use crate::runtime_bootstrap::{
    RuntimeBootstrapError, RuntimeClock, RuntimeProcess, RuntimeSecretsProvider,
    RuntimeServiceContainer,
};
use crate::runtime_request_foundation::{
    RuntimeFeatureFlagOverrides, RuntimeFoundationRequest, RuntimePreparedRequest,
    RuntimeRequestEnvelopeFoundation, RuntimeRequestEnvelopeInput, RuntimeRequestFoundationConfig,
    RuntimeRequestFoundationError, RuntimeRouteDefinition, RuntimeRouteHandlerKind, RuntimeRouter,
    CANONICAL_TURN_ENDPOINT_PATH, INVITE_CLICK_ENDPOINT_PATH, ONBOARDING_CONTINUE_ENDPOINT_PATH,
};
use crate::runtime_session_foundation::{
    RuntimeSessionFoundation, SessionAccessClass, SessionFoundationError,
    SessionFoundationErrorKind, SessionRuntimeProjection, SessionTurnDeferred, SessionTurnPermit,
    SessionTurnResolution,
};

const MAX_AUTHORIZATION_LEN: usize = 512;
const MAX_TEXT_BYTES: usize = 16_384;
const MAX_BINARY_BYTES: usize = 262_144;

mod reason_codes {
    pub const INGRESS_AUTHORIZATION_INVALID: &str = "runtime_turn_ingress_authorization_invalid";
    pub const INGRESS_COMPATIBILITY_ONLY: &str = "runtime_turn_ingress_family_compatibility_only";
    pub const INGRESS_PAYLOAD_INVALID: &str = "runtime_turn_ingress_payload_invalid";
    pub const INGRESS_UNSUPPORTED_CONTENT_TYPE: &str =
        "runtime_turn_ingress_unsupported_content_type";
    pub const INGRESS_TRIGGER_INVALID: &str = "runtime_turn_ingress_trigger_invalid";
    pub const INGRESS_ENVELOPE_INVALID: &str = "runtime_turn_ingress_envelope_invalid";
    pub const INGRESS_STAGE_INVALID: &str = "runtime_turn_ingress_stage_invalid";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalIngressFamily {
    VoiceTurn,
    InviteClickCompatibility,
    OnboardingContinueCompatibility,
}

impl CanonicalIngressFamily {
    pub const fn route_path(self) -> &'static str {
        match self {
            CanonicalIngressFamily::VoiceTurn => CANONICAL_TURN_ENDPOINT_PATH,
            CanonicalIngressFamily::InviteClickCompatibility => INVITE_CLICK_ENDPOINT_PATH,
            CanonicalIngressFamily::OnboardingContinueCompatibility => {
                ONBOARDING_CONTINUE_ENDPOINT_PATH
            }
        }
    }

    pub const fn handler(self) -> RuntimeRouteHandlerKind {
        match self {
            CanonicalIngressFamily::VoiceTurn => RuntimeRouteHandlerKind::CanonicalTurnIngress,
            CanonicalIngressFamily::InviteClickCompatibility => {
                RuntimeRouteHandlerKind::InviteClickCompatibility
            }
            CanonicalIngressFamily::OnboardingContinueCompatibility => {
                RuntimeRouteHandlerKind::OnboardingContinueCompatibility
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanonicalTurnModality {
    Voice,
    Text,
    File,
    Image,
    Camera,
    Compatibility,
}

impl CanonicalTurnModality {
    pub const fn as_str(self) -> &'static str {
        match self {
            CanonicalTurnModality::Voice => "VOICE",
            CanonicalTurnModality::Text => "TEXT",
            CanonicalTurnModality::File => "FILE",
            CanonicalTurnModality::Image => "IMAGE",
            CanonicalTurnModality::Camera => "CAMERA",
            CanonicalTurnModality::Compatibility => "COMPATIBILITY",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionResolveMode {
    ResolveOrOpen,
    ResumeExisting,
    RecoverExisting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage4ActivationSource {
    TypedInput,
    WakeCandidate,
    SideButton,
    ExplicitMic,
    RecordButton,
}

impl Stage4ActivationSource {
    pub const fn requested_trigger(self) -> RuntimeEntryTrigger {
        match self {
            Stage4ActivationSource::WakeCandidate => RuntimeEntryTrigger::WakeWord,
            Stage4ActivationSource::TypedInput
            | Stage4ActivationSource::SideButton
            | Stage4ActivationSource::ExplicitMic
            | Stage4ActivationSource::RecordButton => RuntimeEntryTrigger::Explicit,
        }
    }

    pub const fn is_record_button(self) -> bool {
        matches!(self, Stage4ActivationSource::RecordButton)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage4TurnBoundaryKind {
    CandidatePreview,
    CommittedLiveTurn,
    RecordArtifactOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage4RecordingState {
    Idle,
    Recording,
    Paused,
    Stopped,
    Processing,
    Complete,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage4PacketRouteAuthority {
    pub can_route_tools: bool,
    pub can_route_search: bool,
    pub can_route_providers: bool,
    pub can_route_tts: bool,
    pub can_route_protected_execution: bool,
}

impl Stage4PacketRouteAuthority {
    pub const fn none() -> Self {
        Self {
            can_route_tools: false,
            can_route_search: false,
            can_route_providers: false,
            can_route_tts: false,
            can_route_protected_execution: false,
        }
    }

    pub const fn any_route_enabled(self) -> bool {
        self.can_route_tools
            || self.can_route_search
            || self.can_route_providers
            || self.can_route_tts
            || self.can_route_protected_execution
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage4RecordBoundary {
    pub recording_session_id: String,
    pub recording_state: Stage4RecordingState,
    pub audio_artifact_id: String,
    pub consent_state_id: String,
    pub artifact_lane_handoff_ref: String,
}

impl Validate for Stage4RecordBoundary {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage4_record_boundary.recording_session_id",
            &self.recording_session_id,
        )?;
        validate_stage4_ref(
            "stage4_record_boundary.audio_artifact_id",
            &self.audio_artifact_id,
        )?;
        validate_stage4_ref(
            "stage4_record_boundary.consent_state_id",
            &self.consent_state_id,
        )?;
        validate_stage4_ref(
            "stage4_record_boundary.artifact_lane_handoff_ref",
            &self.artifact_lane_handoff_ref,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage4ActivationPacket {
    pub source: Stage4ActivationSource,
    pub platform_context: PlatformRuntimeContext,
    pub session_hint: Option<SessionId>,
    pub consent_state_id: Option<String>,
    pub device_trust_ref: Option<String>,
    pub provider_budget_ref: Option<String>,
    pub audit_id: Option<String>,
}

impl Stage4ActivationPacket {
    pub fn new(
        source: Stage4ActivationSource,
        platform_context: PlatformRuntimeContext,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            source,
            platform_context,
            session_hint: None,
            consent_state_id: None,
            device_trust_ref: None,
            provider_budget_ref: None,
            audit_id: None,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub const fn route_authority(&self) -> Stage4PacketRouteAuthority {
        Stage4PacketRouteAuthority::none()
    }
}

impl Validate for Stage4ActivationPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.platform_context.validate()?;
        if self.platform_context.requested_trigger != self.source.requested_trigger() {
            return Err(ContractViolation::InvalidValue {
                field: "stage4_activation_packet.platform_context.requested_trigger",
                reason: "activation source and platform trigger must agree",
            });
        }
        if !self.platform_context.trigger_allowed {
            return Err(ContractViolation::InvalidValue {
                field: "stage4_activation_packet.platform_context.trigger_allowed",
                reason: "activation packet cannot carry a disallowed platform trigger",
            });
        }
        validate_stage4_optional_ref(
            "stage4_activation_packet.consent_state_id",
            self.consent_state_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage4_activation_packet.device_trust_ref",
            self.device_trust_ref.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage4_activation_packet.provider_budget_ref",
            self.provider_budget_ref.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage4_activation_packet.audit_id",
            self.audit_id.as_deref(),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage4TurnBoundaryPacket {
    pub activation: Stage4ActivationPacket,
    pub boundary_kind: Stage4TurnBoundaryKind,
    pub turn_id: Option<TurnId>,
    pub device_turn_sequence: Option<u64>,
    pub modality: Option<CanonicalTurnModality>,
    pub record_boundary: Option<Stage4RecordBoundary>,
    pub route_id: Option<String>,
}

impl Stage4TurnBoundaryPacket {
    pub fn candidate_preview(
        activation: Stage4ActivationPacket,
        modality: Option<CanonicalTurnModality>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            activation,
            boundary_kind: Stage4TurnBoundaryKind::CandidatePreview,
            turn_id: None,
            device_turn_sequence: None,
            modality,
            record_boundary: None,
            route_id: None,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn committed_live_turn(
        activation: Stage4ActivationPacket,
        turn_id: TurnId,
        device_turn_sequence: u64,
        modality: CanonicalTurnModality,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            activation,
            boundary_kind: Stage4TurnBoundaryKind::CommittedLiveTurn,
            turn_id: Some(turn_id),
            device_turn_sequence: Some(device_turn_sequence),
            modality: Some(modality),
            record_boundary: None,
            route_id: None,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn record_artifact_only(
        activation: Stage4ActivationPacket,
        record_boundary: Stage4RecordBoundary,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            activation,
            boundary_kind: Stage4TurnBoundaryKind::RecordArtifactOnly,
            turn_id: None,
            device_turn_sequence: None,
            modality: None,
            record_boundary: Some(record_boundary),
            route_id: None,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub const fn route_authority(&self) -> Stage4PacketRouteAuthority {
        Stage4PacketRouteAuthority::none()
    }

    pub const fn is_committed_live_turn(&self) -> bool {
        matches!(
            self.boundary_kind,
            Stage4TurnBoundaryKind::CommittedLiveTurn
        )
    }

    pub const fn record_mode_can_be_live_chat(&self) -> bool {
        false
    }
}

impl Validate for Stage4TurnBoundaryPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.activation.validate()?;
        validate_stage4_optional_ref(
            "stage4_turn_boundary_packet.route_id",
            self.route_id.as_deref(),
        )?;
        match self.boundary_kind {
            Stage4TurnBoundaryKind::CandidatePreview => {
                if self.turn_id.is_some()
                    || self.device_turn_sequence.is_some()
                    || self.record_boundary.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet",
                        reason: "candidate preview packets cannot carry committed turn or record artifact state",
                    });
                }
                if self.activation.source.is_record_button() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet.activation.source",
                        reason: "record button packets must use the artifact-only boundary",
                    });
                }
            }
            Stage4TurnBoundaryKind::CommittedLiveTurn => {
                if self.activation.source.is_record_button() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet.activation.source",
                        reason: "record button activation cannot create a committed live turn",
                    });
                }
                if self.activation.session_hint.is_none()
                    || self.turn_id.is_none()
                    || self.device_turn_sequence.unwrap_or_default() == 0
                    || self.modality.is_none()
                    || self.record_boundary.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet",
                        reason: "committed live turns require session, turn, sequence, modality, and no record boundary",
                    });
                }
            }
            Stage4TurnBoundaryKind::RecordArtifactOnly => {
                if !self.activation.source.is_record_button() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet.activation.source",
                        reason: "record artifact boundary requires record-button activation",
                    });
                }
                if self.turn_id.is_some()
                    || self.device_turn_sequence.is_some()
                    || self.modality.is_some()
                    || self.route_id.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet",
                        reason: "record artifact boundary cannot carry live turn or route state",
                    });
                }
                let Some(record_boundary) = self.record_boundary.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage4_turn_boundary_packet.record_boundary",
                        reason: "record artifact boundary requires record boundary fields",
                    });
                };
                record_boundary.validate()?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawTurnPayload {
    Text {
        content_type: String,
        text: String,
    },
    Binary {
        content_type: String,
        bytes: Vec<u8>,
    },
}

#[derive(Debug, Clone)]
pub enum CompatibilityRequestPayload {
    InviteClick(InviteOpenActivateCommitRequest),
    OnboardingContinue(AppOnboardingContinueRequest),
}

#[derive(Debug, Clone)]
pub struct RuntimeCanonicalIngressRequest {
    pub family: CanonicalIngressFamily,
    pub envelope_input: RuntimeRequestEnvelopeInput,
    pub authorization_bearer: String,
    pub actor_identity: UserId,
    pub device_identity: DeviceId,
    pub platform_context: PlatformRuntimeContext,
    pub session_hint: Option<SessionId>,
    pub device_turn_sequence: Option<u64>,
    pub session_resolve_mode: SessionResolveMode,
    pub modality: Option<CanonicalTurnModality>,
    pub payload: Option<RawTurnPayload>,
    pub compatibility_payload: Option<CompatibilityRequestPayload>,
    pub overload_active: bool,
    pub feature_flag_overrides: RuntimeFeatureFlagOverrides,
}

impl RuntimeCanonicalIngressRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn turn(
        envelope_input: RuntimeRequestEnvelopeInput,
        authorization_bearer: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform_context: PlatformRuntimeContext,
        session_hint: Option<SessionId>,
        device_turn_sequence: u64,
        session_resolve_mode: SessionResolveMode,
        modality: CanonicalTurnModality,
        payload: RawTurnPayload,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            family: CanonicalIngressFamily::VoiceTurn,
            envelope_input,
            authorization_bearer,
            actor_identity,
            device_identity,
            platform_context,
            session_hint,
            device_turn_sequence: Some(device_turn_sequence),
            session_resolve_mode,
            modality: Some(modality),
            payload: Some(payload),
            compatibility_payload: None,
            overload_active: false,
            feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
        };
        request.validate()?;
        Ok(request)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn invite_click(
        envelope_input: RuntimeRequestEnvelopeInput,
        platform_context: PlatformRuntimeContext,
        invite_request: InviteOpenActivateCommitRequest,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            family: CanonicalIngressFamily::InviteClickCompatibility,
            envelope_input,
            authorization_bearer: String::new(),
            actor_identity: compatibility_actor_identity(&invite_request)?,
            device_identity: compatibility_device_identity(&invite_request)?,
            platform_context,
            session_hint: None,
            device_turn_sequence: None,
            session_resolve_mode: SessionResolveMode::ResolveOrOpen,
            modality: None,
            payload: None,
            compatibility_payload: Some(CompatibilityRequestPayload::InviteClick(invite_request)),
            overload_active: false,
            feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
        };
        request.validate()?;
        Ok(request)
    }

    pub fn onboarding_continue(
        envelope_input: RuntimeRequestEnvelopeInput,
        platform_context: PlatformRuntimeContext,
        onboarding_request: AppOnboardingContinueRequest,
    ) -> Result<Self, ContractViolation> {
        revalidate_onboarding_continue_request(&onboarding_request)?;
        let request = Self {
            family: CanonicalIngressFamily::OnboardingContinueCompatibility,
            envelope_input,
            authorization_bearer: String::new(),
            actor_identity: onboarding_compatibility_actor_identity(&onboarding_request)?,
            device_identity: onboarding_compatibility_device_identity(
                &onboarding_request,
                platform_context.platform_type,
            )?,
            platform_context,
            session_hint: None,
            device_turn_sequence: None,
            session_resolve_mode: SessionResolveMode::ResolveOrOpen,
            modality: None,
            payload: None,
            compatibility_payload: Some(CompatibilityRequestPayload::OnboardingContinue(
                onboarding_request,
            )),
            overload_active: false,
            feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
        };
        request.validate()?;
        Ok(request)
    }

    pub fn compatibility(
        family: CanonicalIngressFamily,
        envelope_input: RuntimeRequestEnvelopeInput,
        authorization_bearer: String,
        actor_identity: UserId,
        device_identity: DeviceId,
        platform_context: PlatformRuntimeContext,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            family,
            envelope_input,
            authorization_bearer,
            actor_identity,
            device_identity,
            platform_context,
            session_hint: None,
            device_turn_sequence: None,
            session_resolve_mode: SessionResolveMode::ResolveOrOpen,
            modality: None,
            payload: None,
            compatibility_payload: None,
            overload_active: false,
            feature_flag_overrides: RuntimeFeatureFlagOverrides::default(),
        };
        request.validate()?;
        Ok(request)
    }

    fn to_foundation_request(
        &self,
    ) -> Result<RuntimeFoundationRequest, RuntimeRequestFoundationError> {
        Ok(RuntimeFoundationRequest {
            key: crate::runtime_request_foundation::RuntimeRouteKey::new(
                crate::runtime_request_foundation::RuntimeHttpMethod::Post,
                self.family.route_path(),
            )?,
            envelope_input: Some(self.envelope_input.clone()),
            overload_active: self.overload_active,
            feature_flag_overrides: self.feature_flag_overrides.clone(),
        })
    }

    fn executable_in_slice_2j(&self) -> bool {
        match self.family {
            CanonicalIngressFamily::VoiceTurn
            | CanonicalIngressFamily::InviteClickCompatibility => true,
            CanonicalIngressFamily::OnboardingContinueCompatibility => matches!(
                self.compatibility_payload.as_ref(),
                Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request))
                    if matches!(
                        &onboarding_request.action,
                        AppOnboardingContinueAction::AskMissingSubmit { .. }
                            | AppOnboardingContinueAction::PlatformSetupReceipt { .. }
                            | AppOnboardingContinueAction::TermsAccept { .. }
                            | AppOnboardingContinueAction::PrimaryDeviceConfirm { .. }
                            | AppOnboardingContinueAction::EmployeePhotoCaptureSend { .. }
                            | AppOnboardingContinueAction::EmployeeSenderVerifyCommit { .. }
                            | AppOnboardingContinueAction::VoiceEnrollLock { .. }
                            | AppOnboardingContinueAction::WakeEnrollStartDraft { .. }
                            | AppOnboardingContinueAction::WakeEnrollSampleCommit { .. }
                            | AppOnboardingContinueAction::WakeEnrollCompleteCommit { .. }
                            | AppOnboardingContinueAction::EmoPersonaLock
                            | AppOnboardingContinueAction::AccessProvisionCommit
                            | AppOnboardingContinueAction::CompleteCommit
                            | AppOnboardingContinueAction::PairingCompletionCommit { .. }
                    )
            ),
        }
    }

    fn compatibility_only_detail(&self) -> String {
        match self.compatibility_payload.as_ref() {
            Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) => {
                match &onboarding_request.action {
                    AppOnboardingContinueAction::PlatformSetupReceipt { .. } => {
                        "platform-setup onboarding compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::TermsAccept { .. } => {
                        "terms-accept onboarding compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::PrimaryDeviceConfirm { .. } => {
                        "primary-device-confirm onboarding compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::EmployeePhotoCaptureSend { .. } => {
                        "employee-photo sender-verification compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::EmployeeSenderVerifyCommit { .. } => {
                        "employee-sender-verify compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::VoiceEnrollLock { .. } => {
                        "voice-enroll onboarding compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::AskMissingSubmit { .. } => {
                        "onboarding ask-missing compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::WakeEnrollStartDraft { .. } => {
                        "wake-enroll start compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::WakeEnrollSampleCommit { .. } => {
                        "wake-enroll sample compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::WakeEnrollCompleteCommit { .. } => {
                        "wake-enroll complete compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::EmoPersonaLock => {
                        "emo-persona-lock compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::AccessProvisionCommit => {
                        "access-provision compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::WakeEnrollDeferCommit { .. } => {
                        "wake-enroll defer compatibility remains deferred after Slice 2O"
                            .to_string()
                    }
                    AppOnboardingContinueAction::CompleteCommit => {
                        "complete compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                    AppOnboardingContinueAction::PairingCompletionCommit { .. } => {
                        "pairing-completion compatibility is executable in Slice 2O and should not reach the compatibility-only boundary"
                            .to_string()
                    }
                }
            }
            Some(CompatibilityRequestPayload::InviteClick(_)) => {
                "invite-click compatibility is executable and should not reach the compatibility-only boundary"
                    .to_string()
            }
            None => "compatibility request is missing the bounded Slice 2C payload shape"
                .to_string(),
        }
    }
}

impl Validate for RuntimeCanonicalIngressRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.device_identity.validate()?;
        self.platform_context.validate()?;
        match self.family {
            CanonicalIngressFamily::VoiceTurn => {
                if self.authorization_bearer.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.authorization_bearer",
                        reason: "must not be empty",
                    });
                }
                if self.device_turn_sequence.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.device_turn_sequence",
                        reason: "voice turn requests require a device_turn_sequence",
                    });
                }
                if self.modality.is_none() || self.payload.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.payload",
                        reason: "voice turn requests require a modality and payload",
                    });
                }
                if self.compatibility_payload.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.compatibility_payload",
                        reason: "voice turn requests must not carry compatibility request state",
                    });
                }
            }
            CanonicalIngressFamily::InviteClickCompatibility => {
                if self.device_turn_sequence.is_some()
                    || self.modality.is_some()
                    || self.payload.is_some()
                    || self.session_hint.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request",
                        reason:
                            "compatibility-only family requests may not carry executable turn state",
                    });
                }
                if !matches!(self.session_resolve_mode, SessionResolveMode::ResolveOrOpen) {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.session_resolve_mode",
                        reason: "invite-click compatibility requests require ResolveOrOpen",
                    });
                }
                let Some(CompatibilityRequestPayload::InviteClick(invite_request)) =
                    self.compatibility_payload.as_ref()
                else {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.compatibility_payload",
                        reason:
                            "invite-click compatibility requests require invite-click request state",
                    });
                };
                invite_request.validate()?;
                if self.envelope_input.idempotency_key != invite_request.idempotency_key {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.envelope_input.idempotency_key",
                        reason: "must match invite-click request idempotency_key",
                    });
                }
                if self.platform_context.platform_type != invite_request.app_platform {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.platform_context.platform_type",
                        reason: "must match invite-click request app_platform",
                    });
                }
            }
            CanonicalIngressFamily::OnboardingContinueCompatibility => {
                if self.device_turn_sequence.is_some()
                    || self.modality.is_some()
                    || self.payload.is_some()
                    || self.session_hint.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request",
                        reason:
                            "compatibility-only family requests may not carry executable turn state",
                    });
                }
                if !matches!(self.session_resolve_mode, SessionResolveMode::ResolveOrOpen) {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.session_resolve_mode",
                        reason: "onboarding compatibility requests require ResolveOrOpen",
                    });
                }
                let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
                    self.compatibility_payload.as_ref()
                else {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.compatibility_payload",
                        reason:
                            "onboarding compatibility requests require bounded onboarding request state",
                    });
                };
                revalidate_onboarding_continue_request(onboarding_request)?;
                if self.envelope_input.idempotency_key != onboarding_request.idempotency_key {
                    return Err(ContractViolation::InvalidValue {
                        field: "runtime_canonical_ingress_request.envelope_input.idempotency_key",
                        reason: "must match onboarding request idempotency_key",
                    });
                }
            }
        }
        if !matches!(self.session_resolve_mode, SessionResolveMode::ResolveOrOpen)
            && self.session_hint.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_canonical_ingress_request.session_resolve_mode",
                reason: "resume or recover require a session_hint",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CanonicalTurnPayloadCarrier {
    Text {
        normalized_content_type: String,
        normalized_text: String,
    },
    Binary {
        normalized_content_type: String,
        byte_len: usize,
    },
    InviteClick {
        token_id: TokenId,
        token_signature: String,
        device_fingerprint: String,
        app_instance_id: String,
        deep_link_nonce: String,
        link_opened_at: MonotonicTimeNs,
    },
    OnboardingAskMissing {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        field_value: Option<String>,
    },
    OnboardingPlatformSetupReceipt {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        receipt_kind: String,
        receipt_ref: String,
        signer: String,
        payload_hash: String,
    },
    OnboardingTermsAccept {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        terms_version_id: String,
        accepted: bool,
    },
    OnboardingPrimaryDeviceConfirm {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
        proof_ok: bool,
    },
    OnboardingEmployeePhotoCaptureSend {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        photo_blob_ref: String,
    },
    OnboardingEmployeeSenderVerifyCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        decision: SenderVerifyDecision,
    },
    OnboardingVoiceEnrollLock {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
        sample_seed: String,
    },
    OnboardingWakeEnrollStartDraft {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
    },
    OnboardingWakeEnrollSampleCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
        sample_pass: bool,
    },
    OnboardingWakeEnrollCompleteCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
    },
    OnboardingEmoPersonaLock {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
    },
    OnboardingAccessProvisionCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
    },
    OnboardingCompleteCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
    },
    OnboardingPairingCompletionCommit {
        correlation_id: CorrelationId,
        onboarding_session_id: OnboardingSessionId,
        tenant_id: Option<String>,
        device_id: DeviceId,
        session_id: SessionId,
        session_attach_outcome: SessionAttachOutcome,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTurnRequestCarrier {
    pub canonical_route: &'static str,
    pub family: CanonicalIngressFamily,
    pub modality: CanonicalTurnModality,
    pub actor_identity: UserId,
    pub device_identity: DeviceId,
    pub platform: AppPlatform,
    pub requested_trigger: RuntimeEntryTrigger,
    pub session_hint: Option<SessionId>,
    pub device_turn_sequence: u64,
    pub session_resolve_mode: SessionResolveMode,
    pub request_content_hash: String,
    pub payload: CanonicalTurnPayloadCarrier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurnStartClassification {
    NewSessionOpenBypass,
    ExistingSessionAttached,
    ExistingSessionContinued,
    ExistingSessionResumed,
    ExistingSessionRecovered,
    InviteClickCompatibilityPrepared,
    OnboardingAskMissingCompatibilityPrepared,
    OnboardingPlatformSetupReceiptCompatibilityPrepared,
    OnboardingTermsAcceptCompatibilityPrepared,
    OnboardingPrimaryDeviceConfirmCompatibilityPrepared,
    OnboardingEmployeePhotoCaptureSendCompatibilityPrepared,
    OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared,
    OnboardingVoiceEnrollLockCompatibilityPrepared,
    OnboardingWakeEnrollStartDraftCompatibilityPrepared,
    OnboardingWakeEnrollSampleCommitCompatibilityPrepared,
    OnboardingWakeEnrollCompleteCommitCompatibilityPrepared,
    OnboardingEmoPersonaLockCompatibilityPrepared,
    OnboardingAccessProvisionCommitCompatibilityPrepared,
    OnboardingCompleteCommitCompatibilityPrepared,
    OnboardingPairingCompletionCommitCompatibilityPrepared,
    RetryReused,
    Deferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreAuthorityStage {
    IngressValidated,
    TriggerValidated,
    SessionResolved,
    EnvelopeCreated,
    TurnClassified,
    PreAuthorityReady,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreAuthorityStageRecord {
    pub stage: PreAuthorityStage,
    pub at_unix_ms: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreAuthorityOutcome {
    ReadyForSection04Boundary,
    RetryReused,
    Deferred,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalTurnStartResponse {
    pub request_id: String,
    pub trace_id: String,
    pub session_id: SessionId,
    pub turn_id: Option<TurnId>,
    pub session_state: SessionState,
    pub device_turn_sequence: Option<u64>,
    pub classification: TurnStartClassification,
    pub outcome: PreAuthorityOutcome,
    pub failure_class: Option<FailureClass>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnStartIdempotencyHook {
    pub idempotency_key: String,
    pub request_content_hash: String,
    pub durable_replay_available: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreAuthorityTurnHandoff {
    pub response: CanonicalTurnStartResponse,
    pub normalized_request: CanonicalTurnRequestCarrier,
    pub session_turn_permit: SessionTurnPermit,
    pub runtime_execution_envelope: RuntimeExecutionEnvelope,
    pub stage_history: Vec<PreAuthorityStageRecord>,
    pub idempotency_hook: TurnStartIdempotencyHook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreAuthorityTurnRetry {
    pub response: CanonicalTurnStartResponse,
    pub normalized_request: CanonicalTurnRequestCarrier,
    pub runtime_execution_envelope: RuntimeExecutionEnvelope,
    pub stage_history: Vec<PreAuthorityStageRecord>,
    pub idempotency_hook: TurnStartIdempotencyHook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimePreAuthorityTurnDeferred {
    pub response: CanonicalTurnStartResponse,
    pub normalized_request: CanonicalTurnRequestCarrier,
    pub stage_history: Vec<PreAuthorityStageRecord>,
    pub idempotency_hook: TurnStartIdempotencyHook,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimePreAuthorityTurnResult {
    Ready(RuntimePreAuthorityTurnHandoff),
    Retry(RuntimePreAuthorityTurnRetry),
    Deferred(RuntimePreAuthorityTurnDeferred),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeIngressTurnEventKind {
    TurnNormalized,
    PreAuthorityReady,
    RetryReused,
    TurnDeferred,
    TurnRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIngressTurnEvent {
    pub kind: RuntimeIngressTurnEventKind,
    pub request_id: String,
    pub route_path: String,
    pub session_id: Option<SessionId>,
    pub turn_id: Option<TurnId>,
    pub classification: Option<TurnStartClassification>,
    pub detail: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RuntimeIngressTurnCounters {
    pub normalized_turns: u64,
    pub ready_handoffs: u64,
    pub retries_reused: u64,
    pub turn_deferrals: u64,
    pub rejected_requests: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeIngressTurnError {
    pub reason_code: &'static str,
    pub failure_class: FailureClass,
    pub message: String,
    pub stage_history: Vec<PreAuthorityStageRecord>,
    pub request_envelope: Option<Box<RuntimeRequestEnvelopeFoundation>>,
    pub runtime_execution_envelope: Option<Box<RuntimeExecutionEnvelope>>,
}

impl RuntimeIngressTurnError {
    fn new(
        reason_code: &'static str,
        failure_class: FailureClass,
        message: impl Into<String>,
    ) -> Self {
        Self {
            reason_code,
            failure_class,
            message: message.into(),
            stage_history: Vec::new(),
            request_envelope: None,
            runtime_execution_envelope: None,
        }
    }

    fn with_stage_history(mut self, stage_history: &[PreAuthorityStageRecord]) -> Self {
        self.stage_history = stage_history.to_vec();
        self
    }

    fn with_request_envelope(
        mut self,
        request_envelope: Option<RuntimeRequestEnvelopeFoundation>,
    ) -> Self {
        self.request_envelope = request_envelope.map(Box::new);
        self
    }

    fn with_runtime_envelope(
        mut self,
        runtime_execution_envelope: RuntimeExecutionEnvelope,
    ) -> Self {
        self.runtime_execution_envelope = Some(Box::new(runtime_execution_envelope));
        self
    }

    fn rejecting(mut self, at_unix_ms: i64) -> Self {
        if !matches!(
            self.stage_history.last().map(|record| record.stage),
            Some(PreAuthorityStage::Rejected)
        ) {
            self.stage_history.push(PreAuthorityStageRecord {
                stage: PreAuthorityStage::Rejected,
                at_unix_ms,
            });
        }
        self
    }
}

#[derive(Debug)]
pub struct RuntimeIngressTurnFoundation {
    router: RuntimeRouter,
    events: Vec<RuntimeIngressTurnEvent>,
    counters: RuntimeIngressTurnCounters,
}

impl RuntimeIngressTurnFoundation {
    pub fn with_slice_2a_defaults(
        request_foundation_config: RuntimeRequestFoundationConfig,
    ) -> Result<Self, RuntimeRequestFoundationError> {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config)?;
        router.register_canonical_ingress_route(RuntimeRouteDefinition::canonical_turn()?)?;
        router.register_canonical_ingress_route(
            RuntimeRouteDefinition::invite_click_compatibility()?,
        )?;
        router.register_canonical_ingress_route(
            RuntimeRouteDefinition::onboarding_continue_compatibility()?,
        )?;
        Ok(Self {
            router,
            events: Vec::new(),
            counters: RuntimeIngressTurnCounters::default(),
        })
    }

    pub fn register_slice_2a_foundation_services<C, S>(
        container: &mut RuntimeServiceContainer<C, S>,
    ) -> Result<(), RuntimeBootstrapError>
    where
        C: RuntimeClock,
        S: RuntimeSecretsProvider,
    {
        container.register_service(
            "runtime_turn_request_normalizer",
            &[
                "runtime_route_registry",
                "runtime_request_security_foundation",
            ],
        )?;
        container.register_service(
            "runtime_turn_session_binder",
            &[
                "runtime_session_store",
                "runtime_session_turn_gate",
                "runtime_session_access_gate",
            ],
        )?;
        container.register_service(
            "runtime_turn_pre_authority_scaffold",
            &[
                "runtime_turn_request_normalizer",
                "runtime_turn_session_binder",
                "runtime_admission_controller",
            ],
        )?;
        container.register_service(
            "runtime_turn_observability",
            &["runtime_event_bus", "runtime_metrics_collector"],
        )?;
        container.register_service(
            "runtime_ingress_turn_foundation",
            &[
                "runtime_router",
                "runtime_turn_pre_authority_scaffold",
                "runtime_turn_observability",
            ],
        )?;
        Ok(())
    }

    pub fn section03_route_paths(&self) -> Vec<String> {
        self.router
            .route_keys()
            .into_iter()
            .filter(|key| key.path.starts_with("/v1/"))
            .map(|key| key.path.clone())
            .collect()
    }

    pub fn counters(&self) -> &RuntimeIngressTurnCounters {
        &self.counters
    }

    pub fn events(&self) -> &[RuntimeIngressTurnEvent] {
        &self.events
    }

    pub fn router(&self) -> &RuntimeRouter {
        &self.router
    }

    pub fn process_turn_start<C, S>(
        &mut self,
        runtime: &RuntimeProcess<C, S>,
        sessions: &mut RuntimeSessionFoundation,
        request: RuntimeCanonicalIngressRequest,
    ) -> Result<RuntimePreAuthorityTurnResult, RuntimeIngressTurnError>
    where
        C: RuntimeClock,
        S: RuntimeSecretsProvider,
    {
        request.validate().map_err(map_contract_violation)?;
        let prepared = self
            .router
            .prepare_request(
                runtime,
                request.to_foundation_request().map_err(map_request_error)?,
            )
            .map_err(map_request_error)?;
        let rejection_at_ms = prepared.prepared_at_ms;

        if !request.executable_in_slice_2j() {
            let compatibility_detail = request.compatibility_only_detail();
            self.counters.rejected_requests += 1;
            let request_id = prepared
                .envelope
                .as_ref()
                .map(|env| env.header().request_id.clone())
                .unwrap_or_else(|| request.envelope_input.request_id.clone());
            self.events.push(RuntimeIngressTurnEvent {
                kind: RuntimeIngressTurnEventKind::TurnRejected,
                request_id,
                route_path: prepared.definition.key.path.clone(),
                session_id: None,
                turn_id: None,
                classification: None,
                detail: compatibility_detail.clone(),
            });
            return Err(RuntimeIngressTurnError::new(
                reason_codes::INGRESS_COMPATIBILITY_ONLY,
                FailureClass::PolicyViolation,
                compatibility_detail,
            )
            .with_request_envelope(prepared.envelope)
            .rejecting(rejection_at_ms));
        }

        let mut stage_history = Vec::new();
        let normalized = normalize_turn_request(&request).map_err(|err| {
            err.with_request_envelope(prepared.envelope.clone())
                .with_stage_history(&stage_history)
                .rejecting(runtime.clock().now_unix_ms())
        })?;
        stage_history.push(PreAuthorityStageRecord {
            stage: PreAuthorityStage::IngressValidated,
            at_unix_ms: runtime.clock().now_unix_ms(),
        });
        self.counters.normalized_turns += 1;
        self.events.push(RuntimeIngressTurnEvent {
            kind: RuntimeIngressTurnEventKind::TurnNormalized,
            request_id: request.envelope_input.request_id.clone(),
            route_path: prepared.definition.key.path.clone(),
            session_id: request.session_hint,
            turn_id: None,
            classification: None,
            detail: normalized_event_detail(&normalized),
        });

        validate_trigger_posture(&prepared, &request).map_err(|err| {
            self.counters.rejected_requests += 1;
            self.events.push(RuntimeIngressTurnEvent {
                kind: RuntimeIngressTurnEventKind::TurnRejected,
                request_id: request.envelope_input.request_id.clone(),
                route_path: prepared.definition.key.path.clone(),
                session_id: request.session_hint,
                turn_id: None,
                classification: None,
                detail: "platform trigger posture was rejected".to_string(),
            });
            err.with_request_envelope(prepared.envelope.clone())
                .with_stage_history(&stage_history)
                .rejecting(runtime.clock().now_unix_ms())
        })?;
        stage_history.push(PreAuthorityStageRecord {
            stage: PreAuthorityStage::TriggerValidated,
            at_unix_ms: runtime.clock().now_unix_ms(),
        });

        let resolved = resolve_session_turn(sessions, &request, &normalized).map_err(|err| {
            self.counters.rejected_requests += 1;
            self.events.push(RuntimeIngressTurnEvent {
                kind: RuntimeIngressTurnEventKind::TurnRejected,
                request_id: request.envelope_input.request_id.clone(),
                route_path: prepared.definition.key.path.clone(),
                session_id: request.session_hint,
                turn_id: None,
                classification: None,
                detail: "session resolve-or-open failed closed".to_string(),
            });
            RuntimeIngressTurnError::new(err.reason_code, err.failure_class, err.message)
                .with_request_envelope(prepared.envelope.clone())
                .with_stage_history(&stage_history)
                .rejecting(runtime.clock().now_unix_ms())
        })?;
        stage_history.push(PreAuthorityStageRecord {
            stage: PreAuthorityStage::SessionResolved,
            at_unix_ms: runtime.clock().now_unix_ms(),
        });

        let idempotency_hook = TurnStartIdempotencyHook {
            idempotency_key: request.envelope_input.idempotency_key.clone(),
            request_content_hash: normalized.request_content_hash.clone(),
            durable_replay_available: false,
        };

        match resolved {
            ResolvedSessionTurn::Started {
                permit,
                classification,
                attach_outcome,
                session_state,
            } => {
                let mut runtime_execution_envelope = create_runtime_execution_envelope(
                    &prepared,
                    &request,
                    permit.session_id,
                    permit.turn_id,
                    Some(permit.device_turn_sequence),
                    attach_outcome,
                    AdmissionState::SessionResolved,
                )
                .map_err(|err| {
                    self.counters.rejected_requests += 1;
                    err.with_request_envelope(prepared.envelope.clone())
                        .with_stage_history(&stage_history)
                        .rejecting(runtime.clock().now_unix_ms())
                })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::EnvelopeCreated,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                runtime_execution_envelope = runtime_execution_envelope
                    .with_admission_state(AdmissionState::ExecutionAdmitted)
                    .map_err(|err| {
                        self.counters.rejected_requests += 1;
                        map_contract_violation(err)
                            .with_request_envelope(prepared.envelope.clone())
                            .with_stage_history(&stage_history)
                            .rejecting(runtime.clock().now_unix_ms())
                    })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::TurnClassified,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                validate_ready_invariants(
                    &prepared,
                    &normalized,
                    &runtime_execution_envelope,
                    &stage_history,
                    classification,
                )
                .map_err(|err| {
                    self.counters.rejected_requests += 1;
                    err.with_request_envelope(prepared.envelope.clone())
                        .with_runtime_envelope(runtime_execution_envelope.clone())
                        .with_stage_history(&stage_history)
                        .rejecting(runtime.clock().now_unix_ms())
                })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::PreAuthorityReady,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                self.counters.ready_handoffs += 1;
                self.events.push(RuntimeIngressTurnEvent {
                    kind: RuntimeIngressTurnEventKind::PreAuthorityReady,
                    request_id: request.envelope_input.request_id.clone(),
                    route_path: prepared.definition.key.path.clone(),
                    session_id: Some(permit.session_id),
                    turn_id: Some(permit.turn_id),
                    classification: Some(classification),
                    detail: pre_authority_ready_detail(&normalized),
                });
                Ok(RuntimePreAuthorityTurnResult::Ready(
                    RuntimePreAuthorityTurnHandoff {
                        response: CanonicalTurnStartResponse {
                            request_id: request.envelope_input.request_id.clone(),
                            trace_id: request.envelope_input.trace_id.clone(),
                            session_id: permit.session_id,
                            turn_id: Some(permit.turn_id),
                            session_state,
                            device_turn_sequence: Some(permit.device_turn_sequence),
                            classification,
                            outcome: PreAuthorityOutcome::ReadyForSection04Boundary,
                            failure_class: None,
                        },
                        normalized_request: normalized,
                        session_turn_permit: permit,
                        runtime_execution_envelope,
                        stage_history,
                        idempotency_hook,
                    },
                ))
            }
            ResolvedSessionTurn::Retry { projection } => {
                let mut runtime_execution_envelope =
                    create_runtime_execution_envelope_from_projection(
                        &prepared,
                        &request,
                        &projection,
                        AdmissionState::SessionResolved,
                    )
                    .map_err(|err| {
                        self.counters.rejected_requests += 1;
                        err.with_request_envelope(prepared.envelope.clone())
                            .with_stage_history(&stage_history)
                            .rejecting(runtime.clock().now_unix_ms())
                    })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::EnvelopeCreated,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                runtime_execution_envelope = runtime_execution_envelope
                    .with_admission_state(AdmissionState::ExecutionAdmitted)
                    .map_err(|err| {
                        self.counters.rejected_requests += 1;
                        map_contract_violation(err)
                            .with_request_envelope(prepared.envelope.clone())
                            .with_stage_history(&stage_history)
                            .rejecting(runtime.clock().now_unix_ms())
                    })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::TurnClassified,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                validate_ready_invariants(
                    &prepared,
                    &normalized,
                    &runtime_execution_envelope,
                    &stage_history,
                    TurnStartClassification::RetryReused,
                )
                .map_err(|err| {
                    self.counters.rejected_requests += 1;
                    err.with_request_envelope(prepared.envelope.clone())
                        .with_runtime_envelope(runtime_execution_envelope.clone())
                        .with_stage_history(&stage_history)
                        .rejecting(runtime.clock().now_unix_ms())
                })?;
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::PreAuthorityReady,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                self.counters.retries_reused += 1;
                self.events.push(RuntimeIngressTurnEvent {
                    kind: RuntimeIngressTurnEventKind::RetryReused,
                    request_id: request.envelope_input.request_id.clone(),
                    route_path: prepared.definition.key.path.clone(),
                    session_id: Some(projection.session_id),
                    turn_id: projection.turn_id,
                    classification: Some(TurnStartClassification::RetryReused),
                    detail: "retry reused the prior turn without entering authority".to_string(),
                });
                Ok(RuntimePreAuthorityTurnResult::Retry(
                    RuntimePreAuthorityTurnRetry {
                        response: CanonicalTurnStartResponse {
                            request_id: request.envelope_input.request_id.clone(),
                            trace_id: request.envelope_input.trace_id.clone(),
                            session_id: projection.session_id,
                            turn_id: projection.turn_id,
                            session_state: projection.session_state,
                            device_turn_sequence: projection.device_turn_sequence,
                            classification: TurnStartClassification::RetryReused,
                            outcome: PreAuthorityOutcome::RetryReused,
                            failure_class: None,
                        },
                        normalized_request: normalized,
                        runtime_execution_envelope,
                        stage_history,
                        idempotency_hook,
                    },
                ))
            }
            ResolvedSessionTurn::Deferred {
                deferred,
                session_state,
            } => {
                stage_history.push(PreAuthorityStageRecord {
                    stage: PreAuthorityStage::TurnClassified,
                    at_unix_ms: runtime.clock().now_unix_ms(),
                });
                validate_deferred_invariants(&prepared, &normalized, &stage_history).map_err(
                    |err| {
                        self.counters.rejected_requests += 1;
                        err.with_request_envelope(prepared.envelope.clone())
                            .with_stage_history(&stage_history)
                            .rejecting(runtime.clock().now_unix_ms())
                    },
                )?;
                self.counters.turn_deferrals += 1;
                self.events.push(RuntimeIngressTurnEvent {
                    kind: RuntimeIngressTurnEventKind::TurnDeferred,
                    request_id: request.envelope_input.request_id.clone(),
                    route_path: prepared.definition.key.path.clone(),
                    session_id: Some(deferred.session_id),
                    turn_id: None,
                    classification: Some(TurnStartClassification::Deferred),
                    detail: "turn stopped at the session single-writer gate before authority"
                        .to_string(),
                });
                Ok(RuntimePreAuthorityTurnResult::Deferred(
                    RuntimePreAuthorityTurnDeferred {
                        response: CanonicalTurnStartResponse {
                            request_id: request.envelope_input.request_id.clone(),
                            trace_id: request.envelope_input.trace_id.clone(),
                            session_id: deferred.session_id,
                            turn_id: None,
                            session_state,
                            device_turn_sequence: Some(deferred.device_turn_sequence),
                            classification: TurnStartClassification::Deferred,
                            outcome: PreAuthorityOutcome::Deferred,
                            failure_class: Some(FailureClass::RetryableRuntime),
                        },
                        normalized_request: normalized,
                        stage_history,
                        idempotency_hook,
                    },
                ))
            }
        }
    }
}

#[derive(Debug)]
enum ResolvedSessionTurn {
    Started {
        permit: SessionTurnPermit,
        classification: TurnStartClassification,
        attach_outcome: Option<SessionAttachOutcome>,
        session_state: SessionState,
    },
    Retry {
        projection: SessionRuntimeProjection,
    },
    Deferred {
        deferred: SessionTurnDeferred,
        session_state: SessionState,
    },
}

fn normalize_turn_request(
    request: &RuntimeCanonicalIngressRequest,
) -> Result<CanonicalTurnRequestCarrier, RuntimeIngressTurnError> {
    match request.family {
        CanonicalIngressFamily::VoiceTurn => normalize_executable_turn_request(request),
        CanonicalIngressFamily::InviteClickCompatibility => normalize_invite_click_request(request),
        CanonicalIngressFamily::OnboardingContinueCompatibility => {
            normalize_onboarding_continue_request(request)
        }
    }
}

fn normalize_executable_turn_request(
    request: &RuntimeCanonicalIngressRequest,
) -> Result<CanonicalTurnRequestCarrier, RuntimeIngressTurnError> {
    validate_authorization_header(&request.authorization_bearer)?;
    let modality = request.modality.ok_or_else(|| {
        RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "canonical turn requests require an executable modality",
        )
    })?;
    let device_turn_sequence = request.device_turn_sequence.ok_or_else(|| {
        RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "canonical turn requests require a device_turn_sequence",
        )
    })?;
    let payload = request.payload.as_ref().ok_or_else(|| {
        RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "canonical turn requests require a payload",
        )
    })?;

    match (modality, payload) {
        (CanonicalTurnModality::Text, RawTurnPayload::Text { content_type, text }) => {
            let normalized_content_type =
                normalize_content_type(content_type).ok_or_else(|| {
                    RuntimeIngressTurnError::new(
                        reason_codes::INGRESS_UNSUPPORTED_CONTENT_TYPE,
                        FailureClass::InvalidPayload,
                        "text turns require a supported text content type",
                    )
                })?;
            if normalized_content_type != "text/plain" {
                return Err(RuntimeIngressTurnError::new(
                    reason_codes::INGRESS_UNSUPPORTED_CONTENT_TYPE,
                    FailureClass::InvalidPayload,
                    "text turns require text/plain",
                ));
            }
            let normalized_text = text.trim().to_string();
            if normalized_text.is_empty() {
                return Err(RuntimeIngressTurnError::new(
                    reason_codes::INGRESS_PAYLOAD_INVALID,
                    FailureClass::InvalidPayload,
                    "text turns must not be empty after normalization",
                ));
            }
            if normalized_text.len() > MAX_TEXT_BYTES {
                return Err(RuntimeIngressTurnError::new(
                    reason_codes::INGRESS_PAYLOAD_INVALID,
                    FailureClass::InvalidPayload,
                    "text turns exceeded the bounded Slice 2A payload size",
                ));
            }
            let request_content_hash = canonical_content_hash(
                modality.as_str(),
                normalized_content_type.as_bytes(),
                normalized_text.as_bytes(),
            );
            Ok(CanonicalTurnRequestCarrier {
                canonical_route: CANONICAL_TURN_ENDPOINT_PATH,
                family: request.family,
                modality,
                actor_identity: request.actor_identity.clone(),
                device_identity: request.device_identity.clone(),
                platform: request.platform_context.platform_type,
                requested_trigger: request.platform_context.requested_trigger,
                session_hint: request.session_hint,
                device_turn_sequence,
                session_resolve_mode: request.session_resolve_mode,
                request_content_hash,
                payload: CanonicalTurnPayloadCarrier::Text {
                    normalized_content_type,
                    normalized_text,
                },
            })
        }
        (
            CanonicalTurnModality::Voice
            | CanonicalTurnModality::File
            | CanonicalTurnModality::Image
            | CanonicalTurnModality::Camera,
            RawTurnPayload::Binary {
                content_type,
                bytes,
            },
        ) => {
            let normalized_content_type =
                normalize_content_type(content_type).ok_or_else(|| {
                    RuntimeIngressTurnError::new(
                        reason_codes::INGRESS_UNSUPPORTED_CONTENT_TYPE,
                        FailureClass::InvalidPayload,
                        "binary turns require a supported content type",
                    )
                })?;
            validate_binary_content_type(modality, &normalized_content_type)?;
            if bytes.is_empty() {
                return Err(RuntimeIngressTurnError::new(
                    reason_codes::INGRESS_PAYLOAD_INVALID,
                    FailureClass::InvalidPayload,
                    "binary turns must not be empty",
                ));
            }
            if bytes.len() > MAX_BINARY_BYTES {
                return Err(RuntimeIngressTurnError::new(
                    reason_codes::INGRESS_PAYLOAD_INVALID,
                    FailureClass::InvalidPayload,
                    "binary turns exceeded the bounded Slice 2A payload size",
                ));
            }
            let request_content_hash = canonical_content_hash(
                modality.as_str(),
                normalized_content_type.as_bytes(),
                bytes,
            );
            Ok(CanonicalTurnRequestCarrier {
                canonical_route: CANONICAL_TURN_ENDPOINT_PATH,
                family: request.family,
                modality,
                actor_identity: request.actor_identity.clone(),
                device_identity: request.device_identity.clone(),
                platform: request.platform_context.platform_type,
                requested_trigger: request.platform_context.requested_trigger,
                session_hint: request.session_hint,
                device_turn_sequence,
                session_resolve_mode: request.session_resolve_mode,
                request_content_hash,
                payload: CanonicalTurnPayloadCarrier::Binary {
                    normalized_content_type,
                    byte_len: bytes.len(),
                },
            })
        }
        _ => Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "the provided payload carrier does not match the executable modality",
        )),
    }
}

fn normalize_invite_click_request(
    request: &RuntimeCanonicalIngressRequest,
) -> Result<CanonicalTurnRequestCarrier, RuntimeIngressTurnError> {
    let Some(CompatibilityRequestPayload::InviteClick(invite_request)) =
        request.compatibility_payload.as_ref()
    else {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "invite-click execution requires a bounded invite-click request shape",
        ));
    };

    let request_content_hash = canonical_invite_click_hash(invite_request);
    let device_turn_sequence =
        compatibility_device_turn_sequence(request.device_identity.as_str(), invite_request);
    Ok(CanonicalTurnRequestCarrier {
        canonical_route: INVITE_CLICK_ENDPOINT_PATH,
        family: request.family,
        modality: CanonicalTurnModality::Compatibility,
        actor_identity: request.actor_identity.clone(),
        device_identity: request.device_identity.clone(),
        platform: request.platform_context.platform_type,
        requested_trigger: request.platform_context.requested_trigger,
        session_hint: request.session_hint,
        device_turn_sequence,
        session_resolve_mode: request.session_resolve_mode,
        request_content_hash,
        payload: CanonicalTurnPayloadCarrier::InviteClick {
            token_id: invite_request.token_id.clone(),
            token_signature: invite_request.token_signature.clone(),
            device_fingerprint: invite_request.device_fingerprint.clone(),
            app_instance_id: invite_request.app_instance_id.clone(),
            deep_link_nonce: invite_request.deep_link_nonce.clone(),
            link_opened_at: invite_request.link_opened_at,
        },
    })
}

fn normalize_onboarding_continue_request(
    request: &RuntimeCanonicalIngressRequest,
) -> Result<CanonicalTurnRequestCarrier, RuntimeIngressTurnError> {
    let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
        request.compatibility_payload.as_ref()
    else {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_PAYLOAD_INVALID,
            FailureClass::InvalidPayload,
            "onboarding compatibility execution requires a bounded onboarding request shape",
        ));
    };
    let (request_content_hash, payload) = match &onboarding_request.action {
        AppOnboardingContinueAction::AskMissingSubmit { field_value } => (
            canonical_onboarding_ask_missing_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingAskMissing {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                field_value: field_value.clone(),
            },
        ),
        AppOnboardingContinueAction::PlatformSetupReceipt {
            receipt_kind,
            receipt_ref,
            signer,
            payload_hash,
        } => (
            canonical_onboarding_platform_setup_receipt_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingPlatformSetupReceipt {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                receipt_kind: receipt_kind.clone(),
                receipt_ref: receipt_ref.clone(),
                signer: signer.clone(),
                payload_hash: payload_hash.clone(),
            },
        ),
        AppOnboardingContinueAction::TermsAccept {
            terms_version_id,
            accepted,
        } => (
            canonical_onboarding_terms_accept_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingTermsAccept {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                terms_version_id: terms_version_id.clone(),
                accepted: *accepted,
            },
        ),
        AppOnboardingContinueAction::PrimaryDeviceConfirm {
            device_id,
            proof_ok,
        } => (
            canonical_onboarding_primary_device_confirm_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingPrimaryDeviceConfirm {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
                proof_ok: *proof_ok,
            },
        ),
        AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } => (
            canonical_onboarding_employee_photo_capture_send_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingEmployeePhotoCaptureSend {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                photo_blob_ref: photo_blob_ref.clone(),
            },
        ),
        AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision } => (
            canonical_onboarding_employee_sender_verify_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingEmployeeSenderVerifyCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                decision: *decision,
            },
        ),
        AppOnboardingContinueAction::VoiceEnrollLock {
            device_id,
            sample_seed,
        } => (
            canonical_onboarding_voice_enroll_lock_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingVoiceEnrollLock {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
                sample_seed: sample_seed.clone(),
            },
        ),
        AppOnboardingContinueAction::WakeEnrollStartDraft { device_id } => (
            canonical_onboarding_wake_enroll_start_draft_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollStartDraft {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
            },
        ),
        AppOnboardingContinueAction::WakeEnrollSampleCommit {
            device_id,
            sample_pass,
        } => (
            canonical_onboarding_wake_enroll_sample_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollSampleCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
                sample_pass: *sample_pass,
            },
        ),
        AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id } => (
            canonical_onboarding_wake_enroll_complete_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollCompleteCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
            },
        ),
        AppOnboardingContinueAction::EmoPersonaLock => (
            canonical_onboarding_emo_persona_lock_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingEmoPersonaLock {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
            },
        ),
        AppOnboardingContinueAction::AccessProvisionCommit => (
            canonical_onboarding_access_provision_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingAccessProvisionCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
            },
        ),
        AppOnboardingContinueAction::CompleteCommit => (
            canonical_onboarding_complete_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingCompleteCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
            },
        ),
        AppOnboardingContinueAction::PairingCompletionCommit {
            device_id,
            session_id,
            session_attach_outcome,
        } => (
            canonical_onboarding_pairing_completion_commit_hash(onboarding_request),
            CanonicalTurnPayloadCarrier::OnboardingPairingCompletionCommit {
                correlation_id: onboarding_request.correlation_id,
                onboarding_session_id: onboarding_request.onboarding_session_id.clone(),
                tenant_id: onboarding_request.tenant_id.clone(),
                device_id: device_id.clone(),
                session_id: *session_id,
                session_attach_outcome: *session_attach_outcome,
            },
        ),
        _ => {
            return Err(RuntimeIngressTurnError::new(
                reason_codes::INGRESS_COMPATIBILITY_ONLY,
                FailureClass::PolicyViolation,
                "only onboarding ask-missing, platform-setup, terms-accept, primary-device-confirm, employee-photo-capture-send, employee-sender-verify, voice-enroll-lock, wake-enroll-start, wake-enroll-sample, wake-enroll-complete, emo-persona-lock, access-provision, complete, and pairing-completion compatibility are executable in Slice 2O",
            ))
        }
    };
    let device_turn_sequence = onboarding_compatibility_device_turn_sequence(
        request.device_identity.as_str(),
        onboarding_request,
    );
    Ok(CanonicalTurnRequestCarrier {
        canonical_route: ONBOARDING_CONTINUE_ENDPOINT_PATH,
        family: request.family,
        modality: CanonicalTurnModality::Compatibility,
        actor_identity: request.actor_identity.clone(),
        device_identity: request.device_identity.clone(),
        platform: request.platform_context.platform_type,
        requested_trigger: request.platform_context.requested_trigger,
        session_hint: request.session_hint,
        device_turn_sequence,
        session_resolve_mode: request.session_resolve_mode,
        request_content_hash,
        payload,
    })
}

// Invite-click is pre-authority compatibility execution, so these anchors must
// be deterministic without claiming a pre-known authenticated actor.
fn compatibility_actor_identity(
    invite_request: &InviteOpenActivateCommitRequest,
) -> Result<UserId, ContractViolation> {
    let anchor_material = format!(
        "token_id={}|device_fingerprint={}|app_instance_id={}",
        invite_request.token_id.as_str(),
        invite_request.device_fingerprint,
        invite_request.app_instance_id,
    );
    UserId::new(format!(
        "invite-compat-actor:{}",
        canonical_content_hash(
            "INVITE_CLICK_COMPAT_ACTOR",
            invite_request.token_id.as_str().as_bytes(),
            anchor_material.as_bytes(),
        )
    ))
}

fn compatibility_device_identity(
    invite_request: &InviteOpenActivateCommitRequest,
) -> Result<DeviceId, ContractViolation> {
    let anchor_material = format!(
        "app_platform={}|device_fingerprint={}|app_instance_id={}",
        invite_request.app_platform.as_str(),
        invite_request.device_fingerprint,
        invite_request.app_instance_id,
    );
    DeviceId::new(format!(
        "invite-compat-device:{}",
        canonical_content_hash(
            "INVITE_CLICK_COMPAT_DEVICE",
            invite_request.app_platform.as_str().as_bytes(),
            anchor_material.as_bytes(),
        )
    ))
}

fn onboarding_compatibility_actor_identity(
    onboarding_request: &AppOnboardingContinueRequest,
) -> Result<UserId, ContractViolation> {
    let tenant_anchor = onboarding_request
        .tenant_id
        .as_deref()
        .unwrap_or("tenant:none");
    let anchor_material = format!(
        "onboarding_session_id={}|tenant={tenant_anchor}",
        onboarding_request.onboarding_session_id.as_str(),
    );
    UserId::new(format!(
        "onboarding-compat-actor:{}",
        canonical_content_hash(
            "ONBOARDING_ASK_MISSING_COMPAT_ACTOR",
            onboarding_request.onboarding_session_id.as_str().as_bytes(),
            anchor_material.as_bytes(),
        )
    ))
}

fn onboarding_compatibility_device_identity(
    onboarding_request: &AppOnboardingContinueRequest,
    platform: AppPlatform,
) -> Result<DeviceId, ContractViolation> {
    let tenant_anchor = onboarding_request
        .tenant_id
        .as_deref()
        .unwrap_or("tenant:none");
    let anchor_material = format!(
        "platform={}|onboarding_session_id={}|tenant={tenant_anchor}",
        platform.as_str(),
        onboarding_request.onboarding_session_id.as_str(),
    );
    DeviceId::new(format!(
        "onboarding-compat-device:{}",
        canonical_content_hash(
            "ONBOARDING_ASK_MISSING_COMPAT_DEVICE",
            platform.as_str().as_bytes(),
            anchor_material.as_bytes(),
        )
    ))
}

fn normalized_event_detail(normalized: &CanonicalTurnRequestCarrier) -> String {
    match &normalized.payload {
        CanonicalTurnPayloadCarrier::Text { .. } | CanonicalTurnPayloadCarrier::Binary { .. } => {
            format!(
            "normalized {} turn into the canonical /v1/voice/turn carrier",
            normalized.modality.as_str()
            )
        }
        CanonicalTurnPayloadCarrier::InviteClick { .. } => {
            "normalized invite-click compatibility into the bounded /v1/invite/click carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingAskMissing { .. } => {
            "normalized onboarding ask-missing compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPlatformSetupReceipt { .. } => {
            "normalized onboarding platform-setup receipt compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingTermsAccept { .. } => {
            "normalized onboarding terms-accept compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPrimaryDeviceConfirm { .. } => {
            "normalized onboarding primary-device-confirm compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeePhotoCaptureSend { .. } => {
            "normalized onboarding employee-photo-capture-send compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeeSenderVerifyCommit { .. } => {
            "normalized onboarding employee-sender-verify compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingVoiceEnrollLock { .. } => {
            "normalized onboarding voice-enroll-lock compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollStartDraft { .. } => {
            "normalized onboarding wake-enroll-start compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollSampleCommit { .. } => {
            "normalized onboarding wake-enroll-sample compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollCompleteCommit { .. } => {
            "normalized onboarding wake-enroll-complete compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmoPersonaLock { .. } => {
            "normalized onboarding emo-persona-lock compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingAccessProvisionCommit { .. } => {
            "normalized onboarding access-provision compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingCompleteCommit { .. } => {
            "normalized onboarding complete compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPairingCompletionCommit { .. } => {
            "normalized onboarding pairing-completion compatibility into the bounded /v1/onboarding/continue carrier"
                .to_string()
        }
    }
}

fn canonical_invite_click_hash(invite_request: &InviteOpenActivateCommitRequest) -> String {
    let shape = format!(
        "token_id={}|token_signature={}|device_fingerprint={}|app_platform={}|app_instance_id={}|deep_link_nonce={}|link_opened_at={}|idempotency_key={}",
        invite_request.token_id.as_str(),
        invite_request.token_signature,
        invite_request.device_fingerprint,
        invite_request.app_platform.as_str(),
        invite_request.app_instance_id,
        invite_request.deep_link_nonce,
        invite_request.link_opened_at.0,
        invite_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        INVITE_CLICK_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_ask_missing_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::AskMissingSubmit { field_value } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires ask-missing action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|field_value={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        field_value.as_deref().unwrap_or(""),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_platform_setup_receipt_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::PlatformSetupReceipt {
        receipt_kind,
        receipt_ref,
        signer,
        payload_hash,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires platform-setup receipt action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|receipt_kind={}|receipt_ref={}|signer={}|payload_hash={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        receipt_kind,
        receipt_ref,
        signer,
        payload_hash,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_terms_accept_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::TermsAccept {
        terms_version_id,
        accepted,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires terms-accept action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|terms_version_id={}|accepted={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        terms_version_id,
        accepted,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_primary_device_confirm_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::PrimaryDeviceConfirm {
        device_id,
        proof_ok,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires primary-device-confirm action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|proof_ok={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        proof_ok,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_employee_photo_capture_send_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } =
        &onboarding_request.action
    else {
        unreachable!(
            "selected onboarding normalization requires employee-photo-capture-send action"
        );
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|photo_blob_ref={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        photo_blob_ref,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn sender_verify_decision_token(decision: SenderVerifyDecision) -> &'static str {
    match decision {
        SenderVerifyDecision::Confirm => "CONFIRM",
        SenderVerifyDecision::Reject => "REJECT",
    }
}

fn canonical_onboarding_employee_sender_verify_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision } =
        &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires employee-sender-verify action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|decision={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        sender_verify_decision_token(*decision),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_voice_enroll_lock_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::VoiceEnrollLock {
        device_id,
        sample_seed,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires voice-enroll-lock action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|sample_seed={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        sample_seed,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_wake_enroll_start_draft_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::WakeEnrollStartDraft { device_id } =
        &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires wake-enroll-start action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_wake_enroll_sample_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::WakeEnrollSampleCommit {
        device_id,
        sample_pass,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires wake-enroll-sample action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|sample_pass={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        sample_pass,
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_wake_enroll_complete_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id } =
        &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires wake-enroll-complete action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_emo_persona_lock_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::EmoPersonaLock = &onboarding_request.action else {
        unreachable!("selected onboarding normalization requires emo-persona-lock action");
    };
    let shape = format!(
        "correlation_id={}|onboarding_session_id={}|tenant_id={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_access_provision_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::AccessProvisionCommit = &onboarding_request.action else {
        unreachable!("selected onboarding normalization requires access-provision action");
    };
    let shape = format!(
        "action=access_provision_commit|correlation_id={}|onboarding_session_id={}|tenant_id={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_complete_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::CompleteCommit = &onboarding_request.action else {
        unreachable!("selected onboarding normalization requires complete action");
    };
    let shape = format!(
        "action=complete_commit|correlation_id={}|onboarding_session_id={}|tenant_id={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn canonical_onboarding_pairing_completion_commit_hash(
    onboarding_request: &AppOnboardingContinueRequest,
) -> String {
    let AppOnboardingContinueAction::PairingCompletionCommit {
        device_id,
        session_id,
        session_attach_outcome,
    } = &onboarding_request.action
    else {
        unreachable!("selected onboarding normalization requires pairing-completion action");
    };
    let shape = format!(
        "action=pairing_completion_commit|correlation_id={}|onboarding_session_id={}|tenant_id={}|device_id={}|session_id={}|session_attach_outcome={}|idempotency_key={}",
        onboarding_request.correlation_id.0,
        onboarding_request.onboarding_session_id.as_str(),
        onboarding_request.tenant_id.as_deref().unwrap_or(""),
        device_id.as_str(),
        session_id.0,
        session_attach_outcome.as_str(),
        onboarding_request.idempotency_key,
    );
    canonical_content_hash(
        CanonicalTurnModality::Compatibility.as_str(),
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes(),
        shape.as_bytes(),
    )
}

fn compatibility_device_turn_sequence(
    device_id: &str,
    invite_request: &InviteOpenActivateCommitRequest,
) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for component in [
        INVITE_CLICK_ENDPOINT_PATH.as_bytes(),
        device_id.as_bytes(),
        invite_request.token_id.as_str().as_bytes(),
        invite_request.token_signature.as_bytes(),
        invite_request.device_fingerprint.as_bytes(),
        invite_request.app_instance_id.as_bytes(),
        invite_request.deep_link_nonce.as_bytes(),
        invite_request.idempotency_key.as_bytes(),
    ] {
        for byte in component {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= u64::from(b'|');
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let link_opened_at = invite_request.link_opened_at.0.to_le_bytes();
    for byte in link_opened_at {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn onboarding_compatibility_device_turn_sequence(
    device_id: &str,
    onboarding_request: &AppOnboardingContinueRequest,
) -> u64 {
    let correlation_id = onboarding_request.correlation_id.0.to_string();
    let mut hash = 0xcbf29ce484222325u64;
    let action_components: Vec<Vec<u8>> = match &onboarding_request.action {
        AppOnboardingContinueAction::AskMissingSubmit { field_value } => vec![
            b"ASK_MISSING_SUBMIT".to_vec(),
            field_value.as_deref().unwrap_or("").as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::PlatformSetupReceipt {
            receipt_kind,
            receipt_ref,
            signer,
            payload_hash,
        } => vec![
            b"PLATFORM_SETUP_RECEIPT".to_vec(),
            receipt_kind.as_bytes().to_vec(),
            receipt_ref.as_bytes().to_vec(),
            signer.as_bytes().to_vec(),
            payload_hash.as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::TermsAccept {
            terms_version_id,
            accepted,
        } => vec![
            b"TERMS_ACCEPT".to_vec(),
            terms_version_id.as_bytes().to_vec(),
            if *accepted {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            },
        ],
        AppOnboardingContinueAction::PrimaryDeviceConfirm {
            device_id,
            proof_ok,
        } => vec![
            b"PRIMARY_DEVICE_CONFIRM".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
            if *proof_ok {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            },
        ],
        AppOnboardingContinueAction::EmployeePhotoCaptureSend { photo_blob_ref } => vec![
            b"EMPLOYEE_PHOTO_CAPTURE_SEND".to_vec(),
            photo_blob_ref.as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::EmployeeSenderVerifyCommit { decision } => vec![
            b"EMPLOYEE_SENDER_VERIFY_COMMIT".to_vec(),
            sender_verify_decision_token(*decision).as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::VoiceEnrollLock {
            device_id,
            sample_seed,
        } => vec![
            b"VOICE_ENROLL_LOCK".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
            sample_seed.as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::WakeEnrollStartDraft { device_id } => vec![
            b"WAKE_ENROLL_START_DRAFT".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::WakeEnrollSampleCommit {
            device_id,
            sample_pass,
        } => vec![
            b"WAKE_ENROLL_SAMPLE_COMMIT".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
            if *sample_pass {
                b"true".to_vec()
            } else {
                b"false".to_vec()
            },
        ],
        AppOnboardingContinueAction::WakeEnrollCompleteCommit { device_id } => vec![
            b"WAKE_ENROLL_COMPLETE_COMMIT".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
        ],
        AppOnboardingContinueAction::EmoPersonaLock => vec![b"EMO_PERSONA_LOCK".to_vec()],
        AppOnboardingContinueAction::AccessProvisionCommit => {
            vec![b"ACCESS_PROVISION_COMMIT".to_vec()]
        }
        AppOnboardingContinueAction::CompleteCommit => vec![b"COMPLETE_COMMIT".to_vec()],
        AppOnboardingContinueAction::PairingCompletionCommit {
            device_id,
            session_id,
            session_attach_outcome,
        } => vec![
            b"PAIRING_COMPLETION_COMMIT".to_vec(),
            device_id.as_str().as_bytes().to_vec(),
            session_id.0.to_string().as_bytes().to_vec(),
            session_attach_outcome.as_str().as_bytes().to_vec(),
        ],
        _ => unreachable!(
            "selected onboarding device-turn sequence requires an executable Slice 2O action"
        ),
    };
    let mut components = vec![
        ONBOARDING_CONTINUE_ENDPOINT_PATH.as_bytes().to_vec(),
        device_id.as_bytes().to_vec(),
        correlation_id.as_bytes().to_vec(),
        onboarding_request
            .onboarding_session_id
            .as_str()
            .as_bytes()
            .to_vec(),
        onboarding_request.idempotency_key.as_bytes().to_vec(),
        onboarding_request
            .tenant_id
            .as_deref()
            .unwrap_or("")
            .as_bytes()
            .to_vec(),
    ];
    components.extend(action_components);
    for component in components {
        for byte in component {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        hash ^= u64::from(b'|');
        hash = hash.wrapping_mul(0x100000001b3);
    }
    if hash == 0 {
        1
    } else {
        hash
    }
}

fn revalidate_onboarding_continue_request(
    onboarding_request: &AppOnboardingContinueRequest,
) -> Result<(), ContractViolation> {
    AppOnboardingContinueRequest::v1(
        onboarding_request.correlation_id,
        onboarding_request.onboarding_session_id.clone(),
        onboarding_request.idempotency_key.clone(),
        onboarding_request.tenant_id.clone(),
        onboarding_request.action.clone(),
    )
    .map(|_| ())
}

fn validate_stage4_optional_ref(
    field: &'static str,
    value: Option<&str>,
) -> Result<(), ContractViolation> {
    if let Some(value) = value {
        validate_stage4_ref(field, value)?;
    }
    Ok(())
}

fn validate_stage4_ref(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    let value = value.trim();
    if value.is_empty() || value.len() > 128 || !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be bounded non-empty ASCII",
        });
    }
    Ok(())
}

fn validate_authorization_header(value: &str) -> Result<(), RuntimeIngressTurnError> {
    if value.len() > MAX_AUTHORIZATION_LEN || !value.is_ascii() {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_AUTHORIZATION_INVALID,
            FailureClass::AuthenticationFailure,
            "authorization bearer token must be bounded ASCII",
        ));
    }
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_AUTHORIZATION_INVALID,
            FailureClass::AuthenticationFailure,
            "authorization header must start with 'Bearer '",
        ));
    };
    if token.trim().is_empty() {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_AUTHORIZATION_INVALID,
            FailureClass::AuthenticationFailure,
            "authorization bearer token must not be empty",
        ));
    }
    Ok(())
}

fn normalize_content_type(value: &str) -> Option<String> {
    let normalized = value.trim().split(';').next()?.trim().to_ascii_lowercase();
    (!normalized.is_empty() && normalized.is_ascii()).then_some(normalized)
}

fn validate_binary_content_type(
    modality: CanonicalTurnModality,
    normalized_content_type: &str,
) -> Result<(), RuntimeIngressTurnError> {
    let allowed = match modality {
        CanonicalTurnModality::Voice => normalized_content_type.starts_with("audio/"),
        CanonicalTurnModality::File => {
            normalized_content_type == "application/pdf"
                || normalized_content_type == "application/octet-stream"
        }
        CanonicalTurnModality::Image => normalized_content_type.starts_with("image/"),
        CanonicalTurnModality::Camera => {
            normalized_content_type.starts_with("image/")
                || normalized_content_type.starts_with("video/")
        }
        CanonicalTurnModality::Text => false,
        CanonicalTurnModality::Compatibility => false,
    };
    if allowed {
        Ok(())
    } else {
        Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_UNSUPPORTED_CONTENT_TYPE,
            FailureClass::InvalidPayload,
            "the executable modality rejected the supplied content type",
        ))
    }
}

fn canonical_content_hash(modality: &str, content_type: &[u8], content: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in modality.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= u64::from(b'|');
    hash = hash.wrapping_mul(0x100000001b3);
    for byte in content_type {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash ^= u64::from(b'|');
    hash = hash.wrapping_mul(0x100000001b3);
    for byte in content {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn validate_trigger_posture(
    prepared: &RuntimePreparedRequest,
    request: &RuntimeCanonicalIngressRequest,
) -> Result<(), RuntimeIngressTurnError> {
    let Some(envelope) = prepared.envelope.as_ref() else {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_ENVELOPE_INVALID,
            FailureClass::InvalidPayload,
            "canonical turn routes require the accepted request foundation envelope",
        ));
    };
    let origin = envelope.origin();
    if origin.platform != request.platform_context.platform_type {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_TRIGGER_INVALID,
            FailureClass::PolicyViolation,
            "platform origin and platform runtime context disagreed",
        ));
    }
    if origin.trigger != request.platform_context.requested_trigger {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_TRIGGER_INVALID,
            FailureClass::PolicyViolation,
            "request origin trigger and platform runtime trigger disagreed",
        ));
    }
    if !request.platform_context.trigger_allowed {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_TRIGGER_INVALID,
            FailureClass::PolicyViolation,
            "platform runtime context rejected the requested trigger",
        ));
    }
    if matches!(
        request.family,
        CanonicalIngressFamily::InviteClickCompatibility
    ) && request.platform_context.requested_trigger != RuntimeEntryTrigger::Explicit
    {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_TRIGGER_INVALID,
            FailureClass::PolicyViolation,
            "invite-click compatibility requests require EXPLICIT trigger posture",
        ));
    }
    Ok(())
}

fn compatibility_prepared_classification(
    normalized: &CanonicalTurnRequestCarrier,
) -> Option<TurnStartClassification> {
    match &normalized.payload {
        CanonicalTurnPayloadCarrier::InviteClick { .. } => {
            Some(TurnStartClassification::InviteClickCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingAskMissing { .. } => {
            Some(TurnStartClassification::OnboardingAskMissingCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingPlatformSetupReceipt { .. } => {
            Some(TurnStartClassification::OnboardingPlatformSetupReceiptCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingTermsAccept { .. } => {
            Some(TurnStartClassification::OnboardingTermsAcceptCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingPrimaryDeviceConfirm { .. } => {
            Some(TurnStartClassification::OnboardingPrimaryDeviceConfirmCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeePhotoCaptureSend { .. } => {
            Some(TurnStartClassification::OnboardingEmployeePhotoCaptureSendCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeeSenderVerifyCommit { .. } => {
            Some(TurnStartClassification::OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingVoiceEnrollLock { .. } => {
            Some(TurnStartClassification::OnboardingVoiceEnrollLockCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollStartDraft { .. } => {
            Some(TurnStartClassification::OnboardingWakeEnrollStartDraftCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollSampleCommit { .. } => {
            Some(TurnStartClassification::OnboardingWakeEnrollSampleCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollCompleteCommit { .. } => {
            Some(TurnStartClassification::OnboardingWakeEnrollCompleteCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingEmoPersonaLock { .. } => {
            Some(TurnStartClassification::OnboardingEmoPersonaLockCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingAccessProvisionCommit { .. } => {
            Some(TurnStartClassification::OnboardingAccessProvisionCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingCompleteCommit { .. } => {
            Some(TurnStartClassification::OnboardingCompleteCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::OnboardingPairingCompletionCommit { .. } => {
            Some(TurnStartClassification::OnboardingPairingCompletionCommitCompatibilityPrepared)
        }
        CanonicalTurnPayloadCarrier::Text { .. } | CanonicalTurnPayloadCarrier::Binary { .. } => {
            None
        }
    }
}

fn pre_authority_ready_detail(normalized: &CanonicalTurnRequestCarrier) -> String {
    match &normalized.payload {
        CanonicalTurnPayloadCarrier::InviteClick { .. } => {
            "invite-click compatibility reached the bounded pre-authority handoff".to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingAskMissing { .. } => {
            "onboarding ask-missing compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPlatformSetupReceipt { .. } => {
            "onboarding platform-setup receipt compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingTermsAccept { .. } => {
            "onboarding terms-accept compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPrimaryDeviceConfirm { .. } => {
            "onboarding primary-device-confirm compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeePhotoCaptureSend { .. } => {
            "onboarding employee-photo-capture-send compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmployeeSenderVerifyCommit { .. } => {
            "onboarding employee-sender-verify compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingVoiceEnrollLock { .. } => {
            "onboarding voice-enroll-lock compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollStartDraft { .. } => {
            "onboarding wake-enroll-start compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollSampleCommit { .. } => {
            "onboarding wake-enroll-sample compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingWakeEnrollCompleteCommit { .. } => {
            "onboarding wake-enroll-complete compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingEmoPersonaLock { .. } => {
            "onboarding emo-persona-lock compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingAccessProvisionCommit { .. } => {
            "onboarding access-provision compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingCompleteCommit { .. } => {
            "onboarding complete compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::OnboardingPairingCompletionCommit { .. } => {
            "onboarding pairing-completion compatibility reached the bounded pre-authority handoff"
                .to_string()
        }
        CanonicalTurnPayloadCarrier::Text { .. } | CanonicalTurnPayloadCarrier::Binary { .. } => {
            "turn reached the bounded pre-authority handoff".to_string()
        }
    }
}

fn resolve_session_turn(
    sessions: &mut RuntimeSessionFoundation,
    request: &RuntimeCanonicalIngressRequest,
    normalized: &CanonicalTurnRequestCarrier,
) -> Result<ResolvedSessionTurn, SessionFoundationError> {
    let device_turn_sequence = normalized.device_turn_sequence;
    match request.session_resolve_mode {
        SessionResolveMode::ResolveOrOpen => {
            if let Some(session_id) = request.session_hint {
                match sessions.begin_turn(
                    session_id,
                    &request.device_identity,
                    device_turn_sequence,
                ) {
                    Ok(SessionTurnResolution::Started(permit)) => {
                        Ok(ResolvedSessionTurn::Started {
                            session_state: sessions.session_snapshot(session_id)?.session_state,
                            permit,
                            classification: TurnStartClassification::ExistingSessionContinued,
                            attach_outcome: None,
                        })
                    }
                    Ok(SessionTurnResolution::Retry(projection)) => {
                        Ok(ResolvedSessionTurn::Retry { projection })
                    }
                    Ok(SessionTurnResolution::Deferred(deferred)) => {
                        Ok(ResolvedSessionTurn::Deferred {
                            session_state: sessions.session_snapshot(session_id)?.session_state,
                            deferred,
                        })
                    }
                    Err(err) if err.kind == SessionFoundationErrorKind::DeviceNotAttached => {
                        let attach = sessions.attach_session_with_access_claim(
                            session_id,
                            request.device_identity.clone(),
                            SessionAccessClass::PrimaryInteractor,
                            Some(device_turn_sequence),
                        )?;
                        match sessions.begin_turn(
                            session_id,
                            &request.device_identity,
                            device_turn_sequence,
                        )? {
                            SessionTurnResolution::Started(permit) => {
                                Ok(ResolvedSessionTurn::Started {
                                    session_state: sessions
                                        .session_snapshot(session_id)?
                                        .session_state,
                                    permit,
                                    classification: compatibility_prepared_classification(
                                        normalized,
                                    )
                                    .unwrap_or(TurnStartClassification::ExistingSessionAttached),
                                    attach_outcome: attach.projection.attach_outcome,
                                })
                            }
                            SessionTurnResolution::Retry(projection) => {
                                Ok(ResolvedSessionTurn::Retry { projection })
                            }
                            SessionTurnResolution::Deferred(deferred) => {
                                Ok(ResolvedSessionTurn::Deferred {
                                    session_state: sessions
                                        .session_snapshot(session_id)?
                                        .session_state,
                                    deferred,
                                })
                            }
                        }
                    }
                    Err(err) => Err(err),
                }
            } else {
                match sessions
                    .start_new_session_turn(request.device_identity.clone(), device_turn_sequence)?
                {
                    SessionTurnResolution::Started(permit) => Ok(ResolvedSessionTurn::Started {
                        session_state: sessions.session_snapshot(permit.session_id)?.session_state,
                        attach_outcome: permit.attach_outcome,
                        permit,
                        classification: compatibility_prepared_classification(normalized)
                            .unwrap_or(TurnStartClassification::NewSessionOpenBypass),
                    }),
                    SessionTurnResolution::Retry(_) | SessionTurnResolution::Deferred(_) => {
                        unreachable!("start_new_session_turn cannot reuse or defer the first turn")
                    }
                }
            }
        }
        SessionResolveMode::ResumeExisting => {
            let session_id = request
                .session_hint
                .expect("validated resume requests carry a session_hint");
            let resume = sessions.resume_session(session_id, request.device_identity.clone())?;
            match sessions.begin_turn(session_id, &request.device_identity, device_turn_sequence)? {
                SessionTurnResolution::Started(permit) => Ok(ResolvedSessionTurn::Started {
                    session_state: sessions.session_snapshot(session_id)?.session_state,
                    permit,
                    classification: TurnStartClassification::ExistingSessionResumed,
                    attach_outcome: resume.projection.attach_outcome,
                }),
                SessionTurnResolution::Retry(projection) => {
                    Ok(ResolvedSessionTurn::Retry { projection })
                }
                SessionTurnResolution::Deferred(deferred) => Ok(ResolvedSessionTurn::Deferred {
                    session_state: sessions.session_snapshot(session_id)?.session_state,
                    deferred,
                }),
            }
        }
        SessionResolveMode::RecoverExisting => {
            let session_id = request
                .session_hint
                .expect("validated recover requests carry a session_hint");
            let recover = sessions.recover_session(session_id, request.device_identity.clone())?;
            match sessions.begin_turn(session_id, &request.device_identity, device_turn_sequence)? {
                SessionTurnResolution::Started(permit) => Ok(ResolvedSessionTurn::Started {
                    session_state: sessions.session_snapshot(session_id)?.session_state,
                    permit,
                    classification: TurnStartClassification::ExistingSessionRecovered,
                    attach_outcome: recover.projection.attach_outcome,
                }),
                SessionTurnResolution::Retry(projection) => {
                    Ok(ResolvedSessionTurn::Retry { projection })
                }
                SessionTurnResolution::Deferred(deferred) => Ok(ResolvedSessionTurn::Deferred {
                    session_state: sessions.session_snapshot(session_id)?.session_state,
                    deferred,
                }),
            }
        }
    }
}

fn create_runtime_execution_envelope(
    prepared: &RuntimePreparedRequest,
    request: &RuntimeCanonicalIngressRequest,
    session_id: SessionId,
    turn_id: TurnId,
    device_turn_sequence: Option<u64>,
    session_attach_outcome: Option<SessionAttachOutcome>,
    admission_state: AdmissionState,
) -> Result<RuntimeExecutionEnvelope, RuntimeIngressTurnError> {
    let Some(envelope) = prepared.envelope.as_ref() else {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_ENVELOPE_INVALID,
            FailureClass::InvalidPayload,
            "canonical turn ingress requires a request foundation envelope",
        ));
    };
    RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
        envelope.header().request_id.clone(),
        envelope.header().trace_id.clone(),
        envelope.header().idempotency_key.clone(),
        request.actor_identity.clone(),
        request.device_identity.clone(),
        request.platform_context.platform_type,
        request.platform_context.clone(),
        Some(session_id),
        turn_id,
        device_turn_sequence,
        admission_state,
        session_attach_outcome,
    )
    .map_err(map_contract_violation)
}

fn create_runtime_execution_envelope_from_projection(
    prepared: &RuntimePreparedRequest,
    request: &RuntimeCanonicalIngressRequest,
    projection: &SessionRuntimeProjection,
    admission_state: AdmissionState,
) -> Result<RuntimeExecutionEnvelope, RuntimeIngressTurnError> {
    let turn_id = projection.turn_id.ok_or_else(|| {
        RuntimeIngressTurnError::new(
            reason_codes::INGRESS_ENVELOPE_INVALID,
            FailureClass::ExecutionFailure,
            "retry projection must carry a turn_id before pre-authority binding",
        )
    })?;
    create_runtime_execution_envelope(
        prepared,
        request,
        projection.session_id,
        turn_id,
        projection.device_turn_sequence,
        projection.attach_outcome,
        admission_state,
    )
}

fn validate_ready_invariants(
    prepared: &RuntimePreparedRequest,
    normalized: &CanonicalTurnRequestCarrier,
    envelope: &RuntimeExecutionEnvelope,
    stage_history: &[PreAuthorityStageRecord],
    classification: TurnStartClassification,
) -> Result<(), RuntimeIngressTurnError> {
    if prepared.definition.key.path != normalized.family.route_path()
        || prepared.definition.handler != normalized.family.handler()
    {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "pre-authority handoff must originate from the selected canonical Section 03 route",
        ));
    }
    if normalized.canonical_route != normalized.family.route_path() {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "normalized requests must preserve the canonical route selected for their family",
        ));
    }
    if envelope.session_id.is_none() {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_ENVELOPE_INVALID,
            FailureClass::ExecutionFailure,
            "pre-authority envelopes must carry a lawful session_id",
        ));
    }
    if envelope.platform != normalized.platform
        || envelope.platform_context.requested_trigger != normalized.requested_trigger
    {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_ENVELOPE_INVALID,
            FailureClass::ExecutionFailure,
            "pre-authority envelope platform posture drifted from normalized ingress truth",
        ));
    }
    if envelope.admission_state != AdmissionState::ExecutionAdmitted {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "pre-authority envelopes must stop with ExecutionAdmitted ingress posture",
        ));
    }
    if envelope.persistence_state.is_some()
        || envelope.governance_state.is_some()
        || envelope.proof_state.is_some()
        || envelope.identity_state.is_some()
        || envelope.voice_identity_assertion.is_some()
        || envelope.memory_state.is_some()
        || envelope.authority_state.is_some()
        || envelope.artifact_trust_state.is_some()
        || envelope.law_state.is_some()
    {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "Section 03 ingress must stop before Section 04, Section 05, or later runtime execution state appears",
        ));
    }
    validate_stage_history(
        stage_history,
        &[
            PreAuthorityStage::IngressValidated,
            PreAuthorityStage::TriggerValidated,
            PreAuthorityStage::SessionResolved,
            PreAuthorityStage::EnvelopeCreated,
            PreAuthorityStage::TurnClassified,
        ],
    )?;
    if matches!(classification, TurnStartClassification::Deferred) {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "deferred classifications may not produce a ready pre-authority envelope",
        ));
    }
    Ok(())
}

fn validate_deferred_invariants(
    prepared: &RuntimePreparedRequest,
    normalized: &CanonicalTurnRequestCarrier,
    stage_history: &[PreAuthorityStageRecord],
) -> Result<(), RuntimeIngressTurnError> {
    if prepared.definition.key.path != normalized.family.route_path()
        || prepared.definition.handler != normalized.family.handler()
        || normalized.canonical_route != normalized.family.route_path()
    {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "deferred outcomes must still originate from the selected canonical Section 03 route",
        ));
    }
    validate_stage_history(
        stage_history,
        &[
            PreAuthorityStage::IngressValidated,
            PreAuthorityStage::TriggerValidated,
            PreAuthorityStage::SessionResolved,
            PreAuthorityStage::TurnClassified,
        ],
    )
}

fn validate_stage_history(
    stage_history: &[PreAuthorityStageRecord],
    expected: &[PreAuthorityStage],
) -> Result<(), RuntimeIngressTurnError> {
    let actual: Vec<PreAuthorityStage> = stage_history.iter().map(|record| record.stage).collect();
    if actual != expected {
        return Err(RuntimeIngressTurnError::new(
            reason_codes::INGRESS_STAGE_INVALID,
            FailureClass::ExecutionFailure,
            "pre-authority stage order drifted from the canonical Section 03 scaffold",
        ));
    }
    Ok(())
}

fn map_contract_violation(error: ContractViolation) -> RuntimeIngressTurnError {
    let message = match error {
        ContractViolation::InvalidValue { field, reason } => format!("{field}: {reason}"),
        ContractViolation::InvalidRange {
            field,
            min,
            max,
            got,
        } => format!("{field}: expected range {min}..={max}, got {got}"),
        ContractViolation::NotFinite { field } => format!("{field}: value must be finite"),
    };
    RuntimeIngressTurnError::new(
        reason_codes::INGRESS_ENVELOPE_INVALID,
        FailureClass::InvalidPayload,
        message,
    )
}

fn map_request_error(error: RuntimeRequestFoundationError) -> RuntimeIngressTurnError {
    RuntimeIngressTurnError {
        reason_code: error.reason_code,
        failure_class: error.failure_class,
        message: error.message,
        stage_history: Vec::new(),
        request_envelope: error.rejected_envelope,
        runtime_execution_envelope: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionUnknown,
        SpeakerLabel,
    };
    use selene_kernel_contracts::ph1link::TokenId;
    use selene_kernel_contracts::provider_secrets::ProviderSecretId;

    use crate::runtime_bootstrap::{
        RuntimeBootstrapConfig, RuntimeBuildMetadata, RuntimeSecretValue, RuntimeSecretsProvider,
    };
    use crate::runtime_request_foundation::{
        RuntimeRequestFoundationErrorKind, RuntimeRequestOrigin,
    };

    #[derive(Debug, Default)]
    struct FixedClock {
        now_ms: Cell<i64>,
    }

    impl FixedClock {
        fn new(start_ms: i64) -> Self {
            Self {
                now_ms: Cell::new(start_ms),
            }
        }
    }

    impl RuntimeClock for FixedClock {
        fn now_unix_ms(&self) -> i64 {
            let current = self.now_ms.get();
            self.now_ms.set(current + 1);
            current
        }
    }

    #[derive(Debug, Default, Clone)]
    struct StaticSecretsProvider {
        secrets: std::collections::BTreeMap<ProviderSecretId, RuntimeSecretValue>,
    }

    impl StaticSecretsProvider {
        fn with_secret(
            mut self,
            key: ProviderSecretId,
            value: &str,
        ) -> Result<Self, crate::runtime_bootstrap::RuntimeBootstrapError> {
            self.secrets
                .insert(key, RuntimeSecretValue::new(value.to_string())?);
            Ok(self)
        }
    }

    impl RuntimeSecretsProvider for StaticSecretsProvider {
        fn get_secret(&self, key: ProviderSecretId) -> Option<RuntimeSecretValue> {
            self.secrets.get(&key).cloned()
        }
    }

    fn build_metadata() -> RuntimeBuildMetadata {
        RuntimeBuildMetadata {
            node_id: "node-a".to_string(),
            runtime_instance_identity: "instance-a".to_string(),
            environment_identity: "test".to_string(),
            build_version: "build-1".to_string(),
            git_commit: "abcdef".to_string(),
        }
    }

    fn bootstrap_config(required_secrets: Vec<ProviderSecretId>) -> RuntimeBootstrapConfig {
        RuntimeBootstrapConfig {
            service_name: "selene_runtime".to_string(),
            shutdown_grace_period_ms: 5_000,
            required_secrets,
            build_metadata: build_metadata(),
        }
    }

    fn request_foundation_config() -> RuntimeRequestFoundationConfig {
        RuntimeRequestFoundationConfig {
            service_name: "selene_runtime".to_string(),
            build_metadata: build_metadata(),
            max_clock_skew_ms: 30_000,
        }
    }

    fn ready_runtime() -> RuntimeProcess<FixedClock, StaticSecretsProvider> {
        let clock = FixedClock::new(1_000);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let services =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        let mut runtime = RuntimeProcess::new(
            bootstrap_config(vec![ProviderSecretId::OpenAIApiKey]),
            services,
        );
        runtime.start().expect("runtime should reach ready");
        runtime
    }

    fn device(id: &str) -> DeviceId {
        DeviceId::new(id).expect("device id")
    }

    fn user(id: &str) -> UserId {
        UserId::new(id).expect("user id")
    }

    fn sample_voice_identity_assertion() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1(
                IdentityConfidence::Medium,
                selene_kernel_contracts::ReasonCodeId(1),
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

    fn envelope_input(
        request_id: &str,
        trace_id: &str,
        idempotency_key: &str,
        platform: AppPlatform,
        trigger: RuntimeEntryTrigger,
        timestamp_ms: i64,
    ) -> RuntimeRequestEnvelopeInput {
        RuntimeRequestEnvelopeInput {
            request_id: request_id.to_string(),
            trace_id: trace_id.to_string(),
            idempotency_key: idempotency_key.to_string(),
            timestamp_ms,
            nonce: format!("nonce-{request_id}"),
            origin: RuntimeRequestOrigin { platform, trigger },
        }
    }

    fn platform_context(
        platform: AppPlatform,
        trigger: RuntimeEntryTrigger,
    ) -> PlatformRuntimeContext {
        PlatformRuntimeContext::default_for_platform_and_trigger(platform, trigger)
            .expect("platform context")
    }

    fn foundation() -> RuntimeIngressTurnFoundation {
        RuntimeIngressTurnFoundation::with_slice_2a_defaults(request_foundation_config())
            .expect("slice 2a foundation")
    }

    fn onboarding_session(id: &str) -> OnboardingSessionId {
        OnboardingSessionId::new(id).expect("onboarding session id")
    }

    fn text_turn_request(
        request_id: &str,
        trace_id: &str,
        session_hint: Option<SessionId>,
        sequence: u64,
    ) -> RuntimeCanonicalIngressRequest {
        RuntimeCanonicalIngressRequest::turn(
            envelope_input(
                request_id,
                trace_id,
                &format!("idem-{request_id}"),
                AppPlatform::Android,
                RuntimeEntryTrigger::Explicit,
                1_010,
            ),
            "Bearer token-1".to_string(),
            user("user_runtime_1"),
            device("device-a"),
            platform_context(AppPlatform::Android, RuntimeEntryTrigger::Explicit),
            session_hint,
            sequence,
            SessionResolveMode::ResolveOrOpen,
            CanonicalTurnModality::Text,
            RawTurnPayload::Text {
                content_type: "text/plain".to_string(),
                text: "hello selene".to_string(),
            },
        )
        .expect("text turn request")
    }

    fn stage4_activation(
        source: Stage4ActivationSource,
        trigger: RuntimeEntryTrigger,
    ) -> Stage4ActivationPacket {
        let mut packet =
            Stage4ActivationPacket::new(source, platform_context(AppPlatform::Desktop, trigger))
                .expect("stage 4 activation packet");
        packet.consent_state_id = Some("consent-stage4".to_string());
        packet.device_trust_ref = Some("device-trust-stage4".to_string());
        packet.provider_budget_ref = Some("provider-budget-stage4".to_string());
        packet.audit_id = Some("audit-stage4".to_string());
        packet.validate().expect("stage 4 refs must validate");
        packet
    }

    #[test]
    fn stage_4a_activation_sources_preserve_trigger_posture() {
        let wake = stage4_activation(
            Stage4ActivationSource::WakeCandidate,
            RuntimeEntryTrigger::WakeWord,
        );
        assert_eq!(
            wake.platform_context.requested_trigger,
            RuntimeEntryTrigger::WakeWord
        );
        assert!(!wake.route_authority().any_route_enabled());

        let side_button = stage4_activation(
            Stage4ActivationSource::SideButton,
            RuntimeEntryTrigger::Explicit,
        );
        assert_eq!(
            side_button.platform_context.requested_trigger,
            RuntimeEntryTrigger::Explicit
        );
        assert!(!side_button.route_authority().any_route_enabled());

        let mismatched = Stage4ActivationPacket::new(
            Stage4ActivationSource::SideButton,
            platform_context(AppPlatform::Desktop, RuntimeEntryTrigger::WakeWord),
        );
        assert!(mismatched.is_err());
    }

    #[test]
    fn stage_4a_candidate_preview_packets_cannot_route_work() {
        let packet = Stage4TurnBoundaryPacket::candidate_preview(
            stage4_activation(
                Stage4ActivationSource::ExplicitMic,
                RuntimeEntryTrigger::Explicit,
            ),
            Some(CanonicalTurnModality::Voice),
        )
        .expect("candidate preview packet");

        assert!(!packet.is_committed_live_turn());
        assert!(!packet.route_authority().any_route_enabled());
        assert!(packet.turn_id.is_none());
        assert!(packet.device_turn_sequence.is_none());
        assert!(packet.record_boundary.is_none());
    }

    #[test]
    fn stage_4a_committed_turn_packet_is_not_route_authority() {
        let mut activation = stage4_activation(
            Stage4ActivationSource::TypedInput,
            RuntimeEntryTrigger::Explicit,
        );
        activation.session_hint = Some(SessionId(42));
        let packet = Stage4TurnBoundaryPacket::committed_live_turn(
            activation,
            TurnId(7),
            1,
            CanonicalTurnModality::Text,
        )
        .expect("committed live turn packet");

        assert!(packet.is_committed_live_turn());
        assert!(!packet.route_authority().any_route_enabled());
        assert!(packet.record_boundary.is_none());
        assert_eq!(packet.turn_id, Some(TurnId(7)));
        assert_eq!(packet.device_turn_sequence, Some(1));
    }

    #[test]
    fn stage_4a_record_button_cannot_create_live_chat_turn() {
        let mut activation = stage4_activation(
            Stage4ActivationSource::RecordButton,
            RuntimeEntryTrigger::Explicit,
        );
        activation.session_hint = Some(SessionId(99));

        let live_turn = Stage4TurnBoundaryPacket::committed_live_turn(
            activation,
            TurnId(9),
            1,
            CanonicalTurnModality::Voice,
        );

        assert!(live_turn.is_err());
    }

    #[test]
    fn stage_4a_record_boundary_stays_artifact_only() {
        let record_boundary = Stage4RecordBoundary {
            recording_session_id: "record-session-stage4".to_string(),
            recording_state: Stage4RecordingState::Stopped,
            audio_artifact_id: "audio-artifact-stage4".to_string(),
            consent_state_id: "consent-stage4".to_string(),
            artifact_lane_handoff_ref: "artifact-lane-stage4".to_string(),
        };
        let packet = Stage4TurnBoundaryPacket::record_artifact_only(
            stage4_activation(
                Stage4ActivationSource::RecordButton,
                RuntimeEntryTrigger::Explicit,
            ),
            record_boundary,
        )
        .expect("record artifact packet");

        assert!(!packet.is_committed_live_turn());
        assert!(!packet.record_mode_can_be_live_chat());
        assert!(!packet.route_authority().any_route_enabled());
        assert!(packet.turn_id.is_none());
        assert!(packet.device_turn_sequence.is_none());
        assert!(packet.modality.is_none());
        assert!(packet.record_boundary.is_some());
    }

    fn invite_click_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        RuntimeCanonicalIngressRequest::invite_click(
            envelope_input(
                request_id,
                trace_id,
                &format!("idem-{request_id}"),
                AppPlatform::Android,
                trigger,
                1_040,
            ),
            platform_context(AppPlatform::Android, trigger),
            InviteOpenActivateCommitRequest {
                token_id: TokenId::new("invite-token-1").expect("token"),
                token_signature: "v1.k1.signature".to_string(),
                device_fingerprint: "fingerprint-device-a".to_string(),
                app_platform: AppPlatform::Android,
                app_instance_id: "app-instance-1".to_string(),
                deep_link_nonce: "deep-link-1".to_string(),
                link_opened_at: MonotonicTimeNs(1_000_000_000),
                idempotency_key: format!("idem-{request_id}"),
            },
        )
        .expect("invite click request")
    }

    fn onboarding_continue_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
        action: AppOnboardingContinueAction,
    ) -> RuntimeCanonicalIngressRequest {
        RuntimeCanonicalIngressRequest::onboarding_continue(
            envelope_input(
                request_id,
                trace_id,
                &format!("idem-{request_id}"),
                AppPlatform::Android,
                trigger,
                1_060,
            ),
            platform_context(AppPlatform::Android, trigger),
            AppOnboardingContinueRequest::v1(
                CorrelationId(101),
                onboarding_session("onb-session-1"),
                format!("idem-{request_id}"),
                Some("tenant-a".to_string()),
                action,
            )
            .expect("onboarding continue request"),
        )
        .expect("onboarding compatibility request")
    }

    fn onboarding_ask_missing_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
        field_value: Option<&str>,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::AskMissingSubmit {
                field_value: field_value.map(str::to_string),
            },
        )
    }

    fn onboarding_platform_setup_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::PlatformSetupReceipt {
                receipt_kind: "DEVICE_ATTEST".to_string(),
                receipt_ref: "receipt-ref-1".to_string(),
                signer: "signer-a".to_string(),
                payload_hash: "payload-hash-1".to_string(),
            },
        )
    }

    fn onboarding_terms_accept_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::TermsAccept {
                terms_version_id: "terms-v1".to_string(),
                accepted: true,
            },
        )
    }

    fn onboarding_primary_device_confirm_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::PrimaryDeviceConfirm {
                device_id: device("device-a"),
                proof_ok: true,
            },
        )
    }

    fn onboarding_employee_photo_capture_send_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::EmployeePhotoCaptureSend {
                photo_blob_ref: "photo-blob-ref-1".to_string(),
            },
        )
    }

    fn onboarding_employee_sender_verify_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::EmployeeSenderVerifyCommit {
                decision: selene_kernel_contracts::ph1onb::SenderVerifyDecision::Confirm,
            },
        )
    }

    fn onboarding_voice_enroll_lock_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::VoiceEnrollLock {
                device_id: device("device-a"),
                sample_seed: "seed-1".to_string(),
            },
        )
    }

    fn onboarding_wake_enroll_start_draft_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::WakeEnrollStartDraft {
                device_id: device("device-a"),
            },
        )
    }

    fn onboarding_wake_enroll_sample_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::WakeEnrollSampleCommit {
                device_id: device("device-a"),
                sample_pass: true,
            },
        )
    }

    fn onboarding_wake_enroll_complete_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::WakeEnrollCompleteCommit {
                device_id: device("device-a"),
            },
        )
    }

    fn onboarding_emo_persona_lock_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::EmoPersonaLock,
        )
    }

    fn onboarding_access_provision_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::AccessProvisionCommit,
        )
    }

    fn onboarding_complete_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::CompleteCommit,
        )
    }

    fn onboarding_pairing_completion_commit_request(
        request_id: &str,
        trace_id: &str,
        trigger: RuntimeEntryTrigger,
    ) -> RuntimeCanonicalIngressRequest {
        onboarding_continue_request(
            request_id,
            trace_id,
            trigger,
            AppOnboardingContinueAction::PairingCompletionCommit {
                device_id: device("device-a"),
                session_id: SessionId(9_001),
                session_attach_outcome: SessionAttachOutcome::NewSessionCreated,
            },
        )
    }

    #[test]
    fn slice_2a_registers_only_the_h3_canonical_route_family() {
        let foundation = foundation();
        assert_eq!(
            foundation.section03_route_paths(),
            vec![
                "/v1/invite/click".to_string(),
                "/v1/onboarding/continue".to_string(),
                "/v1/voice/turn".to_string(),
            ]
        );
    }

    #[test]
    fn slice_2a_duplicate_and_disallowed_route_registration_is_rejected() {
        let mut router =
            RuntimeRouter::with_slice_1b_foundation_defaults(request_foundation_config())
                .expect("router");
        router
            .register_canonical_ingress_route(
                RuntimeRouteDefinition::canonical_turn().expect("voice route"),
            )
            .expect("canonical route");
        let duplicate = router
            .register_canonical_ingress_route(
                RuntimeRouteDefinition::canonical_turn().expect("voice route"),
            )
            .expect_err("duplicate route must fail closed");
        assert_eq!(
            duplicate.kind,
            RuntimeRequestFoundationErrorKind::DuplicateRoute
        );

        let disallowed = router
            .register_canonical_ingress_route(RuntimeRouteDefinition {
                key: crate::runtime_request_foundation::RuntimeRouteKey::new(
                    crate::runtime_request_foundation::RuntimeHttpMethod::Post,
                    "/v1/voice/other",
                )
                .expect("key"),
                handler: RuntimeRouteHandlerKind::CanonicalTurnIngress,
                request_class: crate::runtime_request_foundation::RuntimeRequestClass::CanonicalTurnIngress,
                admission_policy: crate::runtime_request_foundation::RuntimeAdmissionPolicy::RequireReady,
                required_middleware: std::collections::BTreeSet::from([
                    crate::runtime_request_foundation::RuntimeRouteMiddlewareKind::EnvelopeFoundation,
                    crate::runtime_request_foundation::RuntimeRouteMiddlewareKind::RequestSecurity,
                    crate::runtime_request_foundation::RuntimeRouteMiddlewareKind::AdmissionControl,
                    crate::runtime_request_foundation::RuntimeRouteMiddlewareKind::FeatureFlags,
                    crate::runtime_request_foundation::RuntimeRouteMiddlewareKind::InvariantValidation,
                ]),
                description: "illegal",
            })
            .expect_err("non-canonical route must fail closed");
        assert_eq!(
            disallowed.kind,
            RuntimeRequestFoundationErrorKind::RouteScopeViolation
        );
    }

    #[test]
    fn slice_2a_normalization_converges_modalities_into_one_canonical_turn_family() {
        let request_id_prefix = "modality";
        let cases = vec![
            (
                CanonicalTurnModality::Voice,
                RawTurnPayload::Binary {
                    content_type: "audio/wav".to_string(),
                    bytes: vec![1, 2, 3],
                },
            ),
            (
                CanonicalTurnModality::Text,
                RawTurnPayload::Text {
                    content_type: "text/plain".to_string(),
                    text: "hello".to_string(),
                },
            ),
            (
                CanonicalTurnModality::File,
                RawTurnPayload::Binary {
                    content_type: "application/pdf".to_string(),
                    bytes: vec![4, 5, 6],
                },
            ),
            (
                CanonicalTurnModality::Image,
                RawTurnPayload::Binary {
                    content_type: "image/jpeg".to_string(),
                    bytes: vec![7, 8, 9],
                },
            ),
            (
                CanonicalTurnModality::Camera,
                RawTurnPayload::Binary {
                    content_type: "image/png".to_string(),
                    bytes: vec![10, 11, 12],
                },
            ),
        ];

        for (index, (modality, payload)) in cases.into_iter().enumerate() {
            let request = RuntimeCanonicalIngressRequest::turn(
                envelope_input(
                    &format!("{request_id_prefix}-{index}"),
                    &format!("trace-{index}"),
                    &format!("idem-{index}"),
                    AppPlatform::Android,
                    RuntimeEntryTrigger::Explicit,
                    1_020,
                ),
                "Bearer token-1".to_string(),
                user("user_runtime_1"),
                device("device-a"),
                platform_context(AppPlatform::Android, RuntimeEntryTrigger::Explicit),
                None,
                (index + 1) as u64,
                SessionResolveMode::ResolveOrOpen,
                modality,
                payload,
            )
            .expect("request");
            let normalized = normalize_turn_request(&request).expect("normalized");
            assert_eq!(normalized.canonical_route, CANONICAL_TURN_ENDPOINT_PATH);
            assert_eq!(normalized.family, CanonicalIngressFamily::VoiceTurn);
            assert_eq!(normalized.modality, modality);
            assert!(!normalized.request_content_hash.is_empty());
        }
    }

    #[test]
    fn slice_2a_invalid_or_unsupported_ingress_is_rejected_fail_closed() {
        let request = RuntimeCanonicalIngressRequest::turn(
            envelope_input(
                "bad-payload",
                "trace-bad",
                "idem-bad",
                AppPlatform::Android,
                RuntimeEntryTrigger::Explicit,
                1_020,
            ),
            "Bearer token-1".to_string(),
            user("user_runtime_1"),
            device("device-a"),
            platform_context(AppPlatform::Android, RuntimeEntryTrigger::Explicit),
            None,
            1,
            SessionResolveMode::ResolveOrOpen,
            CanonicalTurnModality::Voice,
            RawTurnPayload::Text {
                content_type: "text/plain".to_string(),
                text: "not audio".to_string(),
            },
        )
        .expect("request");
        let err = normalize_turn_request(&request).expect_err("payload mismatch must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_PAYLOAD_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2a_trigger_validation_is_deterministic_and_fail_closed() {
        let mut request = text_turn_request("trigger-1", "trace-trigger-1", None, 1);
        request.platform_context =
            platform_context(AppPlatform::Android, RuntimeEntryTrigger::WakeWord);
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("trigger mismatch must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_TRIGGER_INVALID);
        assert_eq!(err.failure_class, FailureClass::PolicyViolation);
    }

    #[test]
    fn slice_2a_session_resolve_or_open_consumes_accepted_session_foundation() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();

        let first = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                text_turn_request("turn-1", "trace-1", None, 1),
            )
            .expect("new session turn");
        let ready = match first {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::NewSessionOpenBypass
        );
        sessions
            .finish_turn(ready.session_turn_permit.clone(), SessionState::Active)
            .expect("test may drain the first turn before starting the next");
        sessions
            .detach_session(ready.response.session_id, &device("device-a"))
            .expect("test may detach the primary device before reattaching");

        let attach_request = RuntimeCanonicalIngressRequest::turn(
            envelope_input(
                "turn-2",
                "trace-2",
                "idem-turn-2",
                AppPlatform::Android,
                RuntimeEntryTrigger::Explicit,
                1_030,
            ),
            "Bearer token-2".to_string(),
            user("user_runtime_1"),
            device("device-a"),
            platform_context(AppPlatform::Android, RuntimeEntryTrigger::Explicit),
            Some(ready.response.session_id),
            2,
            SessionResolveMode::ResolveOrOpen,
            CanonicalTurnModality::Text,
            RawTurnPayload::Text {
                content_type: "text/plain".to_string(),
                text: "follow-up".to_string(),
            },
        )
        .expect("attach request");
        let second = foundation
            .process_turn_start(&runtime, &mut sessions, attach_request)
            .expect("attached turn");
        let ready = match second {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::ExistingSessionAttached
        );
    }

    #[test]
    fn slice_2a_runtime_execution_envelope_binding_is_canonical_and_pre_authority_only() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let ready = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                text_turn_request("bind-1", "trace-bind-1", None, 1),
            )
            .expect("turn");
        let ready = match ready {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "bind-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-bind-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.admission_state,
            AdmissionState::ExecutionAdmitted
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready
            .runtime_execution_envelope
            .voice_identity_assertion
            .is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2a_ready_invariants_reject_prepopulated_voice_identity_assertion() {
        let runtime = ready_runtime();
        let mut runtime_foundation = foundation();
        let mut guardrail_foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = text_turn_request("voice-carrier-1", "trace-voice-carrier-1", None, 1);
        let prepared = guardrail_foundation
            .router
            .prepare_request(
                &runtime,
                request
                    .to_foundation_request()
                    .expect("foundation request must build"),
            )
            .expect("prepared request must succeed");
        let first = runtime_foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("baseline pre-authority handoff must succeed");
        let ready = match first {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert!(ready
            .runtime_execution_envelope
            .voice_identity_assertion
            .is_none());
        let envelope = ready
            .runtime_execution_envelope
            .with_voice_identity_assertion(Some(sample_voice_identity_assertion()))
            .expect("voice carrier attachment must validate");
        let err = validate_ready_invariants(
            &prepared,
            &ready.normalized_request,
            &envelope,
            &ready.stage_history[..ready.stage_history.len() - 1],
            ready.response.classification,
        )
        .expect_err("pre-authority voice assertion transport must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_STAGE_INVALID);
        assert_eq!(err.failure_class, FailureClass::ExecutionFailure);
    }

    #[test]
    fn slice_2a_pre_authority_stage_order_is_deterministic_and_recorded() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let ready = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                text_turn_request("stage-1", "trace-stage-1", None, 1),
            )
            .expect("turn");
        let ready = match ready {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
    }

    #[test]
    fn slice_2a_stage_boundary_invariant_failures_reject_fail_closed() {
        let foundation = foundation();
        let err = validate_stage_history(
            &[
                PreAuthorityStageRecord {
                    stage: PreAuthorityStage::IngressValidated,
                    at_unix_ms: 1,
                },
                PreAuthorityStageRecord {
                    stage: PreAuthorityStage::SessionResolved,
                    at_unix_ms: 2,
                },
            ],
            &[
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
            ],
        )
        .expect_err("missing trigger stage must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_STAGE_INVALID);
        assert_eq!(err.failure_class, FailureClass::ExecutionFailure);
        assert_eq!(foundation.counters().ready_handoffs, 0);
    }

    #[test]
    fn slice_2b_invite_click_is_the_only_newly_executable_compatibility_path() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = invite_click_request(
            "invite-compat-1",
            "trace-invite-compat-1",
            RuntimeEntryTrigger::Explicit,
        );
        assert!(request.authorization_bearer.is_empty());
        assert!(request
            .actor_identity
            .as_str()
            .starts_with("invite-compat-actor:"));
        assert!(request
            .device_identity
            .as_str()
            .starts_with("invite-compat-device:"));

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("invite click compatibility should execute in slice 2B");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::InviteClickCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::InviteClickCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            INVITE_CLICK_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready
            .runtime_execution_envelope
            .actor_identity
            .as_str()
            .starts_with("invite-compat-actor:"));
    }

    #[test]
    fn slice_2a_success_path_stops_before_section04_or_section05_execution() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let ready = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                text_turn_request("boundary-1", "trace-boundary-1", None, 1),
            )
            .expect("turn");
        let ready = match ready {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
    }

    #[test]
    fn slice_2a_observability_emits_turn_start_events_and_counters() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                text_turn_request("obs-1", "trace-obs-1", None, 1),
            )
            .expect("turn");
        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].kind,
            RuntimeIngressTurnEventKind::TurnNormalized
        );
        assert_eq!(
            foundation.events()[1].kind,
            RuntimeIngressTurnEventKind::PreAuthorityReady
        );
    }

    #[test]
    fn slice_2c_onboarding_ask_missing_remains_executable_in_slice_2h() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_ask_missing_request(
            "onb-ask-1",
            "trace-onb-ask-1",
            RuntimeEntryTrigger::Explicit,
            Some("display_name"),
        );
        assert!(request.authorization_bearer.is_empty());
        assert!(request
            .actor_identity
            .as_str()
            .starts_with("onboarding-compat-actor:"));
        assert!(request
            .device_identity
            .as_str()
            .starts_with("onboarding-compat-device:"));

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("onboarding ask-missing should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingAskMissingCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready
            .runtime_execution_envelope
            .actor_identity
            .as_str()
            .starts_with("onboarding-compat-actor:"));
    }

    #[test]
    fn slice_2d_platform_setup_receipt_remains_executable_in_slice_2h() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_platform_setup_request(
            "onb-platform-1",
            "trace-onb-platform-1",
            RuntimeEntryTrigger::Explicit,
        );
        assert!(request.authorization_bearer.is_empty());
        assert!(request
            .actor_identity
            .as_str()
            .starts_with("onboarding-compat-actor:"));
        assert!(request
            .device_identity
            .as_str()
            .starts_with("onboarding-compat-device:"));

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("platform setup should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingPlatformSetupReceiptCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
    }

    #[test]
    fn slice_2e_terms_accept_remains_executable_in_slice_2h() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_terms_accept_request(
            "onb-terms-1",
            "trace-onb-terms-1",
            RuntimeEntryTrigger::Explicit,
        );
        assert!(request.authorization_bearer.is_empty());
        assert!(request
            .actor_identity
            .as_str()
            .starts_with("onboarding-compat-actor:"));
        assert!(request
            .device_identity
            .as_str()
            .starts_with("onboarding-compat-device:"));

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("terms accept should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingTermsAcceptCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
    }

    #[test]
    fn slice_2f_primary_device_confirm_remains_executable_in_slice_2h() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_primary_device_confirm_request(
            "onb-primary-1",
            "trace-onb-primary-1",
            RuntimeEntryTrigger::Explicit,
        );
        assert!(request.authorization_bearer.is_empty());
        assert!(request
            .actor_identity
            .as_str()
            .starts_with("onboarding-compat-actor:"));
        assert!(request
            .device_identity
            .as_str()
            .starts_with("onboarding-compat-device:"));

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("primary device confirm should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingPrimaryDeviceConfirmCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-primary-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-primary-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2h_employee_photo_capture_send_remains_executable_in_slice_2h() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_employee_photo_capture_send_request(
            "onb-photo-1",
            "trace-onb-photo-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("employee photo capture should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmployeePhotoCaptureSendCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-photo-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-photo-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2h_employee_sender_verify_commit_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_employee_sender_verify_commit_request(
            "onb-sender-1",
            "trace-onb-sender-1",
            RuntimeEntryTrigger::Explicit,
        );
        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("sender-verification commit should execute in Slice 2H");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-sender-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-sender-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2i_voice_enroll_lock_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_voice_enroll_lock_request(
            "onb-voice-1",
            "trace-onb-voice-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("voice enroll lock should execute in Slice 2I");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingVoiceEnrollLockCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-voice-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-voice-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2j_wake_enroll_start_draft_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_wake_enroll_start_draft_request(
            "onb-start-1",
            "trace-onb-start-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("wake enroll start draft should execute in Slice 2J");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollStartDraftCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-start-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-start-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2k_wake_enroll_sample_commit_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_wake_enroll_sample_commit_request(
            "onb-sample-1",
            "trace-onb-sample-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("wake enroll sample commit should execute in Slice 2K");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollSampleCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-sample-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-sample-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2l_wake_enroll_complete_commit_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_wake_enroll_complete_commit_request(
            "onb-complete-1",
            "trace-onb-complete-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("wake enroll complete commit should execute in Slice 2L");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollCompleteCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-complete-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-complete-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2m_emo_persona_lock_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_emo_persona_lock_request(
            "onb-emo-1",
            "trace-onb-emo-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("emo persona lock should execute in Slice 2M");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmoPersonaLockCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-emo-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-emo-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2n_access_provision_commit_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_access_provision_commit_request(
            "onb-access-1",
            "trace-onb-access-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("access provision commit should execute in Slice 2N");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingAccessProvisionCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-access-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-access-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2o_complete_commit_is_the_one_newly_executable_onboarding_action() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_complete_commit_request(
            "onb-complete-1",
            "trace-onb-complete-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("complete commit should execute in Slice 2O");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingCompleteCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
        assert_eq!(
            ready.runtime_execution_envelope.request_id,
            "onb-complete-1".to_string()
        );
        assert_eq!(
            ready.runtime_execution_envelope.trace_id,
            "trace-onb-complete-1".to_string()
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn pairing_completion_commit_is_executable_in_the_existing_onboarding_continue_family() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = onboarding_pairing_completion_commit_request(
            "onb-pairing-complete-1",
            "trace-onb-pairing-complete-1",
            RuntimeEntryTrigger::Explicit,
        );

        let result = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect("pairing completion commit should execute in the existing onboarding family");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };

        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingPairingCompletionCommitCompatibilityPrepared
        );
        assert_eq!(
            ready.normalized_request.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(
            ready.normalized_request.canonical_route,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            ready.normalized_request.modality,
            CanonicalTurnModality::Compatibility
        );
        assert_eq!(
            ready.response.outcome,
            PreAuthorityOutcome::ReadyForSection04Boundary
        );
    }

    #[test]
    fn slice_2o_previously_accepted_onboarding_actions_remain_executable() {
        let runtime = ready_runtime();
        let cases = vec![
            (
                "ask-missing",
                onboarding_ask_missing_request(
                    "onb-ask-retained-1",
                    "trace-onb-ask-retained-1",
                    RuntimeEntryTrigger::Explicit,
                    Some("display_name"),
                ),
                TurnStartClassification::OnboardingAskMissingCompatibilityPrepared,
            ),
            (
                "platform-setup",
                onboarding_platform_setup_request(
                    "onb-platform-retained-1",
                    "trace-onb-platform-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingPlatformSetupReceiptCompatibilityPrepared,
            ),
            (
                "terms-accept",
                onboarding_terms_accept_request(
                    "onb-terms-retained-1",
                    "trace-onb-terms-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingTermsAcceptCompatibilityPrepared,
            ),
            (
                "primary-device-confirm",
                onboarding_primary_device_confirm_request(
                    "onb-primary-retained-1",
                    "trace-onb-primary-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingPrimaryDeviceConfirmCompatibilityPrepared,
            ),
            (
                "employee-photo-capture-send",
                onboarding_employee_photo_capture_send_request(
                    "onb-photo-retained-1",
                    "trace-onb-photo-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingEmployeePhotoCaptureSendCompatibilityPrepared,
            ),
            (
                "employee-sender-verify-commit",
                onboarding_employee_sender_verify_commit_request(
                    "onb-sender-retained-1",
                    "trace-onb-sender-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared,
            ),
            (
                "voice-enroll-lock",
                onboarding_voice_enroll_lock_request(
                    "onb-voice-retained-1",
                    "trace-onb-voice-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingVoiceEnrollLockCompatibilityPrepared,
            ),
            (
                "wake-enroll-start-draft",
                onboarding_wake_enroll_start_draft_request(
                    "onb-wake-start-retained-1",
                    "trace-onb-wake-start-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingWakeEnrollStartDraftCompatibilityPrepared,
            ),
            (
                "wake-enroll-sample-commit",
                onboarding_wake_enroll_sample_commit_request(
                    "onb-wake-sample-retained-1",
                    "trace-onb-wake-sample-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingWakeEnrollSampleCommitCompatibilityPrepared,
            ),
            (
                "wake-enroll-complete-commit",
                onboarding_wake_enroll_complete_commit_request(
                    "onb-wake-complete-retained-1",
                    "trace-onb-wake-complete-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingWakeEnrollCompleteCommitCompatibilityPrepared,
            ),
            (
                "emo-persona-lock",
                onboarding_emo_persona_lock_request(
                    "onb-emo-retained-1",
                    "trace-onb-emo-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingEmoPersonaLockCompatibilityPrepared,
            ),
            (
                "access-provision-commit",
                onboarding_access_provision_commit_request(
                    "onb-access-retained-1",
                    "trace-onb-access-retained-1",
                    RuntimeEntryTrigger::Explicit,
                ),
                TurnStartClassification::OnboardingAccessProvisionCommitCompatibilityPrepared,
            ),
        ];

        for (label, request, classification) in cases {
            let mut foundation = foundation();
            let mut sessions = RuntimeSessionFoundation::default();
            let result = foundation
                .process_turn_start(&runtime, &mut sessions, request)
                .unwrap_or_else(|err| {
                    panic!("{label} should remain executable in Slice 2O: {err:?}")
                });
            let ready = match result {
                RuntimePreAuthorityTurnResult::Ready(ready) => ready,
                other => panic!("{label} expected ready handoff, got {other:?}"),
            };

            assert_eq!(ready.response.classification, classification, "{label}");
            assert_eq!(
                ready.response.outcome,
                PreAuthorityOutcome::ReadyForSection04Boundary,
                "{label}"
            );
            assert_eq!(
                ready.normalized_request.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH,
                "{label}"
            );
            assert_eq!(
                ready.normalized_request.modality,
                CanonicalTurnModality::Compatibility,
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.identity_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.authority_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.persistence_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.proof_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.governance_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.computation_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.memory_state.is_none(),
                "{label}"
            );
            assert!(
                ready.runtime_execution_envelope.law_state.is_none(),
                "{label}"
            );
        }
    }

    #[test]
    fn slice_2o_wake_enroll_defer_commit_remains_non_executable() {
        let runtime = ready_runtime();
        let cases = vec![(
            "wake-enroll-defer",
            onboarding_continue_request(
                "onb-defer-2o-1",
                "trace-onb-defer-2o-1",
                RuntimeEntryTrigger::Explicit,
                AppOnboardingContinueAction::WakeEnrollDeferCommit {
                    device_id: device("device-a"),
                },
            ),
        )];

        for (label, request) in cases {
            let mut foundation = foundation();
            let mut sessions = RuntimeSessionFoundation::default();
            let err = foundation
                .process_turn_start(&runtime, &mut sessions, request)
                .expect_err("later onboarding action must remain non-executable");
            assert_eq!(
                err.reason_code,
                reason_codes::INGRESS_COMPATIBILITY_ONLY,
                "{label}"
            );
            assert_eq!(err.failure_class, FailureClass::PolicyViolation, "{label}");
        }
    }

    #[test]
    fn slice_2c_onboarding_ask_missing_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_ask_missing_request(
            "onb-shape-1",
            "trace-onb-shape-1",
            RuntimeEntryTrigger::Explicit,
            Some("display_name"),
        );
        let first = normalize_turn_request(&request).expect("onboarding ask-missing normalized");
        let second = normalize_turn_request(&request).expect("onboarding ask-missing normalized");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingAskMissing {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                field_value,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(field_value, Some("display_name".to_string()));
            }
            other => panic!("expected onboarding ask-missing payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2d_platform_setup_receipt_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_platform_setup_request(
            "onb-platform-shape-1",
            "trace-onb-platform-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request).expect("onboarding platform setup normalized");
        let second =
            normalize_turn_request(&request).expect("onboarding platform setup normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingPlatformSetupReceipt {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                receipt_kind,
                receipt_ref,
                signer,
                payload_hash,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(receipt_kind, "DEVICE_ATTEST".to_string());
                assert_eq!(receipt_ref, "receipt-ref-1".to_string());
                assert_eq!(signer, "signer-a".to_string());
                assert_eq!(payload_hash, "payload-hash-1".to_string());
            }
            other => panic!("expected onboarding platform-setup payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2e_terms_accept_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_terms_accept_request(
            "onb-terms-shape-1",
            "trace-onb-terms-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request).expect("onboarding terms accept normalized");
        let second =
            normalize_turn_request(&request).expect("onboarding terms accept normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingTermsAccept {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                terms_version_id,
                accepted,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(terms_version_id, "terms-v1".to_string());
                assert!(accepted);
            }
            other => panic!("expected onboarding terms-accept payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2f_primary_device_confirm_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_primary_device_confirm_request(
            "onb-primary-shape-1",
            "trace-onb-primary-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding primary device confirm normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding primary device confirm normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingPrimaryDeviceConfirm {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
                proof_ok,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
                assert!(proof_ok);
            }
            other => panic!("expected onboarding primary-device-confirm payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2h_employee_photo_capture_send_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_employee_photo_capture_send_request(
            "onb-photo-shape-1",
            "trace-onb-photo-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding employee photo capture normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding employee photo capture normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingEmployeePhotoCaptureSend {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                photo_blob_ref,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(photo_blob_ref, "photo-blob-ref-1".to_string());
            }
            other => panic!("expected onboarding employee-photo payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2h_employee_sender_verify_commit_normalization_reuses_the_existing_canonical_carrier()
    {
        let request = onboarding_employee_sender_verify_commit_request(
            "onb-sender-shape-1",
            "trace-onb-sender-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding employee sender verify normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding employee sender verify normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingEmployeeSenderVerifyCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                decision,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(decision, SenderVerifyDecision::Confirm);
            }
            other => panic!("expected onboarding employee-sender-verify payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2i_voice_enroll_lock_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_voice_enroll_lock_request(
            "onb-voice-shape-1",
            "trace-onb-voice-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding voice enroll lock normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding voice enroll lock normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingVoiceEnrollLock {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
                sample_seed,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
                assert_eq!(sample_seed, "seed-1".to_string());
            }
            other => panic!("expected onboarding voice-enroll-lock payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2j_wake_enroll_start_draft_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_wake_enroll_start_draft_request(
            "onb-wake-start-shape-1",
            "trace-onb-wake-start-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request)
            .expect("onboarding wake enroll start draft normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding wake enroll start draft normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollStartDraft {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
            }
            other => panic!("expected onboarding wake-enroll-start payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2k_wake_enroll_sample_commit_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_wake_enroll_sample_commit_request(
            "onb-wake-sample-shape-1",
            "trace-onb-wake-sample-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request)
            .expect("onboarding wake enroll sample commit normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding wake enroll sample commit normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollSampleCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
                sample_pass,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
                assert!(sample_pass);
            }
            other => panic!("expected onboarding wake-enroll-sample payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2l_wake_enroll_complete_commit_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_wake_enroll_complete_commit_request(
            "onb-wake-complete-shape-1",
            "trace-onb-wake-complete-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request)
            .expect("onboarding wake enroll complete commit normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding wake enroll complete commit normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingWakeEnrollCompleteCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
            }
            other => panic!("expected onboarding wake-enroll-complete payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2m_emo_persona_lock_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_emo_persona_lock_request(
            "onb-emo-shape-1",
            "trace-onb-emo-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding emo persona lock normalized");
        let second =
            normalize_turn_request(&request).expect("onboarding emo persona lock normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingEmoPersonaLock {
                correlation_id,
                onboarding_session_id,
                tenant_id,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
            }
            other => panic!("expected onboarding emo-persona-lock payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2n_access_provision_commit_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_access_provision_commit_request(
            "onb-access-shape-1",
            "trace-onb-access-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request)
            .expect("onboarding access provision commit normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding access provision commit normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingAccessProvisionCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
            }
            other => panic!("expected onboarding access-provision payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2o_complete_commit_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_complete_commit_request(
            "onb-complete-shape-1",
            "trace-onb-complete-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first =
            normalize_turn_request(&request).expect("onboarding complete commit normalized");
        let second =
            normalize_turn_request(&request).expect("onboarding complete commit normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingCompleteCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
            }
            other => panic!("expected onboarding complete payload, got {other:?}"),
        }
    }

    #[test]
    fn pairing_completion_commit_normalization_reuses_the_existing_canonical_carrier() {
        let request = onboarding_pairing_completion_commit_request(
            "onb-pairing-shape-1",
            "trace-onb-pairing-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request)
            .expect("onboarding pairing completion commit normalized");
        let second = normalize_turn_request(&request)
            .expect("onboarding pairing completion commit normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, ONBOARDING_CONTINUE_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::OnboardingContinueCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::OnboardingPairingCompletionCommit {
                correlation_id,
                onboarding_session_id,
                tenant_id,
                device_id,
                session_id,
                session_attach_outcome,
            } => {
                assert_eq!(correlation_id, CorrelationId(101));
                assert_eq!(onboarding_session_id.as_str(), "onb-session-1");
                assert_eq!(tenant_id, Some("tenant-a".to_string()));
                assert_eq!(device_id.as_str(), "device-a");
                assert_eq!(session_id, SessionId(9_001));
                assert_eq!(
                    session_attach_outcome,
                    SessionAttachOutcome::NewSessionCreated
                );
            }
            other => panic!("expected onboarding pairing completion payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2c_malformed_onboarding_ask_missing_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_ask_missing_request(
            "onb-bad-1",
            "trace-onb-bad-1",
            RuntimeEntryTrigger::Explicit,
            Some("display_name"),
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("onboarding ask-missing idempotency mismatch must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2d_malformed_platform_setup_receipt_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_platform_setup_request(
            "onb-platform-bad-1",
            "trace-onb-platform-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
            request.compatibility_payload.as_mut()
        else {
            panic!("expected onboarding compatibility payload");
        };
        onboarding_request.action = AppOnboardingContinueAction::PlatformSetupReceipt {
            receipt_kind: String::new(),
            receipt_ref: "receipt-ref-1".to_string(),
            signer: "signer-a".to_string(),
            payload_hash: "payload-hash-1".to_string(),
        };

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed platform setup receipt must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2e_malformed_terms_accept_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_terms_accept_request(
            "onb-terms-bad-1",
            "trace-onb-terms-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
            request.compatibility_payload.as_mut()
        else {
            panic!("expected onboarding compatibility payload");
        };
        onboarding_request.action = AppOnboardingContinueAction::TermsAccept {
            terms_version_id: String::new(),
            accepted: true,
        };

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed terms accept must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2f_malformed_primary_device_confirm_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_primary_device_confirm_request(
            "onb-primary-bad-1",
            "trace-onb-primary-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed primary device confirm must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2h_malformed_employee_photo_capture_send_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_employee_photo_capture_send_request(
            "onb-photo-bad-1",
            "trace-onb-photo-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
            request.compatibility_payload.as_mut()
        else {
            panic!("expected onboarding compatibility payload");
        };
        onboarding_request.action = AppOnboardingContinueAction::EmployeePhotoCaptureSend {
            photo_blob_ref: String::new(),
        };

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed employee photo capture send must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2h_malformed_employee_sender_verify_commit_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_employee_sender_verify_commit_request(
            "onb-sender-bad-1",
            "trace-onb-sender-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed employee sender verify commit must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2i_malformed_voice_enroll_lock_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_voice_enroll_lock_request(
            "onb-voice-bad-1",
            "trace-onb-voice-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        let Some(CompatibilityRequestPayload::OnboardingContinue(onboarding_request)) =
            request.compatibility_payload.as_mut()
        else {
            panic!("expected onboarding compatibility payload");
        };
        onboarding_request.action = AppOnboardingContinueAction::VoiceEnrollLock {
            device_id: device("device-a"),
            sample_seed: String::new(),
        };

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed voice enroll lock must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2j_malformed_wake_enroll_start_draft_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_wake_enroll_start_draft_request(
            "onb-wake-start-bad-1",
            "trace-onb-wake-start-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed wake enroll start draft must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2k_malformed_wake_enroll_sample_commit_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_wake_enroll_sample_commit_request(
            "onb-wake-sample-bad-1",
            "trace-onb-wake-sample-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed wake enroll sample commit must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2l_malformed_wake_enroll_complete_commit_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_wake_enroll_complete_commit_request(
            "onb-wake-complete-bad-1",
            "trace-onb-wake-complete-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed wake enroll complete commit must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2m_malformed_emo_persona_lock_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_emo_persona_lock_request(
            "onb-emo-bad-1",
            "trace-onb-emo-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed emo persona lock must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2n_malformed_access_provision_commit_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_access_provision_commit_request(
            "onb-access-bad-1",
            "trace-onb-access-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed access provision commit must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2o_malformed_complete_commit_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = onboarding_complete_commit_request(
            "onb-complete-bad-1",
            "trace-onb-complete-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("malformed complete commit must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2c_onboarding_ask_missing_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_ask_missing_request(
                    "onb-session-1",
                    "trace-onb-session-1",
                    RuntimeEntryTrigger::Explicit,
                    None,
                ),
            )
            .expect("onboarding ask-missing request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingAskMissingCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
    }

    #[test]
    fn slice_2d_platform_setup_receipt_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_platform_setup_request(
                    "onb-platform-session-1",
                    "trace-onb-platform-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding platform setup request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingPlatformSetupReceiptCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
    }

    #[test]
    fn slice_2e_terms_accept_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_terms_accept_request(
                    "onb-terms-session-1",
                    "trace-onb-terms-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding terms accept request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingTermsAcceptCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
    }

    #[test]
    fn slice_2f_primary_device_confirm_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_primary_device_confirm_request(
                    "onb-primary-session-1",
                    "trace-onb-primary-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding primary device confirm request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingPrimaryDeviceConfirmCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2h_employee_photo_capture_send_reuses_session_foundation_and_pre_authority_stage_order(
    ) {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_employee_photo_capture_send_request(
                    "onb-photo-session-1",
                    "trace-onb-photo-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding employee photo capture request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmployeePhotoCaptureSendCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2h_employee_sender_verify_commit_reuses_session_foundation_and_pre_authority_stage_order(
    ) {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_employee_sender_verify_commit_request(
                    "onb-sender-session-1",
                    "trace-onb-sender-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding employee sender verify request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2i_voice_enroll_lock_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_voice_enroll_lock_request(
                    "onb-voice-session-1",
                    "trace-onb-voice-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding voice enroll lock request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingVoiceEnrollLockCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2j_wake_enroll_start_draft_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_start_draft_request(
                    "onb-wake-start-session-1",
                    "trace-onb-wake-start-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll start draft request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollStartDraftCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2k_wake_enroll_sample_commit_reuses_session_foundation_and_pre_authority_stage_order()
    {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_sample_commit_request(
                    "onb-wake-sample-session-1",
                    "trace-onb-wake-sample-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll sample commit request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollSampleCommitCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2l_wake_enroll_complete_commit_reuses_session_foundation_and_pre_authority_stage_order(
    ) {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_complete_commit_request(
                    "onb-wake-complete-session-1",
                    "trace-onb-wake-complete-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll complete commit request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingWakeEnrollCompleteCommitCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2m_emo_persona_lock_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_emo_persona_lock_request(
                    "onb-emo-session-1",
                    "trace-onb-emo-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding emo persona lock request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingEmoPersonaLockCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2n_access_provision_commit_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_access_provision_commit_request(
                    "onb-access-session-1",
                    "trace-onb-access-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding access provision commit request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingAccessProvisionCommitCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2o_complete_commit_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_complete_commit_request(
                    "onb-complete-session-1",
                    "trace-onb-complete-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding complete commit request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::OnboardingCompleteCommitCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
        assert!(ready.runtime_execution_envelope.identity_state.is_none());
        assert!(ready.runtime_execution_envelope.authority_state.is_none());
        assert!(ready.runtime_execution_envelope.persistence_state.is_none());
        assert!(ready.runtime_execution_envelope.proof_state.is_none());
        assert!(ready.runtime_execution_envelope.governance_state.is_none());
        assert!(ready.runtime_execution_envelope.computation_state.is_none());
        assert!(ready.runtime_execution_envelope.memory_state.is_none());
        assert!(ready.runtime_execution_envelope.law_state.is_none());
    }

    #[test]
    fn slice_2c_onboarding_ask_missing_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_ask_missing_request(
                    "onb-obs-1",
                    "trace-onb-obs-1",
                    RuntimeEntryTrigger::Explicit,
                    None,
                ),
            )
            .expect("onboarding ask-missing request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingAskMissingCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2d_platform_setup_receipt_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_platform_setup_request(
                    "onb-platform-obs-1",
                    "trace-onb-platform-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding platform setup request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingPlatformSetupReceiptCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2e_terms_accept_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_terms_accept_request(
                    "onb-terms-obs-1",
                    "trace-onb-terms-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding terms accept request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingTermsAcceptCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2f_primary_device_confirm_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_primary_device_confirm_request(
                    "onb-primary-obs-1",
                    "trace-onb-primary-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding primary device confirm request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingPrimaryDeviceConfirmCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2h_employee_photo_capture_send_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_employee_photo_capture_send_request(
                    "onb-photo-obs-1",
                    "trace-onb-photo-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding employee photo capture request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingEmployeePhotoCaptureSendCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2h_employee_sender_verify_commit_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_employee_sender_verify_commit_request(
                    "onb-sender-obs-1",
                    "trace-onb-sender-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding employee sender verify request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(
                TurnStartClassification::OnboardingEmployeeSenderVerifyCommitCompatibilityPrepared
            )
        );
    }

    #[test]
    fn slice_2i_voice_enroll_lock_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_voice_enroll_lock_request(
                    "onb-voice-obs-1",
                    "trace-onb-voice-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding voice enroll lock request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingVoiceEnrollLockCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2j_wake_enroll_start_draft_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_start_draft_request(
                    "onb-wake-start-obs-1",
                    "trace-onb-wake-start-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll start draft request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingWakeEnrollStartDraftCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2k_wake_enroll_sample_commit_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_sample_commit_request(
                    "onb-wake-sample-obs-1",
                    "trace-onb-wake-sample-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll sample commit request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingWakeEnrollSampleCommitCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2l_wake_enroll_complete_commit_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_wake_enroll_complete_commit_request(
                    "onb-wake-complete-obs-1",
                    "trace-onb-wake-complete-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding wake enroll complete commit request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingWakeEnrollCompleteCommitCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2m_emo_persona_lock_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_emo_persona_lock_request(
                    "onb-emo-obs-1",
                    "trace-onb-emo-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding emo persona lock request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingEmoPersonaLockCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2n_access_provision_commit_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_access_provision_commit_request(
                    "onb-access-obs-1",
                    "trace-onb-access-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding access provision commit request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingAccessProvisionCommitCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2o_complete_commit_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                onboarding_complete_commit_request(
                    "onb-complete-obs-1",
                    "trace-onb-complete-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("onboarding complete commit request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            ONBOARDING_CONTINUE_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::OnboardingCompleteCommitCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2b_invite_click_normalization_reuses_the_existing_canonical_carrier() {
        let request = invite_click_request(
            "invite-shape-1",
            "trace-invite-shape-1",
            RuntimeEntryTrigger::Explicit,
        );
        let first = normalize_turn_request(&request).expect("invite click normalized");
        let second = normalize_turn_request(&request).expect("invite click normalized again");
        assert_eq!(first, second);
        assert_eq!(first.canonical_route, INVITE_CLICK_ENDPOINT_PATH);
        assert_eq!(
            first.family,
            CanonicalIngressFamily::InviteClickCompatibility
        );
        assert_eq!(first.modality, CanonicalTurnModality::Compatibility);
        assert!(first.device_turn_sequence > 0);
        match first.payload {
            CanonicalTurnPayloadCarrier::InviteClick {
                token_id,
                app_instance_id,
                deep_link_nonce,
                ..
            } => {
                assert_eq!(token_id.as_str(), "invite-token-1");
                assert_eq!(app_instance_id, "app-instance-1".to_string());
                assert_eq!(deep_link_nonce, "deep-link-1".to_string());
            }
            other => panic!("expected invite-click payload, got {other:?}"),
        }
    }

    #[test]
    fn slice_2b_malformed_invite_click_inputs_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let mut request = invite_click_request(
            "invite-bad-1",
            "trace-invite-bad-1",
            RuntimeEntryTrigger::Explicit,
        );
        request.envelope_input.idempotency_key = "mismatch".to_string();

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("invite-click idempotency mismatch must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_ENVELOPE_INVALID);
        assert_eq!(err.failure_class, FailureClass::InvalidPayload);
    }

    #[test]
    fn slice_2b_invite_click_trigger_validation_is_deterministic_and_fail_closed() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let request = invite_click_request(
            "invite-trigger-1",
            "trace-invite-trigger-1",
            RuntimeEntryTrigger::WakeWord,
        );

        let err = foundation
            .process_turn_start(&runtime, &mut sessions, request)
            .expect_err("invite click wake-word trigger must fail closed");
        assert_eq!(err.reason_code, reason_codes::INGRESS_TRIGGER_INVALID);
        assert_eq!(err.failure_class, FailureClass::PolicyViolation);
    }

    #[test]
    fn slice_2b_invite_click_reuses_session_foundation_and_pre_authority_stage_order() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let result = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                invite_click_request(
                    "invite-session-1",
                    "trace-invite-session-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("invite-click request");
        let ready = match result {
            RuntimePreAuthorityTurnResult::Ready(ready) => ready,
            other => panic!("expected ready handoff, got {other:?}"),
        };
        assert_eq!(ready.response.session_state, SessionState::Active);
        assert_eq!(
            ready.response.classification,
            TurnStartClassification::InviteClickCompatibilityPrepared
        );
        assert_eq!(
            ready
                .stage_history
                .iter()
                .map(|record| record.stage)
                .collect::<Vec<_>>(),
            vec![
                PreAuthorityStage::IngressValidated,
                PreAuthorityStage::TriggerValidated,
                PreAuthorityStage::SessionResolved,
                PreAuthorityStage::EnvelopeCreated,
                PreAuthorityStage::TurnClassified,
                PreAuthorityStage::PreAuthorityReady,
            ]
        );
        assert_eq!(
            ready.runtime_execution_envelope.session_attach_outcome,
            Some(SessionAttachOutcome::NewSessionCreated)
        );
    }

    #[test]
    fn slice_2b_invite_click_observability_stays_bounded_to_section03() {
        let runtime = ready_runtime();
        let mut foundation = foundation();
        let mut sessions = RuntimeSessionFoundation::default();
        let _ = foundation
            .process_turn_start(
                &runtime,
                &mut sessions,
                invite_click_request(
                    "invite-obs-1",
                    "trace-invite-obs-1",
                    RuntimeEntryTrigger::Explicit,
                ),
            )
            .expect("invite-click request");

        assert_eq!(foundation.counters().normalized_turns, 1);
        assert_eq!(foundation.counters().ready_handoffs, 1);
        assert_eq!(foundation.events().len(), 2);
        assert_eq!(
            foundation.events()[0].route_path,
            INVITE_CLICK_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].route_path,
            INVITE_CLICK_ENDPOINT_PATH
        );
        assert_eq!(
            foundation.events()[1].classification,
            Some(TurnStartClassification::InviteClickCompatibilityPrepared)
        );
    }

    #[test]
    fn slice_2a_registers_only_bounded_services_without_authority_or_persistence_drift() {
        let clock = FixedClock::new(1_000);
        let secrets = StaticSecretsProvider::default()
            .with_secret(ProviderSecretId::OpenAIApiKey, "secret")
            .expect("secret should be valid");
        let mut container =
            RuntimeServiceContainer::with_startup_foundation(clock, secrets).expect("services");
        RuntimeRouter::register_slice_1b_foundation_services(&mut container)
            .expect("slice 1b services");
        RuntimeSessionFoundation::register_slice_1c_session_foundation_services(&mut container)
            .expect("slice 1c services");
        RuntimeIngressTurnFoundation::register_slice_2a_foundation_services(&mut container)
            .expect("slice 2a services");
        let service_ids = container.service_ids();
        let slice_2a_service_ids: Vec<&str> = service_ids
            .iter()
            .copied()
            .filter(|id| {
                id.starts_with("runtime_turn_") || *id == "runtime_ingress_turn_foundation"
            })
            .collect();
        assert!(service_ids.contains(&"runtime_ingress_turn_foundation"));
        assert!(service_ids.contains(&"runtime_turn_request_normalizer"));
        assert!(service_ids.contains(&"runtime_turn_session_binder"));
        assert!(!slice_2a_service_ids
            .iter()
            .any(|id| id.starts_with("runtime_authority")));
        assert!(!slice_2a_service_ids
            .iter()
            .any(|id| id.starts_with("runtime_persistence")));
        assert!(!slice_2a_service_ids.iter().any(|id| id.contains("apple")));
    }
}
