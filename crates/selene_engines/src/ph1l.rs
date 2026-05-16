#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1l::{
    NextAllowedActions, Ph1lInput, Ph1lOutput, PresenceNudge, SessionId, SessionSnapshot,
    TransitionEvent, TtsPlaybackState,
};
use selene_kernel_contracts::ph1w::WakeDecision;
use selene_kernel_contracts::ph1x::Ph1xDirective;
use selene_kernel_contracts::{
    MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.L reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const L_OPEN_WAKE: ReasonCodeId = ReasonCodeId(0x4C00_0001);
    pub const L_TO_CLOSED_SILENCE: ReasonCodeId = ReasonCodeId(0x4C00_0004);
    pub const L_TO_CLOSED_DISMISS: ReasonCodeId = ReasonCodeId(0x4C00_0005);
    pub const L_SUSPEND_AUDIO_DEGRADED: ReasonCodeId = ReasonCodeId(0x4C00_0009);
    pub const L_RESUME_STABLE: ReasonCodeId = ReasonCodeId(0x4C00_000A);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingQuestion {
    Clarify,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1lConfig {
    pub active_silence_timeout_ms: u32,
    pub clarify_timeout_ms: u32,
    pub confirm_timeout_ms: u32,
}

impl Ph1lConfig {
    pub fn mvp_desktop_v1() -> Self {
        Self {
            active_silence_timeout_ms: 30_000,
            clarify_timeout_ms: 30_000,
            confirm_timeout_ms: 30_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1lRuntime {
    config: Ph1lConfig,
    state: SessionState,
    session_id: Option<SessionId>,
    next_session_id: u128,
    pending_question: Option<PendingQuestion>,
    pending_since: Option<MonotonicTimeNs>,
}

impl Ph1lRuntime {
    pub fn new(config: Ph1lConfig) -> Self {
        Self {
            config,
            state: SessionState::Closed,
            session_id: None,
            next_session_id: 1,
            pending_question: None,
            pending_since: None,
        }
    }

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn session_id(&self) -> Option<SessionId> {
        self.session_id
    }

    pub fn step(&mut self, input: Ph1lInput) -> Ph1lOutput {
        // Fail closed on invalid input.
        if input.validate().is_err() {
            return self.transition(
                input.now,
                SessionState::Closed,
                reason_codes::L_TO_CLOSED_SILENCE,
                false,
            );
        }

        let policy_blocks_audible =
            input.policy_context_ref.privacy_mode || input.policy_context_ref.do_not_disturb;

        // Explicit dismiss ends the session deterministically.
        if input.user_dismissed && self.state != SessionState::Closed {
            return self.transition(
                input.now,
                SessionState::Closed,
                reason_codes::L_TO_CLOSED_DISMISS,
                policy_blocks_audible,
            );
        }

        // Audio integrity failure => SUSPENDED.
        if input.audio_degraded && self.state != SessionState::Suspended {
            return self.transition(
                input.now,
                SessionState::Suspended,
                reason_codes::L_SUSPEND_AUDIO_DEGRADED,
                policy_blocks_audible,
            );
        }

        // Stabilization => leave SUSPENDED safely (require re-wake).
        if self.state == SessionState::Suspended && !input.audio_degraded {
            return self.transition(
                input.now,
                SessionState::Closed,
                reason_codes::L_RESUME_STABLE,
                policy_blocks_audible,
            );
        }

        // Policy: privacy/DND can force closed behavior. Keep minimal in skeleton.
        if policy_blocks_audible {
            // If we're not in a live conversation, close.
            if matches!(self.state, SessionState::SoftClosed | SessionState::Open) {
                return self.transition(
                    input.now,
                    SessionState::Closed,
                    reason_codes::L_TO_CLOSED_SILENCE,
                    policy_blocks_audible,
                );
            }
        }

        // Wake behavior.
        if let Some(w) = &input.wake_event {
            if w.accepted {
                return self.on_wake(input.now, w, policy_blocks_audible);
            }
        }

        // Track "waiting for user" posture based on PH1.X directive.
        self.update_pending_question(input.now, input.conversation_directive.as_ref());

        // Never close while TTS is playing.
        if input.tts_state == TtsPlaybackState::Playing {
            return self.snapshot(None, None, policy_blocks_audible);
        }

        // Waiting posture uses separate timeouts.
        if let Some(p) = self.pending_question {
            let timeout_ms = match p {
                PendingQuestion::Clarify => self.config.clarify_timeout_ms,
                PendingQuestion::Confirm => self.config.confirm_timeout_ms,
            };

            if let Some(since) = self.pending_since {
                if input.now.0.saturating_sub(since.0) >= ms_to_ns(timeout_ms) {
                    self.pending_question = None;
                    self.pending_since = None;
                    return self.transition(
                        input.now,
                        SessionState::Closed,
                        reason_codes::L_TO_CLOSED_SILENCE,
                        policy_blocks_audible,
                    );
                }
            }

            // Stay ACTIVE while waiting (do not close on short silence).
            if self.state == SessionState::Closed {
                // Waiting doesn't make sense closed; keep closed.
                return self.snapshot(None, None, policy_blocks_audible);
            }
            if self.state != SessionState::Active {
                // Normalize to ACTIVE while waiting.
                self.state = SessionState::Active;
            }
            return self.snapshot(None, None, policy_blocks_audible);
        }

        // Do not close or nudge while a Resume Buffer is live.
        if input.resume_buffer_live {
            return self.snapshot(None, None, policy_blocks_audible);
        }

        // Silence-driven close: Stage 6 product policy is one 30-second active window,
        // then a sealed logical session that requires wake for the next session.
        if self.state == SessionState::Active
            && input.user_activity.silence_ms >= self.config.active_silence_timeout_ms
        {
            return self.transition(
                input.now,
                SessionState::Closed,
                reason_codes::L_TO_CLOSED_SILENCE,
                policy_blocks_audible,
            );
        }

        self.snapshot(None, None, policy_blocks_audible)
    }

    fn on_wake(
        &mut self,
        now: MonotonicTimeNs,
        _w: &WakeDecision,
        policy_blocks_audible: bool,
    ) -> Ph1lOutput {
        match self.state {
            SessionState::Closed | SessionState::SoftClosed => {
                let id = SessionId(self.next_session_id);
                self.next_session_id = self.next_session_id.saturating_add(1);
                self.session_id = Some(id);
                self.pending_question = None;
                self.pending_since = None;
                self.transition(
                    now,
                    SessionState::Active,
                    reason_codes::L_OPEN_WAKE,
                    policy_blocks_audible,
                )
            }
            _ => self.snapshot(None, None, false),
        }
    }

    fn update_pending_question(&mut self, now: MonotonicTimeNs, directive: Option<&Ph1xDirective>) {
        match directive {
            Some(Ph1xDirective::Clarify(_)) => {
                if self.pending_question != Some(PendingQuestion::Clarify) {
                    self.pending_question = Some(PendingQuestion::Clarify);
                    self.pending_since = Some(now);
                }
            }
            Some(Ph1xDirective::Confirm(_)) => {
                if self.pending_question != Some(PendingQuestion::Confirm) {
                    self.pending_question = Some(PendingQuestion::Confirm);
                    self.pending_since = Some(now);
                }
            }
            Some(Ph1xDirective::Wait(_)) => {
                // Waiting is a posture. Preserve any existing pending question; do not reset timers.
            }
            Some(Ph1xDirective::Respond(_)) | Some(Ph1xDirective::Dispatch(_)) => {
                // A new move was taken; no longer waiting for an answer.
                self.pending_question = None;
                self.pending_since = None;
            }
            None => {
                // Treat missing directive as "no update" so a pending question posture persists across ticks.
            }
        }
    }

    fn transition(
        &mut self,
        now: MonotonicTimeNs,
        to: SessionState,
        reason_code: ReasonCodeId,
        policy_blocks_audible: bool,
    ) -> Ph1lOutput {
        let from = self.state;
        if to == SessionState::Closed {
            self.session_id = None;
            self.pending_question = None;
            self.pending_since = None;
        }
        self.state = to;

        let transition = TransitionEvent {
            schema_version: SchemaVersion(1),
            from,
            to,
            reason_code,
            t_event: now,
        };
        self.snapshot(Some(transition), None, policy_blocks_audible)
    }

    fn snapshot(
        &self,
        transition: Option<TransitionEvent>,
        nudge: Option<PresenceNudge>,
        policy_blocks_audible: bool,
    ) -> Ph1lOutput {
        let next_allowed_actions = match self.state {
            SessionState::Closed => NextAllowedActions {
                may_speak: false,
                must_wait: true,
                must_rewake: true,
            },
            SessionState::Open | SessionState::Active => NextAllowedActions {
                may_speak: !policy_blocks_audible,
                must_wait: false,
                must_rewake: false,
            },
            SessionState::SoftClosed => NextAllowedActions {
                may_speak: false,
                must_wait: true,
                must_rewake: true,
            },
            SessionState::Suspended => NextAllowedActions {
                may_speak: false,
                must_wait: true,
                must_rewake: false,
            },
        };

        Ph1lOutput::v1(
            SessionSnapshot {
                schema_version: SchemaVersion(1),
                session_state: self.state,
                session_id: self.session_id,
                next_allowed_actions,
            },
            transition,
            nudge,
        )
    }
}

fn ms_to_ns(ms: u32) -> u64 {
    (ms as u64).saturating_mul(1_000_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1l::{TtsPlaybackState, UserActivitySignals};
    use selene_kernel_contracts::ph1w::{WakeDecision, WakeGateResults};
    use selene_kernel_contracts::ph1x::{ClarifyDirective, Ph1xDirective};
    use selene_kernel_contracts::ReasonCodeId;

    fn policy_ok() -> selene_kernel_contracts::ph1d::PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn accepted_wake(now: MonotonicTimeNs) -> WakeDecision {
        WakeDecision::accept_v1(
            ReasonCodeId(1),
            WakeGateResults {
                g0_integrity_ok: true,
                g1_activity_ok: true,
                g1a_utterance_start_ok: true,
                g2_light_ok: true,
                g3_strong_ok: true,
                g3a_liveness_ok: true,
                g4_personalization_ok: true,
                g5_policy_ok: true,
            },
            now,
            None,
            None,
            selene_kernel_contracts::ph1w::BoundedAudioSegmentRef::v1(
                selene_kernel_contracts::ph1k::AudioStreamId(1),
                selene_kernel_contracts::ph1k::PreRollBufferId(1),
                now,
                MonotonicTimeNs(now.0 + 1),
                now,
                now,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn input(now: u64, silence_ms: u32) -> Ph1lInput {
        Ph1lInput::v1(
            MonotonicTimeNs(now),
            None,
            None,
            TtsPlaybackState::Stopped,
            UserActivitySignals {
                speech_detected: false,
                barge_in: false,
                silence_ms,
            },
            policy_ok(),
            false,
            false,
            false,
        )
    }

    #[test]
    fn at_l_00_active_window_uses_stage6_thirty_seconds() {
        let config = Ph1lConfig::mvp_desktop_v1();
        assert_eq!(config.active_silence_timeout_ms, 30_000);

        let mut rt = Ph1lRuntime::new(config);
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);

        let out = rt.step(input(1, config.active_silence_timeout_ms - 1));
        assert_eq!(out.snapshot.session_state, SessionState::Active);
        let out = rt.step(input(2, config.active_silence_timeout_ms));
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
        assert_eq!(
            out.transition.unwrap().reason_code,
            reason_codes::L_TO_CLOSED_SILENCE
        );
        assert!(out.snapshot.next_allowed_actions.must_rewake);
    }

    #[test]
    fn at_l_01_idle_close_seals_after_thirty_seconds() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);

        // Silence beyond active timeout -> CLOSED.
        let out = rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
        assert!(out.snapshot.session_id.is_none());
    }

    #[test]
    fn at_l_02_idle_close_requires_rewake_for_new_session() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE then idle-close.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);
        let first_session_id = rt.session_id();
        rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(rt.state(), SessionState::Closed);
        assert_eq!(rt.session_id(), None);

        // User speech without wake does not reopen a sealed session.
        let mut i = input(2, 0);
        i.user_activity.speech_detected = true;
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Closed);

        // Wake opens a fresh logical session.
        let mut i = input(3, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(3)));
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);
        assert_ne!(out.snapshot.session_id, first_session_id);
    }

    #[test]
    fn at_l_03_no_premature_close_during_pending_clarify() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        // Pending clarify means we should not close just due to active silence timeout.
        let mut i = input(1, rt.config.active_silence_timeout_ms + 1);
        i.conversation_directive = Some(Ph1xDirective::Clarify(
            ClarifyDirective::v1(
                "When?".to_string(),
                vec!["Tomorrow 3pm".to_string(), "Friday 10am".to_string()],
                vec![selene_kernel_contracts::ph1n::FieldKey::When],
            )
            .unwrap(),
        ));
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);
    }

    #[test]
    fn at_l_04_pending_clarify_timeout_closes_once() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        // Enter pending clarify.
        let mut i = input(0, 0);
        i.conversation_directive = Some(Ph1xDirective::Clarify(
            ClarifyDirective::v1(
                "When?".to_string(),
                vec!["Tomorrow 3pm".to_string(), "Friday 10am".to_string()],
                vec![selene_kernel_contracts::ph1n::FieldKey::When],
            )
            .unwrap(),
        ));
        rt.step(i);

        // After clarify timeout -> CLOSED. There is no separate soft-close stage.
        let out = rt.step(input(
            ms_to_ns(rt.config.clarify_timeout_ms) + 1,
            rt.config.clarify_timeout_ms + 1,
        ));
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
        assert!(out.transition.is_some());
        assert_eq!(
            out.transition.unwrap().reason_code,
            reason_codes::L_TO_CLOSED_SILENCE
        );
    }

    #[test]
    fn at_l_08_no_close_check_prompt_before_idle_close() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());

        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        // Silence beyond active timeout -> CLOSED with no nudge.
        let out = rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
        assert!(out.nudge.is_none());
    }

    #[test]
    fn at_l_09_legacy_soft_closed_snapshot_requires_rewake() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());

        rt.state = SessionState::SoftClosed;
        rt.session_id = Some(SessionId(42));
        let out = rt.step(input(1, 0));
        assert_eq!(out.snapshot.session_state, SessionState::SoftClosed);
        assert!(out.snapshot.next_allowed_actions.must_rewake);
        assert!(!out.snapshot.next_allowed_actions.may_speak);
    }

    #[test]
    fn at_l_10_privacy_dnd_idle_close_stays_non_audible() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        let mut i = input(1, rt.config.active_silence_timeout_ms);
        i.policy_context_ref = PolicyContextRef::v1(true, false, SafetyTier::Standard);
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
        assert_eq!(out.snapshot.next_allowed_actions.may_speak, false);
    }
}
