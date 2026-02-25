#![forbid(unsafe_code)]

use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1K_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1K_ENGINE_ID: &str = "PH1.K";
pub const PH1K_IMPLEMENTATION_ID: &str = "PH1.K.001";
pub const PH1K_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1K_IMPLEMENTATION_ID];
pub const PH1K_CANONICAL_PROCESSED_SAMPLE_RATE_HZ: SampleRateHz = SampleRateHz(16_000);
pub const PH1K_CANONICAL_PROCESSED_CHANNELS: ChannelCount = ChannelCount(1);
pub const PH1K_CANONICAL_PROCESSED_SAMPLE_FORMAT: SampleFormat = SampleFormat::PcmF32LE;
pub const PH1K_CANONICAL_PROCESSED_FRAME_MS: FrameDurationMs = FrameDurationMs::Ms20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1kImplementation {
    V001,
}

impl Ph1kImplementation {
    pub const fn id(self) -> &'static str {
        match self {
            Ph1kImplementation::V001 => PH1K_IMPLEMENTATION_ID,
        }
    }

    pub fn parse(implementation_id: &str) -> Result<Self, ContractViolation> {
        match implementation_id {
            PH1K_IMPLEMENTATION_ID => Ok(Ph1kImplementation::V001),
            _ => Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioStreamId(pub u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AudioSeqNo(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SampleRateHz(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChannelCount(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SampleFormat {
    PcmS16LE,
    PcmF32LE,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameDurationMs {
    Ms10,
    Ms20,
}

impl FrameDurationMs {
    pub fn as_u16(self) -> u16 {
        match self {
            FrameDurationMs::Ms10 => 10,
            FrameDurationMs::Ms20 => 20,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    pub fn new(value: f32) -> Result<Self, ContractViolation> {
        if !value.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "confidence",
            });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ContractViolation::InvalidRange {
                field: "confidence",
                min: 0.0,
                max: 1.0,
                got: value as f64,
            });
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AudioDeviceId(String);

impl AudioDeviceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "audio_device_id",
                reason: "must not be empty",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioStreamKind {
    MicProcessed,
    MicRaw,
    SpeakerPlayback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioFormat {
    pub sample_rate_hz: SampleRateHz,
    pub channels: ChannelCount,
    pub sample_format: SampleFormat,
}

impl Validate for AudioFormat {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.sample_rate_hz.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "sample_rate_hz",
                reason: "must be > 0",
            });
        }
        if self.channels.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "channels",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioStreamRef {
    pub schema_version: SchemaVersion,
    pub stream_id: AudioStreamId,
    pub kind: AudioStreamKind,
    pub format: AudioFormat,
    pub frame_ms: FrameDurationMs,
}

impl AudioStreamRef {
    pub fn v1(
        stream_id: AudioStreamId,
        kind: AudioStreamKind,
        format: AudioFormat,
        frame_ms: FrameDurationMs,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            stream_id,
            kind,
            format,
            frame_ms,
        }
    }
}

impl Validate for AudioStreamRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.format.validate()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PreRollBufferId(pub u64);

/// Rolling "always-on" pre-roll window maintained by PH1.K.
///
/// This is a reference, not the audio itself. Wake (PH1.W) uses it to guarantee the first
/// syllable isn't missed when a wake candidate begins mid-buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PreRollBufferRef {
    pub schema_version: SchemaVersion,
    pub buffer_id: PreRollBufferId,
    pub stream_id: AudioStreamId,
    pub t_start: MonotonicTimeNs,
    pub t_end: MonotonicTimeNs,
}

impl PreRollBufferRef {
    pub fn v1(
        buffer_id: PreRollBufferId,
        stream_id: AudioStreamId,
        t_start: MonotonicTimeNs,
        t_end: MonotonicTimeNs,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            buffer_id,
            stream_id,
            t_start,
            t_end,
        }
    }

    pub fn duration_ns(&self) -> u64 {
        self.t_end.0.saturating_sub(self.t_start.0)
    }
}

impl Validate for PreRollBufferRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.t_end.0 < self.t_start.0 {
            return Err(ContractViolation::InvalidValue {
                field: "pre_roll_buffer_ref.t_end",
                reason: "must be >= t_start",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioBytesRef {
    pub store_id: u64,
    pub byte_offset: u64,
    pub byte_len: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioPayload {
    Inline(Vec<u8>),
    Ref(AudioBytesRef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioFrame {
    pub schema_version: SchemaVersion,
    pub stream_id: AudioStreamId,
    pub seq_no: AudioSeqNo,
    pub t_capture: MonotonicTimeNs,
    pub format: AudioFormat,
    pub frame_ms: FrameDurationMs,
    pub payload: AudioPayload,
}

impl AudioFrame {
    pub fn v1(
        stream_id: AudioStreamId,
        seq_no: AudioSeqNo,
        t_capture: MonotonicTimeNs,
        format: AudioFormat,
        frame_ms: FrameDurationMs,
        payload: AudioPayload,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            stream_id,
            seq_no,
            t_capture,
            format,
            frame_ms,
            payload,
        }
    }

    pub fn expected_payload_bytes(&self) -> Result<usize, ContractViolation> {
        expected_payload_bytes(self.format, self.frame_ms)
    }
}

impl Validate for AudioFrame {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.format.validate()?;
        // Frame size is intentionally restricted to 10ms or 20ms for determinism.
        let _ = self.frame_ms.as_u16();
        let expected_bytes = self.expected_payload_bytes()?;

        match &self.payload {
            AudioPayload::Inline(bytes) => {
                if bytes.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "audio_frame.payload.inline",
                        reason: "must not be empty",
                    });
                }
                if bytes.len() != expected_bytes {
                    return Err(ContractViolation::InvalidValue {
                        field: "audio_frame.payload.inline",
                        reason: "size must match expected_bytes for format+frame_ms",
                    });
                }
            }
            AudioPayload::Ref(r) => {
                if r.byte_len == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "audio_frame.payload.ref.byte_len",
                        reason: "must be > 0",
                    });
                }
                if usize::try_from(r.byte_len).ok() != Some(expected_bytes) {
                    return Err(ContractViolation::InvalidValue {
                        field: "audio_frame.payload.ref.byte_len",
                        reason: "must match expected_bytes for format+frame_ms",
                    });
                }
            }
        }

        Ok(())
    }
}

fn expected_payload_bytes(
    format: AudioFormat,
    frame_ms: FrameDurationMs,
) -> Result<usize, ContractViolation> {
    let sr = u64::from(format.sample_rate_hz.0);
    let frame_ms_u64 = u64::from(frame_ms.as_u16());
    let samples_per_channel_times_ms = sr.saturating_mul(frame_ms_u64);
    if samples_per_channel_times_ms % 1_000 != 0 {
        return Err(ContractViolation::InvalidValue {
            field: "audio_frame.frame_ms",
            reason: "sample_rate_hz * frame_ms must be divisible by 1000",
        });
    }
    let samples_per_channel = samples_per_channel_times_ms / 1_000;
    let channels = u64::from(format.channels.0);
    let bytes_per_sample = match format.sample_format {
        SampleFormat::PcmS16LE => 2u64,
        SampleFormat::PcmF32LE => 4u64,
    };
    let total = samples_per_channel
        .saturating_mul(channels)
        .saturating_mul(bytes_per_sample);
    usize::try_from(total).map_err(|_| ContractViolation::InvalidValue {
        field: "audio_frame.payload",
        reason: "expected byte size overflow",
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpeechLikeness(pub f32);

impl SpeechLikeness {
    pub fn new(value: f32) -> Result<Self, ContractViolation> {
        if !value.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "speech_likeness",
            });
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ContractViolation::InvalidRange {
                field: "speech_likeness",
                min: 0.0,
                max: 1.0,
                got: value as f64,
            });
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VadEvent {
    pub schema_version: SchemaVersion,
    pub stream_id: AudioStreamId,
    pub t_start: MonotonicTimeNs,
    pub t_end: MonotonicTimeNs,
    pub confidence: Confidence,
    pub speech_likeness: SpeechLikeness,
}

impl VadEvent {
    pub fn v1(
        stream_id: AudioStreamId,
        t_start: MonotonicTimeNs,
        t_end: MonotonicTimeNs,
        confidence: Confidence,
        speech_likeness: SpeechLikeness,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            stream_id,
            t_start,
            t_end,
            confidence,
            speech_likeness,
        }
    }
}

impl Validate for VadEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.t_end.0 < self.t_start.0 {
            return Err(ContractViolation::InvalidValue {
                field: "vad_event.t_end",
                reason: "must be >= t_start",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceHealth {
    Healthy,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeviceRoute {
    BuiltIn,
    Usb,
    Bluetooth,
    Virtual,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceError {
    pub code: ReasonCodeId,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceState {
    pub schema_version: SchemaVersion,
    pub selected_mic: AudioDeviceId,
    pub selected_speaker: AudioDeviceId,
    pub device_route: DeviceRoute,
    pub health: DeviceHealth,
    pub errors: Vec<DeviceError>,
}

impl DeviceState {
    pub fn v1(
        selected_mic: AudioDeviceId,
        selected_speaker: AudioDeviceId,
        health: DeviceHealth,
        errors: Vec<DeviceError>,
    ) -> Self {
        Self::v1_with_route(
            selected_mic,
            selected_speaker,
            DeviceRoute::Unknown,
            health,
            errors,
        )
    }

    pub fn v1_with_route(
        selected_mic: AudioDeviceId,
        selected_speaker: AudioDeviceId,
        device_route: DeviceRoute,
        health: DeviceHealth,
        errors: Vec<DeviceError>,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            selected_mic,
            selected_speaker,
            device_route,
            health,
            errors,
        }
    }
}

impl Validate for DeviceState {
    fn validate(&self) -> Result<(), ContractViolation> {
        // `AudioDeviceId` construction already enforces non-empty, so this is mostly structural.
        for e in &self.errors {
            if e.message.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "device_state.errors[].message",
                    reason: "must not be empty",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimingStats {
    pub schema_version: SchemaVersion,
    pub jitter_ms: f32,
    pub drift_ppm: f32,
    pub buffer_depth_ms: f32,
    pub underruns: u64,
    pub overruns: u64,
}

impl TimingStats {
    pub fn v1(
        jitter_ms: f32,
        drift_ppm: f32,
        buffer_depth_ms: f32,
        underruns: u64,
        overruns: u64,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            jitter_ms,
            drift_ppm,
            buffer_depth_ms,
            underruns,
            overruns,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ph1kState {
    Init,
    Ready,
    FullDuplexActive,
    DeviceSwitching,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StateTransitionEvent {
    pub schema_version: SchemaVersion,
    pub from_state: Ph1kState,
    pub to_state: Ph1kState,
    pub t_event: MonotonicTimeNs,
    pub reason_code: ReasonCodeId,
}

impl StateTransitionEvent {
    pub fn v1(
        from_state: Ph1kState,
        to_state: Ph1kState,
        t_event: MonotonicTimeNs,
        reason_code: ReasonCodeId,
    ) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            from_state,
            to_state,
            t_event,
            reason_code,
        }
    }
}

impl Validate for TimingStats {
    fn validate(&self) -> Result<(), ContractViolation> {
        for (field, v) in [
            ("timing_stats.jitter_ms", self.jitter_ms),
            ("timing_stats.drift_ppm", self.drift_ppm),
            ("timing_stats.buffer_depth_ms", self.buffer_depth_ms),
        ] {
            if !v.is_finite() {
                return Err(ContractViolation::NotFinite { field });
            }
            if v < 0.0 {
                return Err(ContractViolation::InvalidValue {
                    field,
                    reason: "must be >= 0",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdvancedAudioQualityMetrics {
    pub snr_db: f32,
    pub clipping_ratio: f32,
    pub echo_delay_ms: f32,
    pub packet_loss_pct: f32,
    pub double_talk_score: f32,
    pub erle_db: f32,
}

impl AdvancedAudioQualityMetrics {
    pub fn v1(
        snr_db: f32,
        clipping_ratio: f32,
        echo_delay_ms: f32,
        packet_loss_pct: f32,
        double_talk_score: f32,
        erle_db: f32,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            snr_db,
            clipping_ratio,
            echo_delay_ms,
            packet_loss_pct,
            double_talk_score,
            erle_db,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for AdvancedAudioQualityMetrics {
    fn validate(&self) -> Result<(), ContractViolation> {
        for (field, value) in [
            ("advanced_audio_quality_metrics.snr_db", self.snr_db),
            (
                "advanced_audio_quality_metrics.clipping_ratio",
                self.clipping_ratio,
            ),
            (
                "advanced_audio_quality_metrics.echo_delay_ms",
                self.echo_delay_ms,
            ),
            (
                "advanced_audio_quality_metrics.packet_loss_pct",
                self.packet_loss_pct,
            ),
            (
                "advanced_audio_quality_metrics.double_talk_score",
                self.double_talk_score,
            ),
            ("advanced_audio_quality_metrics.erle_db", self.erle_db),
        ] {
            if !value.is_finite() {
                return Err(ContractViolation::NotFinite { field });
            }
        }
        if !(-20.0..=80.0).contains(&self.snr_db) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.snr_db",
                min: -20.0,
                max: 80.0,
                got: self.snr_db as f64,
            });
        }
        if !(0.0..=1.0).contains(&self.clipping_ratio) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.clipping_ratio",
                min: 0.0,
                max: 1.0,
                got: self.clipping_ratio as f64,
            });
        }
        if !(0.0..=2_000.0).contains(&self.echo_delay_ms) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.echo_delay_ms",
                min: 0.0,
                max: 2_000.0,
                got: self.echo_delay_ms as f64,
            });
        }
        if !(0.0..=100.0).contains(&self.packet_loss_pct) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.packet_loss_pct",
                min: 0.0,
                max: 100.0,
                got: self.packet_loss_pct as f64,
            });
        }
        if !(0.0..=1.0).contains(&self.double_talk_score) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.double_talk_score",
                min: 0.0,
                max: 1.0,
                got: self.double_talk_score as f64,
            });
        }
        if !(0.0..=80.0).contains(&self.erle_db) {
            return Err(ContractViolation::InvalidRange {
                field: "advanced_audio_quality_metrics.erle_db",
                min: 0.0,
                max: 80.0,
                got: self.erle_db as f64,
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeviceReliabilityScoreInput {
    pub failures_24h: u32,
    pub recoveries_24h: u32,
    pub mean_recovery_ms: u32,
    pub reliability_score: Confidence,
}

impl DeviceReliabilityScoreInput {
    pub fn v1(
        failures_24h: u32,
        recoveries_24h: u32,
        mean_recovery_ms: u32,
        reliability_score: Confidence,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            failures_24h,
            recoveries_24h,
            mean_recovery_ms,
            reliability_score,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for DeviceReliabilityScoreInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.failures_24h > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "device_reliability_score_input.failures_24h",
                reason: "must be <= 1_000_000",
            });
        }
        if self.recoveries_24h > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "device_reliability_score_input.recoveries_24h",
                reason: "must be <= 1_000_000",
            });
        }
        if self.mean_recovery_ms > 300_000 {
            return Err(ContractViolation::InvalidValue {
                field: "device_reliability_score_input.mean_recovery_ms",
                reason: "must be <= 300000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VadDecisionConfidenceBand {
    High,
    Medium,
    Low,
}

pub fn classify_vad_decision_confidence_band(
    vad_confidence: Confidence,
    speech_likeness: SpeechLikeness,
) -> VadDecisionConfidenceBand {
    if vad_confidence.0 >= 0.90 && speech_likeness.0 >= 0.85 {
        VadDecisionConfidenceBand::High
    } else if vad_confidence.0 >= 0.75 && speech_likeness.0 >= 0.65 {
        VadDecisionConfidenceBand::Medium
    } else {
        VadDecisionConfidenceBand::Low
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct JitterClockRecoveryPolicy {
    pub max_jitter_ms: f32,
    pub max_abs_drift_ppm: f32,
    pub max_handoff_latency_ms: u32,
}

impl JitterClockRecoveryPolicy {
    pub fn v1(
        max_jitter_ms: f32,
        max_abs_drift_ppm: f32,
        max_handoff_latency_ms: u32,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            max_jitter_ms,
            max_abs_drift_ppm,
            max_handoff_latency_ms,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for JitterClockRecoveryPolicy {
    fn validate(&self) -> Result<(), ContractViolation> {
        for (field, value) in [
            (
                "jitter_clock_recovery_policy.max_jitter_ms",
                self.max_jitter_ms,
            ),
            (
                "jitter_clock_recovery_policy.max_abs_drift_ppm",
                self.max_abs_drift_ppm,
            ),
        ] {
            if !value.is_finite() {
                return Err(ContractViolation::NotFinite { field });
            }
            if value <= 0.0 {
                return Err(ContractViolation::InvalidValue {
                    field,
                    reason: "must be > 0",
                });
            }
        }
        if self.max_handoff_latency_ms == 0 || self.max_handoff_latency_ms > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "jitter_clock_recovery_policy.max_handoff_latency_ms",
                reason: "must be in 1..=10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveThresholdPolicyInput {
    pub device_route: DeviceRoute,
    pub quality_metrics: AdvancedAudioQualityMetrics,
    pub device_reliability: DeviceReliabilityScoreInput,
    pub timing_stats: TimingStats,
    pub capture_to_handoff_latency_ms: u32,
}

impl Validate for AdaptiveThresholdPolicyInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.quality_metrics.validate()?;
        self.device_reliability.validate()?;
        self.timing_stats.validate()?;
        if self.capture_to_handoff_latency_ms == 0 || self.capture_to_handoff_latency_ms > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "adaptive_threshold_policy_input.capture_to_handoff_latency_ms",
                reason: "must be in 1..=10000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptPhraseId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptPhraseSetVersion(pub u32);

pub const PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT: &str = "interrupt_policy_default";
pub const PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT: &str = "tenant_interrupt_default";
pub const PH1K_INTERRUPT_LOCALE_TAG_DEFAULT: &str = "en-US";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterruptPolicyProfileId(String);

impl InterruptPolicyProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        validate_interrupt_profile_id("interrupt_policy_profile_id", &id)?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterruptTenantProfileId(String);

impl InterruptTenantProfileId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        validate_interrupt_profile_id("interrupt_tenant_profile_id", &id)?;
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterruptLocaleTag(String);

impl InterruptLocaleTag {
    pub fn new(locale_tag: impl Into<String>) -> Result<Self, ContractViolation> {
        let locale_tag = locale_tag.into();
        validate_interrupt_locale_tag("interrupt_locale_tag", &locale_tag)?;
        Ok(Self(locale_tag))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterruptLexiconPolicyBinding {
    pub policy_profile_id: InterruptPolicyProfileId,
    pub tenant_profile_id: InterruptTenantProfileId,
    pub locale_tag: InterruptLocaleTag,
}

impl InterruptLexiconPolicyBinding {
    pub fn v1(
        policy_profile_id: InterruptPolicyProfileId,
        tenant_profile_id: InterruptTenantProfileId,
        locale_tag: InterruptLocaleTag,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            policy_profile_id,
            tenant_profile_id,
            locale_tag,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for InterruptLexiconPolicyBinding {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_interrupt_profile_id(
            "interrupt_lexicon_policy_binding.policy_profile_id",
            self.policy_profile_id.as_str(),
        )?;
        validate_interrupt_profile_id(
            "interrupt_lexicon_policy_binding.tenant_profile_id",
            self.tenant_profile_id.as_str(),
        )?;
        validate_interrupt_locale_tag(
            "interrupt_lexicon_policy_binding.locale_tag",
            self.locale_tag.as_str(),
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptGates {
    pub vad_ok: bool,
    pub echo_safe_ok: bool,
    pub phrase_ok: bool,
    pub nearfield_ok: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterruptGateConfidences {
    pub vad_confidence: Confidence,
    pub speech_likeness: SpeechLikeness,
    pub echo_safe_confidence: Confidence,
    pub phrase_confidence: Confidence,
    pub nearfield_confidence: Option<Confidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptCandidateConfidenceBand {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CaptureQualityClass {
    Clear,
    Guarded,
    Degraded,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EchoRiskClass {
    Low,
    Elevated,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NetworkStabilityClass {
    Stable,
    Flaky,
    Unstable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecoverabilityClass {
    Fast,
    Guarded,
    Slow,
    FailoverRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DegradationClassBundle {
    pub capture_quality_class: CaptureQualityClass,
    pub echo_risk_class: EchoRiskClass,
    pub network_stability_class: NetworkStabilityClass,
    pub recoverability_class: RecoverabilityClass,
}

impl DegradationClassBundle {
    pub fn from_flags(
        capture_degraded: bool,
        aec_unstable: bool,
        device_changed: bool,
        stream_gap_detected: bool,
    ) -> Self {
        let capture_quality_class = if capture_degraded {
            if stream_gap_detected {
                CaptureQualityClass::Critical
            } else {
                CaptureQualityClass::Degraded
            }
        } else if aec_unstable {
            CaptureQualityClass::Guarded
        } else {
            CaptureQualityClass::Clear
        };
        let echo_risk_class = if aec_unstable {
            EchoRiskClass::High
        } else if capture_degraded {
            EchoRiskClass::Elevated
        } else {
            EchoRiskClass::Low
        };
        let network_stability_class = if stream_gap_detected {
            NetworkStabilityClass::Unstable
        } else if device_changed {
            NetworkStabilityClass::Flaky
        } else {
            NetworkStabilityClass::Stable
        };
        let recoverability_class = if stream_gap_detected || device_changed {
            RecoverabilityClass::FailoverRequired
        } else if capture_degraded || aec_unstable {
            RecoverabilityClass::Slow
        } else {
            RecoverabilityClass::Fast
        };
        Self {
            capture_quality_class,
            echo_risk_class,
            network_stability_class,
            recoverability_class,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptRiskContextClass {
    Low,
    Guarded,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptDegradationContext {
    pub capture_degraded: bool,
    pub aec_unstable: bool,
    pub device_changed: bool,
    pub stream_gap_detected: bool,
    pub class_bundle: DegradationClassBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptTimingMarkers {
    pub window_start: MonotonicTimeNs,
    pub window_end: MonotonicTimeNs,
}

impl Validate for InterruptTimingMarkers {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.window_end.0 < self.window_start.0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_timing_markers.window_end",
                reason: "must be >= window_start",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptSpeechWindowMetrics {
    pub voiced_window_ms: u32,
}

impl Validate for InterruptSpeechWindowMetrics {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.voiced_window_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_speech_window_metrics.voiced_window_ms",
                reason: "must be > 0",
            });
        }
        if self.voiced_window_ms > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_speech_window_metrics.voiced_window_ms",
                reason: "must be <= 10000ms",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InterruptSubjectRelationConfidenceBundle {
    pub lexical_confidence: Confidence,
    pub vad_confidence: Confidence,
    pub speech_likeness: SpeechLikeness,
    pub echo_safe_confidence: Confidence,
    pub nearfield_confidence: Option<Confidence>,
    pub combined_confidence: Confidence,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptCandidate {
    pub schema_version: SchemaVersion,
    pub phrase_set_version: InterruptPhraseSetVersion,
    pub phrase_id: InterruptPhraseId,
    pub trigger_phrase_id: InterruptPhraseId,
    pub trigger_locale: InterruptLocaleTag,
    pub phrase_text: String,
    pub phrase_confidence: Confidence,
    pub candidate_confidence_band: InterruptCandidateConfidenceBand,
    pub risk_context_class: InterruptRiskContextClass,
    pub degradation_context: InterruptDegradationContext,
    pub timing_markers: InterruptTimingMarkers,
    pub speech_window_metrics: InterruptSpeechWindowMetrics,
    pub subject_relation_confidence_bundle: InterruptSubjectRelationConfidenceBundle,
    pub gates: InterruptGates,
    pub gate_confidences: InterruptGateConfidences,
    pub t_event: MonotonicTimeNs,
    pub reason_code: ReasonCodeId,
}

impl InterruptCandidate {
    pub fn v1(
        phrase_set_version: InterruptPhraseSetVersion,
        phrase_id: InterruptPhraseId,
        trigger_phrase_id: InterruptPhraseId,
        trigger_locale: InterruptLocaleTag,
        phrase_text: String,
        phrase_confidence: Confidence,
        candidate_confidence_band: InterruptCandidateConfidenceBand,
        risk_context_class: InterruptRiskContextClass,
        degradation_context: InterruptDegradationContext,
        timing_markers: InterruptTimingMarkers,
        speech_window_metrics: InterruptSpeechWindowMetrics,
        subject_relation_confidence_bundle: InterruptSubjectRelationConfidenceBundle,
        gates: InterruptGates,
        gate_confidences: InterruptGateConfidences,
        t_event: MonotonicTimeNs,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        if phrase_set_version.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_set_version",
                reason: "must be > 0",
            });
        }
        let normalized = normalize_interrupt_phrase_with_field(
            "interrupt_candidate.phrase_text",
            &trigger_locale,
            &phrase_text,
        )?;
        if normalized.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_text",
                reason: "must not be empty",
            });
        }
        if normalized.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_text",
                reason: "must be <= 128 chars",
            });
        }
        if phrase_id.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_id",
                reason: "must be > 0",
            });
        }
        if trigger_phrase_id.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.trigger_phrase_id",
                reason: "must be > 0",
            });
        }
        if phrase_id != trigger_phrase_id {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.trigger_phrase_id",
                reason: "must match phrase_id",
            });
        }
        validate_interrupt_locale_tag(
            "interrupt_candidate.trigger_locale",
            trigger_locale.as_str(),
        )?;
        timing_markers.validate()?;
        speech_window_metrics.validate()?;
        if timing_markers.window_end.0 != t_event.0 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.timing_markers.window_end",
                reason: "must match t_event",
            });
        }
        let duration_ns = timing_markers
            .window_end
            .0
            .saturating_sub(timing_markers.window_start.0);
        let max_duration_ns =
            u64::from(speech_window_metrics.voiced_window_ms).saturating_mul(1_000_000);
        if duration_ns > max_duration_ns {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.timing_markers",
                reason: "window duration must be <= voiced_window_ms",
            });
        }
        if degradation_context.capture_degraded
            && matches!(
                degradation_context.class_bundle.capture_quality_class,
                CaptureQualityClass::Clear
            )
        {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.degradation_context.class_bundle.capture_quality_class",
                reason: "must not be CLEAR when capture_degraded=true",
            });
        }
        if degradation_context.aec_unstable
            && matches!(
                degradation_context.class_bundle.echo_risk_class,
                EchoRiskClass::Low
            )
        {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.degradation_context.class_bundle.echo_risk_class",
                reason: "must not be LOW when aec_unstable=true",
            });
        }
        if degradation_context.stream_gap_detected
            && matches!(
                degradation_context.class_bundle.network_stability_class,
                NetworkStabilityClass::Stable
            )
        {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.degradation_context.class_bundle.network_stability_class",
                reason: "must not be STABLE when stream_gap_detected=true",
            });
        }
        Ok(Self {
            schema_version: PH1K_CONTRACT_VERSION,
            phrase_set_version,
            phrase_id,
            trigger_phrase_id,
            trigger_locale,
            phrase_text: normalized,
            phrase_confidence,
            candidate_confidence_band,
            risk_context_class,
            degradation_context,
            timing_markers,
            speech_window_metrics,
            subject_relation_confidence_bundle,
            gates,
            gate_confidences,
            t_event,
            reason_code,
        })
    }
}

fn normalize_interrupt_phrase_with_field(
    field: &'static str,
    locale_tag: &InterruptLocaleTag,
    s: &str,
) -> Result<String, ContractViolation> {
    validate_interrupt_locale_tag("interrupt_phrase.locale_tag", locale_tag.as_str())?;
    let locale = locale_tag.as_str().to_ascii_lowercase();
    let locale_adjusted = if locale.starts_with("tr") || locale.starts_with("az") {
        // Turkish/Azeri dotted-I handling before lowercasing to keep canonical form stable.
        s.chars()
            .map(|ch| match ch {
                'I' => 'ı',
                'İ' => 'i',
                _ => ch,
            })
            .collect::<String>()
    } else {
        s.to_string()
    };
    let no_controls = locale_adjusted
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    let normalized = no_controls
        .split_whitespace()
        .map(|part| {
            part.chars()
                .flat_map(char::to_lowercase)
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ");
    if normalized.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if normalized.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 128 chars",
        });
    }
    Ok(normalized)
}

pub fn normalize_interrupt_phrase_for_locale(
    locale_tag: &InterruptLocaleTag,
    phrase_text: &str,
) -> Result<String, ContractViolation> {
    normalize_interrupt_phrase_with_field("interrupt_phrase.text", locale_tag, phrase_text)
}

fn validate_interrupt_profile_id(
    field: &'static str,
    value: &str,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must use [A-Za-z0-9_.-]",
        });
    }
    Ok(())
}

fn validate_interrupt_locale_tag(
    field: &'static str,
    value: &str,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 32 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 32 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must use [A-Za-z0-9_-]",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DegradationFlags {
    pub capture_degraded: bool,
    pub aec_unstable: bool,
    pub device_changed: bool,
    pub stream_gap_detected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TtsPlaybackActiveEvent {
    pub schema_version: SchemaVersion,
    pub active: bool,
    pub t_event: MonotonicTimeNs,
}

impl TtsPlaybackActiveEvent {
    pub fn v1(active: bool, t_event: MonotonicTimeNs) -> Self {
        Self {
            schema_version: PH1K_CONTRACT_VERSION,
            active,
            t_event,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_rejects_out_of_range() {
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.0).is_ok());
    }

    #[test]
    fn vad_event_requires_non_negative_duration() {
        let ev = VadEvent::v1(
            AudioStreamId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(9),
            Confidence::new(0.5).unwrap(),
            SpeechLikeness::new(0.5).unwrap(),
        );
        assert!(ev.validate().is_err());
    }

    #[test]
    fn audio_frame_inline_payload_must_not_be_empty() {
        let frame = AudioFrame::v1(
            AudioStreamId(1),
            AudioSeqNo(1),
            MonotonicTimeNs(1),
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms10,
            AudioPayload::Inline(vec![]),
        );
        assert!(frame.validate().is_err());
    }

    #[test]
    fn audio_frame_payload_must_match_expected_bytes() {
        let bad = AudioFrame::v1(
            AudioStreamId(1),
            AudioSeqNo(1),
            MonotonicTimeNs(1),
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
            AudioPayload::Inline(vec![0; 10]),
        );
        assert!(bad.validate().is_err());

        let good = AudioFrame::v1(
            AudioStreamId(1),
            AudioSeqNo(2),
            MonotonicTimeNs(2),
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
            AudioPayload::Inline(vec![0; 640]),
        );
        assert!(good.validate().is_ok());
    }

    #[test]
    fn implementation_id_lock_is_v001() {
        assert_eq!(PH1K_ENGINE_ID, "PH1.K");
        assert_eq!(PH1K_IMPLEMENTATION_ID, "PH1.K.001");
        assert_eq!(PH1K_ACTIVE_IMPLEMENTATION_IDS, &["PH1.K.001"]);
        assert_eq!(Ph1kImplementation::V001.id(), PH1K_IMPLEMENTATION_ID);
    }

    #[test]
    fn implementation_id_parser_fails_closed_on_unknown_values() {
        assert_eq!(
            Ph1kImplementation::parse("PH1.K.001").unwrap(),
            Ph1kImplementation::V001
        );
        assert!(matches!(
            Ph1kImplementation::parse("PH1.K.999"),
            Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn interrupt_lexicon_policy_binding_accepts_valid_values() {
        let binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new(PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT).unwrap(),
            InterruptTenantProfileId::new(PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT).unwrap(),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
        )
        .unwrap();
        assert_eq!(
            binding.policy_profile_id.as_str(),
            PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT
        );
        assert_eq!(
            binding.tenant_profile_id.as_str(),
            PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT
        );
        assert_eq!(
            binding.locale_tag.as_str(),
            PH1K_INTERRUPT_LOCALE_TAG_DEFAULT
        );
    }

    #[test]
    fn interrupt_lexicon_policy_binding_rejects_invalid_profile_id() {
        assert!(matches!(
            InterruptPolicyProfileId::new(""),
            Err(ContractViolation::InvalidValue {
                field: "interrupt_policy_profile_id",
                reason: "must not be empty",
            })
        ));
        assert!(InterruptTenantProfileId::new("tenant*bad").is_err());
    }

    #[test]
    fn interrupt_lexicon_policy_binding_rejects_invalid_locale_tag() {
        assert!(matches!(
            InterruptLocaleTag::new("en US"),
            Err(ContractViolation::InvalidValue {
                field: "interrupt_locale_tag",
                reason: "must use [A-Za-z0-9_-]",
            })
        ));
    }

    #[test]
    fn interrupt_candidate_step2_payload_accepts_valid_values() {
        let candidate = InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(3),
            InterruptPhraseId(3),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
            "wait".to_string(),
            Confidence::new(0.95).unwrap(),
            InterruptCandidateConfidenceBand::High,
            InterruptRiskContextClass::Low,
            InterruptDegradationContext {
                capture_degraded: false,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
                class_bundle: DegradationClassBundle::from_flags(false, false, false, false),
            },
            InterruptTimingMarkers {
                window_start: MonotonicTimeNs(0),
                window_end: MonotonicTimeNs(1),
            },
            InterruptSpeechWindowMetrics {
                voiced_window_ms: 1,
            },
            InterruptSubjectRelationConfidenceBundle {
                lexical_confidence: Confidence::new(0.95).unwrap(),
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
                combined_confidence: Confidence::new(0.92).unwrap(),
            },
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            InterruptGateConfidences {
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                phrase_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
        )
        .unwrap();
        assert_eq!(candidate.trigger_phrase_id.0, 3);
        assert_eq!(
            candidate.trigger_locale.as_str(),
            PH1K_INTERRUPT_LOCALE_TAG_DEFAULT
        );
        assert_eq!(
            candidate.candidate_confidence_band,
            InterruptCandidateConfidenceBand::High
        );
    }

    #[test]
    fn interrupt_candidate_rejects_trigger_phrase_id_mismatch() {
        let out = InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(3),
            InterruptPhraseId(2),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
            "wait".to_string(),
            Confidence::new(0.95).unwrap(),
            InterruptCandidateConfidenceBand::High,
            InterruptRiskContextClass::Low,
            InterruptDegradationContext {
                capture_degraded: false,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
                class_bundle: DegradationClassBundle::from_flags(false, false, false, false),
            },
            InterruptTimingMarkers {
                window_start: MonotonicTimeNs(0),
                window_end: MonotonicTimeNs(1),
            },
            InterruptSpeechWindowMetrics {
                voiced_window_ms: 1,
            },
            InterruptSubjectRelationConfidenceBundle {
                lexical_confidence: Confidence::new(0.95).unwrap(),
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
                combined_confidence: Confidence::new(0.92).unwrap(),
            },
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            InterruptGateConfidences {
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                phrase_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.trigger_phrase_id",
                reason: "must match phrase_id",
            })
        ));
    }

    #[test]
    fn interrupt_candidate_accepts_and_normalizes_unicode_phrase() {
        let candidate = InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(3),
            InterruptPhraseId(3),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
            "ÉCHO STOP".to_string(),
            Confidence::new(0.95).unwrap(),
            InterruptCandidateConfidenceBand::High,
            InterruptRiskContextClass::Low,
            InterruptDegradationContext {
                capture_degraded: false,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
                class_bundle: DegradationClassBundle::from_flags(false, false, false, false),
            },
            InterruptTimingMarkers {
                window_start: MonotonicTimeNs(0),
                window_end: MonotonicTimeNs(1),
            },
            InterruptSpeechWindowMetrics {
                voiced_window_ms: 1,
            },
            InterruptSubjectRelationConfidenceBundle {
                lexical_confidence: Confidence::new(0.95).unwrap(),
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
                combined_confidence: Confidence::new(0.92).unwrap(),
            },
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            InterruptGateConfidences {
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                phrase_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
        )
        .unwrap();

        assert_eq!(candidate.phrase_text, "écho stop");
    }

    #[test]
    fn advanced_audio_quality_metrics_rejects_out_of_range_values() {
        assert!(AdvancedAudioQualityMetrics::v1(-30.0, 0.0, 10.0, 0.0, 0.2, 10.0).is_err());
        assert!(AdvancedAudioQualityMetrics::v1(20.0, 1.1, 10.0, 0.0, 0.2, 10.0).is_err());
        assert!(AdvancedAudioQualityMetrics::v1(20.0, 0.1, 10.0, 5.0, 0.2, 10.0).is_ok());
    }

    #[test]
    fn device_reliability_score_input_rejects_invalid_recovery_window() {
        let bad = DeviceReliabilityScoreInput::v1(10, 9, 300_001, Confidence::new(0.8).unwrap());
        assert!(bad.is_err());
        assert!(
            DeviceReliabilityScoreInput::v1(10, 9, 1200, Confidence::new(0.8).unwrap()).is_ok()
        );
    }

    #[test]
    fn classify_vad_decision_confidence_band_is_deterministic() {
        assert_eq!(
            classify_vad_decision_confidence_band(
                Confidence::new(0.95).unwrap(),
                SpeechLikeness::new(0.90).unwrap()
            ),
            VadDecisionConfidenceBand::High
        );
        assert_eq!(
            classify_vad_decision_confidence_band(
                Confidence::new(0.80).unwrap(),
                SpeechLikeness::new(0.70).unwrap()
            ),
            VadDecisionConfidenceBand::Medium
        );
        assert_eq!(
            classify_vad_decision_confidence_band(
                Confidence::new(0.60).unwrap(),
                SpeechLikeness::new(0.40).unwrap()
            ),
            VadDecisionConfidenceBand::Low
        );
    }

    #[test]
    fn locale_aware_normalization_handles_turkish_i() {
        let tr = InterruptLocaleTag::new("tr-TR").unwrap();
        let norm = normalize_interrupt_phrase_for_locale(&tr, "I İSTANBUL   DUR").unwrap();
        assert_eq!(norm, "ı istanbul dur");
    }

    #[test]
    fn normalize_interrupt_phrase_strips_controls_and_collapses_whitespace() {
        let en = InterruptLocaleTag::new("en-US").unwrap();
        let norm = normalize_interrupt_phrase_for_locale(&en, "  ÉCHO\u{0000}\n\t STOP   ").unwrap();
        assert_eq!(norm, "écho stop");
    }

    #[test]
    fn normalize_interrupt_phrase_rejects_control_only_input_fail_closed() {
        let en = InterruptLocaleTag::new("en-US").unwrap();
        let err = normalize_interrupt_phrase_for_locale(&en, "\u{0000}\u{0007}\n\t")
            .expect_err("control-only interrupt phrase must fail closed");
        assert!(matches!(
            err,
            ContractViolation::InvalidValue {
                field: "interrupt_phrase.text",
                reason: "must not be empty",
            }
        ));
    }

    #[test]
    fn degradation_class_bundle_from_flags_is_deterministic() {
        let clean = DegradationClassBundle::from_flags(false, false, false, false);
        assert_eq!(clean.capture_quality_class, CaptureQualityClass::Clear);
        assert_eq!(clean.echo_risk_class, EchoRiskClass::Low);
        assert_eq!(
            clean.network_stability_class,
            NetworkStabilityClass::Stable
        );
        assert_eq!(clean.recoverability_class, RecoverabilityClass::Fast);

        let severe = DegradationClassBundle::from_flags(true, true, true, true);
        assert_eq!(severe.capture_quality_class, CaptureQualityClass::Critical);
        assert_eq!(severe.echo_risk_class, EchoRiskClass::High);
        assert_eq!(
            severe.network_stability_class,
            NetworkStabilityClass::Unstable
        );
        assert_eq!(
            severe.recoverability_class,
            RecoverabilityClass::FailoverRequired
        );
    }

    #[test]
    fn interrupt_candidate_rejects_inconsistent_degradation_class_bundle() {
        let out = InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(3),
            InterruptPhraseId(3),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
            "wait".to_string(),
            Confidence::new(0.95).unwrap(),
            InterruptCandidateConfidenceBand::High,
            InterruptRiskContextClass::Low,
            InterruptDegradationContext {
                capture_degraded: true,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
                class_bundle: DegradationClassBundle {
                    capture_quality_class: CaptureQualityClass::Clear,
                    echo_risk_class: EchoRiskClass::Low,
                    network_stability_class: NetworkStabilityClass::Stable,
                    recoverability_class: RecoverabilityClass::Fast,
                },
            },
            InterruptTimingMarkers {
                window_start: MonotonicTimeNs(0),
                window_end: MonotonicTimeNs(1),
            },
            InterruptSpeechWindowMetrics {
                voiced_window_ms: 1,
            },
            InterruptSubjectRelationConfidenceBundle {
                lexical_confidence: Confidence::new(0.95).unwrap(),
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
                combined_confidence: Confidence::new(0.92).unwrap(),
            },
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            InterruptGateConfidences {
                vad_confidence: Confidence::new(0.90).unwrap(),
                speech_likeness: SpeechLikeness::new(0.90).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                phrase_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.degradation_context.class_bundle.capture_quality_class",
                ..
            })
        ));
    }
}
