#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::packet_builder::build_computation_packet;
use crate::web_search_plan::analytics::types::{
    AggregateMethod, AnalyticsRequest, ComputationPacket, NumericValue,
};
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};
use rust_decimal::Decimal;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/analytics_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture json should parse")
}

fn request_from_fixture(name: &str) -> AnalyticsRequest {
    let fixture = load_fixture(name);
    let structured_rows: Vec<StructuredRow> = serde_json::from_value(
        fixture
            .get("structured_rows")
            .cloned()
            .expect("fixture structured_rows"),
    )
    .expect("structured rows should deserialize");

    AnalyticsRequest {
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
        policy_snapshot_id: fixture
            .get("policy_snapshot_id")
            .and_then(Value::as_str)
            .expect("policy_snapshot_id")
            .to_string(),
        as_of_ms: fixture.get("as_of_ms").and_then(Value::as_i64),
        evidence_packet: fixture
            .get("evidence_packet")
            .cloned()
            .expect("evidence_packet"),
        structured_rows,
    }
}

#[test]
fn test_t1_same_inputs_identical_computation_packet() {
    let request = request_from_fixture("salary_region_sample.json");
    let first = build_computation_packet(request.clone()).expect("first run should pass");
    let second = build_computation_packet(request).expect("second run should pass");
    assert_eq!(first, second);
}

#[test]
fn test_t2_aggregates_mean_median_quartiles_correct() {
    let request = request_from_fixture("tax_brackets_sample.json");
    let packet = build_computation_packet(request).expect("analytics should succeed");

    assert_eq!(
        aggregate_value(&packet, AggregateMethod::Mean),
        Some("20".to_string())
    );
    assert_eq!(
        aggregate_value(&packet, AggregateMethod::Median),
        Some("20".to_string())
    );
    assert_eq!(
        aggregate_value(&packet, AggregateMethod::P25),
        Some("15".to_string())
    );
    assert_eq!(
        aggregate_value(&packet, AggregateMethod::P50),
        Some("20".to_string())
    );
    assert_eq!(
        aggregate_value(&packet, AggregateMethod::P75),
        Some("25".to_string())
    );
}

#[test]
fn test_t3_outlier_detection_deterministic() {
    let evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.E",
        "intended_consumers": ["PH1.ANALYTICS"],
        "created_at_ms": 1705000000000i64,
        "trace_id": "trace-outlier",
        "query": "outlier",
        "retrieved_at_ms": 1705000000000i64,
        "provider_runs": [],
        "sources": [
            {"url": "https://example.com/o1", "trust_tier": "high"},
            {"url": "https://example.com/o2", "trust_tier": "medium"},
            {"url": "https://example.com/o3", "trust_tier": "medium"},
            {"url": "https://example.com/o4", "trust_tier": "low"}
        ],
        "content_chunks": [],
        "trust_metadata": {"analytics": {"unit_conversions": [], "currency_rates": []}}
    });

    let rows = vec![
        structured_row("Metric", "Value", 100.0, "https://example.com/o1"),
        structured_row("Metric", "Value", 105.0, "https://example.com/o2"),
        structured_row("Metric", "Value", 110.0, "https://example.com/o3"),
        structured_row("Metric", "Value", 300.0, "https://example.com/o4"),
    ];

    let request = AnalyticsRequest {
        trace_id: "trace-outlier".to_string(),
        created_at_ms: 1705000000000,
        intended_consumers: vec!["PH1.D".to_string()],
        policy_snapshot_id: "policy_run21_v1".to_string(),
        as_of_ms: Some(1705000000000),
        evidence_packet,
        structured_rows: rows,
    };

    let first = build_computation_packet(request.clone()).expect("first outlier run should pass");
    let second = build_computation_packet(request).expect("second outlier run should pass");
    assert_eq!(first.consensus, second.consensus);

    let outlier_values = first
        .consensus
        .iter()
        .flat_map(|group| group.outliers.iter())
        .map(|outlier| numeric_value_to_string(&outlier.value))
        .collect::<Vec<String>>();

    assert!(outlier_values.iter().any(|value| value == "300"));
}

#[test]
fn test_t4_consensus_choose_or_conflict_deterministic() {
    let choose_packet = build_computation_packet(request_from_fixture("salary_region_sample.json"))
        .expect("salary packet");
    let chosen_values = choose_packet
        .consensus
        .iter()
        .filter_map(|group| group.chosen.as_ref())
        .map(numeric_value_to_string)
        .collect::<Vec<String>>();
    assert!(chosen_values.iter().any(|value| value == "100000"));

    let conflict_packet =
        build_computation_packet(request_from_fixture("conflicting_values_sample.json"))
            .expect("conflicting packet");
    assert!(conflict_packet
        .consensus
        .iter()
        .all(|group| group.chosen.is_none()));
    assert!(conflict_packet
        .reason_codes
        .iter()
        .any(|code| code == "conflicting_evidence_detected"));
}

#[test]
fn test_t5_confidence_scoring_deterministic() {
    let request = request_from_fixture("tax_brackets_sample.json");
    let first = build_computation_packet(request.clone()).expect("first run should pass");
    let second = build_computation_packet(request).expect("second run should pass");

    assert_eq!(first.confidence, second.confidence);
    assert!(!first.confidence.is_empty());
    for item in &first.confidence {
        let parsed = item
            .confidence_score
            .parse::<Decimal>()
            .expect("confidence score must parse");
        assert!(parsed >= Decimal::ZERO && parsed <= Decimal::ONE);
    }
}

#[test]
fn test_t6_unit_currency_normalization_only_when_factors_present() {
    let mut with_rates = request_from_fixture("pricing_competitors_sample.json");
    let with_rates_packet =
        build_computation_packet(with_rates.clone()).expect("with rates should pass");

    let currencies = with_rates_packet
        .aggregates
        .iter()
        .map(|aggregate| {
            aggregate
                .currency
                .clone()
                .unwrap_or_else(|| "none".to_string())
        })
        .collect::<std::collections::BTreeSet<String>>();
    assert_eq!(
        currencies,
        std::collections::BTreeSet::from(["USD".to_string()])
    );

    with_rates.evidence_packet["trust_metadata"]["analytics"]["currency_rates"] = json!([]);
    let without_rates_packet =
        build_computation_packet(with_rates).expect("without rates should still pass");

    let without_currencies = without_rates_packet
        .aggregates
        .iter()
        .map(|aggregate| {
            aggregate
                .currency
                .clone()
                .unwrap_or_else(|| "none".to_string())
        })
        .collect::<std::collections::BTreeSet<String>>();

    assert!(without_currencies.contains("USD"));
    assert!(without_currencies.contains("EUR"));
    assert!(without_rates_packet
        .reason_codes
        .iter()
        .any(|code| code == "policy_violation"));
}

#[test]
fn test_t7_source_refs_must_exist_in_evidence_packet() {
    let mut request = request_from_fixture("tax_brackets_sample.json");
    request.structured_rows[0].source_ref = "chunk_missing_ref".to_string();

    let error = build_computation_packet(request).expect_err("missing source_ref must fail");
    assert_eq!(error.reason_code(), "citation_mismatch");
}

#[test]
fn test_hash_snapshots_match_expected_file() {
    let fixtures = [
        ("salary_region_sample", "salary_region_sample.json"),
        ("tax_brackets_sample", "tax_brackets_sample.json"),
        (
            "pricing_competitors_sample",
            "pricing_competitors_sample.json",
        ),
        (
            "conflicting_values_sample",
            "conflicting_values_sample.json",
        ),
    ];
    let expected = load_fixture("expected_computation_packet_hashes.json")
        .as_object()
        .cloned()
        .expect("expected hashes object");

    let mut actual = BTreeMap::new();
    for (key, file_name) in fixtures {
        let packet =
            build_computation_packet(request_from_fixture(file_name)).expect("fixture packet");
        let packet_json = serde_json::to_value(packet).expect("serialize packet");
        let hash = hash_canonical_json(&packet_json).expect("hash packet json");
        actual.insert(key.to_string(), hash);
    }

    for (key, value) in actual {
        let expected_hash = expected
            .get(&key)
            .and_then(Value::as_str)
            .expect("expected hash key must exist");
        assert_eq!(value, expected_hash, "snapshot hash mismatch for {}", key);
    }
}

fn structured_row(entity: &str, attribute: &str, value: f64, source: &str) -> StructuredRow {
    StructuredRow {
        entity: entity.to_string(),
        attribute: attribute.to_string(),
        value: StructuredValue::Float { value },
        unit: Some("points".to_string()),
        as_of_ms: Some(1705000000000),
        source_url: source.to_string(),
        source_ref: source.to_string(),
        confidence: Some(0.9),
        schema_version: "1.0.0".to_string(),
    }
}

fn aggregate_value(packet: &ComputationPacket, method: AggregateMethod) -> Option<String> {
    packet
        .aggregates
        .iter()
        .find(|aggregate| aggregate.method == method)
        .map(|aggregate| numeric_value_to_string(&aggregate.value))
}

fn numeric_value_to_string(value: &NumericValue) -> String {
    match value {
        NumericValue::Int { value } => value.to_string(),
        NumericValue::Decimal { value } => value.clone(),
    }
}
