#![forbid(unsafe_code)]

use crate::web_search_plan::competitive::run_competitive_mode;
use crate::web_search_plan::competitive::schema::{BillingPeriod, CompetitiveRequest, TriState};
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::structured::types::StructuredRow;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/web_search_plan/competitive_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse")
}

fn request_from_fixture(name: &str) -> CompetitiveRequest {
    let fixture = load_fixture(name);
    let structured_rows: Vec<StructuredRow> = serde_json::from_value(
        fixture
            .get("structured_rows")
            .cloned()
            .expect("fixture structured_rows"),
    )
    .expect("structured rows must deserialize");

    let computation_packet = fixture
        .get("computation_packet")
        .cloned()
        .map(|value| {
            serde_json::from_value(value).expect("computation packet fixture must deserialize")
        });

    CompetitiveRequest {
        trace_id: fixture
            .get("trace_id")
            .and_then(Value::as_str)
            .expect("trace_id")
            .to_string(),
        created_at_ms: fixture
            .get("created_at_ms")
            .and_then(Value::as_i64)
            .expect("created_at_ms"),
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        target_entity: fixture
            .get("target_entity")
            .and_then(Value::as_str)
            .expect("target_entity")
            .to_string(),
        evidence_packet: fixture
            .get("evidence_packet")
            .cloned()
            .expect("evidence_packet"),
        structured_rows,
        computation_packet,
    }
}

#[test]
fn test_t1_entity_dedup_and_ordering_deterministic() {
    let request = request_from_fixture("pricing_competitors.json");
    let first = run_competitive_mode(request.clone()).expect("first competitive run should pass");
    let second = run_competitive_mode(request).expect("second competitive run should pass");
    assert_eq!(first, second);

    let names = first
        .competitors
        .iter()
        .map(|entity| entity.name.clone())
        .collect::<Vec<String>>();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
}

#[test]
fn test_t2_pricing_parsing_and_period_normalization_deterministic() {
    let request = request_from_fixture("pricing_competitors.json");
    let output = run_competitive_mode(request.clone()).expect("competitive pricing should pass");
    let replay = run_competitive_mode(request).expect("replay should pass");
    assert_eq!(output.pricing_table, replay.pricing_table);

    let yearly = output
        .pricing_table
        .iter()
        .find(|entry| entry.billing_period == BillingPeriod::Yearly)
        .expect("yearly entry should exist in fixture");
    assert_eq!(yearly.normalized_to.as_deref(), Some("100"));
}

#[test]
fn test_t3_mixed_currency_kept_separate_without_fx() {
    let request = request_from_fixture("mixed_currency.json");
    let output = run_competitive_mode(request).expect("mixed currency fixture should pass");

    let currencies = output
        .pricing_table
        .iter()
        .map(|entry| entry.currency.clone())
        .collect::<BTreeSet<String>>();
    assert!(currencies.contains("USD"));
    assert!(currencies.contains("EUR"));
    assert!(output
        .uncertainty_flags
        .iter()
        .any(|flag| flag == "non_comparable_currency"));
}

#[test]
fn test_t4_feature_dedup_and_matrix_ordering_deterministic() {
    let request = request_from_fixture("features_competitors.json");
    let output = run_competitive_mode(request.clone()).expect("feature fixture should pass");
    let replay = run_competitive_mode(request).expect("replay should pass");
    assert_eq!(output.feature_matrix, replay.feature_matrix);

    let keys = output
        .feature_matrix
        .iter()
        .map(|feature| feature.feature_key.clone())
        .collect::<Vec<String>>();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted);
}

#[test]
fn test_t5_position_summary_requires_computation_and_sample() {
    let with_computation = run_competitive_mode(request_from_fixture("pricing_competitors.json"))
        .expect("pricing fixture should pass");
    assert!(with_computation.position_summary.is_some());

    let insufficient = run_competitive_mode(request_from_fixture("insufficient_sample.json"))
        .expect("insufficient fixture should still return packet");
    assert!(insufficient.position_summary.is_none());
    assert!(insufficient
        .reason_codes
        .iter()
        .any(|code| code == "insufficient_evidence"));
}

#[test]
fn test_t6_citation_completeness_enforced() {
    let mut request = request_from_fixture("features_competitors.json");
    request.structured_rows[0].source_ref = "https://missing.ref/not-in-evidence".to_string();
    let error = run_competitive_mode(request).expect_err("citation mismatch must fail");
    assert_eq!(error.reason_code(), "citation_mismatch");
}

#[test]
fn test_t7_unknown_handling_is_deterministic() {
    let request = request_from_fixture("features_competitors.json");
    let first = run_competitive_mode(request.clone()).expect("first run should pass");
    let second = run_competitive_mode(request).expect("second run should pass");
    assert_eq!(first, second);

    assert!(first
        .feature_matrix
        .iter()
        .any(|entry| entry.presence == TriState::Unknown));
}

#[test]
fn test_snapshot_hashes_match_expected_fixture() {
    let fixtures = [
        ("pricing_competitors", "pricing_competitors.json"),
        ("features_competitors", "features_competitors.json"),
        ("mixed_currency", "mixed_currency.json"),
        ("insufficient_sample", "insufficient_sample.json"),
    ];
    let expected = load_fixture("expected_comparison.json")
        .as_object()
        .cloned()
        .expect("expected comparison fixture must be object");

    let mut actual = BTreeMap::new();
    for (key, fixture_name) in fixtures {
        let packet =
            run_competitive_mode(request_from_fixture(fixture_name)).expect("fixture packet");
        let packet_json = serde_json::to_value(packet).expect("packet should serialize");
        let hash = hash_canonical_json(&packet_json).expect("hash should compute");
        actual.insert(key.to_string(), hash);
    }

    for (key, hash) in actual {
        let expected_hash = expected
            .get(&key)
            .and_then(Value::as_str)
            .expect("expected hash must exist");
        assert_eq!(hash, expected_hash, "hash mismatch for {}", key);
    }
}
