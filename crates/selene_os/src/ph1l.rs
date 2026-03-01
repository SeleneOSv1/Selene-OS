#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1d::PolicyContextRef;
use selene_kernel_contracts::ph1l::{
    NextAllowedActions, Ph1lInput, Ph1lOutput, PresenceNudge, SessionId, SessionSnapshot,
    TransitionEvent, TtsPlaybackState, UserActivitySignals,
};
use selene_kernel_contracts::ph1w::WakeDecision;
use selene_kernel_contracts::ph1x::Ph1xDirective;
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.L reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const L_OPEN_WAKE: ReasonCodeId = ReasonCodeId(0x4C00_0001);
    pub const L_RESUME_WAKE_SOFT_CLOSE: ReasonCodeId = ReasonCodeId(0x4C00_0002);
    pub const L_TO_SOFT_CLOSE_SILENCE: ReasonCodeId = ReasonCodeId(0x4C00_0003);
    pub const L_TO_CLOSED_SILENCE: ReasonCodeId = ReasonCodeId(0x4C00_0004);
    pub const L_TO_CLOSED_DISMISS: ReasonCodeId = ReasonCodeId(0x4C00_0005);
    pub const L_WAIT_TIMEOUT_PROMPTED: ReasonCodeId = ReasonCodeId(0x4C00_0006);
    pub const L_CLOSE_CHECK_PROMPTED: ReasonCodeId = ReasonCodeId(0x4C00_0007);
    pub const L_RESUME_USER_ACTIVITY: ReasonCodeId = ReasonCodeId(0x4C00_0008);
    pub const L_SUSPEND_AUDIO_DEGRADED: ReasonCodeId = ReasonCodeId(0x4C00_0009);
    pub const L_RESUME_STABLE: ReasonCodeId = ReasonCodeId(0x4C00_000A);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PendingQuestion {
    Clarify,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ph1lTurnTrigger {
    WakeWord,
    Explicit,
    Other,
}

pub fn trigger_requires_session_open_step(trigger: Ph1lTurnTrigger) -> bool {
    matches!(
        trigger,
        Ph1lTurnTrigger::WakeWord | Ph1lTurnTrigger::Explicit
    )
}

pub fn ph1l_step_voice_turn(
    runtime: &mut Ph1lRuntime,
    now: MonotonicTimeNs,
    trigger: Ph1lTurnTrigger,
    wake_event: Option<WakeDecision>,
    tts_state: TtsPlaybackState,
    policy_context_ref: PolicyContextRef,
) -> Ph1lOutput {
    let wake_event = if trigger_requires_session_open_step(trigger) {
        wake_event
    } else {
        None
    };
    runtime.step(Ph1lInput::v1(
        now,
        wake_event,
        None,
        tts_state,
        UserActivitySignals {
            speech_detected: true,
            barge_in: false,
            silence_ms: 0,
        },
        policy_context_ref,
        false,
        false,
        false,
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1lConfig {
    pub active_silence_timeout_ms: u32,
    pub soft_close_timeout_ms: u32,
    pub clarify_timeout_ms: u32,
    pub confirm_timeout_ms: u32,
    pub close_check_quiet_timeout_ms: u32,
    pub close_check_repeat_timeout_ms: u32,
    pub close_check_max_attempts: u8,
}

impl Ph1lConfig {
    pub fn mvp_desktop_v1() -> Self {
        Self {
            active_silence_timeout_ms: 10_000,
            soft_close_timeout_ms: 120_000,
            clarify_timeout_ms: 30_000,
            confirm_timeout_ms: 30_000,
            close_check_quiet_timeout_ms: 25_000,
            close_check_repeat_timeout_ms: 60_000,
            close_check_max_attempts: 2,
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
    soft_closed_since: Option<MonotonicTimeNs>,
    close_check_attempts: u8,
    close_check_last_prompt_at: Option<MonotonicTimeNs>,
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
            soft_closed_since: None,
            close_check_attempts: 0,
            close_check_last_prompt_at: None,
        }
    }

    pub fn from_persisted_state(
        config: Ph1lConfig,
        state: SessionState,
        session_id: Option<SessionId>,
        next_session_id: u128,
    ) -> Result<Self, ContractViolation> {
        let next_allowed_actions = default_next_allowed_actions_for_state(state);
        SessionSnapshot {
            schema_version: SchemaVersion(1),
            session_state: state,
            session_id,
            next_allowed_actions,
        }
        .validate()?;
        let min_next = session_id
            .map(|id| id.0.saturating_add(1))
            .unwrap_or(1)
            .max(1);
        Ok(Self {
            config,
            state,
            session_id,
            next_session_id: next_session_id.max(min_next),
            pending_question: None,
            pending_since: None,
            soft_closed_since: None,
            close_check_attempts: 0,
            close_check_last_prompt_at: None,
        })
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

        // Resume from SOFT_CLOSED without re-wake if the user speaks.
        if self.state == SessionState::SoftClosed && input.user_activity.speech_detected {
            self.soft_closed_since = None;
            self.close_check_attempts = 0;
            self.close_check_last_prompt_at = None;
            return self.transition(
                input.now,
                SessionState::Active,
                reason_codes::L_RESUME_USER_ACTIVITY,
                policy_blocks_audible,
            );
        }

        // Track "waiting for user" posture based on PH1.X directive.
        self.update_pending_question(input.now, input.conversation_directive.as_ref());

        // Never soft-close while TTS is playing.
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
                    // One gentle prompt signal then soft-close.
                    self.pending_question = None;
                    self.pending_since = None;
                    return self.transition(
                        input.now,
                        SessionState::SoftClosed,
                        reason_codes::L_WAIT_TIMEOUT_PROMPTED,
                        policy_blocks_audible,
                    );
                }
            }

            // Stay ACTIVE while waiting (do not soft-close on short silence).
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

        // Silence-driven soft-close.
        if self.state == SessionState::Active
            && input.user_activity.silence_ms >= self.config.active_silence_timeout_ms
        {
            return self.transition(
                input.now,
                SessionState::SoftClosed,
                reason_codes::L_TO_SOFT_CLOSE_SILENCE,
                policy_blocks_audible,
            );
        }

        // Close-check prompt (bounded, deterministic) before fully closing.
        // Do not emit a prompt at or after the hard close threshold.
        if self.state == SessionState::SoftClosed
            && !policy_blocks_audible
            && input.user_activity.silence_ms < self.config.soft_close_timeout_ms
        {
            if let Some(n) = self.maybe_close_check_prompt(input.now) {
                return self.snapshot(None, Some(n), policy_blocks_audible);
            }
        }

        // Silence-driven close.
        if self.state == SessionState::SoftClosed
            && input.user_activity.silence_ms >= self.config.soft_close_timeout_ms
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
            SessionState::Closed => {
                let id = SessionId(self.next_session_id);
                self.next_session_id = self.next_session_id.saturating_add(1);
                self.session_id = Some(id);
                self.pending_question = None;
                self.pending_since = None;
                self.soft_closed_since = None;
                self.close_check_attempts = 0;
                self.close_check_last_prompt_at = None;
                self.transition(
                    now,
                    SessionState::Active,
                    reason_codes::L_OPEN_WAKE,
                    policy_blocks_audible,
                )
            }
            SessionState::SoftClosed => {
                // Resume same session.
                self.pending_question = None;
                self.pending_since = None;
                self.soft_closed_since = None;
                self.close_check_attempts = 0;
                self.close_check_last_prompt_at = None;
                self.transition(
                    now,
                    SessionState::Active,
                    reason_codes::L_RESUME_WAKE_SOFT_CLOSE,
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
            self.soft_closed_since = None;
            self.close_check_attempts = 0;
            self.close_check_last_prompt_at = None;
        }
        if to == SessionState::SoftClosed && from != SessionState::SoftClosed {
            self.soft_closed_since = Some(now);
            self.close_check_attempts = 0;
            self.close_check_last_prompt_at = None;
        }
        if to == SessionState::Active && from == SessionState::SoftClosed {
            self.soft_closed_since = None;
            self.close_check_attempts = 0;
            self.close_check_last_prompt_at = None;
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
            SessionState::Open | SessionState::Active | SessionState::SoftClosed => {
                NextAllowedActions {
                    may_speak: !policy_blocks_audible,
                    must_wait: false,
                    must_rewake: false,
                }
            }
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

    fn maybe_close_check_prompt(&mut self, now: MonotonicTimeNs) -> Option<PresenceNudge> {
        let since = self.soft_closed_since?;
        let quiet_for_ns = now.0.saturating_sub(since.0);
        if quiet_for_ns < ms_to_ns(self.config.close_check_quiet_timeout_ms) {
            return None;
        }
        if self.close_check_attempts >= self.config.close_check_max_attempts {
            return None;
        }
        if let Some(last) = self.close_check_last_prompt_at {
            let since_last_ns = now.0.saturating_sub(last.0);
            if since_last_ns < ms_to_ns(self.config.close_check_repeat_timeout_ms) {
                return None;
            }
        }

        let attempt = self.close_check_attempts.saturating_add(1);
        let (variant, text) = close_check_phrase(self.session_id, attempt);
        let prompt = PresenceNudge::close_check_v1(
            attempt,
            variant,
            text.to_string(),
            reason_codes::L_CLOSE_CHECK_PROMPTED,
            now,
        )
        .expect("close_check prompt must validate");

        self.close_check_attempts = attempt;
        self.close_check_last_prompt_at = Some(now);
        Some(prompt)
    }
}

fn default_next_allowed_actions_for_state(state: SessionState) -> NextAllowedActions {
    match state {
        SessionState::Closed => NextAllowedActions {
            may_speak: false,
            must_wait: true,
            must_rewake: true,
        },
        SessionState::Open | SessionState::Active | SessionState::SoftClosed => {
            NextAllowedActions {
                may_speak: true,
                must_wait: false,
                must_rewake: false,
            }
        }
        SessionState::Suspended => NextAllowedActions {
            may_speak: false,
            must_wait: true,
            must_rewake: false,
        },
    }
}

fn ms_to_ns(ms: u32) -> u64 {
    (ms as u64).saturating_mul(1_000_000)
}

fn close_check_phrase(session_id: Option<SessionId>, attempt: u8) -> (u8, &'static str) {
    // "Random" feel via deterministic rotation (no true randomness).
    const PHRASES: [&str; 5] = [
        "Are we finished with this topic?",
        "Do you want to keep going on this, or are we done?",
        "Anything else on this topic?",
        "Are you done with this for now?",
        "Should I stay with this topic, or can I close it?",
    ];

    let sid = session_id.map(|s| s.0).unwrap_or(0);
    let lo = sid as u64;
    let hi = (sid >> 64) as u64;
    let seed = lo ^ hi ^ (attempt as u64);
    let idx = (seed % (PHRASES.len() as u64)) as usize;
    (idx as u8, PHRASES[idx])
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::UserId;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1j::DeviceId;
    use selene_kernel_contracts::ph1l::{TtsPlaybackState, UserActivitySignals};
    use selene_kernel_contracts::ph1w::{WakeDecision, WakeGateResults};
    use selene_kernel_contracts::ph1x::{ClarifyDirective, Ph1xDirective};
    use selene_kernel_contracts::ReasonCodeId;
    use selene_storage::ph1f::{
        DeviceRecord, IdentityRecord, IdentityStatus, Ph1fStore, SessionRecord,
    };

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

    fn seed_identity_and_device(store: &mut Ph1fStore, user_id: &UserId, device_id: &DeviceId) {
        store
            .insert_identity(IdentityRecord::v1(
                user_id.clone(),
                None,
                None,
                MonotonicTimeNs(1),
                IdentityStatus::Active,
            ))
            .unwrap();
        store
            .insert_device(
                DeviceRecord::v1(
                    device_id.clone(),
                    user_id.clone(),
                    "desktop".to_string(),
                    MonotonicTimeNs(1),
                    None,
                )
                .unwrap(),
            )
            .unwrap();
    }

    fn persist_snapshot_for_test(
        store: &mut Ph1fStore,
        user_id: &UserId,
        device_id: &DeviceId,
        previous_session_id: Option<SessionId>,
        out: &Ph1lOutput,
        now: MonotonicTimeNs,
        idempotency_key: &str,
    ) -> SessionId {
        let session_id = if out.snapshot.session_state == SessionState::Closed {
            previous_session_id.expect("previous session id must exist for closed snapshot")
        } else {
            out.snapshot
                .session_id
                .expect("open/active snapshot must have session id")
        };
        let opened_at = store
            .get_session(&session_id)
            .map(|row| row.opened_at)
            .unwrap_or(now);
        let closed_at = if out.snapshot.session_state == SessionState::Closed {
            Some(now)
        } else {
            None
        };
        let row = SessionRecord::v1(
            session_id,
            user_id.clone(),
            device_id.clone(),
            out.snapshot.session_state,
            opened_at,
            now,
            closed_at,
        )
        .unwrap();
        store
            .upsert_session_lifecycle(row, Some(idempotency_key.to_string()))
            .unwrap();
        session_id
    }

    fn run_trigger_step_for_test(
        trigger: Ph1lTurnTrigger,
        now: MonotonicTimeNs,
    ) -> (Ph1lOutput, Option<SessionId>) {
        let mut rt = Ph1lRuntime::from_persisted_state(
            Ph1lConfig::mvp_desktop_v1(),
            SessionState::Closed,
            None,
            1,
        )
        .expect("runtime bootstrap should succeed");
        let out = ph1l_step_voice_turn(
            &mut rt,
            now,
            trigger,
            Some(accepted_wake(now)),
            TtsPlaybackState::Stopped,
            policy_ok(),
        );
        (out, rt.session_id())
    }

    #[test]
    fn at_trigger_os_01_wakeword_and_explicit_both_call_ph1l_step() {
        let now = MonotonicTimeNs(10);
        let (wake_out, wake_sid) = run_trigger_step_for_test(Ph1lTurnTrigger::WakeWord, now);
        let (explicit_out, explicit_sid) =
            run_trigger_step_for_test(Ph1lTurnTrigger::Explicit, now);

        assert_eq!(wake_out.snapshot.session_state, SessionState::Active);
        assert_eq!(explicit_out.snapshot.session_state, SessionState::Active);
        assert_eq!(wake_out.snapshot.session_id, Some(SessionId(1)));
        assert_eq!(explicit_out.snapshot.session_id, Some(SessionId(1)));
        assert_eq!(wake_sid, explicit_sid);
    }

    #[test]
    fn at_trigger_01_os_wakeword_and_explicit_both_call_ph1l_step() {
        at_trigger_os_01_wakeword_and_explicit_both_call_ph1l_step();
    }

    #[test]
    fn at_trigger_os_02_session_id_persisted_identically_for_both_triggers() {
        let now = MonotonicTimeNs(20);
        let user_id = UserId::new("tenant_a:trigger_user").unwrap();
        let device_id = DeviceId::new("trigger_device_1").unwrap();

        let mut wake_store = Ph1fStore::new_in_memory();
        let mut explicit_store = Ph1fStore::new_in_memory();
        seed_identity_and_device(&mut wake_store, &user_id, &device_id);
        seed_identity_and_device(&mut explicit_store, &user_id, &device_id);

        let (wake_out, wake_prev_sid) = run_trigger_step_for_test(Ph1lTurnTrigger::WakeWord, now);
        let wake_sid = persist_snapshot_for_test(
            &mut wake_store,
            &user_id,
            &device_id,
            wake_prev_sid,
            &wake_out,
            now,
            "trigger_os_wake_commit",
        );

        let (explicit_out, explicit_prev_sid) =
            run_trigger_step_for_test(Ph1lTurnTrigger::Explicit, now);
        let explicit_sid = persist_snapshot_for_test(
            &mut explicit_store,
            &user_id,
            &device_id,
            explicit_prev_sid,
            &explicit_out,
            now,
            "trigger_os_explicit_commit",
        );

        let wake_row = wake_store
            .get_session(&wake_sid)
            .expect("wake row must exist");
        let explicit_row = explicit_store
            .get_session(&explicit_sid)
            .expect("explicit row must exist");
        assert_eq!(wake_sid, explicit_sid);
        assert_eq!(wake_row.session_state, explicit_row.session_state);
        assert_eq!(wake_row.opened_at, explicit_row.opened_at);
        assert_eq!(wake_row.last_activity_at, explicit_row.last_activity_at);
        assert_eq!(wake_row.closed_at, explicit_row.closed_at);
    }

    #[test]
    fn at_trigger_02_os_session_id_persisted_identically_for_both_triggers() {
        at_trigger_os_02_session_id_persisted_identically_for_both_triggers();
    }

    #[test]
    fn at_l_01_soft_close_feels_human() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);

        // Silence beyond active timeout -> SOFT_CLOSED.
        let out = rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(out.snapshot.session_state, SessionState::SoftClosed);
    }

    #[test]
    fn at_l_02_resume_without_rewake_during_soft_close() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE then soft-close.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);
        rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(rt.state(), SessionState::SoftClosed);

        // User speaks -> ACTIVE.
        let mut i = input(2, 0);
        i.user_activity.speech_detected = true;
        let out = rt.step(i);
        assert_eq!(out.snapshot.session_state, SessionState::Active);
    }

    #[test]
    fn at_l_03_no_premature_close_during_pending_clarify() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());
        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        // Pending clarify means we should not soft-close just due to active silence timeout.
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
    fn at_l_04_one_prompt_then_soft_close_then_close() {
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

        // After clarify timeout -> SOFT_CLOSED with prompt reason.
        let out = rt.step(input(
            ms_to_ns(rt.config.clarify_timeout_ms) + 1,
            rt.config.clarify_timeout_ms + 1,
        ));
        assert_eq!(out.snapshot.session_state, SessionState::SoftClosed);
        assert!(out.transition.is_some());
        assert_eq!(
            out.transition.unwrap().reason_code,
            reason_codes::L_WAIT_TIMEOUT_PROMPTED
        );

        // After soft close timeout -> CLOSED.
        let out = rt.step(input(
            ms_to_ns(rt.config.soft_close_timeout_ms) + 2,
            rt.config.soft_close_timeout_ms + 2,
        ));
        assert_eq!(out.snapshot.session_state, SessionState::Closed);
    }

    #[test]
    fn at_l_08_close_check_prompt_before_fully_closing() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());

        // Wake into ACTIVE.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);

        // Silence beyond active timeout -> SOFT_CLOSED.
        let out = rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(out.snapshot.session_state, SessionState::SoftClosed);

        // Quiet for close_check_quiet_timeout_ms while still below soft_close_timeout_ms => nudge.
        let now = 1 + ms_to_ns(rt.config.close_check_quiet_timeout_ms) + 2;
        let silence_ms =
            rt.config.active_silence_timeout_ms + rt.config.close_check_quiet_timeout_ms;
        let out = rt.step(input(now, silence_ms));
        let n = out.nudge.expect("expected close_check nudge");
        assert_eq!(n.reason_code, reason_codes::L_CLOSE_CHECK_PROMPTED);
        assert_eq!(n.attempt, 1);
        assert!(n.prompt_text.ends_with('?'));
    }

    #[test]
    fn at_l_09_close_check_is_bounded_and_repeats_deterministically() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());

        // Wake into ACTIVE, then SOFT_CLOSED.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);
        rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(rt.state(), SessionState::SoftClosed);

        // First prompt.
        let now1 = 1 + ms_to_ns(rt.config.close_check_quiet_timeout_ms) + 2;
        let silence1 = rt.config.active_silence_timeout_ms + rt.config.close_check_quiet_timeout_ms;
        let out = rt.step(input(now1, silence1));
        let n1 = out.nudge.expect("expected first close_check nudge");
        assert_eq!(n1.attempt, 1);

        // Second prompt after repeat timeout (still below hard close).
        let now2 = now1 + ms_to_ns(rt.config.close_check_repeat_timeout_ms) + 2;
        let silence2 = silence1 + rt.config.close_check_repeat_timeout_ms;
        let out = rt.step(input(now2, silence2));
        let n2 = out.nudge.expect("expected second close_check nudge");
        assert_eq!(n2.attempt, 2);
        assert_ne!(n1.variant, n2.variant);

        // Further prompts are suppressed after max_attempts.
        let now3 = now2 + ms_to_ns(rt.config.close_check_repeat_timeout_ms) + 2;
        let silence3 = silence2 + rt.config.close_check_repeat_timeout_ms;
        let out = rt.step(input(now3, silence3));
        assert!(out.nudge.is_none());
    }

    #[test]
    fn at_l_10_no_close_check_prompt_in_privacy_or_dnd() {
        let mut rt = Ph1lRuntime::new(Ph1lConfig::mvp_desktop_v1());

        // Wake into ACTIVE, then SOFT_CLOSED.
        let mut i = input(0, 0);
        i.wake_event = Some(accepted_wake(MonotonicTimeNs(0)));
        rt.step(i);
        rt.step(input(1, rt.config.active_silence_timeout_ms));
        assert_eq!(rt.state(), SessionState::SoftClosed);

        let now = 1 + ms_to_ns(rt.config.close_check_quiet_timeout_ms) + 2;
        let silence_ms =
            rt.config.active_silence_timeout_ms + rt.config.close_check_quiet_timeout_ms;
        let mut i = input(now, silence_ms);
        i.policy_context_ref = PolicyContextRef::v1(true, false, SafetyTier::Standard);
        let out = rt.step(i);
        assert!(out.nudge.is_none());
        assert_eq!(out.snapshot.next_allowed_actions.may_speak, false);
    }
}
