#![forbid(unsafe_code)]

use crate::ph1_voice_id::UserId;
use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1l::SessionId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1F_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1F_INTERNAL_HISTORY_REF_MAX_CHARS: usize = 160;
pub const PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS: usize = 128;
pub const PH1F_INTERNAL_HISTORY_MAX_REFS: usize = 48;
pub const PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS: usize = 160;

fn validate_optional_history_text(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not be empty when provided",
            });
        }
        if value.len() > max_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "exceeds max length",
            });
        }
        if value.chars().any(char::is_control) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain control characters",
            });
        }
    }
    Ok(())
}

fn validate_history_ref_list(
    field: &'static str,
    values: &[String],
) -> Result<(), ContractViolation> {
    if values.len() > PH1F_INTERNAL_HISTORY_MAX_REFS {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max refs",
        });
    }
    for value in values {
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "refs must not be empty",
            });
        }
        if value.len() > PH1F_INTERNAL_HISTORY_REF_MAX_CHARS {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "ref exceeds max length",
            });
        }
        if value.chars().any(char::is_control) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "refs must not contain control characters",
            });
        }
    }
    Ok(())
}

fn validate_optional_confidence_bp(
    field: &'static str,
    value: Option<u16>,
) -> Result<(), ContractViolation> {
    if let Some(value) = value {
        if value > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must be <= 10000 basis points",
            });
        }
    }
    Ok(())
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct ConversationTurnId(pub u64);

impl Validate for ConversationTurnId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConversationRole {
    User,
    Selene,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ConversationSource {
    VoiceTranscript,
    TypedText,
    SeleneOutput,
    Tombstone,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PrivacyScope {
    PublicChat,
    PrivateDelivery,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct InternalHistoryEventId(pub u64);

impl Validate for InternalHistoryEventId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history_event_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InternalHistoryEventKind {
    CommittedTurn,
    RejectedInput,
    LifecycleBoundary,
    ToolEvidence,
    CorrectionEvent,
    ProtectedFailClosed,
    MemoryEvidence,
    MultimodalEvidence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum InternalHistoryModality {
    Voice,
    Typed,
    File,
    Image,
    System,
    Lifecycle,
    Multimodal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SpeakerIdentityPosture {
    Known,
    Unknown,
    Guest,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TranscriptEvidenceStatus {
    Accepted,
    Rejected,
    Empty,
    NoiseRejected,
    SelfEchoRejected,
    TypedCommitted,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum TtsEvidenceStatus {
    NotRequested,
    Requested,
    Ready,
    PlaybackStarted,
    PlaybackEnded,
    FailedClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MemoryCandidateStatus {
    Allowed,
    BlockedRejectedTranscript,
    BlockedNoise,
    BlockedSelfEcho,
    BlockedProtected,
    BlockedPrivacy,
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SpeakerEvidenceRefs {
    pub user_id: Option<UserId>,
    pub actor_id: Option<String>,
    pub device_id: Option<DeviceId>,
    pub speaker_id: Option<String>,
    pub speaker_label: Option<String>,
    pub voice_profile_id: Option<String>,
    pub identity_posture: SpeakerIdentityPosture,
    pub voice_id_confidence_bp: Option<u16>,
    pub voice_id_score_bp: Option<u16>,
    pub voice_id_margin_bp: Option<u16>,
    pub same_speaker_as_previous: Option<bool>,
    pub speaker_changed: Option<bool>,
    pub voice_identity_assertion_ref: Option<String>,
    pub liveness_ref: Option<String>,
    pub capture_attestation_ref: Option<String>,
    pub typed_actor_identity_ref: Option<String>,
    pub privacy_scope_ref: Option<String>,
    pub memory_scope_ref: Option<String>,
    pub access_posture_ref: Option<String>,
}

impl SpeakerEvidenceRefs {
    pub fn none() -> Self {
        Self {
            user_id: None,
            actor_id: None,
            device_id: None,
            speaker_id: None,
            speaker_label: None,
            voice_profile_id: None,
            identity_posture: SpeakerIdentityPosture::NotApplicable,
            voice_id_confidence_bp: None,
            voice_id_score_bp: None,
            voice_id_margin_bp: None,
            same_speaker_as_previous: None,
            speaker_changed: None,
            voice_identity_assertion_ref: None,
            liveness_ref: None,
            capture_attestation_ref: None,
            typed_actor_identity_ref: None,
            privacy_scope_ref: None,
            memory_scope_ref: None,
            access_posture_ref: None,
        }
    }

    pub fn typed_actor(user_id: UserId, device_id: Option<DeviceId>) -> Self {
        let mut refs = Self::none();
        refs.user_id = Some(user_id.clone());
        refs.actor_id = Some(user_id.as_str().to_string());
        refs.device_id = device_id;
        refs.identity_posture = SpeakerIdentityPosture::Known;
        refs.typed_actor_identity_ref = Some(format!("typed_actor:{}", user_id.as_str()));
        refs
    }

    pub fn voice_unknown(user_id: UserId, device_id: Option<DeviceId>) -> Self {
        let mut refs = Self::none();
        refs.user_id = Some(user_id.clone());
        refs.actor_id = Some(user_id.as_str().to_string());
        refs.device_id = device_id;
        refs.identity_posture = SpeakerIdentityPosture::Unknown;
        refs
    }
}

impl Validate for SpeakerEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(user_id) = &self.user_id {
            if user_id.as_str().trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "speaker_evidence.user_id",
                    reason: "must not be empty when provided",
                });
            }
        }
        if let Some(device_id) = &self.device_id {
            device_id.validate()?;
        }
        validate_optional_history_text(
            "speaker_evidence.actor_id",
            &self.actor_id,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.speaker_id",
            &self.speaker_id,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.speaker_label",
            &self.speaker_label,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.voice_profile_id",
            &self.voice_profile_id,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_confidence_bp(
            "speaker_evidence.voice_id_confidence_bp",
            self.voice_id_confidence_bp,
        )?;
        validate_optional_confidence_bp(
            "speaker_evidence.voice_id_score_bp",
            self.voice_id_score_bp,
        )?;
        validate_optional_confidence_bp(
            "speaker_evidence.voice_id_margin_bp",
            self.voice_id_margin_bp,
        )?;
        validate_optional_history_text(
            "speaker_evidence.voice_identity_assertion_ref",
            &self.voice_identity_assertion_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.liveness_ref",
            &self.liveness_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.capture_attestation_ref",
            &self.capture_attestation_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.typed_actor_identity_ref",
            &self.typed_actor_identity_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.privacy_scope_ref",
            &self.privacy_scope_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.memory_scope_ref",
            &self.memory_scope_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "speaker_evidence.access_posture_ref",
            &self.access_posture_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        if self.typed_actor_identity_ref.is_some() && self.voice_identity_assertion_ref.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_evidence",
                reason: "typed actor identity must not fabricate voice identity evidence",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InputTranscriptEvidenceRefs {
    pub committed_text_hash: Option<String>,
    pub transcript_confidence_bp: Option<u16>,
    pub ph1c_status: TranscriptEvidenceStatus,
    pub rejected_reason_ref: Option<String>,
    pub wake_boundary_ref: Option<String>,
    pub audio_capture_ref: Option<String>,
    pub input_language: Option<String>,
    pub input_surface: Option<String>,
    pub unsent_draft_exclusion_ref: Option<String>,
    pub accidental_keystroke_rejection_ref: Option<String>,
}

impl InputTranscriptEvidenceRefs {
    pub fn none() -> Self {
        Self {
            committed_text_hash: None,
            transcript_confidence_bp: None,
            ph1c_status: TranscriptEvidenceStatus::NotApplicable,
            rejected_reason_ref: None,
            wake_boundary_ref: None,
            audio_capture_ref: None,
            input_language: None,
            input_surface: None,
            unsent_draft_exclusion_ref: None,
            accidental_keystroke_rejection_ref: None,
        }
    }

    pub fn voice_accepted(text_hash: String) -> Self {
        let mut refs = Self::none();
        refs.committed_text_hash = Some(text_hash);
        refs.ph1c_status = TranscriptEvidenceStatus::Accepted;
        refs.input_surface = Some("Desktop".to_string());
        refs
    }

    pub fn typed_committed(text_hash: String) -> Self {
        let mut refs = Self::none();
        refs.committed_text_hash = Some(text_hash);
        refs.ph1c_status = TranscriptEvidenceStatus::TypedCommitted;
        refs.input_surface = Some("Desktop".to_string());
        refs
    }
}

impl Validate for InputTranscriptEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_history_text(
            "input_evidence.committed_text_hash",
            &self.committed_text_hash,
            PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS,
        )?;
        validate_optional_confidence_bp(
            "input_evidence.transcript_confidence_bp",
            self.transcript_confidence_bp,
        )?;
        validate_optional_history_text(
            "input_evidence.rejected_reason_ref",
            &self.rejected_reason_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.wake_boundary_ref",
            &self.wake_boundary_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.audio_capture_ref",
            &self.audio_capture_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.input_language",
            &self.input_language,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.input_surface",
            &self.input_surface,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.unsent_draft_exclusion_ref",
            &self.unsent_draft_exclusion_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "input_evidence.accidental_keystroke_rejection_ref",
            &self.accidental_keystroke_rejection_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        if matches!(
            self.ph1c_status,
            TranscriptEvidenceStatus::Rejected
                | TranscriptEvidenceStatus::NoiseRejected
                | TranscriptEvidenceStatus::SelfEchoRejected
        ) && self.rejected_reason_ref.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "input_evidence.rejected_reason_ref",
                reason: "required for rejected transcript evidence",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResponseSpokenEvidenceRefs {
    pub final_response_text_hash: Option<String>,
    pub approved_tts_text_hash: Option<String>,
    pub visible_response_hash: Option<String>,
    pub spoken_text_hash: Option<String>,
    pub tts_provider: Option<String>,
    pub tts_status: TtsEvidenceStatus,
    pub audio_generation_ref: Option<String>,
    pub playback_started_ref: Option<String>,
    pub playback_ended_ref: Option<String>,
    pub playback_failure_ref: Option<String>,
    pub thinking_duration_ref: Option<String>,
    pub rearm_ref: Option<String>,
    pub spoken_matches_final_answer: Option<bool>,
}

impl ResponseSpokenEvidenceRefs {
    pub fn none() -> Self {
        Self {
            final_response_text_hash: None,
            approved_tts_text_hash: None,
            visible_response_hash: None,
            spoken_text_hash: None,
            tts_provider: None,
            tts_status: TtsEvidenceStatus::NotRequested,
            audio_generation_ref: None,
            playback_started_ref: None,
            playback_ended_ref: None,
            playback_failure_ref: None,
            thinking_duration_ref: None,
            rearm_ref: None,
            spoken_matches_final_answer: None,
        }
    }

    pub fn selene_text(text_hash: String) -> Self {
        let mut refs = Self::none();
        refs.final_response_text_hash = Some(text_hash.clone());
        refs.visible_response_hash = Some(text_hash);
        refs
    }
}

impl Validate for ResponseSpokenEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_history_text(
            "response_evidence.final_response_text_hash",
            &self.final_response_text_hash,
            PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.approved_tts_text_hash",
            &self.approved_tts_text_hash,
            PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.visible_response_hash",
            &self.visible_response_hash,
            PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.spoken_text_hash",
            &self.spoken_text_hash,
            PH1F_INTERNAL_HISTORY_TEXT_HASH_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.tts_provider",
            &self.tts_provider,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.audio_generation_ref",
            &self.audio_generation_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.playback_started_ref",
            &self.playback_started_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.playback_ended_ref",
            &self.playback_ended_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.playback_failure_ref",
            &self.playback_failure_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.thinking_duration_ref",
            &self.thinking_duration_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "response_evidence.rearm_ref",
            &self.rearm_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        if matches!(self.tts_status, TtsEvidenceStatus::FailedClosed)
            && self.playback_failure_ref.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "response_evidence.playback_failure_ref",
                reason: "required when TTS failed closed",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LiveContextEvidenceRefs {
    pub active_context_packet_ref: Option<String>,
    pub human_conversation_directive_ref: Option<String>,
    pub active_topic_ref: Option<String>,
    pub active_intent_ref: Option<String>,
    pub continuation_ref: Option<String>,
    pub protected_risk_ref: Option<String>,
}

impl LiveContextEvidenceRefs {
    pub fn none() -> Self {
        Self {
            active_context_packet_ref: None,
            human_conversation_directive_ref: None,
            active_topic_ref: None,
            active_intent_ref: None,
            continuation_ref: None,
            protected_risk_ref: None,
        }
    }
}

impl Validate for LiveContextEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_history_text(
            "ph1x_evidence.active_context_packet_ref",
            &self.active_context_packet_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1x_evidence.human_conversation_directive_ref",
            &self.human_conversation_directive_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1x_evidence.active_topic_ref",
            &self.active_topic_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1x_evidence.active_intent_ref",
            &self.active_intent_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1x_evidence.continuation_ref",
            &self.continuation_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1x_evidence.protected_risk_ref",
            &self.protected_risk_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MemoryEvidenceRefs {
    pub memory_evidence_packet_ref: Option<String>,
    pub memory_recall_request_ref: Option<String>,
    pub fresh_memory_handoff_ref: Option<String>,
    pub memory_continuation_decision_ref: Option<String>,
    pub memory_no_match_ref: Option<String>,
    pub memory_proposal_ref: Option<String>,
    pub memory_write_ref: Option<String>,
    pub memory_reject_ref: Option<String>,
    pub memory_candidate_status: MemoryCandidateStatus,
}

impl MemoryEvidenceRefs {
    pub fn none() -> Self {
        Self {
            memory_evidence_packet_ref: None,
            memory_recall_request_ref: None,
            fresh_memory_handoff_ref: None,
            memory_continuation_decision_ref: None,
            memory_no_match_ref: None,
            memory_proposal_ref: None,
            memory_write_ref: None,
            memory_reject_ref: None,
            memory_candidate_status: MemoryCandidateStatus::NotApplicable,
        }
    }
}

impl Validate for MemoryEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_optional_history_text(
            "ph1m_evidence.memory_evidence_packet_ref",
            &self.memory_evidence_packet_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_recall_request_ref",
            &self.memory_recall_request_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.fresh_memory_handoff_ref",
            &self.fresh_memory_handoff_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_continuation_decision_ref",
            &self.memory_continuation_decision_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_no_match_ref",
            &self.memory_no_match_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_proposal_ref",
            &self.memory_proposal_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_write_ref",
            &self.memory_write_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        validate_optional_history_text(
            "ph1m_evidence.memory_reject_ref",
            &self.memory_reject_ref,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InternalHistoryEvidenceRefs {
    pub tool_provider_refs: Vec<String>,
    pub source_refs: Vec<String>,
    pub presentation_refs: Vec<String>,
    pub multimodal_refs: Vec<String>,
    pub correction_refs: Vec<String>,
    pub decision_task_refs: Vec<String>,
    pub privacy_retention_refs: Vec<String>,
    pub protected_execution_refs: Vec<String>,
    pub timing_refs: Vec<String>,
    pub device_surface_provenance_refs: Vec<String>,
    pub audit_refs: Vec<String>,
    pub replay_integrity_refs: Vec<String>,
}

impl InternalHistoryEvidenceRefs {
    pub fn none() -> Self {
        Self {
            tool_provider_refs: Vec::new(),
            source_refs: Vec::new(),
            presentation_refs: Vec::new(),
            multimodal_refs: Vec::new(),
            correction_refs: Vec::new(),
            decision_task_refs: Vec::new(),
            privacy_retention_refs: Vec::new(),
            protected_execution_refs: Vec::new(),
            timing_refs: Vec::new(),
            device_surface_provenance_refs: Vec::new(),
            audit_refs: Vec::new(),
            replay_integrity_refs: Vec::new(),
        }
    }
}

impl Validate for InternalHistoryEvidenceRefs {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_history_ref_list("stage7_refs.tool_provider_refs", &self.tool_provider_refs)?;
        validate_history_ref_list("stage7_refs.source_refs", &self.source_refs)?;
        validate_history_ref_list("stage7_refs.presentation_refs", &self.presentation_refs)?;
        validate_history_ref_list("stage7_refs.multimodal_refs", &self.multimodal_refs)?;
        validate_history_ref_list("stage7_refs.correction_refs", &self.correction_refs)?;
        validate_history_ref_list("stage7_refs.decision_task_refs", &self.decision_task_refs)?;
        validate_history_ref_list(
            "stage7_refs.privacy_retention_refs",
            &self.privacy_retention_refs,
        )?;
        validate_history_ref_list(
            "stage7_refs.protected_execution_refs",
            &self.protected_execution_refs,
        )?;
        validate_history_ref_list("stage7_refs.timing_refs", &self.timing_refs)?;
        validate_history_ref_list(
            "stage7_refs.device_surface_provenance_refs",
            &self.device_surface_provenance_refs,
        )?;
        validate_history_ref_list("stage7_refs.audit_refs", &self.audit_refs)?;
        validate_history_ref_list(
            "stage7_refs.replay_integrity_refs",
            &self.replay_integrity_refs,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InternalHistoryEvidenceInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub event_kind: InternalHistoryEventKind,
    pub conversation_turn_id: Option<ConversationTurnId>,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub thread_key: Option<String>,
    pub role: Option<ConversationRole>,
    pub source: Option<ConversationSource>,
    pub modality: InternalHistoryModality,
    pub speaker: SpeakerEvidenceRefs,
    pub input: InputTranscriptEvidenceRefs,
    pub response: ResponseSpokenEvidenceRefs,
    pub ph1x: LiveContextEvidenceRefs,
    pub ph1m: MemoryEvidenceRefs,
    pub refs: InternalHistoryEvidenceRefs,
    pub idempotency_key: Option<String>,
}

impl InternalHistoryEvidenceInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        event_kind: InternalHistoryEventKind,
        conversation_turn_id: Option<ConversationTurnId>,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        thread_key: Option<String>,
        role: Option<ConversationRole>,
        source: Option<ConversationSource>,
        modality: InternalHistoryModality,
        speaker: SpeakerEvidenceRefs,
        input: InputTranscriptEvidenceRefs,
        response: ResponseSpokenEvidenceRefs,
        ph1x: LiveContextEvidenceRefs,
        ph1m: MemoryEvidenceRefs,
        refs: InternalHistoryEvidenceRefs,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            created_at,
            event_kind,
            conversation_turn_id,
            correlation_id,
            turn_id,
            session_id,
            thread_key,
            role,
            source,
            modality,
            speaker,
            input,
            response,
            ph1x,
            ph1m,
            refs,
            idempotency_key,
        };
        input.validate()?;
        Ok(input)
    }

    pub fn from_conversation_turn_record(
        record: &ConversationTurnRecord,
    ) -> Result<Self, ContractViolation> {
        let modality = match record.source {
            ConversationSource::VoiceTranscript => InternalHistoryModality::Voice,
            ConversationSource::TypedText => InternalHistoryModality::Typed,
            ConversationSource::SeleneOutput => InternalHistoryModality::System,
            ConversationSource::Tombstone => InternalHistoryModality::System,
        };
        let speaker = match record.source {
            ConversationSource::TypedText => {
                SpeakerEvidenceRefs::typed_actor(record.user_id.clone(), record.device_id.clone())
            }
            ConversationSource::VoiceTranscript => {
                SpeakerEvidenceRefs::voice_unknown(record.user_id.clone(), record.device_id.clone())
            }
            ConversationSource::SeleneOutput | ConversationSource::Tombstone => {
                SpeakerEvidenceRefs::none()
            }
        };
        let input_refs = match record.source {
            ConversationSource::VoiceTranscript => {
                InputTranscriptEvidenceRefs::voice_accepted(record.text_hash.clone())
            }
            ConversationSource::TypedText => {
                InputTranscriptEvidenceRefs::typed_committed(record.text_hash.clone())
            }
            ConversationSource::SeleneOutput | ConversationSource::Tombstone => {
                InputTranscriptEvidenceRefs::none()
            }
        };
        let response_refs = match record.source {
            ConversationSource::SeleneOutput => {
                ResponseSpokenEvidenceRefs::selene_text(record.text_hash.clone())
            }
            ConversationSource::VoiceTranscript
            | ConversationSource::TypedText
            | ConversationSource::Tombstone => ResponseSpokenEvidenceRefs::none(),
        };
        let mut refs = InternalHistoryEvidenceRefs::none();
        refs.replay_integrity_refs.push(format!(
            "conversation_turn:{}:text_hash:{}",
            record.conversation_turn_id.0, record.text_hash
        ));
        Self::v1(
            record.created_at,
            InternalHistoryEventKind::CommittedTurn,
            Some(record.conversation_turn_id),
            record.correlation_id,
            record.turn_id,
            record.session_id,
            None,
            Some(record.role),
            Some(record.source),
            modality,
            speaker,
            input_refs,
            response_refs,
            LiveContextEvidenceRefs::none(),
            MemoryEvidenceRefs::none(),
            refs,
            Some(format!(
                "stage7_internal_history:conversation_turn:{}",
                record.conversation_turn_id.0
            )),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn transcript_reject_v1(
        created_at: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: Option<DeviceId>,
        transcript_hash: Option<String>,
        rejected_reason_ref: String,
        audit_refs: Vec<String>,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let mut input_refs = InputTranscriptEvidenceRefs::none();
        input_refs.committed_text_hash = transcript_hash;
        input_refs.ph1c_status = TranscriptEvidenceStatus::Rejected;
        input_refs.rejected_reason_ref = Some(rejected_reason_ref);
        let mut memory_refs = MemoryEvidenceRefs::none();
        memory_refs.memory_candidate_status = MemoryCandidateStatus::BlockedRejectedTranscript;
        let mut refs = InternalHistoryEvidenceRefs::none();
        refs.audit_refs = audit_refs;
        Self::v1(
            created_at,
            InternalHistoryEventKind::RejectedInput,
            None,
            correlation_id,
            turn_id,
            session_id,
            None,
            Some(ConversationRole::User),
            Some(ConversationSource::VoiceTranscript),
            InternalHistoryModality::Voice,
            SpeakerEvidenceRefs::voice_unknown(user_id, device_id),
            input_refs,
            ResponseSpokenEvidenceRefs::none(),
            LiveContextEvidenceRefs::none(),
            memory_refs,
            refs,
            Some(idempotency_key),
        )
    }
}

impl Validate for InternalHistoryEvidenceInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1F_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history.schema_version",
                reason: "must match PH1F_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history.created_at",
                reason: "must be > 0",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if let Some(conversation_turn_id) = self.conversation_turn_id {
            conversation_turn_id.validate()?;
        }
        if let Some(session_id) = self.session_id {
            if session_id.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "internal_history.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        validate_optional_history_text(
            "internal_history.thread_key",
            &self.thread_key,
            PH1F_INTERNAL_HISTORY_MAX_LABEL_CHARS,
        )?;
        self.speaker.validate()?;
        self.input.validate()?;
        self.response.validate()?;
        self.ph1x.validate()?;
        self.ph1m.validate()?;
        self.refs.validate()?;
        validate_optional_history_text(
            "internal_history.idempotency_key",
            &self.idempotency_key,
            PH1F_INTERNAL_HISTORY_REF_MAX_CHARS,
        )?;
        if self.event_kind == InternalHistoryEventKind::CommittedTurn
            && self.conversation_turn_id.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history.conversation_turn_id",
                reason: "required for committed turn evidence",
            });
        }
        if self.modality == InternalHistoryModality::Typed
            && self.speaker.voice_identity_assertion_ref.is_some()
        {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history.speaker.voice_identity_assertion_ref",
                reason: "typed turns must not carry fabricated voice evidence",
            });
        }
        if matches!(
            self.input.ph1c_status,
            TranscriptEvidenceStatus::Rejected
                | TranscriptEvidenceStatus::NoiseRejected
                | TranscriptEvidenceStatus::SelfEchoRejected
        ) && matches!(
            self.ph1m.memory_candidate_status,
            MemoryCandidateStatus::Allowed
        ) {
            return Err(ContractViolation::InvalidValue {
                field: "internal_history.ph1m.memory_candidate_status",
                reason: "rejected input cannot become a memory candidate",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InternalHistoryEvidenceRecord {
    pub schema_version: SchemaVersion,
    pub internal_history_event_id: InternalHistoryEventId,
    pub created_at: MonotonicTimeNs,
    pub event_kind: InternalHistoryEventKind,
    pub conversation_turn_id: Option<ConversationTurnId>,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub thread_key: Option<String>,
    pub role: Option<ConversationRole>,
    pub source: Option<ConversationSource>,
    pub modality: InternalHistoryModality,
    pub speaker: SpeakerEvidenceRefs,
    pub input: InputTranscriptEvidenceRefs,
    pub response: ResponseSpokenEvidenceRefs,
    pub ph1x: LiveContextEvidenceRefs,
    pub ph1m: MemoryEvidenceRefs,
    pub refs: InternalHistoryEvidenceRefs,
    pub idempotency_key: Option<String>,
}

impl InternalHistoryEvidenceRecord {
    pub fn from_input_v1(
        internal_history_event_id: InternalHistoryEventId,
        input: InternalHistoryEvidenceInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        internal_history_event_id.validate()?;
        let record = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            internal_history_event_id,
            created_at: input.created_at,
            event_kind: input.event_kind,
            conversation_turn_id: input.conversation_turn_id,
            correlation_id: input.correlation_id,
            turn_id: input.turn_id,
            session_id: input.session_id,
            thread_key: input.thread_key,
            role: input.role,
            source: input.source,
            modality: input.modality,
            speaker: input.speaker,
            input: input.input,
            response: input.response,
            ph1x: input.ph1x,
            ph1m: input.ph1m,
            refs: input.refs,
            idempotency_key: input.idempotency_key,
        };
        record.validate()?;
        Ok(record)
    }
}

impl Validate for InternalHistoryEvidenceRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.internal_history_event_id.validate()?;
        InternalHistoryEvidenceInput {
            schema_version: self.schema_version,
            created_at: self.created_at,
            event_kind: self.event_kind,
            conversation_turn_id: self.conversation_turn_id,
            correlation_id: self.correlation_id,
            turn_id: self.turn_id,
            session_id: self.session_id,
            thread_key: self.thread_key.clone(),
            role: self.role,
            source: self.source,
            modality: self.modality,
            speaker: self.speaker.clone(),
            input: self.input.clone(),
            response: self.response.clone(),
            ph1x: self.ph1x.clone(),
            ph1m: self.ph1m.clone(),
            refs: self.refs.clone(),
            idempotency_key: self.idempotency_key.clone(),
        }
        .validate()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationTurnInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub device_id: Option<DeviceId>,
    pub role: ConversationRole,
    pub source: ConversationSource,
    pub text: String,
    pub text_hash: String,
    pub privacy_scope: PrivacyScope,
    /// Optional key to dedupe storage writes on retries (PH1.F invariant).
    pub idempotency_key: Option<String>,
    /// Required when source=Tombstone: references the original conversation_turn_id.
    pub tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
    /// Required when source=Tombstone.
    pub tombstone_reason_code: Option<ReasonCodeId>,
}

impl ConversationTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        session_id: Option<SessionId>,
        user_id: UserId,
        device_id: Option<DeviceId>,
        role: ConversationRole,
        source: ConversationSource,
        text: String,
        text_hash: String,
        privacy_scope: PrivacyScope,
        idempotency_key: Option<String>,
        tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
        tombstone_reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let t = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            created_at,
            correlation_id,
            turn_id,
            session_id,
            user_id,
            device_id,
            role,
            source,
            text,
            text_hash,
            privacy_scope,
            idempotency_key,
            tombstone_of_conversation_turn_id,
            tombstone_reason_code,
        };
        t.validate()?;
        Ok(t)
    }
}

impl Validate for ConversationTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1F_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.schema_version",
                reason: "must match PH1F_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.created_at",
                reason: "must be > 0",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if let Some(s) = self.session_id {
            if s.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if self.user_id.as_str().trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.user_id",
                reason: "must not be empty",
            });
        }
        if let Some(d) = &self.device_id {
            d.validate()?;
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 8192 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                reason: "must be <= 8192 chars",
            });
        }
        if self.text_hash.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text_hash",
                reason: "must not be empty",
            });
        }
        if self.text_hash.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text_hash",
                reason: "must be <= 128 chars",
            });
        }
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "conversation_turn_input.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }

        match self.source {
            ConversationSource::VoiceTranscript | ConversationSource::TypedText => {
                if self.role != ConversationRole::User {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be USER for voice_transcript/typed_text",
                    });
                }
                if self.tombstone_of_conversation_turn_id.is_some()
                    || self.tombstone_reason_code.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input",
                        reason: "tombstone fields must be None unless source=Tombstone",
                    });
                }
            }
            ConversationSource::SeleneOutput => {
                if self.role != ConversationRole::Selene {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be SELENE for selene_output",
                    });
                }
                if self.tombstone_of_conversation_turn_id.is_some()
                    || self.tombstone_reason_code.is_some()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input",
                        reason: "tombstone fields must be None unless source=Tombstone",
                    });
                }
            }
            ConversationSource::Tombstone => {
                if self.role != ConversationRole::Selene {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.role",
                        reason: "must be SELENE for tombstone",
                    });
                }
                if self.text != "[REDACTED]" {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.text",
                        reason: "tombstone text must be the fixed placeholder [REDACTED]",
                    });
                }
                let Some(id) = self.tombstone_of_conversation_turn_id else {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_of_conversation_turn_id",
                        reason: "required for tombstone",
                    });
                };
                id.validate()?;
                let Some(rc) = self.tombstone_reason_code else {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_reason_code",
                        reason: "required for tombstone",
                    });
                };
                if rc.0 == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "conversation_turn_input.tombstone_reason_code",
                        reason: "must be > 0",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ConversationTurnRecord {
    pub schema_version: SchemaVersion,
    pub conversation_turn_id: ConversationTurnId,
    pub created_at: MonotonicTimeNs,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub session_id: Option<SessionId>,
    pub user_id: UserId,
    pub device_id: Option<DeviceId>,
    pub role: ConversationRole,
    pub source: ConversationSource,
    pub text: String,
    pub text_hash: String,
    pub privacy_scope: PrivacyScope,
    pub idempotency_key: Option<String>,
    pub tombstone_of_conversation_turn_id: Option<ConversationTurnId>,
    pub tombstone_reason_code: Option<ReasonCodeId>,
}

impl ConversationTurnRecord {
    pub fn from_input_v1(
        conversation_turn_id: ConversationTurnId,
        input: ConversationTurnInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        conversation_turn_id.validate()?;
        let r = Self {
            schema_version: PH1F_CONTRACT_VERSION,
            conversation_turn_id,
            created_at: input.created_at,
            correlation_id: input.correlation_id,
            turn_id: input.turn_id,
            session_id: input.session_id,
            user_id: input.user_id,
            device_id: input.device_id,
            role: input.role,
            source: input.source,
            text: input.text,
            text_hash: input.text_hash,
            privacy_scope: input.privacy_scope,
            idempotency_key: input.idempotency_key,
            tombstone_of_conversation_turn_id: input.tombstone_of_conversation_turn_id,
            tombstone_reason_code: input.tombstone_reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for ConversationTurnRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1F_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "conversation_turn_record.schema_version",
                reason: "must match PH1F_CONTRACT_VERSION",
            });
        }
        self.conversation_turn_id.validate()?;
        // Reuse input validation for the rest of the rules.
        ConversationTurnInput {
            schema_version: self.schema_version,
            created_at: self.created_at,
            correlation_id: self.correlation_id,
            turn_id: self.turn_id,
            session_id: self.session_id,
            user_id: self.user_id.clone(),
            device_id: self.device_id.clone(),
            role: self.role,
            source: self.source,
            text: self.text.clone(),
            text_hash: self.text_hash.clone(),
            privacy_scope: self.privacy_scope,
            idempotency_key: self.idempotency_key.clone(),
            tombstone_of_conversation_turn_id: self.tombstone_of_conversation_turn_id,
            tombstone_reason_code: self.tombstone_reason_code,
        }
        .validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> ConversationTurnInput {
        ConversationTurnInput::v1(
            MonotonicTimeNs(1),
            CorrelationId(9601),
            TurnId(1),
            None,
            UserId::new("f_contract_user_1").unwrap(),
            None,
            ConversationRole::User,
            ConversationSource::TypedText,
            "hello".to_string(),
            "hash_1".to_string(),
            PrivacyScope::PublicChat,
            None,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_f_contract_01_typed_text_requires_user_role() {
        let mut input = base_input();
        input.role = ConversationRole::Selene;
        assert!(matches!(
            input.validate(),
            Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.role",
                ..
            })
        ));
    }

    #[test]
    fn at_f_contract_02_tombstone_requires_placeholder_and_reason() {
        let mut input = base_input();
        input.role = ConversationRole::Selene;
        input.source = ConversationSource::Tombstone;
        input.text = "not redacted".to_string();
        input.tombstone_of_conversation_turn_id = Some(ConversationTurnId(10));
        input.tombstone_reason_code = Some(ReasonCodeId(1));

        assert!(matches!(
            input.validate(),
            Err(ContractViolation::InvalidValue {
                field: "conversation_turn_input.text",
                ..
            })
        ));
    }

    #[test]
    fn at_f_contract_03_record_from_input_roundtrip_is_valid() {
        let input = base_input();
        let record = ConversationTurnRecord::from_input_v1(ConversationTurnId(1), input).unwrap();
        assert_eq!(record.conversation_turn_id, ConversationTurnId(1));
        assert_eq!(record.source, ConversationSource::TypedText);
        assert!(record.validate().is_ok());
    }

    #[test]
    fn at_f_stage7_01_committed_typed_turn_has_actor_identity_but_no_voice_evidence() {
        let input = base_input();
        let record = ConversationTurnRecord::from_input_v1(ConversationTurnId(7), input).unwrap();
        let evidence =
            InternalHistoryEvidenceInput::from_conversation_turn_record(&record).unwrap();

        assert_eq!(evidence.event_kind, InternalHistoryEventKind::CommittedTurn);
        assert_eq!(evidence.modality, InternalHistoryModality::Typed);
        assert_eq!(
            evidence.input.ph1c_status,
            TranscriptEvidenceStatus::TypedCommitted
        );
        assert_eq!(
            evidence.speaker.identity_posture,
            SpeakerIdentityPosture::Known
        );
        assert!(evidence.speaker.typed_actor_identity_ref.is_some());
        assert!(evidence.speaker.voice_identity_assertion_ref.is_none());
        assert!(evidence.validate().is_ok());
    }

    #[test]
    fn at_f_stage7_02_committed_voice_turn_can_carry_nullable_unknown_speaker_evidence() {
        let voice_input = ConversationTurnInput::v1(
            MonotonicTimeNs(2),
            CorrelationId(9602),
            TurnId(2),
            Some(SessionId(3)),
            UserId::new("voice_user_1").unwrap(),
            Some(DeviceId::new("desktop_1").unwrap()),
            ConversationRole::User,
            ConversationSource::VoiceTranscript,
            "what time is it in Sydney".to_string(),
            "hash_voice_sydney".to_string(),
            PrivacyScope::PublicChat,
            Some("voice_turn_1".to_string()),
            None,
            None,
        )
        .unwrap();
        let record =
            ConversationTurnRecord::from_input_v1(ConversationTurnId(8), voice_input).unwrap();
        let evidence =
            InternalHistoryEvidenceInput::from_conversation_turn_record(&record).unwrap();

        assert_eq!(evidence.modality, InternalHistoryModality::Voice);
        assert_eq!(
            evidence.input.ph1c_status,
            TranscriptEvidenceStatus::Accepted
        );
        assert_eq!(
            evidence.speaker.identity_posture,
            SpeakerIdentityPosture::Unknown
        );
        assert!(evidence.speaker.voice_identity_assertion_ref.is_none());
    }

    #[test]
    fn at_f_stage7_03_rejected_transcript_cannot_be_memory_candidate() {
        let evidence = InternalHistoryEvidenceInput::transcript_reject_v1(
            MonotonicTimeNs(3),
            CorrelationId(9603),
            TurnId(3),
            Some(SessionId(4)),
            UserId::new("voice_user_2").unwrap(),
            Some(DeviceId::new("desktop_2").unwrap()),
            Some("hash_rejected_noise".to_string()),
            "reason_code:1124073473".to_string(),
            vec!["audit_event:11".to_string()],
            "reject_noise_1".to_string(),
        )
        .unwrap();

        assert_eq!(evidence.event_kind, InternalHistoryEventKind::RejectedInput);
        assert_eq!(
            evidence.input.ph1c_status,
            TranscriptEvidenceStatus::Rejected
        );
        assert_eq!(
            evidence.ph1m.memory_candidate_status,
            MemoryCandidateStatus::BlockedRejectedTranscript
        );

        let mut bad = evidence;
        bad.ph1m.memory_candidate_status = MemoryCandidateStatus::Allowed;
        assert!(matches!(
            bad.validate(),
            Err(ContractViolation::InvalidValue {
                field: "internal_history.ph1m.memory_candidate_status",
                ..
            })
        ));
    }

    #[test]
    fn at_f_stage7_04_refs_cover_ph1x_ph1m_tools_presentation_and_integrity_hooks() {
        let mut ph1x = LiveContextEvidenceRefs::none();
        ph1x.active_context_packet_ref = Some("ph1x_active_context_packet:turn_10".to_string());
        ph1x.human_conversation_directive_ref =
            Some("ph1x_human_conversation_directive:turn_10".to_string());

        let mut ph1m = MemoryEvidenceRefs::none();
        ph1m.memory_evidence_packet_ref = Some("ph1m_memory_evidence:fresh_1".to_string());
        ph1m.memory_recall_request_ref = Some("ph1m_recall_request:req_1".to_string());
        ph1m.fresh_memory_handoff_ref = Some("ph1m_fresh_handoff:handoff_1".to_string());
        ph1m.memory_continuation_decision_ref = Some("ph1m_continuation:continue_1".to_string());

        let mut refs = InternalHistoryEvidenceRefs::none();
        refs.tool_provider_refs.push("ph1e_tool:time".to_string());
        refs.source_refs.push("source_ref:world_time".to_string());
        refs.presentation_refs
            .push("ph1write:plain_answer".to_string());
        refs.multimodal_refs
            .push("attachment_ref:image_nullable".to_string());
        refs.correction_refs.push("correction_ref:none".to_string());
        refs.decision_task_refs
            .push("decision_candidate:none".to_string());
        refs.privacy_retention_refs
            .push("retention:public_chat".to_string());
        refs.protected_execution_refs
            .push("protected_fail_closed:none".to_string());
        refs.timing_refs.push("latency:turn_10".to_string());
        refs.device_surface_provenance_refs
            .push("surface:desktop".to_string());
        refs.audit_refs.push("audit_event:22".to_string());
        refs.replay_integrity_refs
            .push("envelope_hash:abcdef".to_string());

        let evidence = InternalHistoryEvidenceInput::v1(
            MonotonicTimeNs(10),
            InternalHistoryEventKind::ToolEvidence,
            None,
            CorrelationId(9610),
            TurnId(10),
            None,
            Some("thread_main".to_string()),
            None,
            None,
            InternalHistoryModality::System,
            SpeakerEvidenceRefs::none(),
            InputTranscriptEvidenceRefs::none(),
            ResponseSpokenEvidenceRefs::none(),
            ph1x,
            ph1m,
            refs,
            Some("stage7_ref_hooks_1".to_string()),
        )
        .unwrap();

        assert!(evidence.validate().is_ok());
    }
}
