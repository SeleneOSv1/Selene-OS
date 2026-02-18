#![forbid(unsafe_code)]

use std::collections::HashMap;

use selene_kernel_contracts::ph1k::{
    AudioDeviceId, Confidence, DegradationFlags, DeviceError, DeviceHealth, DeviceRoute,
    DeviceState, InterruptCandidate, InterruptGateConfidences, InterruptGates, InterruptPhraseId,
    InterruptPhraseSetVersion, Ph1kState, SpeechLikeness, StateTransitionEvent,
    PH1K_IMPLEMENTATION_ID,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.K reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const K_DEVICE_SELECTION_FAILED: ReasonCodeId = ReasonCodeId(0x4B00_0001);
    pub const K_DEVICE_CHANGED: ReasonCodeId = ReasonCodeId(0x4B00_0002);
    pub const K_PERMISSION_LOST: ReasonCodeId = ReasonCodeId(0x4B00_0003);
    pub const K_AEC_UNSTABLE: ReasonCodeId = ReasonCodeId(0x4B00_0004);
    pub const K_STREAM_GAP: ReasonCodeId = ReasonCodeId(0x4B00_0005);
    pub const K_INTERRUPT_CANDIDATE: ReasonCodeId = ReasonCodeId(0x4B00_0006);
    pub const K_STATE_READY: ReasonCodeId = ReasonCodeId(0x4B00_0007);
    pub const K_STATE_FULL_DUPLEX_ACTIVE: ReasonCodeId = ReasonCodeId(0x4B00_0008);
    pub const K_STATE_DEVICE_SWITCHING: ReasonCodeId = ReasonCodeId(0x4B00_0009);
    pub const K_STATE_DEGRADED: ReasonCodeId = ReasonCodeId(0x4B00_000A);
    pub const K_STATE_FAILED: ReasonCodeId = ReasonCodeId(0x4B00_000B);
    pub const K_DEVICE_LIST_UNSTABLE: ReasonCodeId = ReasonCodeId(0x4B00_000C);
}

pub const PH1_K_ENGINE_ID: &str = "PH1.K";
pub const PH1_K_ACTIVE_IMPLEMENTATION_IDS: &[&str] = &[PH1K_IMPLEMENTATION_ID];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableDevices {
    pub mics: Vec<AudioDeviceId>,
    pub speakers: Vec<AudioDeviceId>,
    pub system_default_mic: Option<AudioDeviceId>,
    pub system_default_speaker: Option<AudioDeviceId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DevicePreference {
    pub preferred_mic: Option<AudioDeviceId>,
    pub preferred_speaker: Option<AudioDeviceId>,
    pub last_known_good_mic: Option<AudioDeviceId>,
    pub last_known_good_speaker: Option<AudioDeviceId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceSelection {
    pub mic: AudioDeviceId,
    pub speaker: AudioDeviceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceSelectionError {
    NoMicAvailable,
    NoSpeakerAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePolicy {
    pub preference: DevicePreference,
}

impl DevicePolicy {
    pub fn select(
        &self,
        available: &AvailableDevices,
    ) -> Result<DeviceSelection, DeviceSelectionError> {
        let mic = select_one(
            &available.mics,
            &self.preference.preferred_mic,
            &self.preference.last_known_good_mic,
            &available.system_default_mic,
        )
        .ok_or(DeviceSelectionError::NoMicAvailable)?;

        let speaker = select_one(
            &available.speakers,
            &self.preference.preferred_speaker,
            &self.preference.last_known_good_speaker,
            &available.system_default_speaker,
        )
        .ok_or(DeviceSelectionError::NoSpeakerAvailable)?;

        Ok(DeviceSelection { mic, speaker })
    }
}

fn select_one(
    devices: &[AudioDeviceId],
    preferred: &Option<AudioDeviceId>,
    last_known_good: &Option<AudioDeviceId>,
    system_default: &Option<AudioDeviceId>,
) -> Option<AudioDeviceId> {
    if let Some(d) = preferred {
        if devices.contains(d) {
            return Some(d.clone());
        }
    }
    if let Some(d) = last_known_good {
        if devices.contains(d) {
            return Some(d.clone());
        }
    }
    if let Some(d) = system_default {
        if devices.contains(d) {
            return Some(d.clone());
        }
    }
    devices.iter().min_by_key(|d| d.as_str()).cloned()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1kEvent {
    Boot {
        available: AvailableDevices,
        now: MonotonicTimeNs,
    },
    StartFullDuplex {
        now: MonotonicTimeNs,
    },
    Stop {
        now: MonotonicTimeNs,
    },
    DeviceListChanged {
        available: AvailableDevices,
        now: MonotonicTimeNs,
    },
    PermissionLost {
        now: MonotonicTimeNs,
    },
    AecUnstable {
        now: MonotonicTimeNs,
    },
    AecStable {
        now: MonotonicTimeNs,
    },
    StreamGapDetected {
        now: MonotonicTimeNs,
    },
    StreamRecovered {
        now: MonotonicTimeNs,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1kOutputEvent {
    StateChanged(StateTransitionEvent),
    DeviceState(DeviceState),
    DegradationFlags(DegradationFlags),
}

#[derive(Debug, Clone)]
pub struct Ph1kRuntime {
    policy: DevicePolicy,
    state: Ph1kState,
    selection: Option<DeviceSelection>,
    pending_selection: Option<DeviceSelection>,
    pending_selection_since: Option<MonotonicTimeNs>,
    last_switch_at: Option<MonotonicTimeNs>,
    recovery_ready_since: Option<MonotonicTimeNs>,
    degradation: DegradationFlags,
    health: DeviceHealth,
}

impl Ph1kRuntime {
    const DEVICE_STABILITY_WINDOW_NS: u64 = 300_000_000;
    const DEVICE_SWITCH_COOLDOWN_NS: u64 = 2_000_000_000;
    const RECOVERY_STABILITY_WINDOW_NS: u64 = 500_000_000;

    pub fn new(policy: DevicePolicy) -> Self {
        Self {
            policy,
            state: Ph1kState::Init,
            selection: None,
            pending_selection: None,
            pending_selection_since: None,
            last_switch_at: None,
            recovery_ready_since: None,
            degradation: DegradationFlags {
                capture_degraded: false,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
            },
            health: DeviceHealth::Healthy,
        }
    }

    pub fn state(&self) -> Ph1kState {
        self.state
    }

    pub fn selection(&self) -> Option<&DeviceSelection> {
        self.selection.as_ref()
    }

    pub fn handle(&mut self, event: Ph1kEvent) -> Vec<Ph1kOutputEvent> {
        self.handle_for_implementation(PH1K_IMPLEMENTATION_ID, event)
            .expect("PH1.K.001 must be valid")
    }

    pub fn handle_for_implementation(
        &mut self,
        implementation_id: &str,
        event: Ph1kEvent,
    ) -> Result<Vec<Ph1kOutputEvent>, ContractViolation> {
        if implementation_id != PH1K_IMPLEMENTATION_ID {
            return Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            });
        }
        Ok(self.evaluate(event))
    }

    fn evaluate(&mut self, event: Ph1kEvent) -> Vec<Ph1kOutputEvent> {
        match event {
            Ph1kEvent::Boot { available, now } => self.on_boot(available, now),
            Ph1kEvent::StartFullDuplex { now } => self.transition_to(
                Ph1kState::FullDuplexActive,
                now,
                reason_codes::K_STATE_FULL_DUPLEX_ACTIVE,
            ),
            Ph1kEvent::Stop { now } => {
                self.transition_to(Ph1kState::Ready, now, reason_codes::K_STATE_READY)
            }
            Ph1kEvent::DeviceListChanged { available, now } => {
                self.on_device_list_changed(available, now)
            }
            Ph1kEvent::PermissionLost { now } => self.on_permission_lost(now),
            Ph1kEvent::AecUnstable { now } => self.on_aec_unstable(now),
            Ph1kEvent::AecStable { now } => self.on_aec_stable(now),
            Ph1kEvent::StreamGapDetected { now } => self.on_stream_gap(now),
            Ph1kEvent::StreamRecovered { now } => self.on_stream_recovered(now),
        }
    }

    fn on_boot(
        &mut self,
        available: AvailableDevices,
        now: MonotonicTimeNs,
    ) -> Vec<Ph1kOutputEvent> {
        let mut out = Vec::new();
        match self.policy.select(&available) {
            Ok(sel) => {
                self.selection = Some(sel.clone());
                self.pending_selection = None;
                self.pending_selection_since = None;
                out.extend(self.transition_to(Ph1kState::Ready, now, reason_codes::K_STATE_READY));
                let route = classify_device_route(&sel.mic, &sel.speaker);
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                    sel.mic.clone(),
                    sel.speaker.clone(),
                    route,
                    self.health,
                    vec![],
                )));
            }
            Err(err) => {
                self.health = DeviceHealth::Failed;
                out.extend(self.transition_to(
                    Ph1kState::Failed,
                    now,
                    reason_codes::K_STATE_FAILED,
                ));
                let (code, msg) = match err {
                    DeviceSelectionError::NoMicAvailable => {
                        ("no_mic_available", "No microphone device is available.")
                    }
                    DeviceSelectionError::NoSpeakerAvailable => (
                        "no_speaker_available",
                        "No speaker/output device is available.",
                    ),
                };
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                    AudioDeviceId::new("UNSELECTED_MIC").unwrap(),
                    AudioDeviceId::new("UNSELECTED_SPEAKER").unwrap(),
                    DeviceRoute::Unknown,
                    self.health,
                    vec![DeviceError {
                        code: reason_codes::K_DEVICE_SELECTION_FAILED,
                        message: format!("{code}: {msg}"),
                    }],
                )));
            }
        }
        out
    }

    fn on_device_list_changed(
        &mut self,
        available: AvailableDevices,
        now: MonotonicTimeNs,
    ) -> Vec<Ph1kOutputEvent> {
        let mut out = Vec::new();

        let Some(current) = self.selection.clone() else {
            return self.on_boot(available, now);
        };

        let mic_ok = available.mics.contains(&current.mic);
        let speaker_ok = available.speakers.contains(&current.speaker);
        if mic_ok && speaker_ok {
            self.pending_selection = None;
            self.pending_selection_since = None;
            return out;
        }

        self.degradation.device_changed = true;
        self.health = DeviceHealth::Degraded;
        out.extend(self.transition_to(
            Ph1kState::DeviceSwitching,
            now,
            reason_codes::K_STATE_DEVICE_SWITCHING,
        ));

        match self.policy.select(&available) {
            Ok(sel) => {
                let pending_same = self.pending_selection.as_ref() == Some(&sel);
                if !pending_same {
                    self.pending_selection = Some(sel.clone());
                    self.pending_selection_since = Some(now);
                    let route = classify_device_route(&sel.mic, &sel.speaker);
                    out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                    out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                        sel.mic.clone(),
                        sel.speaker.clone(),
                        route,
                        self.health,
                        vec![DeviceError {
                            code: reason_codes::K_DEVICE_LIST_UNSTABLE,
                            message: "Device list changed; waiting for stability window."
                                .to_string(),
                        }],
                    )));
                    return out;
                }

                if let Some(since) = self.pending_selection_since {
                    let stable_ns = now.0.saturating_sub(since.0);
                    if stable_ns < Self::DEVICE_STABILITY_WINDOW_NS {
                        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                        return out;
                    }
                }

                if let Some(last_switch_at) = self.last_switch_at {
                    let switched_recently =
                        now.0.saturating_sub(last_switch_at.0) < Self::DEVICE_SWITCH_COOLDOWN_NS;
                    if switched_recently {
                        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                        return out;
                    }
                }

                self.selection = Some(sel.clone());
                self.pending_selection = None;
                self.pending_selection_since = None;
                self.last_switch_at = Some(now);

                let route = classify_device_route(&sel.mic, &sel.speaker);
                out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                    sel.mic.clone(),
                    sel.speaker.clone(),
                    route,
                    self.health,
                    vec![DeviceError {
                        code: reason_codes::K_DEVICE_CHANGED,
                        message: "Device changed; switched to fallback selection.".to_string(),
                    }],
                )));
                out.extend(self.transition_to(
                    Ph1kState::FullDuplexActive,
                    now,
                    reason_codes::K_STATE_FULL_DUPLEX_ACTIVE,
                ));
            }
            Err(_) => {
                self.health = DeviceHealth::Failed;
                out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                out.extend(self.transition_to(
                    Ph1kState::Failed,
                    now,
                    reason_codes::K_STATE_FAILED,
                ));
            }
        }

        out
    }

    fn on_permission_lost(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.health = DeviceHealth::Failed;
        self.degradation.capture_degraded = true;
        self.recovery_ready_since = None;
        let mut out = self.transition_to(Ph1kState::Failed, now, reason_codes::K_PERMISSION_LOST);
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_aec_unstable(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = true;
        self.health = DeviceHealth::Degraded;
        self.recovery_ready_since = None;
        let mut out = Vec::new();
        if self.state != Ph1kState::Failed {
            out.extend(self.transition_to(
                Ph1kState::Degraded,
                now,
                reason_codes::K_STATE_DEGRADED,
            ));
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_aec_stable(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = false;
        self.on_recovery_tick(now)
    }

    fn on_stream_gap(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.stream_gap_detected = true;
        self.health = DeviceHealth::Degraded;
        self.recovery_ready_since = None;
        let mut out = Vec::new();
        if self.state != Ph1kState::Failed {
            out.extend(self.transition_to(
                Ph1kState::Degraded,
                now,
                reason_codes::K_STATE_DEGRADED,
            ));
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_stream_recovered(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.stream_gap_detected = false;
        self.on_recovery_tick(now)
    }

    fn on_recovery_tick(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        let mut out = Vec::new();
        if self.state == Ph1kState::Degraded && !self.any_degradation() {
            let ready_since = self.recovery_ready_since.get_or_insert(now);
            let stable_for_ns = now.0.saturating_sub(ready_since.0);
            if stable_for_ns >= Self::RECOVERY_STABILITY_WINDOW_NS {
                self.health = DeviceHealth::Healthy;
                self.recovery_ready_since = None;
                out.extend(self.transition_to(
                    Ph1kState::FullDuplexActive,
                    now,
                    reason_codes::K_STATE_FULL_DUPLEX_ACTIVE,
                ));
            }
        } else {
            self.recovery_ready_since = None;
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn any_degradation(&self) -> bool {
        self.degradation.capture_degraded
            || self.degradation.aec_unstable
            || self.degradation.device_changed
            || self.degradation.stream_gap_detected
    }

    fn transition_to(
        &mut self,
        next: Ph1kState,
        now: MonotonicTimeNs,
        reason_code: ReasonCodeId,
    ) -> Vec<Ph1kOutputEvent> {
        if self.state == next {
            return Vec::new();
        }
        let from = self.state;
        self.state = next;
        vec![Ph1kOutputEvent::StateChanged(StateTransitionEvent::v1(
            from,
            next,
            now,
            reason_code,
        ))]
    }
}

// ---- Interruption gating (PH1.K produces InterruptCandidate; PH1.X decides the action) ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptPhraseMatcher {
    by_phrase: HashMap<String, InterruptPhraseId>,
    phrase_set_version: InterruptPhraseSetVersion,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseDetection {
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptInput {
    pub tts_playback_active: bool,
    pub capture_degraded: bool,
    pub stream_gap_detected: bool,
    pub aec_unstable: bool,
    pub voiced_window_ms: u32,
    pub vad_confidence: f32,
    pub speech_likeness: f32,
    pub echo_safe_confidence: f32,
    pub nearfield_confidence: Option<f32>,
    pub detection: Option<PhraseDetection>,
    pub t_event: MonotonicTimeNs,
}

impl InterruptPhraseMatcher {
    pub fn built_in() -> Self {
        let mut by_phrase = HashMap::new();
        for (i, p) in built_in_interrupt_phrases().into_iter().enumerate() {
            let id = InterruptPhraseId((i as u32) + 1);
            by_phrase.insert(normalize_phrase(&p), id);
        }
        Self {
            by_phrase,
            phrase_set_version: InterruptPhraseSetVersion(1),
        }
    }

    pub fn match_phrase(&self, text: &str) -> Option<InterruptPhraseId> {
        self.by_phrase.get(&normalize_phrase(text)).copied()
    }
}

pub const DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE: f32 = 0.85;

pub fn maybe_interrupt_candidate(
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<Option<InterruptCandidate>, ContractViolation> {
    maybe_interrupt_candidate_for_implementation(PH1K_IMPLEMENTATION_ID, matcher, input)
}

pub fn maybe_interrupt_candidate_for_implementation(
    implementation_id: &str,
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<Option<InterruptCandidate>, ContractViolation> {
    if implementation_id != PH1K_IMPLEMENTATION_ID {
        return Err(ContractViolation::InvalidValue {
            field: "ph1_k.implementation_id",
            reason: "unknown implementation_id",
        });
    }
    maybe_interrupt_candidate_inner(matcher, input)
}

fn maybe_interrupt_candidate_inner(
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<Option<InterruptCandidate>, ContractViolation> {
    if !input.tts_playback_active {
        return Ok(None);
    }
    if input.capture_degraded || input.stream_gap_detected {
        return Ok(None);
    }
    if input.aec_unstable {
        return Ok(None);
    }
    if input.voiced_window_ms < 80 {
        return Ok(None);
    }

    let Some(det) = input.detection else {
        return Ok(None);
    };

    let Some(phrase_id) = matcher.match_phrase(&det.text) else {
        return Ok(None);
    };

    let phrase_conf = match Confidence::new(det.confidence) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if det.confidence < DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE {
        return Ok(None);
    }

    let vad_conf = match Confidence::new(input.vad_confidence) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let speech_like = match SpeechLikeness::new(input.speech_likeness) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let echo_safe_conf = match Confidence::new(input.echo_safe_confidence) {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    let nearfield_conf = match input.nearfield_confidence {
        Some(v) => match Confidence::new(v) {
            Ok(conf) => Some(conf),
            Err(_) => return Ok(None),
        },
        None => None,
    };

    let gates = InterruptGates {
        vad_ok: input.speech_likeness >= 0.70 && input.vad_confidence >= 0.80,
        echo_safe_ok: input.echo_safe_confidence >= 0.90,
        phrase_ok: true,
        nearfield_ok: input
            .nearfield_confidence
            .map(|v| v >= 0.60)
            .unwrap_or(true),
    };

    if !(gates.vad_ok && gates.echo_safe_ok && gates.phrase_ok && gates.nearfield_ok) {
        return Ok(None);
    }

    let gate_confidences = InterruptGateConfidences {
        vad_confidence: vad_conf,
        speech_likeness: speech_like,
        echo_safe_confidence: echo_safe_conf,
        phrase_confidence: phrase_conf,
        nearfield_confidence: nearfield_conf,
    };

    let candidate = InterruptCandidate::v1(
        matcher.phrase_set_version,
        phrase_id,
        det.text,
        phrase_conf,
        gates,
        gate_confidences,
        input.t_event,
        reason_codes::K_INTERRUPT_CANDIDATE,
    )?;
    Ok(Some(candidate))
}

fn normalize_phrase(s: &str) -> String {
    s.split_whitespace()
        .map(|part| part.to_ascii_lowercase())
        .collect::<Vec<_>>()
        .join(" ")
}

fn built_in_interrupt_phrases() -> Vec<&'static str> {
    vec![
        // Core set
        "wait",
        "selene wait",
        "hold on",
        "selene hold on",
        "stop",
        "hang on",
        "excuse me",
        "just a sec",
        "hey wait",
        "hey hold on",
        "selene",
        "selene selene",
        // Additional variants (examples)
        "one second",
        "a second",
        "give me a second",
        "give me a sec",
        "hold up",
        "wait a second",
        "wait a minute",
        "pause",
        "pause please",
        "stop please",
        "stop there",
        "stop talking",
        "shut up",
        "not now",
        "later",
        "cancel",
        "cancel that",
        "back up",
        "rewind",
        "sorry",
        "sorry wait",
        "excuse me wait",
        "selene stop",
        "selene pause",
        "selene cancel",
        "hey selene",
        "listen",
        "hang on a sec",
        "just a moment",
    ]
}

fn classify_device_route(mic: &AudioDeviceId, speaker: &AudioDeviceId) -> DeviceRoute {
    let joined = format!(
        "{} {}",
        mic.as_str().to_ascii_lowercase(),
        speaker.as_str().to_ascii_lowercase()
    );
    if joined.contains("usb") {
        DeviceRoute::Usb
    } else if joined.contains("bluetooth") || joined.contains("bt") {
        DeviceRoute::Bluetooth
    } else if joined.contains("virtual") {
        DeviceRoute::Virtual
    } else if joined.contains("built") || joined.contains("internal") {
        DeviceRoute::BuiltIn
    } else {
        DeviceRoute::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    #[test]
    fn device_policy_prefers_user_override_then_last_known_good_then_default_then_lexical() {
        let available = AvailableDevices {
            mics: vec![dev("mic_b"), dev("mic_a")],
            speakers: vec![dev("spk_b"), dev("spk_a")],
            system_default_mic: Some(dev("mic_b")),
            system_default_speaker: Some(dev("spk_b")),
        };

        let policy = DevicePolicy {
            preference: DevicePreference {
                preferred_mic: Some(dev("mic_a")),
                preferred_speaker: Some(dev("spk_a")),
                ..Default::default()
            },
        };

        let sel = policy.select(&available).unwrap();
        assert_eq!(sel.mic.as_str(), "mic_a");
        assert_eq!(sel.speaker.as_str(), "spk_a");
    }

    #[test]
    fn runtime_requires_device_stability_window_before_switching() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
        };
        let mut rt = Ph1kRuntime::new(policy);

        let out = rt.handle(Ph1kEvent::Boot {
            available: AvailableDevices {
                mics: vec![dev("mic_a")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(0),
        });
        assert!(out
            .iter()
            .any(|e| matches!(e, Ph1kOutputEvent::DeviceState(_))));

        // First event starts the stability window; no immediate switch.
        let _ = rt.handle(Ph1kEvent::DeviceListChanged {
            available: AvailableDevices {
                mics: vec![dev("mic_b")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(100_000_000),
        });
        assert_eq!(rt.selection().unwrap().mic.as_str(), "mic_a");

        // Same list after stability window allows switch.
        let _ = rt.handle(Ph1kEvent::DeviceListChanged {
            available: AvailableDevices {
                mics: vec![dev("mic_b")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(500_000_000),
        });

        assert!(rt.degradation.device_changed);
        assert_eq!(rt.health, DeviceHealth::Degraded);
        assert_eq!(rt.selection().unwrap().mic.as_str(), "mic_b");
    }

    #[test]
    fn interrupt_candidate_requires_confidence_and_all_gates() {
        let matcher = InterruptPhraseMatcher::built_in();
        let t_event = MonotonicTimeNs(123);

        let none = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                capture_degraded: false,
                stream_gap_detected: false,
                aec_unstable: false,
                voiced_window_ms: 100,
                vad_confidence: 0.9,
                speech_likeness: 0.9,
                echo_safe_confidence: 0.89,
                nearfield_confidence: Some(0.9),
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event,
            },
        )
        .unwrap();
        assert!(none.is_none());

        let some = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                capture_degraded: false,
                stream_gap_detected: false,
                aec_unstable: false,
                voiced_window_ms: 100,
                vad_confidence: 0.9,
                speech_likeness: 0.9,
                echo_safe_confidence: 0.95,
                nearfield_confidence: Some(0.9),
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event,
            },
        )
        .unwrap();
        assert!(some.is_some());
    }

    #[test]
    fn interrupt_candidate_fails_closed_on_degradation() {
        let matcher = InterruptPhraseMatcher::built_in();
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                capture_degraded: true,
                stream_gap_detected: false,
                aec_unstable: false,
                voiced_window_ms: 100,
                vad_confidence: 0.9,
                speech_likeness: 0.9,
                echo_safe_confidence: 0.95,
                nearfield_confidence: Some(0.9),
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event: MonotonicTimeNs(1),
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_impl_01_unknown_implementation_fails_closed() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
        };
        let mut rt = Ph1kRuntime::new(policy);
        let out = rt.handle_for_implementation(
            "PH1.K.999",
            Ph1kEvent::StartFullDuplex {
                now: MonotonicTimeNs(1),
            },
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_k_impl_02_interrupt_unknown_implementation_fails_closed() {
        let matcher = InterruptPhraseMatcher::built_in();
        let out = maybe_interrupt_candidate_for_implementation(
            "PH1.K.999",
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                capture_degraded: false,
                stream_gap_detected: false,
                aec_unstable: false,
                voiced_window_ms: 100,
                vad_confidence: 0.9,
                speech_likeness: 0.9,
                echo_safe_confidence: 0.95,
                nearfield_confidence: Some(0.9),
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event: MonotonicTimeNs(1),
            },
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "ph1_k.implementation_id",
                reason: "unknown implementation_id",
            })
        ));
    }

    #[test]
    fn at_k_impl_03_active_implementation_list_is_locked() {
        assert_eq!(PH1_K_ENGINE_ID, "PH1.K");
        assert_eq!(PH1_K_ACTIVE_IMPLEMENTATION_IDS, &["PH1.K.001"]);
    }
}
