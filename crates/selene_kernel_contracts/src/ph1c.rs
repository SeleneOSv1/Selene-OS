#![forbid(unsafe_code)]

use crate::ph1k::{
    AdvancedAudioQualityMetrics, Confidence, DegradationClassBundle, DeviceState,
    InterruptCandidateConfidenceBand, VadDecisionConfidenceBand,
};
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

impl Validate for SessionStateRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "session_state_ref.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        Ok(())
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

impl Validate for LanguageTag {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "language_tag",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "language_tag",
                reason: "must be <= 32 chars",
            });
        }
        Ok(())
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

impl Validate for LanguageHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "language_hint.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        self.language_tag.validate()?;
        Ok(())
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

impl Validate for NoiseLevelHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.0.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "noise_level_hint",
            });
        }
        if !(0.0..=1.0).contains(&self.0) {
            return Err(ContractViolation::InvalidRange {
                field: "noise_level_hint",
                min: 0.0,
                max: 1.0,
                got: self.0 as f64,
            });
        }
        Ok(())
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

impl Validate for VadQualityHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.0.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "vad_quality_hint",
            });
        }
        if !(0.0..=1.0).contains(&self.0) {
            return Err(ContractViolation::InvalidRange {
                field: "vad_quality_hint",
                min: 0.0,
                max: 1.0,
                got: self.0 as f64,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1cSttStrategy {
    Standard,
    NoiseRobust,
    CloudAssist,
    ClarifyOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeakerOverlapClass {
    SingleSpeaker,
    MultiSpeaker,
    InterruptionOverlap,
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpeakerOverlapHint {
    pub schema_version: SchemaVersion,
    pub overlap_class: SpeakerOverlapClass,
    pub confidence: Confidence,
}

impl SpeakerOverlapHint {
    pub fn v1(
        overlap_class: SpeakerOverlapClass,
        confidence: Confidence,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            overlap_class,
            confidence,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SpeakerOverlapHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "speaker_overlap_hint.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        if !self.confidence.0.is_finite() || !(0.0..=1.0).contains(&self.confidence.0) {
            return Err(ContractViolation::InvalidRange {
                field: "speaker_overlap_hint.confidence",
                min: 0.0,
                max: 1.0,
                got: self.confidence.0 as f64,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1kToPh1cHandoff {
    pub schema_version: SchemaVersion,
    pub interrupt_confidence_band: InterruptCandidateConfidenceBand,
    pub vad_confidence_band: VadDecisionConfidenceBand,
    pub quality_metrics: AdvancedAudioQualityMetrics,
    pub degradation_class_bundle: DegradationClassBundle,
}

impl Ph1kToPh1cHandoff {
    pub fn v1(
        interrupt_confidence_band: InterruptCandidateConfidenceBand,
        vad_confidence_band: VadDecisionConfidenceBand,
        quality_metrics: AdvancedAudioQualityMetrics,
        degradation_class_bundle: DegradationClassBundle,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            interrupt_confidence_band,
            vad_confidence_band,
            quality_metrics,
            degradation_class_bundle,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for Ph1kToPh1cHandoff {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1k_to_ph1c_handoff.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        self.quality_metrics.validate()?;
        Ok(())
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
    pub speaker_overlap_hint: Option<SpeakerOverlapHint>,
    pub ph1k_handoff: Option<Ph1kToPh1cHandoff>,
}

impl Ph1cRequest {
    pub fn v1(
        bounded_audio_segment_ref: BoundedAudioSegmentRef,
        session_state_ref: SessionStateRef,
        device_state_ref: DeviceState,
        language_hint: Option<LanguageHint>,
        noise_level_hint: Option<NoiseLevelHint>,
        vad_quality_hint: Option<VadQualityHint>,
        ph1k_handoff: Option<Ph1kToPh1cHandoff>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            bounded_audio_segment_ref,
            session_state_ref,
            device_state_ref,
            language_hint,
            noise_level_hint,
            vad_quality_hint,
            speaker_overlap_hint: None,
            ph1k_handoff,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn with_speaker_overlap_hint(
        mut self,
        speaker_overlap_hint: Option<SpeakerOverlapHint>,
    ) -> Result<Self, ContractViolation> {
        self.speaker_overlap_hint = speaker_overlap_hint;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for Ph1cRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1c_request.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        self.bounded_audio_segment_ref.validate()?;
        self.session_state_ref.validate()?;
        self.device_state_ref.validate()?;
        if let Some(h) = &self.language_hint {
            h.validate()?;
        }
        if let Some(h) = &self.noise_level_hint {
            h.validate()?;
        }
        if let Some(h) = &self.vad_quality_hint {
            h.validate()?;
        }
        if let Some(h) = &self.speaker_overlap_hint {
            h.validate()?;
        }
        if let Some(h) = &self.ph1k_handoff {
            h.validate()?;
        }
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
    SwitchToText,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RouteClassUsed {
    OnDevice,
    OnPrem,
    CloudAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelectedSlot {
    Primary,
    Secondary,
    Tertiary,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoutingModeUsed {
    Shadow,
    Assist,
    Lead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QualityBucket {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ph1cAuditMeta {
    pub schema_version: SchemaVersion,
    pub route_class_used: RouteClassUsed,
    pub attempt_count: u8,
    pub candidate_count: u8,
    pub selected_slot: SelectedSlot,
    pub mode_used: RoutingModeUsed,
    pub second_pass_used: bool,
    pub total_latency_ms: u32,
    pub quality_coverage_bucket: QualityBucket,
    pub quality_confidence_bucket: QualityBucket,
    pub quality_plausibility_bucket: QualityBucket,
    pub tenant_vocabulary_pack_id: Option<String>,
    pub user_vocabulary_pack_id: Option<String>,
    pub policy_profile_id: Option<String>,
    pub stt_routing_policy_pack_id: Option<String>,
}

impl Ph1cAuditMeta {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        route_class_used: RouteClassUsed,
        attempt_count: u8,
        candidate_count: u8,
        selected_slot: SelectedSlot,
        mode_used: RoutingModeUsed,
        second_pass_used: bool,
        total_latency_ms: u32,
        quality_coverage_bucket: QualityBucket,
        quality_confidence_bucket: QualityBucket,
        quality_plausibility_bucket: QualityBucket,
        tenant_vocabulary_pack_id: Option<String>,
        user_vocabulary_pack_id: Option<String>,
        policy_profile_id: Option<String>,
        stt_routing_policy_pack_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let m = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            route_class_used,
            attempt_count,
            candidate_count,
            selected_slot,
            mode_used,
            second_pass_used,
            total_latency_ms,
            quality_coverage_bucket,
            quality_confidence_bucket,
            quality_plausibility_bucket,
            tenant_vocabulary_pack_id,
            user_vocabulary_pack_id,
            policy_profile_id,
            stt_routing_policy_pack_id,
        };
        m.validate()?;
        Ok(m)
    }
}

impl Validate for Ph1cAuditMeta {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1c_audit_meta.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        if self.candidate_count < self.attempt_count {
            return Err(ContractViolation::InvalidValue {
                field: "ph1c_audit_meta.candidate_count",
                reason: "must be >= attempt_count",
            });
        }
        for (field, value, max_len) in [
            (
                "ph1c_audit_meta.tenant_vocabulary_pack_id",
                self.tenant_vocabulary_pack_id.as_ref(),
                128usize,
            ),
            (
                "ph1c_audit_meta.user_vocabulary_pack_id",
                self.user_vocabulary_pack_id.as_ref(),
                128usize,
            ),
            (
                "ph1c_audit_meta.policy_profile_id",
                self.policy_profile_id.as_ref(),
                128usize,
            ),
            (
                "ph1c_audit_meta.stt_routing_policy_pack_id",
                self.stt_routing_policy_pack_id.as_ref(),
                128usize,
            ),
        ] {
            if let Some(v) = value {
                if v.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field,
                        reason: "must not be empty when provided",
                    });
                }
                if v.len() > max_len {
                    return Err(ContractViolation::InvalidValue {
                        field,
                        reason: "too long",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncertainSpan {
    pub start_byte: u32,
    pub end_byte: u32,
    pub field_hint: Option<String>,
}

impl UncertainSpan {
    pub fn v1(
        start_byte: u32,
        end_byte: u32,
        field_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            start_byte,
            end_byte,
            field_hint,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for UncertainSpan {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.end_byte <= self.start_byte {
            return Err(ContractViolation::InvalidValue {
                field: "uncertain_span.end_byte",
                reason: "must be > start_byte",
            });
        }
        if let Some(h) = &self.field_hint {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "uncertain_span.field_hint",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "uncertain_span.field_hint",
                    reason: "must be <= 64 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TranscriptOk {
    pub schema_version: SchemaVersion,
    pub transcript_text: String,
    pub language_tag: LanguageTag,
    pub confidence_bucket: ConfidenceBucket,
    pub uncertain_spans: Vec<UncertainSpan>,
    pub audit_meta: Option<Ph1cAuditMeta>,
}

impl TranscriptOk {
    pub fn v1(
        transcript_text: String,
        language_tag: LanguageTag,
        confidence_bucket: ConfidenceBucket,
    ) -> Result<Self, ContractViolation> {
        Self::v1_with_metadata(
            transcript_text,
            language_tag,
            confidence_bucket,
            vec![],
            None,
        )
    }

    pub fn v1_with_metadata(
        transcript_text: String,
        language_tag: LanguageTag,
        confidence_bucket: ConfidenceBucket,
        uncertain_spans: Vec<UncertainSpan>,
        audit_meta: Option<Ph1cAuditMeta>,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            transcript_text,
            language_tag,
            confidence_bucket,
            uncertain_spans,
            audit_meta,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for TranscriptOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_ok.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        self.language_tag.validate()?;
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
        if self.uncertain_spans.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_ok.uncertain_spans",
                reason: "must contain <= 8 entries",
            });
        }
        for s in &self.uncertain_spans {
            s.validate()?;
            if (s.end_byte as usize) > self.transcript_text.len() {
                return Err(ContractViolation::InvalidValue {
                    field: "transcript_ok.uncertain_spans.end_byte",
                    reason: "must be <= transcript_text byte length",
                });
            }
            if !self.transcript_text.is_char_boundary(s.start_byte as usize)
                || !self.transcript_text.is_char_boundary(s.end_byte as usize)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "transcript_ok.uncertain_spans",
                    reason: "start/end must align to UTF-8 char boundaries",
                });
            }
        }
        if let Some(m) = &self.audit_meta {
            m.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TranscriptReject {
    pub schema_version: SchemaVersion,
    pub reason_code: ReasonCodeId,
    pub retry_advice: RetryAdvice,
    pub audit_meta: Option<Ph1cAuditMeta>,
}

impl TranscriptReject {
    pub fn v1(reason_code: ReasonCodeId, retry_advice: RetryAdvice) -> Self {
        Self::v1_with_metadata(reason_code, retry_advice, None)
    }

    pub fn v1_with_metadata(
        reason_code: ReasonCodeId,
        retry_advice: RetryAdvice,
        audit_meta: Option<Ph1cAuditMeta>,
    ) -> Self {
        Self {
            schema_version: PH1C_CONTRACT_VERSION,
            reason_code,
            retry_advice,
            audit_meta,
        }
    }
}

impl Validate for TranscriptReject {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_reject.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "transcript_reject.reason_code",
                reason: "must be > 0",
            });
        }
        if let Some(m) = &self.audit_meta {
            m.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1cResponse {
    TranscriptOk(TranscriptOk),
    TranscriptReject(TranscriptReject),
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartialTranscript {
    pub schema_version: SchemaVersion,
    pub text_chunk: String,
    pub confidence: Confidence,
    pub stable: bool,
    pub revision_id: u32,
}

impl PartialTranscript {
    pub fn v1(
        text_chunk: String,
        confidence: Confidence,
        stable: bool,
        revision_id: u32,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            text_chunk,
            confidence,
            stable,
            revision_id,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PartialTranscript {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        if self.text_chunk.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript.text_chunk",
                reason: "must not be empty",
            });
        }
        if self.text_chunk.len() > 8_192 {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript.text_chunk",
                reason: "must be <= 8192 bytes",
            });
        }
        if self.revision_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript.revision_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PartialTranscriptBatch {
    pub schema_version: SchemaVersion,
    pub partials: Vec<PartialTranscript>,
    pub finalized: bool,
}

impl PartialTranscriptBatch {
    pub fn v1(
        partials: Vec<PartialTranscript>,
        finalized: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            partials,
            finalized,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PartialTranscriptBatch {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        if self.partials.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.partials",
                reason: "must not be empty",
            });
        }
        if self.partials.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.partials",
                reason: "must contain <= 128 entries",
            });
        }
        for (idx, partial) in self.partials.iter().enumerate() {
            partial.validate()?;
            let expected_revision_id = (idx as u32) + 1;
            if partial.revision_id != expected_revision_id {
                return Err(ContractViolation::InvalidValue {
                    field: "partial_transcript_batch.partials.revision_id",
                    reason: "must be strictly ordered, unique, and contiguous from 1",
                });
            }
        }
        if self.finalized && !self.partials.last().is_some_and(|p| p.stable) {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.finalized",
                reason: "requires last partial revision to be stable=true",
            });
        }
        Ok(())
    }
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
    use crate::ph1k::{
        AdvancedAudioQualityMetrics, AudioDeviceId, DegradationClassBundle, DeviceHealth,
        DeviceState, InterruptCandidateConfidenceBand, PreRollBufferId, VadDecisionConfidenceBand,
    };
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
            None,
        );
        assert!(req.is_ok());
    }

    #[test]
    fn request_accepts_ph1k_handoff_payload() {
        let seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            MonotonicTimeNs(12),
            MonotonicTimeNs(13),
        )
        .unwrap();
        let handoff = Ph1kToPh1cHandoff::v1(
            InterruptCandidateConfidenceBand::Medium,
            VadDecisionConfidenceBand::Medium,
            AdvancedAudioQualityMetrics::v1(24.0, 0.03, 50.0, 2.0, 0.2, 16.0).unwrap(),
            DegradationClassBundle::from_flags(false, true, false, false),
        )
        .unwrap();
        let req = Ph1cRequest::v1(
            seg,
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
            Some(handoff),
        );
        assert!(req.is_ok());
    }

    #[test]
    fn request_accepts_speaker_overlap_hint() {
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
            None,
        )
        .unwrap()
        .with_speaker_overlap_hint(Some(
            SpeakerOverlapHint::v1(
                SpeakerOverlapClass::InterruptionOverlap,
                Confidence::new(0.91).unwrap(),
            )
            .unwrap(),
        ));
        assert!(req.is_ok());
    }

    #[test]
    fn speaker_overlap_hint_rejects_schema_drift() {
        let mut hint = SpeakerOverlapHint::v1(
            SpeakerOverlapClass::MultiSpeaker,
            Confidence::new(0.8).unwrap(),
        )
        .unwrap();
        hint.schema_version = SchemaVersion(999);
        assert!(hint.validate().is_err());
    }

    #[test]
    fn request_rejects_session_state_schema_drift() {
        let seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(20),
            MonotonicTimeNs(12),
            MonotonicTimeNs(13),
        )
        .unwrap();
        let mut req = Ph1cRequest::v1(
            seg,
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        req.session_state_ref.schema_version = SchemaVersion(999);
        assert!(req.validate().is_err());
    }

    #[test]
    fn transcript_ok_rejects_schema_drift() {
        let mut out = TranscriptOk::v1(
            "set reminder".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        out.schema_version = SchemaVersion(999);
        assert!(out.validate().is_err());
    }

    #[test]
    fn partial_transcript_requires_non_empty_chunk() {
        let out = PartialTranscript::v1("   ".to_string(), Confidence::new(0.8).unwrap(), false, 1);
        assert!(out.is_err());
    }

    #[test]
    fn partial_transcript_batch_requires_ordered_contiguous_revisions() {
        let one =
            PartialTranscript::v1("hello".to_string(), Confidence::new(0.7).unwrap(), false, 1)
                .unwrap();
        let three = PartialTranscript::v1(
            "hello world".to_string(),
            Confidence::new(0.8).unwrap(),
            true,
            3,
        )
        .unwrap();
        let out = PartialTranscriptBatch::v1(vec![one, three], false);
        assert!(out.is_err());
    }

    #[test]
    fn partial_transcript_batch_finalized_requires_stable_last_revision() {
        let one =
            PartialTranscript::v1("hello".to_string(), Confidence::new(0.7).unwrap(), false, 1)
                .unwrap();
        let two = PartialTranscript::v1(
            "hello there".to_string(),
            Confidence::new(0.9).unwrap(),
            false,
            2,
        )
        .unwrap();
        let out = PartialTranscriptBatch::v1(vec![one, two], true);
        assert!(out.is_err());
    }

    #[test]
    fn partial_transcript_batch_accepts_deterministic_finalized_sequence() {
        let one =
            PartialTranscript::v1("hello".to_string(), Confidence::new(0.7).unwrap(), false, 1)
                .unwrap();
        let two = PartialTranscript::v1(
            "hello there".to_string(),
            Confidence::new(0.9).unwrap(),
            true,
            2,
        )
        .unwrap();
        let out = PartialTranscriptBatch::v1(vec![one, two], true).unwrap();
        assert_eq!(out.partials.len(), 2);
        assert!(out.finalized);
    }
}
