#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::load_turn_state_machine;
use crate::web_search_plan::turn_state_machine_validator::{
    validate_fail_closed_reason_codes, validate_transition_sequence, validate_turn_state_machine_spec,
};
use serde::{Deserialize, Serialize};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnStateTransition {
    pub from: String,
    pub to: String,
    pub at_ms: i64,
}

pub trait MonotonicClock {
    fn now_ms(&self) -> i64;
}

#[derive(Debug, Clone)]
pub struct SystemMonotonicClock {
    origin_unix_ms: i64,
    started: Instant,
}

impl Default for SystemMonotonicClock {
    fn default() -> Self {
        let origin_unix_ms = now_unix_ms();
        Self {
            origin_unix_ms,
            started: Instant::now(),
        }
    }
}

impl MonotonicClock for SystemMonotonicClock {
    fn now_ms(&self) -> i64 {
        self.origin_unix_ms
            .saturating_add(self.started.elapsed().as_millis() as i64)
    }
}

#[derive(Debug, Clone)]
pub struct StateTraceRecorder<C: MonotonicClock> {
    clock: C,
    current_state: String,
    transitions: Vec<TurnStateTransition>,
}

impl<C: MonotonicClock> StateTraceRecorder<C> {
    pub fn new(clock: C, initial_state: &str) -> Result<Self, String> {
        if initial_state.trim().is_empty() {
            return Err("initial_state must not be empty".to_string());
        }
        Ok(Self {
            clock,
            current_state: initial_state.trim().to_string(),
            transitions: Vec::new(),
        })
    }

    pub fn transition(&mut self, to_state: &str) -> Result<(), String> {
        let to = to_state.trim();
        if to.is_empty() {
            return Err("to_state must not be empty".to_string());
        }

        let transition = TurnStateTransition {
            from: self.current_state.clone(),
            to: to.to_string(),
            at_ms: self.clock.now_ms(),
        };

        let mut candidate = self.transitions.clone();
        candidate.push(transition.clone());
        let fail_closed_reason_codes = if to == "TURN_FAILED_CLOSED" {
            vec!["provider_upstream_failed".to_string()]
        } else {
            Vec::new()
        };
        validate_turn_state_transitions(&candidate, &fail_closed_reason_codes)?;

        self.transitions.push(transition);
        self.current_state = to.to_string();
        Ok(())
    }

    pub fn current_state(&self) -> &str {
        &self.current_state
    }

    pub fn transitions(&self) -> &[TurnStateTransition] {
        &self.transitions
    }

    pub fn into_transitions(self) -> Vec<TurnStateTransition> {
        self.transitions
    }
}

pub fn default_failed_transitions(at_ms: i64) -> Vec<TurnStateTransition> {
    vec![
        TurnStateTransition {
            from: "TURN_ACCEPTED".to_string(),
            to: "INPUT_PARSED".to_string(),
            at_ms,
        },
        TurnStateTransition {
            from: "INPUT_PARSED".to_string(),
            to: "TURN_FAILED_CLOSED".to_string(),
            at_ms: at_ms.saturating_add(1),
        },
    ]
}

pub fn default_degraded_transitions(at_ms: i64) -> Vec<TurnStateTransition> {
    vec![
        TurnStateTransition {
            from: "TURN_ACCEPTED".to_string(),
            to: "INPUT_PARSED".to_string(),
            at_ms,
        },
        TurnStateTransition {
            from: "INPUT_PARSED".to_string(),
            to: "INTENT_CLASSIFIED".to_string(),
            at_ms: at_ms.saturating_add(1),
        },
        TurnStateTransition {
            from: "INTENT_CLASSIFIED".to_string(),
            to: "PLAN_SELECTED".to_string(),
            at_ms: at_ms.saturating_add(2),
        },
        TurnStateTransition {
            from: "PLAN_SELECTED".to_string(),
            to: "RETRIEVAL_EXECUTED".to_string(),
            at_ms: at_ms.saturating_add(3),
        },
    ]
}

pub fn validate_turn_state_transitions(
    transitions: &[TurnStateTransition],
    reason_codes: &[String],
) -> Result<(), String> {
    if transitions.is_empty() {
        return Err("turn_state_transitions must not be empty".to_string());
    }

    for transition in transitions {
        if transition.from.trim().is_empty() || transition.to.trim().is_empty() {
            return Err("turn_state_transitions cannot contain empty states".to_string());
        }
    }

    for window in transitions.windows(2) {
        let first = &window[0];
        let second = &window[1];
        if first.to != second.from {
            return Err(format!(
                "turn_state_transitions must be contiguous; found {} -> {} then {} -> {}",
                first.from, first.to, second.from, second.to
            ));
        }
        if second.at_ms < first.at_ms {
            return Err("turn_state_transitions timestamps must be non-decreasing".to_string());
        }
    }

    let mut states = Vec::with_capacity(transitions.len().saturating_add(1));
    states.push(transitions[0].from.clone());
    states.extend(transitions.iter().map(|t| t.to.clone()));

    let machine = load_turn_state_machine()?;
    validate_turn_state_machine_spec(&machine)?;
    validate_transition_sequence(&machine, &states)?;
    validate_fail_closed_reason_codes(&machine, &states, reason_codes)?;

    Ok(())
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or(0)
}
