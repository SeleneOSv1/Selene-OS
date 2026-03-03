#![forbid(unsafe_code)]

use super::apply_regulatory_mode;
use super::compliance_confidence::ComplianceConfidence;
use super::filters::apply_filters;
use super::jurisdiction::{resolve_jurisdiction, JurisdictionConfidence};
use super::trust_tier::{classify_url, TrustTier};
use serde_json::{json, Value};
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/regulatory_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse")
}

#[test]
fn test_t1_jurisdiction_resolver_deterministic_precedence() {
    let request = json!({
        "query": "california compliance update",
        "budgets": {
            "jurisdiction_code": "SG"
        }
    });
    let evidence = json!({
        "sources": [
            {"url": "https://www.sec.gov/rules"}
        ]
    });
    let resolved = resolve_jurisdiction(&request, &evidence).expect("explicit should resolve");
    assert_eq!(resolved.jurisdiction_code, "SG");
    assert_eq!(resolved.confidence, JurisdictionConfidence::High);

    let keyword_request = json!({
        "query": "california compliance update",
        "budgets": {}
    });
    let keyword_resolved =
        resolve_jurisdiction(&keyword_request, &evidence).expect("keyword should resolve");
    assert_eq!(keyword_resolved.jurisdiction_code, "US-CA");
    assert_eq!(keyword_resolved.confidence, JurisdictionConfidence::Medium);

    let source_only_request = json!({
        "query": "compliance update",
        "budgets": {}
    });
    let source_only = resolve_jurisdiction(&source_only_request, &evidence)
        .expect("source URL map should resolve");
    assert_eq!(source_only.jurisdiction_code, "US");
    assert_eq!(source_only.confidence, JurisdictionConfidence::Low);
}

#[test]
fn test_t2_trust_tier_classification_deterministic() {
    assert_eq!(
        classify_url("https://www.sec.gov/rules"),
        TrustTier::Official
    );
    assert_eq!(
        classify_url("https://www.standards.org/framework"),
        TrustTier::High
    );
    assert_eq!(
        classify_url("https://www.reuters.com/world/us"),
        TrustTier::Medium
    );
    assert_eq!(
        classify_url("https://blog.example.com/post"),
        TrustTier::Low
    );
    assert_eq!(classify_url("not-a-url"), TrustTier::Unknown);
}

#[test]
fn test_t3_filtering_removes_low_and_unknown_for_compliance_mode() {
    let fixture = load_fixture("mixed_trust_sources.json");
    let outcome = apply_filters(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect("mixed trust filtering should pass");

    let urls = outcome
        .retained_sources
        .iter()
        .filter_map(|source| source.get("url").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    assert_eq!(
        urls,
        vec!["https://www.sec.gov/news", "https://www.standards.org/controls"]
    );
    assert_eq!(outcome.filtered_source_count, 3);
}

#[test]
fn test_t4_all_filtered_sources_fail_with_insufficient_regulatory_evidence() {
    let fixture = load_fixture("low_confidence.json");
    let mut tool_request = fixture.get("tool_request").cloned().expect("tool_request");
    tool_request["budgets"]["min_trust_tier"] = json!("official");

    let error = apply_filters(
        &tool_request,
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect_err("all filtered sources should fail");
    assert_eq!(error.reason_code(), "insufficient_regulatory_evidence");
}

#[test]
fn test_t5_jurisdiction_mismatch_triggers_fail_closed() {
    let fixture = load_fixture("jurisdiction_mismatch.json");
    let error = apply_filters(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect_err("jurisdiction mismatch must fail");
    assert_eq!(error.reason_code(), "jurisdiction_mismatch");
}

#[test]
fn test_t6_low_compliance_confidence_triggers_fail_closed() {
    let fixture = load_fixture("low_confidence.json");
    let error = apply_filters(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect_err("low confidence should fail");
    assert_eq!(error.reason_code(), "compliance_confidence_low");
}

#[test]
fn test_t7_filtered_ordering_preserves_original_sequence() {
    let fixture = load_fixture("official_sources.json");
    let result = apply_regulatory_mode(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect("official fixture should pass");
    let urls = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources")
        .iter()
        .filter_map(|source| source.get("url").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    assert_eq!(
        urls,
        vec!["https://www.mas.gov.sg/regulation", "https://www.gov.sg/policy"]
    );
}

#[test]
fn test_t8_provenance_fields_deterministic() {
    let fixture = load_fixture("official_sources.json");
    let first = apply_regulatory_mode(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect("first pass should succeed");
    let second = apply_regulatory_mode(
        fixture.get("tool_request").expect("tool_request"),
        fixture.get("evidence_packet").expect("evidence_packet"),
    )
    .expect("second pass should succeed");
    assert_eq!(first, second);

    let trust_metadata = first
        .evidence_packet
        .pointer("/trust_metadata/regulatory")
        .expect("regulatory trust_metadata must exist");
    assert_eq!(
        trust_metadata
            .get("jurisdiction_code")
            .and_then(Value::as_str)
            .expect("jurisdiction_code"),
        "SG"
    );
    assert_eq!(
        trust_metadata
            .get("compliance_confidence")
            .and_then(Value::as_str)
            .expect("compliance_confidence"),
        ComplianceConfidence::High.as_str()
    );
    assert_eq!(
        trust_metadata
            .get("filtered_source_count")
            .and_then(Value::as_u64)
            .expect("filtered_source_count"),
        0
    );
}
