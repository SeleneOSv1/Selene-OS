#![forbid(unsafe_code)]

use crate::web_search_plan::gap_closers::claim_confidence::{
    calibrate_claim_confidence, ClaimConfidenceItem,
};
use crate::web_search_plan::gap_closers::freshness_watchdog::evaluate_freshness_watchdog;
use crate::web_search_plan::gap_closers::injection_defense::sanitize_fetched_content;
use crate::web_search_plan::gap_closers::table_render::{
    render_competitive_pricing_table, render_risk_factor_table, render_temporal_changes_table,
};
use crate::web_search_plan::gap_closers::transparency::build_trace_report;
use crate::web_search_plan::gap_closers::unknown_first::{
    evaluate_unknown_first, evaluate_unknown_first_pre_synthesis, UnknownFirstSignals,
};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/gap_closers_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let path = fixture_dir().join(name);
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {}", path.display(), error));
    serde_json::from_str(&content)
        .unwrap_or_else(|error| panic!("failed to parse fixture {}: {}", path.display(), error))
}

#[test]
fn test_t1_injection_defense_removes_or_flags_malicious_segments_deterministically() {
    let fixture = load_fixture("injection_content_sample.json");
    let input_text = fixture
        .get("input_text")
        .and_then(Value::as_str)
        .expect("input_text must exist");

    let first = sanitize_fetched_content(input_text);
    let second = sanitize_fetched_content(input_text);
    assert_eq!(first, second);

    let expected_patterns = fixture
        .get("expected_patterns")
        .and_then(Value::as_array)
        .expect("expected_patterns must be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let actual_patterns = first
        .flagged_segments
        .iter()
        .map(|segment| segment.matched_pattern.clone())
        .collect::<Vec<String>>();
    assert_eq!(actual_patterns, expected_patterns);
    assert!(!first.sanitized_text.to_ascii_lowercase().contains("ignore previous instructions"));
}

#[test]
fn test_t2_unknown_first_triggers_on_low_conflict_or_missing_citations() {
    let fixture = load_fixture("low_confidence_sample.json");
    let claim_confidences = serde_json::from_value::<Vec<ClaimConfidenceItem>>(
        fixture
            .get("claim_confidences")
            .cloned()
            .expect("claim_confidences must exist"),
    )
    .expect("claim_confidences fixture should decode");
    let explicit_reason_codes = fixture
        .get("explicit_reason_codes")
        .and_then(Value::as_array)
        .expect("explicit_reason_codes must be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let decision = evaluate_unknown_first(&UnknownFirstSignals {
        claim_confidences,
        has_unresolved_conflict: fixture
            .get("has_unresolved_conflict")
            .and_then(Value::as_bool)
            .expect("has_unresolved_conflict must exist"),
        citation_coverage_ratio: Decimal::from_str_exact(
            fixture
                .get("citation_coverage_ratio")
                .and_then(Value::as_str)
                .expect("citation_coverage_ratio must exist"),
        )
        .expect("citation_coverage_ratio should parse"),
        explicit_reason_codes,
    });
    assert!(decision.unknown_required);
    assert_eq!(decision.reason_code.as_deref(), Some("insufficient_evidence"));
    assert!(!decision.causes.is_empty());
}

#[test]
fn test_t3_per_claim_confidence_is_deterministic_and_stable() {
    let fixture = load_fixture("trace_mode_sample.json");
    let synthesis = fixture.get("synthesis_packet").expect("synthesis_packet");
    let evidence = fixture.get("evidence_packet").expect("evidence_packet");
    let computation = fixture.get("computation_packet");
    let report = build_trace_report(synthesis, evidence, computation, None, None, None)
        .expect("trace report should build");

    let first = calibrate_claim_confidence(&report, evidence, synthesis, computation);
    let second = calibrate_claim_confidence(&report, evidence, synthesis, computation);
    assert_eq!(first, second);
    assert!(
        first.iter().all(|item| {
            Decimal::from_str_exact(item.confidence_score.as_str())
                .map(|value| value >= Decimal::ZERO && value <= Decimal::ONE)
                .unwrap_or(false)
        }),
        "confidence_score values must stay within [0,1]"
    );
}

#[test]
fn test_t4_trace_mode_output_is_deterministic_and_uses_existing_refs_only() {
    let fixture = load_fixture("trace_mode_sample.json");
    let synthesis = fixture.get("synthesis_packet").expect("synthesis_packet");
    let evidence = fixture.get("evidence_packet").expect("evidence_packet");
    let first = build_trace_report(synthesis, evidence, None, None, None, None)
        .expect("trace report should build");
    let second = build_trace_report(synthesis, evidence, None, None, None, None)
        .expect("trace report should build deterministically");
    assert_eq!(first, second);

    let known_refs = collect_known_refs(evidence);
    for claim in &first.claims {
        for citation in &claim.citations {
            assert!(
                known_refs.contains(citation),
                "trace report citation must exist in EvidencePacket refs: {}",
                citation
            );
        }
    }
}

#[test]
fn test_t5_freshness_watchdog_flags_stale_citations_deterministically() {
    let fixture = load_fixture("stale_citations_sample.json");
    let query = fixture
        .get("query")
        .and_then(Value::as_str)
        .expect("query should exist");
    let tier = fixture
        .get("importance_tier")
        .and_then(Value::as_str)
        .expect("importance_tier should exist");
    let retrieved_at_ms = fixture
        .get("retrieved_at_ms")
        .and_then(Value::as_i64)
        .expect("retrieved_at_ms should exist");
    let sources = fixture
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources should be array")
        .clone();

    let first = evaluate_freshness_watchdog(query, tier, retrieved_at_ms, sources.as_slice());
    let second = evaluate_freshness_watchdog(query, tier, retrieved_at_ms, sources.as_slice());
    assert_eq!(first, second);

    let expected = fixture
        .get("expected_stale_refs")
        .and_then(Value::as_array)
        .expect("expected_stale_refs should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let actual = first
        .stale_citations
        .iter()
        .map(|entry| entry.citation_ref.clone())
        .collect::<Vec<String>>();
    assert_eq!(actual, expected);
    assert_eq!(first.refresh_required, !expected.is_empty());
}

#[test]
fn test_t6_table_rendering_is_stable_with_deterministic_ordering() {
    let fixture = load_fixture("table_render_sample.json");
    let comparison_packet = fixture
        .get("comparison_packet")
        .expect("comparison_packet must exist");
    let temporal_packet = fixture
        .get("temporal_packet")
        .expect("temporal_packet must exist");
    let risk_packet = fixture.get("risk_packet").expect("risk_packet must exist");

    let cmp_first = render_competitive_pricing_table(comparison_packet).expect("comparison table");
    let cmp_second =
        render_competitive_pricing_table(comparison_packet).expect("comparison table repeat");
    assert_eq!(cmp_first, cmp_second);

    let tmp_first = render_temporal_changes_table(temporal_packet).expect("temporal table");
    let tmp_second = render_temporal_changes_table(temporal_packet).expect("temporal table repeat");
    assert_eq!(tmp_first, tmp_second);

    let risk_first = render_risk_factor_table(risk_packet).expect("risk table");
    let risk_second = render_risk_factor_table(risk_packet).expect("risk table repeat");
    assert_eq!(risk_first, risk_second);

    let expected_competitive = fixture
        .get("expected_competitive_table")
        .and_then(Value::as_str)
        .expect("expected_competitive_table must exist");
    let expected_temporal = fixture
        .get("expected_temporal_table")
        .and_then(Value::as_str)
        .expect("expected_temporal_table must exist");
    let expected_risk = fixture
        .get("expected_risk_table")
        .and_then(Value::as_str)
        .expect("expected_risk_table must exist");

    assert_eq!(cmp_first, expected_competitive);
    assert_eq!(tmp_first, expected_temporal);
    assert_eq!(risk_first, expected_risk);
}

#[test]
fn test_unknown_first_pre_synthesis_can_read_planning_metadata() {
    let evidence = json::json!({
        "sources": [{"url": "https://example.com/a"}],
        "content_chunks": [],
        "trust_metadata": {
            "planning": {
                "degraded_evidence_mode": true,
                "reason_codes": ["budget_exhausted"],
                "parity": {
                    "stitching_summary": {
                        "has_conflict": true
                    }
                }
            }
        }
    });

    let decision = evaluate_unknown_first_pre_synthesis(&evidence);
    assert!(decision.unknown_required);
    assert_eq!(decision.reason_code.as_deref(), Some("insufficient_evidence"));
}

fn collect_known_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                refs.insert(url.to_string());
            }
            if let Some(url) = source.get("canonical_url").and_then(Value::as_str) {
                refs.insert(url.to_string());
            }
        }
    }
    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                refs.insert(chunk_id.to_string());
            }
            if let Some(url) = chunk.get("source_url").and_then(Value::as_str) {
                refs.insert(url.to_string());
            }
        }
    }
    refs
}

mod json {
    pub use serde_json::json;
}
