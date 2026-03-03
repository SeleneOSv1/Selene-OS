#![forbid(unsafe_code)]

use crate::web_search_plan::proxy::{ProxyErrorKind, ProxyMode};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub const FIXED_BACKOFF_SCHEDULE_MS: [u64; 3] = [0, 250, 1000];
pub const MAX_ATTEMPTS: usize = FIXED_BACKOFF_SCHEDULE_MS.len();
const COOLDOWN_TRIGGER_CONSECUTIVE_FAILURES: u32 = 3;
const FAILURE_COOLDOWN_MS: u64 = 30_000;

pub fn fixed_backoff_schedule_ms() -> &'static [u64] {
    &FIXED_BACKOFF_SCHEDULE_MS
}

pub fn backoff_for_attempt(attempt_index: usize) -> Option<u64> {
    FIXED_BACKOFF_SCHEDULE_MS.get(attempt_index).copied()
}

pub fn diagnostic_cooldown_ms(mode: ProxyMode, error_kind: ProxyErrorKind) -> u64 {
    match (mode, error_kind) {
        (ProxyMode::Off, _) => 0,
        (ProxyMode::Env, ProxyErrorKind::ProxyMisconfigured) => 2_000,
        (ProxyMode::Explicit, ProxyErrorKind::ProxyMisconfigured) => 5_000,
        (_, ProxyErrorKind::ProxyTimeout) => 1_000,
        (_, ProxyErrorKind::ProxyDnsFailed) => 2_000,
        (_, ProxyErrorKind::ProxyConnectFailed) => 2_500,
        (_, ProxyErrorKind::ProxyTlsFailed) => 3_000,
        (_, ProxyErrorKind::ProxyAuthFailed) => 3_000,
    }
}

pub trait MonotonicClock {
    fn now_ms(&self) -> u64;
}

#[derive(Debug, Clone, Default)]
pub struct FakeClock {
    now_ms: Rc<Cell<u64>>,
}

impl FakeClock {
    pub fn new(initial_ms: u64) -> Self {
        Self {
            now_ms: Rc::new(Cell::new(initial_ms)),
        }
    }

    pub fn advance_ms(&self, delta_ms: u64) {
        self.now_ms.set(self.now_ms.get().saturating_add(delta_ms));
    }
}

impl MonotonicClock for FakeClock {
    fn now_ms(&self) -> u64 {
        self.now_ms.get()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct DiagnosticKey {
    mode: ProxyMode,
    error_kind: ProxyErrorKind,
}

#[derive(Debug)]
pub struct DiagnosticRateLimiter<C: MonotonicClock> {
    clock: C,
    next_emit_ms: BTreeMap<DiagnosticKey, u64>,
}

impl<C: MonotonicClock> DiagnosticRateLimiter<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            next_emit_ms: BTreeMap::new(),
        }
    }

    pub fn should_emit(&mut self, mode: ProxyMode, error_kind: ProxyErrorKind) -> bool {
        let key = DiagnosticKey { mode, error_kind };
        let now = self.clock.now_ms();
        let next = self.next_emit_ms.get(&key).copied().unwrap_or(0);
        if now < next {
            return false;
        }
        let cooldown_ms = diagnostic_cooldown_ms(mode, error_kind);
        self.next_emit_ms
            .insert(key, now.saturating_add(cooldown_ms));
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FailureSignature {
    pub mode: ProxyMode,
    pub error_kind: ProxyErrorKind,
    pub redacted_proxy_target: String,
}

#[derive(Debug)]
struct FailureState {
    consecutive_failures: u32,
    cooldown_until_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureCooldownOutcome {
    pub blocked_by_cooldown: bool,
    pub cooldown_engaged: bool,
    pub cooldown_until_ms: Option<u64>,
}

#[derive(Debug)]
pub struct FailureCooldownTracker<C: MonotonicClock> {
    clock: C,
    states: BTreeMap<FailureSignature, FailureState>,
}

impl<C: MonotonicClock> FailureCooldownTracker<C> {
    pub fn new(clock: C) -> Self {
        Self {
            clock,
            states: BTreeMap::new(),
        }
    }

    pub fn record_failure(&mut self, signature: FailureSignature) -> FailureCooldownOutcome {
        let now = self.clock.now_ms();
        let state = self.states.entry(signature).or_insert(FailureState {
            consecutive_failures: 0,
            cooldown_until_ms: 0,
        });

        if now < state.cooldown_until_ms {
            return FailureCooldownOutcome {
                blocked_by_cooldown: true,
                cooldown_engaged: true,
                cooldown_until_ms: Some(state.cooldown_until_ms),
            };
        }

        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures >= COOLDOWN_TRIGGER_CONSECUTIVE_FAILURES {
            state.cooldown_until_ms = now.saturating_add(FAILURE_COOLDOWN_MS);
            state.consecutive_failures = 0;
            return FailureCooldownOutcome {
                blocked_by_cooldown: false,
                cooldown_engaged: true,
                cooldown_until_ms: Some(state.cooldown_until_ms),
            };
        }

        FailureCooldownOutcome {
            blocked_by_cooldown: false,
            cooldown_engaged: false,
            cooldown_until_ms: None,
        }
    }

    pub fn can_attempt(&self, signature: &FailureSignature) -> bool {
        match self.states.get(signature) {
            Some(state) => self.clock.now_ms() >= state.cooldown_until_ms,
            None => true,
        }
    }
}
