#![forbid(unsafe_code)]

use std::collections::HashMap;

use selene_kernel_contracts::ph1k::{
    AudioDeviceId, DegradationFlags, DeviceError, DeviceHealth, DeviceState, InterruptCandidate,
    InterruptGates, InterruptPhraseId,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.K reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const K_DEVICE_SELECTION_FAILED: ReasonCodeId = ReasonCodeId(0x4B00_0001);
    pub const K_DEVICE_CHANGED: ReasonCodeId = ReasonCodeId(0x4B00_0002);
    pub const K_PERMISSION_LOST: ReasonCodeId = ReasonCodeId(0x4B00_0003);
    pub const K_AEC_UNSTABLE: ReasonCodeId = ReasonCodeId(0x4B00_0004);
    pub const K_STREAM_GAP: ReasonCodeId = ReasonCodeId(0x4B00_0005);
    pub const K_INTERRUPT_CANDIDATE: ReasonCodeId = ReasonCodeId(0x4B00_0006);
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1kState {
    Init,
    Ready,
    FullDuplexActive,
    DeviceSwitching,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1kEvent {
    Boot { available: AvailableDevices },
    StartFullDuplex,
    Stop,
    DeviceListChanged { available: AvailableDevices },
    PermissionLost,
    AecUnstable,
    AecStable,
    StreamGapDetected,
    StreamRecovered,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1kOutputEvent {
    StateChanged { from: Ph1kState, to: Ph1kState },
    DeviceState(DeviceState),
    DegradationFlags(DegradationFlags),
}

#[derive(Debug, Clone)]
pub struct Ph1kRuntime {
    policy: DevicePolicy,
    state: Ph1kState,
    selection: Option<DeviceSelection>,
    degradation: DegradationFlags,
    health: DeviceHealth,
}

impl Ph1kRuntime {
    pub fn new(policy: DevicePolicy) -> Self {
        Self {
            policy,
            state: Ph1kState::Init,
            selection: None,
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
        match event {
            Ph1kEvent::Boot { available } => self.on_boot(available),
            Ph1kEvent::StartFullDuplex => self.transition_to(Ph1kState::FullDuplexActive),
            Ph1kEvent::Stop => self.transition_to(Ph1kState::Ready),
            Ph1kEvent::DeviceListChanged { available } => self.on_device_list_changed(available),
            Ph1kEvent::PermissionLost => self.on_permission_lost(),
            Ph1kEvent::AecUnstable => self.on_aec_unstable(),
            Ph1kEvent::AecStable => self.on_aec_stable(),
            Ph1kEvent::StreamGapDetected => self.on_stream_gap(),
            Ph1kEvent::StreamRecovered => self.on_stream_recovered(),
        }
    }

    fn on_boot(&mut self, available: AvailableDevices) -> Vec<Ph1kOutputEvent> {
        let mut out = Vec::new();
        match self.policy.select(&available) {
            Ok(sel) => {
                self.selection = Some(sel.clone());
                out.extend(self.transition_to(Ph1kState::Ready));
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1(
                    sel.mic,
                    sel.speaker,
                    self.health,
                    vec![],
                )));
            }
            Err(err) => {
                self.health = DeviceHealth::Failed;
                out.extend(self.transition_to(Ph1kState::Failed));
                let (code, msg) = match err {
                    DeviceSelectionError::NoMicAvailable => {
                        ("no_mic_available", "No microphone device is available.")
                    }
                    DeviceSelectionError::NoSpeakerAvailable => (
                        "no_speaker_available",
                        "No speaker/output device is available.",
                    ),
                };
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1(
                    // Placeholder IDs; the point is to surface the failure deterministically.
                    AudioDeviceId::new("UNSELECTED_MIC").unwrap(),
                    AudioDeviceId::new("UNSELECTED_SPEAKER").unwrap(),
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

    fn on_device_list_changed(&mut self, available: AvailableDevices) -> Vec<Ph1kOutputEvent> {
        let mut out = Vec::new();

        let Some(current) = self.selection.clone() else {
            // Treat as boot-time selection.
            return self.on_boot(available);
        };

        let mic_ok = available.mics.contains(&current.mic);
        let speaker_ok = available.speakers.contains(&current.speaker);
        if mic_ok && speaker_ok {
            return out;
        }

        self.degradation.device_changed = true;
        self.health = DeviceHealth::Degraded;

        out.extend(self.transition_to(Ph1kState::DeviceSwitching));

        match self.policy.select(&available) {
            Ok(sel) => {
                self.selection = Some(sel.clone());
                out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1(
                    sel.mic,
                    sel.speaker,
                    self.health,
                    vec![DeviceError {
                        code: reason_codes::K_DEVICE_CHANGED,
                        message: "Device changed; switched to fallback selection.".to_string(),
                    }],
                )));
                out.extend(self.transition_to(Ph1kState::FullDuplexActive));
            }
            Err(_) => {
                self.health = DeviceHealth::Failed;
                out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
                out.extend(self.transition_to(Ph1kState::Failed));
            }
        }

        out
    }

    fn on_permission_lost(&mut self) -> Vec<Ph1kOutputEvent> {
        self.health = DeviceHealth::Failed;
        self.degradation.capture_degraded = true;
        let mut out = self.transition_to(Ph1kState::Failed);
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_aec_unstable(&mut self) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = true;
        self.health = DeviceHealth::Degraded;
        let mut out = Vec::new();
        if self.state != Ph1kState::Failed {
            out.extend(self.transition_to(Ph1kState::Degraded));
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_aec_stable(&mut self) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = false;
        let mut out = Vec::new();
        if self.state == Ph1kState::Degraded && !self.any_degradation() {
            self.health = DeviceHealth::Healthy;
            out.extend(self.transition_to(Ph1kState::FullDuplexActive));
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_stream_gap(&mut self) -> Vec<Ph1kOutputEvent> {
        self.degradation.stream_gap_detected = true;
        self.health = DeviceHealth::Degraded;
        let mut out = Vec::new();
        if self.state != Ph1kState::Failed {
            out.extend(self.transition_to(Ph1kState::Degraded));
        }
        out.push(Ph1kOutputEvent::DegradationFlags(self.degradation));
        out
    }

    fn on_stream_recovered(&mut self) -> Vec<Ph1kOutputEvent> {
        self.degradation.stream_gap_detected = false;
        let mut out = Vec::new();
        if self.state == Ph1kState::Degraded && !self.any_degradation() {
            self.health = DeviceHealth::Healthy;
            out.extend(self.transition_to(Ph1kState::FullDuplexActive));
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

    fn transition_to(&mut self, next: Ph1kState) -> Vec<Ph1kOutputEvent> {
        if self.state == next {
            return Vec::new();
        }
        let from = self.state;
        self.state = next;
        vec![Ph1kOutputEvent::StateChanged { from, to: next }]
    }
}

// ---- Interruption gating (PH1.K produces InterruptCandidate; PH1.X decides the action) ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptPhraseMatcher {
    by_phrase: HashMap<String, InterruptPhraseId>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseDetection {
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptInput {
    pub tts_playback_active: bool,
    pub vad_ok: bool,
    pub echo_safe_ok: bool,
    pub nearfield_ok: bool,
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
        Self { by_phrase }
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
    if !input.tts_playback_active {
        return Ok(None);
    }

    let Some(det) = input.detection else {
        return Ok(None);
    };

    let Some(phrase_id) = matcher.match_phrase(&det.text) else {
        return Ok(None);
    };

    let phrase_conf = selene_kernel_contracts::ph1k::Confidence::new(det.confidence)?;
    if det.confidence < DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE {
        return Ok(None);
    }

    let gates = InterruptGates {
        vad_ok: input.vad_ok,
        echo_safe_ok: input.echo_safe_ok,
        phrase_ok: true,
        nearfield_ok: input.nearfield_ok,
    };

    if !(gates.vad_ok && gates.echo_safe_ok && gates.phrase_ok && gates.nearfield_ok) {
        return Ok(None);
    }

    let candidate = InterruptCandidate::v1(
        phrase_id,
        det.text,
        phrase_conf,
        gates,
        input.t_event,
        reason_codes::K_INTERRUPT_CANDIDATE,
    )?;
    Ok(Some(candidate))
}

fn normalize_phrase(s: &str) -> String {
    s.trim().to_ascii_lowercase()
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
    fn runtime_marks_device_changed_and_switches_when_selection_disappears() {
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
        });
        assert!(out
            .iter()
            .any(|e| matches!(e, Ph1kOutputEvent::DeviceState(_))));

        let out = rt.handle(Ph1kEvent::DeviceListChanged {
            available: AvailableDevices {
                mics: vec![dev("mic_b")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
        });

        assert!(out
            .iter()
            .any(|e| matches!(e, Ph1kOutputEvent::DegradationFlags(_))));
        assert!(rt.degradation.device_changed);
        assert_eq!(rt.health, DeviceHealth::Degraded);
        assert_eq!(rt.selection().unwrap().mic.as_str(), "mic_b");
    }

    #[test]
    fn interrupt_candidate_requires_all_gates_and_min_confidence() {
        let matcher = InterruptPhraseMatcher::built_in();
        let t_event = MonotonicTimeNs(123);

        // Not playing -> no interrupt.
        let none = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: false,
                vad_ok: true,
                echo_safe_ok: true,
                nearfield_ok: true,
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event,
            },
        )
        .unwrap();
        assert!(none.is_none());

        // Low confidence -> no interrupt.
        let none = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                vad_ok: true,
                echo_safe_ok: true,
                nearfield_ok: true,
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.2,
                }),
                t_event,
            },
        )
        .unwrap();
        assert!(none.is_none());

        // Echo not safe -> no interrupt.
        let none = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                vad_ok: true,
                echo_safe_ok: false,
                nearfield_ok: true,
                detection: Some(PhraseDetection {
                    text: "stop".to_string(),
                    confidence: 0.99,
                }),
                t_event,
            },
        )
        .unwrap();
        assert!(none.is_none());

        // All gates pass -> interrupt candidate.
        let some = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                vad_ok: true,
                echo_safe_ok: true,
                nearfield_ok: true,
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
}
