#![forbid(unsafe_code)]

use crate::ph1k::{
    AdvancedAudioQualityMetrics, Confidence, DegradationClassBundle, DeviceState,
    InterruptCandidateConfidenceBand, VadDecisionConfidenceBand,
};
use crate::ph1w::{BoundedAudioSegmentRef, SessionState};
use crate::provider_secrets::ConsentScope;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1cSttStrategy {
    Standard,
    NoiseRobust,
    CloudAssist,
    ClarifyOnly,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpeakerOverlapClass {
    Unknown,
    SingleSpeaker,
    MultiSpeaker,
    InterruptionOverlap,
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
        let hint = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            overlap_class,
            confidence,
        };
        hint.validate()?;
        Ok(hint)
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
        self.bounded_audio_segment_ref.validate()?;
        self.session_state_ref.validate()?;
        self.device_state_ref.validate()?;
        if let Some(hint) = &self.speaker_overlap_hint {
            hint.validate()?;
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
        let partial = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            text_chunk,
            confidence,
            stable,
            revision_id,
        };
        partial.validate()?;
        Ok(partial)
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
                reason: "must be <= 8192 chars",
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
        let batch = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            partials,
            finalized,
        };
        batch.validate()?;
        Ok(batch)
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
                reason: "must contain at least one entry",
            });
        }
        if self.partials.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.partials",
                reason: "must contain <= 64 entries",
            });
        }
        let mut expected_revision: u32 = 1;
        for partial in &self.partials {
            partial.validate()?;
            if partial.revision_id != expected_revision {
                return Err(ContractViolation::InvalidValue {
                    field: "partial_transcript_batch.partials[].revision_id",
                    reason: "must be contiguous and start at 1",
                });
            }
            expected_revision = expected_revision.saturating_add(1);
        }
        if self.finalized && !self.partials.last().is_some_and(|partial| partial.stable) {
            return Err(ContractViolation::InvalidValue {
                field: "partial_transcript_batch.finalized",
                reason: "last partial must be stable when finalized=true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderTaskKind {
    Stt,
    Tts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderKind {
    ApplePlatform,
    OpenAiRealtime,
    OpenAiTranscription,
    OpenAiSpeech,
    LocalOffline,
    LocalPlatform,
    OtherApproved,
}

impl VoiceProviderKind {
    pub const fn requires_cloud_dispatch(self) -> bool {
        matches!(
            self,
            VoiceProviderKind::OpenAiRealtime
                | VoiceProviderKind::OpenAiTranscription
                | VoiceProviderKind::OpenAiSpeech
                | VoiceProviderKind::OtherApproved
        )
    }

    pub const fn requires_secret_by_default(self) -> bool {
        matches!(
            self,
            VoiceProviderKind::OpenAiRealtime
                | VoiceProviderKind::OpenAiTranscription
                | VoiceProviderKind::OpenAiSpeech
                | VoiceProviderKind::OtherApproved
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderPlatform {
    Mac,
    Iphone,
    Android,
    Windows,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderPrivacyMode {
    LocalOnly,
    ProviderCloud,
    Hybrid,
    Disabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderLatencyClass {
    Realtime,
    NearRealtime,
    Batch,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderCostClass {
    PlatformIncluded,
    Low,
    Medium,
    High,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VoiceProviderFallbackReason {
    ProviderDisabled,
    ProviderSecretMissing,
    ProviderBudgetBlocked,
    ProviderTimeout,
    ProviderError,
    ProviderLatencyTooHigh,
    ProviderConfidenceTooLow,
    ProviderLanguageUnsupported,
    ProviderPlatformUnsupported,
    PrivacyPolicyBlocked,
    ConsentMissingOrRevoked,
    OfflineModeRequired,
    ProtectedSlotRequiresSecondPass,
    NoApprovedProviderAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceProviderSelection {
    pub provider_id: String,
    pub provider_kind: VoiceProviderKind,
}

impl VoiceProviderSelection {
    pub fn v1(
        provider_id: impl Into<String>,
        provider_kind: VoiceProviderKind,
    ) -> Result<Self, ContractViolation> {
        let selection = Self {
            provider_id: provider_id.into(),
            provider_kind,
        };
        selection.validate()?;
        Ok(selection)
    }
}

impl Validate for VoiceProviderSelection {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_voice_provider_token(
            "voice_provider_selection.provider_id",
            &self.provider_id,
            96,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceProviderQualitySignal {
    pub language_supported: bool,
    pub platform_supported: bool,
    pub confidence_bp: Option<u16>,
    pub latency_ms: Option<u32>,
    pub privacy_policy_passed: bool,
    pub budget_available: bool,
    pub provider_secret_available: bool,
}

impl VoiceProviderQualitySignal {
    pub const fn ready_contract_only(
        language_supported: bool,
        platform_supported: bool,
        privacy_policy_passed: bool,
        budget_available: bool,
        provider_secret_available: bool,
    ) -> Self {
        Self {
            language_supported,
            platform_supported,
            confidence_bp: None,
            latency_ms: None,
            privacy_policy_passed,
            budget_available,
            provider_secret_available,
        }
    }
}

impl Validate for VoiceProviderQualitySignal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if matches!(self.confidence_bp, Some(confidence_bp) if confidence_bp > 10_000) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_quality_signal.confidence_bp",
                reason: "must be <= 10000",
            });
        }
        if matches!(self.latency_ms, Some(latency_ms) if latency_ms > 600_000) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_quality_signal.latency_ms",
                reason: "must be <= 600000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SttProviderProfilePacket {
    pub schema_version: SchemaVersion,
    pub provider_id: String,
    pub provider_kind: VoiceProviderKind,
    pub supported_platforms: Vec<VoiceProviderPlatform>,
    pub supported_languages: Vec<LanguageTag>,
    pub privacy_mode: VoiceProviderPrivacyMode,
    pub latency_class: VoiceProviderLatencyClass,
    pub offline_capable: bool,
    pub requires_network: bool,
    pub requires_user_consent: bool,
    pub requires_provider_secret: bool,
    pub confidence_available: bool,
    pub alternatives_available: bool,
    pub timestamps_available: bool,
    pub word_timestamps_available: bool,
    pub cost_class: VoiceProviderCostClass,
    pub provider_gate_id: String,
    pub consent_scope: Option<ConsentScope>,
    pub audit_id: Option<String>,
}

impl SttProviderProfilePacket {
    pub fn apple_platform(
        provider_id: impl Into<String>,
        supported_languages: Vec<LanguageTag>,
        provider_gate_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            provider_id,
            VoiceProviderKind::ApplePlatform,
            vec![VoiceProviderPlatform::Mac, VoiceProviderPlatform::Iphone],
            supported_languages,
            VoiceProviderPrivacyMode::LocalOnly,
            VoiceProviderLatencyClass::Realtime,
            true,
            false,
            true,
            false,
            true,
            true,
            true,
            true,
            VoiceProviderCostClass::PlatformIncluded,
            provider_gate_id,
            Some(ConsentScope::ProviderCapableVoiceProcessing),
            None,
        )
    }

    pub fn openai_realtime(
        provider_id: impl Into<String>,
        supported_languages: Vec<LanguageTag>,
        provider_gate_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            provider_id,
            VoiceProviderKind::OpenAiRealtime,
            vec![
                VoiceProviderPlatform::Mac,
                VoiceProviderPlatform::Iphone,
                VoiceProviderPlatform::Android,
                VoiceProviderPlatform::Windows,
                VoiceProviderPlatform::Server,
            ],
            supported_languages,
            VoiceProviderPrivacyMode::ProviderCloud,
            VoiceProviderLatencyClass::Realtime,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            VoiceProviderCostClass::Medium,
            provider_gate_id,
            Some(ConsentScope::ProviderCapableVoiceProcessing),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        provider_id: impl Into<String>,
        provider_kind: VoiceProviderKind,
        supported_platforms: Vec<VoiceProviderPlatform>,
        supported_languages: Vec<LanguageTag>,
        privacy_mode: VoiceProviderPrivacyMode,
        latency_class: VoiceProviderLatencyClass,
        offline_capable: bool,
        requires_network: bool,
        requires_user_consent: bool,
        requires_provider_secret: bool,
        confidence_available: bool,
        alternatives_available: bool,
        timestamps_available: bool,
        word_timestamps_available: bool,
        cost_class: VoiceProviderCostClass,
        provider_gate_id: impl Into<String>,
        consent_scope: Option<ConsentScope>,
        audit_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            provider_id: provider_id.into(),
            provider_kind,
            supported_platforms,
            supported_languages,
            privacy_mode,
            latency_class,
            offline_capable,
            requires_network,
            requires_user_consent,
            requires_provider_secret,
            confidence_available,
            alternatives_available,
            timestamps_available,
            word_timestamps_available,
            cost_class,
            provider_gate_id: provider_gate_id.into(),
            consent_scope,
            audit_id,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for SttProviderProfilePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "stt_provider_profile_packet.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        validate_voice_provider_token(
            "stt_provider_profile_packet.provider_id",
            &self.provider_id,
            96,
        )?;
        validate_voice_provider_token(
            "stt_provider_profile_packet.provider_gate_id",
            &self.provider_gate_id,
            128,
        )?;
        validate_optional_voice_provider_token(
            "stt_provider_profile_packet.audit_id",
            self.audit_id.as_deref(),
            128,
        )?;
        validate_voice_provider_platforms(
            "stt_provider_profile_packet.supported_platforms",
            &self.supported_platforms,
        )?;
        validate_voice_provider_languages(
            "stt_provider_profile_packet.supported_languages",
            &self.supported_languages,
        )?;
        validate_voice_provider_profile_constraints(
            "stt_provider_profile_packet",
            self.provider_kind,
            self.privacy_mode,
            self.offline_capable,
            self.requires_network,
            self.requires_provider_secret,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsProviderProfilePacket {
    pub schema_version: SchemaVersion,
    pub provider_id: String,
    pub provider_kind: VoiceProviderKind,
    pub supported_platforms: Vec<VoiceProviderPlatform>,
    pub supported_languages: Vec<LanguageTag>,
    pub voice_locales: Vec<LanguageTag>,
    pub privacy_mode: VoiceProviderPrivacyMode,
    pub latency_class: VoiceProviderLatencyClass,
    pub offline_capable: bool,
    pub requires_network: bool,
    pub requires_provider_secret: bool,
    pub streaming_supported: bool,
    pub interruption_supported: bool,
    pub prosody_supported: bool,
    pub pronunciation_memory_supported: bool,
    pub cost_class: VoiceProviderCostClass,
    pub provider_gate_id: String,
    pub consent_scope: Option<ConsentScope>,
    pub audit_id: Option<String>,
}

impl TtsProviderProfilePacket {
    pub fn apple_platform(
        provider_id: impl Into<String>,
        voice_locales: Vec<LanguageTag>,
        provider_gate_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            provider_id,
            VoiceProviderKind::ApplePlatform,
            vec![VoiceProviderPlatform::Mac, VoiceProviderPlatform::Iphone],
            voice_locales.clone(),
            voice_locales,
            VoiceProviderPrivacyMode::LocalOnly,
            VoiceProviderLatencyClass::Realtime,
            true,
            false,
            false,
            true,
            true,
            false,
            true,
            VoiceProviderCostClass::PlatformIncluded,
            provider_gate_id,
            Some(ConsentScope::ProviderCapableVoiceProcessing),
            None,
        )
    }

    pub fn openai_speech(
        provider_id: impl Into<String>,
        voice_locales: Vec<LanguageTag>,
        provider_gate_id: impl Into<String>,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            provider_id,
            VoiceProviderKind::OpenAiSpeech,
            vec![
                VoiceProviderPlatform::Mac,
                VoiceProviderPlatform::Iphone,
                VoiceProviderPlatform::Android,
                VoiceProviderPlatform::Windows,
                VoiceProviderPlatform::Server,
            ],
            voice_locales.clone(),
            voice_locales,
            VoiceProviderPrivacyMode::ProviderCloud,
            VoiceProviderLatencyClass::NearRealtime,
            false,
            true,
            true,
            true,
            true,
            true,
            true,
            VoiceProviderCostClass::Medium,
            provider_gate_id,
            Some(ConsentScope::ProviderCapableVoiceProcessing),
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        provider_id: impl Into<String>,
        provider_kind: VoiceProviderKind,
        supported_platforms: Vec<VoiceProviderPlatform>,
        supported_languages: Vec<LanguageTag>,
        voice_locales: Vec<LanguageTag>,
        privacy_mode: VoiceProviderPrivacyMode,
        latency_class: VoiceProviderLatencyClass,
        offline_capable: bool,
        requires_network: bool,
        requires_provider_secret: bool,
        streaming_supported: bool,
        interruption_supported: bool,
        prosody_supported: bool,
        pronunciation_memory_supported: bool,
        cost_class: VoiceProviderCostClass,
        provider_gate_id: impl Into<String>,
        consent_scope: Option<ConsentScope>,
        audit_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            provider_id: provider_id.into(),
            provider_kind,
            supported_platforms,
            supported_languages,
            voice_locales,
            privacy_mode,
            latency_class,
            offline_capable,
            requires_network,
            requires_provider_secret,
            streaming_supported,
            interruption_supported,
            prosody_supported,
            pronunciation_memory_supported,
            cost_class,
            provider_gate_id: provider_gate_id.into(),
            consent_scope,
            audit_id,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for TtsProviderProfilePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tts_provider_profile_packet.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        validate_voice_provider_token(
            "tts_provider_profile_packet.provider_id",
            &self.provider_id,
            96,
        )?;
        validate_voice_provider_token(
            "tts_provider_profile_packet.provider_gate_id",
            &self.provider_gate_id,
            128,
        )?;
        validate_optional_voice_provider_token(
            "tts_provider_profile_packet.audit_id",
            self.audit_id.as_deref(),
            128,
        )?;
        validate_voice_provider_platforms(
            "tts_provider_profile_packet.supported_platforms",
            &self.supported_platforms,
        )?;
        validate_voice_provider_languages(
            "tts_provider_profile_packet.supported_languages",
            &self.supported_languages,
        )?;
        validate_voice_provider_languages(
            "tts_provider_profile_packet.voice_locales",
            &self.voice_locales,
        )?;
        validate_voice_provider_profile_constraints(
            "tts_provider_profile_packet",
            self.provider_kind,
            self.privacy_mode,
            self.offline_capable,
            self.requires_network,
            self.requires_provider_secret,
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VoiceProviderRouteDecisionPacket {
    pub schema_version: SchemaVersion,
    pub route_id: String,
    pub task_kind: VoiceProviderTaskKind,
    pub requested_platform: VoiceProviderPlatform,
    pub device_class: String,
    pub network_state: String,
    pub privacy_requirement: VoiceProviderPrivacyMode,
    pub latency_requirement: VoiceProviderLatencyClass,
    pub language_hint: Option<LanguageTag>,
    pub confidence_requirement_bp: Option<u16>,
    pub protected_slot_risk: bool,
    pub candidate_providers: Vec<VoiceProviderSelection>,
    pub selected_provider: Option<VoiceProviderSelection>,
    pub fallback_provider: Option<VoiceProviderSelection>,
    pub fallback_reason: Option<VoiceProviderFallbackReason>,
    pub provider_off_state: bool,
    pub budget_state: String,
    pub consent_state_id: Option<String>,
    pub provider_budget_id: Option<String>,
    pub session_id: Option<String>,
    pub turn_id: Option<String>,
    pub activation_id: Option<String>,
    pub device_trust_id: Option<String>,
    pub transcript_hash: Option<String>,
    pub tts_hash: Option<String>,
    pub audit_id: String,
    pub quality_signal: VoiceProviderQualitySignal,
    pub provider_call_attempt_count: u32,
    pub provider_network_dispatch_count: u32,
}

impl VoiceProviderRouteDecisionPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn contract_only(
        route_id: impl Into<String>,
        task_kind: VoiceProviderTaskKind,
        requested_platform: VoiceProviderPlatform,
        device_class: impl Into<String>,
        network_state: impl Into<String>,
        privacy_requirement: VoiceProviderPrivacyMode,
        latency_requirement: VoiceProviderLatencyClass,
        language_hint: Option<LanguageTag>,
        confidence_requirement_bp: Option<u16>,
        protected_slot_risk: bool,
        candidate_providers: Vec<VoiceProviderSelection>,
        selected_provider: Option<VoiceProviderSelection>,
        fallback_provider: Option<VoiceProviderSelection>,
        fallback_reason: Option<VoiceProviderFallbackReason>,
        provider_off_state: bool,
        budget_state: impl Into<String>,
        consent_state_id: Option<String>,
        provider_budget_id: Option<String>,
        audit_id: impl Into<String>,
        quality_signal: VoiceProviderQualitySignal,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: PH1C_CONTRACT_VERSION,
            route_id: route_id.into(),
            task_kind,
            requested_platform,
            device_class: device_class.into(),
            network_state: network_state.into(),
            privacy_requirement,
            latency_requirement,
            language_hint,
            confidence_requirement_bp,
            protected_slot_risk,
            candidate_providers,
            selected_provider,
            fallback_provider,
            fallback_reason,
            provider_off_state,
            budget_state: budget_state.into(),
            consent_state_id,
            provider_budget_id,
            session_id: None,
            turn_id: None,
            activation_id: None,
            device_trust_id: None,
            transcript_hash: None,
            tts_hash: None,
            audit_id: audit_id.into(),
            quality_signal,
            provider_call_attempt_count: 0,
            provider_network_dispatch_count: 0,
        };
        packet.validate()?;
        Ok(packet)
    }

    pub fn with_trace_refs(
        mut self,
        session_id: Option<String>,
        turn_id: Option<String>,
        activation_id: Option<String>,
        device_trust_id: Option<String>,
        transcript_hash: Option<String>,
        tts_hash: Option<String>,
    ) -> Result<Self, ContractViolation> {
        self.session_id = session_id;
        self.turn_id = turn_id;
        self.activation_id = activation_id;
        self.device_trust_id = device_trust_id;
        self.transcript_hash = transcript_hash;
        self.tts_hash = tts_hash;
        self.validate()?;
        Ok(self)
    }

    pub const fn can_call_provider(&self) -> bool {
        false
    }

    pub const fn can_capture_audio(&self) -> bool {
        false
    }

    pub const fn can_transcribe(&self) -> bool {
        false
    }

    pub const fn can_synthesize_speech(&self) -> bool {
        false
    }

    pub const fn can_search_or_route_tools(&self) -> bool {
        false
    }

    pub const fn can_identify_authorize_or_mutate(&self) -> bool {
        false
    }

    pub fn selected_cloud_provider(&self) -> bool {
        self.selected_provider
            .as_ref()
            .is_some_and(|provider| provider.provider_kind.requires_cloud_dispatch())
            || self
                .fallback_provider
                .as_ref()
                .is_some_and(|provider| provider.provider_kind.requires_cloud_dispatch())
    }
}

impl Validate for VoiceProviderRouteDecisionPacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1C_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_route_decision_packet.schema_version",
                reason: "must match PH1C_CONTRACT_VERSION",
            });
        }
        validate_voice_provider_token(
            "voice_provider_route_decision_packet.route_id",
            &self.route_id,
            128,
        )?;
        validate_voice_provider_token(
            "voice_provider_route_decision_packet.device_class",
            &self.device_class,
            64,
        )?;
        validate_voice_provider_token(
            "voice_provider_route_decision_packet.network_state",
            &self.network_state,
            64,
        )?;
        validate_voice_provider_token(
            "voice_provider_route_decision_packet.budget_state",
            &self.budget_state,
            64,
        )?;
        validate_voice_provider_token(
            "voice_provider_route_decision_packet.audit_id",
            &self.audit_id,
            128,
        )?;
        for (field, value) in [
            (
                "voice_provider_route_decision_packet.consent_state_id",
                self.consent_state_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.provider_budget_id",
                self.provider_budget_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.session_id",
                self.session_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.turn_id",
                self.turn_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.activation_id",
                self.activation_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.device_trust_id",
                self.device_trust_id.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.transcript_hash",
                self.transcript_hash.as_deref(),
            ),
            (
                "voice_provider_route_decision_packet.tts_hash",
                self.tts_hash.as_deref(),
            ),
        ] {
            validate_optional_voice_provider_token(field, value, 160)?;
        }
        if matches!(self.confidence_requirement_bp, Some(confidence_bp) if confidence_bp > 10_000) {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_route_decision_packet.confidence_requirement_bp",
                reason: "must be <= 10000",
            });
        }
        if self.candidate_providers.is_empty() || self.candidate_providers.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_route_decision_packet.candidate_providers",
                reason: "must contain 1..=8 entries",
            });
        }
        for provider in &self.candidate_providers {
            provider.validate()?;
        }
        if let Some(selected) = self.selected_provider.as_ref() {
            selected.validate()?;
            if !self
                .candidate_providers
                .iter()
                .any(|provider| provider.provider_id == selected.provider_id)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_provider_route_decision_packet.selected_provider",
                    reason: "must be one of candidate_providers",
                });
            }
        }
        if let Some(fallback) = self.fallback_provider.as_ref() {
            fallback.validate()?;
            if !self
                .candidate_providers
                .iter()
                .any(|provider| provider.provider_id == fallback.provider_id)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_provider_route_decision_packet.fallback_provider",
                    reason: "must be one of candidate_providers",
                });
            }
        }
        self.quality_signal.validate()?;
        if self.provider_call_attempt_count != 0 || self.provider_network_dispatch_count != 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_provider_route_decision_packet.provider_counts",
                reason: "contract-only route decisions cannot attempt or dispatch providers",
            });
        }
        if self.provider_off_state {
            let Some(reason) = self.fallback_reason else {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_provider_route_decision_packet.fallback_reason",
                    reason: "provider-off decisions require an explicit fallback reason",
                });
            };
            if !matches!(
                reason,
                VoiceProviderFallbackReason::ProviderDisabled
                    | VoiceProviderFallbackReason::OfflineModeRequired
                    | VoiceProviderFallbackReason::NoApprovedProviderAvailable
                    | VoiceProviderFallbackReason::PrivacyPolicyBlocked
            ) {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_provider_route_decision_packet.fallback_reason",
                    reason: "provider-off reason must be a provider-off/offline/privacy/no-approved-provider reason",
                });
            }
            if self.selected_cloud_provider() {
                return Err(ContractViolation::InvalidValue {
                    field: "voice_provider_route_decision_packet.selected_provider",
                    reason: "provider-off decisions cannot select cloud providers",
                });
            }
        }
        Ok(())
    }
}

fn validate_voice_provider_profile_constraints(
    field_prefix: &'static str,
    provider_kind: VoiceProviderKind,
    privacy_mode: VoiceProviderPrivacyMode,
    offline_capable: bool,
    requires_network: bool,
    requires_provider_secret: bool,
) -> Result<(), ContractViolation> {
    if matches!(privacy_mode, VoiceProviderPrivacyMode::Disabled)
        && (offline_capable || requires_network || requires_provider_secret)
    {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason:
                "disabled provider profiles cannot require network/secret or be offline capable",
        });
    }
    if provider_kind.requires_secret_by_default() && !requires_provider_secret {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "cloud provider profiles require provider secret handles",
        });
    }
    if provider_kind.requires_cloud_dispatch() && !requires_network {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "cloud provider profiles require network",
        });
    }
    if matches!(
        provider_kind,
        VoiceProviderKind::ApplePlatform
            | VoiceProviderKind::LocalOffline
            | VoiceProviderKind::LocalPlatform
    ) && requires_provider_secret
    {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "local/platform provider profiles must not require provider secrets",
        });
    }
    if matches!(provider_kind, VoiceProviderKind::LocalOffline)
        && (!offline_capable
            || requires_network
            || !matches!(privacy_mode, VoiceProviderPrivacyMode::LocalOnly))
    {
        return Err(ContractViolation::InvalidValue {
            field: field_prefix,
            reason: "local offline provider profiles must be local-only, offline-capable, and network-free",
        });
    }
    Ok(())
}

fn validate_voice_provider_platforms(
    field: &'static str,
    platforms: &[VoiceProviderPlatform],
) -> Result<(), ContractViolation> {
    if platforms.is_empty() || platforms.len() > 8 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain 1..=8 entries",
        });
    }
    Ok(())
}

fn validate_voice_provider_languages(
    field: &'static str,
    languages: &[LanguageTag],
) -> Result<(), ContractViolation> {
    if languages.is_empty() || languages.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain 1..=128 entries",
        });
    }
    Ok(())
}

fn validate_optional_voice_provider_token(
    field: &'static str,
    value: Option<&str>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value {
        validate_voice_provider_token(field, value, max_len)?;
    }
    Ok(())
}

fn validate_voice_provider_token(
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
    if value.len() > max_len || !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be bounded ASCII",
        });
    }
    Ok(())
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

    fn lang(tag: &str) -> LanguageTag {
        LanguageTag::new(tag).unwrap()
    }

    fn selection(id: &str, kind: VoiceProviderKind) -> VoiceProviderSelection {
        VoiceProviderSelection::v1(id, kind).unwrap()
    }

    #[test]
    fn stage_3b_stt_profiles_cover_apple_and_openai_without_live_integration() {
        let apple = SttProviderProfilePacket::apple_platform(
            "apple-platform-stt",
            vec![lang("en-US"), lang("zh-CN")],
            "provider-gate-apple-platform",
        )
        .unwrap();
        assert_eq!(apple.provider_kind, VoiceProviderKind::ApplePlatform);
        assert!(apple
            .supported_platforms
            .contains(&VoiceProviderPlatform::Mac));
        assert!(apple
            .supported_platforms
            .contains(&VoiceProviderPlatform::Iphone));
        assert_eq!(apple.privacy_mode, VoiceProviderPrivacyMode::LocalOnly);
        assert!(apple.offline_capable);
        assert!(!apple.requires_network);
        assert!(!apple.requires_provider_secret);

        let openai = SttProviderProfilePacket::openai_realtime(
            "openai-realtime-stt",
            vec![lang("en-US"), lang("zh-CN")],
            "provider-gate-openai-realtime",
        )
        .unwrap();
        assert_eq!(openai.provider_kind, VoiceProviderKind::OpenAiRealtime);
        assert_eq!(openai.privacy_mode, VoiceProviderPrivacyMode::ProviderCloud);
        assert!(openai.requires_network);
        assert!(openai.requires_provider_secret);
        assert!(!openai.offline_capable);
    }

    #[test]
    fn stage_3b_tts_profiles_cover_apple_and_openai_without_live_integration() {
        let apple = TtsProviderProfilePacket::apple_platform(
            "apple-platform-tts",
            vec![lang("en-US")],
            "provider-gate-apple-tts",
        )
        .unwrap();
        assert_eq!(apple.provider_kind, VoiceProviderKind::ApplePlatform);
        assert!(apple.streaming_supported);
        assert!(apple.interruption_supported);
        assert!(!apple.requires_provider_secret);
        assert!(!apple.requires_network);

        let openai = TtsProviderProfilePacket::openai_speech(
            "openai-speech-tts",
            vec![lang("en-US")],
            "provider-gate-openai-speech",
        )
        .unwrap();
        assert_eq!(openai.provider_kind, VoiceProviderKind::OpenAiSpeech);
        assert!(openai.requires_network);
        assert!(openai.requires_provider_secret);
        assert!(openai.prosody_supported);
        assert!(openai.pronunciation_memory_supported);
    }

    #[test]
    fn stage_3b_route_decision_is_contract_only_and_cannot_call_providers() {
        let decision = VoiceProviderRouteDecisionPacket::contract_only(
            "voice-route-stage3b-stt",
            VoiceProviderTaskKind::Stt,
            VoiceProviderPlatform::Mac,
            "desktop",
            "online",
            VoiceProviderPrivacyMode::Hybrid,
            VoiceProviderLatencyClass::Realtime,
            Some(lang("en-US")),
            Some(9_000),
            false,
            vec![
                selection("apple-platform-stt", VoiceProviderKind::ApplePlatform),
                selection("openai-realtime-stt", VoiceProviderKind::OpenAiRealtime),
            ],
            Some(selection(
                "apple-platform-stt",
                VoiceProviderKind::ApplePlatform,
            )),
            Some(selection(
                "openai-realtime-stt",
                VoiceProviderKind::OpenAiRealtime,
            )),
            Some(VoiceProviderFallbackReason::ProviderLatencyTooHigh),
            false,
            "budget-available",
            Some("consent-provider-voice".to_string()),
            Some("provider-budget-stage3b".to_string()),
            "audit-stage3b-route",
            VoiceProviderQualitySignal::ready_contract_only(true, true, true, true, true),
        )
        .unwrap()
        .with_trace_refs(
            Some("session-stage3b".to_string()),
            Some("turn-stage3b".to_string()),
            Some("activation-stage3b".to_string()),
            Some("device-trust-stage3b".to_string()),
            Some("transcript-hash-stage3b".to_string()),
            None,
        )
        .unwrap();

        assert!(!decision.can_call_provider());
        assert!(!decision.can_capture_audio());
        assert!(!decision.can_transcribe());
        assert!(!decision.can_synthesize_speech());
        assert!(!decision.can_search_or_route_tools());
        assert!(!decision.can_identify_authorize_or_mutate());
        assert_eq!(decision.provider_call_attempt_count, 0);
        assert_eq!(decision.provider_network_dispatch_count, 0);
    }

    #[test]
    fn stage_3b_provider_off_blocks_cloud_selection_and_dispatch_counts() {
        let provider_off = VoiceProviderRouteDecisionPacket::contract_only(
            "voice-route-stage3b-provider-off",
            VoiceProviderTaskKind::Tts,
            VoiceProviderPlatform::Iphone,
            "phone",
            "provider-off",
            VoiceProviderPrivacyMode::LocalOnly,
            VoiceProviderLatencyClass::Realtime,
            Some(lang("en-US")),
            None,
            false,
            vec![
                selection("apple-platform-tts", VoiceProviderKind::ApplePlatform),
                selection("openai-speech-tts", VoiceProviderKind::OpenAiSpeech),
            ],
            Some(selection(
                "apple-platform-tts",
                VoiceProviderKind::ApplePlatform,
            )),
            None,
            Some(VoiceProviderFallbackReason::ProviderDisabled),
            true,
            "cloud-budget-blocked",
            Some("consent-provider-voice".to_string()),
            Some("provider-budget-stage3b".to_string()),
            "audit-stage3b-provider-off",
            VoiceProviderQualitySignal::ready_contract_only(true, true, true, false, false),
        )
        .unwrap();
        assert!(!provider_off.selected_cloud_provider());
        assert_eq!(provider_off.provider_call_attempt_count, 0);
        assert_eq!(provider_off.provider_network_dispatch_count, 0);

        let rejected_cloud = VoiceProviderRouteDecisionPacket::contract_only(
            "voice-route-stage3b-provider-off-cloud",
            VoiceProviderTaskKind::Stt,
            VoiceProviderPlatform::Mac,
            "desktop",
            "provider-off",
            VoiceProviderPrivacyMode::LocalOnly,
            VoiceProviderLatencyClass::Realtime,
            Some(lang("en-US")),
            None,
            false,
            vec![selection(
                "openai-realtime-stt",
                VoiceProviderKind::OpenAiRealtime,
            )],
            Some(selection(
                "openai-realtime-stt",
                VoiceProviderKind::OpenAiRealtime,
            )),
            None,
            Some(VoiceProviderFallbackReason::ProviderDisabled),
            true,
            "cloud-budget-blocked",
            Some("consent-provider-voice".to_string()),
            Some("provider-budget-stage3b".to_string()),
            "audit-stage3b-provider-off-cloud",
            VoiceProviderQualitySignal::ready_contract_only(true, true, true, false, false),
        );
        assert!(rejected_cloud.is_err());
    }

    #[test]
    fn stage_3b_route_decision_rejects_attempt_or_dispatch_counts() {
        let mut decision = VoiceProviderRouteDecisionPacket::contract_only(
            "voice-route-stage3b-bad-count",
            VoiceProviderTaskKind::Stt,
            VoiceProviderPlatform::Server,
            "server",
            "online",
            VoiceProviderPrivacyMode::ProviderCloud,
            VoiceProviderLatencyClass::NearRealtime,
            Some(lang("en-US")),
            Some(9_500),
            true,
            vec![selection(
                "openai-realtime-stt",
                VoiceProviderKind::OpenAiRealtime,
            )],
            Some(selection(
                "openai-realtime-stt",
                VoiceProviderKind::OpenAiRealtime,
            )),
            None,
            Some(VoiceProviderFallbackReason::ProtectedSlotRequiresSecondPass),
            false,
            "budget-available",
            Some("consent-provider-voice".to_string()),
            Some("provider-budget-stage3b".to_string()),
            "audit-stage3b-bad-count",
            VoiceProviderQualitySignal::ready_contract_only(true, true, true, true, true),
        )
        .unwrap();

        decision.provider_call_attempt_count = 1;
        assert!(decision.validate().is_err());
        decision.provider_call_attempt_count = 0;
        decision.provider_network_dispatch_count = 1;
        assert!(decision.validate().is_err());
    }
}
