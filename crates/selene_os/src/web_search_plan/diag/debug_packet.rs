#![forbid(unsafe_code)]

use crate::web_search_plan::diag::error_taxonomy::map_internal_failure;
use crate::web_search_plan::diag::redaction::{redact_url, sanitize_debug_hint};
use crate::web_search_plan::diag::state_trace::{validate_turn_state_transitions, TurnStateTransition};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugStatus {
    Failed,
    Degraded,
    Ok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatusBeforeFallback {
    Healthy,
    Degraded,
    Cooldown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DebugPacket {
    pub trace_id: String,
    pub status: DebugStatus,
    pub provider: String,
    pub error_kind: String,
    pub reason_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redacted_url: Option<String>,
    pub created_at_ms: i64,
    pub turn_state_transitions: Vec<TurnStateTransition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debug_hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_used: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_status_before_fallback: Option<HealthStatusBeforeFallback>,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugPacketContext<'a> {
    pub trace_id: &'a str,
    pub status: DebugStatus,
    pub provider: &'a str,
    pub error_kind: &'a str,
    pub reason_code: &'a str,
    pub proxy_mode: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub created_at_ms: i64,
    pub turn_state_transitions: &'a [TurnStateTransition],
    pub debug_hint: Option<&'a str>,
    pub fallback_used: Option<bool>,
    pub health_status_before_fallback: Option<HealthStatusBeforeFallback>,
}

pub fn try_build_debug_packet(context: DebugPacketContext<'_>) -> Result<DebugPacket, String> {
    let trace_id = context.trace_id.trim();
    if trace_id.is_empty() {
        return Err("debug packet trace_id must not be empty".to_string());
    }

    let taxonomy = map_internal_failure(
        context.provider,
        context.error_kind,
        Some(context.reason_code),
    )?;

    let reason_codes = vec![taxonomy.reason_code.clone()];
    validate_turn_state_transitions(context.turn_state_transitions, &reason_codes)?;

    if matches!(context.status, DebugStatus::Failed) {
        let ended_in_failure = context
            .turn_state_transitions
            .last()
            .map(|entry| entry.to.as_str() == "TURN_FAILED_CLOSED")
            .unwrap_or(false);
        if !ended_in_failure {
            return Err(
                "failed debug packets must end in TURN_FAILED_CLOSED transition".to_string(),
            );
        }
    }

    let proxy_mode = match context.proxy_mode {
        Some(raw) => {
            let value = raw.trim().to_ascii_lowercase();
            match value.as_str() {
                "off" | "env" | "explicit" => Some(value),
                _ => return Err(format!("invalid proxy_mode {}", raw)),
            }
        }
        None => None,
    };

    let redacted_url = match context.source_url {
        Some(url) => Some(redact_url(url)?),
        None => None,
    };

    let debug_hint = match context.debug_hint {
        Some(hint) => {
            let sanitized = sanitize_debug_hint(hint);
            if sanitized.trim().is_empty() {
                None
            } else {
                if leaks_sensitive_material(&sanitized) {
                    return Err("debug_hint still contains sensitive material after redaction".to_string());
                }
                Some(sanitized)
            }
        }
        None => None,
    };

    Ok(DebugPacket {
        trace_id: trace_id.to_string(),
        status: context.status,
        provider: taxonomy.provider,
        error_kind: taxonomy.error_kind,
        reason_code: taxonomy.reason_code,
        proxy_mode,
        redacted_url,
        created_at_ms: context.created_at_ms,
        turn_state_transitions: context.turn_state_transitions.to_vec(),
        debug_hint,
        fallback_used: context.fallback_used,
        health_status_before_fallback: context.health_status_before_fallback,
    })
}

fn leaks_sensitive_material(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower.contains("token=")
        || lower.contains("bearer ")
        || lower.contains("authorization")
        || lower.contains("cookie=")
        || lower.contains("api_key")
        || lower.contains("sk-")
        || value.contains("@")
}
