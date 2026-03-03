#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::{validate_packet, validate_packet_schema_registry};
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::trust::apply::enrich_evidence_sources;
use crate::web_search_plan::trust::explain::build_factors;
use crate::web_search_plan::trust::official_detector::{detect_from_url, TrustTier};
use crate::web_search_plan::trust::spam_signals::compute_spam_signals;
use crate::web_search_plan::trust::trust_score::compute_trust_score;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/trust_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str(&text).expect("fixture should parse")
}

#[test]
fn test_t1_official_detection_deterministic() {
    let fixture = load_fixture("official_domains.json");
    let cases = fixture
        .get("cases")
        .and_then(Value::as_array)
        .expect("cases must be array");
    for case in cases {
        let url = case.get("url").and_then(Value::as_str).expect("url");
        let expected = case
            .get("expected_tier")
            .and_then(Value::as_str)
            .expect("expected_tier");
        let first = detect_from_url(url);
        let second = detect_from_url(url);
        assert_eq!(first, second);
        assert_eq!(first.trust_tier.as_str(), expected);
    }
}

#[test]
fn test_t2_spam_penalties_deterministic() {
    let fixture = load_fixture("spammy_urls.json");
    let cases = fixture
        .get("cases")
        .and_then(Value::as_array)
        .expect("cases must be array");
    for case in cases {
        let url = case.get("url").and_then(Value::as_str).expect("url");
        let title = case.get("title").and_then(Value::as_str).expect("title");
        let snippet = case
            .get("snippet")
            .and_then(Value::as_str)
            .expect("snippet");
        let first = compute_spam_signals(url, title, snippet);
        let second = compute_spam_signals(url, title, snippet);
        assert_eq!(first, second);
        if let Some(min) = case.get("min_spam_risk_score").and_then(Value::as_f64) {
            assert!(first.spam_risk_score >= min);
        }
        if let Some(max) = case.get("max_spam_risk_score").and_then(Value::as_f64) {
            assert!(first.spam_risk_score <= max);
        }
    }
}

#[test]
fn test_t3_trust_score_deterministic() {
    let spam = compute_spam_signals(
        "https://www.reuters.com/world/us?utm_source=ref",
        "REUTERS MARKET UPDATE",
        "Market coverage",
    );
    let first = compute_trust_score(
        TrustTier::Medium,
        &spam,
        Some("reuters.com"),
        Some(1_707_613_600_000),
        1_707_700_000_000,
        2,
    );
    let second = compute_trust_score(
        TrustTier::Medium,
        &spam,
        Some("reuters.com"),
        Some(1_707_613_600_000),
        1_707_700_000_000,
        2,
    );
    assert_eq!(first, second);
}

#[test]
fn test_t4_explain_factors_stable_ordering() {
    let detection = detect_from_url("https://www.sec.gov/rules/final");
    let spam = compute_spam_signals(
        "https://www.sec.gov/rules/final",
        "SEC filing update",
        "Official SEC release.",
    );
    let score = compute_trust_score(
        detection.trust_tier,
        &spam,
        detection.host.as_deref(),
        Some(1_707_696_400_000),
        1_707_700_000_000,
        2,
    );
    let first = build_factors(detection.trust_tier, &detection.reasons, &spam.reasons, &score);
    let second = build_factors(detection.trust_tier, &detection.reasons, &spam.reasons, &score);
    assert_eq!(first, second);

    let mut sorted = first.clone();
    sorted.sort();
    assert_eq!(first, sorted, "factors should be in stable sorted order");
}

#[test]
fn test_t5_enrichment_preserves_source_order() {
    let mixed = load_fixture("mixed_sources.json");
    let expected = load_fixture("expected_scored_sources.json");
    let now_ms = mixed
        .get("now_ms")
        .and_then(Value::as_i64)
        .expect("now_ms");
    let evidence_packet = mixed.get("evidence_packet").expect("evidence_packet");
    let result = enrich_evidence_sources(evidence_packet, now_ms).expect("enrichment should pass");

    let actual_urls = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources")
        .iter()
        .filter_map(|source| source.get("url").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    let expected_urls = expected
        .get("expected_order_urls")
        .and_then(Value::as_array)
        .expect("expected_order_urls")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<&str>>();
    assert_eq!(actual_urls, expected_urls, "source ordering must be preserved");

    let actual_tiers = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources")
        .iter()
        .filter_map(|source| source.get("trust_tier").and_then(Value::as_str))
        .collect::<Vec<&str>>();
    let expected_tiers = expected
        .get("expected_tiers")
        .and_then(Value::as_array)
        .expect("expected_tiers")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<&str>>();
    assert_eq!(actual_tiers, expected_tiers);

    let required_factor_subsets = expected
        .get("required_factor_subsets")
        .and_then(Value::as_array)
        .expect("required_factor_subsets");
    let sources = result
        .evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .expect("sources");
    for (source, required_subset) in sources.iter().zip(required_factor_subsets.iter()) {
        let factors = source
            .get("trust_factors")
            .and_then(Value::as_array)
            .expect("trust_factors");
        let factor_values = factors.iter().filter_map(Value::as_str).collect::<Vec<&str>>();
        let required = required_subset
            .as_array()
            .expect("required subset array")
            .iter()
            .filter_map(Value::as_str);
        for needle in required {
            assert!(
                factor_values.contains(&needle),
                "missing required factor {} in {:?}",
                needle,
                factor_values
            );
        }
    }
}

#[test]
fn test_t6_schema_validation_passes_with_enriched_sources() {
    let mixed = load_fixture("mixed_sources.json");
    let now_ms = mixed
        .get("now_ms")
        .and_then(Value::as_i64)
        .expect("now_ms");
    let evidence_packet = mixed.get("evidence_packet").expect("evidence_packet");
    let result = enrich_evidence_sources(evidence_packet, now_ms).expect("enrichment should pass");

    let registry = load_packet_schema_registry().expect("packet schema registry should load");
    validate_packet_schema_registry(&registry).expect("packet schema registry should be valid");
    validate_packet("EvidencePacket", &result.evidence_packet, &registry)
        .expect("enriched evidence should still satisfy schema");
}
