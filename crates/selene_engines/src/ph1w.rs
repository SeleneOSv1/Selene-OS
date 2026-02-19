#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1k::{
    AudioStreamRef, Confidence, DeviceHealth, DeviceState, PreRollBufferRef, TimingStats, VadEvent,
};
use selene_kernel_contracts::ph1w::{
    BoundedAudioSegmentRef, SessionState, WakeDecision, WakeGateResults, WakePolicyContext,
    PH1W_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.W reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const W_FAIL_G0_DEVICE_UNHEALTHY: ReasonCodeId = ReasonCodeId(0x5700_0001);
    pub const W_FAIL_G0_AEC_UNSTABLE: ReasonCodeId = ReasonCodeId(0x5700_0002);
    pub const W_FAIL_G0_TIMING_UNTRUSTWORTHY: ReasonCodeId = ReasonCodeId(0x5700_0003);
    pub const W_FAIL_G0_CONTRACT_MISMATCH: ReasonCodeId = ReasonCodeId(0x5700_0004);
    pub const W_FAIL_G1_NOISE: ReasonCodeId = ReasonCodeId(0x5700_0010);
    pub const W_FAIL_G1A_NOT_UTTERANCE_START: ReasonCodeId = ReasonCodeId(0x5700_0011);
    pub const W_FAIL_G2_NOT_WAKE_LIKE: ReasonCodeId = ReasonCodeId(0x5700_0020);
    pub const W_FAIL_G3_SCORE_LOW: ReasonCodeId = ReasonCodeId(0x5700_0030);
    pub const W_FAIL_G3_UNSTABLE_SCORE: ReasonCodeId = ReasonCodeId(0x5700_0031);
    pub const W_FAIL_G3_ALIGNMENT: ReasonCodeId = ReasonCodeId(0x5700_0032);
    pub const W_FAIL_G3A_REPLAY_SUSPECTED: ReasonCodeId = ReasonCodeId(0x5700_0033);
    pub const W_FAIL_G4_USER_MISMATCH: ReasonCodeId = ReasonCodeId(0x5700_0040);
    pub const W_FAIL_G5_POLICY_BLOCKED: ReasonCodeId = ReasonCodeId(0x5700_0050);
    pub const W_SUPPRESS_EXPLICIT_TRIGGER_ONLY: ReasonCodeId = ReasonCodeId(0x5700_0051);

    pub const W_WAKE_ACCEPTED: ReasonCodeId = ReasonCodeId(0x5700_0100);
    pub const W_WAKE_REJECTED_TIMEOUT: ReasonCodeId = ReasonCodeId(0x5700_0101);
}

pub const PH1_W_ENGINE_ID: &str = "PH1.W";
pub const PH1_W_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1W_IMPLEMENTATION_ID];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1wState {
    Disarmed,
    ArmedIdle,
    Candidate,
    Confirmed,
    Capture,
    Cooldown,
    Suspended,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WakeConfig {
    pub candidate_validation_window_ms: u32,
    pub cooldown_ms: u32,
    pub suspended_stabilization_ms: u32,
    pub max_capture_ms: u32,

    pub min_voiced_ms: u32,
    pub min_vad_confidence: f32,
    pub min_speech_likeness: f32,
    pub min_preceding_silence_ms: u32,
    pub max_utterance_start_offset_ms: u32,

    pub light_score_threshold: f32,
    pub strong_score_threshold: f32,
    pub strong_score_threshold_tts: f32,
    pub strong_stable_required: u8,
    pub strong_stable_required_tts: u8,
    pub require_personalization_when_tts_playback_active: bool,
    pub require_liveness_gate: bool,
    pub require_near_field_when_media_playback_active: bool,

    pub max_jitter_ms: f32,
    pub max_drift_ppm: f32,
    pub max_underruns: u64,
}

impl WakeConfig {
    pub fn mvp_desktop_v1() -> Self {
        Self {
            candidate_validation_window_ms: 800,
            cooldown_ms: 500,
            suspended_stabilization_ms: 500,
            max_capture_ms: 4_000,
            min_voiced_ms: 120,
            min_vad_confidence: 0.60,
            min_speech_likeness: 0.60,
            min_preceding_silence_ms: 200,
            max_utterance_start_offset_ms: 1200,
            light_score_threshold: 0.50,
            strong_score_threshold: 0.75,
            strong_score_threshold_tts: 0.85,
            strong_stable_required: 2,
            strong_stable_required_tts: 3,
            require_personalization_when_tts_playback_active: true,
            require_liveness_gate: true,
            require_near_field_when_media_playback_active: true,
            max_jitter_ms: 40.0,
            max_drift_ppm: 200.0,
            max_underruns: 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceLivenessHint {
    Live,
    ReplaySuspected,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct WakeStepInput {
    pub now: MonotonicTimeNs,
    pub policy: WakePolicyContext,
    pub processed_stream: AudioStreamRef,
    pub pre_roll: PreRollBufferRef,
    pub vad: Option<VadEvent>,
    pub preceding_silence_ms: Option<u32>,
    pub utterance_start_offset_ms: Option<u32>,
    pub timing: TimingStats,
    pub device_state: DeviceState,
    pub aec_stable: bool,
    pub light_score: Option<Confidence>,
    pub strong_score: Option<Confidence>,
    pub strong_alignment_ok: bool,
    pub speaker_match_ok: bool,
    pub source_liveness_hint: SourceLivenessHint,
    pub near_field_speech_hint: Option<bool>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1wOutputEvent {
    StateChanged { from: Ph1wState, to: Ph1wState },
    Decision(WakeDecision),
}

#[derive(Debug, Clone)]
struct Candidate {
    t_started: MonotonicTimeNs,
    t_deadline: MonotonicTimeNs,
    processed_stream: AudioStreamRef,
    pre_roll: PreRollBufferRef,
    strong_stable_count: u8,
    last_light_score: Option<Confidence>,
    last_strong_score: Option<Confidence>,
}

#[derive(Debug, Clone)]
pub struct Ph1wRuntime {
    config: WakeConfig,
    state: Ph1wState,
    candidate: Option<Candidate>,
    capture: Option<BoundedAudioSegmentRef>,
    cooldown_until: Option<MonotonicTimeNs>,
    integrity_restored_at: Option<MonotonicTimeNs>,
}

impl Ph1wRuntime {
    pub fn new(config: WakeConfig) -> Self {
        Self {
            config,
            state: Ph1wState::Disarmed,
            candidate: None,
            capture: None,
            cooldown_until: None,
            integrity_restored_at: None,
        }
    }

    pub fn state(&self) -> Ph1wState {
        self.state
    }

    pub fn step(&mut self, input: WakeStepInput) -> Vec<Ph1wOutputEvent> {
        self.step_for_implementation(PH1W_IMPLEMENTATION_ID, input)
            .expect("PH1.W.001 must be valid")
    }

    pub fn step_for_implementation(
        &mut self,
        implementation_id: &str,
        input: WakeStepInput,
    ) -> Result<Vec<Ph1wOutputEvent>, ContractViolation> {
        if implementation_id != PH1W_IMPLEMENTATION_ID {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_w.implementation_id",
                reason: "unknown implementation_id",
            });
        }
        Ok(self.evaluate(input))
    }

    fn evaluate(&mut self, input: WakeStepInput) -> Vec<Ph1wOutputEvent> {
        let mut out = Vec::new();

        let policy_disarm_reason = policy_disarm_reason(input.policy);
        let policy_disarmed = policy_disarm_reason.is_some();
        let policy_ok = gate5_policy_ok(input.policy);

        let integrity_fail = classify_gate0_failure(&self.config, &input);
        if let Some(rc) = integrity_fail {
            if self.state == Ph1wState::Candidate {
                out.push(Ph1wOutputEvent::Decision(
                    WakeDecision::reject_v1(
                        rc,
                        WakeGateResults {
                            g0_integrity_ok: false,
                            g1_activity_ok: false,
                            g1a_utterance_start_ok: false,
                            g2_light_ok: false,
                            g3_strong_ok: false,
                            g3a_liveness_ok: false,
                            g4_personalization_ok: false,
                            g5_policy_ok: policy_ok,
                        },
                        input.now,
                        self.candidate.as_ref().and_then(|c| c.last_light_score),
                        self.candidate.as_ref().and_then(|c| c.last_strong_score),
                    )
                    .expect("WakeDecision::reject_v1 should be constructible"),
                ));
            }
            self.candidate = None;
            self.capture = None;
            self.cooldown_until = None;
            self.integrity_restored_at = None;
            out.extend(self.transition_to(Ph1wState::Suspended));
            return out;
        }

        // Integrity is ok. If suspended, require stabilization before re-arming.
        if self.state == Ph1wState::Suspended {
            let restored_at = self.integrity_restored_at.get_or_insert(input.now);
            if input.now.0.saturating_sub(restored_at.0)
                >= ms_to_ns(self.config.suspended_stabilization_ms)
            {
                self.integrity_restored_at = None;
                out.extend(self.transition_to(if policy_disarmed {
                    Ph1wState::Disarmed
                } else {
                    Ph1wState::ArmedIdle
                }));
            }
            return out;
        }

        // Policy can disarm wake regardless of other conditions.
        if policy_disarmed {
            if self.state == Ph1wState::Candidate {
                out.push(Ph1wOutputEvent::Decision(
                    WakeDecision::reject_v1(
                        policy_disarm_reason.unwrap_or(reason_codes::W_FAIL_G5_POLICY_BLOCKED),
                        WakeGateResults {
                            g0_integrity_ok: true,
                            g1_activity_ok: false,
                            g1a_utterance_start_ok: false,
                            g2_light_ok: false,
                            g3_strong_ok: false,
                            g3a_liveness_ok: false,
                            g4_personalization_ok: false,
                            g5_policy_ok: false,
                        },
                        input.now,
                        self.candidate.as_ref().and_then(|c| c.last_light_score),
                        self.candidate.as_ref().and_then(|c| c.last_strong_score),
                    )
                    .expect("WakeDecision::reject_v1 should be constructible"),
                ));
            }
            self.candidate = None;
            self.capture = None;
            self.cooldown_until = None;
            out.extend(self.transition_to(Ph1wState::Disarmed));
            return out;
        }

        // Auto-arm once policy allows.
        if self.state == Ph1wState::Disarmed {
            out.extend(self.transition_to(Ph1wState::ArmedIdle));
        }

        // Timed transitions.
        if self.state == Ph1wState::Capture {
            if let Some(seg) = self.capture {
                if input.now.0 >= seg.t_end.0 {
                    self.capture = None;
                    self.cooldown_until = Some(MonotonicTimeNs(
                        input
                            .now
                            .0
                            .saturating_add(ms_to_ns(self.config.cooldown_ms)),
                    ));
                    out.extend(self.transition_to(Ph1wState::Cooldown));
                }
            } else {
                // Defensive: capture state without a segment.
                out.extend(self.transition_to(Ph1wState::Cooldown));
            }
            // Avoid immediately re-entering candidate logic on the same tick as a timed transition.
            if self.state == Ph1wState::Cooldown {
                return out;
            }
        }

        if self.state == Ph1wState::Cooldown {
            if let Some(until) = self.cooldown_until {
                if input.now.0 >= until.0 {
                    self.cooldown_until = None;
                    out.extend(self.transition_to(Ph1wState::ArmedIdle));
                }
            } else {
                out.extend(self.transition_to(Ph1wState::ArmedIdle));
            }
            // Avoid immediately triggering a new candidate on the same tick the cooldown ends.
            if self.state == Ph1wState::ArmedIdle {
                return out;
            }
        }

        // Core state machine.
        match self.state {
            Ph1wState::ArmedIdle => {
                // Start candidate only when Gate-0, Gate-1, Gate-1A, and Gate-2 pass.
                let g1_ok = gate1_activity_ok(&self.config, &input);
                let g1a_ok = gate1a_utterance_start_ok(&self.config, &input);
                let g2_ok = gate2_light_ok(&self.config, input.light_score);

                if g1_ok && g1a_ok && g2_ok {
                    let t_deadline = MonotonicTimeNs(
                        input
                            .now
                            .0
                            .saturating_add(ms_to_ns(self.config.candidate_validation_window_ms)),
                    );
                    self.candidate = Some(Candidate {
                        t_started: input.now,
                        t_deadline,
                        processed_stream: input.processed_stream,
                        pre_roll: input.pre_roll,
                        strong_stable_count: 0,
                        last_light_score: input.light_score,
                        last_strong_score: input.strong_score,
                    });
                    out.extend(self.transition_to(Ph1wState::Candidate));
                }
            }
            Ph1wState::Candidate => {
                let Some(mut cand) = self.candidate.clone() else {
                    out.extend(self.transition_to(Ph1wState::ArmedIdle));
                    return out;
                };

                cand.last_light_score = input.light_score.or(cand.last_light_score);
                cand.last_strong_score = input.strong_score.or(cand.last_strong_score);

                if input.now.0 > cand.t_deadline.0 {
                    let (rc, gates) = classify_candidate_reject(&self.config, &input, &cand);
                    out.push(Ph1wOutputEvent::Decision(
                        WakeDecision::reject_v1(
                            rc,
                            gates,
                            input.now,
                            cand.last_light_score,
                            cand.last_strong_score,
                        )
                        .expect("WakeDecision::reject_v1 should be constructible"),
                    ));
                    self.candidate = None;
                    out.extend(self.transition_to(Ph1wState::ArmedIdle));
                    return out;
                }

                // Update strong stability counter.
                let (strong_threshold, strong_required) = if input.policy.tts_playback_active {
                    (
                        self.config.strong_score_threshold_tts,
                        self.config.strong_stable_required_tts,
                    )
                } else {
                    (
                        self.config.strong_score_threshold,
                        self.config.strong_stable_required,
                    )
                };

                let strong_ok_now = input
                    .strong_score
                    .is_some_and(|s| input.strong_alignment_ok && s.0 >= strong_threshold);

                if strong_ok_now {
                    cand.strong_stable_count = cand.strong_stable_count.saturating_add(1);
                } else {
                    cand.strong_stable_count = 0;
                }

                let g3_ok = cand.strong_stable_count >= strong_required;
                let g3a_ok = gate3a_liveness_ok(&self.config, &input);
                let g4_ok = gate4_personalization_ok(&self.config, &input);
                let g5_ok = policy_ok;

                if g3_ok && g3a_ok && g4_ok && g5_ok {
                    // CONFIRMED -> CAPTURE immediately (bounded segment ref).
                    out.extend(self.transition_to(Ph1wState::Confirmed));

                    let capture_start = cand.pre_roll.t_start;
                    let capture_end = MonotonicTimeNs(
                        capture_start
                            .0
                            .saturating_add(ms_to_ns(self.config.max_capture_ms)),
                    );
                    let seg = BoundedAudioSegmentRef::v1(
                        cand.processed_stream.stream_id,
                        cand.pre_roll.buffer_id,
                        capture_start,
                        capture_end,
                        cand.t_started,
                        input.now,
                    )
                    .expect("BoundedAudioSegmentRef must be constructible");

                    let gates = WakeGateResults {
                        g0_integrity_ok: true,
                        g1_activity_ok: true,
                        g1a_utterance_start_ok: true,
                        g2_light_ok: true,
                        g3_strong_ok: true,
                        g3a_liveness_ok: true,
                        g4_personalization_ok: true,
                        g5_policy_ok: true,
                    };

                    out.push(Ph1wOutputEvent::Decision(
                        WakeDecision::accept_v1(
                            reason_codes::W_WAKE_ACCEPTED,
                            gates,
                            input.now,
                            cand.last_light_score,
                            cand.last_strong_score,
                            seg,
                        )
                        .expect("WakeDecision::accept_v1 should be constructible"),
                    ));

                    self.capture = Some(seg);
                    self.candidate = None;
                    out.extend(self.transition_to(Ph1wState::Capture));
                } else {
                    self.candidate = Some(cand);
                }
            }
            _ => {
                // Other states ignore wake-evaluation steps.
            }
        }

        out
    }

    fn transition_to(&mut self, next: Ph1wState) -> Vec<Ph1wOutputEvent> {
        if self.state == next {
            return Vec::new();
        }
        let from = self.state;
        self.state = next;
        vec![Ph1wOutputEvent::StateChanged { from, to: next }]
    }
}

fn ms_to_ns(ms: u32) -> u64 {
    (ms as u64).saturating_mul(1_000_000)
}

fn policy_disarmed(policy: WakePolicyContext) -> bool {
    policy_disarm_reason(policy).is_some()
}

fn policy_disarm_reason(policy: WakePolicyContext) -> Option<ReasonCodeId> {
    if policy.explicit_trigger_only {
        return Some(reason_codes::W_SUPPRESS_EXPLICIT_TRIGGER_ONLY);
    }
    if matches!(policy.session_state, SessionState::Closed)
        || policy.do_not_disturb
        || policy.privacy_mode
    {
        return Some(reason_codes::W_FAIL_G5_POLICY_BLOCKED);
    }
    None
}

fn gate5_policy_ok(policy: WakePolicyContext) -> bool {
    !policy_disarmed(policy)
}

fn classify_gate0_failure(config: &WakeConfig, input: &WakeStepInput) -> Option<ReasonCodeId> {
    if input.processed_stream.stream_id != input.pre_roll.stream_id {
        return Some(reason_codes::W_FAIL_G0_CONTRACT_MISMATCH);
    }

    if input.device_state.health != DeviceHealth::Healthy {
        return Some(reason_codes::W_FAIL_G0_DEVICE_UNHEALTHY);
    }

    if input.policy.tts_playback_active && !input.aec_stable {
        return Some(reason_codes::W_FAIL_G0_AEC_UNSTABLE);
    }

    // TimingStats are floats; treat non-finite as untrustworthy.
    for v in [input.timing.jitter_ms, input.timing.drift_ppm] {
        if !v.is_finite() {
            return Some(reason_codes::W_FAIL_G0_TIMING_UNTRUSTWORTHY);
        }
    }

    if input.timing.jitter_ms > config.max_jitter_ms
        || input.timing.drift_ppm > config.max_drift_ppm
        || input.timing.underruns > config.max_underruns
    {
        return Some(reason_codes::W_FAIL_G0_TIMING_UNTRUSTWORTHY);
    }

    None
}

fn gate1_activity_ok(config: &WakeConfig, input: &WakeStepInput) -> bool {
    let Some(vad) = &input.vad else {
        return false;
    };

    let dur_ns = vad.t_end.0.saturating_sub(vad.t_start.0);
    if dur_ns < ms_to_ns(config.min_voiced_ms) {
        return false;
    }

    vad.confidence.0 >= config.min_vad_confidence
        && vad.speech_likeness.0 >= config.min_speech_likeness
}

fn gate1a_utterance_start_ok(config: &WakeConfig, input: &WakeStepInput) -> bool {
    let Some(preceding_silence_ms) = input.preceding_silence_ms else {
        return false;
    };
    let Some(utterance_start_offset_ms) = input.utterance_start_offset_ms else {
        return false;
    };
    preceding_silence_ms >= config.min_preceding_silence_ms
        && utterance_start_offset_ms <= config.max_utterance_start_offset_ms
}

fn gate2_light_ok(config: &WakeConfig, light_score: Option<Confidence>) -> bool {
    light_score.is_some_and(|s| s.0 >= config.light_score_threshold)
}

fn gate3a_liveness_ok(config: &WakeConfig, input: &WakeStepInput) -> bool {
    if !config.require_liveness_gate {
        return true;
    }
    if input.source_liveness_hint == SourceLivenessHint::ReplaySuspected {
        return false;
    }
    if input.policy.tts_playback_active && !input.aec_stable {
        return false;
    }
    if input.policy.media_playback_active && config.require_near_field_when_media_playback_active {
        return input.near_field_speech_hint.unwrap_or(false);
    }
    true
}

fn gate4_personalization_ok(config: &WakeConfig, input: &WakeStepInput) -> bool {
    if input.policy.tts_playback_active && config.require_personalization_when_tts_playback_active {
        return input.speaker_match_ok;
    }
    input.speaker_match_ok
}

fn classify_candidate_reject(
    config: &WakeConfig,
    input: &WakeStepInput,
    cand: &Candidate,
) -> (ReasonCodeId, WakeGateResults) {
    let g5_ok = gate5_policy_ok(input.policy);
    let g1a_ok = gate1a_utterance_start_ok(config, input);
    let g4_ok = gate4_personalization_ok(config, input);
    let g3a_ok = gate3a_liveness_ok(config, input);

    let (strong_threshold, strong_required) = if input.policy.tts_playback_active {
        (
            config.strong_score_threshold_tts,
            config.strong_stable_required_tts,
        )
    } else {
        (config.strong_score_threshold, config.strong_stable_required)
    };

    let had_strong = cand.last_strong_score.is_some();
    let strong_above = cand
        .last_strong_score
        .is_some_and(|s| s.0 >= strong_threshold);
    let g3_ok = cand.strong_stable_count >= strong_required;

    let rc = if !g5_ok {
        reason_codes::W_FAIL_G5_POLICY_BLOCKED
    } else if !g1a_ok {
        reason_codes::W_FAIL_G1A_NOT_UTTERANCE_START
    } else if !g4_ok {
        reason_codes::W_FAIL_G4_USER_MISMATCH
    } else if !g3a_ok {
        reason_codes::W_FAIL_G3A_REPLAY_SUSPECTED
    } else if !input.strong_alignment_ok && had_strong && strong_above {
        reason_codes::W_FAIL_G3_ALIGNMENT
    } else if had_strong && strong_above {
        reason_codes::W_FAIL_G3_UNSTABLE_SCORE
    } else if had_strong {
        reason_codes::W_FAIL_G3_SCORE_LOW
    } else {
        reason_codes::W_WAKE_REJECTED_TIMEOUT
    };

    (
        rc,
        WakeGateResults {
            g0_integrity_ok: true,
            g1_activity_ok: true,
            g1a_utterance_start_ok: g1a_ok,
            g2_light_ok: true,
            g3_strong_ok: g3_ok,
            g3a_liveness_ok: g3a_ok,
            g4_personalization_ok: g4_ok,
            g5_policy_ok: g5_ok,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioFormat, AudioStreamId, AudioStreamKind, ChannelCount, FrameDurationMs,
        PreRollBufferId, SampleFormat, SampleRateHz, SpeechLikeness,
    };

    fn stream_ref() -> AudioStreamRef {
        AudioStreamRef::v1(
            AudioStreamId(1),
            AudioStreamKind::MicProcessed,
            AudioFormat {
                sample_rate_hz: SampleRateHz(16_000),
                channels: ChannelCount(1),
                sample_format: SampleFormat::PcmS16LE,
            },
            FrameDurationMs::Ms20,
        )
    }

    fn pre_roll(t_start: u64, t_end: u64) -> PreRollBufferRef {
        PreRollBufferRef::v1(
            PreRollBufferId(7),
            AudioStreamId(1),
            MonotonicTimeNs(t_start),
            MonotonicTimeNs(t_end),
        )
    }

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    fn healthy_device() -> DeviceState {
        DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![])
    }

    fn timing_ok() -> TimingStats {
        TimingStats::v1(1.0, 1.0, 10.0, 0, 0)
    }

    fn vad_ok(t_start: u64, t_end: u64) -> VadEvent {
        VadEvent::v1(
            AudioStreamId(1),
            MonotonicTimeNs(t_start),
            MonotonicTimeNs(t_end),
            Confidence::new(0.9).unwrap(),
            SpeechLikeness::new(0.9).unwrap(),
        )
    }

    fn base_input(now: u64) -> WakeStepInput {
        WakeStepInput {
            now: MonotonicTimeNs(now),
            policy: WakePolicyContext::v1(SessionState::Active, false, false, false),
            processed_stream: stream_ref(),
            pre_roll: pre_roll(0, now),
            vad: None,
            preceding_silence_ms: Some(300),
            utterance_start_offset_ms: Some(500),
            timing: timing_ok(),
            device_state: healthy_device(),
            aec_stable: true,
            light_score: None,
            strong_score: None,
            strong_alignment_ok: true,
            speaker_match_ok: true,
            source_liveness_hint: SourceLivenessHint::Live,
            near_field_speech_hint: Some(true),
        }
    }

    #[test]
    fn armed_idle_to_candidate_requires_gate1_and_gate2() {
        let mut rt = Ph1wRuntime::new(WakeConfig::mvp_desktop_v1());

        // Provide enough to pass Gate-1 and Gate-2.
        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());

        let out = rt.step(input);
        assert!(out.iter().any(|e| matches!(
            e,
            Ph1wOutputEvent::StateChanged {
                to: Ph1wState::Candidate,
                ..
            }
        )));
    }

    #[test]
    fn gate1a_blocks_mid_utterance_wake_candidate() {
        let mut rt = Ph1wRuntime::new(WakeConfig::mvp_desktop_v1());
        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());
        input.preceding_silence_ms = Some(100);
        input.utterance_start_offset_ms = Some(1500);

        let out = rt.step(input);
        assert!(!out.iter().any(|e| matches!(
            e,
            Ph1wOutputEvent::StateChanged {
                to: Ph1wState::Candidate,
                ..
            }
        )));
        assert_eq!(rt.state(), Ph1wState::ArmedIdle);
    }

    #[test]
    fn candidate_confirms_then_capture_then_cooldown_then_returns_to_idle() {
        let cfg = WakeConfig::mvp_desktop_v1();
        let mut rt = Ph1wRuntime::new(cfg);

        // Start candidate.
        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Candidate);

        // Feed strong verifier stability frames.
        input.strong_score = Some(Confidence::new(0.95).unwrap());
        input.now = MonotonicTimeNs(ms_to_ns(10));
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Candidate);

        input.now = MonotonicTimeNs(ms_to_ns(20));
        let out = rt.step(input.clone());
        assert!(out
            .iter()
            .any(|e| matches!(e, Ph1wOutputEvent::Decision(d) if d.accepted)));
        assert_eq!(rt.state(), Ph1wState::Capture);

        // Advance to capture end.
        let capture_end = rt.capture.unwrap().t_end.0;
        input.now = MonotonicTimeNs(capture_end);
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Cooldown);

        // Advance past cooldown.
        let cooldown_until = rt.cooldown_until.unwrap().0;
        input.now = MonotonicTimeNs(cooldown_until);
        rt.step(input);
        assert_eq!(rt.state(), Ph1wState::ArmedIdle);
    }

    #[test]
    fn candidate_times_out_and_rejects() {
        let cfg = WakeConfig::mvp_desktop_v1();
        let mut rt = Ph1wRuntime::new(cfg);

        // Start candidate.
        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Candidate);

        // Jump beyond deadline with no strong score.
        input.now = MonotonicTimeNs(ms_to_ns(cfg.candidate_validation_window_ms) + 1);
        let out = rt.step(input);
        assert!(out
            .iter()
            .any(|e| matches!(e, Ph1wOutputEvent::Decision(d) if !d.accepted)));
        assert_eq!(rt.state(), Ph1wState::ArmedIdle);
    }

    #[test]
    fn candidate_rejects_when_liveness_flags_replay() {
        let cfg = WakeConfig::mvp_desktop_v1();
        let mut rt = Ph1wRuntime::new(cfg);

        // Start candidate.
        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Candidate);

        // Replay suspicion prevents confirmation and yields deterministic reject on timeout.
        input.source_liveness_hint = SourceLivenessHint::ReplaySuspected;
        input.strong_score = Some(Confidence::new(0.95).unwrap());
        input.now = MonotonicTimeNs(ms_to_ns(cfg.candidate_validation_window_ms) + 1);
        let out = rt.step(input);
        assert!(out.iter().any(|e| matches!(
            e,
            Ph1wOutputEvent::Decision(d)
                if !d.accepted && d.reason_code == reason_codes::W_FAIL_G3A_REPLAY_SUSPECTED
        )));
    }

    #[test]
    fn integrity_failure_suspends_then_restores_after_stabilization() {
        let cfg = WakeConfig::mvp_desktop_v1();
        let mut rt = Ph1wRuntime::new(cfg);

        let mut input = base_input(0);
        input.device_state =
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Degraded, vec![]);
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Suspended);

        // Restore health but not enough stabilization time -> still suspended.
        input.device_state = healthy_device();
        input.now = MonotonicTimeNs(0);
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Suspended);

        input.now = MonotonicTimeNs(ms_to_ns(cfg.suspended_stabilization_ms) - 1);
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Suspended);

        // Stabilization elapsed since integrity restore -> armed idle.
        input.now = MonotonicTimeNs(ms_to_ns(cfg.suspended_stabilization_ms));
        rt.step(input);
        assert_eq!(rt.state(), Ph1wState::ArmedIdle);
    }

    #[test]
    fn policy_disarms_wake() {
        let mut rt = Ph1wRuntime::new(WakeConfig::mvp_desktop_v1());

        let mut input = base_input(0);
        input.policy = WakePolicyContext::v1(SessionState::Active, true, false, false);
        rt.step(input);
        assert_eq!(rt.state(), Ph1wState::Disarmed);
    }

    #[test]
    fn explicit_trigger_only_disarms_with_reason_coded_reject_from_candidate() {
        let mut rt = Ph1wRuntime::new(WakeConfig::mvp_desktop_v1());

        let mut input = base_input(0);
        input.vad = Some(vad_ok(0, ms_to_ns(200)));
        input.light_score = Some(Confidence::new(0.9).unwrap());
        rt.step(input.clone());
        assert_eq!(rt.state(), Ph1wState::Candidate);

        input.policy = WakePolicyContext::v1_with_media_and_trigger(
            SessionState::Active,
            false,
            false,
            false,
            false,
            true,
        );
        let out = rt.step(input);

        assert!(out.iter().any(|e| matches!(
            e,
            Ph1wOutputEvent::Decision(d)
                if !d.accepted && d.reason_code == reason_codes::W_SUPPRESS_EXPLICIT_TRIGGER_ONLY
        )));
        assert_eq!(rt.state(), Ph1wState::Disarmed);
    }

    #[test]
    fn at_w_impl_01_unknown_implementation_fails_closed() {
        let mut rt = Ph1wRuntime::new(WakeConfig::mvp_desktop_v1());
        let out = rt.step_for_implementation("PH1.W.999", base_input(0));
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1_w.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_w_impl_02_active_implementation_list_is_locked() {
        assert_eq!(PH1_W_ENGINE_ID, "PH1.W");
        assert_eq!(PH1_W_ACTIVE_IMPLEMENTATION_IDS, &["PH1.W.001"]);
    }
}
