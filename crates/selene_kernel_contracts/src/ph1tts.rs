#![forbid(unsafe_code)]

use crate::ph1c::{LanguageTag, SessionStateRef};
use crate::ph1d::PolicyContextRef;
use crate::MonotonicTimeNs;
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1TTS_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AnswerId(pub u128);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VoiceId(String);

impl VoiceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_id",
                reason: "must be <= 64 chars",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TtsControl {
    Play,
    Cancel,
    Pause,
    Resume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleProfileRef {
    Dominant,
    Gentle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StyleModifier {
    Brief,
    Warm,
    Formal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BargeInPolicyRef {
    Aggressive,
    Standard,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoicePrefRef(String);

impl VoicePrefRef {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "voice_pref_ref",
                reason: "must not be empty",
            });
        }
        Ok(Self(id))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceRenderPlan {
    pub schema_version: SchemaVersion,
    pub style_profile_ref: StyleProfileRef,
    pub modifiers: Vec<StyleModifier>,
    pub barge_in_policy_ref: BargeInPolicyRef,
    pub language_tag: LanguageTag,
    pub voice_pref_ref: Option<VoicePrefRef>,
}

impl VoiceRenderPlan {
    pub fn v1(
        style_profile_ref: StyleProfileRef,
        modifiers: Vec<StyleModifier>,
        barge_in_policy_ref: BargeInPolicyRef,
        language_tag: LanguageTag,
        voice_pref_ref: Option<VoicePrefRef>,
    ) -> Self {
        Self {
            schema_version: PH1TTS_CONTRACT_VERSION,
            style_profile_ref,
            modifiers,
            barge_in_policy_ref,
            language_tag,
            voice_pref_ref,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1ttsRequest {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub response_text: String,
    pub tts_control: TtsControl,
    pub session_state_ref: SessionStateRef,
    pub render_plan: VoiceRenderPlan,
    pub policy_context_ref: PolicyContextRef,
}

impl Ph1ttsRequest {
    pub fn v1(
        answer_id: AnswerId,
        response_text: String,
        tts_control: TtsControl,
        session_state_ref: SessionStateRef,
        render_plan: VoiceRenderPlan,
        policy_context_ref: PolicyContextRef,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1TTS_CONTRACT_VERSION,
            answer_id,
            response_text,
            tts_control,
            session_state_ref,
            render_plan,
            policy_context_ref,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1ttsRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TTS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tts_request.schema_version",
                reason: "must match PH1TTS_CONTRACT_VERSION",
            });
        }
        self.policy_context_ref.validate()?;
        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tts_request.response_text",
                reason: "must not be empty",
            });
        }
        if self.response_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tts_request.response_text",
                reason: "must be <= 32768 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpokenCursor {
    pub schema_version: SchemaVersion,
    /// Byte offset into `response_text` (UTF-8). Must be a safe boundary (segment end).
    pub byte_offset: u32,
    /// Number of segments fully spoken so far (0..=segments_total).
    pub segments_spoken: u16,
    /// Total number of segments in the deterministic segment plan.
    pub segments_total: u16,
}

impl SpokenCursor {
    pub fn v1(
        byte_offset: u32,
        segments_spoken: u16,
        segments_total: u16,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1TTS_CONTRACT_VERSION,
            byte_offset,
            segments_spoken,
            segments_total,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for SpokenCursor {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TTS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "spoken_cursor.schema_version",
                reason: "must match PH1TTS_CONTRACT_VERSION",
            });
        }
        if self.segments_total == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "spoken_cursor.segments_total",
                reason: "must be > 0",
            });
        }
        if self.segments_total > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "spoken_cursor.segments_total",
                reason: "must be <= 512",
            });
        }
        if self.segments_spoken > self.segments_total {
            return Err(ContractViolation::InvalidValue {
                field: "spoken_cursor.segments_spoken",
                reason: "must be <= segments_total",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TtsStopReason {
    Completed,
    Cancelled,
    Interrupted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsStarted {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub voice_id: VoiceId,
    pub t_started: MonotonicTimeNs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsProgress {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub ms_played: u32,
    pub spoken_cursor: SpokenCursor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsStopped {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub reason: TtsStopReason,
    pub t_stopped: MonotonicTimeNs,
    pub spoken_cursor: SpokenCursor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsFailed {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub reason_code: ReasonCodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ttsEvent {
    Started(TtsStarted),
    Progress(TtsProgress),
    Stopped(TtsStopped),
    Failed(TtsFailed),
}
