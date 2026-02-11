#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, DeviceId, TurnId};
use crate::ph1k::{AudioDeviceId, AudioStreamKind, AudioStreamRef, VadEvent};
use crate::ph1l::SessionSnapshot;
use crate::ph1w::WakeDecision;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1_VOICE_ID_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpeakerId(String);

impl SpeakerId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_id",
                reason: "must be <= 128 chars",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserId(String);

impl UserId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "user_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "user_id",
                reason: "must be <= 128 chars",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentityConfidence {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceTrustLevel {
    Trusted,
    Untrusted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpeakerLabel(pub u8);

impl SpeakerLabel {
    pub const fn speaker_a() -> Self {
        Self(0)
    }

    pub const fn speaker_b() -> Self {
        Self(1)
    }
}

impl Validate for SpeakerLabel {
    fn validate(&self) -> Result<(), ContractViolation> {
        // Deterministic, practical upper bound.
        if self.0 > 25 {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_label",
                reason: "must be <= 25",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiarizationSegment {
    pub schema_version: SchemaVersion,
    pub t_start: MonotonicTimeNs,
    pub t_end: MonotonicTimeNs,
    /// Stable temporary label within a session (e.g., SPEAKER_A, SPEAKER_B).
    /// If identity is unknown, this may be omitted.
    pub speaker_label: Option<SpeakerLabel>,
}

impl DiarizationSegment {
    pub fn v1(
        t_start: MonotonicTimeNs,
        t_end: MonotonicTimeNs,
        speaker_label: Option<SpeakerLabel>,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1_VOICE_ID_CONTRACT_VERSION,
            t_start,
            t_end,
            speaker_label,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for DiarizationSegment {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.t_end.0 < self.t_start.0 {
            return Err(ContractViolation::InvalidValue {
                field: "diarization_segment.t_end",
                reason: "must be >= t_start",
            });
        }
        if let Some(l) = self.speaker_label {
            l.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1VoiceIdRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub processed_audio_stream_ref: AudioStreamRef,
    pub vad_events: Vec<VadEvent>,
    pub device_id: AudioDeviceId,
    pub device_trust_level: DeviceTrustLevel,
    /// Optional. When present, enables deterministic "foreign device" claim behavior.
    pub device_owner_user_id: Option<UserId>,
    pub session_state_ref: SessionSnapshot,
    pub wake_event: Option<WakeDecision>,
    /// Echo-safety hint derived from TTS playback markers/events.
    pub tts_playback_active: bool,
}

impl Ph1VoiceIdRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        now: MonotonicTimeNs,
        processed_audio_stream_ref: AudioStreamRef,
        vad_events: Vec<VadEvent>,
        device_id: AudioDeviceId,
        session_state_ref: SessionSnapshot,
        wake_event: Option<WakeDecision>,
        tts_playback_active: bool,
        device_trust_level: DeviceTrustLevel,
        device_owner_user_id: Option<UserId>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1_VOICE_ID_CONTRACT_VERSION,
            now,
            processed_audio_stream_ref,
            vad_events,
            device_id,
            device_trust_level,
            device_owner_user_id,
            session_state_ref,
            wake_event,
            tts_playback_active,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1VoiceIdRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.session_state_ref.validate()?;
        if let Some(w) = &self.wake_event {
            w.validate()?;
        }
        self.processed_audio_stream_ref.validate()?;
        if self.processed_audio_stream_ref.kind != AudioStreamKind::MicProcessed {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_request.processed_audio_stream_ref.kind",
                reason: "must be MicProcessed",
            });
        }
        if self.vad_events.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_request.vad_events",
                reason: "must be <= 256 entries",
            });
        }
        for ev in &self.vad_events {
            ev.validate()?;
            if ev.stream_id != self.processed_audio_stream_ref.stream_id {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1_voice_id_request.vad_events[].stream_id",
                    reason: "must match processed_audio_stream_ref.stream_id",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerAssertionOk {
    pub schema_version: SchemaVersion,
    pub speaker_id: SpeakerId,
    pub user_id: Option<UserId>,
    pub confidence: IdentityConfidence,
    pub diarization_segments: Vec<DiarizationSegment>,
    pub active_speaker_label: SpeakerLabel,
}

impl SpeakerAssertionOk {
    pub fn v1(
        speaker_id: SpeakerId,
        user_id: Option<UserId>,
        diarization_segments: Vec<DiarizationSegment>,
        active_speaker_label: SpeakerLabel,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1_VOICE_ID_CONTRACT_VERSION,
            speaker_id,
            user_id,
            confidence: IdentityConfidence::High,
            diarization_segments,
            active_speaker_label,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for SpeakerAssertionOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.confidence != IdentityConfidence::High {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_assertion_ok.confidence",
                reason: "must be High",
            });
        }
        self.active_speaker_label.validate()?;
        if self.diarization_segments.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_assertion_ok.diarization_segments",
                reason: "must not be empty",
            });
        }
        for seg in &self.diarization_segments {
            seg.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerAssertionUnknown {
    pub schema_version: SchemaVersion,
    pub confidence: IdentityConfidence,
    pub reason_code: ReasonCodeId,
    pub diarization_segments: Vec<DiarizationSegment>,
    /// Optional. Used when Selene recognizes a user but is blocked by reauth or device-claim policy.
    pub candidate_user_id: Option<UserId>,
    /// Optional. Echoed for claim UX (owned device flows).
    pub device_owner_user_id: Option<UserId>,
}

impl SpeakerAssertionUnknown {
    pub fn v1(
        confidence: IdentityConfidence,
        reason_code: ReasonCodeId,
        diarization_segments: Vec<DiarizationSegment>,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_candidate(confidence, reason_code, diarization_segments, None, None)
    }

    pub fn v1_with_candidate(
        confidence: IdentityConfidence,
        reason_code: ReasonCodeId,
        diarization_segments: Vec<DiarizationSegment>,
        candidate_user_id: Option<UserId>,
        device_owner_user_id: Option<UserId>,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1_VOICE_ID_CONTRACT_VERSION,
            confidence,
            reason_code,
            diarization_segments,
            candidate_user_id,
            device_owner_user_id,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for SpeakerAssertionUnknown {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.confidence == IdentityConfidence::High {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_assertion_unknown.confidence",
                reason: "must be Medium or Low",
            });
        }
        for seg in &self.diarization_segments {
            seg.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1VoiceIdResponse {
    SpeakerAssertionOk(SpeakerAssertionOk),
    SpeakerAssertionUnknown(SpeakerAssertionUnknown),
}

impl Validate for Ph1VoiceIdResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VoiceIdResponse::SpeakerAssertionOk(ok) => ok.validate(),
            Ph1VoiceIdResponse::SpeakerAssertionUnknown(u) => u.validate(),
        }
    }
}

// ---------------------------------
// PH1.VOICE.ID simulation contracts
// ---------------------------------

pub const PH1VOICEID_SIM_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

pub const VOICE_ID_ENROLL_START_DRAFT: &str = "VOICE_ID_ENROLL_START_DRAFT";
pub const VOICE_ID_ENROLL_SAMPLE_COMMIT: &str = "VOICE_ID_ENROLL_SAMPLE_COMMIT";
pub const VOICE_ID_ENROLL_COMPLETE_COMMIT: &str = "VOICE_ID_ENROLL_COMPLETE_COMMIT";
pub const VOICE_ID_ENROLL_DEFER_REMINDER_COMMIT: &str = "VOICE_ID_ENROLL_DEFER_REMINDER_COMMIT";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceIdSimulationType {
    Draft,
    Commit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceEnrollStatus {
    InProgress,
    Locked,
    Pending,
    Declined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceSampleResult {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VoiceEnrollmentSessionId(String);

impl VoiceEnrollmentSessionId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        let v = Self(id);
        v.validate()?;
        Ok(v)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for VoiceEnrollmentSessionId {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id("voice_enrollment_session_id", &self.0, 64)
    }
}

fn validate_id(field: &'static str, s: &str, max_len: usize) -> Result<(), ContractViolation> {
    if s.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if s.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too long",
        });
    }
    if !s.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollStartDraftRequest {
    pub onboarding_session_id: String,
    pub device_id: DeviceId,
    pub consent_asserted: bool,
    pub max_total_attempts: u8,
    pub max_session_enroll_time_ms: u32,
    pub lock_after_consecutive_passes: u8,
}

impl Validate for VoiceIdEnrollStartDraftRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "voice_id_enroll_start_draft_request.onboarding_session_id",
            &self.onboarding_session_id,
            64,
        )?;
        self.device_id.validate()?;
        if !self.consent_asserted {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_draft_request.consent_asserted",
                reason: "must be true",
            });
        }
        if !(5..=20).contains(&self.max_total_attempts) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_draft_request.max_total_attempts",
                reason: "must be in [5, 20]",
            });
        }
        if !(60_000..=300_000).contains(&self.max_session_enroll_time_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_draft_request.max_session_enroll_time_ms",
                reason: "must be in [60000, 300000]",
            });
        }
        if !(2..=5).contains(&self.lock_after_consecutive_passes) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_draft_request.lock_after_consecutive_passes",
                reason: "must be in [2, 5]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollSampleCommitRequest {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub audio_sample_ref: String,
    pub attempt_index: u16,
    pub sample_result: VoiceSampleResult,
    pub reason_code: Option<ReasonCodeId>,
    pub idempotency_key: String,
}

impl Validate for VoiceIdEnrollSampleCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        validate_id(
            "voice_id_enroll_sample_commit_request.audio_sample_ref",
            &self.audio_sample_ref,
            256,
        )?;
        if self.attempt_index == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_sample_commit_request.attempt_index",
                reason: "must be > 0",
            });
        }
        validate_id(
            "voice_id_enroll_sample_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        if matches!(self.sample_result, VoiceSampleResult::Fail) && self.reason_code.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_sample_commit_request.reason_code",
                reason: "required when sample_result=Fail",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollCompleteCommitRequest {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub idempotency_key: String,
}

impl Validate for VoiceIdEnrollCompleteCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        validate_id(
            "voice_id_enroll_complete_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollDeferReminderCommitRequest {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: String,
}

impl Validate for VoiceIdEnrollDeferReminderCommitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_defer_reminder_commit_request.reason_code",
                reason: "must be > 0",
            });
        }
        validate_id(
            "voice_id_enroll_defer_reminder_commit_request.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollStartResult {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub voice_enroll_status: VoiceEnrollStatus,
    pub max_total_attempts: u8,
    pub max_session_enroll_time_ms: u32,
    pub lock_after_consecutive_passes: u8,
}

impl VoiceIdEnrollStartResult {
    pub fn v1(
        voice_enrollment_session_id: VoiceEnrollmentSessionId,
        voice_enroll_status: VoiceEnrollStatus,
        max_total_attempts: u8,
        max_session_enroll_time_ms: u32,
        lock_after_consecutive_passes: u8,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            voice_enrollment_session_id,
            voice_enroll_status,
            max_total_attempts,
            max_session_enroll_time_ms,
            lock_after_consecutive_passes,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for VoiceIdEnrollStartResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        if !(5..=20).contains(&self.max_total_attempts) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_result.max_total_attempts",
                reason: "must be in [5, 20]",
            });
        }
        if !(60_000..=300_000).contains(&self.max_session_enroll_time_ms) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_result.max_session_enroll_time_ms",
                reason: "must be in [60000, 300000]",
            });
        }
        if !(2..=5).contains(&self.lock_after_consecutive_passes) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_start_result.lock_after_consecutive_passes",
                reason: "must be in [2, 5]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollSampleResult {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub sample_result: VoiceSampleResult,
    pub reason_code: Option<ReasonCodeId>,
    pub consecutive_passes: u8,
    pub voice_enroll_status: VoiceEnrollStatus,
}

impl VoiceIdEnrollSampleResult {
    pub fn v1(
        voice_enrollment_session_id: VoiceEnrollmentSessionId,
        sample_result: VoiceSampleResult,
        reason_code: Option<ReasonCodeId>,
        consecutive_passes: u8,
        voice_enroll_status: VoiceEnrollStatus,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            voice_enrollment_session_id,
            sample_result,
            reason_code,
            consecutive_passes,
            voice_enroll_status,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for VoiceIdEnrollSampleResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        if matches!(self.sample_result, VoiceSampleResult::Fail) && self.reason_code.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_sample_result.reason_code",
                reason: "required when sample_result=Fail",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollCompleteResult {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub voice_profile_id: String,
    pub voice_enroll_status: VoiceEnrollStatus,
}

impl VoiceIdEnrollCompleteResult {
    pub fn v1(
        voice_enrollment_session_id: VoiceEnrollmentSessionId,
        voice_profile_id: String,
        voice_enroll_status: VoiceEnrollStatus,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            voice_enrollment_session_id,
            voice_profile_id,
            voice_enroll_status,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for VoiceIdEnrollCompleteResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        validate_id(
            "voice_id_enroll_complete_result.voice_profile_id",
            &self.voice_profile_id,
            64,
        )?;
        if self.voice_enroll_status != VoiceEnrollStatus::Locked {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_complete_result.voice_enroll_status",
                reason: "must be Locked",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceIdEnrollDeferResult {
    pub voice_enrollment_session_id: VoiceEnrollmentSessionId,
    pub voice_enroll_status: VoiceEnrollStatus,
    pub reason_code: ReasonCodeId,
}

impl VoiceIdEnrollDeferResult {
    pub fn v1(
        voice_enrollment_session_id: VoiceEnrollmentSessionId,
        voice_enroll_status: VoiceEnrollStatus,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            voice_enrollment_session_id,
            voice_enroll_status,
            reason_code,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for VoiceIdEnrollDeferResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.voice_enrollment_session_id.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id_enroll_defer_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VoiceIdSimulationRequest {
    EnrollStartDraft(VoiceIdEnrollStartDraftRequest),
    EnrollSampleCommit(VoiceIdEnrollSampleCommitRequest),
    EnrollCompleteCommit(VoiceIdEnrollCompleteCommitRequest),
    EnrollDeferReminderCommit(VoiceIdEnrollDeferReminderCommitRequest),
}

impl Validate for VoiceIdSimulationRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            VoiceIdSimulationRequest::EnrollStartDraft(v) => v.validate(),
            VoiceIdSimulationRequest::EnrollSampleCommit(v) => v.validate(),
            VoiceIdSimulationRequest::EnrollCompleteCommit(v) => v.validate(),
            VoiceIdSimulationRequest::EnrollDeferReminderCommit(v) => v.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1VoiceIdSimRequest {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub now: MonotonicTimeNs,
    pub simulation_id: String,
    pub simulation_type: VoiceIdSimulationType,
    pub request: VoiceIdSimulationRequest,
}

impl Validate for Ph1VoiceIdSimRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VOICEID_SIM_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_request.schema_version",
                reason: "must match PH1VOICEID_SIM_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_id(
            "ph1_voice_id_sim_request.simulation_id",
            &self.simulation_id,
            128,
        )?;
        self.request.validate()?;

        let expected_simulation_id = match &self.request {
            VoiceIdSimulationRequest::EnrollStartDraft(_) => VOICE_ID_ENROLL_START_DRAFT,
            VoiceIdSimulationRequest::EnrollSampleCommit(_) => VOICE_ID_ENROLL_SAMPLE_COMMIT,
            VoiceIdSimulationRequest::EnrollCompleteCommit(_) => VOICE_ID_ENROLL_COMPLETE_COMMIT,
            VoiceIdSimulationRequest::EnrollDeferReminderCommit(_) => {
                VOICE_ID_ENROLL_DEFER_REMINDER_COMMIT
            }
        };
        if self.simulation_id != expected_simulation_id {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_request.simulation_id",
                reason: "does not match request variant",
            });
        }

        let expected_type = match self.request {
            VoiceIdSimulationRequest::EnrollStartDraft(_) => VoiceIdSimulationType::Draft,
            _ => VoiceIdSimulationType::Commit,
        };
        if self.simulation_type != expected_type {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_request.simulation_type",
                reason: "does not match request variant",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1VoiceIdSimOk {
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub enroll_start_result: Option<VoiceIdEnrollStartResult>,
    pub enroll_sample_result: Option<VoiceIdEnrollSampleResult>,
    pub enroll_complete_result: Option<VoiceIdEnrollCompleteResult>,
    pub enroll_defer_result: Option<VoiceIdEnrollDeferResult>,
}

impl Ph1VoiceIdSimOk {
    pub fn v1(
        simulation_id: String,
        reason_code: ReasonCodeId,
        enroll_start_result: Option<VoiceIdEnrollStartResult>,
        enroll_sample_result: Option<VoiceIdEnrollSampleResult>,
        enroll_complete_result: Option<VoiceIdEnrollCompleteResult>,
        enroll_defer_result: Option<VoiceIdEnrollDeferResult>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            simulation_id,
            reason_code,
            enroll_start_result,
            enroll_sample_result,
            enroll_complete_result,
            enroll_defer_result,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for Ph1VoiceIdSimOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "ph1_voice_id_sim_ok.simulation_id",
            &self.simulation_id,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_ok.reason_code",
                reason: "must be > 0",
            });
        }

        let mut count = 0_u8;
        if self.enroll_start_result.is_some() {
            count += 1;
        }
        if self.enroll_sample_result.is_some() {
            count += 1;
        }
        if self.enroll_complete_result.is_some() {
            count += 1;
        }
        if self.enroll_defer_result.is_some() {
            count += 1;
        }
        if count != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_ok",
                reason: "must contain exactly one result payload",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1VoiceIdSimRefuse {
    pub simulation_id: String,
    pub reason_code: ReasonCodeId,
    pub detail: String,
}

impl Validate for Ph1VoiceIdSimRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_id(
            "ph1_voice_id_sim_refuse.simulation_id",
            &self.simulation_id,
            128,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_refuse.reason_code",
                reason: "must be > 0",
            });
        }
        if self.detail.trim().is_empty() || self.detail.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_voice_id_sim_refuse.detail",
                reason: "must be non-empty and <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1VoiceIdSimResponse {
    Ok(Ph1VoiceIdSimOk),
    Refuse(Ph1VoiceIdSimRefuse),
}

impl Validate for Ph1VoiceIdSimResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VoiceIdSimResponse::Ok(v) => v.validate(),
            Ph1VoiceIdSimResponse::Refuse(v) => v.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1j::{CorrelationId, DeviceId, TurnId};
    use crate::ph1k::{
        AudioFormat, AudioStreamId, AudioStreamRef, ChannelCount, Confidence, SampleFormat,
        SampleRateHz, SpeechLikeness, VadEvent,
    };
    use crate::ph1l::{NextAllowedActions, SessionId, SessionSnapshot};
    use crate::SessionState;

    fn processed_stream() -> AudioStreamRef {
        AudioStreamRef::v1(
            AudioStreamId(1),
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
        )
    }

    fn snapshot_active() -> SessionSnapshot {
        SessionSnapshot {
            schema_version: SchemaVersion(1),
            session_state: SessionState::Active,
            session_id: Some(SessionId(1)),
            next_allowed_actions: NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            },
        }
    }

    #[test]
    fn request_rejects_non_processed_stream_kind() {
        let mut s = processed_stream();
        s.kind = AudioStreamKind::MicRaw;
        let r = Ph1VoiceIdRequest::v1(
            MonotonicTimeNs(0),
            s,
            vec![],
            AudioDeviceId::new("mic").unwrap(),
            snapshot_active(),
            None,
            false,
            DeviceTrustLevel::Trusted,
            None,
        );
        assert!(r.is_err());
    }

    #[test]
    fn ok_requires_high_confidence() {
        let mut ok = SpeakerAssertionOk::v1(
            SpeakerId::new("spk").unwrap(),
            Some(UserId::new("user").unwrap()),
            vec![DiarizationSegment::v1(
                MonotonicTimeNs(0),
                MonotonicTimeNs(1),
                Some(SpeakerLabel::speaker_a()),
            )
            .unwrap()],
            SpeakerLabel::speaker_a(),
        )
        .unwrap();
        ok.confidence = IdentityConfidence::Medium;
        assert!(ok.validate().is_err());
    }

    #[test]
    fn unknown_rejects_high_confidence() {
        let u = SpeakerAssertionUnknown::v1(
            IdentityConfidence::High,
            ReasonCodeId(1),
            vec![DiarizationSegment::v1(MonotonicTimeNs(0), MonotonicTimeNs(1), None).unwrap()],
        );
        assert!(u.is_err());
    }

    #[test]
    fn diarization_segment_requires_ordered_time_range() {
        let s = DiarizationSegment::v1(MonotonicTimeNs(2), MonotonicTimeNs(1), None);
        assert!(s.is_err());
    }

    #[test]
    fn request_validates_vad_stream_matches_processed_stream() {
        let stream = processed_stream();
        let vad = VadEvent::v1(
            AudioStreamId(999),
            MonotonicTimeNs(0),
            MonotonicTimeNs(10),
            Confidence::new(0.9).unwrap(),
            SpeechLikeness::new(0.9).unwrap(),
        );
        let r = Ph1VoiceIdRequest::v1(
            MonotonicTimeNs(0),
            stream,
            vec![vad],
            AudioDeviceId::new("mic").unwrap(),
            snapshot_active(),
            None,
            false,
            DeviceTrustLevel::Trusted,
            None,
        );
        assert!(r.is_err());
    }

    #[test]
    fn sim_request_rejects_mismatched_simulation_id() {
        let req = Ph1VoiceIdSimRequest {
            schema_version: PH1VOICEID_SIM_CONTRACT_VERSION,
            correlation_id: CorrelationId(1),
            turn_id: TurnId(1),
            now: MonotonicTimeNs(1),
            simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
            simulation_type: VoiceIdSimulationType::Draft,
            request: VoiceIdSimulationRequest::EnrollStartDraft(VoiceIdEnrollStartDraftRequest {
                onboarding_session_id: "onb_1".to_string(),
                device_id: DeviceId::new("device_1").unwrap(),
                consent_asserted: true,
                max_total_attempts: 8,
                max_session_enroll_time_ms: 120_000,
                lock_after_consecutive_passes: 3,
            }),
        };
        assert!(req.validate().is_err());
    }

    #[test]
    fn sim_ok_requires_exactly_one_result_payload() {
        let sid = VoiceEnrollmentSessionId::new("vid_enr_1").unwrap();
        let start =
            VoiceIdEnrollStartResult::v1(sid.clone(), VoiceEnrollStatus::InProgress, 8, 120_000, 3)
                .unwrap();
        let sample = VoiceIdEnrollSampleResult::v1(
            sid,
            VoiceSampleResult::Pass,
            None,
            1,
            VoiceEnrollStatus::InProgress,
        )
        .unwrap();
        let ok = Ph1VoiceIdSimOk {
            simulation_id: VOICE_ID_ENROLL_SAMPLE_COMMIT.to_string(),
            reason_code: ReasonCodeId(1),
            enroll_start_result: Some(start),
            enroll_sample_result: Some(sample),
            enroll_complete_result: None,
            enroll_defer_result: None,
        };
        assert!(ok.validate().is_err());
    }
}
