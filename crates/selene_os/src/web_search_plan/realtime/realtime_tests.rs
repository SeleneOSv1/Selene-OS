#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use super::adapters::{self, RealtimeAdapterOutput};
use super::domains::{detect_domain, RealtimeDomain};
use super::freshness::evaluate;
use super::ttl_policy::ttl_ms;
use super::{finalize_realtime_result, parse_tool_request_packet, ParsedRealtimeToolRequest, RealtimeRuntimeConfig};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/realtime_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse")
}

fn fixture_request_and_output(name: &str) -> (ParsedRealtimeToolRequest, RealtimeAdapterOutput) {
    let fixture = load_fixture(name);
    let now_ms = fixture
        .get("now_ms")
        .and_then(Value::as_i64)
        .expect("fixture now_ms");
    let request = parse_tool_request_packet(
        fixture.get("tool_request").expect("fixture tool_request"),
        now_ms,
    )
    .expect("tool request should parse");
    let adapter_output = parse_adapter_output(fixture.get("adapter_output").expect("fixture adapter_output"));
    (request, adapter_output)
}

fn parse_adapter_output(value: &Value) -> RealtimeAdapterOutput {
    RealtimeAdapterOutput {
        provider_id: value
            .get("provider_id")
            .and_then(Value::as_str)
            .expect("provider_id")
            .to_string(),
        endpoint_url: value
            .get("endpoint_url")
            .and_then(Value::as_str)
            .expect("endpoint_url")
            .to_string(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .expect("title")
            .to_string(),
        trust_tier: value
            .get("trust_tier")
            .and_then(Value::as_str)
            .expect("trust_tier")
            .to_string(),
        retrieved_at_ms: value
            .get("retrieved_at_ms")
            .and_then(Value::as_i64)
            .expect("retrieved_at_ms"),
        latency_ms: value
            .get("latency_ms")
            .and_then(Value::as_u64)
            .expect("latency_ms"),
        payload: value.get("payload").cloned().expect("payload"),
    }
}

#[test]
fn test_t1_domain_routing_deterministic() {
    let weather = detect_domain("weather in san francisco", None);
    let weather_again = detect_domain("weather in san francisco", None);
    assert_eq!(weather, RealtimeDomain::Weather);
    assert_eq!(weather, weather_again);

    let finance = detect_domain("stock quote AAPL", None);
    assert_eq!(finance, RealtimeDomain::Finance);

    let flights = detect_domain("flight UA100 status", None);
    assert_eq!(flights, RealtimeDomain::Flights);

    let generic = detect_domain("http://api.example.com/realtime", None);
    assert_eq!(generic, RealtimeDomain::GenericRealTime);
}

#[test]
fn test_t2_ttl_policy_deterministic_by_tier() {
    assert_eq!(ttl_ms(RealtimeDomain::Weather, ImportanceTier::Low), 3_600_000);
    assert_eq!(ttl_ms(RealtimeDomain::Weather, ImportanceTier::Medium), 1_800_000);
    assert_eq!(ttl_ms(RealtimeDomain::Weather, ImportanceTier::High), 900_000);

    assert_eq!(ttl_ms(RealtimeDomain::Finance, ImportanceTier::Low), 1_800_000);
    assert_eq!(ttl_ms(RealtimeDomain::Finance, ImportanceTier::Medium), 900_000);
    assert_eq!(ttl_ms(RealtimeDomain::Finance, ImportanceTier::High), 300_000);
}

#[test]
fn test_t3_freshness_score_deterministic_with_fixed_clock() {
    let first = evaluate(1_700_000_600_000, 1_700_000_300_000, 900_000).expect("freshness first");
    let second = evaluate(1_700_000_600_000, 1_700_000_300_000, 900_000).expect("freshness second");
    assert_eq!(first, second);
    assert!((first.freshness_score - 0.666667).abs() < 0.000001);
}

#[test]
fn test_t4_stale_triggers_stale_data_fail_closed() {
    let (request, output) = fixture_request_and_output("finance_stale.json");
    let error = finalize_realtime_result(&request, output).expect_err("stale fixture must fail");
    assert_eq!(error.reason_code(), "stale_data");
}

#[test]
fn test_t5_provider_unconfigured_path_deterministic() {
    let tool_request = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.X",
        "intended_consumers": ["PH1.E"],
        "created_at_ms": 1707600000000i64,
        "trace_id": "trace-realtime-provider-unconfigured",
        "mode": "structured",
        "query": "https://api.example.com/realtime",
        "importance_tier": "medium",
        "budgets": {
            "domain_hint": "generic_real_time"
        }
    });
    let parsed = parse_tool_request_packet(&tool_request, 1_707_600_000_000).expect("request should parse");

    let mut config = RealtimeRuntimeConfig::default();
    config.generic_api_key_override = Some(" ".to_string());
    config.generic_vault_secret_id_override = Some("unknown_secret_id".to_string());

    let first = adapters::generic_json::execute(&parsed, &config).expect_err("first call must fail");
    let second = adapters::generic_json::execute(&parsed, &config).expect_err("second call must fail");
    assert_eq!(first.reason_code(), "provider_unconfigured");
    assert_eq!(second.reason_code(), "provider_unconfigured");
}

#[test]
fn test_t6_evidence_packet_population_and_schema_validation() {
    let (request, output) = fixture_request_and_output("finance_ok.json");
    let result = finalize_realtime_result(&request, output).expect("finance_ok should pass");

    let ttl = result
        .evidence_packet
        .pointer("/provider_runs/0/ttl_ms_applied")
        .and_then(Value::as_u64)
        .expect("provider_runs ttl should exist");
    assert_eq!(ttl, result.ttl_ms);

    let freshness = result
        .evidence_packet
        .pointer("/trust_metadata/realtime/freshness_score")
        .and_then(Value::as_f64)
        .expect("freshness_score should exist");
    assert!((0.0..=1.0).contains(&freshness));

    let retrieved_at = result
        .evidence_packet
        .get("retrieved_at_ms")
        .and_then(Value::as_i64)
        .expect("retrieved_at_ms should exist");
    assert_eq!(retrieved_at, 1_707_399_760_000);

    let registry = load_packet_schema_registry().expect("packet registry should load");
    validate_packet_schema_registry(&registry).expect("packet schema registry should validate");
    validate_packet("EvidencePacket", &result.evidence_packet, &registry)
        .expect("realtime evidence packet must validate");
}

#[test]
fn test_t7_missing_retrieved_at_fails_without_inference() {
    let payload = json!({
        "trust_tier": "high",
        "price": 123.45
    });
    let err = adapters::parse_retrieved_at_ms("RealtimeFinance", &payload)
        .expect_err("missing retrieved_at_ms must fail");
    assert_eq!(err.reason_code(), "policy_violation");
}
