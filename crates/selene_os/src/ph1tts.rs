#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1k::TtsPlaybackActiveEvent;
use selene_kernel_contracts::ph1tts::{Ph1ttsEvent, Ph1ttsRequest, TtsControl, TtsFailed};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.TTS OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_TTS_PAUSE_RESUME_DISABLED: ReasonCodeId = ReasonCodeId(0x5454_8101);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1TtsWiringConfig {
    pub tts_enabled: bool,
    pub pause_resume_enabled: bool,
    pub max_tick_delta_ms: u32,
}

impl Ph1TtsWiringConfig {
    pub fn mvp_v1(tts_enabled: bool) -> Self {
        Self {
            tts_enabled,
            pause_resume_enabled: true,
            max_tick_delta_ms: 5_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1TtsWiringOutput {
    Event(Ph1ttsEvent),
    PlaybackMarker(TtsPlaybackActiveEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1TtsWiringOutcome {
    NotInvokedDisabled,
    Refused(TtsFailed),
    Forwarded(Vec<Ph1TtsWiringOutput>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34oTtsProofSource {
    DeterministicReplayPack,
    ForegroundNativePlayback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34oTtsTrialKind {
    PositiveSample,
    QuietControl,
    SelfEchoControl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Stage34oTtsProofConfig {
    pub max_playback_window_seconds: u16,
    pub max_positive_samples: u8,
    pub max_control_trials: u8,
    pub min_mos_score_milli: u16,
    pub min_pronunciation_bp: u16,
    pub min_prosody_bp: u16,
    pub min_readability_bp: u16,
}

impl Stage34oTtsProofConfig {
    pub fn controlled_stage34o() -> Self {
        Self {
            max_playback_window_seconds: 20,
            max_positive_samples: 6,
            max_control_trials: 6,
            min_mos_score_milli: 4_000,
            min_pronunciation_bp: 8_500,
            min_prosody_bp: 8_000,
            min_readability_bp: 9_000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage34oTtsTrial {
    pub trial_id: String,
    pub kind: Stage34oTtsTrialKind,
    pub approved_tts_text: Option<String>,
    pub spoken_text: Option<String>,
    pub playback_started: bool,
    pub playback_duration_ms: u16,
    pub foreground_playback_window: bool,
    pub background_playback: bool,
    pub mos_score_milli: u16,
    pub pronunciation_bp: u16,
    pub prosody_bp: u16,
    pub readability_bp: u16,
    pub source_chips_spoken: bool,
    pub citations_spoken: bool,
    pub raw_urls_spoken: bool,
    pub debug_traces_spoken: bool,
    pub provider_json_spoken: bool,
    pub internal_classes_spoken: bool,
    pub stt_transcript_committed: bool,
    pub voice_id_invoked: bool,
    pub provider_call_count: u8,
    pub tool_route_invoked: bool,
    pub protected_execution_requested: bool,
    pub answer_generation_invoked: bool,
    pub billing_attempted: bool,
    pub model_promotion_attempted: bool,
    pub rollback_automation_attempted: bool,
    pub raw_audio_committed: bool,
}

impl Stage34oTtsTrial {
    #[allow(clippy::too_many_arguments)]
    pub fn controlled(
        trial_id: impl Into<String>,
        kind: Stage34oTtsTrialKind,
        approved_tts_text: Option<impl Into<String>>,
        spoken_text: Option<impl Into<String>>,
        playback_started: bool,
        playback_duration_ms: u16,
        mos_score_milli: u16,
        pronunciation_bp: u16,
        prosody_bp: u16,
        readability_bp: u16,
    ) -> Self {
        Self {
            trial_id: trial_id.into(),
            kind,
            approved_tts_text: approved_tts_text.map(Into::into),
            spoken_text: spoken_text.map(Into::into),
            playback_started,
            playback_duration_ms,
            foreground_playback_window: true,
            background_playback: false,
            mos_score_milli,
            pronunciation_bp,
            prosody_bp,
            readability_bp,
            source_chips_spoken: false,
            citations_spoken: false,
            raw_urls_spoken: false,
            debug_traces_spoken: false,
            provider_json_spoken: false,
            internal_classes_spoken: false,
            stt_transcript_committed: false,
            voice_id_invoked: false,
            provider_call_count: 0,
            tool_route_invoked: false,
            protected_execution_requested: false,
            answer_generation_invoked: false,
            billing_attempted: false,
            model_promotion_attempted: false,
            rollback_automation_attempted: false,
            raw_audio_committed: false,
        }
    }

    pub fn with_forbidden_downstream_work(mut self) -> Self {
        self.answer_generation_invoked = true;
        self.voice_id_invoked = true;
        self.provider_call_count = 1;
        self.tool_route_invoked = true;
        self.protected_execution_requested = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage34oTtsProofReport {
    pub source: Stage34oTtsProofSource,
    pub playback_window_seconds: u16,
    pub positive_samples: u8,
    pub control_trials: u8,
    pub max_playback_duration_ms: u16,
    pub min_mos_score_milli: u16,
    pub min_pronunciation_bp: u16,
    pub min_prosody_bp: u16,
    pub min_readability_bp: u16,
    pub provider_call_count: u8,
    pub raw_audio_committed: bool,
    pub background_playback: bool,
    pub clean_tts_text_only: bool,
    pub self_echo_prevented: bool,
    pub downstream_work_absent: bool,
    pub playback_foreground_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage34oTtsProofError {
    PlaybackWindowTooLong,
    MissingPositiveSample,
    MissingControlTrial,
    TooManyPositiveSamples,
    TooManyControlTrials,
    MissingApprovedTtsText,
    MissingSpokenText,
    SpokenTextNotApprovedText,
    DirtyTtsText,
    PositivePlaybackMissing,
    PositivePlaybackOutsideForegroundWindow,
    MosBelowTarget,
    PronunciationBelowTarget,
    ProsodyBelowTarget,
    ReadabilityBelowTarget,
    ControlPlaybackAttempted,
    SelfEchoCommittedTranscript,
    BackgroundPlayback,
    RawAudioCommitted,
    ProviderCallAttempted,
    DownstreamWorkAttempted,
    ProtectedExecutionAttempted,
    BillingPromotionOrRollbackAttempted,
}

pub fn verify_stage34o_controlled_tts_proof(
    config: Stage34oTtsProofConfig,
    source: Stage34oTtsProofSource,
    playback_window_seconds: u16,
    trials: &[Stage34oTtsTrial],
) -> Result<Stage34oTtsProofReport, Stage34oTtsProofError> {
    if playback_window_seconds > config.max_playback_window_seconds {
        return Err(Stage34oTtsProofError::PlaybackWindowTooLong);
    }

    let mut positive_samples = 0u8;
    let mut control_trials = 0u8;
    let mut max_playback_duration_ms = 0u16;
    let mut min_mos_score_milli = u16::MAX;
    let mut min_pronunciation_bp = u16::MAX;
    let mut min_prosody_bp = u16::MAX;
    let mut min_readability_bp = u16::MAX;
    let mut provider_call_count = 0u8;
    let mut self_echo_control_seen = false;

    for trial in trials {
        if trial.background_playback {
            return Err(Stage34oTtsProofError::BackgroundPlayback);
        }
        if trial.raw_audio_committed {
            return Err(Stage34oTtsProofError::RawAudioCommitted);
        }
        if trial.provider_call_count > 0 {
            return Err(Stage34oTtsProofError::ProviderCallAttempted);
        }
        if trial.answer_generation_invoked
            || trial.voice_id_invoked
            || trial.tool_route_invoked
            || trial.stt_transcript_committed
        {
            return Err(Stage34oTtsProofError::DownstreamWorkAttempted);
        }
        if trial.protected_execution_requested {
            return Err(Stage34oTtsProofError::ProtectedExecutionAttempted);
        }
        if trial.billing_attempted
            || trial.model_promotion_attempted
            || trial.rollback_automation_attempted
        {
            return Err(Stage34oTtsProofError::BillingPromotionOrRollbackAttempted);
        }

        provider_call_count = provider_call_count.saturating_add(trial.provider_call_count);

        match trial.kind {
            Stage34oTtsTrialKind::PositiveSample => {
                positive_samples = positive_samples.saturating_add(1);
                if positive_samples > config.max_positive_samples {
                    return Err(Stage34oTtsProofError::TooManyPositiveSamples);
                }
                if !trial.playback_started {
                    return Err(Stage34oTtsProofError::PositivePlaybackMissing);
                }
                if !trial.foreground_playback_window {
                    return Err(Stage34oTtsProofError::PositivePlaybackOutsideForegroundWindow);
                }

                let approved = trial
                    .approved_tts_text
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(Stage34oTtsProofError::MissingApprovedTtsText)?;
                let spoken = trial
                    .spoken_text
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
                    .ok_or(Stage34oTtsProofError::MissingSpokenText)?;
                if !stage34o_tts_text_is_clean(approved)
                    || !stage34o_tts_text_is_clean(spoken)
                    || trial.source_chips_spoken
                    || trial.citations_spoken
                    || trial.raw_urls_spoken
                    || trial.debug_traces_spoken
                    || trial.provider_json_spoken
                    || trial.internal_classes_spoken
                {
                    return Err(Stage34oTtsProofError::DirtyTtsText);
                }
                if approved.trim() != spoken.trim() {
                    return Err(Stage34oTtsProofError::SpokenTextNotApprovedText);
                }
                if trial.mos_score_milli < config.min_mos_score_milli {
                    return Err(Stage34oTtsProofError::MosBelowTarget);
                }
                if trial.pronunciation_bp < config.min_pronunciation_bp {
                    return Err(Stage34oTtsProofError::PronunciationBelowTarget);
                }
                if trial.prosody_bp < config.min_prosody_bp {
                    return Err(Stage34oTtsProofError::ProsodyBelowTarget);
                }
                if trial.readability_bp < config.min_readability_bp {
                    return Err(Stage34oTtsProofError::ReadabilityBelowTarget);
                }

                max_playback_duration_ms = max_playback_duration_ms.max(trial.playback_duration_ms);
                min_mos_score_milli = min_mos_score_milli.min(trial.mos_score_milli);
                min_pronunciation_bp = min_pronunciation_bp.min(trial.pronunciation_bp);
                min_prosody_bp = min_prosody_bp.min(trial.prosody_bp);
                min_readability_bp = min_readability_bp.min(trial.readability_bp);
            }
            Stage34oTtsTrialKind::QuietControl | Stage34oTtsTrialKind::SelfEchoControl => {
                control_trials = control_trials.saturating_add(1);
                if control_trials > config.max_control_trials {
                    return Err(Stage34oTtsProofError::TooManyControlTrials);
                }
                if trial.playback_started {
                    return Err(Stage34oTtsProofError::ControlPlaybackAttempted);
                }
                if trial.stt_transcript_committed {
                    return Err(Stage34oTtsProofError::SelfEchoCommittedTranscript);
                }
                if trial.kind == Stage34oTtsTrialKind::SelfEchoControl {
                    self_echo_control_seen = true;
                }
            }
        }
    }

    if positive_samples == 0 {
        return Err(Stage34oTtsProofError::MissingPositiveSample);
    }
    if control_trials == 0 {
        return Err(Stage34oTtsProofError::MissingControlTrial);
    }

    Ok(Stage34oTtsProofReport {
        source,
        playback_window_seconds,
        positive_samples,
        control_trials,
        max_playback_duration_ms,
        min_mos_score_milli,
        min_pronunciation_bp,
        min_prosody_bp,
        min_readability_bp,
        provider_call_count,
        raw_audio_committed: false,
        background_playback: false,
        clean_tts_text_only: true,
        self_echo_prevented: self_echo_control_seen,
        downstream_work_absent: true,
        playback_foreground_only: true,
    })
}

fn stage34o_tts_text_is_clean(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    let forbidden = [
        "http://",
        "https://",
        "source_chip",
        "source chip:",
        "citation:",
        "raw_url",
        "url:",
        "debug_trace",
        "trace_id",
        "provider_json",
        "provider json",
        "\"provider\"",
        "internal_class",
        "internal class:",
    ];
    !forbidden.iter().any(|token| lower.contains(token))
        && !trimmed.contains('{')
        && !trimmed.contains('}')
}

pub trait Ph1TtsEngine {
    fn handle(&mut self, now: MonotonicTimeNs, req: Ph1ttsRequest) -> Vec<Ph1TtsWiringOutput>;
    fn tick_progress(&mut self, now: MonotonicTimeNs, delta_ms: u32) -> Vec<Ph1TtsWiringOutput>;
}

#[derive(Debug, Clone)]
pub struct Ph1TtsWiring<E>
where
    E: Ph1TtsEngine,
{
    config: Ph1TtsWiringConfig,
    engine: E,
}

impl<E> Ph1TtsWiring<E>
where
    E: Ph1TtsEngine,
{
    pub fn new(config: Ph1TtsWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_tick_delta_ms == 0 || config.max_tick_delta_ms > 60_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tts_wiring_config.max_tick_delta_ms",
                reason: "must be within 1..=60000",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &mut self,
        now: MonotonicTimeNs,
        req: Ph1ttsRequest,
    ) -> Result<Ph1TtsWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.tts_enabled && req.tts_control == TtsControl::Play {
            return Ok(Ph1TtsWiringOutcome::NotInvokedDisabled);
        }

        if !self.config.pause_resume_enabled
            && matches!(req.tts_control, TtsControl::Pause | TtsControl::Resume)
        {
            let failed = TtsFailed {
                schema_version: selene_kernel_contracts::ph1tts::PH1TTS_CONTRACT_VERSION,
                answer_id: req.answer_id,
                reason_code: reason_codes::PH1_TTS_PAUSE_RESUME_DISABLED,
            };
            failed.validate()?;
            return Ok(Ph1TtsWiringOutcome::Refused(failed));
        }

        Ok(Ph1TtsWiringOutcome::Forwarded(self.engine.handle(now, req)))
    }

    pub fn run_tick(
        &mut self,
        now: MonotonicTimeNs,
        delta_ms: u32,
    ) -> Result<Vec<Ph1TtsWiringOutput>, ContractViolation> {
        if delta_ms == 0 || delta_ms > self.config.max_tick_delta_ms {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tts_wiring.tick_delta_ms",
                reason: "must be within 1..=max_tick_delta_ms",
            });
        }
        Ok(self.engine.tick_progress(now, delta_ms))
    }

    #[allow(dead_code)]
    pub fn engine_ref(&self) -> &E {
        &self.engine
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{LanguageTag, SessionStateRef};
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1tts::{
        AnswerId, BargeInPolicyRef, SpokenCursor, StyleModifier, StyleProfileRef, TtsProgress,
        TtsStarted, TtsStopReason, TtsStopped, VoiceId, VoiceRenderPlan,
    };
    use selene_kernel_contracts::ph1w::SessionState;

    #[derive(Debug, Default, Clone)]
    struct FakeEngine {
        handle_calls: usize,
        tick_calls: usize,
        active_answer: Option<AnswerId>,
    }

    impl Ph1TtsEngine for FakeEngine {
        fn handle(&mut self, now: MonotonicTimeNs, req: Ph1ttsRequest) -> Vec<Ph1TtsWiringOutput> {
            self.handle_calls += 1;
            match req.tts_control {
                TtsControl::Play => {
                    self.active_answer = Some(req.answer_id);
                    vec![
                        Ph1TtsWiringOutput::Event(Ph1ttsEvent::Started(TtsStarted {
                            schema_version:
                                selene_kernel_contracts::ph1tts::PH1TTS_CONTRACT_VERSION,
                            answer_id: req.answer_id,
                            voice_id: VoiceId::new("VOICE_DEFAULT").unwrap(),
                            t_started: now,
                        })),
                        Ph1TtsWiringOutput::PlaybackMarker(TtsPlaybackActiveEvent::v1(true, now)),
                    ]
                }
                TtsControl::Cancel => {
                    if self.active_answer == Some(req.answer_id) {
                        self.active_answer = None;
                        vec![
                            Ph1TtsWiringOutput::Event(Ph1ttsEvent::Stopped(TtsStopped {
                                schema_version:
                                    selene_kernel_contracts::ph1tts::PH1TTS_CONTRACT_VERSION,
                                answer_id: req.answer_id,
                                reason: TtsStopReason::Cancelled,
                                t_stopped: now,
                                spoken_cursor: SpokenCursor::v1(0, 0, 1).unwrap(),
                            })),
                            Ph1TtsWiringOutput::PlaybackMarker(TtsPlaybackActiveEvent::v1(
                                false, now,
                            )),
                        ]
                    } else {
                        vec![]
                    }
                }
                TtsControl::Pause | TtsControl::Resume => vec![],
            }
        }

        fn tick_progress(
            &mut self,
            _now: MonotonicTimeNs,
            delta_ms: u32,
        ) -> Vec<Ph1TtsWiringOutput> {
            self.tick_calls += 1;
            let Some(answer_id) = self.active_answer else {
                return vec![];
            };
            vec![Ph1TtsWiringOutput::Event(Ph1ttsEvent::Progress(
                TtsProgress {
                    schema_version: selene_kernel_contracts::ph1tts::PH1TTS_CONTRACT_VERSION,
                    answer_id,
                    ms_played: delta_ms,
                    spoken_cursor: SpokenCursor::v1(0, 0, 1).unwrap(),
                },
            ))]
        }
    }

    fn plan() -> VoiceRenderPlan {
        VoiceRenderPlan::v1(
            StyleProfileRef::Gentle,
            vec![StyleModifier::Warm],
            BargeInPolicyRef::Standard,
            LanguageTag::new("en").unwrap(),
            None,
        )
    }

    fn req(answer_id: AnswerId, control: TtsControl) -> Ph1ttsRequest {
        Ph1ttsRequest::v1(
            answer_id,
            "hello world".to_string(),
            control,
            SessionStateRef::v1(SessionState::Active, true),
            plan(),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
        )
        .unwrap()
    }

    #[test]
    fn at_tts_wiring_01_disabled_play_is_not_invoked() {
        let engine = FakeEngine::default();
        let mut wiring = Ph1TtsWiring::new(Ph1TtsWiringConfig::mvp_v1(false), engine).unwrap();
        let out = wiring
            .run_turn(MonotonicTimeNs(1), req(AnswerId(1), TtsControl::Play))
            .unwrap();
        assert_eq!(out, Ph1TtsWiringOutcome::NotInvokedDisabled);
        assert_eq!(wiring.engine_ref().handle_calls, 0);
    }

    #[test]
    fn at_tts_wiring_02_enabled_play_forwards_to_engine() {
        let engine = FakeEngine::default();
        let mut wiring = Ph1TtsWiring::new(Ph1TtsWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring
            .run_turn(MonotonicTimeNs(1), req(AnswerId(7), TtsControl::Play))
            .unwrap();
        match out {
            Ph1TtsWiringOutcome::Forwarded(events) => {
                assert!(events
                    .iter()
                    .any(|e| matches!(e, Ph1TtsWiringOutput::Event(Ph1ttsEvent::Started(_)))));
                assert!(events.iter().any(|e| matches!(
                    e,
                    Ph1TtsWiringOutput::PlaybackMarker(TtsPlaybackActiveEvent { active: true, .. })
                )));
            }
            _ => panic!("expected forwarded outcome"),
        }
        assert_eq!(wiring.engine_ref().handle_calls, 1);
    }

    #[test]
    fn at_tts_wiring_03_pause_resume_can_be_policy_blocked() {
        let engine = FakeEngine::default();
        let mut cfg = Ph1TtsWiringConfig::mvp_v1(true);
        cfg.pause_resume_enabled = false;
        let mut wiring = Ph1TtsWiring::new(cfg, engine).unwrap();
        let out = wiring
            .run_turn(MonotonicTimeNs(1), req(AnswerId(9), TtsControl::Pause))
            .unwrap();
        match out {
            Ph1TtsWiringOutcome::Refused(f) => {
                assert_eq!(f.reason_code, reason_codes::PH1_TTS_PAUSE_RESUME_DISABLED);
            }
            _ => panic!("expected refused outcome"),
        }
        assert_eq!(wiring.engine_ref().handle_calls, 0);
    }

    #[test]
    fn at_tts_wiring_04_tick_delta_bounds_are_enforced() {
        let engine = FakeEngine::default();
        let mut wiring = Ph1TtsWiring::new(Ph1TtsWiringConfig::mvp_v1(true), engine).unwrap();
        let err = wiring.run_tick(MonotonicTimeNs(1), 9_999).unwrap_err();
        assert!(matches!(
            err,
            ContractViolation::InvalidValue {
                field: "ph1tts_wiring.tick_delta_ms",
                reason: "must be within 1..=max_tick_delta_ms",
            }
        ));
    }

    fn stage34o_positive_sample(id: &str, text: &str) -> Stage34oTtsTrial {
        Stage34oTtsTrial::controlled(
            id,
            Stage34oTtsTrialKind::PositiveSample,
            Some(text),
            Some(text),
            true,
            2_400,
            4_500,
            9_600,
            9_300,
            9_800,
        )
    }

    fn stage34o_quiet_control() -> Stage34oTtsTrial {
        Stage34oTtsTrial::controlled(
            "quiet_1",
            Stage34oTtsTrialKind::QuietControl,
            Option::<String>::None,
            Option::<String>::None,
            false,
            0,
            0,
            0,
            0,
            0,
        )
    }

    fn stage34o_self_echo_control() -> Stage34oTtsTrial {
        Stage34oTtsTrial::controlled(
            "self_echo_1",
            Stage34oTtsTrialKind::SelfEchoControl,
            Option::<String>::None,
            Option::<String>::None,
            false,
            0,
            0,
            0,
            0,
            0,
        )
    }

    #[test]
    fn stage_34o_tts_manifest_records_audio_device_samples_and_stop_rules() {
        let trials = vec![
            stage34o_positive_sample("sample_1", "Selene is ready when you are."),
            stage34o_positive_sample(
                "sample_2",
                "The session is open and waiting for your next request.",
            ),
            stage34o_quiet_control(),
            stage34o_self_echo_control(),
        ];

        let report = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            20,
            &trials,
        )
        .unwrap();

        assert_eq!(
            report.source,
            Stage34oTtsProofSource::DeterministicReplayPack
        );
        assert_eq!(report.playback_window_seconds, 20);
        assert_eq!(report.positive_samples, 2);
        assert_eq!(report.control_trials, 2);
        assert_eq!(report.provider_call_count, 0);
        assert!(report.clean_tts_text_only);
        assert!(report.self_echo_prevented);
        assert!(report.playback_foreground_only);
        assert!(report.downstream_work_absent);
        assert!(!report.raw_audio_committed);
    }

    #[test]
    fn stage_34o_tts_uses_clean_tts_text_without_sources_urls_or_debug_metadata() {
        let mut dirty = stage34o_positive_sample("dirty_1", "Selene is ready when you are.");
        dirty.spoken_text =
            Some("Selene is ready. citation: source_chip http://x.test".to_string());
        dirty.citations_spoken = true;
        dirty.source_chips_spoken = true;
        dirty.raw_urls_spoken = true;

        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[dirty, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::DirtyTtsText);

        let mut mismatch = stage34o_positive_sample("mismatch_1", "Selene is ready when you are.");
        mismatch.spoken_text = Some("The session is open and waiting.".to_string());
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[mismatch, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::SpokenTextNotApprovedText);
    }

    #[test]
    fn stage_34o_tts_naturalness_pronunciation_and_prosody_gates_fail_closed() {
        let mut low_mos = stage34o_positive_sample("low_mos", "Selene is ready when you are.");
        low_mos.mos_score_milli = 3_900;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[low_mos, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::MosBelowTarget);

        let mut low_pron = stage34o_positive_sample("low_pron", "Selene is ready when you are.");
        low_pron.pronunciation_bp = 8_000;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[low_pron, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::PronunciationBelowTarget);

        let mut low_prosody =
            stage34o_positive_sample("low_prosody", "Selene is ready when you are.");
        low_prosody.prosody_bp = 7_500;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[low_prosody, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::ProsodyBelowTarget);

        let mut low_readability =
            stage34o_positive_sample("low_readability", "Selene is ready when you are.");
        low_readability.readability_bp = 8_500;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[low_readability, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::ReadabilityBelowTarget);
    }

    #[test]
    fn stage_34o_tts_self_echo_cannot_commit_listening_transcript() {
        let mut echo = stage34o_self_echo_control();
        echo.stt_transcript_committed = true;

        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::ForegroundNativePlayback,
            10,
            &[
                stage34o_positive_sample("sample_1", "Selene is ready when you are."),
                echo,
            ],
        )
        .unwrap_err();

        assert_eq!(err, Stage34oTtsProofError::DownstreamWorkAttempted);
    }

    #[test]
    fn stage_34o_quiet_control_rejects_playback_without_downstream_work() {
        let mut quiet = stage34o_quiet_control();
        quiet.playback_started = true;

        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[
                stage34o_positive_sample("sample_1", "Selene is ready when you are."),
                quiet,
            ],
        )
        .unwrap_err();

        assert_eq!(err, Stage34oTtsProofError::ControlPlaybackAttempted);
    }

    #[test]
    fn stage_34o_tts_artifacts_are_redacted_and_raw_audio_is_not_committed() {
        let mut raw_audio = stage34o_positive_sample("raw_audio", "Selene is ready when you are.");
        raw_audio.raw_audio_committed = true;

        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::ForegroundNativePlayback,
            10,
            &[raw_audio, stage34o_quiet_control()],
        )
        .unwrap_err();

        assert_eq!(err, Stage34oTtsProofError::RawAudioCommitted);
    }

    #[test]
    fn stage_34o_tts_cannot_route_to_voice_id_provider_tool_or_protected_execution() {
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[
                stage34o_positive_sample("sample_1", "Selene is ready when you are.")
                    .with_forbidden_downstream_work(),
                stage34o_quiet_control(),
            ],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::ProviderCallAttempted);

        let mut downstream =
            stage34o_positive_sample("downstream", "Selene is ready when you are.");
        downstream.answer_generation_invoked = true;
        downstream.voice_id_invoked = true;
        downstream.tool_route_invoked = true;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[downstream, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::DownstreamWorkAttempted);

        let mut protected = stage34o_positive_sample("protected", "Selene is ready when you are.");
        protected.protected_execution_requested = true;
        let err = verify_stage34o_controlled_tts_proof(
            Stage34oTtsProofConfig::controlled_stage34o(),
            Stage34oTtsProofSource::DeterministicReplayPack,
            10,
            &[protected, stage34o_quiet_control()],
        )
        .unwrap_err();
        assert_eq!(err, Stage34oTtsProofError::ProtectedExecutionAttempted);
    }
}
