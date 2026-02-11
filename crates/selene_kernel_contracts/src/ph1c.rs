#![forbid(unsafe_code)]

use crate::ph1k::{Confidence, DeviceState};
use crate::ph1w::{BoundedAudioSegmentRef, SessionState};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1C_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionStateRef {
    pub schema_version: SchemaVersion,
    pub session_state: SessionState,
    pub tts_playback_active: bool,
}

impl SessionStateRef {
    pub fn v1(session_state: SessionState, tts_playback_active: bool) -> Self {
        Self {
            schema_version: PH1C_CONTRACT_VERSION,
            session_state,
            tts_playback_active,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn new(tag: impl Into<String>) -> Result<Self, ContractViolation> {
        let tag = tag.into();
        if tag.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "language_tag",
                reason: "must not be empty",
            });
        }
        if tag.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "language_tag",
                reason: "must be <= 32 chars",
            });
        }
        Ok(Self(tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LanguageHintConfidence {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LanguageHint {
    pub schema_version: SchemaVersion,
    pub language_tag: LanguageTag,
    pub confidence: LanguageHintConfidence,
}

impl LanguageHint {
    pub fn v1(language_tag: LanguageTag, confidence: LanguageHintConfidence) -> Self {
        Self {
            schema_version: PH1C_CONTRACT_VERSION,
            language_tag,
            confidence,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoiseLevelHint(pub f32);

impl NoiseLevelHint {
    pub fn new(value: f32) -> Result<Self, ContractViolation> {
        if !value.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "noise_level_hint",
            });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ContractViolation::InvalidRange {
                field: "noise_level_hint",
                min: 0.0,
                max: 1.0,
                got: value as f64,
            });
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VadQualityHint(pub f32);

impl VadQualityHint {
    pub fn new(value: f32) -> Result<Self, ContractViolation> {
        if !value.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "vad_quality_hint",
            });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ContractViolation::InvalidRange {
                field: "vad_quality_hint",
                min: 0.0,
                max: 1.0,
                got: value as f64,
            });
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1cRequest {
    pub schema_version: SchemaVersion,
    pub bounded_audio_segment_ref: BoundedAudioSegmentRef,
    pub session_state_ref: SessionStateRef,
    pub device_state_ref: DeviceState,
    pub language_hint: Option<LanguageHint>,
    pub noise_level_hint: Option<NoiseLevelHint>,
    pub vad_quality_hint: Option<VadQualityHint>,
}

impl Ph1cRequest {
    pub fn v1(
        bounded_audio_segment_ref: BoundedAudioSegmentRef,
        session_state_ref: SessionStateRef,
        device_state_ref: DeviceState,
        language_hint: Option<LanguageHint>,
        noise_level_hint: Option<NoiseLevelHint>,
        vad_quality_hint: Option<VadQualityHint>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            bounded_audio_segment_ref,
            session_state_ref,
            device_state_ref,
            language_hint,
            noise_level_hint,
            vad_quality_hint,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1cRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.bounded_audio_segment_ref.validate()?;
        self.device_state_ref.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfidenceBucket {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RetryAdvice {
    Repeat,
    SpeakSlower,
    MoveCloser,
    QuietEnv,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptOk {
    pub schema_version: SchemaVersion,
    pub transcript_text: String,
    pub language_tag: LanguageTag,
    pub confidence_bucket: ConfidenceBucket,
}

impl TranscriptOk {
    pub fn v1(
        transcript_text: String,
        language_tag: LanguageTag,
        confidence_bucket: ConfidenceBucket,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            transcript_text,
            language_tag,
            confidence_bucket,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for TranscriptOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.transcript_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_ok.transcript_text",
                reason: "must not be empty",
            });
        }
        if self.transcript_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_ok.transcript_text",
                reason: "must be <= 32768 bytes",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptReject {
    pub schema_version: SchemaVersion,
    pub reason_code: ReasonCodeId,
    pub retry_advice: RetryAdvice,
}

impl TranscriptReject {
    pub fn v1(reason_code: ReasonCodeId, retry_advice: RetryAdvice) -> Self {
        Self {
            schema_version: PH1C_CONTRACT_VERSION,
            reason_code,
            retry_advice,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1cResponse {
    TranscriptOk(TranscriptOk),
    TranscriptReject(TranscriptReject),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NormalizedSttConfidence {
    pub avg_word_confidence: Confidence,
    pub low_confidence_ratio: f32,
    pub stable: bool,
}

impl NormalizedSttConfidence {
    pub fn v1(
        avg_word_confidence: Confidence,
        low_confidence_ratio: f32,
        stable: bool,
    ) -> Result<Self, ContractViolation> {
        if !low_confidence_ratio.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "normalized_stt_confidence.low_confidence_ratio",
            });
        }
        if !(0.0..=1.0).contains(&low_confidence_ratio) {
            return Err(ContractViolation::InvalidRange {
                field: "normalized_stt_confidence.low_confidence_ratio",
                min: 0.0,
                max: 1.0,
                got: low_confidence_ratio as f64,
            });
        }
        Ok(Self {
            avg_word_confidence,
            low_confidence_ratio,
            stable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1k::AudioStreamId;
    use crate::ph1k::{AudioDeviceId, DeviceHealth, DeviceState, PreRollBufferId};
    use crate::ph1w::BoundedAudioSegmentRef;
    use crate::MonotonicTimeNs;

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    #[test]
    fn transcript_ok_requires_non_empty_text() {
        let out = TranscriptOk::v1(
            "   ".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        );
        assert!(out.is_err());
    }

    #[test]
    fn request_validates_audio_segment_ref() {
        let bad_seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(9),
            MonotonicTimeNs(10),
            MonotonicTimeNs(10),
        );
        assert!(bad_seg.is_err());

        let seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            MonotonicTimeNs(12),
            MonotonicTimeNs(13),
        )
        .unwrap();

        let req = Ph1cRequest::v1(
            seg,
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
        );
        assert!(req.is_ok());
    }
}
