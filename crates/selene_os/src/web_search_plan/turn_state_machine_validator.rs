#![forbid(unsafe_code)]

use crate::web_search_plan::registry_loader::TurnStateMachine;
use std::collections::BTreeSet;

pub fn validate_turn_state_machine_spec(spec: &TurnStateMachine) -> Result<(), String> {
    if spec.registry_version.trim().is_empty() {
        return Err("turn state machine registry_version must not be empty".to_string());
    }
    if spec.states.is_empty() {
        return Err("turn state machine states must not be empty".to_string());
    }
    let mut state_set = BTreeSet::new();
    for state in &spec.states {
        if state.trim().is_empty() {
            return Err("turn state machine state must not be empty".to_string());
        }
        if !state_set.insert(state.as_str()) {
            return Err(format!("duplicate state {}", state));
        }
    }
    if !state_set.contains(spec.failure_state.as_str()) {
        return Err(format!(
            "failure_state {} is not in states",
            spec.failure_state
        ));
    }
    if spec.gate_order.len() != 9 {
        return Err(format!(
            "gate_order must contain 9 entries, got {}",
            spec.gate_order.len()
        ));
    }
    for transition in &spec.allowed_transitions {
        if transition.from != "*" && !state_set.contains(transition.from.as_str()) {
            return Err(format!("transition.from unknown state {}", transition.from));
        }
        if !state_set.contains(transition.to.as_str()) {
            return Err(format!("transition.to unknown state {}", transition.to));
        }
    }
    Ok(())
}

pub fn validate_transition_sequence(
    spec: &TurnStateMachine,
    states: &[String],
) -> Result<(), String> {
    if states.is_empty() {
        return Err("state sequence must not be empty".to_string());
    }
    if states[0] != "TURN_ACCEPTED" {
        return Err("state sequence must start at TURN_ACCEPTED".to_string());
    }

    for window in states.windows(2) {
        let from = &window[0];
        let to = &window[1];
        if !is_transition_allowed(spec, from, to) {
            return Err(format!("illegal transition {} -> {}", from, to));
        }
    }

    for (idx, state) in states.iter().enumerate() {
        if state == "AUDIT_COMMITTED" {
            if idx == 0 {
                return Err("AUDIT_COMMITTED cannot be first state".to_string());
            }
            let prev = &states[idx - 1];
            if prev != "OUTPUT_RENDERED" && prev != "TURN_FAILED_CLOSED" {
                return Err(format!(
                    "AUDIT_COMMITTED must follow OUTPUT_RENDERED or TURN_FAILED_CLOSED, got {}",
                    prev
                ));
            }
        }
    }

    Ok(())
}

pub fn validate_fail_closed_reason_codes(
    spec: &TurnStateMachine,
    states: &[String],
    reason_codes: &[String],
) -> Result<(), String> {
    if states.iter().any(|state| state == &spec.failure_state) && reason_codes.is_empty() {
        return Err("TURN_FAILED_CLOSED requires at least one reason code".to_string());
    }
    Ok(())
}

fn is_transition_allowed(spec: &TurnStateMachine, from: &str, to: &str) -> bool {
    for transition in &spec.allowed_transitions {
        if transition.from == from && transition.to == to {
            return true;
        }
        if transition.from == "*" && transition.to == to {
            if to == spec.failure_state && from != "TURN_COMPLETED" {
                return true;
            }
        }
    }
    false
}
