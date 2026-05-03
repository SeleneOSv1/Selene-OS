#![forbid(unsafe_code)]

use crate::app_ingress::{AppOnboardingContinueAction, AppOnboardingContinueRequest};
use selene_kernel_contracts::ph1_voice_id::UserId;
use selene_kernel_contracts::ph1j::{
    BenchmarkComparisonOutcome, BenchmarkResultPacket, BenchmarkTargetPacket,
    BenchmarkTargetStatus, CorrelationId, DeviceId, TurnId,
};
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
    SessionTurnResolution, Stage5TurnAuthorityDisposition, Stage5TurnAuthorityPacket,
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
    pub const STAGE7_WAKE_ATTENTION_ONLY: &str = "stage7_wake_attention_only";
    pub const STAGE7_EXPLICIT_ACTIVATION_ONLY: &str = "stage7_explicit_activation_only";
    pub const STAGE7_SIDE_BUTTON_EXPLICIT_ONLY: &str = "stage7_side_button_explicit_only";
    pub const STAGE7_RECORD_ARTIFACT_DEFERRED: &str = "stage7_record_artifact_deferred";
    pub const STAGE8_AUDIO_SUBSTRATE_ONLY: &str = "stage8_audio_substrate_only";
    pub const STAGE8_PARTIAL_TRANSCRIPT_PREVIEW_ONLY: &str =
        "stage8_partial_transcript_preview_only";
    pub const STAGE8_FINAL_TRANSCRIPT_COMMIT_BOUNDARY: &str =
        "stage8_final_transcript_commit_boundary";
    pub const STAGE8_BACKGROUND_OR_SELF_ECHO_BLOCKED: &str =
        "stage8_background_or_self_echo_blocked";
    pub const STAGE8_RECORD_AUDIO_ARTIFACT_ONLY: &str = "stage8_record_audio_artifact_only";
    pub const STAGE8B_VAD_ENDPOINT_BOUNDARY_ONLY: &str = "stage8b_vad_endpoint_boundary_only";
    pub const STAGE8B_CONFIDENCE_GATE_REJECTED: &str = "stage8b_confidence_gate_rejected";
    pub const STAGE8C_AUDIO_SCENE_BOUNDARY_ONLY: &str = "stage8c_audio_scene_boundary_only";
    pub const STAGE8C_LISTENING_SCENE_BLOCKED: &str = "stage8c_listening_scene_blocked";
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage7ActivationDisposition {
    WakeAttentionOnly,
    ExplicitActivationOnly,
    SideButtonExplicitOnly,
    RecordArtifactDeferred,
}

impl Stage7ActivationDisposition {
    pub const fn default_reason_code(self) -> &'static str {
        match self {
            Stage7ActivationDisposition::WakeAttentionOnly => {
                reason_codes::STAGE7_WAKE_ATTENTION_ONLY
            }
            Stage7ActivationDisposition::ExplicitActivationOnly => {
                reason_codes::STAGE7_EXPLICIT_ACTIVATION_ONLY
            }
            Stage7ActivationDisposition::SideButtonExplicitOnly => {
                reason_codes::STAGE7_SIDE_BUTTON_EXPLICIT_ONLY
            }
            Stage7ActivationDisposition::RecordArtifactDeferred => {
                reason_codes::STAGE7_RECORD_ARTIFACT_DEFERRED
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage7ActivationWorkAuthority {
    pub can_open_or_resume_session: bool,
    pub can_update_attention_state: bool,
    pub can_understand_intent: bool,
    pub can_answer: bool,
    pub can_search: bool,
    pub can_call_providers: bool,
    pub can_trigger_voice_id_matching: bool,
    pub can_authorize: bool,
    pub can_route_tools: bool,
    pub can_emit_tts: bool,
    pub can_execute_protected_mutation: bool,
    pub can_connector_write: bool,
}

impl Stage7ActivationWorkAuthority {
    pub const fn session_attention_only() -> Self {
        Self {
            can_open_or_resume_session: true,
            can_update_attention_state: true,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
        }
    }

    pub const fn deferred_artifact_only() -> Self {
        Self {
            can_open_or_resume_session: false,
            can_update_attention_state: false,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
        }
    }

    pub const fn can_perform_downstream_work(self) -> bool {
        self.can_understand_intent
            || self.can_answer
            || self.can_search
            || self.can_call_providers
            || self.can_trigger_voice_id_matching
            || self.can_authorize
            || self.can_route_tools
            || self.can_emit_tts
            || self.can_execute_protected_mutation
            || self.can_connector_write
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage7ActivationContextPacket {
    pub activation: Stage4ActivationPacket,
    pub disposition: Stage7ActivationDisposition,
    pub reason_code: &'static str,
    pub session_id: Option<SessionId>,
    pub turn_id: Option<TurnId>,
    pub activation_id: String,
    pub wake_event_id: Option<String>,
    pub wake_artifact_id: Option<String>,
    pub consent_state_id: Option<String>,
    pub device_trust_id: Option<String>,
    pub provider_budget_id: Option<String>,
    pub access_context_id: Option<String>,
    pub audit_id: Option<String>,
    pub iphone_always_listening_attempt: bool,
    pub work_authority: Stage7ActivationWorkAuthority,
}

impl Stage7ActivationContextPacket {
    pub fn from_activation(
        activation: Stage4ActivationPacket,
        activation_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        let disposition = Self::disposition_for_source(activation.source);
        let work_authority = Self::work_authority_for(disposition);
        let packet = Self {
            session_id: activation.session_hint,
            turn_id: None,
            activation_id: activation_id.into(),
            wake_event_id: None,
            wake_artifact_id: None,
            consent_state_id: activation.consent_state_id.clone(),
            device_trust_id: activation.device_trust_ref.clone(),
            provider_budget_id: activation.provider_budget_ref.clone(),
            access_context_id: None,
            audit_id: activation.audit_id.clone(),
            reason_code: disposition.default_reason_code(),
            disposition,
            iphone_always_listening_attempt: false,
            work_authority,
            activation,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn with_access_context_id(
        mut self,
        access_context_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        self.access_context_id = Some(access_context_id.into());
        self.validate()?;
        Ok(self)
    }

    pub fn with_wake_refs(
        mut self,
        wake_event_id: impl Into<String>,
        wake_artifact_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        self.wake_event_id = Some(wake_event_id.into());
        self.wake_artifact_id = Some(wake_artifact_id.into());
        self.validate()?;
        Ok(self)
    }

    pub const fn can_perform_downstream_work(&self) -> bool {
        self.work_authority.can_perform_downstream_work()
    }

    pub const fn can_only_open_or_resume_session(&self) -> bool {
        self.work_authority.can_open_or_resume_session
            && self.work_authority.can_update_attention_state
            && !self.work_authority.can_perform_downstream_work()
    }

    const fn disposition_for_source(source: Stage4ActivationSource) -> Stage7ActivationDisposition {
        match source {
            Stage4ActivationSource::WakeCandidate => Stage7ActivationDisposition::WakeAttentionOnly,
            Stage4ActivationSource::SideButton => {
                Stage7ActivationDisposition::SideButtonExplicitOnly
            }
            Stage4ActivationSource::TypedInput | Stage4ActivationSource::ExplicitMic => {
                Stage7ActivationDisposition::ExplicitActivationOnly
            }
            Stage4ActivationSource::RecordButton => {
                Stage7ActivationDisposition::RecordArtifactDeferred
            }
        }
    }

    const fn work_authority_for(
        disposition: Stage7ActivationDisposition,
    ) -> Stage7ActivationWorkAuthority {
        match disposition {
            Stage7ActivationDisposition::WakeAttentionOnly
            | Stage7ActivationDisposition::ExplicitActivationOnly
            | Stage7ActivationDisposition::SideButtonExplicitOnly => {
                Stage7ActivationWorkAuthority::session_attention_only()
            }
            Stage7ActivationDisposition::RecordArtifactDeferred => {
                Stage7ActivationWorkAuthority::deferred_artifact_only()
            }
        }
    }
}

impl Validate for Stage7ActivationContextPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.activation.validate()?;
        validate_stage4_ref(
            "stage7_activation_context_packet.activation_id",
            &self.activation_id,
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.wake_event_id",
            self.wake_event_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.wake_artifact_id",
            self.wake_artifact_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.consent_state_id",
            self.consent_state_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.device_trust_id",
            self.device_trust_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.provider_budget_id",
            self.provider_budget_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.access_context_id",
            self.access_context_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage7_activation_context_packet.audit_id",
            self.audit_id.as_deref(),
        )?;
        if self.reason_code != self.disposition.default_reason_code() {
            return Err(ContractViolation::InvalidValue {
                field: "stage7_activation_context_packet.reason_code",
                reason: "must match disposition",
            });
        }
        if self.iphone_always_listening_attempt {
            return Err(ContractViolation::InvalidValue {
                field: "stage7_activation_context_packet.iphone_always_listening_attempt",
                reason: "iPhone always-listening wake attempts are not allowed",
            });
        }
        if self.turn_id.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "stage7_activation_context_packet.turn_id",
                reason: "activation alone cannot create a turn",
            });
        }
        if self.work_authority.can_perform_downstream_work() {
            return Err(ContractViolation::InvalidValue {
                field: "stage7_activation_context_packet.work_authority",
                reason: "activation cannot understand, answer, search, call providers, identify, authorize, speak, or execute",
            });
        }
        match self.disposition {
            Stage7ActivationDisposition::WakeAttentionOnly => {
                if self.activation.source != Stage4ActivationSource::WakeCandidate
                    || self.activation.platform_context.platform_type == AppPlatform::Ios
                    || self.activation.platform_context.requested_trigger
                        != RuntimeEntryTrigger::WakeWord
                    || !self.work_authority.can_open_or_resume_session
                    || !self.work_authority.can_update_attention_state
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage7_activation_context_packet",
                        reason: "wake activation can only open/resume attention on non-iPhone wake surfaces",
                    });
                }
                if self.wake_artifact_id.is_some() && self.consent_state_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage7_activation_context_packet.consent_state_id",
                        reason: "wake artifact references require consent state",
                    });
                }
            }
            Stage7ActivationDisposition::SideButtonExplicitOnly => {
                if self.activation.source != Stage4ActivationSource::SideButton
                    || self.activation.platform_context.platform_type != AppPlatform::Ios
                    || self.activation.platform_context.requested_trigger
                        != RuntimeEntryTrigger::Explicit
                    || self.wake_event_id.is_some()
                    || self.wake_artifact_id.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage7_activation_context_packet",
                        reason: "side-button activation must be explicit iPhone activation with no wake artifacts",
                    });
                }
            }
            Stage7ActivationDisposition::ExplicitActivationOnly => {
                if !matches!(
                    self.activation.source,
                    Stage4ActivationSource::TypedInput | Stage4ActivationSource::ExplicitMic
                ) || self.activation.platform_context.requested_trigger
                    != RuntimeEntryTrigger::Explicit
                    || self.wake_event_id.is_some()
                    || self.wake_artifact_id.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage7_activation_context_packet",
                        reason: "explicit activation cannot carry wake event or artifact state",
                    });
                }
            }
            Stage7ActivationDisposition::RecordArtifactDeferred => {
                if self.activation.source != Stage4ActivationSource::RecordButton
                    || self.work_authority.can_open_or_resume_session
                    || self.work_authority.can_update_attention_state
                    || self.wake_event_id.is_some()
                    || self.wake_artifact_id.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage7_activation_context_packet",
                        reason: "record activation remains deferred to the artifact lane",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8TranscriptGateKind {
    AudioSubstrateOnly,
    VadEndpointBoundaryOnly,
    PartialTranscriptPreviewOnly,
    FinalTranscriptCommitBoundary,
    ConfidenceGateRejected,
    AudioSceneBoundaryOnly,
    ListeningSceneBlocked,
    BackgroundOrSelfEchoBlocked,
    RecordAudioArtifactOnly,
}

impl Stage8TranscriptGateKind {
    pub const fn default_reason_code(self) -> &'static str {
        match self {
            Stage8TranscriptGateKind::AudioSubstrateOnly => {
                reason_codes::STAGE8_AUDIO_SUBSTRATE_ONLY
            }
            Stage8TranscriptGateKind::VadEndpointBoundaryOnly => {
                reason_codes::STAGE8B_VAD_ENDPOINT_BOUNDARY_ONLY
            }
            Stage8TranscriptGateKind::PartialTranscriptPreviewOnly => {
                reason_codes::STAGE8_PARTIAL_TRANSCRIPT_PREVIEW_ONLY
            }
            Stage8TranscriptGateKind::FinalTranscriptCommitBoundary => {
                reason_codes::STAGE8_FINAL_TRANSCRIPT_COMMIT_BOUNDARY
            }
            Stage8TranscriptGateKind::ConfidenceGateRejected => {
                reason_codes::STAGE8B_CONFIDENCE_GATE_REJECTED
            }
            Stage8TranscriptGateKind::AudioSceneBoundaryOnly => {
                reason_codes::STAGE8C_AUDIO_SCENE_BOUNDARY_ONLY
            }
            Stage8TranscriptGateKind::ListeningSceneBlocked => {
                reason_codes::STAGE8C_LISTENING_SCENE_BLOCKED
            }
            Stage8TranscriptGateKind::BackgroundOrSelfEchoBlocked => {
                reason_codes::STAGE8_BACKGROUND_OR_SELF_ECHO_BLOCKED
            }
            Stage8TranscriptGateKind::RecordAudioArtifactOnly => {
                reason_codes::STAGE8_RECORD_AUDIO_ARTIFACT_ONLY
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8EndpointState {
    NotEvaluated,
    VadSignalOnly,
    EndpointCandidate,
    EndpointFinal,
}

impl Stage8EndpointState {
    pub const fn is_endpoint_signal(self) -> bool {
        matches!(
            self,
            Stage8EndpointState::VadSignalOnly
                | Stage8EndpointState::EndpointCandidate
                | Stage8EndpointState::EndpointFinal
        )
    }

    pub const fn is_final(self) -> bool {
        matches!(self, Stage8EndpointState::EndpointFinal)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8ConfidenceGateDisposition {
    NotEvaluated,
    Passed,
    RejectedLowConfidence,
    RejectedLowCoverage,
    RejectedEmptyTranscript,
    RejectedGarbledTranscript,
    RejectedEchoSuspect,
    RejectedBackgroundOrNonUser,
    RejectedStaleOrClosedTurn,
}

impl Stage8ConfidenceGateDisposition {
    pub const fn is_rejection(self) -> bool {
        matches!(
            self,
            Stage8ConfidenceGateDisposition::RejectedLowConfidence
                | Stage8ConfidenceGateDisposition::RejectedLowCoverage
                | Stage8ConfidenceGateDisposition::RejectedEmptyTranscript
                | Stage8ConfidenceGateDisposition::RejectedGarbledTranscript
                | Stage8ConfidenceGateDisposition::RejectedEchoSuspect
                | Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser
                | Stage8ConfidenceGateDisposition::RejectedStaleOrClosedTurn
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8ProtectedSlotDisposition {
    NotApplicable,
    NoProtectedSlots,
    HighConfidenceProtectedSlots,
    ClarificationRequired,
    FailClosed,
    DeferredToStage10Or12,
}

impl Stage8ProtectedSlotDisposition {
    pub const fn blocks_commit(self) -> bool {
        matches!(
            self,
            Stage8ProtectedSlotDisposition::ClarificationRequired
                | Stage8ProtectedSlotDisposition::FailClosed
                | Stage8ProtectedSlotDisposition::DeferredToStage10Or12
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8ProtectedSlotKind {
    Name,
    Date,
    Amount,
    Address,
    Recipient,
    AccountOrActionIdentifier,
    AuthorizationRelevantField,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8ProtectedSlotUncertainty {
    pub slot_kind: Stage8ProtectedSlotKind,
    pub field_hint: String,
    pub confidence_bp: u16,
}

impl Stage8ProtectedSlotUncertainty {
    pub fn v1(
        slot_kind: Stage8ProtectedSlotKind,
        field_hint: impl Into<String>,
        confidence_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let uncertainty = Self {
            slot_kind,
            field_hint: field_hint.into(),
            confidence_bp,
        };
        uncertainty.validate()?;
        Ok(uncertainty)
    }
}

impl Validate for Stage8ProtectedSlotUncertainty {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8_protected_slot_uncertainty.field_hint",
            &self.field_hint,
        )?;
        if self.confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_protected_slot_uncertainty.confidence_bp",
                reason: "must be <= 10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8NoiseDegradationClass {
    NotEvaluated,
    Clear,
    Moderate,
    High,
    Severe,
}

impl Stage8NoiseDegradationClass {
    pub const fn blocks_commit(self) -> bool {
        matches!(
            self,
            Stage8NoiseDegradationClass::High | Stage8NoiseDegradationClass::Severe
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8ForegroundSpeakerPacket {
    pub speaker_segment_id: String,
    pub foreground_speaker_id: Option<String>,
    pub foreground_confidence_bp: u16,
    pub is_user_speech_candidate: bool,
    pub advisory_only: bool,
}

impl Stage8ForegroundSpeakerPacket {
    pub fn advisory(
        speaker_segment_id: impl Into<String>,
        foreground_speaker_id: Option<String>,
        foreground_confidence_bp: u16,
        is_user_speech_candidate: bool,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            speaker_segment_id: speaker_segment_id.into(),
            foreground_speaker_id,
            foreground_confidence_bp,
            is_user_speech_candidate,
            advisory_only: true,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for Stage8ForegroundSpeakerPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8_foreground_speaker_packet.speaker_segment_id",
            &self.speaker_segment_id,
        )?;
        validate_stage4_optional_ref(
            "stage8_foreground_speaker_packet.foreground_speaker_id",
            self.foreground_speaker_id.as_deref(),
        )?;
        if self.foreground_confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_foreground_speaker_packet.foreground_confidence_bp",
                reason: "must be <= 10000",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_foreground_speaker_packet.advisory_only",
                reason: "foreground speaker evidence is advisory and cannot identify or authorize",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8AddressedToSelenePacket {
    pub addressed_to_selene_id: String,
    pub confidence_bp: u16,
    pub addressed: bool,
    pub advisory_only: bool,
}

impl Stage8AddressedToSelenePacket {
    pub fn advisory(
        addressed_to_selene_id: impl Into<String>,
        confidence_bp: u16,
        addressed: bool,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            addressed_to_selene_id: addressed_to_selene_id.into(),
            confidence_bp,
            addressed,
            advisory_only: true,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for Stage8AddressedToSelenePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8_addressed_to_selene_packet.addressed_to_selene_id",
            &self.addressed_to_selene_id,
        )?;
        if self.confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_addressed_to_selene_packet.confidence_bp",
                reason: "must be <= 10000",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_addressed_to_selene_packet.advisory_only",
                reason: "addressed-to-Selene evidence is advisory and cannot identify or authorize",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8AudioSceneDisposition {
    AdvisoryOnly,
    CleanForegroundAddressed,
    BlockedLowAddressingConfidence,
    BlockedBackgroundSpeech,
    BlockedSelfEcho,
    BlockedOverlappingSpeakers,
    BlockedUnknownOrNonUserSpeaker,
    BlockedHighNoiseOrDegradation,
    BlockedRecordArtifactOnly,
}

impl Stage8AudioSceneDisposition {
    pub const fn blocks_commit(self) -> bool {
        matches!(
            self,
            Stage8AudioSceneDisposition::BlockedLowAddressingConfidence
                | Stage8AudioSceneDisposition::BlockedBackgroundSpeech
                | Stage8AudioSceneDisposition::BlockedSelfEcho
                | Stage8AudioSceneDisposition::BlockedOverlappingSpeakers
                | Stage8AudioSceneDisposition::BlockedUnknownOrNonUserSpeaker
                | Stage8AudioSceneDisposition::BlockedHighNoiseOrDegradation
                | Stage8AudioSceneDisposition::BlockedRecordArtifactOnly
        )
    }

    pub const fn clean_for_commit(self) -> bool {
        matches!(self, Stage8AudioSceneDisposition::CleanForegroundAddressed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8AudioScenePacket {
    pub audio_scene_id: String,
    pub foreground_speaker: Option<Stage8ForegroundSpeakerPacket>,
    pub addressed_to_selene: Option<Stage8AddressedToSelenePacket>,
    pub disposition: Stage8AudioSceneDisposition,
    pub noise_degradation: Stage8NoiseDegradationClass,
    pub echo_suspect: bool,
    pub self_echo_suspect: bool,
    pub background_speech: bool,
    pub overlapping_speakers: bool,
    pub unknown_or_non_user_speaker: bool,
    pub barge_in_or_interruption_marker: bool,
    pub record_mode_audio: bool,
    pub reason_code: String,
}

impl Stage8AudioScenePacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        audio_scene_id: impl Into<String>,
        foreground_speaker: Option<Stage8ForegroundSpeakerPacket>,
        addressed_to_selene: Option<Stage8AddressedToSelenePacket>,
        disposition: Stage8AudioSceneDisposition,
        noise_degradation: Stage8NoiseDegradationClass,
        echo_suspect: bool,
        self_echo_suspect: bool,
        background_speech: bool,
        overlapping_speakers: bool,
        unknown_or_non_user_speaker: bool,
        barge_in_or_interruption_marker: bool,
        record_mode_audio: bool,
        reason_code: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            audio_scene_id: audio_scene_id.into(),
            foreground_speaker,
            addressed_to_selene,
            disposition,
            noise_degradation,
            echo_suspect,
            self_echo_suspect,
            background_speech,
            overlapping_speakers,
            unknown_or_non_user_speaker,
            barge_in_or_interruption_marker,
            record_mode_audio,
            reason_code: reason_code.into(),
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn has_blocking_signal(&self) -> bool {
        self.disposition.blocks_commit()
            || self.noise_degradation.blocks_commit()
            || self.echo_suspect
            || self.self_echo_suspect
            || self.background_speech
            || self.overlapping_speakers
            || self.unknown_or_non_user_speaker
            || self.record_mode_audio
    }

    pub fn clean_foreground_addressed(&self) -> bool {
        self.disposition.clean_for_commit()
            && !self.has_blocking_signal()
            && matches!(
                self.foreground_speaker.as_ref(),
                Some(foreground)
                    if foreground.advisory_only
                        && foreground.is_user_speech_candidate
                        && foreground.foreground_confidence_bp >= 8_000
            )
            && matches!(
                self.addressed_to_selene.as_ref(),
                Some(addressed)
                    if addressed.advisory_only && addressed.addressed && addressed.confidence_bp >= 8_000
            )
    }
}

impl Validate for Stage8AudioScenePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8_audio_scene_packet.audio_scene_id",
            &self.audio_scene_id,
        )?;
        validate_stage4_ref("stage8_audio_scene_packet.reason_code", &self.reason_code)?;
        if let Some(foreground) = self.foreground_speaker.as_ref() {
            foreground.validate()?;
        }
        if let Some(addressed) = self.addressed_to_selene.as_ref() {
            addressed.validate()?;
        }
        if self.disposition.clean_for_commit()
            && (self.foreground_speaker.is_none() || self.addressed_to_selene.is_none())
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_audio_scene_packet.disposition",
                reason:
                    "clean foreground-addressed scene requires foreground and addressed evidence",
            });
        }
        if self.disposition == Stage8AudioSceneDisposition::BlockedLowAddressingConfidence
            && !matches!(
                self.addressed_to_selene.as_ref(),
                Some(addressed) if !addressed.addressed || addressed.confidence_bp < 8_000
            )
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_audio_scene_packet.addressed_to_selene",
                reason: "low-addressing block requires low or negative addressed evidence",
            });
        }
        if self.disposition == Stage8AudioSceneDisposition::BlockedHighNoiseOrDegradation
            && !self.noise_degradation.blocks_commit()
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_audio_scene_packet.noise_degradation",
                reason: "high-noise block requires high or severe degradation",
            });
        }
        if self.disposition == Stage8AudioSceneDisposition::BlockedRecordArtifactOnly
            && !self.record_mode_audio
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_audio_scene_packet.record_mode_audio",
                reason: "record artifact block requires record-mode audio",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage8DEditCounts {
    pub substitutions: u16,
    pub insertions: u16,
    pub deletions: u16,
    pub reference_len: u16,
    pub observed_len: u16,
}

impl Stage8DEditCounts {
    pub const fn total_errors(self) -> u16 {
        self.substitutions
            .saturating_add(self.insertions)
            .saturating_add(self.deletions)
    }

    pub const fn error_rate_bp(self) -> u16 {
        let denominator = if self.reference_len == 0 {
            1
        } else {
            self.reference_len
        };
        let value = (self.total_errors() as u32)
            .saturating_mul(10_000)
            .saturating_div(denominator as u32);
        if value > 10_000 {
            10_000
        } else {
            value as u16
        }
    }

    fn candidate(
        substitutions: u16,
        insertions: u16,
        deletions: u16,
        reference_len: u16,
        observed_len: u16,
    ) -> Self {
        Self {
            substitutions,
            insertions,
            deletions,
            reference_len,
            observed_len,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8DConfidenceBucket {
    NotMeasured,
    High,
    Medium,
    Low,
    Rejected,
}

impl Stage8DConfidenceBucket {
    pub const fn from_confidence_bp(confidence_bp: Option<u16>, rejected: bool) -> Self {
        if rejected {
            return Self::Rejected;
        }
        match confidence_bp {
            Some(value) if value >= 9_000 => Self::High,
            Some(value) if value >= 8_000 => Self::Medium,
            Some(_) => Self::Low,
            None => Self::NotMeasured,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8DEndpointLatencyClass {
    OnTime,
    Late,
    Premature,
    TimeoutOrDegraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8DSignalBucket {
    NotMeasured,
    Low,
    Medium,
    High,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8DTranscriptFixture {
    pub fixture_id: String,
    pub reference_transcript: String,
    pub observed_transcript: String,
    pub protected_tokens: Vec<String>,
    pub mixed_language_tokens: Vec<String>,
    pub slang_filler_tokens: Vec<String>,
}

impl Stage8DTranscriptFixture {
    pub fn v1(
        fixture_id: impl Into<String>,
        reference_transcript: impl Into<String>,
        observed_transcript: impl Into<String>,
        protected_tokens: Vec<String>,
        mixed_language_tokens: Vec<String>,
        slang_filler_tokens: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let fixture = Self {
            fixture_id: fixture_id.into(),
            reference_transcript: reference_transcript.into(),
            observed_transcript: observed_transcript.into(),
            protected_tokens,
            mixed_language_tokens,
            slang_filler_tokens,
        };
        fixture.validate()?;
        Ok(fixture)
    }

    pub fn score(
        &self,
        metric_id: impl Into<String>,
    ) -> Result<Stage8DTranscriptMetricPacket, ContractViolation> {
        let metric_id = metric_id.into();
        let normalized_reference = stage8d_normalize_transcript(&self.reference_transcript);
        let normalized_observed = stage8d_normalize_transcript(&self.observed_transcript);
        let reference_words = stage8d_words(&normalized_reference);
        let observed_words = stage8d_words(&normalized_observed);
        let word_edits = stage8d_edit_counts(&reference_words, &observed_words)?;
        let reference_chars: Vec<char> = normalized_reference.chars().collect();
        let observed_chars: Vec<char> = normalized_observed.chars().collect();
        let char_edits = stage8d_edit_counts(&reference_chars, &observed_chars)?;
        let protected_token_mismatch_count =
            stage8d_token_mismatch_count(&self.protected_tokens, &normalized_observed)?;
        let mixed_language_preserved =
            stage8d_all_tokens_present(&self.mixed_language_tokens, &normalized_observed)?;
        let slang_filler_preserved =
            stage8d_all_tokens_present(&self.slang_filler_tokens, &normalized_observed)?;
        let metric = Stage8DTranscriptMetricPacket {
            metric_id,
            fixture_id: self.fixture_id.clone(),
            exact_match: self.reference_transcript == self.observed_transcript,
            normalized_match: normalized_reference == normalized_observed,
            reference_word_count: reference_words.len() as u16,
            observed_word_count: observed_words.len() as u16,
            word_edits,
            char_edits,
            protected_token_mismatch_count,
            empty_transcript: normalized_observed.is_empty(),
            garbled_transcript: stage8d_is_garbled(&self.observed_transcript),
            mixed_language_preserved,
            slang_filler_preserved,
            reason_code: "stage8d_transcript_fixture_scored".to_string(),
        };
        metric.validate()?;
        Ok(metric)
    }
}

impl Validate for Stage8DTranscriptFixture {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8d_transcript_fixture.fixture_id", &self.fixture_id)?;
        validate_stage8d_text(
            "stage8d_transcript_fixture.reference_transcript",
            &self.reference_transcript,
        )?;
        validate_stage8d_text(
            "stage8d_transcript_fixture.observed_transcript",
            &self.observed_transcript,
        )?;
        validate_stage8d_token_list(
            "stage8d_transcript_fixture.protected_tokens",
            &self.protected_tokens,
        )?;
        validate_stage8d_token_list(
            "stage8d_transcript_fixture.mixed_language_tokens",
            &self.mixed_language_tokens,
        )?;
        validate_stage8d_token_list(
            "stage8d_transcript_fixture.slang_filler_tokens",
            &self.slang_filler_tokens,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8DTranscriptMetricPacket {
    pub metric_id: String,
    pub fixture_id: String,
    pub exact_match: bool,
    pub normalized_match: bool,
    pub reference_word_count: u16,
    pub observed_word_count: u16,
    pub word_edits: Stage8DEditCounts,
    pub char_edits: Stage8DEditCounts,
    pub protected_token_mismatch_count: u16,
    pub empty_transcript: bool,
    pub garbled_transcript: bool,
    pub mixed_language_preserved: bool,
    pub slang_filler_preserved: bool,
    pub reason_code: String,
}

impl Stage8DTranscriptMetricPacket {
    pub const fn word_error_rate_bp(&self) -> u16 {
        self.word_edits.error_rate_bp()
    }

    pub const fn character_error_rate_bp(&self) -> u16 {
        self.char_edits.error_rate_bp()
    }
}

impl Validate for Stage8DTranscriptMetricPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8d_transcript_metric.metric_id", &self.metric_id)?;
        validate_stage4_ref("stage8d_transcript_metric.fixture_id", &self.fixture_id)?;
        validate_stage4_ref("stage8d_transcript_metric.reason_code", &self.reason_code)?;
        if self.word_edits.reference_len != self.reference_word_count
            || self.word_edits.observed_len != self.observed_word_count
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_transcript_metric.word_edits",
                reason: "word edit counts must match word counters",
            });
        }
        if self.empty_transcript && self.normalized_match {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_transcript_metric.empty_transcript",
                reason: "empty observed transcript cannot be a normalized match",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8DEndpointLatencyMetricPacket {
    pub metric_id: String,
    pub speech_start_ms: u32,
    pub speech_end_ms: u32,
    pub endpoint_candidate_ms: u32,
    pub endpoint_final_ms: u32,
    pub endpoint_latency_ms: u32,
    pub classification: Stage8DEndpointLatencyClass,
    pub reason_code: String,
}

impl Stage8DEndpointLatencyMetricPacket {
    pub fn v1(
        metric_id: impl Into<String>,
        speech_start_ms: u32,
        speech_end_ms: u32,
        endpoint_candidate_ms: u32,
        endpoint_final_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let endpoint_latency_ms = endpoint_final_ms.saturating_sub(speech_end_ms);
        let classification = if endpoint_final_ms < speech_end_ms {
            Stage8DEndpointLatencyClass::Premature
        } else if endpoint_latency_ms > 3_000 {
            Stage8DEndpointLatencyClass::TimeoutOrDegraded
        } else if endpoint_latency_ms > 800 {
            Stage8DEndpointLatencyClass::Late
        } else {
            Stage8DEndpointLatencyClass::OnTime
        };
        let packet = Self {
            metric_id: metric_id.into(),
            speech_start_ms,
            speech_end_ms,
            endpoint_candidate_ms,
            endpoint_final_ms,
            endpoint_latency_ms,
            classification,
            reason_code: "stage8d_endpoint_latency_scored".to_string(),
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for Stage8DEndpointLatencyMetricPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8d_endpoint_latency.metric_id", &self.metric_id)?;
        validate_stage4_ref("stage8d_endpoint_latency.reason_code", &self.reason_code)?;
        if self.speech_start_ms >= self.speech_end_ms {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_endpoint_latency.speech_window",
                reason: "speech_start_ms must be < speech_end_ms",
            });
        }
        if self.endpoint_candidate_ms < self.speech_start_ms
            || self.endpoint_candidate_ms > self.endpoint_final_ms
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_endpoint_latency.endpoint_candidate_ms",
                reason: "candidate timestamp must be within speech start and endpoint final",
            });
        }
        if self.endpoint_final_ms >= self.speech_end_ms
            && self.endpoint_latency_ms != self.endpoint_final_ms.saturating_sub(self.speech_end_ms)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_endpoint_latency.endpoint_latency_ms",
                reason: "latency must equal endpoint_final_ms - speech_end_ms",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8DSceneCalibrationMetricPacket {
    pub metric_id: String,
    pub audio_scene_id: String,
    pub noise_degradation: Stage8NoiseDegradationClass,
    pub echo_suspect_count: u16,
    pub self_echo_suspect_count: u16,
    pub background_speech_suspect_count: u16,
    pub overlapping_speaker_bucket: Stage8DSignalBucket,
    pub foreground_confidence_bucket: Stage8DConfidenceBucket,
    pub addressed_confidence_bucket: Stage8DConfidenceBucket,
    pub diarization_segment_mismatch_count: u16,
    pub reason_code: String,
}

impl Stage8DSceneCalibrationMetricPacket {
    pub fn from_scene(
        metric_id: impl Into<String>,
        scene: &Stage8AudioScenePacket,
        expected_speaker_segments: u16,
        observed_speaker_segments: u16,
    ) -> Result<Self, ContractViolation> {
        let diarization_segment_mismatch_count =
            expected_speaker_segments.abs_diff(observed_speaker_segments);
        let packet = Self {
            metric_id: metric_id.into(),
            audio_scene_id: scene.audio_scene_id.clone(),
            noise_degradation: scene.noise_degradation,
            echo_suspect_count: u16::from(scene.echo_suspect),
            self_echo_suspect_count: u16::from(scene.self_echo_suspect),
            background_speech_suspect_count: u16::from(scene.background_speech),
            overlapping_speaker_bucket: if scene.overlapping_speakers {
                Stage8DSignalBucket::High
            } else {
                Stage8DSignalBucket::Low
            },
            foreground_confidence_bucket: Stage8DConfidenceBucket::from_confidence_bp(
                scene
                    .foreground_speaker
                    .as_ref()
                    .map(|foreground| foreground.foreground_confidence_bp),
                scene.unknown_or_non_user_speaker,
            ),
            addressed_confidence_bucket: Stage8DConfidenceBucket::from_confidence_bp(
                scene
                    .addressed_to_selene
                    .as_ref()
                    .map(|addressed| addressed.confidence_bp),
                matches!(
                    scene.disposition,
                    Stage8AudioSceneDisposition::BlockedLowAddressingConfidence
                ),
            ),
            diarization_segment_mismatch_count,
            reason_code: "stage8d_scene_calibration_scored".to_string(),
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for Stage8DSceneCalibrationMetricPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8d_scene_calibration.metric_id", &self.metric_id)?;
        validate_stage4_ref(
            "stage8d_scene_calibration.audio_scene_id",
            &self.audio_scene_id,
        )?;
        validate_stage4_ref("stage8d_scene_calibration.reason_code", &self.reason_code)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage8DBenchmarkWorkAuthority {
    pub can_understand_intent: bool,
    pub can_answer: bool,
    pub can_search: bool,
    pub can_call_providers: bool,
    pub can_capture_microphone_audio: bool,
    pub can_transcribe_live_audio: bool,
    pub can_trigger_voice_id_matching: bool,
    pub can_authorize: bool,
    pub can_emit_tts: bool,
    pub can_route_tools: bool,
    pub can_connector_write: bool,
    pub can_execute_protected_mutation: bool,
    pub can_update_memory_persona_emotion: bool,
}

impl Stage8DBenchmarkWorkAuthority {
    pub const fn benchmark_evidence_only() -> Self {
        Self {
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_capture_microphone_audio: false,
            can_transcribe_live_audio: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_emit_tts: false,
            can_route_tools: false,
            can_connector_write: false,
            can_execute_protected_mutation: false,
            can_update_memory_persona_emotion: false,
        }
    }

    pub const fn can_route_or_mutate(self) -> bool {
        self.can_understand_intent
            || self.can_answer
            || self.can_search
            || self.can_call_providers
            || self.can_capture_microphone_audio
            || self.can_transcribe_live_audio
            || self.can_trigger_voice_id_matching
            || self.can_authorize
            || self.can_emit_tts
            || self.can_route_tools
            || self.can_connector_write
            || self.can_execute_protected_mutation
            || self.can_update_memory_persona_emotion
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8DListeningBenchmarkPacket {
    pub benchmark_target_id: String,
    pub benchmark_result_id: String,
    pub fixture_id: String,
    pub metric_id: String,
    pub replay_id: String,
    pub audit_id: String,
    pub reason_code: String,
    pub target_status: BenchmarkTargetStatus,
    pub comparison_outcome: BenchmarkComparisonOutcome,
    pub transcript_metric: Option<Stage8DTranscriptMetricPacket>,
    pub endpoint_metric: Option<Stage8DEndpointLatencyMetricPacket>,
    pub scene_metric: Option<Stage8DSceneCalibrationMetricPacket>,
    pub confidence_bucket: Stage8DConfidenceBucket,
    pub work_authority: Stage8DBenchmarkWorkAuthority,
}

impl Stage8DListeningBenchmarkPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn from_stage2_envelope(
        target: &BenchmarkTargetPacket,
        result: &BenchmarkResultPacket,
        fixture_id: impl Into<String>,
        metric_id: impl Into<String>,
        replay_id: impl Into<String>,
        audit_id: impl Into<String>,
        transcript_metric: Option<Stage8DTranscriptMetricPacket>,
        endpoint_metric: Option<Stage8DEndpointLatencyMetricPacket>,
        scene_metric: Option<Stage8DSceneCalibrationMetricPacket>,
        confidence_bucket: Stage8DConfidenceBucket,
    ) -> Result<Self, ContractViolation> {
        target.validate()?;
        result.validate()?;
        let packet = Self {
            benchmark_target_id: target.benchmark_target_id.clone(),
            benchmark_result_id: result.benchmark_result_id.clone(),
            fixture_id: fixture_id.into(),
            metric_id: metric_id.into(),
            replay_id: replay_id.into(),
            audit_id: audit_id.into(),
            reason_code: "stage8d_listening_benchmark_envelope".to_string(),
            target_status: result.target_status,
            comparison_outcome: result.comparison_outcome,
            transcript_metric,
            endpoint_metric,
            scene_metric,
            confidence_bucket,
            work_authority: Stage8DBenchmarkWorkAuthority::benchmark_evidence_only(),
        };
        if result.benchmark_target_id != target.benchmark_target_id
            || result.target_status != target.target_status
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_listening_benchmark_packet.stage2_envelope",
                reason: "benchmark result must match target id and target status",
            });
        }
        if target.target_status == BenchmarkTargetStatus::CertificationTargetPassed
            && !result.certifies_target(target)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_listening_benchmark_packet.stage2_envelope",
                reason: "certification packet requires a passing Stage 2 benchmark result",
            });
        }
        packet.validate()?;
        Ok(packet)
    }

    pub const fn can_route_or_mutate(&self) -> bool {
        self.work_authority.can_route_or_mutate()
    }
}

impl Validate for Stage8DListeningBenchmarkPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.benchmark_target_id",
            &self.benchmark_target_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.benchmark_result_id",
            &self.benchmark_result_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.fixture_id",
            &self.fixture_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.metric_id",
            &self.metric_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.replay_id",
            &self.replay_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.audit_id",
            &self.audit_id,
        )?;
        validate_stage4_ref(
            "stage8d_listening_benchmark_packet.reason_code",
            &self.reason_code,
        )?;
        if let Some(metric) = self.transcript_metric.as_ref() {
            metric.validate()?;
        }
        if let Some(metric) = self.endpoint_metric.as_ref() {
            metric.validate()?;
        }
        if let Some(metric) = self.scene_metric.as_ref() {
            metric.validate()?;
        }
        if self.work_authority.can_route_or_mutate() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_listening_benchmark_packet.work_authority",
                reason: "benchmark evidence cannot execute, route, speak, capture, call providers, identify, authorize, or mutate",
            });
        }
        if self.target_status == BenchmarkTargetStatus::CertificationTargetPassed
            && (self.transcript_metric.is_none()
                && self.endpoint_metric.is_none()
                && self.scene_metric.is_none())
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_listening_benchmark_packet.metrics",
                reason: "certification requires at least one deterministic metric packet",
            });
        }
        if self.target_status == BenchmarkTargetStatus::BlockedWithOwnerAndNextAction
            && self.comparison_outcome != BenchmarkComparisonOutcome::Blocked
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_listening_benchmark_packet.comparison_outcome",
                reason: "blocked benchmark status requires blocked comparison outcome",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8EAlternativeTranscriptSource {
    FixtureOffline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage8ERepairDisposition {
    NotAttempted,
    AcceptedFixtureNormalization,
    RejectedEmptyRepair,
    RejectedProtectedTokenInvented,
    RejectedProtectedTokenMismatch,
    RejectedDomainTokenMismatch,
    RejectedMeaningDrift,
    RejectedOverRepair,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8EAlternativeTranscriptCandidate {
    pub candidate_id: String,
    pub rank: u16,
    pub transcript_text: String,
    pub transcript_hash: String,
    pub confidence_bucket: Stage8DConfidenceBucket,
    pub protected_token_mismatch_count: u16,
    pub source: Stage8EAlternativeTranscriptSource,
    pub can_commit_directly: bool,
}

impl Stage8EAlternativeTranscriptCandidate {
    pub fn fixture_offline(
        candidate_id: impl Into<String>,
        rank: u16,
        transcript_text: impl Into<String>,
        confidence_bucket: Stage8DConfidenceBucket,
        protected_tokens: &[String],
    ) -> Result<Self, ContractViolation> {
        let transcript_text = transcript_text.into();
        let normalized_transcript = stage8d_normalize_transcript(&transcript_text);
        let packet = Self {
            candidate_id: candidate_id.into(),
            rank,
            transcript_hash: stage8_exact_transcript_hash(&transcript_text),
            protected_token_mismatch_count: stage8d_token_mismatch_count(
                protected_tokens,
                &normalized_transcript,
            )?,
            transcript_text,
            confidence_bucket,
            source: Stage8EAlternativeTranscriptSource::FixtureOffline,
            can_commit_directly: false,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for Stage8EAlternativeTranscriptCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8e_alternative_candidate.candidate_id",
            &self.candidate_id,
        )?;
        validate_stage8d_text(
            "stage8e_alternative_candidate.transcript_text",
            &self.transcript_text,
        )?;
        validate_stage4_ref(
            "stage8e_alternative_candidate.transcript_hash",
            &self.transcript_hash,
        )?;
        if self.rank == 0 || self.rank > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_alternative_candidate.rank",
                reason: "candidate rank must be within 1..=16",
            });
        }
        if self.can_commit_directly {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_alternative_candidate.can_commit_directly",
                reason: "alternative transcript candidates are benchmark evidence and cannot commit directly",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8EAlternativeTranscriptCandidateSetPacket {
    pub candidate_set_id: String,
    pub fixture_id: String,
    pub candidates: Vec<Stage8EAlternativeTranscriptCandidate>,
    pub selected_candidate_id: Option<String>,
    pub reason_code: String,
}

impl Stage8EAlternativeTranscriptCandidateSetPacket {
    pub fn v1(
        candidate_set_id: impl Into<String>,
        fixture_id: impl Into<String>,
        candidates: Vec<Stage8EAlternativeTranscriptCandidate>,
        selected_candidate_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            candidate_set_id: candidate_set_id.into(),
            fixture_id: fixture_id.into(),
            candidates,
            selected_candidate_id,
            reason_code: "stage8e_alternative_candidates_fixture_only".to_string(),
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn selected_candidate(&self) -> Option<&Stage8EAlternativeTranscriptCandidate> {
        let selected_id = self.selected_candidate_id.as_deref()?;
        self.candidates
            .iter()
            .find(|candidate| candidate.candidate_id == selected_id)
    }
}

impl Validate for Stage8EAlternativeTranscriptCandidateSetPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8e_candidate_set.candidate_set_id",
            &self.candidate_set_id,
        )?;
        validate_stage4_ref("stage8e_candidate_set.fixture_id", &self.fixture_id)?;
        validate_stage4_ref("stage8e_candidate_set.reason_code", &self.reason_code)?;
        validate_stage4_optional_ref(
            "stage8e_candidate_set.selected_candidate_id",
            self.selected_candidate_id.as_deref(),
        )?;
        if self.candidates.is_empty() || self.candidates.len() > 5 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_candidate_set.candidates",
                reason: "candidate set must contain 1..=5 deterministic fixture candidates",
            });
        }
        for (index, candidate) in self.candidates.iter().enumerate() {
            candidate.validate()?;
            if candidate.rank != (index as u16).saturating_add(1) {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8e_candidate_set.candidates",
                    reason: "candidate ranks must be contiguous and already sorted",
                });
            }
            if self.candidates[..index]
                .iter()
                .any(|previous| previous.candidate_id == candidate.candidate_id)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8e_candidate_set.candidates",
                    reason: "candidate ids must be unique",
                });
            }
        }
        if let Some(selected_id) = self.selected_candidate_id.as_deref() {
            let selected = self
                .candidates
                .iter()
                .any(|candidate| candidate.candidate_id == selected_id);
            if !selected {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8e_candidate_set.selected_candidate_id",
                    reason: "selected candidate must be present in the bounded candidate set",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8ERepairDecisionPacket {
    pub repair_decision_id: String,
    pub fixture_id: String,
    pub original_transcript: String,
    pub reference_transcript: String,
    pub repair_candidate: Option<String>,
    pub accepted_repair: bool,
    pub disposition: Stage8ERepairDisposition,
    pub protected_token_mismatch_count: u16,
    pub domain_token_mismatch_count: u16,
    pub protected_token_invented: bool,
    pub meaning_drift_detected: bool,
    pub over_repair_detected: bool,
    pub reason_code: String,
}

impl Stage8ERepairDecisionPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn fixture_only(
        repair_decision_id: impl Into<String>,
        fixture_id: impl Into<String>,
        original_transcript: impl Into<String>,
        reference_transcript: impl Into<String>,
        repair_candidate: Option<String>,
        protected_tokens: &[String],
        domain_tokens: &[String],
    ) -> Result<Self, ContractViolation> {
        let original_transcript = original_transcript.into();
        let reference_transcript = reference_transcript.into();
        let normalized_original = stage8d_normalize_transcript(&original_transcript);
        let normalized_reference = stage8d_normalize_transcript(&reference_transcript);
        let normalized_repair = repair_candidate
            .as_deref()
            .map(stage8d_normalize_transcript)
            .unwrap_or_default();
        let protected_token_invented = repair_candidate.as_ref().is_some()
            && stage8e_any_token_invented(
                protected_tokens,
                &normalized_original,
                &normalized_repair,
            )?;
        let protected_token_mismatch_count =
            stage8d_token_mismatch_count(protected_tokens, &normalized_repair)?;
        let domain_token_mismatch_count =
            stage8d_token_mismatch_count(domain_tokens, &normalized_repair)?;
        let over_repair_detected =
            stage8e_over_repair_detected(&normalized_reference, &normalized_repair);
        let meaning_drift_detected = repair_candidate.as_ref().is_some()
            && !normalized_repair.is_empty()
            && !stage8e_words_subset_of_reference(&normalized_reference, &normalized_repair);
        let disposition = if repair_candidate.is_none() {
            Stage8ERepairDisposition::NotAttempted
        } else if normalized_repair.is_empty() {
            Stage8ERepairDisposition::RejectedEmptyRepair
        } else if protected_token_invented {
            Stage8ERepairDisposition::RejectedProtectedTokenInvented
        } else if protected_token_mismatch_count > 0 {
            Stage8ERepairDisposition::RejectedProtectedTokenMismatch
        } else if domain_token_mismatch_count > 0 {
            Stage8ERepairDisposition::RejectedDomainTokenMismatch
        } else if over_repair_detected {
            Stage8ERepairDisposition::RejectedOverRepair
        } else if meaning_drift_detected {
            Stage8ERepairDisposition::RejectedMeaningDrift
        } else {
            Stage8ERepairDisposition::AcceptedFixtureNormalization
        };
        let packet = Self {
            repair_decision_id: repair_decision_id.into(),
            fixture_id: fixture_id.into(),
            original_transcript,
            reference_transcript,
            repair_candidate,
            accepted_repair: disposition == Stage8ERepairDisposition::AcceptedFixtureNormalization,
            disposition,
            protected_token_mismatch_count,
            domain_token_mismatch_count,
            protected_token_invented,
            meaning_drift_detected,
            over_repair_detected,
            reason_code: "stage8e_repair_fixture_decision".to_string(),
        };
        packet.validate()?;
        Ok(packet)
    }

    fn effective_transcript<'a>(&'a self, fallback: &'a str) -> &'a str {
        if self.accepted_repair {
            self.repair_candidate.as_deref().unwrap_or(fallback)
        } else {
            fallback
        }
    }
}

impl Validate for Stage8ERepairDecisionPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8e_repair_decision.repair_decision_id",
            &self.repair_decision_id,
        )?;
        validate_stage4_ref("stage8e_repair_decision.fixture_id", &self.fixture_id)?;
        validate_stage8d_text(
            "stage8e_repair_decision.original_transcript",
            &self.original_transcript,
        )?;
        validate_stage8d_text(
            "stage8e_repair_decision.reference_transcript",
            &self.reference_transcript,
        )?;
        if let Some(candidate) = self.repair_candidate.as_deref() {
            validate_stage8d_text("stage8e_repair_decision.repair_candidate", candidate)?;
        }
        validate_stage4_ref("stage8e_repair_decision.reason_code", &self.reason_code)?;
        if self.accepted_repair
            && self.disposition != Stage8ERepairDisposition::AcceptedFixtureNormalization
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_decision.accepted_repair",
                reason: "accepted repair requires accepted fixture normalization disposition",
            });
        }
        if self.accepted_repair
            && (self.protected_token_mismatch_count > 0
                || self.domain_token_mismatch_count > 0
                || self.protected_token_invented
                || self.meaning_drift_detected
                || self.over_repair_detected)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_decision.accepted_repair",
                reason:
                    "accepted repair cannot invent or lose protected/domain tokens or drift meaning",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8ERepairBenchmarkFixture {
    pub fixture_id: String,
    pub reference_transcript: String,
    pub observed_transcript: String,
    pub accent_marker: Option<String>,
    pub mixed_language_tokens: Vec<String>,
    pub domain_vocabulary_tokens: Vec<String>,
    pub protected_tokens: Vec<String>,
    pub vocabulary_pack_id: Option<String>,
    pub pronunciation_profile_id: Option<String>,
}

impl Stage8ERepairBenchmarkFixture {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        fixture_id: impl Into<String>,
        reference_transcript: impl Into<String>,
        observed_transcript: impl Into<String>,
        accent_marker: Option<String>,
        mixed_language_tokens: Vec<String>,
        domain_vocabulary_tokens: Vec<String>,
        protected_tokens: Vec<String>,
        vocabulary_pack_id: Option<String>,
        pronunciation_profile_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let fixture = Self {
            fixture_id: fixture_id.into(),
            reference_transcript: reference_transcript.into(),
            observed_transcript: observed_transcript.into(),
            accent_marker,
            mixed_language_tokens,
            domain_vocabulary_tokens,
            protected_tokens,
            vocabulary_pack_id,
            pronunciation_profile_id,
        };
        fixture.validate()?;
        Ok(fixture)
    }

    pub fn score(
        &self,
        metric_id: impl Into<String>,
        candidate_set: &Stage8EAlternativeTranscriptCandidateSetPacket,
        repair_decision: &Stage8ERepairDecisionPacket,
    ) -> Result<Stage8ERepairBenchmarkMetricPacket, ContractViolation> {
        self.validate()?;
        candidate_set.validate()?;
        repair_decision.validate()?;
        if candidate_set.fixture_id != self.fixture_id
            || repair_decision.fixture_id != self.fixture_id
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_fixture.fixture_id",
                reason: "candidate set and repair decision must belong to fixture",
            });
        }
        let effective_transcript = repair_decision.effective_transcript(&self.observed_transcript);
        let normalized_effective = stage8d_normalize_transcript(effective_transcript);
        let metric = Stage8ERepairBenchmarkMetricPacket {
            metric_id: metric_id.into(),
            fixture_id: self.fixture_id.clone(),
            accent_marker: self.accent_marker.clone(),
            accent_benchmark_only: self.accent_marker.is_some(),
            mixed_language_preserved: stage8d_all_tokens_present(
                &self.mixed_language_tokens,
                &normalized_effective,
            )?,
            domain_vocabulary_preserved: stage8d_all_tokens_present(
                &self.domain_vocabulary_tokens,
                &normalized_effective,
            )?,
            alternative_candidate_count: candidate_set.candidates.len() as u16,
            selected_candidate_id: candidate_set.selected_candidate_id.clone(),
            repair_decision_id: repair_decision.repair_decision_id.clone(),
            repair_disposition: repair_decision.disposition,
            protected_token_mismatch_count: stage8d_token_mismatch_count(
                &self.protected_tokens,
                &normalized_effective,
            )?,
            language_script_token_mismatch_count: stage8d_token_mismatch_count(
                &self.mixed_language_tokens,
                &normalized_effective,
            )?,
            domain_token_mismatch_count: stage8d_token_mismatch_count(
                &self.domain_vocabulary_tokens,
                &normalized_effective,
            )?,
            vocabulary_pack_id: self.vocabulary_pack_id.clone(),
            pronunciation_profile_id: self.pronunciation_profile_id.clone(),
            reason_code: "stage8e_repair_benchmark_scored".to_string(),
        };
        metric.validate()?;
        Ok(metric)
    }
}

impl Validate for Stage8ERepairBenchmarkFixture {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8e_repair_fixture.fixture_id", &self.fixture_id)?;
        validate_stage8d_text(
            "stage8e_repair_fixture.reference_transcript",
            &self.reference_transcript,
        )?;
        validate_stage8d_text(
            "stage8e_repair_fixture.observed_transcript",
            &self.observed_transcript,
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_fixture.accent_marker",
            self.accent_marker.as_deref(),
        )?;
        validate_stage8d_token_list(
            "stage8e_repair_fixture.mixed_language_tokens",
            &self.mixed_language_tokens,
        )?;
        validate_stage8d_token_list(
            "stage8e_repair_fixture.domain_vocabulary_tokens",
            &self.domain_vocabulary_tokens,
        )?;
        validate_stage8d_token_list(
            "stage8e_repair_fixture.protected_tokens",
            &self.protected_tokens,
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_fixture.vocabulary_pack_id",
            self.vocabulary_pack_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_fixture.pronunciation_profile_id",
            self.pronunciation_profile_id.as_deref(),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8ERepairBenchmarkMetricPacket {
    pub metric_id: String,
    pub fixture_id: String,
    pub accent_marker: Option<String>,
    pub accent_benchmark_only: bool,
    pub mixed_language_preserved: bool,
    pub domain_vocabulary_preserved: bool,
    pub alternative_candidate_count: u16,
    pub selected_candidate_id: Option<String>,
    pub repair_decision_id: String,
    pub repair_disposition: Stage8ERepairDisposition,
    pub protected_token_mismatch_count: u16,
    pub language_script_token_mismatch_count: u16,
    pub domain_token_mismatch_count: u16,
    pub vocabulary_pack_id: Option<String>,
    pub pronunciation_profile_id: Option<String>,
    pub reason_code: String,
}

impl Validate for Stage8ERepairBenchmarkMetricPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref("stage8e_repair_metric.metric_id", &self.metric_id)?;
        validate_stage4_ref("stage8e_repair_metric.fixture_id", &self.fixture_id)?;
        validate_stage4_optional_ref(
            "stage8e_repair_metric.accent_marker",
            self.accent_marker.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_metric.selected_candidate_id",
            self.selected_candidate_id.as_deref(),
        )?;
        validate_stage4_ref(
            "stage8e_repair_metric.repair_decision_id",
            &self.repair_decision_id,
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_metric.vocabulary_pack_id",
            self.vocabulary_pack_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_metric.pronunciation_profile_id",
            self.pronunciation_profile_id.as_deref(),
        )?;
        validate_stage4_ref("stage8e_repair_metric.reason_code", &self.reason_code)?;
        if self.alternative_candidate_count == 0 || self.alternative_candidate_count > 5 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_metric.alternative_candidate_count",
                reason: "alternative transcript candidate count must be within 1..=5",
            });
        }
        if self.repair_disposition == Stage8ERepairDisposition::AcceptedFixtureNormalization
            && (self.protected_token_mismatch_count > 0
                || self.language_script_token_mismatch_count > 0
                || self.domain_token_mismatch_count > 0)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_metric.repair_disposition",
                reason: "accepted repair must preserve protected, language, and domain tokens",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage8EBenchmarkWorkAuthority {
    pub can_understand_intent: bool,
    pub can_answer: bool,
    pub can_search: bool,
    pub can_call_providers: bool,
    pub can_capture_microphone_audio: bool,
    pub can_transcribe_live_audio: bool,
    pub can_trigger_voice_id_matching: bool,
    pub can_authorize: bool,
    pub can_emit_tts: bool,
    pub can_route_tools: bool,
    pub can_connector_write: bool,
    pub can_execute_protected_mutation: bool,
    pub can_update_memory_persona_emotion: bool,
    pub can_promote_provider_model_router: bool,
}

impl Stage8EBenchmarkWorkAuthority {
    pub const fn benchmark_evidence_only() -> Self {
        Self {
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_capture_microphone_audio: false,
            can_transcribe_live_audio: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_emit_tts: false,
            can_route_tools: false,
            can_connector_write: false,
            can_execute_protected_mutation: false,
            can_update_memory_persona_emotion: false,
            can_promote_provider_model_router: false,
        }
    }

    pub const fn can_route_or_mutate(self) -> bool {
        self.can_understand_intent
            || self.can_answer
            || self.can_search
            || self.can_call_providers
            || self.can_capture_microphone_audio
            || self.can_transcribe_live_audio
            || self.can_trigger_voice_id_matching
            || self.can_authorize
            || self.can_emit_tts
            || self.can_route_tools
            || self.can_connector_write
            || self.can_execute_protected_mutation
            || self.can_update_memory_persona_emotion
            || self.can_promote_provider_model_router
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8EListeningRepairBenchmarkPacket {
    pub benchmark_target_id: String,
    pub benchmark_result_id: String,
    pub fixture_id: String,
    pub metric_id: String,
    pub candidate_set_id: String,
    pub repair_decision_id: String,
    pub vocabulary_pack_id: Option<String>,
    pub pronunciation_profile_id: Option<String>,
    pub replay_id: String,
    pub audit_id: String,
    pub reason_code: String,
    pub target_status: BenchmarkTargetStatus,
    pub comparison_outcome: BenchmarkComparisonOutcome,
    pub repair_metric: Option<Stage8ERepairBenchmarkMetricPacket>,
    pub work_authority: Stage8EBenchmarkWorkAuthority,
}

impl Stage8EListeningRepairBenchmarkPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn from_stage2_envelope(
        target: &BenchmarkTargetPacket,
        result: &BenchmarkResultPacket,
        fixture_id: impl Into<String>,
        metric_id: impl Into<String>,
        candidate_set_id: impl Into<String>,
        repair_decision_id: impl Into<String>,
        vocabulary_pack_id: Option<String>,
        pronunciation_profile_id: Option<String>,
        replay_id: impl Into<String>,
        audit_id: impl Into<String>,
        repair_metric: Option<Stage8ERepairBenchmarkMetricPacket>,
    ) -> Result<Self, ContractViolation> {
        target.validate()?;
        result.validate()?;
        let packet = Self {
            benchmark_target_id: target.benchmark_target_id.clone(),
            benchmark_result_id: result.benchmark_result_id.clone(),
            fixture_id: fixture_id.into(),
            metric_id: metric_id.into(),
            candidate_set_id: candidate_set_id.into(),
            repair_decision_id: repair_decision_id.into(),
            vocabulary_pack_id,
            pronunciation_profile_id,
            replay_id: replay_id.into(),
            audit_id: audit_id.into(),
            reason_code: "stage8e_repair_benchmark_envelope".to_string(),
            target_status: result.target_status,
            comparison_outcome: result.comparison_outcome,
            repair_metric,
            work_authority: Stage8EBenchmarkWorkAuthority::benchmark_evidence_only(),
        };
        if result.benchmark_target_id != target.benchmark_target_id
            || result.target_status != target.target_status
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_benchmark_packet.stage2_envelope",
                reason: "benchmark result must match target id and target status",
            });
        }
        if target.target_status == BenchmarkTargetStatus::CertificationTargetPassed
            && !result.certifies_target(target)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_benchmark_packet.stage2_envelope",
                reason: "certification packet requires a passing Stage 2 benchmark result",
            });
        }
        packet.validate()?;
        Ok(packet)
    }

    pub const fn can_route_or_mutate(&self) -> bool {
        self.work_authority.can_route_or_mutate()
    }
}

impl Validate for Stage8EListeningRepairBenchmarkPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.benchmark_target_id",
            &self.benchmark_target_id,
        )?;
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.benchmark_result_id",
            &self.benchmark_result_id,
        )?;
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.fixture_id",
            &self.fixture_id,
        )?;
        validate_stage4_ref("stage8e_repair_benchmark_packet.metric_id", &self.metric_id)?;
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.candidate_set_id",
            &self.candidate_set_id,
        )?;
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.repair_decision_id",
            &self.repair_decision_id,
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_benchmark_packet.vocabulary_pack_id",
            self.vocabulary_pack_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8e_repair_benchmark_packet.pronunciation_profile_id",
            self.pronunciation_profile_id.as_deref(),
        )?;
        validate_stage4_ref("stage8e_repair_benchmark_packet.replay_id", &self.replay_id)?;
        validate_stage4_ref("stage8e_repair_benchmark_packet.audit_id", &self.audit_id)?;
        validate_stage4_ref(
            "stage8e_repair_benchmark_packet.reason_code",
            &self.reason_code,
        )?;
        if let Some(metric) = self.repair_metric.as_ref() {
            metric.validate()?;
            if metric.fixture_id != self.fixture_id
                || metric.metric_id != self.metric_id
                || metric.repair_decision_id != self.repair_decision_id
            {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8e_repair_benchmark_packet.repair_metric",
                    reason: "metric ids must match the benchmark envelope",
                });
            }
        }
        if self.work_authority.can_route_or_mutate() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_benchmark_packet.work_authority",
                reason: "repair benchmark evidence cannot execute, route, speak, capture, call providers, identify, authorize, promote routers, or mutate",
            });
        }
        if self.target_status == BenchmarkTargetStatus::CertificationTargetPassed
            && self.repair_metric.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_benchmark_packet.repair_metric",
                reason: "certification requires a deterministic Stage 8E repair metric packet",
            });
        }
        if self.target_status == BenchmarkTargetStatus::BlockedWithOwnerAndNextAction
            && self.comparison_outcome != BenchmarkComparisonOutcome::Blocked
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_repair_benchmark_packet.comparison_outcome",
                reason: "blocked benchmark status requires blocked comparison outcome",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage8VoiceWorkAuthority {
    pub can_update_listen_state: bool,
    pub can_update_preview: bool,
    pub can_emit_committed_turn: bool,
    pub can_enter_understanding: bool,
    pub can_understand_intent: bool,
    pub can_answer: bool,
    pub can_search: bool,
    pub can_call_providers: bool,
    pub can_trigger_voice_id_matching: bool,
    pub can_authorize: bool,
    pub can_route_tools: bool,
    pub can_emit_tts: bool,
    pub can_execute_protected_mutation: bool,
    pub can_connector_write: bool,
    pub can_update_memory_persona_emotion: bool,
}

impl Stage8VoiceWorkAuthority {
    pub const fn listen_state_only() -> Self {
        Self {
            can_update_listen_state: true,
            can_update_preview: false,
            can_emit_committed_turn: false,
            can_enter_understanding: false,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
            can_update_memory_persona_emotion: false,
        }
    }

    pub const fn preview_only() -> Self {
        Self {
            can_update_listen_state: true,
            can_update_preview: true,
            can_emit_committed_turn: false,
            can_enter_understanding: false,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
            can_update_memory_persona_emotion: false,
        }
    }

    pub const fn final_transcript_boundary() -> Self {
        Self {
            can_update_listen_state: true,
            can_update_preview: false,
            can_emit_committed_turn: true,
            can_enter_understanding: true,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
            can_update_memory_persona_emotion: false,
        }
    }

    pub const fn blocked_or_artifact_only() -> Self {
        Self {
            can_update_listen_state: false,
            can_update_preview: false,
            can_emit_committed_turn: false,
            can_enter_understanding: false,
            can_understand_intent: false,
            can_answer: false,
            can_search: false,
            can_call_providers: false,
            can_trigger_voice_id_matching: false,
            can_authorize: false,
            can_route_tools: false,
            can_emit_tts: false,
            can_execute_protected_mutation: false,
            can_connector_write: false,
            can_update_memory_persona_emotion: false,
        }
    }

    pub const fn can_route_or_mutate(self) -> bool {
        self.can_understand_intent
            || self.can_answer
            || self.can_search
            || self.can_call_providers
            || self.can_trigger_voice_id_matching
            || self.can_authorize
            || self.can_route_tools
            || self.can_emit_tts
            || self.can_execute_protected_mutation
            || self.can_connector_write
            || self.can_update_memory_persona_emotion
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8TranscriptGatePacket {
    pub activation_context: Stage7ActivationContextPacket,
    pub boundary_kind: Stage8TranscriptGateKind,
    pub reason_code: &'static str,
    pub audio_scene_id: String,
    pub audio_scene_packet: Option<Stage8AudioScenePacket>,
    pub vad_signal_id: Option<String>,
    pub endpoint_id: Option<String>,
    pub endpoint_state: Stage8EndpointState,
    pub transcript_id: Option<String>,
    pub transcript_text: Option<String>,
    pub exact_transcript_hash: Option<String>,
    pub partial_revision_id: Option<u32>,
    pub confidence_bp: Option<u16>,
    pub coverage_bp: Option<u16>,
    pub confidence_gate_id: Option<String>,
    pub confidence_gate: Stage8ConfidenceGateDisposition,
    pub protected_slot_disposition: Stage8ProtectedSlotDisposition,
    pub protected_slot_uncertainties: Vec<Stage8ProtectedSlotUncertainty>,
    pub language_tag: Option<String>,
    pub session_id: Option<SessionId>,
    pub turn_id: Option<TurnId>,
    pub consent_state_id: Option<String>,
    pub device_trust_id: Option<String>,
    pub provider_budget_id: Option<String>,
    pub access_context_id: Option<String>,
    pub audit_id: Option<String>,
    pub candidate_preview: Option<Stage4TurnBoundaryPacket>,
    pub committed_turn: Option<Stage4TurnBoundaryPacket>,
    pub stage5_turn_authority: Option<Stage5TurnAuthorityPacket>,
    pub tts_self_echo_active: bool,
    pub background_speech_detected: bool,
    pub foreground_user_speech: bool,
    pub addressed_to_selene: bool,
    pub record_mode_audio: bool,
    pub work_authority: Stage8VoiceWorkAuthority,
}

impl Stage8TranscriptGatePacket {
    pub fn audio_substrate_only(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self::base(
            activation_context,
            Stage8TranscriptGateKind::AudioSubstrateOnly,
            audio_scene_id.into(),
            Stage8VoiceWorkAuthority::listen_state_only(),
        );
        packet.validate()?;
        Ok(packet)
    }

    pub fn vad_endpoint_boundary(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
        vad_signal_id: impl Into<String>,
        endpoint_id: impl Into<String>,
        endpoint_state: Stage8EndpointState,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            vad_signal_id: Some(vad_signal_id.into()),
            endpoint_id: Some(endpoint_id.into()),
            endpoint_state,
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::VadEndpointBoundaryOnly,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::listen_state_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn partial_transcript_preview(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
        transcript_id: impl Into<String>,
        transcript_text: impl Into<String>,
        confidence_bp: u16,
        partial_revision_id: u32,
    ) -> Result<Self, ContractViolation> {
        let candidate_preview = Stage4TurnBoundaryPacket::candidate_preview(
            activation_context.activation.clone(),
            Some(CanonicalTurnModality::Voice),
        )?;
        let transcript_text = transcript_text.into();
        let packet = Self {
            endpoint_state: Stage8EndpointState::EndpointCandidate,
            transcript_id: Some(transcript_id.into()),
            transcript_text: Some(transcript_text.clone()),
            exact_transcript_hash: Some(stage8_exact_transcript_hash(&transcript_text)),
            partial_revision_id: Some(partial_revision_id),
            confidence_bp: Some(confidence_bp),
            candidate_preview: Some(candidate_preview),
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::PartialTranscriptPreviewOnly,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::preview_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn final_transcript_commit(
        activation_context: Stage7ActivationContextPacket,
        stage5_turn_authority: Stage5TurnAuthorityPacket,
        audio_scene_id: impl Into<String>,
        endpoint_id: impl Into<String>,
        confidence_gate_id: impl Into<String>,
        transcript_id: impl Into<String>,
        transcript_text: impl Into<String>,
        language_tag: impl Into<String>,
        confidence_bp: u16,
        coverage_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let turn_id = stage5_turn_authority
            .turn_id
            .ok_or(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.stage5_turn_authority.turn_id",
                reason: "final transcript commit requires current turn id",
            })?;
        let device_turn_sequence =
            stage5_turn_authority
                .device_turn_sequence
                .ok_or(ContractViolation::InvalidValue {
                    field:
                        "stage8_transcript_gate_packet.stage5_turn_authority.device_turn_sequence",
                    reason: "final transcript commit requires device turn sequence",
                })?;
        let committed_turn = Stage4TurnBoundaryPacket::committed_live_turn(
            activation_context.activation.clone(),
            turn_id,
            device_turn_sequence,
            CanonicalTurnModality::Voice,
        )?;
        let transcript_text = transcript_text.into();
        let packet = Self {
            endpoint_id: Some(endpoint_id.into()),
            endpoint_state: Stage8EndpointState::EndpointFinal,
            transcript_id: Some(transcript_id.into()),
            transcript_text: Some(transcript_text.clone()),
            exact_transcript_hash: Some(stage8_exact_transcript_hash(&transcript_text)),
            confidence_bp: Some(confidence_bp),
            coverage_bp: Some(coverage_bp),
            confidence_gate_id: Some(confidence_gate_id.into()),
            confidence_gate: Stage8ConfidenceGateDisposition::Passed,
            protected_slot_disposition: Stage8ProtectedSlotDisposition::NoProtectedSlots,
            language_tag: Some(language_tag.into()),
            session_id: Some(stage5_turn_authority.session_id),
            turn_id: Some(turn_id),
            committed_turn: Some(committed_turn),
            stage5_turn_authority: Some(stage5_turn_authority),
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::FinalTranscriptCommitBoundary,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::final_transcript_boundary(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn confidence_gate_reject(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
        endpoint_id: impl Into<String>,
        confidence_gate_id: impl Into<String>,
        confidence_gate: Stage8ConfidenceGateDisposition,
        transcript_id: Option<String>,
        transcript_text: Option<String>,
        confidence_bp: Option<u16>,
        coverage_bp: Option<u16>,
        protected_slot_disposition: Stage8ProtectedSlotDisposition,
        protected_slot_uncertainties: Vec<Stage8ProtectedSlotUncertainty>,
    ) -> Result<Self, ContractViolation> {
        let exact_transcript_hash = transcript_text
            .as_ref()
            .map(|text| stage8_exact_transcript_hash(text));
        let packet = Self {
            endpoint_id: Some(endpoint_id.into()),
            endpoint_state: Stage8EndpointState::EndpointFinal,
            transcript_id,
            transcript_text,
            exact_transcript_hash,
            confidence_bp,
            coverage_bp,
            confidence_gate_id: Some(confidence_gate_id.into()),
            confidence_gate,
            protected_slot_disposition,
            protected_slot_uncertainties,
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::ConfidenceGateRejected,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::blocked_or_artifact_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn blocked_background_or_self_echo(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
        tts_self_echo_active: bool,
        background_speech_detected: bool,
        foreground_user_speech: bool,
        addressed_to_selene: bool,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            tts_self_echo_active,
            background_speech_detected,
            foreground_user_speech,
            addressed_to_selene,
            confidence_gate: Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser,
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::BackgroundOrSelfEchoBlocked,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::blocked_or_artifact_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn audio_scene_boundary(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_packet: Stage8AudioScenePacket,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            audio_scene_id: audio_scene_packet.audio_scene_id.clone(),
            audio_scene_packet: Some(audio_scene_packet),
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::AudioSceneBoundaryOnly,
                String::new(),
                Stage8VoiceWorkAuthority::listen_state_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn listening_scene_blocked(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_packet: Stage8AudioScenePacket,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            audio_scene_id: audio_scene_packet.audio_scene_id.clone(),
            audio_scene_packet: Some(audio_scene_packet),
            confidence_gate: Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser,
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::ListeningSceneBlocked,
                String::new(),
                Stage8VoiceWorkAuthority::blocked_or_artifact_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn record_audio_artifact_only(
        activation_context: Stage7ActivationContextPacket,
        audio_scene_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            record_mode_audio: true,
            foreground_user_speech: false,
            addressed_to_selene: false,
            ..Self::base(
                activation_context,
                Stage8TranscriptGateKind::RecordAudioArtifactOnly,
                audio_scene_id.into(),
                Stage8VoiceWorkAuthority::blocked_or_artifact_only(),
            )
        };
        packet.validate()?;
        Ok(packet)
    }

    pub const fn can_route_or_mutate(&self) -> bool {
        self.work_authority.can_route_or_mutate()
    }

    pub const fn can_emit_committed_turn(&self) -> bool {
        self.work_authority.can_emit_committed_turn
    }

    pub const fn is_preview_only(&self) -> bool {
        matches!(
            self.boundary_kind,
            Stage8TranscriptGateKind::PartialTranscriptPreviewOnly
        ) && self.candidate_preview.is_some()
            && self.committed_turn.is_none()
            && !self.work_authority.can_emit_committed_turn
            && !self.work_authority.can_enter_understanding
    }

    fn base(
        activation_context: Stage7ActivationContextPacket,
        boundary_kind: Stage8TranscriptGateKind,
        audio_scene_id: String,
        work_authority: Stage8VoiceWorkAuthority,
    ) -> Self {
        Self {
            session_id: activation_context.session_id,
            turn_id: None,
            consent_state_id: activation_context.consent_state_id.clone(),
            device_trust_id: activation_context.device_trust_id.clone(),
            provider_budget_id: activation_context.provider_budget_id.clone(),
            access_context_id: activation_context.access_context_id.clone(),
            audit_id: activation_context.audit_id.clone(),
            reason_code: boundary_kind.default_reason_code(),
            boundary_kind,
            audio_scene_id,
            audio_scene_packet: None,
            vad_signal_id: None,
            endpoint_id: None,
            endpoint_state: Stage8EndpointState::NotEvaluated,
            transcript_id: None,
            transcript_text: None,
            exact_transcript_hash: None,
            partial_revision_id: None,
            confidence_bp: None,
            coverage_bp: None,
            confidence_gate_id: None,
            confidence_gate: Stage8ConfidenceGateDisposition::NotEvaluated,
            protected_slot_disposition: Stage8ProtectedSlotDisposition::NotApplicable,
            protected_slot_uncertainties: Vec::new(),
            language_tag: None,
            candidate_preview: None,
            committed_turn: None,
            stage5_turn_authority: None,
            tts_self_echo_active: false,
            background_speech_detected: false,
            foreground_user_speech: true,
            addressed_to_selene: true,
            record_mode_audio: false,
            work_authority,
            activation_context,
        }
    }
}

impl Validate for Stage8TranscriptGatePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.activation_context.validate()?;
        validate_stage4_ref(
            "stage8_transcript_gate_packet.audio_scene_id",
            &self.audio_scene_id,
        )?;
        if let Some(scene) = self.audio_scene_packet.as_ref() {
            scene.validate()?;
            if scene.audio_scene_id != self.audio_scene_id {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.audio_scene_packet",
                    reason: "audio scene packet id must match transcript gate audio scene id",
                });
            }
        }
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.vad_signal_id",
            self.vad_signal_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.endpoint_id",
            self.endpoint_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.transcript_id",
            self.transcript_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.exact_transcript_hash",
            self.exact_transcript_hash.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.consent_state_id",
            self.consent_state_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.device_trust_id",
            self.device_trust_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.provider_budget_id",
            self.provider_budget_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.access_context_id",
            self.access_context_id.as_deref(),
        )?;
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.audit_id",
            self.audit_id.as_deref(),
        )?;
        if self.reason_code != self.boundary_kind.default_reason_code() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.reason_code",
                reason: "must match boundary kind",
            });
        }
        if self.work_authority.can_route_or_mutate() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.work_authority",
                reason: "voice/listen/transcript boundary cannot route tools, search, providers, TTS, identity, authority, memory, connector writes, or protected execution",
            });
        }
        if matches!(self.confidence_bp, Some(confidence_bp) if confidence_bp > 10_000) {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.confidence_bp",
                reason: "must be <= 10000",
            });
        }
        if matches!(self.coverage_bp, Some(coverage_bp) if coverage_bp > 10_000) {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.coverage_bp",
                reason: "must be <= 10000",
            });
        }
        validate_stage4_optional_ref(
            "stage8_transcript_gate_packet.confidence_gate_id",
            self.confidence_gate_id.as_deref(),
        )?;
        if self.protected_slot_uncertainties.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.protected_slot_uncertainties",
                reason: "must contain <= 8 entries",
            });
        }
        for uncertainty in &self.protected_slot_uncertainties {
            uncertainty.validate()?;
        }
        match self.protected_slot_disposition {
            Stage8ProtectedSlotDisposition::ClarificationRequired
            | Stage8ProtectedSlotDisposition::FailClosed => {
                if self.protected_slot_uncertainties.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.protected_slot_uncertainties",
                        reason: "protected-slot clarify/fail-closed requires uncertainty evidence",
                    });
                }
            }
            Stage8ProtectedSlotDisposition::NoProtectedSlots
            | Stage8ProtectedSlotDisposition::HighConfidenceProtectedSlots
            | Stage8ProtectedSlotDisposition::NotApplicable => {
                if !self.protected_slot_uncertainties.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.protected_slot_uncertainties",
                        reason: "uncertain protected slots cannot be carried as guessed slots",
                    });
                }
            }
            Stage8ProtectedSlotDisposition::DeferredToStage10Or12 => {}
        }
        if matches!(self.partial_revision_id, Some(0)) {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.partial_revision_id",
                reason: "must be > 0 when present",
            });
        }
        if let Some(language_tag) = self.language_tag.as_ref() {
            if language_tag.trim().is_empty() || language_tag.len() > 32 || !language_tag.is_ascii()
            {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.language_tag",
                    reason: "must be bounded non-empty ASCII",
                });
            }
        }
        if let Some(text) = self.transcript_text.as_ref() {
            if self.transcript_id.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.transcript_id",
                    reason: "transcript text requires transcript id",
                });
            }
            if text.trim().is_empty() || text.len() > MAX_TEXT_BYTES {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.transcript_text",
                    reason: "must be bounded non-empty text",
                });
            }
            let Some(hash) = self.exact_transcript_hash.as_ref() else {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.exact_transcript_hash",
                    reason: "transcript text requires exact transcript hash",
                });
            };
            if hash != &stage8_exact_transcript_hash(text) {
                return Err(ContractViolation::InvalidValue {
                    field: "stage8_transcript_gate_packet.exact_transcript_hash",
                    reason: "must match exact transcript text",
                });
            }
        }
        if !matches!(
            self.boundary_kind,
            Stage8TranscriptGateKind::BackgroundOrSelfEchoBlocked
                | Stage8TranscriptGateKind::ListeningSceneBlocked
                | Stage8TranscriptGateKind::RecordAudioArtifactOnly
        ) && (self.tts_self_echo_active
            || self.background_speech_detected
            || !self.foreground_user_speech
            || !self.addressed_to_selene)
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.audio_posture",
                reason:
                    "background, non-user, or TTS self-echo posture must use the blocked boundary",
            });
        }
        if !matches!(
            self.boundary_kind,
            Stage8TranscriptGateKind::RecordAudioArtifactOnly
        ) && self.record_mode_audio
        {
            return Err(ContractViolation::InvalidValue {
                field: "stage8_transcript_gate_packet.record_mode_audio",
                reason: "record-mode audio must use the artifact-only boundary",
            });
        }

        match self.boundary_kind {
            Stage8TranscriptGateKind::AudioSubstrateOnly => {
                if !self.work_authority.can_update_listen_state
                    || self.audio_scene_packet.is_some()
                    || self.vad_signal_id.is_some()
                    || self.endpoint_id.is_some()
                    || self.endpoint_state != Stage8EndpointState::NotEvaluated
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::NotEvaluated
                    || self.confidence_gate_id.is_some()
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                    || self.record_mode_audio
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "audio substrate may update listen state only",
                    });
                }
                validate_stage8_voice_activation(&self.activation_context)?;
            }
            Stage8TranscriptGateKind::VadEndpointBoundaryOnly => {
                validate_stage8_voice_activation(&self.activation_context)?;
                if !self.endpoint_state.is_endpoint_signal()
                    || self.audio_scene_packet.is_some()
                    || self.vad_signal_id.is_none()
                    || self.endpoint_id.is_none()
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || !self.work_authority.can_update_listen_state
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::NotEvaluated
                    || self.confidence_gate_id.is_some()
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "VAD and endpoint signals are boundary-only and cannot commit or route work",
                    });
                }
            }
            Stage8TranscriptGateKind::PartialTranscriptPreviewOnly => {
                validate_stage8_voice_activation(&self.activation_context)?;
                let Some(candidate_preview) = self.candidate_preview.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.candidate_preview",
                        reason: "partial transcript preview requires candidate preview boundary",
                    });
                };
                candidate_preview.validate()?;
                if self.audio_scene_packet.is_some()
                    || candidate_preview.is_committed_live_turn()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.transcript_id.is_none()
                    || self.transcript_text.is_none()
                    || self.partial_revision_id.is_none()
                    || self.endpoint_state == Stage8EndpointState::EndpointFinal
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::NotEvaluated
                    || self.confidence_gate_id.is_some()
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                    || !self.work_authority.can_update_preview
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "partial transcripts are preview-only and cannot commit or enter understanding",
                    });
                }
            }
            Stage8TranscriptGateKind::FinalTranscriptCommitBoundary => {
                validate_stage8_voice_activation(&self.activation_context)?;
                let Some(authority) = self.stage5_turn_authority.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.stage5_turn_authority",
                        reason: "final transcript commit requires Stage 5 current-turn authority",
                    });
                };
                authority.validate()?;
                let Some(committed_turn) = self.committed_turn.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.committed_turn",
                        reason: "final transcript commit requires committed live turn boundary",
                    });
                };
                committed_turn.validate()?;
                if authority.disposition != Stage5TurnAuthorityDisposition::CurrentCommittedTurn
                    || !authority.can_enter_understanding()
                    || authority.can_route_any_work()
                    || self.endpoint_state != Stage8EndpointState::EndpointFinal
                    || self.endpoint_id.is_none()
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::Passed
                    || self.confidence_gate_id.is_none()
                    || !matches!(self.confidence_bp, Some(confidence_bp) if confidence_bp >= 8_500)
                    || !matches!(self.coverage_bp, Some(coverage_bp) if coverage_bp >= 7_000)
                    || self.protected_slot_disposition.blocks_commit()
                    || !self.protected_slot_uncertainties.is_empty()
                    || self.session_id != Some(authority.session_id)
                    || self.turn_id != authority.turn_id
                    || self.activation_context.session_id != Some(authority.session_id)
                    || self.activation_context.activation.session_hint != Some(authority.session_id)
                    || !committed_turn.is_committed_live_turn()
                    || committed_turn.turn_id != authority.turn_id
                    || committed_turn.device_turn_sequence != authority.device_turn_sequence
                    || committed_turn.modality != Some(CanonicalTurnModality::Voice)
                    || self.candidate_preview.is_some()
                    || self.transcript_id.is_none()
                    || self.transcript_text.is_none()
                    || self.language_tag.is_none()
                    || self.confidence_bp.is_none()
                    || !self.work_authority.can_emit_committed_turn
                    || !self.work_authority.can_enter_understanding
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "final transcript commit requires endpoint-final, confidence pass, current turn authority, and no protected-slot uncertainty",
                    });
                }
                if let Some(scene) = self.audio_scene_packet.as_ref() {
                    if !scene.clean_foreground_addressed() {
                        return Err(ContractViolation::InvalidValue {
                            field: "stage8_transcript_gate_packet.audio_scene_packet",
                            reason: "scene evidence attached to final commit must be clean foreground user speech addressed to Selene and cannot replace Stage 5 authority or confidence gates",
                        });
                    }
                }
            }
            Stage8TranscriptGateKind::ConfidenceGateRejected => {
                validate_stage8_voice_activation(&self.activation_context)?;
                if !self.confidence_gate.is_rejection()
                    || self.audio_scene_packet.is_some()
                    || self.confidence_gate_id.is_none()
                    || !self.endpoint_state.is_final()
                    || self.endpoint_id.is_none()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.work_authority.can_update_listen_state
                    || self.work_authority.can_update_preview
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.record_mode_audio
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "confidence gate rejection cannot commit, preview, or route work",
                    });
                }
                if self.confidence_gate == Stage8ConfidenceGateDisposition::RejectedEmptyTranscript
                    && self.transcript_text.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.transcript_text",
                        reason: "empty transcript rejection cannot carry guessed transcript text",
                    });
                }
                if self.protected_slot_disposition.blocks_commit()
                    && !matches!(
                        self.confidence_gate,
                        Stage8ConfidenceGateDisposition::RejectedLowConfidence
                            | Stage8ConfidenceGateDisposition::RejectedLowCoverage
                            | Stage8ConfidenceGateDisposition::RejectedGarbledTranscript
                    )
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.protected_slot_disposition",
                        reason: "protected-slot uncertainty must clarify or fail closed from a transcript confidence rejection",
                    });
                }
            }
            Stage8TranscriptGateKind::AudioSceneBoundaryOnly => {
                validate_stage8_voice_activation(&self.activation_context)?;
                let Some(scene) = self.audio_scene_packet.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.audio_scene_packet",
                        reason: "audio scene boundary requires scene evidence",
                    });
                };
                if scene.has_blocking_signal()
                    || self.vad_signal_id.is_some()
                    || self.endpoint_id.is_some()
                    || self.endpoint_state != Stage8EndpointState::NotEvaluated
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || !self.work_authority.can_update_listen_state
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::NotEvaluated
                    || self.confidence_gate_id.is_some()
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "audio-scene signals are advisory listen-state evidence only",
                    });
                }
            }
            Stage8TranscriptGateKind::ListeningSceneBlocked => {
                let Some(scene) = self.audio_scene_packet.as_ref() else {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.audio_scene_packet",
                        reason: "blocked listening scene requires scene evidence",
                    });
                };
                if !scene.has_blocking_signal()
                    || self.vad_signal_id.is_some()
                    || self.endpoint_id.is_some()
                    || self.endpoint_state != Stage8EndpointState::NotEvaluated
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.work_authority.can_update_listen_state
                    || self.work_authority.can_update_preview
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.confidence_gate
                        != Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser
                    || self.confidence_gate_id.is_some()
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "blocked listening-scene evidence cannot commit, preview, route, or enter understanding",
                    });
                }
                if scene.record_mode_audio {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet.audio_scene_packet",
                        reason: "record-mode scene evidence must use the artifact-only boundary",
                    });
                }
                validate_stage8_voice_activation(&self.activation_context)?;
            }
            Stage8TranscriptGateKind::BackgroundOrSelfEchoBlocked => {
                if !(self.tts_self_echo_active
                    || self.background_speech_detected
                    || !self.foreground_user_speech
                    || !self.addressed_to_selene)
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.work_authority.can_update_listen_state
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.confidence_gate
                        != Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason: "background, non-user, or TTS self-echo audio must be blocked before turns",
                    });
                }
            }
            Stage8TranscriptGateKind::RecordAudioArtifactOnly => {
                if self.activation_context.disposition
                    != Stage7ActivationDisposition::RecordArtifactDeferred
                    || !self.record_mode_audio
                    || self.foreground_user_speech
                    || self.addressed_to_selene
                    || self.transcript_id.is_some()
                    || self.transcript_text.is_some()
                    || self.candidate_preview.is_some()
                    || self.committed_turn.is_some()
                    || self.stage5_turn_authority.is_some()
                    || self.work_authority.can_update_listen_state
                    || self.work_authority.can_emit_committed_turn
                    || self.work_authority.can_enter_understanding
                    || self.endpoint_state != Stage8EndpointState::NotEvaluated
                    || self.confidence_gate != Stage8ConfidenceGateDisposition::NotEvaluated
                    || self.protected_slot_disposition
                        != Stage8ProtectedSlotDisposition::NotApplicable
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "stage8_transcript_gate_packet",
                        reason:
                            "record-mode audio remains artifact-only and cannot become live chat",
                    });
                }
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

fn stage8_exact_transcript_hash(text: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("stage8fnv64-{hash:016x}")
}

fn stage8d_normalize_transcript(text: &str) -> String {
    let mut normalized = String::new();
    let mut previous_space = false;
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
            previous_space = false;
        } else if ch.is_ascii_whitespace() || ch.is_ascii_punctuation() {
            if !previous_space && !normalized.is_empty() {
                normalized.push(' ');
                previous_space = true;
            }
        }
    }
    normalized.trim().to_string()
}

fn stage8d_words(text: &str) -> Vec<String> {
    text.split_whitespace().map(ToString::to_string).collect()
}

fn stage8d_edit_counts<T: Eq>(
    reference: &[T],
    observed: &[T],
) -> Result<Stage8DEditCounts, ContractViolation> {
    if reference.len() > 512 || observed.len() > 512 {
        return Err(ContractViolation::InvalidValue {
            field: "stage8d_edit_counts.sequence_len",
            reason: "must be <= 512",
        });
    }
    let reference_len = reference.len() as u16;
    let observed_len = observed.len() as u16;
    let columns = observed.len() + 1;
    let empty = Stage8DEditCounts::candidate(0, 0, 0, reference_len, observed_len);
    let mut matrix = vec![empty; (reference.len() + 1) * columns];
    for i in 1..=reference.len() {
        let previous = matrix[(i - 1) * columns];
        matrix[i * columns] = Stage8DEditCounts::candidate(
            previous.substitutions,
            previous.insertions,
            previous.deletions.saturating_add(1),
            reference_len,
            observed_len,
        );
    }
    for j in 1..=observed.len() {
        let previous = matrix[j - 1];
        matrix[j] = Stage8DEditCounts::candidate(
            previous.substitutions,
            previous.insertions.saturating_add(1),
            previous.deletions,
            reference_len,
            observed_len,
        );
    }
    for i in 1..=reference.len() {
        for j in 1..=observed.len() {
            let diagonal = matrix[(i - 1) * columns + (j - 1)];
            let deletion = matrix[(i - 1) * columns + j];
            let insertion = matrix[i * columns + (j - 1)];
            matrix[i * columns + j] = if reference[i - 1] == observed[j - 1] {
                diagonal
            } else {
                stage8d_best_edit_count([
                    Stage8DEditCounts::candidate(
                        diagonal.substitutions.saturating_add(1),
                        diagonal.insertions,
                        diagonal.deletions,
                        reference_len,
                        observed_len,
                    ),
                    Stage8DEditCounts::candidate(
                        insertion.substitutions,
                        insertion.insertions.saturating_add(1),
                        insertion.deletions,
                        reference_len,
                        observed_len,
                    ),
                    Stage8DEditCounts::candidate(
                        deletion.substitutions,
                        deletion.insertions,
                        deletion.deletions.saturating_add(1),
                        reference_len,
                        observed_len,
                    ),
                ])
            };
        }
    }
    Ok(matrix[reference.len() * columns + observed.len()])
}

fn stage8d_best_edit_count(candidates: [Stage8DEditCounts; 3]) -> Stage8DEditCounts {
    let mut best = candidates[0];
    for candidate in candidates.iter().skip(1) {
        if stage8d_edit_count_sort_key(*candidate) < stage8d_edit_count_sort_key(best) {
            best = *candidate;
        }
    }
    best
}

fn stage8d_edit_count_sort_key(counts: Stage8DEditCounts) -> (u16, u16, u16, u16) {
    (
        counts.total_errors(),
        counts.substitutions,
        counts.insertions,
        counts.deletions,
    )
}

fn stage8d_token_mismatch_count(
    protected_tokens: &[String],
    normalized_observed: &str,
) -> Result<u16, ContractViolation> {
    let observed_words = stage8d_words(normalized_observed);
    let mut mismatches = 0_u16;
    for token in protected_tokens {
        let normalized = stage8d_normalize_transcript(token);
        if normalized.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_token_mismatch_count.protected_token",
                reason: "protected tokens must normalize to non-empty text",
            });
        }
        if !observed_words
            .iter()
            .any(|observed| observed == &normalized)
        {
            mismatches = mismatches.saturating_add(1);
        }
    }
    Ok(mismatches)
}

fn stage8d_all_tokens_present(
    tokens: &[String],
    normalized_observed: &str,
) -> Result<bool, ContractViolation> {
    let observed_words = stage8d_words(normalized_observed);
    for token in tokens {
        let normalized = stage8d_normalize_transcript(token);
        if normalized.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8d_all_tokens_present.token",
                reason: "tokens must normalize to non-empty text",
            });
        }
        if !observed_words
            .iter()
            .any(|observed| observed == &normalized)
        {
            return Ok(false);
        }
    }
    Ok(true)
}

fn stage8d_is_garbled(text: &str) -> bool {
    let trimmed = text.trim();
    !trimmed.is_empty() && !trimmed.chars().any(|ch| ch.is_ascii_alphanumeric())
}

fn validate_stage8d_text(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    let trimmed = value.trim();
    if trimmed.len() > 2_048 || !trimmed.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be bounded ASCII fixture text",
        });
    }
    Ok(())
}

fn validate_stage8d_token_list(
    field: &'static str,
    values: &[String],
) -> Result<(), ContractViolation> {
    if values.len() > 32 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain <= 32 tokens",
        });
    }
    for value in values {
        validate_stage8d_text(field, value)?;
        if stage8d_normalize_transcript(value).is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "tokens must normalize to non-empty text",
            });
        }
    }
    Ok(())
}

fn stage8e_any_token_invented(
    tokens: &[String],
    normalized_original: &str,
    normalized_repair: &str,
) -> Result<bool, ContractViolation> {
    let original_words = stage8d_words(normalized_original);
    let repair_words = stage8d_words(normalized_repair);
    for token in tokens {
        let normalized = stage8d_normalize_transcript(token);
        if normalized.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "stage8e_any_token_invented.token",
                reason: "tokens must normalize to non-empty text",
            });
        }
        let was_absent = !original_words
            .iter()
            .any(|original| original == &normalized);
        let now_present = repair_words.iter().any(|repair| repair == &normalized);
        if was_absent && now_present {
            return Ok(true);
        }
    }
    Ok(false)
}

fn stage8e_words_subset_of_reference(normalized_reference: &str, normalized_repair: &str) -> bool {
    let reference_words = stage8d_words(normalized_reference);
    stage8d_words(normalized_repair)
        .iter()
        .all(|repair| reference_words.iter().any(|reference| reference == repair))
}

fn stage8e_over_repair_detected(normalized_reference: &str, normalized_repair: &str) -> bool {
    let reference_words = stage8d_words(normalized_reference);
    let repair_words = stage8d_words(normalized_repair);
    repair_words.len() > reference_words.len().saturating_add(2)
        || normalized_repair.len() > normalized_reference.len().saturating_add(64)
}

fn validate_stage8_voice_activation(
    activation_context: &Stage7ActivationContextPacket,
) -> Result<(), ContractViolation> {
    if activation_context.disposition == Stage7ActivationDisposition::RecordArtifactDeferred {
        return Err(ContractViolation::InvalidValue {
            field: "stage8_transcript_gate_packet.activation_context",
            reason: "record activation cannot start live voice/listen state",
        });
    }
    if activation_context.can_perform_downstream_work() {
        return Err(ContractViolation::InvalidValue {
            field: "stage8_transcript_gate_packet.activation_context",
            reason: "Stage 7 activation context must remain non-authoritative",
        });
    }
    Ok(())
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
        stage4_activation_for_platform(AppPlatform::Desktop, source, trigger)
    }

    fn stage4_activation_for_platform(
        platform: AppPlatform,
        source: Stage4ActivationSource,
        trigger: RuntimeEntryTrigger,
    ) -> Stage4ActivationPacket {
        let mut packet = Stage4ActivationPacket::new(source, platform_context(platform, trigger))
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

    #[test]
    fn stage_7a_wake_candidate_is_attention_only_and_non_authoritative() {
        let packet = Stage7ActivationContextPacket::from_activation(
            stage4_activation(
                Stage4ActivationSource::WakeCandidate,
                RuntimeEntryTrigger::WakeWord,
            ),
            "activation-stage7-wake",
        )
        .expect("stage 7 wake packet")
        .with_wake_refs("wake-event-stage7", "wake-artifact-stage7")
        .expect("wake refs are consent-bound");

        assert_eq!(
            packet.disposition,
            Stage7ActivationDisposition::WakeAttentionOnly
        );
        assert!(packet.can_only_open_or_resume_session());
        assert!(!packet.can_perform_downstream_work());
        assert_eq!(packet.consent_state_id.as_deref(), Some("consent-stage4"));
        assert_eq!(
            packet.provider_budget_id.as_deref(),
            Some("provider-budget-stage4")
        );
    }

    #[test]
    fn stage_7a_side_button_is_explicit_iphone_activation_only() {
        let packet = Stage7ActivationContextPacket::from_activation(
            stage4_activation_for_platform(
                AppPlatform::Ios,
                Stage4ActivationSource::SideButton,
                RuntimeEntryTrigger::Explicit,
            ),
            "activation-stage7-side-button",
        )
        .expect("stage 7 side-button packet");

        assert_eq!(
            packet.disposition,
            Stage7ActivationDisposition::SideButtonExplicitOnly
        );
        assert!(packet.can_only_open_or_resume_session());
        assert!(!packet.can_perform_downstream_work());
        assert!(!packet.iphone_always_listening_attempt);
        assert!(packet.wake_event_id.is_none());
        assert!(packet.wake_artifact_id.is_none());

        let desktop_side_button = Stage7ActivationContextPacket::from_activation(
            stage4_activation(
                Stage4ActivationSource::SideButton,
                RuntimeEntryTrigger::Explicit,
            ),
            "activation-stage7-desktop-side-button",
        );
        assert!(desktop_side_button.is_err());
    }

    #[test]
    fn stage_7a_iphone_wake_candidate_is_blocked_before_activation_packet() {
        let ios_wake = Stage4ActivationPacket::new(
            Stage4ActivationSource::WakeCandidate,
            platform_context(AppPlatform::Ios, RuntimeEntryTrigger::WakeWord),
        );

        assert!(ios_wake.is_err());
    }

    #[test]
    fn stage_7a_access_context_ref_remains_non_authoritative() {
        let packet = Stage7ActivationContextPacket::from_activation(
            stage4_activation(
                Stage4ActivationSource::ExplicitMic,
                RuntimeEntryTrigger::Explicit,
            ),
            "activation-stage7-explicit",
        )
        .expect("stage 7 explicit packet")
        .with_access_context_id("access-context-stage6")
        .expect("access context ref");

        assert_eq!(
            packet.disposition,
            Stage7ActivationDisposition::ExplicitActivationOnly
        );
        assert_eq!(
            packet.access_context_id.as_deref(),
            Some("access-context-stage6")
        );
        assert!(packet.can_only_open_or_resume_session());
        assert!(!packet.can_perform_downstream_work());
    }

    #[test]
    fn stage_7a_wake_artifact_requires_consent_reference() {
        let mut activation = stage4_activation(
            Stage4ActivationSource::WakeCandidate,
            RuntimeEntryTrigger::WakeWord,
        );
        activation.consent_state_id = None;
        activation
            .validate()
            .expect("activation without consent is valid");

        let packet = Stage7ActivationContextPacket::from_activation(
            activation,
            "activation-stage7-no-consent",
        )
        .expect("wake packet without artifact");

        assert!(packet
            .with_wake_refs("wake-event-stage7", "wake-artifact-stage7")
            .is_err());
    }

    fn stage8_explicit_mic_activation(
        session_id: Option<SessionId>,
    ) -> Stage7ActivationContextPacket {
        let mut activation = stage4_activation(
            Stage4ActivationSource::ExplicitMic,
            RuntimeEntryTrigger::Explicit,
        );
        activation.session_hint = session_id;
        activation.validate().expect("stage 8 activation");
        Stage7ActivationContextPacket::from_activation(activation, "activation-stage8-explicit")
            .expect("stage 8 activation context")
            .with_access_context_id("access-context-stage8")
            .expect("access context")
    }

    fn stage8_record_activation() -> Stage7ActivationContextPacket {
        Stage7ActivationContextPacket::from_activation(
            stage4_activation(
                Stage4ActivationSource::RecordButton,
                RuntimeEntryTrigger::Explicit,
            ),
            "activation-stage8-record",
        )
        .expect("stage 8 record activation context")
    }

    fn stage8_current_authority() -> Stage5TurnAuthorityPacket {
        Stage5TurnAuthorityPacket::current_committed(
            SessionId(88),
            TurnId(8),
            "device-stage8",
            4,
            SessionState::Active,
        )
        .expect("stage 8 current turn authority")
    }

    fn stage8c_foreground(
        confidence_bp: u16,
        is_user_speech_candidate: bool,
    ) -> Stage8ForegroundSpeakerPacket {
        Stage8ForegroundSpeakerPacket::advisory(
            "speaker-segment-stage8c",
            Some("foreground-speaker-stage8c".to_string()),
            confidence_bp,
            is_user_speech_candidate,
        )
        .expect("foreground speaker packet")
    }

    fn stage8c_addressed(confidence_bp: u16, addressed: bool) -> Stage8AddressedToSelenePacket {
        Stage8AddressedToSelenePacket::advisory(
            "addressed-to-selene-stage8c",
            confidence_bp,
            addressed,
        )
        .expect("addressed-to-selene packet")
    }

    fn stage8c_clean_scene(audio_scene_id: &str) -> Stage8AudioScenePacket {
        Stage8AudioScenePacket::v1(
            audio_scene_id,
            Some(stage8c_foreground(9_000, true)),
            Some(stage8c_addressed(9_100, true)),
            Stage8AudioSceneDisposition::CleanForegroundAddressed,
            Stage8NoiseDegradationClass::Clear,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            "stage8c-clean",
        )
        .expect("clean stage 8c scene")
    }

    fn stage8c_blocked_scene(
        audio_scene_id: &str,
        disposition: Stage8AudioSceneDisposition,
    ) -> Stage8AudioScenePacket {
        let (
            foreground,
            addressed,
            noise,
            echo,
            self_echo,
            background,
            overlapping,
            non_user,
            record_mode,
        ) = match disposition {
            Stage8AudioSceneDisposition::BlockedLowAddressingConfidence => (
                Some(stage8c_foreground(9_000, true)),
                Some(stage8c_addressed(4_500, false)),
                Stage8NoiseDegradationClass::Clear,
                false,
                false,
                false,
                false,
                false,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedBackgroundSpeech => (
                Some(stage8c_foreground(7_000, false)),
                Some(stage8c_addressed(6_000, false)),
                Stage8NoiseDegradationClass::Moderate,
                false,
                false,
                true,
                false,
                false,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedSelfEcho => (
                Some(stage8c_foreground(7_000, false)),
                Some(stage8c_addressed(7_000, false)),
                Stage8NoiseDegradationClass::Clear,
                true,
                true,
                false,
                false,
                false,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedOverlappingSpeakers => (
                Some(stage8c_foreground(7_500, true)),
                Some(stage8c_addressed(7_500, true)),
                Stage8NoiseDegradationClass::Moderate,
                false,
                false,
                false,
                true,
                false,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedUnknownOrNonUserSpeaker => (
                Some(stage8c_foreground(5_500, false)),
                Some(stage8c_addressed(6_000, false)),
                Stage8NoiseDegradationClass::Moderate,
                false,
                false,
                false,
                false,
                true,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedHighNoiseOrDegradation => (
                Some(stage8c_foreground(6_000, true)),
                Some(stage8c_addressed(6_000, true)),
                Stage8NoiseDegradationClass::Severe,
                false,
                false,
                false,
                false,
                false,
                false,
            ),
            Stage8AudioSceneDisposition::BlockedRecordArtifactOnly => (
                None,
                None,
                Stage8NoiseDegradationClass::NotEvaluated,
                false,
                false,
                false,
                false,
                false,
                true,
            ),
            Stage8AudioSceneDisposition::AdvisoryOnly
            | Stage8AudioSceneDisposition::CleanForegroundAddressed => unreachable!(),
        };
        Stage8AudioScenePacket::v1(
            audio_scene_id,
            foreground,
            addressed,
            disposition,
            noise,
            echo,
            self_echo,
            background,
            overlapping,
            non_user,
            false,
            record_mode,
            "stage8c-blocked",
        )
        .expect("blocked stage 8c scene")
    }

    fn stage8d_target(
        target_id: &str,
        metric_name: &str,
        status: BenchmarkTargetStatus,
    ) -> BenchmarkTargetPacket {
        let status_reason = match status {
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction => {
                Some("stage8d_live_lab_deferred_to_later_slice".to_string())
            }
            _ => None,
        };
        let replay_corpus_ref = match status {
            BenchmarkTargetStatus::CertificationTargetPassed
            | BenchmarkTargetStatus::BaselineMeasured => {
                Some("stage8d_fixture_corpus_v1".to_string())
            }
            _ => None,
        };
        let certification_target_ref = match status {
            BenchmarkTargetStatus::CertificationTargetPassed => {
                Some("stage8d_deterministic_cert_target_v1".to_string())
            }
            _ => None,
        };
        BenchmarkTargetPacket::v1(
            target_id.to_string(),
            "stage8d_listening_lab".to_string(),
            "stage8d".to_string(),
            metric_name.to_string(),
            "stage8d_replay_fixture_threshold".to_string(),
            status,
            status_reason,
            replay_corpus_ref,
            certification_target_ref,
            MonotonicTimeNs(1),
        )
        .expect("stage8d benchmark target")
    }

    fn stage8d_result(
        result_id: &str,
        target: &BenchmarkTargetPacket,
        outcome: BenchmarkComparisonOutcome,
        status: BenchmarkTargetStatus,
    ) -> BenchmarkResultPacket {
        let (measured_value, replay_artifact_ref, evidence_hash, blocked_owner, next_action) =
            match status {
                BenchmarkTargetStatus::CertificationTargetPassed
                | BenchmarkTargetStatus::BaselineMeasured => (
                    Some("passed_stage8d_fixture_metric".to_string()),
                    Some("stage8d_replay_artifact_v1".to_string()),
                    Some(
                        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                            .to_string(),
                    ),
                    None,
                    None,
                ),
                BenchmarkTargetStatus::BlockedWithOwnerAndNextAction => (
                    None,
                    None,
                    None,
                    Some("stage8_live_listening_lab".to_string()),
                    Some("measure_live_provider_native_lab_after_explicit_allowance".to_string()),
                ),
                BenchmarkTargetStatus::NotApplicableWithReason => (None, None, None, None, None),
                BenchmarkTargetStatus::DraftTarget => unreachable!(),
            };
        BenchmarkResultPacket::v1(
            result_id.to_string(),
            target.benchmark_target_id.clone(),
            "stage8d_run_v1".to_string(),
            measured_value,
            outcome,
            status,
            None,
            replay_artifact_ref,
            evidence_hash,
            blocked_owner,
            next_action,
            MonotonicTimeNs(2),
        )
        .expect("stage8d benchmark result")
    }

    fn stage8e_target(
        target_id: &str,
        metric_name: &str,
        status: BenchmarkTargetStatus,
    ) -> BenchmarkTargetPacket {
        let status_reason = match status {
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction => {
                Some("stage8e_live_second_pass_lab_deferred".to_string())
            }
            _ => None,
        };
        let replay_corpus_ref = match status {
            BenchmarkTargetStatus::CertificationTargetPassed
            | BenchmarkTargetStatus::BaselineMeasured => {
                Some("stage8e_fixture_corpus_v1".to_string())
            }
            _ => None,
        };
        let certification_target_ref = match status {
            BenchmarkTargetStatus::CertificationTargetPassed => {
                Some("stage8e_deterministic_cert_target_v1".to_string())
            }
            _ => None,
        };
        BenchmarkTargetPacket::v1(
            target_id.to_string(),
            "stage8e_listening_repair_lab".to_string(),
            "stage8e".to_string(),
            metric_name.to_string(),
            "stage8e_replay_fixture_threshold".to_string(),
            status,
            status_reason,
            replay_corpus_ref,
            certification_target_ref,
            MonotonicTimeNs(1),
        )
        .expect("stage8e benchmark target")
    }

    fn stage8e_result(
        result_id: &str,
        target: &BenchmarkTargetPacket,
        outcome: BenchmarkComparisonOutcome,
        status: BenchmarkTargetStatus,
    ) -> BenchmarkResultPacket {
        let (measured_value, replay_artifact_ref, evidence_hash, blocked_owner, next_action) =
            match status {
                BenchmarkTargetStatus::CertificationTargetPassed
                | BenchmarkTargetStatus::BaselineMeasured => (
                    Some("passed_stage8e_fixture_metric".to_string()),
                    Some("stage8e_replay_artifact_v1".to_string()),
                    Some(
                        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
                            .to_string(),
                    ),
                    None,
                    None,
                ),
                BenchmarkTargetStatus::BlockedWithOwnerAndNextAction => (
                    None,
                    None,
                    None,
                    Some("stage8_live_repair_lab".to_string()),
                    Some(
                        "measure_live_second_pass_provider_native_lab_after_explicit_allowance"
                            .to_string(),
                    ),
                ),
                BenchmarkTargetStatus::NotApplicableWithReason => (None, None, None, None, None),
                BenchmarkTargetStatus::DraftTarget => unreachable!(),
            };
        BenchmarkResultPacket::v1(
            result_id.to_string(),
            target.benchmark_target_id.clone(),
            "stage8e_run_v1".to_string(),
            measured_value,
            outcome,
            status,
            None,
            replay_artifact_ref,
            evidence_hash,
            blocked_owner,
            next_action,
            MonotonicTimeNs(2),
        )
        .expect("stage8e benchmark result")
    }

    #[test]
    fn stage_8d_transcript_fixture_scores_exact_wer_cer_and_preserves_tokens() {
        let fixture = Stage8DTranscriptFixture::v1(
            "fixture-stage8d-exact",
            "selene hola set alpha timer yeah",
            "selene hola set alpha timer yeah",
            vec!["alpha".to_string()],
            vec!["hola".to_string()],
            vec!["yeah".to_string()],
        )
        .expect("stage8d transcript fixture");
        let metric = fixture
            .score("metric-stage8d-exact")
            .expect("stage8d transcript metric");

        assert!(metric.exact_match);
        assert!(metric.normalized_match);
        assert_eq!(metric.word_edits.total_errors(), 0);
        assert_eq!(metric.char_edits.total_errors(), 0);
        assert_eq!(metric.word_error_rate_bp(), 0);
        assert_eq!(metric.character_error_rate_bp(), 0);
        assert_eq!(metric.protected_token_mismatch_count, 0);
        assert!(metric.mixed_language_preserved);
        assert!(metric.slang_filler_preserved);
        assert!(!metric.empty_transcript);
        assert!(!metric.garbled_transcript);
    }

    #[test]
    fn stage_8d_transcript_fixture_counts_errors_and_protected_token_mismatch() {
        let fixture = Stage8DTranscriptFixture::v1(
            "fixture-stage8d-errors",
            "selene send token alpha amount nine",
            "selene send token beta amount nine",
            vec!["alpha".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .expect("stage8d transcript fixture");
        let metric = fixture
            .score("metric-stage8d-errors")
            .expect("stage8d transcript metric");

        assert!(!metric.exact_match);
        assert!(!metric.normalized_match);
        assert_eq!(metric.word_edits.substitutions, 1);
        assert_eq!(metric.word_edits.insertions, 0);
        assert_eq!(metric.word_edits.deletions, 0);
        assert_eq!(metric.protected_token_mismatch_count, 1);
        assert!(metric.word_error_rate_bp() > 0);

        let garbled = Stage8DTranscriptFixture::v1(
            "fixture-stage8d-garbled",
            "selene repeat alpha",
            "###",
            vec!["alpha".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .expect("stage8d garbled fixture")
        .score("metric-stage8d-garbled")
        .expect("stage8d garbled metric");
        assert!(garbled.garbled_transcript);
        assert!(garbled.protected_token_mismatch_count > 0);
    }

    #[test]
    fn stage_8d_endpoint_latency_classifies_deterministically() {
        let on_time = Stage8DEndpointLatencyMetricPacket::v1(
            "metric-stage8d-latency-on-time",
            1_000,
            2_000,
            2_050,
            2_240,
        )
        .expect("on-time endpoint metric");
        assert_eq!(on_time.endpoint_latency_ms, 240);
        assert_eq!(on_time.classification, Stage8DEndpointLatencyClass::OnTime);

        let late = Stage8DEndpointLatencyMetricPacket::v1(
            "metric-stage8d-latency-late",
            1_000,
            2_000,
            2_200,
            3_100,
        )
        .expect("late endpoint metric");
        assert_eq!(late.classification, Stage8DEndpointLatencyClass::Late);

        let premature = Stage8DEndpointLatencyMetricPacket::v1(
            "metric-stage8d-latency-premature",
            1_000,
            2_000,
            1_700,
            1_900,
        )
        .expect("premature endpoint metric");
        assert_eq!(
            premature.classification,
            Stage8DEndpointLatencyClass::Premature
        );

        let timeout = Stage8DEndpointLatencyMetricPacket::v1(
            "metric-stage8d-latency-timeout",
            1_000,
            2_000,
            2_500,
            5_500,
        )
        .expect("timeout endpoint metric");
        assert_eq!(
            timeout.classification,
            Stage8DEndpointLatencyClass::TimeoutOrDegraded
        );
    }

    #[test]
    fn stage_8d_scene_metrics_are_benchmark_evidence_only() {
        let clean = stage8c_clean_scene("audio-scene-stage8d-clean");
        let metric = Stage8DSceneCalibrationMetricPacket::from_scene(
            "metric-stage8d-scene-clean",
            &clean,
            1,
            1,
        )
        .expect("clean scene metric");
        assert_eq!(
            metric.foreground_confidence_bucket,
            Stage8DConfidenceBucket::High
        );
        assert_eq!(
            metric.addressed_confidence_bucket,
            Stage8DConfidenceBucket::High
        );
        assert_eq!(metric.diarization_segment_mismatch_count, 0);

        let overlap = stage8c_blocked_scene(
            "audio-scene-stage8d-overlap",
            Stage8AudioSceneDisposition::BlockedOverlappingSpeakers,
        );
        let overlap_metric = Stage8DSceneCalibrationMetricPacket::from_scene(
            "metric-stage8d-scene-overlap",
            &overlap,
            1,
            3,
        )
        .expect("overlap scene metric");
        assert_eq!(
            overlap_metric.overlapping_speaker_bucket,
            Stage8DSignalBucket::High
        );
        assert_eq!(overlap_metric.diarization_segment_mismatch_count, 2);
    }

    #[test]
    fn stage_8d_benchmark_packet_reuses_stage2_envelope_and_cannot_route() {
        let target = stage8d_target(
            "stage8d-target-deterministic",
            "wer_cer_endpoint_scene",
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let result = stage8d_result(
            "stage8d-result-deterministic",
            &target,
            BenchmarkComparisonOutcome::Passed,
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let transcript_metric = Stage8DTranscriptFixture::v1(
            "fixture-stage8d-envelope",
            "selene set alpha timer",
            "selene set alpha timer",
            vec!["alpha".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .expect("stage8d fixture")
        .score("metric-stage8d-envelope-transcript")
        .expect("stage8d transcript metric");
        let endpoint_metric = Stage8DEndpointLatencyMetricPacket::v1(
            "metric-stage8d-envelope-endpoint",
            1_000,
            2_000,
            2_010,
            2_200,
        )
        .expect("stage8d endpoint metric");
        let scene_metric = Stage8DSceneCalibrationMetricPacket::from_scene(
            "metric-stage8d-envelope-scene",
            &stage8c_clean_scene("audio-scene-stage8d-envelope"),
            1,
            1,
        )
        .expect("stage8d scene metric");

        let packet = Stage8DListeningBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8d-envelope",
            "metric-stage8d-envelope",
            "replay-stage8d-envelope",
            "audit-stage8d-envelope",
            Some(transcript_metric),
            Some(endpoint_metric),
            Some(scene_metric),
            Stage8DConfidenceBucket::High,
        )
        .expect("stage8d benchmark packet");

        assert_eq!(
            packet.target_status,
            BenchmarkTargetStatus::CertificationTargetPassed
        );
        assert!(!packet.can_route_or_mutate());
        assert!(!packet.work_authority.can_capture_microphone_audio);
        assert!(!packet.work_authority.can_transcribe_live_audio);
        assert!(!packet.work_authority.can_call_providers);
        assert!(!packet.work_authority.can_emit_tts);

        let invalid = Stage8DListeningBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8d-envelope",
            "metric-stage8d-envelope-empty",
            "replay-stage8d-envelope",
            "audit-stage8d-envelope",
            None,
            None,
            None,
            Stage8DConfidenceBucket::NotMeasured,
        );
        assert!(invalid.is_err());
    }

    #[test]
    fn stage_8d_blocked_live_benchmark_records_owner_without_metrics() {
        let target = stage8d_target(
            "stage8d-target-live-blocked",
            "live_far_field_stt_wer",
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction,
        );
        let result = stage8d_result(
            "stage8d-result-live-blocked",
            &target,
            BenchmarkComparisonOutcome::Blocked,
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction,
        );

        let packet = Stage8DListeningBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8d-live-blocked",
            "metric-stage8d-live-blocked",
            "replay-stage8d-live-blocked",
            "audit-stage8d-live-blocked",
            None,
            None,
            None,
            Stage8DConfidenceBucket::NotMeasured,
        )
        .expect("blocked live benchmark packet");

        assert_eq!(
            packet.target_status,
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction
        );
        assert_eq!(
            packet.comparison_outcome,
            BenchmarkComparisonOutcome::Blocked
        );
        assert!(!packet.can_route_or_mutate());
    }

    #[test]
    fn stage_8d_benchmark_packet_rejects_mismatched_stage2_result() {
        let target = stage8d_target(
            "stage8d-target-match-a",
            "wer_cer",
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let other_target = stage8d_target(
            "stage8d-target-match-b",
            "wer_cer",
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let result = stage8d_result(
            "stage8d-result-mismatch",
            &other_target,
            BenchmarkComparisonOutcome::Passed,
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let transcript_metric = Stage8DTranscriptFixture::v1(
            "fixture-stage8d-mismatch",
            "selene set beta timer",
            "selene set beta timer",
            vec!["beta".to_string()],
            Vec::new(),
            Vec::new(),
        )
        .expect("stage8d mismatch fixture")
        .score("metric-stage8d-mismatch")
        .expect("stage8d mismatch metric");

        let packet = Stage8DListeningBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8d-mismatch",
            "metric-stage8d-mismatch",
            "replay-stage8d-mismatch",
            "audit-stage8d-mismatch",
            Some(transcript_metric),
            None,
            None,
            Stage8DConfidenceBucket::High,
        );

        assert!(packet.is_err());
    }

    #[test]
    fn stage_8e_accent_mixed_language_and_domain_fixture_scores_without_authority() {
        let protected_tokens = vec!["alpha99".to_string()];
        let domain_tokens = vec!["domainx".to_string()];
        let mixed_tokens = vec!["hola".to_string()];
        let fixture = Stage8ERepairBenchmarkFixture::v1(
            "fixture-stage8e-accent",
            "selene hola set alpha99 timer domainx",
            "selene hola set alpha99 timer domainx",
            Some("accent-bucket-alpha".to_string()),
            mixed_tokens,
            domain_tokens.clone(),
            protected_tokens.clone(),
            Some("vocab-pack-alpha".to_string()),
            Some("pron-profile-alpha".to_string()),
        )
        .expect("stage8e fixture");
        let candidate = Stage8EAlternativeTranscriptCandidate::fixture_offline(
            "candidate-stage8e-accent-1",
            1,
            "selene hola set alpha99 timer domainx",
            Stage8DConfidenceBucket::High,
            &protected_tokens,
        )
        .expect("stage8e candidate");
        let candidate_set = Stage8EAlternativeTranscriptCandidateSetPacket::v1(
            "candidate-set-stage8e-accent",
            "fixture-stage8e-accent",
            vec![candidate],
            Some("candidate-stage8e-accent-1".to_string()),
        )
        .expect("stage8e candidate set");
        let repair = Stage8ERepairDecisionPacket::fixture_only(
            "repair-stage8e-accent",
            "fixture-stage8e-accent",
            "selene hola set alpha99 timer domainx",
            "selene hola set alpha99 timer domainx",
            Some("selene hola set alpha99 timer domainx".to_string()),
            &protected_tokens,
            &domain_tokens,
        )
        .expect("stage8e repair");
        let metric = fixture
            .score("metric-stage8e-accent", &candidate_set, &repair)
            .expect("stage8e metric");

        assert!(metric.accent_benchmark_only);
        assert!(metric.mixed_language_preserved);
        assert!(metric.domain_vocabulary_preserved);
        assert_eq!(metric.protected_token_mismatch_count, 0);
        assert_eq!(metric.language_script_token_mismatch_count, 0);
        assert_eq!(metric.domain_token_mismatch_count, 0);
        assert_eq!(
            metric.repair_disposition,
            Stage8ERepairDisposition::AcceptedFixtureNormalization
        );
    }

    #[test]
    fn stage_8e_alternative_candidates_are_bounded_ordered_and_non_committing() {
        let protected_tokens = vec!["alpha99".to_string()];
        let first = Stage8EAlternativeTranscriptCandidate::fixture_offline(
            "candidate-stage8e-order-1",
            1,
            "selene set alpha99 timer",
            Stage8DConfidenceBucket::High,
            &protected_tokens,
        )
        .expect("first candidate");
        let second = Stage8EAlternativeTranscriptCandidate::fixture_offline(
            "candidate-stage8e-order-2",
            2,
            "selene set alpha99 timer please",
            Stage8DConfidenceBucket::Medium,
            &protected_tokens,
        )
        .expect("second candidate");
        let set = Stage8EAlternativeTranscriptCandidateSetPacket::v1(
            "candidate-set-stage8e-order",
            "fixture-stage8e-order",
            vec![first.clone(), second],
            Some("candidate-stage8e-order-1".to_string()),
        )
        .expect("ordered candidate set");

        assert_eq!(set.candidates.len(), 2);
        assert_eq!(
            set.selected_candidate().map(|candidate| candidate.rank),
            Some(1)
        );
        assert!(set
            .candidates
            .iter()
            .all(|candidate| !candidate.can_commit_directly));

        let mut committing = first.clone();
        committing.can_commit_directly = true;
        assert!(committing.validate().is_err());

        let rank_gap = Stage8EAlternativeTranscriptCandidate::fixture_offline(
            "candidate-stage8e-order-3",
            3,
            "selene set alpha99 timer",
            Stage8DConfidenceBucket::Low,
            &protected_tokens,
        )
        .expect("rank gap candidate");
        let invalid_set = Stage8EAlternativeTranscriptCandidateSetPacket::v1(
            "candidate-set-stage8e-rank-gap",
            "fixture-stage8e-order",
            vec![first, rank_gap],
            None,
        );
        assert!(invalid_set.is_err());
    }

    #[test]
    fn stage_8e_second_pass_repair_rejects_protected_invention_and_overrepair() {
        let protected_tokens = vec!["alpha99".to_string()];
        let domain_tokens = vec!["domainx".to_string()];
        let invented = Stage8ERepairDecisionPacket::fixture_only(
            "repair-stage8e-invented",
            "fixture-stage8e-invented",
            "selene send amount nine domainx",
            "selene send alpha99 amount nine domainx",
            Some("selene send alpha99 amount nine domainx".to_string()),
            &protected_tokens,
            &domain_tokens,
        )
        .expect("invented protected token repair");

        assert_eq!(
            invented.disposition,
            Stage8ERepairDisposition::RejectedProtectedTokenInvented
        );
        assert!(invented.protected_token_invented);
        assert!(!invented.accepted_repair);

        let over_repair = Stage8ERepairDecisionPacket::fixture_only(
            "repair-stage8e-over",
            "fixture-stage8e-over",
            "selene send alpha99 amount nine domainx",
            "selene send alpha99 amount nine domainx",
            Some("selene send alpha99 amount nine domainx extra one two three".to_string()),
            &protected_tokens,
            &domain_tokens,
        )
        .expect("over repair");

        assert_eq!(
            over_repair.disposition,
            Stage8ERepairDisposition::RejectedOverRepair
        );
        assert!(over_repair.over_repair_detected);
        assert!(!over_repair.accepted_repair);
    }

    #[test]
    fn stage_8e_repair_accepts_punctuation_and_case_without_meaning_drift() {
        let protected_tokens = vec!["alpha99".to_string()];
        let repair = Stage8ERepairDecisionPacket::fixture_only(
            "repair-stage8e-normalize",
            "fixture-stage8e-normalize",
            "SELENE set alpha99 timer!!!",
            "selene set alpha99 timer",
            Some("selene set alpha99 timer".to_string()),
            &protected_tokens,
            &[],
        )
        .expect("normalizing repair");

        assert!(repair.accepted_repair);
        assert_eq!(
            repair.disposition,
            Stage8ERepairDisposition::AcceptedFixtureNormalization
        );
        assert_eq!(repair.protected_token_mismatch_count, 0);
        assert!(!repair.meaning_drift_detected);
        assert!(!repair.over_repair_detected);
    }

    #[test]
    fn stage_8e_packet_reuses_stage2_envelope_and_blocks_live_work() {
        let protected_tokens = vec!["alpha99".to_string()];
        let domain_tokens = vec!["domainx".to_string()];
        let fixture = Stage8ERepairBenchmarkFixture::v1(
            "fixture-stage8e-envelope",
            "selene hola set alpha99 timer domainx",
            "selene hola set alpha99 timer domainx",
            Some("accent-bucket-beta".to_string()),
            vec!["hola".to_string()],
            domain_tokens.clone(),
            protected_tokens.clone(),
            Some("vocab-pack-beta".to_string()),
            Some("pron-profile-beta".to_string()),
        )
        .expect("stage8e envelope fixture");
        let candidate = Stage8EAlternativeTranscriptCandidate::fixture_offline(
            "candidate-stage8e-envelope-1",
            1,
            "selene hola set alpha99 timer domainx",
            Stage8DConfidenceBucket::High,
            &protected_tokens,
        )
        .expect("stage8e envelope candidate");
        let candidate_set = Stage8EAlternativeTranscriptCandidateSetPacket::v1(
            "candidate-set-stage8e-envelope",
            "fixture-stage8e-envelope",
            vec![candidate],
            Some("candidate-stage8e-envelope-1".to_string()),
        )
        .expect("stage8e envelope candidate set");
        let repair = Stage8ERepairDecisionPacket::fixture_only(
            "repair-stage8e-envelope",
            "fixture-stage8e-envelope",
            "selene hola set alpha99 timer domainx",
            "selene hola set alpha99 timer domainx",
            Some("selene hola set alpha99 timer domainx".to_string()),
            &protected_tokens,
            &domain_tokens,
        )
        .expect("stage8e envelope repair");
        let metric = fixture
            .score("metric-stage8e-envelope", &candidate_set, &repair)
            .expect("stage8e envelope metric");
        let target = stage8e_target(
            "stage8e-target-envelope",
            "accent_mixed_domain_alternatives_repair",
            BenchmarkTargetStatus::CertificationTargetPassed,
        );
        let result = stage8e_result(
            "stage8e-result-envelope",
            &target,
            BenchmarkComparisonOutcome::Passed,
            BenchmarkTargetStatus::CertificationTargetPassed,
        );

        let packet = Stage8EListeningRepairBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8e-envelope",
            "metric-stage8e-envelope",
            "candidate-set-stage8e-envelope",
            "repair-stage8e-envelope",
            Some("vocab-pack-beta".to_string()),
            Some("pron-profile-beta".to_string()),
            "replay-stage8e-envelope",
            "audit-stage8e-envelope",
            Some(metric),
        )
        .expect("stage8e benchmark packet");

        assert_eq!(
            packet.target_status,
            BenchmarkTargetStatus::CertificationTargetPassed
        );
        assert!(!packet.can_route_or_mutate());
        assert!(!packet.work_authority.can_capture_microphone_audio);
        assert!(!packet.work_authority.can_transcribe_live_audio);
        assert!(!packet.work_authority.can_call_providers);
        assert!(!packet.work_authority.can_emit_tts);
        assert!(!packet.work_authority.can_promote_provider_model_router);
    }

    #[test]
    fn stage_8e_blocked_live_repair_benchmark_records_owner_without_metrics() {
        let target = stage8e_target(
            "stage8e-target-live-blocked",
            "live_second_pass_provider_repair",
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction,
        );
        let result = stage8e_result(
            "stage8e-result-live-blocked",
            &target,
            BenchmarkComparisonOutcome::Blocked,
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction,
        );

        let packet = Stage8EListeningRepairBenchmarkPacket::from_stage2_envelope(
            &target,
            &result,
            "fixture-stage8e-live-blocked",
            "metric-stage8e-live-blocked",
            "candidate-set-stage8e-live-blocked",
            "repair-stage8e-live-blocked",
            None,
            None,
            "replay-stage8e-live-blocked",
            "audit-stage8e-live-blocked",
            None,
        )
        .expect("blocked stage8e benchmark packet");

        assert_eq!(
            packet.target_status,
            BenchmarkTargetStatus::BlockedWithOwnerAndNextAction
        );
        assert_eq!(
            packet.comparison_outcome,
            BenchmarkComparisonOutcome::Blocked
        );
        assert!(!packet.can_route_or_mutate());
    }

    #[test]
    fn stage_8a_audio_substrate_consumes_stage7_activation_without_downstream_work() {
        let packet = Stage8TranscriptGatePacket::audio_substrate_only(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8",
        )
        .expect("audio substrate packet");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::AudioSubstrateOnly
        );
        assert!(packet.work_authority.can_update_listen_state);
        assert!(!packet.can_route_or_mutate());
        assert!(!packet.can_emit_committed_turn());
        assert!(packet.transcript_id.is_none());
        assert!(packet.candidate_preview.is_none());
        assert!(packet.committed_turn.is_none());
        assert_eq!(packet.consent_state_id.as_deref(), Some("consent-stage4"));
        assert_eq!(
            packet.provider_budget_id.as_deref(),
            Some("provider-budget-stage4")
        );
        assert_eq!(
            packet.access_context_id.as_deref(),
            Some("access-context-stage8")
        );
    }

    #[test]
    fn stage_8a_partial_transcript_is_preview_only_and_cannot_commit() {
        let packet = Stage8TranscriptGatePacket::partial_transcript_preview(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8",
            "transcript-stage8-partial",
            "partially heard words",
            8_800,
            1,
        )
        .expect("partial transcript preview");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::PartialTranscriptPreviewOnly
        );
        assert!(packet.is_preview_only());
        assert!(!packet.can_route_or_mutate());
        assert!(!packet.can_emit_committed_turn());
        assert!(packet.committed_turn.is_none());
        assert!(packet.stage5_turn_authority.is_none());
        assert!(packet.candidate_preview.is_some());
        assert_eq!(packet.partial_revision_id, Some(1));
        assert_eq!(packet.confidence_bp, Some(8_800));
    }

    #[test]
    fn stage_8a_final_transcript_commit_requires_stage5_current_turn_authority() {
        let current = stage8_current_authority();
        let packet = Stage8TranscriptGatePacket::final_transcript_commit(
            stage8_explicit_mic_activation(Some(current.session_id)),
            current.clone(),
            "audio-scene-stage8",
            "endpoint-stage8-final",
            "confidence-gate-stage8",
            "transcript-stage8-final",
            "final exact transcript",
            "en-US",
            9_600,
            9_400,
        )
        .expect("final transcript commit boundary");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::FinalTranscriptCommitBoundary
        );
        assert!(packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
        assert_eq!(packet.session_id, Some(SessionId(88)));
        assert_eq!(packet.turn_id, Some(TurnId(8)));
        assert!(packet.candidate_preview.is_none());
        let committed_turn = packet.committed_turn.as_ref().expect("committed turn");
        assert!(committed_turn.is_committed_live_turn());
        assert_eq!(committed_turn.modality, Some(CanonicalTurnModality::Voice));

        let stale = Stage5TurnAuthorityPacket::quarantined(
            SessionId(88),
            Some(TurnId(8)),
            Some("device-stage8".to_string()),
            Some(4),
            SessionState::Active,
            Stage5TurnAuthorityDisposition::SupersededTurnQuarantined,
        )
        .expect("stale authority");
        let rejected = Stage8TranscriptGatePacket::final_transcript_commit(
            stage8_explicit_mic_activation(Some(SessionId(88))),
            stale,
            "audio-scene-stage8",
            "endpoint-stage8-final",
            "confidence-gate-stage8",
            "transcript-stage8-final",
            "final exact transcript",
            "en-US",
            9_600,
            9_400,
        );
        assert!(rejected.is_err());
    }

    #[test]
    fn stage_8b_vad_endpoint_boundary_is_inert_even_when_endpoint_final() {
        let packet = Stage8TranscriptGatePacket::vad_endpoint_boundary(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8b",
            "vad-signal-stage8b",
            "endpoint-stage8b",
            Stage8EndpointState::EndpointFinal,
        )
        .expect("endpoint boundary packet");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::VadEndpointBoundaryOnly
        );
        assert_eq!(packet.endpoint_state, Stage8EndpointState::EndpointFinal);
        assert!(packet.work_authority.can_update_listen_state);
        assert!(!packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
        assert!(packet.transcript_id.is_none());
        assert!(packet.candidate_preview.is_none());
        assert!(packet.committed_turn.is_none());
    }

    #[test]
    fn stage_8b_final_commit_requires_endpoint_final_and_confidence_pass() {
        let current = stage8_current_authority();
        let packet = Stage8TranscriptGatePacket::final_transcript_commit(
            stage8_explicit_mic_activation(Some(current.session_id)),
            current,
            "audio-scene-stage8b",
            "endpoint-stage8b-final",
            "confidence-gate-stage8b",
            "transcript-stage8b-final",
            "send the report",
            "en-US",
            9_200,
            8_800,
        )
        .expect("final transcript commit");

        assert_eq!(packet.endpoint_state, Stage8EndpointState::EndpointFinal);
        assert_eq!(
            packet.confidence_gate,
            Stage8ConfidenceGateDisposition::Passed
        );
        assert!(packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());

        let mut missing_endpoint = packet.clone();
        missing_endpoint.endpoint_state = Stage8EndpointState::EndpointCandidate;
        assert!(missing_endpoint.validate().is_err());

        let mut low_confidence = packet.clone();
        low_confidence.confidence_bp = Some(8_499);
        assert!(low_confidence.validate().is_err());

        let mut low_coverage = packet;
        low_coverage.coverage_bp = Some(6_999);
        assert!(low_coverage.validate().is_err());
    }

    #[test]
    fn stage_8b_confidence_rejections_cannot_commit_or_enter_understanding() {
        for gate in [
            Stage8ConfidenceGateDisposition::RejectedLowConfidence,
            Stage8ConfidenceGateDisposition::RejectedLowCoverage,
            Stage8ConfidenceGateDisposition::RejectedEmptyTranscript,
            Stage8ConfidenceGateDisposition::RejectedGarbledTranscript,
            Stage8ConfidenceGateDisposition::RejectedEchoSuspect,
            Stage8ConfidenceGateDisposition::RejectedBackgroundOrNonUser,
            Stage8ConfidenceGateDisposition::RejectedStaleOrClosedTurn,
        ] {
            let (transcript_id, transcript_text) =
                if gate == Stage8ConfidenceGateDisposition::RejectedEmptyTranscript {
                    (None, None)
                } else {
                    (
                        Some("transcript-stage8b-rejected".to_string()),
                        Some("uncertain transcript fragment".to_string()),
                    )
                };
            let packet = Stage8TranscriptGatePacket::confidence_gate_reject(
                stage8_explicit_mic_activation(None),
                "audio-scene-stage8b",
                "endpoint-stage8b-rejected",
                "confidence-gate-stage8b-rejected",
                gate,
                transcript_id,
                transcript_text,
                Some(5_500),
                Some(6_000),
                Stage8ProtectedSlotDisposition::NotApplicable,
                Vec::new(),
            )
            .expect("confidence rejection");

            assert_eq!(
                packet.boundary_kind,
                Stage8TranscriptGateKind::ConfidenceGateRejected
            );
            assert!(!packet.can_emit_committed_turn());
            assert!(!packet.work_authority.can_enter_understanding);
            assert!(!packet.can_route_or_mutate());
            assert!(packet.committed_turn.is_none());
        }
    }

    #[test]
    fn stage_8b_low_confidence_protected_slots_clarify_or_fail_closed_without_guessing() {
        let uncertainty = Stage8ProtectedSlotUncertainty::v1(
            Stage8ProtectedSlotKind::Recipient,
            "recipient",
            4_200,
        )
        .expect("protected slot uncertainty");
        let clarify = Stage8TranscriptGatePacket::confidence_gate_reject(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8b",
            "endpoint-stage8b-protected",
            "confidence-gate-stage8b-protected",
            Stage8ConfidenceGateDisposition::RejectedLowConfidence,
            Some("transcript-stage8b-protected".to_string()),
            Some("send it to maybe alex".to_string()),
            Some(4_900),
            Some(8_800),
            Stage8ProtectedSlotDisposition::ClarificationRequired,
            vec![uncertainty.clone()],
        )
        .expect("protected slot clarification");

        assert_eq!(
            clarify.protected_slot_disposition,
            Stage8ProtectedSlotDisposition::ClarificationRequired
        );
        assert!(!clarify.can_emit_committed_turn());
        assert!(clarify.committed_turn.is_none());

        let fail_closed = Stage8TranscriptGatePacket::confidence_gate_reject(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8b",
            "endpoint-stage8b-protected",
            "confidence-gate-stage8b-protected-fail",
            Stage8ConfidenceGateDisposition::RejectedLowConfidence,
            Some("transcript-stage8b-protected".to_string()),
            Some("approve invoice maybe nine hundred".to_string()),
            Some(4_600),
            Some(8_700),
            Stage8ProtectedSlotDisposition::FailClosed,
            vec![uncertainty],
        )
        .expect("protected slot fail closed");
        assert_eq!(
            fail_closed.protected_slot_disposition,
            Stage8ProtectedSlotDisposition::FailClosed
        );
        assert!(!fail_closed.can_emit_committed_turn());

        let current = stage8_current_authority();
        let mut invalid_commit = Stage8TranscriptGatePacket::final_transcript_commit(
            stage8_explicit_mic_activation(Some(current.session_id)),
            current,
            "audio-scene-stage8b",
            "endpoint-stage8b-final",
            "confidence-gate-stage8b-final",
            "transcript-stage8b-final",
            "send it to alex",
            "en-US",
            9_200,
            9_000,
        )
        .expect("final commit");
        invalid_commit.protected_slot_disposition =
            Stage8ProtectedSlotDisposition::ClarificationRequired;
        invalid_commit.protected_slot_uncertainties = vec![Stage8ProtectedSlotUncertainty::v1(
            Stage8ProtectedSlotKind::Recipient,
            "recipient",
            4_200,
        )
        .expect("uncertainty")];
        assert!(invalid_commit.validate().is_err());
    }

    #[test]
    fn stage_8b_record_mode_cannot_enter_endpoint_or_confidence_commit_path() {
        let rejected = Stage8TranscriptGatePacket::vad_endpoint_boundary(
            stage8_record_activation(),
            "audio-scene-stage8b-record",
            "vad-signal-stage8b-record",
            "endpoint-stage8b-record",
            Stage8EndpointState::EndpointFinal,
        );
        assert!(rejected.is_err());

        let rejected = Stage8TranscriptGatePacket::confidence_gate_reject(
            stage8_record_activation(),
            "audio-scene-stage8b-record",
            "endpoint-stage8b-record",
            "confidence-gate-stage8b-record",
            Stage8ConfidenceGateDisposition::RejectedLowConfidence,
            Some("transcript-stage8b-record".to_string()),
            Some("recorded artifact text".to_string()),
            Some(5_000),
            Some(8_000),
            Stage8ProtectedSlotDisposition::NotApplicable,
            Vec::new(),
        );
        assert!(rejected.is_err());
    }

    #[test]
    fn stage_8c_audio_scene_boundary_is_advisory_and_inert() {
        let scene = Stage8AudioScenePacket::v1(
            "audio-scene-stage8c-advisory",
            Some(stage8c_foreground(8_800, true)),
            Some(stage8c_addressed(8_700, true)),
            Stage8AudioSceneDisposition::AdvisoryOnly,
            Stage8NoiseDegradationClass::Clear,
            false,
            false,
            false,
            false,
            false,
            true,
            false,
            "stage8c-advisory",
        )
        .expect("advisory audio scene");
        let packet = Stage8TranscriptGatePacket::audio_scene_boundary(
            stage8_explicit_mic_activation(None),
            scene,
        )
        .expect("audio scene boundary");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::AudioSceneBoundaryOnly
        );
        assert!(packet.work_authority.can_update_listen_state);
        assert!(!packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
        assert!(packet.transcript_id.is_none());
        assert!(packet.candidate_preview.is_none());
        assert!(packet.committed_turn.is_none());
        assert!(packet.stage5_turn_authority.is_none());
        let scene = packet.audio_scene_packet.as_ref().expect("scene packet");
        assert!(scene.barge_in_or_interruption_marker);
        assert!(!scene.has_blocking_signal());
    }

    #[test]
    fn stage_8c_foreground_and_addressed_are_advisory_not_identity_or_authority() {
        let scene = stage8c_clean_scene("audio-scene-stage8c-clean");
        assert!(scene.clean_foreground_addressed());
        let packet = Stage8TranscriptGatePacket::audio_scene_boundary(
            stage8_explicit_mic_activation(None),
            scene,
        )
        .expect("clean scene boundary");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::AudioSceneBoundaryOnly
        );
        assert!(packet.audio_scene_packet.is_some());
        assert!(!packet.work_authority.can_trigger_voice_id_matching);
        assert!(!packet.work_authority.can_authorize);
        assert!(!packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
    }

    #[test]
    fn stage_8c_echo_noise_overlap_background_and_non_user_block_before_commit() {
        for disposition in [
            Stage8AudioSceneDisposition::BlockedLowAddressingConfidence,
            Stage8AudioSceneDisposition::BlockedBackgroundSpeech,
            Stage8AudioSceneDisposition::BlockedSelfEcho,
            Stage8AudioSceneDisposition::BlockedOverlappingSpeakers,
            Stage8AudioSceneDisposition::BlockedUnknownOrNonUserSpeaker,
            Stage8AudioSceneDisposition::BlockedHighNoiseOrDegradation,
        ] {
            let packet = Stage8TranscriptGatePacket::listening_scene_blocked(
                stage8_explicit_mic_activation(None),
                stage8c_blocked_scene("audio-scene-stage8c-blocked", disposition),
            )
            .expect("blocked listening scene");

            assert_eq!(
                packet.boundary_kind,
                Stage8TranscriptGateKind::ListeningSceneBlocked
            );
            assert!(!packet.work_authority.can_update_listen_state);
            assert!(!packet.can_emit_committed_turn());
            assert!(!packet.work_authority.can_enter_understanding);
            assert!(!packet.can_route_or_mutate());
            assert!(packet.committed_turn.is_none());
            assert!(packet.stage5_turn_authority.is_none());
            assert!(packet
                .audio_scene_packet
                .as_ref()
                .expect("scene")
                .has_blocking_signal());
        }
    }

    #[test]
    fn stage_8c_scene_cannot_replace_stage5_authority_or_confidence_gate() {
        let scene = stage8c_clean_scene("audio-scene-stage8c-final");
        let current = stage8_current_authority();
        let mut packet = Stage8TranscriptGatePacket::final_transcript_commit(
            stage8_explicit_mic_activation(Some(current.session_id)),
            current.clone(),
            "audio-scene-stage8c-final",
            "endpoint-stage8c-final",
            "confidence-gate-stage8c-final",
            "transcript-stage8c-final",
            "answer the question",
            "en-US",
            9_300,
            9_100,
        )
        .expect("final transcript commit");
        packet.audio_scene_packet = Some(scene);
        packet.validate().expect("clean scene may be attached");
        assert!(packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());

        let mut missing_authority = packet.clone();
        missing_authority.stage5_turn_authority = None;
        assert!(missing_authority.validate().is_err());

        let mut low_confidence = packet;
        low_confidence.confidence_bp = Some(7_999);
        assert!(low_confidence.validate().is_err());
    }

    #[test]
    fn stage_8c_record_mode_scene_evidence_stays_artifact_only() {
        let scene = stage8c_blocked_scene(
            "audio-scene-stage8c-record",
            Stage8AudioSceneDisposition::BlockedRecordArtifactOnly,
        );
        let rejected =
            Stage8TranscriptGatePacket::listening_scene_blocked(stage8_record_activation(), scene);
        assert!(rejected.is_err());

        let artifact = Stage8TranscriptGatePacket::record_audio_artifact_only(
            stage8_record_activation(),
            "audio-scene-stage8c-record",
        )
        .expect("record artifact boundary");
        assert_eq!(
            artifact.boundary_kind,
            Stage8TranscriptGateKind::RecordAudioArtifactOnly
        );
        assert!(artifact.record_mode_audio);
        assert!(!artifact.can_emit_committed_turn());
        assert!(!artifact.can_route_or_mutate());
        assert!(artifact.committed_turn.is_none());
    }

    #[test]
    fn stage_8a_background_and_tts_self_echo_cannot_create_turns() {
        let packet = Stage8TranscriptGatePacket::blocked_background_or_self_echo(
            stage8_explicit_mic_activation(None),
            "audio-scene-stage8",
            true,
            false,
            false,
            false,
        )
        .expect("self-echo blocked");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::BackgroundOrSelfEchoBlocked
        );
        assert!(!packet.work_authority.can_update_listen_state);
        assert!(!packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
        assert!(packet.transcript_id.is_none());
        assert!(packet.candidate_preview.is_none());
        assert!(packet.committed_turn.is_none());
    }

    #[test]
    fn stage_8a_record_mode_audio_remains_artifact_only() {
        let packet = Stage8TranscriptGatePacket::record_audio_artifact_only(
            stage8_record_activation(),
            "audio-scene-stage8-record",
        )
        .expect("record-mode artifact-only audio");

        assert_eq!(
            packet.boundary_kind,
            Stage8TranscriptGateKind::RecordAudioArtifactOnly
        );
        assert!(packet.record_mode_audio);
        assert!(!packet.work_authority.can_update_listen_state);
        assert!(!packet.can_emit_committed_turn());
        assert!(!packet.can_route_or_mutate());
        assert!(packet.transcript_id.is_none());
        assert!(packet.candidate_preview.is_none());
        assert!(packet.committed_turn.is_none());

        let rejected = Stage8TranscriptGatePacket::partial_transcript_preview(
            stage8_record_activation(),
            "audio-scene-stage8-record",
            "transcript-stage8-record-partial",
            "recorded words",
            8_000,
            1,
        );
        assert!(rejected.is_err());
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
