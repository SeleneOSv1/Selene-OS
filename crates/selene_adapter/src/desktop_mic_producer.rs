#![forbid(unsafe_code)]

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};

use crate::VoiceTurnAudioCaptureRef;

const TARGET_SAMPLE_RATE_HZ: u32 = 16_000;
const FRAME_DURATION_MS: u32 = 20;
const MIN_RING_BUFFER_MS: u64 = 3_000;
const MIN_PRE_ROLL_MS: u64 = 1_200;

#[derive(Debug, Clone)]
pub struct DesktopMicProducerConfig {
    pub input_device_name_substring: Option<String>,
    pub locale_tag: String,
    pub ring_buffer_ms: u64,
    pub pre_roll_ms: u64,
}

impl Default for DesktopMicProducerConfig {
    fn default() -> Self {
        Self {
            input_device_name_substring: None,
            locale_tag: "en-US".to_string(),
            ring_buffer_ms: MIN_RING_BUFFER_MS,
            pre_roll_ms: MIN_PRE_ROLL_MS,
        }
    }
}

#[derive(Debug)]
struct CaptureState {
    stream_id: u128,
    next_pre_roll_buffer_id: u64,
    locale_tag: String,
    selected_mic: String,
    selected_speaker: String,
    device_route: String,

    source_sample_rate_hz: u32,
    source_channels: u16,
    started_ns: u64,
    started_instant: Instant,

    ring_capacity_samples: usize,
    pre_roll_samples: usize,
    ring: VecDeque<i16>,

    resample_accumulator: f64,
    total_input_frames: u64,
    total_output_samples: u64,

    clipped_output_samples: u64,
    output_samples_seen_for_metrics: u64,
    rms_ema: f64,
    noise_floor_ema: f64,
    speech_likeness_ema: f64,
    frame_energy_accum: f64,
    frame_sample_count: usize,
    vad_confidence_ema: f64,
    noise_floor_reference_energy: f64,
    noise_floor_min_energy: f64,
    speech_energy_ema: f64,
    calibration_samples_remaining: u64,

    callback_count: u64,
    last_callback_instant: Option<Instant>,
    timing_jitter_ema_ms: f64,
    timing_drift_ema_ppm: f64,
    stream_gap_detected: bool,
    timing_underruns: u64,
    timing_overruns: u64,

    device_failures_24h: u32,
    device_recoveries_24h: u32,
    device_mean_recovery_ms: u32,

    aec_unstable: bool,
    device_changed: bool,
    capture_degraded: bool,

    last_error: Option<String>,
}

impl CaptureState {
    fn new(
        locale_tag: String,
        selected_mic: String,
        selected_speaker: String,
        device_route: String,
        source_sample_rate_hz: u32,
        source_channels: u16,
        ring_capacity_samples: usize,
        pre_roll_samples: usize,
    ) -> Result<Self, String> {
        let started_ns = now_ns();
        let started_instant = Instant::now();
        let stream_id_seed = format!(
            "desktop:{}:{}:{}",
            selected_mic, started_ns, source_sample_rate_hz
        );
        let stream_id = stable_hash_u128(&stream_id_seed).max(1);
        if ring_capacity_samples < pre_roll_samples {
            return Err("ring_capacity_samples must be >= pre_roll_samples".to_string());
        }
        Ok(Self {
            stream_id,
            next_pre_roll_buffer_id: 1,
            locale_tag,
            selected_mic,
            selected_speaker,
            device_route,
            source_sample_rate_hz,
            source_channels,
            started_ns,
            started_instant,
            ring_capacity_samples,
            pre_roll_samples,
            ring: VecDeque::with_capacity(ring_capacity_samples),
            resample_accumulator: 0.0,
            total_input_frames: 0,
            total_output_samples: 0,
            clipped_output_samples: 0,
            output_samples_seen_for_metrics: 0,
            rms_ema: 0.0,
            noise_floor_ema: 0.0,
            speech_likeness_ema: 0.0,
            frame_energy_accum: 0.0,
            frame_sample_count: 0,
            vad_confidence_ema: 0.0,
            noise_floor_reference_energy: 1e-8,
            noise_floor_min_energy: f64::MAX,
            speech_energy_ema: 1e-8,
            calibration_samples_remaining: (TARGET_SAMPLE_RATE_HZ as u64).saturating_mul(2),
            callback_count: 0,
            last_callback_instant: None,
            timing_jitter_ema_ms: 0.0,
            timing_drift_ema_ppm: 0.0,
            stream_gap_detected: false,
            timing_underruns: 0,
            timing_overruns: 0,
            device_failures_24h: 0,
            device_recoveries_24h: 0,
            device_mean_recovery_ms: 100,
            aec_unstable: false,
            device_changed: false,
            capture_degraded: false,
            last_error: None,
        })
    }

    fn on_stream_error(&mut self, reason: String) {
        self.device_failures_24h = self.device_failures_24h.saturating_add(1);
        self.capture_degraded = true;
        self.last_error = Some(reason);
    }

    fn on_input<T: Copy + ToMonoF32>(&mut self, data: &[T], channels: usize) {
        if channels == 0 {
            self.on_stream_error("cpal callback channels == 0".to_string());
            return;
        }
        let callback_now = Instant::now();
        let frame_count = data.len() / channels;

        if let Some(last) = self.last_callback_instant {
            let actual_ms = callback_now.duration_since(last).as_secs_f64() * 1_000.0;
            let expected_ms = (frame_count as f64 / self.source_sample_rate_hz as f64) * 1_000.0;
            let deviation_ms = (actual_ms - expected_ms).abs();
            if self.callback_count == 0 {
                self.timing_jitter_ema_ms = deviation_ms;
            } else {
                self.timing_jitter_ema_ms = (self.timing_jitter_ema_ms * 0.9) + (deviation_ms * 0.1);
            }
            if deviation_ms > 120.0 {
                self.stream_gap_detected = true;
                self.capture_degraded = true;
            }
        }
        self.last_callback_instant = Some(callback_now);
        self.callback_count = self.callback_count.saturating_add(1);

        for frame in data.chunks(channels) {
            let mut mono = 0.0_f32;
            for sample in frame {
                mono += sample.to_f32();
            }
            mono /= channels as f32;
            mono = mono.clamp(-1.0, 1.0);
            self.total_input_frames = self.total_input_frames.saturating_add(1);
            self.push_resampled_sample(mono);
        }

        let elapsed_ns = self.started_instant.elapsed().as_nanos() as u64;
        if elapsed_ns > 0 {
            let expected_ns = self
                .total_output_samples
                .saturating_mul(1_000_000_000)
                .saturating_div(TARGET_SAMPLE_RATE_HZ as u64);
            let drift_ppm = ((expected_ns as f64 - elapsed_ns as f64) / elapsed_ns as f64) * 1_000_000.0;
            if self.callback_count == 1 {
                self.timing_drift_ema_ppm = drift_ppm;
            } else {
                self.timing_drift_ema_ppm = (self.timing_drift_ema_ppm * 0.9) + (drift_ppm * 0.1);
            }
        }
    }

    fn push_resampled_sample(&mut self, mono: f32) {
        self.resample_accumulator += TARGET_SAMPLE_RATE_HZ as f64;
        while self.resample_accumulator >= self.source_sample_rate_hz as f64 {
            self.resample_accumulator -= self.source_sample_rate_hz as f64;
            let pcm = (mono * i16::MAX as f32).round() as i16;
            self.total_output_samples = self.total_output_samples.saturating_add(1);
            self.output_samples_seen_for_metrics = self.output_samples_seen_for_metrics.saturating_add(1);
            if pcm.abs() >= (i16::MAX - 8) {
                self.clipped_output_samples = self.clipped_output_samples.saturating_add(1);
            }

            let sample_abs = (pcm as f64).abs() / i16::MAX as f64;
            let sample_sq = sample_abs * sample_abs;
            if self.output_samples_seen_for_metrics == 1 {
                self.rms_ema = sample_sq;
                self.noise_floor_ema = sample_sq.max(1e-8);
                self.speech_likeness_ema = sample_abs;
            } else {
                self.rms_ema = (self.rms_ema * 0.995) + (sample_sq * 0.005);
                self.noise_floor_ema = ((self.noise_floor_ema * 0.999) + (sample_sq * 0.001)).min(self.rms_ema.max(1e-8));
                self.speech_likeness_ema = (self.speech_likeness_ema * 0.99) + (sample_abs * 0.01);
            }

            if self.ring.len() >= self.ring_capacity_samples {
                self.ring.pop_front();
                self.timing_overruns = self.timing_overruns.saturating_add(1);
            }
            self.ring.push_back(pcm);

            self.frame_energy_accum += sample_sq;
            self.frame_sample_count = self.frame_sample_count.saturating_add(1);

            let frame_samples_target =
                ((TARGET_SAMPLE_RATE_HZ as usize).saturating_mul(FRAME_DURATION_MS as usize))
                    .saturating_div(1_000)
                    .max(1);
            if self.frame_sample_count >= frame_samples_target {
                self.consume_frame_energy();
            }
        }
    }

    fn consume_frame_energy(&mut self) {
        if self.frame_sample_count == 0 {
            return;
        }
        let frame_energy = (self.frame_energy_accum / self.frame_sample_count as f64).max(1e-10);
        let frame_rms = frame_energy.sqrt();

        if self.calibration_samples_remaining > 0 {
            self.noise_floor_min_energy = self.noise_floor_min_energy.min(frame_energy);
            if self.output_samples_seen_for_metrics <= self.frame_sample_count as u64 {
                self.noise_floor_reference_energy = frame_energy;
            } else {
                self.noise_floor_reference_energy =
                    (self.noise_floor_reference_energy * 0.95) + (frame_energy * 0.05);
            }
            self.calibration_samples_remaining = self
                .calibration_samples_remaining
                .saturating_sub(self.frame_sample_count as u64);
            if self.calibration_samples_remaining == 0 {
                let calibrated_floor = self
                    .noise_floor_min_energy
                    .min(self.noise_floor_reference_energy)
                    .max(1e-10);
                self.noise_floor_reference_energy = calibrated_floor;
            }
        } else {
            let alpha = if frame_energy <= self.noise_floor_reference_energy {
                0.08
            } else {
                0.002
            };
            self.noise_floor_reference_energy = (self.noise_floor_reference_energy * (1.0 - alpha))
                + (frame_energy * alpha);
        }

        let noise_floor_energy = self.noise_floor_reference_energy.max(1e-10);
        let energy_ratio = (frame_energy / noise_floor_energy).max(1.0);
        let ratio_db = 10.0 * energy_ratio.log10();
        let energy_speech_score = ((ratio_db - 1.0) / 10.0).clamp(0.0, 1.0);
        let rms_speech_score = ((frame_rms - 0.01) / 0.12).clamp(0.0, 1.0);
        let speech_score = (energy_speech_score * 0.7) + (rms_speech_score * 0.3);

        if self.output_samples_seen_for_metrics <= self.frame_sample_count as u64 {
            self.vad_confidence_ema = speech_score;
            self.speech_energy_ema = frame_energy.max(noise_floor_energy);
        } else {
            self.vad_confidence_ema = (self.vad_confidence_ema * 0.88) + (speech_score * 0.12);
            if speech_score >= 0.20 {
                self.speech_energy_ema = (self.speech_energy_ema * 0.92) + (frame_energy * 0.08);
            } else {
                self.speech_energy_ema = (self.speech_energy_ema * 0.995) + (frame_energy * 0.005);
            }
        }

        self.frame_energy_accum = 0.0;
        self.frame_sample_count = 0;
    }

    fn is_pre_roll_ready(&self) -> bool {
        self.ring.len() >= self.pre_roll_samples
    }

    fn buffered_ms(&self) -> f64 {
        (self.ring.len() as f64 / TARGET_SAMPLE_RATE_HZ as f64) * 1_000.0
    }

    fn build_capture_ref(&mut self) -> Result<VoiceTurnAudioCaptureRef, String> {
        if let Some(err) = self.last_error.as_ref() {
            return Err(format!("desktop mic producer failed: {err}"));
        }
        if !self.is_pre_roll_ready() {
            return Err(format!(
                "desktop mic producer pre-roll not ready: have={} need={} samples",
                self.ring.len(), self.pre_roll_samples
            ));
        }
        if self.total_output_samples == 0 {
            return Err("desktop mic producer has no output samples".to_string());
        }

        let pre_roll_ns = (self.pre_roll_samples as u64)
            .saturating_mul(1_000_000_000)
            .saturating_div(TARGET_SAMPLE_RATE_HZ as u64);
        let t_end_ns = self
            .started_ns
            .saturating_add(
                self.total_output_samples
                    .saturating_mul(1_000_000_000)
                    .saturating_div(TARGET_SAMPLE_RATE_HZ as u64),
            )
            .max(self.started_ns.saturating_add(1));
        let t_start_ns = t_end_ns.saturating_sub(pre_roll_ns).max(1);
        let candidate_offset_ns = (320_u64).saturating_mul(1_000_000);
        let t_candidate_start_ns = t_end_ns.saturating_sub(candidate_offset_ns).max(t_start_ns);
        let t_confirmed_ns = t_end_ns.max(t_candidate_start_ns);

        let noise_floor = self
            .noise_floor_reference_energy
            .max(self.noise_floor_ema.max(1e-10))
            .sqrt()
            .max(1e-6);
        let speech_energy = self.speech_energy_ema.max(noise_floor * noise_floor);
        let snr_db = 10.0 * (speech_energy / (noise_floor * noise_floor)).log10();
        let snr_db = snr_db.clamp(0.0, 45.0);
        let vad_conf = ((self.vad_confidence_ema * 0.75) + (((snr_db - 1.0) / 10.0).clamp(0.0, 1.0) * 0.25))
            .clamp(0.0, 1.0);
        let speech_likeness = ((self.speech_likeness_ema - 0.01) / 0.3).clamp(0.0, 1.0);

        let clipping_ratio = if self.output_samples_seen_for_metrics == 0 {
            0.0
        } else {
            self.clipped_output_samples as f64 / self.output_samples_seen_for_metrics as f64
        }
        .clamp(0.0, 1.0);

        let reliability_penalty = (self.device_failures_24h as u16).saturating_mul(200);
        let device_reliability_bp = 10_000_u16.saturating_sub(reliability_penalty).max(7_500);
        let jitter_ms = self.timing_jitter_ema_ms.clamp(0.0, 2_000.0);
        let drift_ppm = self.timing_drift_ema_ppm.abs().clamp(0.0, 10_000.0);
        let buffer_depth_ms = self.buffered_ms().clamp(0.0, 30_000.0);

        if self.locale_tag.trim().is_empty() {
            return Err("desktop mic producer missing locale_tag".to_string());
        }
        if self.selected_mic.trim().is_empty() {
            return Err("desktop mic producer missing selected_mic".to_string());
        }
        if self.selected_speaker.trim().is_empty() {
            return Err("desktop mic producer missing selected_speaker".to_string());
        }

        let pre_roll_buffer_id = self.next_pre_roll_buffer_id;
        self.next_pre_roll_buffer_id = self.next_pre_roll_buffer_id.saturating_add(1);

        Ok(VoiceTurnAudioCaptureRef {
            stream_id: self.stream_id,
            pre_roll_buffer_id,
            t_start_ns,
            t_end_ns,
            t_candidate_start_ns,
            t_confirmed_ns,
            locale_tag: Some(self.locale_tag.clone()),
            device_route: Some(self.device_route.clone()),
            selected_mic: Some(self.selected_mic.clone()),
            selected_speaker: Some(self.selected_speaker.clone()),
            tts_playback_active: Some(false),
            detection_text: None,
            detection_confidence_bp: None,
            vad_confidence_bp: Some(to_bp(vad_conf as f32)),
            acoustic_confidence_bp: Some(to_bp(((vad_conf * 0.95).clamp(0.0, 1.0)) as f32)),
            prosody_confidence_bp: Some(to_bp(((speech_likeness * 0.90).clamp(0.0, 1.0)) as f32)),
            speech_likeness_bp: Some(to_bp(speech_likeness as f32)),
            echo_safe_confidence_bp: Some(to_bp(if self.stream_gap_detected { 0.55 } else { 0.90 })),
            nearfield_confidence_bp: Some(to_bp(((speech_likeness * 0.92).clamp(0.0, 1.0)) as f32)),
            capture_degraded: Some(self.capture_degraded),
            stream_gap_detected: Some(self.stream_gap_detected),
            aec_unstable: Some(self.aec_unstable),
            device_changed: Some(self.device_changed),
            snr_db_milli: Some((snr_db * 1_000.0).round() as i32),
            clipping_ratio_bp: Some(to_bp(clipping_ratio as f32)),
            echo_delay_ms_milli: Some(25_000),
            packet_loss_bp: Some(if self.stream_gap_detected { 120 } else { 0 }),
            double_talk_bp: Some(to_bp(((speech_likeness * 0.15).clamp(0.0, 1.0)) as f32)),
            erle_db_milli: Some(18_000),
            device_failures_24h: Some(self.device_failures_24h),
            device_recoveries_24h: Some(self.device_recoveries_24h),
            device_mean_recovery_ms: Some(self.device_mean_recovery_ms),
            device_reliability_bp: Some(device_reliability_bp),
            timing_jitter_ms_milli: Some((jitter_ms * 1_000.0).round() as u32),
            timing_drift_ppm_milli: Some((drift_ppm * 1_000.0).round() as u32),
            timing_buffer_depth_ms_milli: Some((buffer_depth_ms * 1_000.0).round() as u32),
            timing_underruns: Some(self.timing_underruns),
            timing_overruns: Some(self.timing_overruns),
        })
    }
}

pub struct DesktopMicProducer {
    state: Arc<Mutex<CaptureState>>,
    stream: Stream,
}

impl DesktopMicProducer {
    pub fn start(config: DesktopMicProducerConfig) -> Result<Self, String> {
        let host = cpal::default_host();
        let input_device = resolve_input_device(&host, config.input_device_name_substring.as_deref())?;
        let output_device = host.default_output_device();

        let mic_name = input_device
            .name()
            .map_err(|err| format!("failed to read input device name: {err}"))?;
        let speaker_name = output_device
            .and_then(|dev| dev.name().ok())
            .unwrap_or_else(|| "desktop_speaker_default".to_string());

        let input_config = input_device
            .default_input_config()
            .map_err(|err| format!("failed to read default input config: {err}"))?;

        let source_sample_rate_hz = input_config.sample_rate().0;
        let source_channels = input_config.channels();
        let stream_config = StreamConfig {
            channels: source_channels,
            sample_rate: input_config.sample_rate(),
            buffer_size: cpal::BufferSize::Default,
        };

        let ring_buffer_ms = config.ring_buffer_ms.max(MIN_RING_BUFFER_MS);
        let pre_roll_ms = config.pre_roll_ms.max(MIN_PRE_ROLL_MS);
        let ring_capacity_samples = ms_to_samples(ring_buffer_ms, TARGET_SAMPLE_RATE_HZ)?;
        let pre_roll_samples = ms_to_samples(pre_roll_ms, TARGET_SAMPLE_RATE_HZ)?;

        let locale_tag = truncate_ascii(config.locale_tag.trim(), 32);
        if locale_tag.is_empty() {
            return Err("desktop mic producer locale_tag must not be empty".to_string());
        }

        let device_route = classify_device_route_label(&mic_name);
        let state = Arc::new(Mutex::new(CaptureState::new(
            locale_tag,
            sanitize_audio_device_token(&mic_name),
            sanitize_audio_device_token(&speaker_name),
            device_route,
            source_sample_rate_hz,
            source_channels,
            ring_capacity_samples,
            pre_roll_samples,
        )?));

        let stream = build_input_stream(
            &input_device,
            &stream_config,
            input_config.sample_format(),
            state.clone(),
        )?;

        stream
            .play()
            .map_err(|err| format!("failed to start input stream: {err}"))?;

        Ok(Self { state, stream })
    }

    pub fn wait_until_pre_roll_ready(&self, timeout: Duration) -> Result<(), String> {
        let deadline = Instant::now() + timeout;
        loop {
            {
                let state = self
                    .state
                    .lock()
                    .map_err(|_| "desktop mic producer state lock poisoned".to_string())?;
                if let Some(err) = state.last_error.as_ref() {
                    return Err(format!("desktop mic producer failed: {err}"));
                }
                if state.is_pre_roll_ready() {
                    return Ok(());
                }
            }
            if Instant::now() >= deadline {
                return Err("desktop mic producer timed out waiting for pre-roll".to_string());
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    pub fn build_capture_ref(&self) -> Result<VoiceTurnAudioCaptureRef, String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "desktop mic producer state lock poisoned".to_string())?;
        state.build_capture_ref()
    }

    pub fn source_sample_rate_hz(&self) -> Result<u32, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "desktop mic producer state lock poisoned".to_string())?;
        Ok(state.source_sample_rate_hz)
    }

    pub fn source_channels(&self) -> Result<u16, String> {
        let state = self
            .state
            .lock()
            .map_err(|_| "desktop mic producer state lock poisoned".to_string())?;
        Ok(state.source_channels)
    }

    pub fn target_sample_rate_hz() -> u32 {
        TARGET_SAMPLE_RATE_HZ
    }

    pub fn frame_duration_ms() -> u32 {
        FRAME_DURATION_MS
    }

    pub fn stop(self) {
        drop(self.stream);
    }
}

pub fn synthetic_capture_ref_for_tests(now_ns: u64) -> VoiceTurnAudioCaptureRef {
    let now_ns = now_ns.max(2_000_000_000);
    let t_start_ns = now_ns.saturating_sub(1_200_000_000);
    VoiceTurnAudioCaptureRef {
        stream_id: 44,
        pre_roll_buffer_id: 7,
        t_start_ns,
        t_end_ns: now_ns,
        t_candidate_start_ns: now_ns.saturating_sub(320_000_000),
        t_confirmed_ns: now_ns,
        locale_tag: Some("en-US".to_string()),
        device_route: Some("BUILT_IN".to_string()),
        selected_mic: Some("desktop_mic_default".to_string()),
        selected_speaker: Some("desktop_speaker_default".to_string()),
        tts_playback_active: Some(false),
        detection_text: None,
        detection_confidence_bp: None,
        vad_confidence_bp: Some(8_800),
        acoustic_confidence_bp: Some(8_500),
        prosody_confidence_bp: Some(8_100),
        speech_likeness_bp: Some(8_700),
        echo_safe_confidence_bp: Some(9_200),
        nearfield_confidence_bp: Some(8_300),
        capture_degraded: Some(false),
        stream_gap_detected: Some(false),
        aec_unstable: Some(false),
        device_changed: Some(false),
        snr_db_milli: Some(21_000),
        clipping_ratio_bp: Some(80),
        echo_delay_ms_milli: Some(25_000),
        packet_loss_bp: Some(0),
        double_talk_bp: Some(350),
        erle_db_milli: Some(18_000),
        device_failures_24h: Some(0),
        device_recoveries_24h: Some(0),
        device_mean_recovery_ms: Some(100),
        device_reliability_bp: Some(9_900),
        timing_jitter_ms_milli: Some(4_000),
        timing_drift_ppm_milli: Some(2_000),
        timing_buffer_depth_ms_milli: Some(1_250_000),
        timing_underruns: Some(0),
        timing_overruns: Some(0),
    }
}

fn build_input_stream(
    device: &cpal::Device,
    stream_config: &StreamConfig,
    sample_format: SampleFormat,
    state: Arc<Mutex<CaptureState>>,
) -> Result<Stream, String> {
    match sample_format {
        SampleFormat::F32 => build_input_stream_for_sample::<f32>(device, stream_config, state),
        SampleFormat::I16 => build_input_stream_for_sample::<i16>(device, stream_config, state),
        SampleFormat::U16 => build_input_stream_for_sample::<u16>(device, stream_config, state),
        other => Err(format!("unsupported input sample format: {other:?}")),
    }
}

fn build_input_stream_for_sample<T>(
    device: &cpal::Device,
    stream_config: &StreamConfig,
    state: Arc<Mutex<CaptureState>>,
) -> Result<Stream, String>
where
    T: cpal::SizedSample + Copy + ToMonoF32,
{
    let input_state = state.clone();
    let err_state = state;
    let channels = stream_config.channels as usize;
    device
        .build_input_stream(
            stream_config,
            move |data: &[T], _| {
                if let Ok(mut st) = input_state.lock() {
                    st.on_input(data, channels);
                }
            },
            move |err| {
                if let Ok(mut st) = err_state.lock() {
                    st.on_stream_error(format!("cpal stream error: {err}"));
                }
            },
            None,
        )
        .map_err(|err| format!("failed to build input stream: {err}"))
}

fn resolve_input_device(
    host: &cpal::Host,
    preferred_substring: Option<&str>,
) -> Result<cpal::Device, String> {
    if let Some(fragment) = preferred_substring {
        let fragment = fragment.trim().to_ascii_lowercase();
        if !fragment.is_empty() {
            let devices = host
                .input_devices()
                .map_err(|err| format!("failed to enumerate input devices: {err}"))?;
            for device in devices {
                if let Ok(name) = device.name() {
                    if name.to_ascii_lowercase().contains(&fragment) {
                        return Ok(device);
                    }
                }
            }
            return Err(format!(
                "no desktop input device matched substring '{}'",
                fragment
            ));
        }
    }

    host.default_input_device()
        .ok_or_else(|| "no default desktop input device available".to_string())
}

fn ms_to_samples(ms: u64, sample_rate_hz: u32) -> Result<usize, String> {
    let samples = ms
        .saturating_mul(sample_rate_hz as u64)
        .saturating_div(1_000);
    usize::try_from(samples).map_err(|_| "sample count overflow".to_string())
}

fn now_ns() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos() as u64)
        .unwrap_or(1)
        .max(1)
}

fn stable_hash_u128(value: &str) -> u128 {
    use std::hash::{Hash, Hasher};

    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    value.hash(&mut h1);
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    (value, "desktop").hash(&mut h2);
    ((h1.finish() as u128) << 64) | h2.finish() as u128
}

fn to_bp(value_0_to_1: f32) -> u16 {
    ((value_0_to_1.clamp(0.0, 1.0) * 10_000.0).round() as u16).min(10_000)
}

fn truncate_ascii(value: &str, max_len: usize) -> String {
    value.chars().take(max_len).collect::<String>()
}

fn sanitize_audio_device_token(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.') {
            out.push(c);
        } else {
            out.push('_');
        }
    }
    let out = truncate_ascii(out.trim(), 56);
    if out.is_empty() {
        "desktop_device".to_string()
    } else {
        out
    }
}

fn classify_device_route_label(device_name: &str) -> String {
    let lower = device_name.to_ascii_lowercase();
    if lower.contains("bluetooth") {
        "BLUETOOTH".to_string()
    } else if lower.contains("usb") {
        "USB".to_string()
    } else {
        "BUILT_IN".to_string()
    }
}

trait ToMonoF32 {
    fn to_f32(self) -> f32;
}

impl ToMonoF32 for f32 {
    fn to_f32(self) -> f32 {
        self
    }
}

impl ToMonoF32 for i16 {
    fn to_f32(self) -> f32 {
        self as f32 / i16::MAX as f32
    }
}

impl ToMonoF32 for u16 {
    fn to_f32(self) -> f32 {
        (self as f32 / u16::MAX as f32) * 2.0 - 1.0
    }
}
