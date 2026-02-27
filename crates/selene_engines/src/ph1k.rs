#![forbid(unsafe_code)]

use std::collections::HashMap;

use selene_kernel_contracts::ph1c::Ph1kToPh1cHandoff;
use selene_kernel_contracts::ph1feedback::{
    FeedbackConfidenceBucket, FeedbackEventType, FeedbackSignalTarget,
};
use selene_kernel_contracts::ph1k::{
    classify_vad_decision_confidence_band, normalize_interrupt_phrase_for_locale,
    AdaptiveThresholdPolicyInput, AdvancedAudioQualityMetrics, AudioDeviceId, CaptureQualityClass,
    Confidence, DegradationClassBundle, DegradationFlags, DeviceError, DeviceHealth,
    DeviceReliabilityScoreInput, DeviceRoute, DeviceState, DuplexFrame, DuplexFrameId,
    EchoRiskClass, InterruptCandidate, InterruptCandidateConfidenceBand,
    InterruptDegradationContext, InterruptGateConfidences, InterruptGates,
    InterruptLexiconPolicyBinding, InterruptLocaleTag, InterruptPhraseId,
    InterruptPhraseSetVersion, InterruptPolicyProfileId, InterruptRiskContextClass,
    InterruptSpeechWindowMetrics, InterruptSubjectRelationConfidenceBundle,
    InterruptTenantProfileId, InterruptTimingMarkers, JitterClockRecoveryPolicy,
    NetworkStabilityClass, Ph1kState, RecoverabilityClass, SpeechLikeness, StateTransitionEvent,
    TimingStats, VadDecisionConfidenceBand, PH1K_IMPLEMENTATION_ID,
    PH1K_INTERRUPT_LOCALE_TAG_DEFAULT, PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT,
    PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT,
};
use selene_kernel_contracts::ph1x::Ph1kToPh1xInterruptHandoff;
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};

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
    pub const K_INTERRUPT_LEXICAL_TRIGGER_ACCEPTED: ReasonCodeId = ReasonCodeId(0x4B00_000D);
    pub const K_INTERRUPT_LEXICAL_TRIGGER_REJECTED: ReasonCodeId = ReasonCodeId(0x4B00_000E);
    pub const K_INTERRUPT_NOISE_GATE_REJECTED: ReasonCodeId = ReasonCodeId(0x4B00_000F);
    pub const K_INTERRUPT_CANDIDATE_EMITTED_HIGH: ReasonCodeId = ReasonCodeId(0x4B00_0010);
    pub const K_INTERRUPT_CANDIDATE_EMITTED_MEDIUM: ReasonCodeId = ReasonCodeId(0x4B00_0011);
    pub const K_INTERRUPT_CANDIDATE_EMITTED_LOW: ReasonCodeId = ReasonCodeId(0x4B00_0012);
    pub const K_INTERRUPT_FEEDBACK_FALSE_LEXICAL_TRIGGER: ReasonCodeId = ReasonCodeId(0x4B00_0013);
    pub const K_INTERRUPT_FEEDBACK_MISSED_LEXICAL_TRIGGER: ReasonCodeId = ReasonCodeId(0x4B00_0014);
    pub const K_INTERRUPT_FEEDBACK_WRONG_CONFIDENCE_BAND: ReasonCodeId = ReasonCodeId(0x4B00_0015);
    pub const K_FAILOVER_RELIABILITY_SELECTED: ReasonCodeId = ReasonCodeId(0x4B00_0016);
    pub const K_CALIBRATION_AUTO_TUNE_APPLIED: ReasonCodeId = ReasonCodeId(0x4B00_0017);
    pub const K_CALIBRATION_AUTO_TUNE_ROLLBACK: ReasonCodeId = ReasonCodeId(0x4B00_0018);
    pub const K_DEGRADATION_CLASS_BUNDLE_EMITTED: ReasonCodeId = ReasonCodeId(0x4B00_0019);
    pub const K_PH1C_HANDOFF_STRATEGY_EMITTED: ReasonCodeId = ReasonCodeId(0x4B00_001A);
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
pub struct DeviceReliabilityProfile {
    pub failures_24h: u32,
    pub recoveries_24h: u32,
    pub mean_recovery_ms: u32,
    pub reliability_bp: u16,
}

impl DeviceReliabilityProfile {
    pub fn stable_default() -> Self {
        Self {
            failures_24h: 0,
            recoveries_24h: 0,
            mean_recovery_ms: 0,
            reliability_bp: 10_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePolicy {
    pub preference: DevicePreference,
    pub reliability_profiles: HashMap<AudioDeviceId, DeviceReliabilityProfile>,
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
            &self.reliability_profiles,
        )
        .ok_or(DeviceSelectionError::NoMicAvailable)?;

        let speaker = select_one(
            &available.speakers,
            &self.preference.preferred_speaker,
            &self.preference.last_known_good_speaker,
            &available.system_default_speaker,
            &self.reliability_profiles,
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
    reliability_profiles: &HashMap<AudioDeviceId, DeviceReliabilityProfile>,
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
    devices
        .iter()
        .min_by_key(|d| {
            let reliability = reliability_profiles
                .get(*d)
                .map(|p| p.reliability_bp)
                .unwrap_or(0);
            (u16::MAX.saturating_sub(reliability), d.as_str().to_string())
        })
        .cloned()
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
    DegradationState(InterruptDegradationContext),
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalibrationProfile {
    pub mic_device_id: AudioDeviceId,
    pub speaker_device_id: AudioDeviceId,
    pub mic_gain_db: f32,
    pub speaker_gain_db: f32,
    pub tune_steps: u16,
    pub rollback_count: u16,
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
    calibration_profile: Option<CalibrationProfile>,
}

impl Ph1kRuntime {
    const DEVICE_STABILITY_WINDOW_NS: u64 = 300_000_000;
    const DEVICE_SWITCH_COOLDOWN_NS: u64 = 2_000_000_000;
    const RECOVERY_STABILITY_WINDOW_NS: u64 = 500_000_000;
    const CALIBRATION_GAIN_STEP_DB: f32 = 0.5;
    const CALIBRATION_GAIN_MIN_DB: f32 = -6.0;
    const CALIBRATION_GAIN_MAX_DB: f32 = 6.0;

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
            calibration_profile: None,
        }
    }

    pub fn state(&self) -> Ph1kState {
        self.state
    }

    pub fn selection(&self) -> Option<&DeviceSelection> {
        self.selection.as_ref()
    }

    pub fn calibration_profile(&self) -> Option<&CalibrationProfile> {
        self.calibration_profile.as_ref()
    }

    fn current_degradation_state(&self) -> InterruptDegradationContext {
        InterruptDegradationContext {
            capture_degraded: self.degradation.capture_degraded,
            aec_unstable: self.degradation.aec_unstable,
            device_changed: self.degradation.device_changed,
            stream_gap_detected: self.degradation.stream_gap_detected,
            // Keep richer class state strictly derivable from append-only flags.
            class_bundle: DegradationClassBundle::from_flags(
                self.degradation.capture_degraded,
                self.degradation.aec_unstable,
                self.degradation.device_changed,
                self.degradation.stream_gap_detected,
            ),
        }
    }

    fn push_degradation_state(&self, out: &mut Vec<Ph1kOutputEvent>) {
        out.push(Ph1kOutputEvent::DegradationState(
            self.current_degradation_state(),
        ));
    }

    fn reset_calibration_for_selection(&mut self) {
        let Some(sel) = self.selection.clone() else {
            self.calibration_profile = None;
            return;
        };
        self.calibration_profile = Some(CalibrationProfile {
            mic_device_id: sel.mic,
            speaker_device_id: sel.speaker,
            mic_gain_db: 0.0,
            speaker_gain_db: 0.0,
            tune_steps: 0,
            rollback_count: 0,
        });
    }

    fn apply_calibration_autotune_for_aec_unstable(&mut self) -> bool {
        let Some(profile) = self.calibration_profile.as_mut() else {
            return false;
        };
        let next_mic = profile.mic_gain_db - Self::CALIBRATION_GAIN_STEP_DB;
        let next_speaker = profile.speaker_gain_db - Self::CALIBRATION_GAIN_STEP_DB;
        if next_mic < Self::CALIBRATION_GAIN_MIN_DB || next_speaker < Self::CALIBRATION_GAIN_MIN_DB
        {
            // Fail-closed rollback: restore neutral profile instead of over-tuning.
            profile.mic_gain_db = 0.0;
            profile.speaker_gain_db = 0.0;
            profile.tune_steps = 0;
            profile.rollback_count = profile.rollback_count.saturating_add(1);
            return true;
        }
        profile.mic_gain_db = next_mic;
        profile.speaker_gain_db = next_speaker;
        profile.tune_steps = profile.tune_steps.saturating_add(1);
        false
    }

    fn stabilize_calibration_after_recovery(&mut self) {
        if let Some(profile) = self.calibration_profile.as_mut() {
            profile.mic_gain_db =
                (profile.mic_gain_db + (Self::CALIBRATION_GAIN_STEP_DB / 2.0)).min(0.0);
            profile.speaker_gain_db =
                (profile.speaker_gain_db + (Self::CALIBRATION_GAIN_STEP_DB / 2.0)).min(0.0);
            profile.mic_gain_db = profile
                .mic_gain_db
                .clamp(Self::CALIBRATION_GAIN_MIN_DB, Self::CALIBRATION_GAIN_MAX_DB);
            profile.speaker_gain_db = profile
                .speaker_gain_db
                .clamp(Self::CALIBRATION_GAIN_MIN_DB, Self::CALIBRATION_GAIN_MAX_DB);
        }
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
                self.reset_calibration_for_selection();
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
                    self.push_degradation_state(&mut out);
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
                        self.push_degradation_state(&mut out);
                        return out;
                    }
                }

                if let Some(last_switch_at) = self.last_switch_at {
                    let switched_recently =
                        now.0.saturating_sub(last_switch_at.0) < Self::DEVICE_SWITCH_COOLDOWN_NS;
                    if switched_recently {
                        self.push_degradation_state(&mut out);
                        return out;
                    }
                }

                self.selection = Some(sel.clone());
                self.pending_selection = None;
                self.pending_selection_since = None;
                self.last_switch_at = Some(now);
                self.reset_calibration_for_selection();

                let route = classify_device_route(&sel.mic, &sel.speaker);
                self.push_degradation_state(&mut out);
                out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                    sel.mic.clone(),
                    sel.speaker.clone(),
                    route,
                    self.health,
                    vec![DeviceError {
                        code: reason_codes::K_FAILOVER_RELIABILITY_SELECTED,
                        message:
                            "Device changed; switched to reliability-ranked fallback selection."
                                .to_string(),
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
                self.push_degradation_state(&mut out);
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
        self.push_degradation_state(&mut out);
        out
    }

    fn on_aec_unstable(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = true;
        self.health = DeviceHealth::Degraded;
        self.recovery_ready_since = None;
        let rolled_back = self.apply_calibration_autotune_for_aec_unstable();
        let mut out = Vec::new();
        if self.state != Ph1kState::Failed {
            out.extend(self.transition_to(
                Ph1kState::Degraded,
                now,
                reason_codes::K_STATE_DEGRADED,
            ));
        }
        if let Some(sel) = self.selection.clone() {
            let route = classify_device_route(&sel.mic, &sel.speaker);
            let (code, message) = if rolled_back {
                (
                    reason_codes::K_CALIBRATION_AUTO_TUNE_ROLLBACK,
                    "AEC unstable; calibration auto-tune exceeded safe bounds and rolled back."
                        .to_string(),
                )
            } else if let Some(profile) = self.calibration_profile.as_ref() {
                (
                    reason_codes::K_CALIBRATION_AUTO_TUNE_APPLIED,
                    format!(
                        "AEC unstable; auto-tuned calibration (mic_gain_db={:.2}, speaker_gain_db={:.2}, steps={}).",
                        profile.mic_gain_db, profile.speaker_gain_db, profile.tune_steps
                    ),
                )
            } else {
                (
                    reason_codes::K_CALIBRATION_AUTO_TUNE_APPLIED,
                    "AEC unstable; calibration auto-tune attempted without active profile."
                        .to_string(),
                )
            };
            out.push(Ph1kOutputEvent::DeviceState(DeviceState::v1_with_route(
                sel.mic,
                sel.speaker,
                route,
                self.health,
                vec![DeviceError { code, message }],
            )));
        }
        self.push_degradation_state(&mut out);
        out
    }

    fn on_aec_stable(&mut self, now: MonotonicTimeNs) -> Vec<Ph1kOutputEvent> {
        self.degradation.aec_unstable = false;
        self.stabilize_calibration_after_recovery();
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
        self.push_degradation_state(&mut out);
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
        self.push_degradation_state(&mut out);
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
    profiles: HashMap<InterruptPolicyProfileId, InterruptLexiconPolicyProfile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct InterruptLexiconPolicyProfile {
    tenant_profile_id: InterruptTenantProfileId,
    phrase_set_version: InterruptPhraseSetVersion,
    by_locale: HashMap<InterruptLocaleTag, HashMap<String, InterruptPhraseId>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PhraseDetection {
    pub text: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptInput {
    pub lexicon_policy_binding: InterruptLexiconPolicyBinding,
    pub adaptive_policy_input: AdaptiveThresholdPolicyInput,
    pub tts_playback_active: bool,
    pub capture_degraded: bool,
    pub stream_gap_detected: bool,
    pub aec_unstable: bool,
    pub device_changed: bool,
    pub voiced_window_ms: u32,
    pub vad_confidence: f32,
    pub acoustic_confidence: f32,
    pub prosody_confidence: f32,
    pub speech_likeness: f32,
    pub echo_safe_confidence: f32,
    pub nearfield_confidence: Option<f32>,
    pub detection: Option<PhraseDetection>,
    pub t_event: MonotonicTimeNs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct InterruptDecisionTrace {
    pub candidate: Option<InterruptCandidate>,
    pub reason_code: ReasonCodeId,
    pub lexical_trigger_accepted: bool,
    pub noise_gate_rejected: bool,
    pub vad_confidence_band: Option<VadDecisionConfidenceBand>,
    pub adaptive_noise_class: Option<InterruptNoiseClass>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InterruptNoiseClass {
    Clean,
    Elevated,
    Severe,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct AdaptiveThresholdProfile {
    min_phrase_confidence: f32,
    min_vad_confidence: f32,
    min_acoustic_confidence: f32,
    min_prosody_confidence: f32,
    min_speech_likeness: f32,
    min_echo_safe_confidence: f32,
    min_nearfield_confidence: f32,
    min_voiced_window_ms: u32,
    min_reliability_score: f32,
    clock_recovery_policy: JitterClockRecoveryPolicy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptFeedbackSignalKind {
    FalseLexicalTrigger,
    MissedLexicalTrigger,
    WrongConfidenceBand,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptFeedbackSignal {
    pub event_type: FeedbackEventType,
    pub reason_code: ReasonCodeId,
    pub confidence_bucket: FeedbackConfidenceBucket,
    pub route_targets: [FeedbackSignalTarget; 2],
}

pub fn build_interrupt_feedback_signal(
    kind: InterruptFeedbackSignalKind,
    observed_band: Option<InterruptCandidateConfidenceBand>,
) -> InterruptFeedbackSignal {
    let (event_type, reason_code) = match kind {
        InterruptFeedbackSignalKind::FalseLexicalTrigger => (
            FeedbackEventType::BargeIn,
            reason_codes::K_INTERRUPT_FEEDBACK_FALSE_LEXICAL_TRIGGER,
        ),
        InterruptFeedbackSignalKind::MissedLexicalTrigger => (
            FeedbackEventType::BargeIn,
            reason_codes::K_INTERRUPT_FEEDBACK_MISSED_LEXICAL_TRIGGER,
        ),
        InterruptFeedbackSignalKind::WrongConfidenceBand => (
            FeedbackEventType::UserCorrection,
            reason_codes::K_INTERRUPT_FEEDBACK_WRONG_CONFIDENCE_BAND,
        ),
    };
    let confidence_bucket = match observed_band {
        Some(InterruptCandidateConfidenceBand::High) => FeedbackConfidenceBucket::High,
        Some(InterruptCandidateConfidenceBand::Medium) => FeedbackConfidenceBucket::Med,
        Some(InterruptCandidateConfidenceBand::Low) => FeedbackConfidenceBucket::Low,
        None => FeedbackConfidenceBucket::Unknown,
    };
    InterruptFeedbackSignal {
        event_type,
        reason_code,
        confidence_bucket,
        route_targets: [
            FeedbackSignalTarget::LearnPackage,
            FeedbackSignalTarget::PaeScorecard,
        ],
    }
}

impl InterruptPhraseMatcher {
    pub fn built_in() -> Self {
        let policy_profile_id =
            InterruptPolicyProfileId::new(PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT)
                .expect("default interrupt policy profile id must be valid");
        let tenant_profile_id =
            InterruptTenantProfileId::new(PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT)
                .expect("default interrupt tenant profile id must be valid");
        let mut by_locale = HashMap::new();
        let mut next_phrase_id = 1u32;
        for (locale, phrases) in built_in_interrupt_phrases_by_locale() {
            let locale_tag = InterruptLocaleTag::new(locale)
                .expect("built-in interrupt locale tag must be valid");
            let mut by_phrase = HashMap::new();
            for phrase in phrases {
                let normalized = normalize_interrupt_phrase_for_locale(&locale_tag, phrase)
                    .expect("built-in interrupt phrase must normalize");
                by_phrase.insert(normalized, InterruptPhraseId(next_phrase_id));
                next_phrase_id = next_phrase_id.saturating_add(1);
            }
            by_locale.insert(locale_tag, by_phrase);
        }
        let mut profiles = HashMap::new();
        profiles.insert(
            policy_profile_id,
            InterruptLexiconPolicyProfile {
                tenant_profile_id,
                phrase_set_version: InterruptPhraseSetVersion(1),
                by_locale,
            },
        );
        Self { profiles }
    }

    pub fn default_policy_binding(&self) -> InterruptLexiconPolicyBinding {
        InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new(PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT)
                .expect("default interrupt policy profile id must be valid"),
            InterruptTenantProfileId::new(PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT)
                .expect("default interrupt tenant profile id must be valid"),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT)
                .expect("default interrupt locale tag must be valid"),
        )
        .expect("default interrupt lexicon policy binding must be valid")
    }

    pub fn match_phrase(
        &self,
        binding: &InterruptLexiconPolicyBinding,
        text: &str,
    ) -> Result<Option<(InterruptPhraseSetVersion, InterruptPhraseId)>, ContractViolation> {
        let Some(profile) = self.profiles.get(&binding.policy_profile_id) else {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_input.lexicon_policy_binding.policy_profile_id",
                reason: "unknown interrupt lexicon policy profile",
            });
        };
        if profile.tenant_profile_id != binding.tenant_profile_id {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_input.lexicon_policy_binding.tenant_profile_id",
                reason: "does not match policy profile binding",
            });
        }
        let Some(by_phrase) = profile.by_locale.get(&binding.locale_tag) else {
            return Ok(None);
        };
        let normalized =
            normalize_interrupt_phrase_for_locale(&binding.locale_tag, text).map_err(|_| {
                ContractViolation::InvalidValue {
                    field: "interrupt_input.detection.text",
                    reason: "invalid phrase normalization",
                }
            })?;
        Ok(by_phrase
            .get(&normalized)
            .copied()
            .map(|phrase_id| (profile.phrase_set_version, phrase_id)))
    }

    pub fn register_profile_from_phrases(
        &mut self,
        policy_profile_id: InterruptPolicyProfileId,
        tenant_profile_id: InterruptTenantProfileId,
        phrase_set_version: InterruptPhraseSetVersion,
        locale_phrases: Vec<(InterruptLocaleTag, Vec<String>)>,
    ) -> Result<(), ContractViolation> {
        if locale_phrases.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "interrupt_profile.locale_phrases",
                reason: "must not be empty",
            });
        }
        let mut by_locale: HashMap<InterruptLocaleTag, HashMap<String, InterruptPhraseId>> =
            HashMap::new();
        let mut next_phrase_id = 1u32;
        for (locale_tag, phrases) in locale_phrases {
            if phrases.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "interrupt_profile.locale_phrases[].phrases",
                    reason: "must not be empty",
                });
            }
            let mut by_phrase = HashMap::new();
            for phrase in phrases {
                let normalized = normalize_interrupt_phrase_for_locale(&locale_tag, &phrase)
                    .map_err(|_| ContractViolation::InvalidValue {
                        field: "interrupt_profile.locale_phrases[].phrases[]",
                        reason: "invalid phrase normalization",
                    })?;
                by_phrase.entry(normalized).or_insert_with(|| {
                    let phrase_id = InterruptPhraseId(next_phrase_id);
                    next_phrase_id = next_phrase_id.saturating_add(1);
                    phrase_id
                });
            }
            by_locale.insert(locale_tag, by_phrase);
        }
        self.profiles.insert(
            policy_profile_id,
            InterruptLexiconPolicyProfile {
                tenant_profile_id,
                phrase_set_version,
                by_locale,
            },
        );
        Ok(())
    }

    pub fn has_profile(&self, policy_profile_id: &InterruptPolicyProfileId) -> bool {
        self.profiles.contains_key(policy_profile_id)
    }
}

pub const DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE: f32 = 0.85;
pub const DEFAULT_MIN_INTERRUPT_VOICED_WINDOW_MS: u32 = 80;
pub const DEFAULT_MIN_INTERRUPT_VAD_CONFIDENCE: f32 = 0.80;
pub const DEFAULT_MIN_INTERRUPT_ACOUSTIC_CONFIDENCE: f32 = 0.78;
pub const DEFAULT_MIN_INTERRUPT_PROSODY_CONFIDENCE: f32 = 0.72;
pub const DEFAULT_MIN_INTERRUPT_SPEECH_LIKENESS: f32 = 0.70;
pub const DEFAULT_MIN_INTERRUPT_ECHO_SAFE_CONFIDENCE: f32 = 0.90;
pub const DEFAULT_MIN_INTERRUPT_NEARFIELD_CONFIDENCE: f32 = 0.60;
pub const DEFAULT_MIN_INTERRUPT_DEVICE_RELIABILITY_SCORE: f32 = 0.50;

pub fn default_adaptive_policy_input(device_route: DeviceRoute) -> AdaptiveThresholdPolicyInput {
    AdaptiveThresholdPolicyInput {
        device_route,
        quality_metrics: AdvancedAudioQualityMetrics::v1(28.0, 0.02, 45.0, 0.5, 0.08, 22.0)
            .expect("default PH1.K quality metrics must be valid"),
        device_reliability: DeviceReliabilityScoreInput::v1(
            0,
            6,
            800,
            Confidence::new(0.95).expect("default reliability score must be valid"),
        )
        .expect("default PH1.K reliability metrics must be valid"),
        timing_stats: TimingStats::v1(8.0, 18.0, 20.0, 0, 0),
        capture_to_handoff_latency_ms: 90,
    }
}

fn classify_noise_class(
    quality: &AdvancedAudioQualityMetrics,
    degraded: bool,
    stream_gap: bool,
    aec_unstable: bool,
    device_changed: bool,
) -> InterruptNoiseClass {
    if degraded
        || stream_gap
        || aec_unstable
        || device_changed
        || quality.packet_loss_pct >= 8.0
        || quality.snr_db < 10.0
        || quality.clipping_ratio >= 0.12
    {
        InterruptNoiseClass::Severe
    } else if quality.packet_loss_pct >= 3.0
        || quality.snr_db < 18.0
        || quality.clipping_ratio >= 0.05
        || quality.double_talk_score >= 0.65
    {
        InterruptNoiseClass::Elevated
    } else {
        InterruptNoiseClass::Clean
    }
}

fn derive_degradation_class_bundle(
    quality: &AdvancedAudioQualityMetrics,
    noise_class: InterruptNoiseClass,
    capture_degraded: bool,
    aec_unstable: bool,
    device_changed: bool,
    stream_gap_detected: bool,
) -> DegradationClassBundle {
    let capture_quality_class =
        if capture_degraded || quality.snr_db < 8.0 || quality.clipping_ratio >= 0.15 {
            CaptureQualityClass::Critical
        } else if quality.snr_db < 14.0 || quality.clipping_ratio >= 0.08 {
            CaptureQualityClass::Degraded
        } else if quality.snr_db < 22.0 || quality.clipping_ratio >= 0.04 {
            CaptureQualityClass::Guarded
        } else {
            CaptureQualityClass::Clear
        };
    let echo_risk_class = if aec_unstable || quality.echo_delay_ms >= 200.0 || quality.erle_db < 6.0
    {
        EchoRiskClass::High
    } else if quality.echo_delay_ms >= 80.0
        || quality.erle_db < 12.0
        || quality.double_talk_score >= 0.65
    {
        EchoRiskClass::Elevated
    } else {
        EchoRiskClass::Low
    };
    let network_stability_class = if stream_gap_detected
        || quality.packet_loss_pct >= 8.0
        || matches!(noise_class, InterruptNoiseClass::Severe)
    {
        NetworkStabilityClass::Unstable
    } else if device_changed
        || quality.packet_loss_pct >= 3.0
        || matches!(noise_class, InterruptNoiseClass::Elevated)
    {
        NetworkStabilityClass::Flaky
    } else {
        NetworkStabilityClass::Stable
    };
    let recoverability_class = if matches!(network_stability_class, NetworkStabilityClass::Unstable)
        || matches!(capture_quality_class, CaptureQualityClass::Critical)
        || device_changed
    {
        RecoverabilityClass::FailoverRequired
    } else if matches!(capture_quality_class, CaptureQualityClass::Degraded)
        || matches!(echo_risk_class, EchoRiskClass::High)
    {
        RecoverabilityClass::Slow
    } else if matches!(capture_quality_class, CaptureQualityClass::Guarded)
        || matches!(echo_risk_class, EchoRiskClass::Elevated)
    {
        RecoverabilityClass::Guarded
    } else {
        RecoverabilityClass::Fast
    };
    DegradationClassBundle {
        capture_quality_class,
        echo_risk_class,
        network_stability_class,
        recoverability_class,
    }
}

fn select_adaptive_threshold_profile(
    binding: &InterruptLexiconPolicyBinding,
    input: &AdaptiveThresholdPolicyInput,
    noise_class: InterruptNoiseClass,
) -> Result<AdaptiveThresholdProfile, ContractViolation> {
    binding.validate()?;
    let policy_profile_key = binding.policy_profile_id.as_str().to_ascii_lowercase();
    let governance_penalty: f32 = if policy_profile_key.contains("pae_shadow") {
        0.03
    } else if policy_profile_key.contains("pae_assist") {
        0.015
    } else {
        0.0
    };
    let governance_window_penalty_ms: u32 = if policy_profile_key.contains("pae_shadow") {
        20
    } else if policy_profile_key.contains("pae_assist") {
        10
    } else {
        0
    };
    let route_penalty: f32 = match input.device_route {
        DeviceRoute::Bluetooth => 0.03,
        DeviceRoute::Virtual => 0.02,
        DeviceRoute::Unknown => 0.04,
        DeviceRoute::BuiltIn | DeviceRoute::Usb => 0.0,
    };
    let noise_penalty: f32 = match noise_class {
        InterruptNoiseClass::Clean => 0.0,
        InterruptNoiseClass::Elevated => 0.04,
        InterruptNoiseClass::Severe => 0.10,
    };
    let strict = (route_penalty + noise_penalty + governance_penalty).clamp(0.0, 0.20);
    let voiced_window = match noise_class {
        InterruptNoiseClass::Clean => DEFAULT_MIN_INTERRUPT_VOICED_WINDOW_MS,
        InterruptNoiseClass::Elevated => 110,
        InterruptNoiseClass::Severe => 140,
    }
    .saturating_add(governance_window_penalty_ms);
    let clock_recovery_policy = match noise_class {
        InterruptNoiseClass::Clean => JitterClockRecoveryPolicy::v1(28.0, 120.0, 180),
        InterruptNoiseClass::Elevated => JitterClockRecoveryPolicy::v1(20.0, 90.0, 160),
        InterruptNoiseClass::Severe => JitterClockRecoveryPolicy::v1(14.0, 65.0, 140),
    }
    .expect("built-in PH1.K jitter policy must be valid");

    Ok(AdaptiveThresholdProfile {
        min_phrase_confidence: (DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE + strict).clamp(0.0, 1.0),
        min_vad_confidence: (DEFAULT_MIN_INTERRUPT_VAD_CONFIDENCE + strict).clamp(0.0, 1.0),
        min_acoustic_confidence: (DEFAULT_MIN_INTERRUPT_ACOUSTIC_CONFIDENCE + strict)
            .clamp(0.0, 1.0),
        min_prosody_confidence: (DEFAULT_MIN_INTERRUPT_PROSODY_CONFIDENCE + strict).clamp(0.0, 1.0),
        min_speech_likeness: (DEFAULT_MIN_INTERRUPT_SPEECH_LIKENESS + (strict * 0.8))
            .clamp(0.0, 1.0),
        min_echo_safe_confidence: (DEFAULT_MIN_INTERRUPT_ECHO_SAFE_CONFIDENCE + (strict * 0.5))
            .clamp(0.0, 1.0),
        min_nearfield_confidence: (DEFAULT_MIN_INTERRUPT_NEARFIELD_CONFIDENCE + (strict * 0.6))
            .clamp(0.0, 1.0),
        min_voiced_window_ms: voiced_window,
        min_reliability_score: (DEFAULT_MIN_INTERRUPT_DEVICE_RELIABILITY_SCORE + strict)
            .clamp(0.0, 1.0),
        clock_recovery_policy,
    })
}

pub fn evaluate_interrupt_candidate(
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<InterruptDecisionTrace, ContractViolation> {
    evaluate_interrupt_candidate_for_implementation(PH1K_IMPLEMENTATION_ID, matcher, input)
}

pub fn maybe_interrupt_candidate(
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<Option<InterruptCandidate>, ContractViolation> {
    Ok(evaluate_interrupt_candidate(matcher, input)?.candidate)
}

pub fn maybe_interrupt_candidate_for_implementation(
    implementation_id: &str,
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<Option<InterruptCandidate>, ContractViolation> {
    Ok(
        evaluate_interrupt_candidate_for_implementation(implementation_id, matcher, input)?
            .candidate,
    )
}

pub fn evaluate_interrupt_candidate_for_implementation(
    implementation_id: &str,
    matcher: &InterruptPhraseMatcher,
    input: InterruptInput,
) -> Result<InterruptDecisionTrace, ContractViolation> {
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
) -> Result<InterruptDecisionTrace, ContractViolation> {
    input.adaptive_policy_input.validate()?;
    let noise_class = classify_noise_class(
        &input.adaptive_policy_input.quality_metrics,
        input.capture_degraded,
        input.stream_gap_detected,
        input.aec_unstable,
        input.device_changed,
    );
    let threshold_profile = select_adaptive_threshold_profile(
        &input.lexicon_policy_binding,
        &input.adaptive_policy_input,
        noise_class,
    )?;

    if !input.tts_playback_active {
        return Ok(InterruptDecisionTrace {
            candidate: None,
            reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
            lexical_trigger_accepted: false,
            noise_gate_rejected: true,
            vad_confidence_band: None,
            adaptive_noise_class: Some(noise_class),
        });
    }

    let Some(det) = input.detection else {
        return Ok(InterruptDecisionTrace {
            candidate: None,
            reason_code: reason_codes::K_INTERRUPT_LEXICAL_TRIGGER_REJECTED,
            lexical_trigger_accepted: false,
            noise_gate_rejected: false,
            vad_confidence_band: None,
            adaptive_noise_class: Some(noise_class),
        });
    };
    if det.text.trim().is_empty() {
        return Ok(InterruptDecisionTrace {
            candidate: None,
            reason_code: reason_codes::K_INTERRUPT_LEXICAL_TRIGGER_REJECTED,
            lexical_trigger_accepted: false,
            noise_gate_rejected: false,
            vad_confidence_band: None,
            adaptive_noise_class: Some(noise_class),
        });
    }

    let Some((phrase_set_version, phrase_id)) =
        matcher.match_phrase(&input.lexicon_policy_binding, &det.text)?
    else {
        return Ok(InterruptDecisionTrace {
            candidate: None,
            reason_code: reason_codes::K_INTERRUPT_LEXICAL_TRIGGER_REJECTED,
            lexical_trigger_accepted: false,
            noise_gate_rejected: false,
            vad_confidence_band: None,
            adaptive_noise_class: Some(noise_class),
        });
    };

    let phrase_conf = match Confidence::new(det.confidence) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let phrase_ok = det.confidence >= threshold_profile.min_phrase_confidence;

    let vad_conf = match Confidence::new(input.vad_confidence) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let acoustic_conf = match Confidence::new(input.acoustic_confidence) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let prosody_conf = match Confidence::new(input.prosody_confidence) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let speech_like = match SpeechLikeness::new(input.speech_likeness) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let echo_safe_conf = match Confidence::new(input.echo_safe_confidence) {
        Ok(v) => v,
        Err(_) => {
            return Ok(InterruptDecisionTrace {
                candidate: None,
                reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                lexical_trigger_accepted: true,
                noise_gate_rejected: true,
                vad_confidence_band: None,
                adaptive_noise_class: Some(noise_class),
            });
        }
    };
    let nearfield_conf = match input.nearfield_confidence {
        Some(v) => match Confidence::new(v) {
            Ok(conf) => Some(conf),
            Err(_) => {
                return Ok(InterruptDecisionTrace {
                    candidate: None,
                    reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
                    lexical_trigger_accepted: true,
                    noise_gate_rejected: true,
                    vad_confidence_band: None,
                    adaptive_noise_class: Some(noise_class),
                });
            }
        },
        None => None,
    };
    let vad_confidence_band = classify_vad_decision_confidence_band(vad_conf, speech_like);

    let degradation_ok = !input.capture_degraded
        && !input.stream_gap_detected
        && !input.aec_unstable
        && !input.device_changed;
    let clock_recovery_ok = input.adaptive_policy_input.timing_stats.jitter_ms
        <= threshold_profile.clock_recovery_policy.max_jitter_ms
        && input.adaptive_policy_input.timing_stats.drift_ppm.abs()
            <= threshold_profile.clock_recovery_policy.max_abs_drift_ppm
        && input.adaptive_policy_input.capture_to_handoff_latency_ms
            <= threshold_profile
                .clock_recovery_policy
                .max_handoff_latency_ms;
    let reliability_ok = input
        .adaptive_policy_input
        .device_reliability
        .reliability_score
        .0
        >= threshold_profile.min_reliability_score;
    let acoustic_ok = input.acoustic_confidence >= threshold_profile.min_acoustic_confidence
        && input.adaptive_policy_input.quality_metrics.snr_db >= 8.0
        && input.adaptive_policy_input.quality_metrics.packet_loss_pct <= 20.0;
    let prosody_ok = input.prosody_confidence >= threshold_profile.min_prosody_confidence
        && input.speech_likeness >= threshold_profile.min_speech_likeness;
    let gates = InterruptGates {
        vad_ok: input.speech_likeness >= threshold_profile.min_speech_likeness
            && input.vad_confidence >= threshold_profile.min_vad_confidence,
        echo_safe_ok: input.echo_safe_confidence >= threshold_profile.min_echo_safe_confidence,
        phrase_ok,
        nearfield_ok: input
            .nearfield_confidence
            .map(|v| v >= threshold_profile.min_nearfield_confidence)
            .unwrap_or(true),
    };

    if !(degradation_ok
        && reliability_ok
        && clock_recovery_ok
        && acoustic_ok
        && prosody_ok
        && input.voiced_window_ms >= threshold_profile.min_voiced_window_ms
        && gates.vad_ok
        && gates.echo_safe_ok
        && gates.phrase_ok
        && gates.nearfield_ok)
    {
        return Ok(InterruptDecisionTrace {
            candidate: None,
            reason_code: reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED,
            lexical_trigger_accepted: true,
            noise_gate_rejected: true,
            vad_confidence_band: Some(vad_confidence_band),
            adaptive_noise_class: Some(noise_class),
        });
    }

    let gate_confidences = InterruptGateConfidences {
        vad_confidence: vad_conf,
        speech_likeness: speech_like,
        echo_safe_confidence: echo_safe_conf,
        phrase_confidence: phrase_conf,
        nearfield_confidence: nearfield_conf,
    };

    let candidate_confidence_band = classify_candidate_confidence_band(
        det.confidence,
        input.vad_confidence,
        input.acoustic_confidence,
        input.prosody_confidence,
        input.echo_safe_confidence,
        input.nearfield_confidence,
    );
    let degradation_class_bundle = derive_degradation_class_bundle(
        &input.adaptive_policy_input.quality_metrics,
        noise_class,
        input.capture_degraded,
        input.aec_unstable,
        input.device_changed,
        input.stream_gap_detected,
    );
    let degradation_context = InterruptDegradationContext {
        capture_degraded: input.capture_degraded,
        aec_unstable: input.aec_unstable,
        device_changed: input.device_changed,
        stream_gap_detected: input.stream_gap_detected,
        class_bundle: degradation_class_bundle,
    };
    let risk_context_class = classify_risk_context_class(
        candidate_confidence_band,
        &degradation_context,
        input.nearfield_confidence,
    );
    let timing_markers = InterruptTimingMarkers {
        window_start: MonotonicTimeNs(
            input
                .t_event
                .0
                .saturating_sub(u64::from(input.voiced_window_ms).saturating_mul(1_000_000)),
        ),
        window_end: input.t_event,
    };
    let speech_window_metrics = InterruptSpeechWindowMetrics {
        voiced_window_ms: input.voiced_window_ms,
    };
    let subject_relation_confidence_bundle = InterruptSubjectRelationConfidenceBundle {
        lexical_confidence: phrase_conf,
        vad_confidence: vad_conf,
        speech_likeness: speech_like,
        echo_safe_confidence: echo_safe_conf,
        nearfield_confidence: nearfield_conf,
        combined_confidence: combined_subject_relation_confidence(
            det.confidence,
            input.vad_confidence,
            acoustic_conf.0,
            prosody_conf.0,
            input.speech_likeness,
            input.echo_safe_confidence,
            input.nearfield_confidence,
        )?,
    };
    let reason_code = reason_code_for_candidate_band(candidate_confidence_band);

    let candidate = InterruptCandidate::v1(
        phrase_set_version,
        phrase_id,
        phrase_id,
        input.lexicon_policy_binding.locale_tag.clone(),
        det.text,
        phrase_conf,
        candidate_confidence_band,
        risk_context_class,
        degradation_context,
        timing_markers,
        speech_window_metrics,
        subject_relation_confidence_bundle,
        gates,
        gate_confidences,
        input.t_event,
        reason_code,
    )?;
    Ok(InterruptDecisionTrace {
        candidate: Some(candidate),
        reason_code,
        lexical_trigger_accepted: true,
        noise_gate_rejected: false,
        vad_confidence_band: Some(vad_confidence_band),
        adaptive_noise_class: Some(noise_class),
    })
}

pub fn build_ph1k_to_ph1c_handoff(
    input: &InterruptInput,
    decision_trace: &InterruptDecisionTrace,
) -> Result<Ph1kToPh1cHandoff, ContractViolation> {
    let phrase_conf = input
        .detection
        .as_ref()
        .map(|d| normalize_unit_interval(d.confidence))
        .unwrap_or(0.0);
    let vad_conf = normalize_unit_interval(input.vad_confidence);
    let acoustic_conf = normalize_unit_interval(input.acoustic_confidence);
    let prosody_conf = normalize_unit_interval(input.prosody_confidence);
    let speech_likeness = normalize_unit_interval(input.speech_likeness);
    let echo_safe_conf = normalize_unit_interval(input.echo_safe_confidence);
    let nearfield_conf = input.nearfield_confidence.map(normalize_unit_interval);

    let interrupt_confidence_band = decision_trace
        .candidate
        .as_ref()
        .map(|candidate| candidate.candidate_confidence_band)
        .unwrap_or_else(|| {
            classify_candidate_confidence_band(
                phrase_conf,
                vad_conf,
                acoustic_conf,
                prosody_conf,
                echo_safe_conf,
                nearfield_conf,
            )
        });

    let vad_confidence_band = decision_trace.vad_confidence_band.unwrap_or_else(|| {
        let vad_confidence =
            Confidence::new(vad_conf).expect("normalized vad confidence must remain bounded");
        let speech_likeness = SpeechLikeness::new(speech_likeness)
            .expect("normalized speech likeness must remain bounded");
        classify_vad_decision_confidence_band(vad_confidence, speech_likeness)
    });

    let degradation_class_bundle = decision_trace
        .candidate
        .as_ref()
        .map(|candidate| candidate.degradation_context.class_bundle)
        .unwrap_or_else(|| {
            let noise_class = decision_trace.adaptive_noise_class.unwrap_or_else(|| {
                classify_noise_class(
                    &input.adaptive_policy_input.quality_metrics,
                    input.capture_degraded,
                    input.stream_gap_detected,
                    input.aec_unstable,
                    input.device_changed,
                )
            });
            derive_degradation_class_bundle(
                &input.adaptive_policy_input.quality_metrics,
                noise_class,
                input.capture_degraded,
                input.aec_unstable,
                input.device_changed,
                input.stream_gap_detected,
            )
        });

    Ph1kToPh1cHandoff::v1(
        interrupt_confidence_band,
        vad_confidence_band,
        input.adaptive_policy_input.quality_metrics,
        degradation_class_bundle,
    )
}

pub fn build_ph1k_to_ph1x_handoff(
    decision_trace: &InterruptDecisionTrace,
) -> Result<Option<Ph1kToPh1xInterruptHandoff>, ContractViolation> {
    decision_trace
        .candidate
        .as_ref()
        .map(Ph1kToPh1xInterruptHandoff::from_interrupt_candidate)
        .transpose()
}

#[allow(clippy::too_many_arguments)]
pub fn build_duplex_frame(
    frame_id: u64,
    stream_id: selene_kernel_contracts::ph1k::AudioStreamId,
    pre_roll_buffer_id: selene_kernel_contracts::ph1k::PreRollBufferId,
    t_frame_start: MonotonicTimeNs,
    t_frame_end: MonotonicTimeNs,
    t_capture: MonotonicTimeNs,
    tts_playback_active: bool,
    capture_to_handoff_latency_ms: u32,
) -> Result<DuplexFrame, ContractViolation> {
    DuplexFrame::v1(
        DuplexFrameId(frame_id),
        stream_id,
        pre_roll_buffer_id,
        t_frame_start,
        t_frame_end,
        t_capture,
        tts_playback_active,
        capture_to_handoff_latency_ms,
    )
}

fn normalize_unit_interval(value: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

fn combined_subject_relation_confidence(
    phrase_conf: f32,
    vad_conf: f32,
    acoustic_conf: f32,
    prosody_conf: f32,
    speech_likeness: f32,
    echo_safe_conf: f32,
    nearfield_conf: Option<f32>,
) -> Result<Confidence, ContractViolation> {
    let nearfield = nearfield_conf.unwrap_or(1.0);
    let weighted = (phrase_conf * 0.28)
        + (vad_conf * 0.16)
        + (acoustic_conf * 0.14)
        + (prosody_conf * 0.12)
        + (speech_likeness * 0.15)
        + (echo_safe_conf * 0.10)
        + (nearfield * 0.05);
    Confidence::new(weighted.clamp(0.0, 1.0))
}

fn classify_candidate_confidence_band(
    phrase_conf: f32,
    vad_conf: f32,
    acoustic_conf: f32,
    prosody_conf: f32,
    echo_safe_conf: f32,
    nearfield_conf: Option<f32>,
) -> InterruptCandidateConfidenceBand {
    let nearfield_high = nearfield_conf.map(|v| v >= 0.80).unwrap_or(true);
    let nearfield_mid = nearfield_conf.map(|v| v >= 0.70).unwrap_or(true);
    if phrase_conf >= 0.95
        && vad_conf >= 0.90
        && acoustic_conf >= 0.92
        && prosody_conf >= 0.90
        && echo_safe_conf >= 0.95
        && nearfield_high
    {
        InterruptCandidateConfidenceBand::High
    } else if phrase_conf >= 0.90
        && vad_conf >= 0.85
        && acoustic_conf >= 0.85
        && prosody_conf >= 0.82
        && echo_safe_conf >= 0.92
        && nearfield_mid
    {
        InterruptCandidateConfidenceBand::Medium
    } else {
        InterruptCandidateConfidenceBand::Low
    }
}

fn reason_code_for_candidate_band(
    candidate_confidence_band: InterruptCandidateConfidenceBand,
) -> ReasonCodeId {
    match candidate_confidence_band {
        InterruptCandidateConfidenceBand::High => reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_HIGH,
        InterruptCandidateConfidenceBand::Medium => {
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_MEDIUM
        }
        InterruptCandidateConfidenceBand::Low => reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_LOW,
    }
}

fn classify_risk_context_class(
    confidence_band: InterruptCandidateConfidenceBand,
    degradation_context: &InterruptDegradationContext,
    nearfield_conf: Option<f32>,
) -> InterruptRiskContextClass {
    if degradation_context.capture_degraded
        || degradation_context.aec_unstable
        || degradation_context.device_changed
        || degradation_context.stream_gap_detected
    {
        return InterruptRiskContextClass::High;
    }
    if matches!(confidence_band, InterruptCandidateConfidenceBand::Low)
        || nearfield_conf.map(|v| v < 0.70).unwrap_or(false)
    {
        return InterruptRiskContextClass::Guarded;
    }
    InterruptRiskContextClass::Low
}

fn built_in_interrupt_phrases_by_locale() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        (
            "en-US",
            vec![
                "wait",
                "selene wait",
                "hold on",
                "stop",
                "pause",
                "cancel that",
                "just a second",
                "hey selene",
                "excuse me",
            ],
        ),
        (
            "es-ES",
            vec![
                "espera",
                "selene espera",
                "alto",
                "pausa",
                "cancela eso",
                "un momento",
            ],
        ),
        (
            "zh-CN",
            vec!["等一下", "停一下", "暂停", "先别说", "selene 等一下"],
        ),
        ("tr-TR", vec!["bekle", "dur", "selene bekle", "bir saniye"]),
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
    use crate::ph1c::{Ph1cConfig, Ph1cRuntime, ProviderSlot, SttAttempt};
    use selene_kernel_contracts::ph1c::{LanguageTag, Ph1cRequest, Ph1cResponse, SessionStateRef};
    use selene_kernel_contracts::ph1k::{AudioStreamId, PreRollBufferId};
    use selene_kernel_contracts::ph1w::{BoundedAudioSegmentRef, SessionState};
    use selene_kernel_contracts::ph1x::Ph1kToPh1xInterruptHandoff;

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    fn default_interrupt_binding(
        matcher: &InterruptPhraseMatcher,
    ) -> InterruptLexiconPolicyBinding {
        matcher.default_policy_binding()
    }

    fn default_interrupt_input(
        binding: InterruptLexiconPolicyBinding,
        detection: Option<PhraseDetection>,
        t_event: MonotonicTimeNs,
    ) -> InterruptInput {
        InterruptInput {
            lexicon_policy_binding: binding,
            adaptive_policy_input: default_adaptive_policy_input(DeviceRoute::BuiltIn),
            tts_playback_active: true,
            capture_degraded: false,
            stream_gap_detected: false,
            aec_unstable: false,
            device_changed: false,
            voiced_window_ms: 100,
            vad_confidence: 0.9,
            acoustic_confidence: 0.92,
            prosody_confidence: 0.91,
            speech_likeness: 0.9,
            echo_safe_confidence: 0.95,
            nearfield_confidence: Some(0.9),
            detection,
            t_event,
        }
    }

    fn detect(text: &str, confidence: f32) -> Option<PhraseDetection> {
        Some(PhraseDetection {
            text: text.to_string(),
            confidence,
        })
    }

    fn reliability_profile(reliability_bp: u16) -> DeviceReliabilityProfile {
        DeviceReliabilityProfile {
            failures_24h: 0,
            recoveries_24h: 0,
            mean_recovery_ms: 0,
            reliability_bp,
        }
    }

    fn first_device_error_code(events: &[Ph1kOutputEvent]) -> Option<ReasonCodeId> {
        events.iter().find_map(|event| match event {
            Ph1kOutputEvent::DeviceState(state) => state.errors.first().map(|error| error.code),
            _ => None,
        })
    }

    fn last_degradation_state(events: &[Ph1kOutputEvent]) -> Option<InterruptDegradationContext> {
        events.iter().rev().find_map(|event| match event {
            Ph1kOutputEvent::DegradationState(state) => Some(*state),
            _ => None,
        })
    }

    fn first_selected_mic(events: &[Ph1kOutputEvent]) -> Option<&str> {
        events.iter().find_map(|event| match event {
            Ph1kOutputEvent::DeviceState(state) => Some(state.selected_mic.as_str()),
            _ => None,
        })
    }

    fn segment(duration_ms: u64) -> BoundedAudioSegmentRef {
        BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(0),
            MonotonicTimeNs(duration_ms * 1_000_000),
            MonotonicTimeNs(0),
            MonotonicTimeNs(0),
        )
        .expect("segment must be valid")
    }

    fn run_runtime_sequence(
        events: &[Ph1kEvent],
        policy: DevicePolicy,
    ) -> Vec<Vec<Ph1kOutputEvent>> {
        let mut rt = Ph1kRuntime::new(policy);
        events
            .iter()
            .cloned()
            .map(|event| rt.handle(event))
            .collect()
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
            reliability_profiles: HashMap::new(),
        };

        let sel = policy.select(&available).unwrap();
        assert_eq!(sel.mic.as_str(), "mic_a");
        assert_eq!(sel.speaker.as_str(), "spk_a");
    }

    #[test]
    fn device_policy_fallback_prefers_highest_reliability_then_lexical_tiebreak() {
        let available = AvailableDevices {
            mics: vec![dev("mic_b"), dev("mic_a")],
            speakers: vec![dev("spk_b"), dev("spk_a")],
            system_default_mic: None,
            system_default_speaker: None,
        };
        let mut reliability_profiles = HashMap::new();
        reliability_profiles.insert(dev("mic_a"), reliability_profile(2_000));
        reliability_profiles.insert(dev("mic_b"), reliability_profile(9_500));
        reliability_profiles.insert(dev("spk_a"), reliability_profile(1_000));
        reliability_profiles.insert(dev("spk_b"), reliability_profile(9_000));

        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles,
        };

        let sel = policy.select(&available).unwrap();
        assert_eq!(sel.mic.as_str(), "mic_b");
        assert_eq!(sel.speaker.as_str(), "spk_b");
    }

    #[test]
    fn runtime_requires_device_stability_window_before_switching() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
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
    fn runtime_failover_uses_reliability_ranked_selection_with_reason_code() {
        let mut reliability_profiles = HashMap::new();
        reliability_profiles.insert(dev("mic_b"), reliability_profile(3_000));
        reliability_profiles.insert(dev("mic_c"), reliability_profile(9_200));
        reliability_profiles.insert(dev("spk_a"), reliability_profile(8_000));
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles,
        };
        let mut rt = Ph1kRuntime::new(policy);

        let _ = rt.handle(Ph1kEvent::Boot {
            available: AvailableDevices {
                mics: vec![dev("mic_a")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(0),
        });

        let _ = rt.handle(Ph1kEvent::DeviceListChanged {
            available: AvailableDevices {
                mics: vec![dev("mic_b"), dev("mic_c")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(100_000_000),
        });
        assert_eq!(rt.selection().unwrap().mic.as_str(), "mic_a");

        let out = rt.handle(Ph1kEvent::DeviceListChanged {
            available: AvailableDevices {
                mics: vec![dev("mic_b"), dev("mic_c")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(450_000_000),
        });

        assert_eq!(rt.selection().unwrap().mic.as_str(), "mic_c");
        assert_eq!(
            first_device_error_code(&out),
            Some(reason_codes::K_FAILOVER_RELIABILITY_SELECTED)
        );
    }

    #[test]
    fn runtime_aec_autotune_applies_then_rolls_back_within_safe_bounds() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
        };
        let mut rt = Ph1kRuntime::new(policy);
        let _ = rt.handle(Ph1kEvent::Boot {
            available: AvailableDevices {
                mics: vec![dev("mic_a")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(0),
        });

        let mut last_code = None;
        for step in 0..12 {
            let out = rt.handle(Ph1kEvent::AecUnstable {
                now: MonotonicTimeNs(10 + step),
            });
            last_code = first_device_error_code(&out);
        }

        let profile = rt.calibration_profile().expect("profile must exist");
        assert_eq!(profile.mic_gain_db, -6.0);
        assert_eq!(profile.speaker_gain_db, -6.0);
        assert_eq!(profile.tune_steps, 12);
        assert_eq!(
            last_code,
            Some(reason_codes::K_CALIBRATION_AUTO_TUNE_APPLIED)
        );

        let rollback_out = rt.handle(Ph1kEvent::AecUnstable {
            now: MonotonicTimeNs(100),
        });
        let profile = rt.calibration_profile().expect("profile must exist");
        assert_eq!(profile.mic_gain_db, 0.0);
        assert_eq!(profile.speaker_gain_db, 0.0);
        assert_eq!(profile.tune_steps, 0);
        assert_eq!(profile.rollback_count, 1);
        assert_eq!(
            first_device_error_code(&rollback_out),
            Some(reason_codes::K_CALIBRATION_AUTO_TUNE_ROLLBACK)
        );
    }

    #[test]
    fn runtime_degradation_state_class_bundle_is_rebuildable_from_flags() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
        };
        let mut rt = Ph1kRuntime::new(policy);

        let _ = rt.handle(Ph1kEvent::Boot {
            available: AvailableDevices {
                mics: vec![dev("mic_a")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(0),
        });

        let unstable_out = rt.handle(Ph1kEvent::AecUnstable {
            now: MonotonicTimeNs(10),
        });
        let unstable_state =
            last_degradation_state(&unstable_out).expect("degradation state must be emitted");
        assert_eq!(
            unstable_state.class_bundle,
            DegradationClassBundle::from_flags(
                unstable_state.capture_degraded,
                unstable_state.aec_unstable,
                unstable_state.device_changed,
                unstable_state.stream_gap_detected,
            )
        );
        assert_eq!(
            unstable_state.class_bundle.echo_risk_class,
            EchoRiskClass::High
        );

        let stream_gap_out = rt.handle(Ph1kEvent::StreamGapDetected {
            now: MonotonicTimeNs(20),
        });
        let stream_gap_state =
            last_degradation_state(&stream_gap_out).expect("degradation state must be emitted");
        assert_eq!(
            stream_gap_state.class_bundle,
            DegradationClassBundle::from_flags(
                stream_gap_state.capture_degraded,
                stream_gap_state.aec_unstable,
                stream_gap_state.device_changed,
                stream_gap_state.stream_gap_detected,
            )
        );
        assert_eq!(
            stream_gap_state.class_bundle.network_stability_class,
            NetworkStabilityClass::Unstable
        );
        assert_eq!(
            stream_gap_state.class_bundle.recoverability_class,
            RecoverabilityClass::FailoverRequired
        );
    }

    #[test]
    fn runtime_degradation_state_returns_to_clear_bundle_after_recovery_window() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
        };
        let mut rt = Ph1kRuntime::new(policy);

        let _ = rt.handle(Ph1kEvent::Boot {
            available: AvailableDevices {
                mics: vec![dev("mic_a")],
                speakers: vec![dev("spk_a")],
                system_default_mic: None,
                system_default_speaker: None,
            },
            now: MonotonicTimeNs(0),
        });

        let _ = rt.handle(Ph1kEvent::AecUnstable {
            now: MonotonicTimeNs(1),
        });
        let _ = rt.handle(Ph1kEvent::AecStable {
            now: MonotonicTimeNs(2),
        });
        let recovered = rt.handle(Ph1kEvent::StreamRecovered {
            now: MonotonicTimeNs(600_000_000),
        });

        let state = last_degradation_state(&recovered).expect("degradation state must be emitted");
        assert!(!state.capture_degraded);
        assert!(!state.aec_unstable);
        assert!(!state.device_changed);
        assert!(!state.stream_gap_detected);
        assert_eq!(
            state.class_bundle,
            DegradationClassBundle::from_flags(false, false, false, false)
        );
        assert_eq!(
            state.class_bundle.capture_quality_class,
            CaptureQualityClass::Clear
        );
        assert_eq!(state.class_bundle.echo_risk_class, EchoRiskClass::Low);
        assert_eq!(
            state.class_bundle.network_stability_class,
            NetworkStabilityClass::Stable
        );
        assert_eq!(
            state.class_bundle.recoverability_class,
            RecoverabilityClass::Fast
        );
    }

    #[test]
    fn ph1k_to_ph1c_handoff_uses_candidate_band_when_interrupt_candidate_exists() {
        let matcher = InterruptPhraseMatcher::built_in();
        let input = default_interrupt_input(
            default_interrupt_binding(&matcher),
            detect("stop", 0.99),
            MonotonicTimeNs(321),
        );
        let decision = evaluate_interrupt_candidate(&matcher, input.clone())
            .expect("interrupt candidate evaluation must succeed");
        let candidate = decision
            .candidate
            .as_ref()
            .expect("candidate should be present");

        let handoff =
            build_ph1k_to_ph1c_handoff(&input, &decision).expect("handoff build must pass");

        assert_eq!(
            handoff.interrupt_confidence_band,
            candidate.candidate_confidence_band
        );
        assert_eq!(
            handoff.vad_confidence_band,
            decision
                .vad_confidence_band
                .expect("vad band should be available on candidate path")
        );
        assert_eq!(
            handoff.quality_metrics,
            input.adaptive_policy_input.quality_metrics
        );
        assert_eq!(
            handoff.degradation_class_bundle,
            candidate.degradation_context.class_bundle
        );
    }

    #[test]
    fn ph1k_to_ph1c_handoff_derives_fallback_bands_without_candidate() {
        let matcher = InterruptPhraseMatcher::built_in();
        let input = InterruptInput {
            tts_playback_active: false,
            detection: None,
            ..default_interrupt_input(
                default_interrupt_binding(&matcher),
                detect("stop", 0.99),
                MonotonicTimeNs(654),
            )
        };
        let decision = evaluate_interrupt_candidate(&matcher, input.clone())
            .expect("interrupt evaluation should still be deterministic");
        assert!(decision.candidate.is_none());

        let handoff =
            build_ph1k_to_ph1c_handoff(&input, &decision).expect("handoff build must pass");

        let expected_bundle = derive_degradation_class_bundle(
            &input.adaptive_policy_input.quality_metrics,
            classify_noise_class(
                &input.adaptive_policy_input.quality_metrics,
                input.capture_degraded,
                input.stream_gap_detected,
                input.aec_unstable,
                input.device_changed,
            ),
            input.capture_degraded,
            input.aec_unstable,
            input.device_changed,
            input.stream_gap_detected,
        );
        let expected_vad_band = classify_vad_decision_confidence_band(
            Confidence::new(input.vad_confidence).unwrap(),
            SpeechLikeness::new(input.speech_likeness).unwrap(),
        );

        assert_eq!(
            handoff.interrupt_confidence_band,
            InterruptCandidateConfidenceBand::Low
        );
        assert_eq!(handoff.vad_confidence_band, expected_vad_band);
        assert_eq!(handoff.degradation_class_bundle, expected_bundle);
    }

    #[test]
    fn ph1k_to_ph1x_handoff_projects_interrupt_risk_context_when_candidate_exists() {
        let matcher = InterruptPhraseMatcher::built_in();
        let input = default_interrupt_input(
            default_interrupt_binding(&matcher),
            detect("stop", 0.99),
            MonotonicTimeNs(777),
        );
        let decision = evaluate_interrupt_candidate(&matcher, input)
            .expect("interrupt candidate evaluation must succeed");
        let candidate = decision
            .candidate
            .as_ref()
            .expect("candidate should be present");

        let handoff = build_ph1k_to_ph1x_handoff(&decision)
            .expect("handoff build must pass")
            .expect("candidate path must emit handoff");

        assert_eq!(
            handoff.candidate_confidence_band,
            candidate.candidate_confidence_band
        );
        assert_eq!(handoff.gate_confidences, candidate.gate_confidences);
        assert_eq!(handoff.degradation_context, candidate.degradation_context);
        assert_eq!(handoff.risk_context_class, candidate.risk_context_class);
    }

    #[test]
    fn ph1k_to_ph1x_handoff_returns_none_without_interrupt_candidate() {
        let matcher = InterruptPhraseMatcher::built_in();
        let input = InterruptInput {
            tts_playback_active: false,
            detection: None,
            ..default_interrupt_input(
                default_interrupt_binding(&matcher),
                detect("stop", 0.99),
                MonotonicTimeNs(778),
            )
        };
        let decision = evaluate_interrupt_candidate(&matcher, input)
            .expect("interrupt evaluation should remain deterministic");
        assert!(decision.candidate.is_none());

        let handoff = build_ph1k_to_ph1x_handoff(&decision).expect("handoff build must pass");
        assert!(handoff.is_none());
    }

    #[test]
    fn interrupt_candidate_requires_confidence_and_all_gates() {
        let matcher = InterruptPhraseMatcher::built_in();
        let t_event = MonotonicTimeNs(123);
        let binding = default_interrupt_binding(&matcher);

        let none = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                echo_safe_confidence: 0.89,
                ..default_interrupt_input(binding.clone(), detect("stop", 0.99), t_event)
            },
        )
        .unwrap();
        assert!(none.is_none());

        let some = maybe_interrupt_candidate(
            &matcher,
            default_interrupt_input(binding, detect("stop", 0.99), t_event),
        )
        .unwrap();
        let candidate = some.expect("candidate must be present");
        assert_eq!(candidate.trigger_phrase_id.0, candidate.phrase_id.0);
        assert_eq!(
            candidate.trigger_locale.as_str(),
            PH1K_INTERRUPT_LOCALE_TAG_DEFAULT
        );
        assert_eq!(
            candidate.candidate_confidence_band,
            InterruptCandidateConfidenceBand::High
        );
        assert_eq!(candidate.risk_context_class, InterruptRiskContextClass::Low);
        assert!(!candidate.degradation_context.capture_degraded);
        assert!(!candidate.degradation_context.aec_unstable);
        assert!(!candidate.degradation_context.device_changed);
        assert!(!candidate.degradation_context.stream_gap_detected);
        assert_eq!(candidate.timing_markers.window_end, t_event);
        assert_eq!(candidate.speech_window_metrics.voiced_window_ms, 100);
        assert_eq!(
            candidate
                .subject_relation_confidence_bundle
                .lexical_confidence,
            candidate.phrase_confidence
        );
        assert_eq!(
            candidate.subject_relation_confidence_bundle.vad_confidence,
            candidate.gate_confidences.vad_confidence
        );
        assert_eq!(
            candidate.subject_relation_confidence_bundle.speech_likeness,
            candidate.gate_confidences.speech_likeness
        );
        assert_eq!(
            candidate
                .subject_relation_confidence_bundle
                .echo_safe_confidence,
            candidate.gate_confidences.echo_safe_confidence
        );
    }

    #[test]
    fn interrupt_candidate_fails_closed_on_degradation() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: true,
                capture_degraded: true,
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(1))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_impl_01_unknown_implementation_fails_closed() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
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
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate_for_implementation(
            "PH1.K.999",
            &matcher,
            default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(1)),
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
    fn at_k_interrupt_04_unknown_policy_profile_fails_closed() {
        let matcher = InterruptPhraseMatcher::built_in();
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                lexicon_policy_binding: InterruptLexiconPolicyBinding::v1(
                    InterruptPolicyProfileId::new("unknown_policy_profile").unwrap(),
                    InterruptTenantProfileId::new(PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT)
                        .unwrap(),
                    InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
                )
                .unwrap(),
                ..default_interrupt_input(
                    default_interrupt_binding(&matcher),
                    detect("stop", 0.99),
                    MonotonicTimeNs(1),
                )
            },
        );
        assert!(matches!(
            out,
            Err(ContractViolation::InvalidValue {
                field: "interrupt_input.lexicon_policy_binding.policy_profile_id",
                ..
            })
        ));
    }

    #[test]
    fn at_k_interrupt_05_noise_only_without_lexical_detection_does_not_emit_candidate() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                voiced_window_ms: 140,
                vad_confidence: 0.97,
                acoustic_confidence: 0.96,
                prosody_confidence: 0.95,
                speech_likeness: 0.95,
                echo_safe_confidence: 0.98,
                nearfield_confidence: Some(0.95),
                detection: None,
                ..default_interrupt_input(binding, None, MonotonicTimeNs(1))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_interrupt_06_unknown_phrase_with_strong_acoustic_signals_does_not_emit_candidate() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                voiced_window_ms: 120,
                vad_confidence: 0.95,
                acoustic_confidence: 0.96,
                prosody_confidence: 0.95,
                speech_likeness: 0.94,
                echo_safe_confidence: 0.97,
                nearfield_confidence: Some(0.92),
                detection: detect("random noise token", 0.99),
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(1))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_interrupt_07_low_phrase_confidence_does_not_emit_candidate_even_with_strong_audio() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                voiced_window_ms: 120,
                vad_confidence: 0.95,
                acoustic_confidence: 0.96,
                prosody_confidence: 0.95,
                speech_likeness: 0.94,
                echo_safe_confidence: 0.97,
                nearfield_confidence: Some(0.92),
                detection: detect("stop", 0.50),
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(1))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_interrupt_08_low_prosody_gate_blocks_candidate_even_with_lexical_match() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                voiced_window_ms: 120,
                vad_confidence: 0.95,
                acoustic_confidence: 0.96,
                prosody_confidence: 0.40,
                speech_likeness: 0.40,
                echo_safe_confidence: 0.97,
                nearfield_confidence: Some(0.92),
                detection: detect("stop", 0.99),
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(1))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_interrupt_09_decision_trace_reason_codes_are_deterministic() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let t_event = MonotonicTimeNs(42);

        let rejected = evaluate_interrupt_candidate(
            &matcher,
            InterruptInput {
                tts_playback_active: false,
                ..default_interrupt_input(binding.clone(), detect("stop", 0.99), t_event)
            },
        )
        .unwrap();
        assert!(rejected.candidate.is_none());
        assert_eq!(
            rejected.reason_code,
            reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED
        );
        assert!(rejected.noise_gate_rejected);
        assert!(!rejected.lexical_trigger_accepted);

        let emitted = evaluate_interrupt_candidate(
            &matcher,
            InterruptInput {
                vad_confidence: 0.95,
                acoustic_confidence: 0.96,
                prosody_confidence: 0.95,
                speech_likeness: 0.95,
                echo_safe_confidence: 0.97,
                ..default_interrupt_input(binding, detect("stop", 0.99), t_event)
            },
        )
        .unwrap();
        assert!(emitted.candidate.is_some());
        assert_eq!(
            emitted.reason_code,
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_HIGH
        );
        assert!(!emitted.noise_gate_rejected);
        assert!(emitted.lexical_trigger_accepted);
    }

    #[test]
    fn at_k_interrupt_10_feedback_signal_mapping_is_reason_coded() {
        let signal = build_interrupt_feedback_signal(
            InterruptFeedbackSignalKind::FalseLexicalTrigger,
            Some(InterruptCandidateConfidenceBand::Low),
        );
        assert_eq!(
            signal.reason_code,
            reason_codes::K_INTERRUPT_FEEDBACK_FALSE_LEXICAL_TRIGGER
        );
        assert_eq!(
            signal.route_targets,
            [
                FeedbackSignalTarget::LearnPackage,
                FeedbackSignalTarget::PaeScorecard
            ]
        );
        assert_eq!(signal.confidence_bucket, FeedbackConfidenceBucket::Low);
    }

    #[test]
    fn at_k_interrupt_11_multilingual_unicode_phrase_matches_by_locale() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new(PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT).unwrap(),
            InterruptTenantProfileId::new(PH1K_INTERRUPT_TENANT_PROFILE_ID_DEFAULT).unwrap(),
            InterruptLocaleTag::new("zh-CN").unwrap(),
        )
        .unwrap();
        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                detection: detect("等一下", 0.98),
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(9))
            },
        )
        .unwrap();
        assert!(out.is_some());
    }

    #[test]
    fn at_k_interrupt_12_clock_recovery_budget_failure_blocks_candidate() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);
        let mut adaptive = default_adaptive_policy_input(DeviceRoute::Bluetooth);
        adaptive.timing_stats = TimingStats::v1(45.0, 220.0, 18.0, 0, 0);
        adaptive.capture_to_handoff_latency_ms = 250;

        let out = maybe_interrupt_candidate(
            &matcher,
            InterruptInput {
                adaptive_policy_input: adaptive,
                voiced_window_ms: 160,
                detection: detect("stop", 0.99),
                ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(7))
            },
        )
        .unwrap();
        assert!(out.is_none());
    }

    #[test]
    fn at_k_interrupt_13_confidence_band_and_reason_code_mapping_boundaries_are_locked() {
        let high = classify_candidate_confidence_band(0.95, 0.90, 0.92, 0.90, 0.95, Some(0.80));
        assert_eq!(high, InterruptCandidateConfidenceBand::High);
        assert_eq!(
            reason_code_for_candidate_band(high),
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_HIGH
        );

        let medium = classify_candidate_confidence_band(0.90, 0.85, 0.85, 0.82, 0.92, Some(0.70));
        assert_eq!(medium, InterruptCandidateConfidenceBand::Medium);
        assert_eq!(
            reason_code_for_candidate_band(medium),
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_MEDIUM
        );

        let low = classify_candidate_confidence_band(0.89, 0.85, 0.85, 0.82, 0.92, Some(0.70));
        assert_eq!(low, InterruptCandidateConfidenceBand::Low);
        assert_eq!(
            reason_code_for_candidate_band(low),
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_LOW
        );

        let high_without_nearfield =
            classify_candidate_confidence_band(0.95, 0.90, 0.92, 0.90, 0.95, None);
        assert_eq!(
            high_without_nearfield,
            InterruptCandidateConfidenceBand::High
        );
    }

    #[test]
    fn at_k_interrupt_14_threshold_profile_selection_is_deterministic_by_route_and_noise() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);

        let clean_input = default_adaptive_policy_input(DeviceRoute::BuiltIn);
        let clean_a =
            select_adaptive_threshold_profile(&binding, &clean_input, InterruptNoiseClass::Clean)
                .expect("clean profile selection must pass");
        let clean_b =
            select_adaptive_threshold_profile(&binding, &clean_input, InterruptNoiseClass::Clean)
                .expect("clean profile selection replay must pass");
        assert_eq!(clean_a, clean_b);

        let severe_input = default_adaptive_policy_input(DeviceRoute::Bluetooth);
        let severe =
            select_adaptive_threshold_profile(&binding, &severe_input, InterruptNoiseClass::Severe)
                .expect("severe profile selection must pass");
        assert!(severe.min_phrase_confidence > clean_a.min_phrase_confidence);
        assert!(severe.min_vad_confidence > clean_a.min_vad_confidence);
        assert!(severe.min_voiced_window_ms > clean_a.min_voiced_window_ms);
        assert!(
            severe.clock_recovery_policy.max_handoff_latency_ms
                < clean_a.clock_recovery_policy.max_handoff_latency_ms
        );
    }

    #[test]
    fn at_k_interrupt_15_threshold_profile_selection_accepts_valid_dynamic_profile_ids() {
        let bad_binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new(PH1K_INTERRUPT_POLICY_PROFILE_ID_DEFAULT).unwrap(),
            InterruptTenantProfileId::new("tenant_interrupt_unknown").unwrap(),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
        )
        .unwrap();
        let profile = select_adaptive_threshold_profile(
            &bad_binding,
            &default_adaptive_policy_input(DeviceRoute::Usb),
            InterruptNoiseClass::Clean,
        )
        .expect("valid dynamic tenant profile ids must be accepted");
        assert!(profile.min_phrase_confidence >= DEFAULT_MIN_INTERRUPT_PHRASE_CONFIDENCE);
    }

    #[test]
    fn at_k_interrupt_15a_threshold_profile_is_governed_by_pae_profile_mode() {
        let shadow_binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new("interrupt_policy_pae_shadow_abcd").unwrap(),
            InterruptTenantProfileId::new("tenant_interrupt_pae_shadow_abcd").unwrap(),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
        )
        .unwrap();
        let assist_binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new("interrupt_policy_pae_assist_abcd").unwrap(),
            InterruptTenantProfileId::new("tenant_interrupt_pae_assist_abcd").unwrap(),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
        )
        .unwrap();
        let lead_binding = InterruptLexiconPolicyBinding::v1(
            InterruptPolicyProfileId::new("interrupt_policy_pae_lead_abcd").unwrap(),
            InterruptTenantProfileId::new("tenant_interrupt_pae_lead_abcd").unwrap(),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
        )
        .unwrap();
        let input = default_adaptive_policy_input(DeviceRoute::BuiltIn);

        let shadow =
            select_adaptive_threshold_profile(&shadow_binding, &input, InterruptNoiseClass::Clean)
                .expect("shadow profile selection must pass");
        let assist =
            select_adaptive_threshold_profile(&assist_binding, &input, InterruptNoiseClass::Clean)
                .expect("assist profile selection must pass");
        let lead =
            select_adaptive_threshold_profile(&lead_binding, &input, InterruptNoiseClass::Clean)
                .expect("lead profile selection must pass");

        assert!(shadow.min_phrase_confidence > assist.min_phrase_confidence);
        assert!(assist.min_phrase_confidence > lead.min_phrase_confidence);
        assert!(shadow.min_voiced_window_ms > assist.min_voiced_window_ms);
        assert!(assist.min_voiced_window_ms > lead.min_voiced_window_ms);
    }

    #[test]
    fn at_k_runtime_16_noisy_environment_recovery_replay_is_deterministic() {
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles: HashMap::new(),
        };
        let events = vec![
            Ph1kEvent::Boot {
                available: AvailableDevices {
                    mics: vec![dev("mic_a")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(0),
            },
            Ph1kEvent::StartFullDuplex {
                now: MonotonicTimeNs(5),
            },
            Ph1kEvent::AecUnstable {
                now: MonotonicTimeNs(10),
            },
            Ph1kEvent::StreamGapDetected {
                now: MonotonicTimeNs(20),
            },
            Ph1kEvent::AecStable {
                now: MonotonicTimeNs(30),
            },
            Ph1kEvent::StreamRecovered {
                now: MonotonicTimeNs(100_000_000),
            },
            Ph1kEvent::StreamRecovered {
                now: MonotonicTimeNs(700_000_000),
            },
        ];

        let out_a = run_runtime_sequence(&events, policy.clone());
        let out_b = run_runtime_sequence(&events, policy);
        assert_eq!(out_a, out_b);

        let early_recovery = &out_a[5];
        assert!(!early_recovery.iter().any(|e| matches!(
            e,
            Ph1kOutputEvent::StateChanged(s) if s.to_state == Ph1kState::FullDuplexActive
        )));

        let settled_recovery = &out_a[6];
        assert!(settled_recovery.iter().any(|e| matches!(
            e,
            Ph1kOutputEvent::StateChanged(s) if s.to_state == Ph1kState::FullDuplexActive
        )));

        let final_state =
            last_degradation_state(settled_recovery).expect("degradation state must be emitted");
        assert_eq!(
            final_state.class_bundle,
            DegradationClassBundle::from_flags(false, false, false, false)
        );
    }

    #[test]
    fn at_k_runtime_17_overlap_speech_interrupt_decision_is_replay_deterministic() {
        let matcher = InterruptPhraseMatcher::built_in();
        let binding = default_interrupt_binding(&matcher);

        let mut adaptive = default_adaptive_policy_input(DeviceRoute::Bluetooth);
        adaptive.quality_metrics =
            AdvancedAudioQualityMetrics::v1(20.0, 0.04, 85.0, 2.0, 0.78, 14.0)
                .expect("overlap quality metrics must be valid");

        let uncertain_input = InterruptInput {
            adaptive_policy_input: adaptive,
            voiced_window_ms: 120,
            vad_confidence: 0.85,
            acoustic_confidence: 0.85,
            prosody_confidence: 0.82,
            speech_likeness: 0.80,
            echo_safe_confidence: 0.93,
            nearfield_confidence: Some(0.70),
            detection: detect("stop", 0.90),
            ..default_interrupt_input(binding.clone(), detect("stop", 0.99), MonotonicTimeNs(16))
        };
        let uncertain_a = evaluate_interrupt_candidate(&matcher, uncertain_input.clone())
            .expect("overlap decision should evaluate");
        let uncertain_b = evaluate_interrupt_candidate(&matcher, uncertain_input)
            .expect("overlap replay decision should evaluate");
        assert_eq!(uncertain_a, uncertain_b);
        assert_eq!(
            uncertain_a.adaptive_noise_class,
            Some(InterruptNoiseClass::Elevated)
        );
        assert!(uncertain_a.candidate.is_none());
        assert_eq!(
            uncertain_a.reason_code,
            reason_codes::K_INTERRUPT_NOISE_GATE_REJECTED
        );

        let mut strong_adaptive = default_adaptive_policy_input(DeviceRoute::Bluetooth);
        strong_adaptive.quality_metrics =
            AdvancedAudioQualityMetrics::v1(22.0, 0.04, 70.0, 1.8, 0.72, 15.0)
                .expect("strong overlap quality metrics must be valid");

        let strong_input = InterruptInput {
            adaptive_policy_input: strong_adaptive,
            voiced_window_ms: 150,
            vad_confidence: 0.94,
            acoustic_confidence: 0.93,
            prosody_confidence: 0.92,
            speech_likeness: 0.91,
            echo_safe_confidence: 0.96,
            nearfield_confidence: Some(0.88),
            detection: detect("stop", 0.98),
            ..default_interrupt_input(binding, detect("stop", 0.99), MonotonicTimeNs(17))
        };
        let strong = evaluate_interrupt_candidate(&matcher, strong_input)
            .expect("strong overlap decision should evaluate");
        assert_eq!(
            strong.adaptive_noise_class,
            Some(InterruptNoiseClass::Elevated)
        );
        assert!(strong.candidate.is_some());
        assert_eq!(
            strong.reason_code,
            reason_codes::K_INTERRUPT_CANDIDATE_EMITTED_HIGH
        );
    }

    #[test]
    fn at_k_runtime_18_failover_cooldown_stability_windows_are_deterministic() {
        let mut reliability_profiles = HashMap::new();
        reliability_profiles.insert(dev("mic_b"), reliability_profile(4_000));
        reliability_profiles.insert(dev("mic_c"), reliability_profile(9_200));
        reliability_profiles.insert(dev("mic_d"), reliability_profile(9_600));
        reliability_profiles.insert(dev("spk_a"), reliability_profile(8_000));
        let policy = DevicePolicy {
            preference: DevicePreference::default(),
            reliability_profiles,
        };

        let events = vec![
            Ph1kEvent::Boot {
                available: AvailableDevices {
                    mics: vec![dev("mic_a")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(0),
            },
            Ph1kEvent::DeviceListChanged {
                available: AvailableDevices {
                    mics: vec![dev("mic_b"), dev("mic_c")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(100_000_000),
            },
            Ph1kEvent::DeviceListChanged {
                available: AvailableDevices {
                    mics: vec![dev("mic_b"), dev("mic_c")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(450_000_000),
            },
            Ph1kEvent::DeviceListChanged {
                available: AvailableDevices {
                    mics: vec![dev("mic_d")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(600_000_000),
            },
            Ph1kEvent::DeviceListChanged {
                available: AvailableDevices {
                    mics: vec![dev("mic_d")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(1_000_000_000),
            },
            Ph1kEvent::DeviceListChanged {
                available: AvailableDevices {
                    mics: vec![dev("mic_d")],
                    speakers: vec![dev("spk_a")],
                    system_default_mic: None,
                    system_default_speaker: None,
                },
                now: MonotonicTimeNs(2_700_000_000),
            },
        ];

        let out_a = run_runtime_sequence(&events, policy.clone());
        let out_b = run_runtime_sequence(&events, policy);
        assert_eq!(out_a, out_b);

        assert_eq!(first_selected_mic(&out_a[2]), Some("mic_c"));
        assert_eq!(
            first_device_error_code(&out_a[2]),
            Some(reason_codes::K_FAILOVER_RELIABILITY_SELECTED)
        );
        assert_eq!(first_selected_mic(&out_a[4]), None);
        assert_eq!(first_device_error_code(&out_a[4]), None);
        assert_eq!(first_selected_mic(&out_a[5]), Some("mic_d"));
        assert_eq!(
            first_device_error_code(&out_a[5]),
            Some(reason_codes::K_FAILOVER_RELIABILITY_SELECTED)
        );
    }

    #[test]
    fn at_k_runtime_19_ph1c_and_ph1x_handoff_envelopes_are_compatible() {
        let matcher = InterruptPhraseMatcher::built_in();
        let input = default_interrupt_input(
            default_interrupt_binding(&matcher),
            detect("stop", 0.99),
            MonotonicTimeNs(901),
        );
        let decision = evaluate_interrupt_candidate(&matcher, input.clone())
            .expect("interrupt evaluation must pass");
        let candidate = decision
            .candidate
            .as_ref()
            .expect("candidate must exist for compatibility test");

        let ph1c_handoff =
            build_ph1k_to_ph1c_handoff(&input, &decision).expect("ph1c handoff must build");
        let ph1x_handoff = build_ph1k_to_ph1x_handoff(&decision)
            .expect("ph1x handoff build must pass")
            .expect("candidate path must emit ph1x handoff");
        let from_candidate = Ph1kToPh1xInterruptHandoff::from_interrupt_candidate(candidate)
            .expect("ph1x handoff from candidate must validate");
        assert_eq!(ph1x_handoff, from_candidate);

        let req = Ph1cRequest::v1(
            segment(800),
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic_a"), dev("spk_a"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
            Some(ph1c_handoff),
        )
        .expect("ph1c request must accept ph1k handoff");
        let rt = Ph1cRuntime::new(Ph1cConfig::mvp_desktop_v1());
        let attempts = vec![SttAttempt {
            provider: ProviderSlot::Primary,
            latency_ms: 90,
            transcript_text: "cross engine handoff compatibility".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            avg_word_confidence: 0.97,
            low_confidence_ratio: 0.01,
            stable: true,
        }];
        let out = rt.run(&req, &attempts);
        assert!(matches!(out, Ph1cResponse::TranscriptOk(_)));
    }

    #[test]
    fn interrupt_profile_registration_supports_dynamic_tenant_locale_phrases() {
        let mut matcher = InterruptPhraseMatcher::built_in();
        let policy_profile_id = InterruptPolicyProfileId::new("interrupt_policy_tenant_a").unwrap();
        let tenant_profile_id = InterruptTenantProfileId::new("tenant_interrupt_tenant_a").unwrap();
        matcher
            .register_profile_from_phrases(
                policy_profile_id.clone(),
                tenant_profile_id.clone(),
                InterruptPhraseSetVersion(2),
                vec![(
                    InterruptLocaleTag::new("en-US").unwrap(),
                    vec!["please wait now".to_string(), "Selene hold".to_string()],
                )],
            )
            .expect("dynamic profile registration must pass");
        assert!(matcher.has_profile(&policy_profile_id));
        let binding = InterruptLexiconPolicyBinding::v1(
            policy_profile_id,
            tenant_profile_id,
            InterruptLocaleTag::new("en-US").unwrap(),
        )
        .unwrap();
        let hit = matcher
            .match_phrase(&binding, "selene HOLD")
            .expect("dynamic phrase match should not fail");
        assert!(hit.is_some());
    }

    #[test]
    fn duplex_frame_builder_produces_valid_contract_frame() {
        let frame = build_duplex_frame(
            1,
            AudioStreamId(11),
            PreRollBufferId(7),
            MonotonicTimeNs(100),
            MonotonicTimeNs(200),
            MonotonicTimeNs(150),
            true,
            90,
        )
        .expect("duplex frame builder must produce valid frame");
        assert!(frame.tts_playback_active);
        assert_eq!(frame.capture_to_handoff_latency_ms, 90);
    }

    #[test]
    fn at_k_impl_03_active_implementation_list_is_locked() {
        assert_eq!(PH1_K_ENGINE_ID, "PH1.K");
        assert_eq!(PH1_K_ACTIVE_IMPLEMENTATION_IDS, &["PH1.K.001"]);
    }
}
