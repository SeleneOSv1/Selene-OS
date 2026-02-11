#![forbid(unsafe_code)]

use crate::ph1d::PolicyContextRef;
use crate::ph1w::WakeDecision;
use crate::ph1x::Ph1xDirective;
use crate::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, SessionState, Validate,
};

pub const PH1L_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionId(pub u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NextAllowedActions {
    pub may_speak: bool,
    pub must_wait: bool,
    pub must_rewake: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TtsPlaybackState {
    Playing,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct UserActivitySignals {
    pub speech_detected: bool,
    pub barge_in: bool,
    pub silence_ms: u32,
}

impl Validate for UserActivitySignals {
    fn validate(&self) -> Result<(), ContractViolation> {
        // Keep silence bounded so timeouts are stable and non-pathological.
        if self.silence_ms > 86_400_000 {
            return Err(ContractViolation::InvalidValue {
                field: "user_activity_signals.silence_ms",
                reason: "must be <= 86400000 (24h)",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1lInput {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub wake_event: Option<WakeDecision>,
    pub conversation_directive: Option<Ph1xDirective>,
    pub tts_state: TtsPlaybackState,
    pub user_activity: UserActivitySignals,
    pub policy_context_ref: PolicyContextRef,
    /// True when a Resume Buffer is live and must not be discarded by closing.
    pub resume_buffer_live: bool,
    /// True when the user explicitly dismisses/ends the conversation ("thanks that's all", etc).
    pub user_dismissed: bool,
    /// True when audio integrity is degraded (e.g., mic disconnected). Fail closed.
    pub audio_degraded: bool,
}

impl Ph1lInput {
    pub fn v1(
        now: MonotonicTimeNs,
        wake_event: Option<WakeDecision>,
        conversation_directive: Option<Ph1xDirective>,
        tts_state: TtsPlaybackState,
        user_activity: UserActivitySignals,
        policy_context_ref: PolicyContextRef,
        resume_buffer_live: bool,
        user_dismissed: bool,
        audio_degraded: bool,
    ) -> Self {
        Self {
            schema_version: PH1L_CONTRACT_VERSION,
            now,
            wake_event,
            conversation_directive,
            tts_state,
            user_activity,
            policy_context_ref,
            resume_buffer_live,
            user_dismissed,
            audio_degraded,
        }
    }
}

impl Validate for Ph1lInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1L_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1l_input.schema_version",
                reason: "must match PH1L_CONTRACT_VERSION",
            });
        }
        self.policy_context_ref.validate()?;
        self.user_activity.validate()?;
        if let Some(w) = &self.wake_event {
            w.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TransitionEvent {
    pub schema_version: SchemaVersion,
    pub from: SessionState,
    pub to: SessionState,
    pub reason_code: ReasonCodeId,
    pub t_event: MonotonicTimeNs,
}

impl Validate for TransitionEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1L_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "transition_event.schema_version",
                reason: "must match PH1L_CONTRACT_VERSION",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionSnapshot {
    pub schema_version: SchemaVersion,
    pub session_state: SessionState,
    pub session_id: Option<SessionId>,
    pub next_allowed_actions: NextAllowedActions,
}

impl Validate for SessionSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1L_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "session_snapshot.schema_version",
                reason: "must match PH1L_CONTRACT_VERSION",
            });
        }
        match (self.session_state, self.session_id.is_some()) {
            (SessionState::Closed, false) => {}
            (SessionState::Closed, true) => {
                return Err(ContractViolation::InvalidValue {
                    field: "session_snapshot.session_id",
                    reason: "must be None when session_state=Closed",
                });
            }
            (_, true) => {}
            (_, false) => {
                return Err(ContractViolation::InvalidValue {
                    field: "session_snapshot.session_id",
                    reason: "must be Some(...) when session_state != Closed",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PresenceNudgeKind {
    /// Ask the user if they are finished before fully closing.
    CloseCheck,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresenceNudge {
    pub schema_version: SchemaVersion,
    pub kind: PresenceNudgeKind,
    /// 1-based attempt counter within the current SOFT_CLOSED window.
    pub attempt: u8,
    /// Deterministic variant id for phrase rotation.
    pub variant: u8,
    /// The short line Selene should say (must be a question).
    pub prompt_text: String,
    /// Deterministic reason code for audit and replay.
    pub reason_code: ReasonCodeId,
    pub t_event: MonotonicTimeNs,
}

impl PresenceNudge {
    pub fn close_check_v1(
        attempt: u8,
        variant: u8,
        prompt_text: String,
        reason_code: ReasonCodeId,
        t_event: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let n = Self {
            schema_version: PH1L_CONTRACT_VERSION,
            kind: PresenceNudgeKind::CloseCheck,
            attempt,
            variant,
            prompt_text,
            reason_code,
            t_event,
        };
        n.validate()?;
        Ok(n)
    }
}

impl Validate for PresenceNudge {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1L_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.schema_version",
                reason: "must match PH1L_CONTRACT_VERSION",
            });
        }
        if self.attempt == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.attempt",
                reason: "must be >= 1",
            });
        }
        if self.prompt_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.prompt_text",
                reason: "must not be empty",
            });
        }
        if self.prompt_text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.prompt_text",
                reason: "must be <= 256 chars",
            });
        }
        // Keep it a question; avoid accidental statements when nudging close.
        if !self.prompt_text.trim_end().ends_with('?') {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.prompt_text",
                reason: "must end with '?'",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "presence_nudge.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1lOutput {
    pub schema_version: SchemaVersion,
    pub snapshot: SessionSnapshot,
    pub transition: Option<TransitionEvent>,
    pub nudge: Option<PresenceNudge>,
}

impl Ph1lOutput {
    pub fn v1(
        snapshot: SessionSnapshot,
        transition: Option<TransitionEvent>,
        nudge: Option<PresenceNudge>,
    ) -> Self {
        Self {
            schema_version: PH1L_CONTRACT_VERSION,
            snapshot,
            transition,
            nudge,
        }
    }
}

impl Validate for Ph1lOutput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1L_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1l_output.schema_version",
                reason: "must match PH1L_CONTRACT_VERSION",
            });
        }
        self.snapshot.validate()?;
        if let Some(t) = &self.transition {
            t.validate()?;
        }
        if let Some(n) = &self.nudge {
            n.validate()?;
        }
        Ok(())
    }
}
