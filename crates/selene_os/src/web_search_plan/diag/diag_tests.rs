#![forbid(unsafe_code)]

use crate::web_search_plan::diag::debug_packet::{
    try_build_debug_packet, DebugPacketContext, DebugStatus, HealthStatusBeforeFallback,
};
use crate::web_search_plan::diag::error_taxonomy::{
    ensure_reason_code_registered, map_internal_failure,
};
use crate::web_search_plan::diag::redaction::{redact_proxy, redact_url};
use crate::web_search_plan::diag::state_trace::{
    default_degraded_transitions, default_failed_transitions, MonotonicClock, StateTraceRecorder,
};
use serde_json::Value;
use std::sync::atomic::{AtomicI64, Ordering};

#[derive(Debug, Default)]
struct FixedClock {
    now_ms: AtomicI64,
}

impl FixedClock {
    fn with_start(ms: i64) -> Self {
        Self {
            now_ms: AtomicI64::new(ms),
        }
    }
}

impl MonotonicClock for FixedClock {
    fn now_ms(&self) -> i64 {
        self.now_ms.fetch_add(1, Ordering::Relaxed)
    }
}

#[test]
fn test_debug_packet_serialization_is_deterministic_and_complete() {
    let transitions = default_failed_transitions(1_700_000_000_000);

    let packet = try_build_debug_packet(DebugPacketContext {
        trace_id: "trace-debug-1",
        status: DebugStatus::Failed,
        provider: "UrlFetch",
        error_kind: "timeout_exceeded",
        reason_code: "timeout_exceeded",
        proxy_mode: Some("env"),
        source_url: Some("https://example.com/path?a=1#frag"),
        created_at_ms: 1_700_000_000_123,
        turn_state_transitions: &transitions,
        debug_hint: Some("request failed token=abc123 at /tmp/secret.log"),
        fallback_used: Some(false),
        health_status_before_fallback: Some(HealthStatusBeforeFallback::Healthy),
    })
    .expect("debug packet must build");

    let serialized_once = serde_json::to_string(&packet).expect("serialize once");
    let serialized_twice = serde_json::to_string(&packet).expect("serialize twice");
    assert_eq!(serialized_once, serialized_twice);

    let value: Value = serde_json::from_str(&serialized_once).expect("valid json");
    for field in [
        "trace_id",
        "status",
        "provider",
        "error_kind",
        "reason_code",
        "created_at_ms",
        "turn_state_transitions",
    ] {
        assert!(value.get(field).is_some(), "missing required field {field}");
    }

    assert_eq!(
        packet
            .redacted_url
            .as_deref()
            .expect("redacted url must exist"),
        "https://example.com/path?redacted"
    );
    assert!(
        packet
            .debug_hint
            .as_deref()
            .expect("hint exists")
            .contains("[REDACTED]")
    );
}

#[test]
fn test_redact_url_removes_query_and_fragment() {
    let redacted = redact_url("https://example.com/a/b?token=abc#section").expect("redaction");
    assert_eq!(redacted, "https://example.com/a/b?redacted");
    assert!(!redacted.contains("token="));
    assert!(!redacted.contains('#'));
}

#[test]
fn test_redact_proxy_removes_userinfo() {
    let redacted = redact_proxy("http://user:pass@proxy.example.com:8080").expect("redaction");
    assert_eq!(redacted, "http://proxy.example.com:8080");
    assert!(!redacted.contains("user"));
    assert!(!redacted.contains("pass"));
    assert!(!redacted.contains('@'));
}

#[test]
fn test_debug_hint_tokens_are_redacted() {
    let transitions = default_failed_transitions(50);
    let packet = try_build_debug_packet(DebugPacketContext {
        trace_id: "trace-debug-2",
        status: DebugStatus::Failed,
        provider: "Proxy",
        error_kind: "proxy_misconfigured",
        reason_code: "proxy_misconfigured",
        proxy_mode: Some("explicit"),
        source_url: Some("https://example.com"),
        created_at_ms: 60,
        turn_state_transitions: &transitions,
        debug_hint: Some("Authorization: Bearer sk-secret-value"),
        fallback_used: None,
        health_status_before_fallback: None,
    })
    .expect("packet should build");

    let hint = packet.debug_hint.expect("debug hint exists");
    assert!(hint.contains("[REDACTED]"));
    assert!(!hint.to_ascii_lowercase().contains("bearer"));
    assert!(!hint.contains("sk-secret-value"));
}

#[test]
fn test_state_trace_ordered_and_valid() {
    let mut recorder =
        StateTraceRecorder::new(FixedClock::with_start(1000), "TURN_ACCEPTED").expect("recorder");
    recorder
        .transition("INPUT_PARSED")
        .expect("accepted->input parsed");
    recorder
        .transition("INTENT_CLASSIFIED")
        .expect("input parsed->intent classified");
    recorder
        .transition("TURN_FAILED_CLOSED")
        .expect("intent classified->failed closed");

    let transitions = recorder.into_transitions();
    let packet = try_build_debug_packet(DebugPacketContext {
        trace_id: "trace-debug-3",
        status: DebugStatus::Failed,
        provider: "Synthesis",
        error_kind: "citation_mismatch",
        reason_code: "citation_mismatch",
        proxy_mode: None,
        source_url: None,
        created_at_ms: 1234,
        turn_state_transitions: &transitions,
        debug_hint: Some("citation coverage failed"),
        fallback_used: Some(false),
        health_status_before_fallback: Some(HealthStatusBeforeFallback::Degraded),
    })
    .expect("packet should be valid");

    let times: Vec<i64> = packet
        .turn_state_transitions
        .iter()
        .map(|entry| entry.at_ms)
        .collect();
    assert!(times.windows(2).all(|window| window[0] <= window[1]));
}

#[test]
fn test_taxonomy_maps_to_registered_reason_code() {
    let mapped = map_internal_failure("BraveWebSearch", "http_non_200", None)
        .expect("taxonomy mapping should succeed");
    ensure_reason_code_registered(&mapped.reason_code).expect("reason code must be registered");
    assert_eq!(mapped.provider, "BraveWebSearch");
    assert_eq!(mapped.reason_code, "provider_upstream_failed");
}

#[test]
fn test_identical_inputs_produce_identical_debug_packets() {
    let transitions = default_degraded_transitions(1_700_000_010_000);

    let build = || {
        try_build_debug_packet(DebugPacketContext {
            trace_id: "trace-debug-repeat",
            status: DebugStatus::Degraded,
            provider: "OpenAI_WebSearch",
            error_kind: "empty_results",
            reason_code: "empty_results",
            proxy_mode: Some("off"),
            source_url: Some("https://example.com/search?q=query"),
            created_at_ms: 1_700_000_010_500,
            turn_state_transitions: &transitions,
            debug_hint: Some("provider returned empty payload"),
            fallback_used: Some(true),
            health_status_before_fallback: Some(HealthStatusBeforeFallback::Cooldown),
        })
    };

    let first = build().expect("first packet");
    let second = build().expect("second packet");
    assert_eq!(first, second);
}
