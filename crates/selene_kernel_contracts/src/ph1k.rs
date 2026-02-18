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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptPhraseId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterruptPhraseSetVersion(pub u32);

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

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptCandidate {
    pub schema_version: SchemaVersion,
    pub phrase_set_version: InterruptPhraseSetVersion,
    pub phrase_id: InterruptPhraseId,
    pub phrase_text: String,
    pub phrase_confidence: Confidence,
    pub gates: InterruptGates,
    pub gate_confidences: InterruptGateConfidences,
    pub t_event: MonotonicTimeNs,
    pub reason_code: ReasonCodeId,
}

impl InterruptCandidate {
    pub fn v1(
        phrase_set_version: InterruptPhraseSetVersion,
        phrase_id: InterruptPhraseId,
        phrase_text: String,
        phrase_confidence: Confidence,
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
        let normalized = normalize_ascii_phrase(&phrase_text);
        if normalized.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_text",
                reason: "must not be empty",
            });
        }
        if !normalized.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_text",
                reason: "must be ASCII normalized text",
            });
        }
        if normalized.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_candidate.phrase_text",
                reason: "must be <= 128 chars",
            });
        }
        Ok(Self {
            schema_version: PH1K_CONTRACT_VERSION,
            phrase_set_version,
            phrase_id,
            phrase_text: normalized,
            phrase_confidence,
            gates,
            gate_confidences,
            t_event,
            reason_code,
        })
    }
}

fn normalize_ascii_phrase(s: &str) -> String {
    s.split_whitespace()
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
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
}
