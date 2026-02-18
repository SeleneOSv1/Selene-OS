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
}
