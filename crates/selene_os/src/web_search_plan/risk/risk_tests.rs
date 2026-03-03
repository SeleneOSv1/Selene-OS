#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::risk::factors::{extract_factor_scores, FactorId};
use crate::web_search_plan::risk::guardrails::enforce_non_advice_guardrails;
use crate::web_search_plan::risk::risk_packet::{build_risk_packet, RiskRequest};
use crate::web_search_plan::risk::scoring::compute_composite_risk;
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct RiskFixture {
    request: RiskFixtureRequest,
}

#[derive(Debug, Clone, Deserialize)]
struct RiskFixtureRequest {
    trace_id: String,
    created_at_ms: i64,
    intended_consumers: Vec<String>,
    evidence_packet: Value,
    #[serde(default)]
    computation_packet: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedRiskPacketFixture {
    trace_id: String,
    risk_level: String,
    risk_score: String,
    confidence_score: String,
    risk_model_version: String,
    required_factor_ids: Vec<String>,
    required_disclaimer: String,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/risk_fixtures")
}

fn load_fixture<T: for<'de> Deserialize<'de>>(name: &str) -> T {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str::<T>(&text).expect("fixture should parse")
}

fn to_request(entry: RiskFixtureRequest) -> RiskRequest {
    RiskRequest {
        trace_id: entry.trace_id,
        created_at_ms: entry.created_at_ms,
        intended_consumers: entry.intended_consumers,
        evidence_packet: entry.evidence_packet,
        computation_packet: entry.computation_packet,
    }
}

#[test]
fn test_t1_deterministic_factor_extraction_and_scoring() {
    let fixture: RiskFixture = load_fixture("sufficient_evidence.json");
    let first = extract_factor_scores(
        &fixture.request.evidence_packet,
        fixture.request.computation_packet.as_ref(),
    )
    .expect("factor extraction should pass");
    let second = extract_factor_scores(
        &fixture.request.evidence_packet,
        fixture.request.computation_packet.as_ref(),
    )
    .expect("factor extraction should pass");
    assert_eq!(first, second, "factor extraction must be deterministic");

    let composite_first = compute_composite_risk(&first.factors).expect("composite score should pass");
    let composite_second =
        compute_composite_risk(&second.factors).expect("composite score should pass");
    assert_eq!(composite_first, composite_second, "composite scoring must be deterministic");

    let factor_ids = first
        .factors
        .iter()
        .map(|factor| factor.factor_id.as_str().to_string())
        .collect::<Vec<String>>();
    assert_eq!(
        factor_ids,
        vec![
            FactorId::FinancialStress.as_str().to_string(),
            FactorId::LegalEvents.as_str().to_string(),
            FactorId::RegulatoryEvents.as_str().to_string(),
            FactorId::NegativeNewsCluster.as_str().to_string(),
            FactorId::OperationalReliability.as_str().to_string()
        ]
    );
}

#[test]
fn test_t2_composite_score_deterministic() {
    let fixture: RiskFixture = load_fixture("sufficient_evidence.json");
    let request_a = to_request(fixture.request.clone());
    let request_b = to_request(fixture.request);
    let packet_a = build_risk_packet(&request_a).expect("risk packet should build");
    let packet_b = build_risk_packet(&request_b).expect("risk packet should build");
    assert_eq!(packet_a, packet_b, "same inputs must produce identical risk packet");

    let expected: ExpectedRiskPacketFixture = load_fixture("expected_risk_packet.json");
    assert_eq!(packet_a.trace_id, expected.trace_id);
    assert_eq!(packet_a.risk_level, expected.risk_level);
    assert_eq!(packet_a.risk_score, expected.risk_score);
    assert_eq!(packet_a.confidence_score, expected.confidence_score);
    assert_eq!(packet_a.risk_model_version, expected.risk_model_version);
    assert_eq!(packet_a.disclaimer, expected.required_disclaimer);

    let actual_factor_ids = packet_a
        .factor_breakdown
        .iter()
        .map(|factor| factor.factor_id.clone())
        .collect::<Vec<String>>();
    assert_eq!(actual_factor_ids, expected.required_factor_ids);
}

#[test]
fn test_t3_missing_evidence_fails_closed_with_insufficient_evidence() {
    let fixture: RiskFixture = load_fixture("missing_evidence.json");
    let request = to_request(fixture.request);
    let err = build_risk_packet(&request).expect_err("missing evidence should fail closed");
    assert_eq!(err.reason_code, "insufficient_evidence");
}

#[test]
fn test_t4_conflict_penalty_deterministic_and_disclosed() {
    let sufficient: RiskFixture = load_fixture("sufficient_evidence.json");
    let conflict: RiskFixture = load_fixture("conflicting_evidence.json");
    let sufficient_packet =
        build_risk_packet(&to_request(sufficient.request)).expect("sufficient packet");
    let conflict_packet = build_risk_packet(&to_request(conflict.request)).expect("conflict packet");

    let sufficient_confidence = sufficient_packet
        .confidence_score
        .parse::<rust_decimal::Decimal>()
        .expect("confidence decimal");
    let conflict_confidence = conflict_packet
        .confidence_score
        .parse::<rust_decimal::Decimal>()
        .expect("confidence decimal");
    assert!(
        conflict_confidence < sufficient_confidence,
        "conflict should reduce confidence deterministically"
    );
    assert!(conflict_packet
        .reason_codes
        .iter()
        .any(|code| code == "conflicting_evidence_detected"));
}

#[test]
fn test_t5_guardrails_block_recommendation_language() {
    let err = enforce_non_advice_guardrails(&[
        "Users should buy this stock immediately".to_string()
    ])
    .expect_err("recommendation language must be blocked");
    assert_eq!(err.reason_code, "policy_violation");
}

#[test]
fn test_t6_risk_packet_validates_against_schema() {
    let fixture: RiskFixture = load_fixture("sufficient_evidence.json");
    let packet = build_risk_packet(&to_request(fixture.request)).expect("risk packet should build");
    let registry = load_packet_schema_registry().expect("packet registry should load");
    validate_packet_schema_registry(&registry).expect("packet registry should validate");
    let value = serde_json::to_value(packet).expect("risk packet serialization");
    validate_packet("RiskPacket", &value, &registry).expect("risk packet should validate");
}

#[test]
fn test_t7_all_evidence_refs_exist_in_evidence_packet() {
    let fixture: RiskFixture = load_fixture("sufficient_evidence.json");
    let evidence_packet = fixture.request.evidence_packet.clone();
    let packet = build_risk_packet(&to_request(fixture.request)).expect("risk packet should build");

    let mut known_refs = BTreeSet::new();
    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                known_refs.insert(url.to_string());
            }
        }
    }
    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                known_refs.insert(chunk_id.to_string());
            }
            if let Some(source_url) = chunk.get("source_url").and_then(Value::as_str) {
                known_refs.insert(source_url.to_string());
            }
        }
    }

    for evidence_ref in packet.evidence_refs {
        assert!(
            known_refs.contains(&evidence_ref),
            "unknown evidence_ref {}",
            evidence_ref
        );
    }
}

#[test]
fn test_low_confidence_fixture_sets_confidence_low_flag() {
    let fixture: RiskFixture = load_fixture("low_confidence.json");
    let packet = build_risk_packet(&to_request(fixture.request)).expect("risk packet should build");
    assert!(
        packet
            .uncertainty_flags
            .iter()
            .any(|flag| flag == "confidence_low")
    );
}
