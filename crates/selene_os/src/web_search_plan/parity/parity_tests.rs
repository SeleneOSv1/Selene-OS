#![forbid(unsafe_code)]

use crate::web_search_plan::parity::ambiguity::select_single_best_clarification;
use crate::web_search_plan::parity::diversification::diversify_for_high_tier;
use crate::web_search_plan::parity::multi_query::{decompose_query, MAX_SUB_QUERIES};
use crate::web_search_plan::parity::presentation_modes::plan_for_mode;
use crate::web_search_plan::parity::reformulation::apply_reformulation_ladder;
use crate::web_search_plan::parity::reranker::{rerank_candidates, RerankInput, RerankWeights};
use crate::web_search_plan::parity::stitching::{
    build_stitching_summary, deep_mode_contradiction_lines, stitch_sources,
};
use crate::web_search_plan::write::{render_write_packet, WriteFormatMode};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/parity_fixtures")
}

fn load_fixture(name: &str) -> Value {
    let path = fixture_dir().join(name);
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read fixture {}: {}", path.display(), e));
    serde_json::from_str::<Value>(&text)
        .unwrap_or_else(|e| panic!("failed to parse fixture {}: {}", path.display(), e))
}

#[test]
fn test_t1_multi_query_decomposition_stable_and_capped() {
    let fixture = load_fixture("multi_query_case.json");
    let query = fixture
        .get("query")
        .and_then(Value::as_str)
        .expect("query should be present");
    let existing = fixture
        .get("existing_sub_queries")
        .and_then(Value::as_array)
        .expect("existing_sub_queries should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let expected = fixture
        .get("expected_sub_queries")
        .and_then(Value::as_array)
        .expect("expected_sub_queries should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let first = decompose_query(query, &existing);
    let second = decompose_query(query, &existing);
    assert_eq!(first, second);
    assert_eq!(first.sub_queries, expected);
    assert!(first.sub_queries.len() <= MAX_SUB_QUERIES);
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DiversificationItem {
    id: String,
    url: String,
}

#[test]
fn test_t2_diversification_rule_triggers_deterministically_on_high_tier() {
    let fixture = load_fixture("diversification_case.json");
    let tier = fixture
        .get("importance_tier")
        .and_then(Value::as_str)
        .expect("importance_tier should be present");
    let min_distinct_domains = fixture
        .get("min_distinct_domains")
        .and_then(Value::as_u64)
        .expect("min_distinct_domains should be present") as usize;
    let items = serde_json::from_value::<Vec<DiversificationItem>>(
        fixture
            .get("items")
            .cloned()
            .expect("items should be present"),
    )
    .expect("items should decode");

    let first = diversify_for_high_tier(&items, tier, min_distinct_domains, |item| {
        url::Url::parse(item.url.as_str())
            .ok()
            .and_then(|parsed| parsed.host_str().map(ToString::to_string))
            .unwrap_or_default()
    });
    let second = diversify_for_high_tier(&items, tier, min_distinct_domains, |item| {
        url::Url::parse(item.url.as_str())
            .ok()
            .and_then(|parsed| parsed.host_str().map(ToString::to_string))
            .unwrap_or_default()
    });
    assert_eq!(first, second);

    let expected_order = fixture
        .get("expected_order")
        .and_then(Value::as_array)
        .expect("expected_order should exist")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let actual_order = first
        .reordered
        .iter()
        .map(|item| item.id.clone())
        .collect::<Vec<String>>();
    assert_eq!(actual_order, expected_order);
    assert_eq!(
        first.distinct_domain_count,
        fixture
            .get("expected_distinct_domain_count")
            .and_then(Value::as_u64)
            .expect("expected_distinct_domain_count should exist") as usize
    );
    assert_eq!(
        first.threshold_met,
        fixture
            .get("expected_threshold_met")
            .and_then(Value::as_bool)
            .expect("expected_threshold_met should exist")
    );
    assert_eq!(
        first.limitation_flag,
        fixture
            .get("expected_limited")
            .and_then(Value::as_bool)
            .expect("expected_limited should exist")
    );
}

#[test]
fn test_t3_reformulation_ladder_bounded_and_deterministic() {
    let fixture = load_fixture("reformulation_case.json");
    let query = fixture
        .get("query")
        .and_then(Value::as_str)
        .expect("query should exist");
    let existing_attempts = fixture
        .get("existing_attempts")
        .and_then(Value::as_array)
        .expect("existing_attempts should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let triggered = fixture
        .get("triggered")
        .and_then(Value::as_bool)
        .expect("triggered should exist");
    let max_attempts = fixture
        .get("max_attempts")
        .and_then(Value::as_u64)
        .expect("max_attempts should exist") as usize;

    let first = apply_reformulation_ladder(query, &existing_attempts, triggered, max_attempts);
    let second = apply_reformulation_ladder(query, &existing_attempts, triggered, max_attempts);
    assert_eq!(first, second);

    let expected_attempts = fixture
        .get("expected_attempts")
        .and_then(Value::as_array)
        .expect("expected_attempts should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let expected_queries = fixture
        .get("expected_reformulated_queries")
        .and_then(Value::as_array)
        .expect("expected_reformulated_queries should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    assert_eq!(first.rewrite_attempts, expected_attempts);
    assert_eq!(first.reformulated_queries, expected_queries);
    assert_eq!(
        first.exhausted,
        fixture
            .get("expected_exhausted")
            .and_then(Value::as_bool)
            .expect("expected_exhausted should exist")
    );
}

#[test]
fn test_t4_reranker_ordering_stable() {
    let fixture = load_fixture("rerank_case.json");
    let weights_obj = fixture.get("weights").expect("weights should exist");
    let weights = RerankWeights {
        w_relevance: weights_obj
            .get("w_relevance")
            .and_then(Value::as_i64)
            .expect("w_relevance") as i32,
        w_trust: weights_obj
            .get("w_trust")
            .and_then(Value::as_i64)
            .expect("w_trust") as i32,
        w_freshness: weights_obj
            .get("w_freshness")
            .and_then(Value::as_i64)
            .expect("w_freshness") as i32,
        w_corroboration: weights_obj
            .get("w_corroboration")
            .and_then(Value::as_i64)
            .expect("w_corroboration") as i32,
        w_spam_risk: weights_obj
            .get("w_spam_risk")
            .and_then(Value::as_i64)
            .expect("w_spam_risk") as i32,
    };

    let inputs = fixture
        .get("items")
        .and_then(Value::as_array)
        .expect("items should be array")
        .iter()
        .map(|item| RerankInput {
            stable_id: item
                .get("id")
                .and_then(Value::as_str)
                .expect("id")
                .to_string(),
            canonical_url: item
                .get("canonical_url")
                .and_then(Value::as_str)
                .expect("canonical_url")
                .to_string(),
            relevance: item.get("relevance").and_then(Value::as_i64).expect("relevance") as i32,
            trust: item.get("trust").and_then(Value::as_i64).expect("trust") as i32,
            freshness: item.get("freshness").and_then(Value::as_i64).expect("freshness") as i32,
            corroboration: item
                .get("corroboration")
                .and_then(Value::as_i64)
                .expect("corroboration") as i32,
            spam_risk: item
                .get("spam_risk")
                .and_then(Value::as_i64)
                .expect("spam_risk") as i32,
        })
        .collect::<Vec<RerankInput>>();

    let first = rerank_candidates(&inputs, weights);
    let second = rerank_candidates(&inputs, weights);
    assert_eq!(first, second);

    let expected_order = fixture
        .get("expected_order")
        .and_then(Value::as_array)
        .expect("expected_order should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let actual_order = first
        .iter()
        .map(|entry| entry.stable_id.clone())
        .collect::<Vec<String>>();
    assert_eq!(actual_order, expected_order);
}

#[test]
fn test_t5_ambiguity_selects_exactly_one_clarify() {
    let fixture = load_fixture("ambiguity_case.json");
    let query = fixture
        .get("query")
        .and_then(Value::as_str)
        .expect("query should exist");
    let clarify =
        select_single_best_clarification(query).expect("clarification should be selected");
    assert_eq!(
        clarify.missing_field,
        fixture
            .get("expected_missing_field")
            .and_then(Value::as_str)
            .expect("expected_missing_field")
    );
    assert_eq!(
        clarify.question,
        fixture
            .get("expected_question")
            .and_then(Value::as_str)
            .expect("expected_question")
    );
}

#[test]
fn test_t6_presentation_modes_preserve_truth_and_citations() {
    let fixture = load_fixture("presentation_modes_case.json");
    let synthesis_packet = fixture
        .get("synthesis_packet")
        .expect("synthesis_packet should exist");
    let expected_fragment = fixture
        .get("expected_direct_answer_fragment")
        .and_then(Value::as_str)
        .expect("expected_direct_answer_fragment should exist");
    let expected_citations = fixture
        .get("expected_citations")
        .and_then(Value::as_array)
        .expect("expected_citations should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let brief = render_write_packet(
        synthesis_packet,
        1_700_001_000_000i64,
        "trace-run33-brief",
        WriteFormatMode::Brief,
    )
    .expect("brief render should pass");
    let standard = render_write_packet(
        synthesis_packet,
        1_700_001_000_000i64,
        "trace-run33-standard",
        WriteFormatMode::Standard,
    )
    .expect("standard render should pass");
    let deep = render_write_packet(
        synthesis_packet,
        1_700_001_000_000i64,
        "trace-run33-deep",
        WriteFormatMode::Deep,
    )
    .expect("deep render should pass");

    for packet in [&brief.write_packet, &standard.write_packet, &deep.write_packet] {
        let formatted = packet
            .get("formatted_text")
            .and_then(Value::as_str)
            .expect("formatted_text should exist");
        assert!(formatted.contains(expected_fragment));
        for citation in &expected_citations {
            assert!(formatted.contains(citation));
        }
    }

    let brief_formatted = brief
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("brief formatted text should exist");
    let standard_formatted = standard
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("standard formatted text should exist");
    let deep_formatted = deep
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("deep formatted text should exist");

    assert!(
        !brief_formatted.contains("Contradictions:"),
        "brief mode must not include expanded contradiction section"
    );
    assert!(
        !standard_formatted.contains("Contradictions:"),
        "standard mode must not include expanded contradiction section"
    );
    assert!(
        deep_formatted.contains("Contradictions:"),
        "deep mode must include contradiction detail when conflict exists"
    );
}

#[test]
fn test_t7_stitching_produces_deterministic_contradiction_summary() {
    let fixture = load_fixture("stitching_conflict_case.json");
    let primary_sources = fixture
        .get("primary_sources")
        .and_then(Value::as_array)
        .expect("primary_sources should be array")
        .clone();
    let fallback_sources = fixture
        .get("fallback_sources")
        .and_then(Value::as_array)
        .expect("fallback_sources should be array")
        .clone();
    let open_failure_urls = fixture
        .get("open_failure_urls")
        .and_then(Value::as_array)
        .expect("open_failure_urls should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let reason_codes = fixture
        .get("reason_codes")
        .and_then(Value::as_array)
        .expect("reason_codes should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();

    let stitched_a = stitch_sources(&primary_sources, &fallback_sources);
    let stitched_b = stitch_sources(&primary_sources, &fallback_sources);
    assert_eq!(stitched_a, stitched_b);

    let stitched_urls = stitched_a
        .iter()
        .filter_map(|source| source.get("canonical_url").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let expected_urls = fixture
        .get("expected_stitched_urls")
        .and_then(Value::as_array)
        .expect("expected_stitched_urls should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    assert_eq!(stitched_urls, expected_urls);

    let source_titles = stitched_a
        .iter()
        .filter_map(|source| source.get("title").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let summary_a = build_stitching_summary(&source_titles, &open_failure_urls, &reason_codes);
    let summary_b = build_stitching_summary(&source_titles, &open_failure_urls, &reason_codes);
    assert_eq!(summary_a, summary_b);
    assert!(!summary_a.conflicts.is_empty());
    assert_eq!(
        summary_a.unknowns,
        fixture
            .get("expected_unknowns")
            .and_then(Value::as_array)
            .expect("expected_unknowns should be array")
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect::<Vec<String>>()
    );
}

#[test]
fn test_t8_no_new_facts_introduced_guard() {
    let fixture = load_fixture("presentation_modes_case.json");
    let synthesis_packet = fixture
        .get("synthesis_packet")
        .expect("synthesis_packet should exist");
    let source_bullets = synthesis_packet
        .get("bullet_evidence")
        .and_then(Value::as_array)
        .expect("bullet_evidence should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(|entry| entry.trim().to_string())
        .collect::<Vec<String>>();

    let standard = render_write_packet(
        synthesis_packet,
        1_700_001_100_000i64,
        "trace-run33-guard-standard",
        WriteFormatMode::Standard,
    )
    .expect("standard render should pass");
    let deep = render_write_packet(
        synthesis_packet,
        1_700_001_100_000i64,
        "trace-run33-guard-deep",
        WriteFormatMode::Deep,
    )
    .expect("deep render should pass");

    let standard_text = standard
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("standard formatted text should exist");
    let deep_text = deep
        .write_packet
        .get("formatted_text")
        .and_then(Value::as_str)
        .expect("deep formatted text should exist");

    let standard_claims = evidence_claims(standard_text);
    let deep_claims = evidence_claims(deep_text);
    assert_eq!(standard_claims, source_bullets);
    assert_eq!(deep_claims, source_bullets);

    let reason_codes = synthesis_packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .expect("reason_codes should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let uncertainty_flags = synthesis_packet
        .get("uncertainty_flags")
        .and_then(Value::as_array)
        .expect("uncertainty_flags should be array")
        .iter()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let contradiction_lines =
        deep_mode_contradiction_lines(&source_bullets, &uncertainty_flags, &reason_codes);
    for line in contradiction_lines {
        if line.starts_with("- Unknown:") {
            continue;
        }
        if line.starts_with("- Agrees:") {
            continue;
        }
        if line.starts_with("- Conflicts:") {
            continue;
        }
        panic!("unexpected contradiction line format: {}", line);
    }

    let brief_plan = plan_for_mode(WriteFormatMode::Brief);
    let deep_plan = plan_for_mode(WriteFormatMode::Deep);
    assert_eq!(brief_plan.inline_citations_per_bullet, 1);
    assert!(deep_plan.include_conflict_detail);
}

fn evidence_claims(formatted_text: &str) -> Vec<String> {
    lines_after_heading(formatted_text, "Evidence:")
        .iter()
        .map(|line| {
            line.trim_start_matches("- ")
                .split('[')
                .next()
                .unwrap_or_default()
                .trim()
                .to_string()
        })
        .collect::<Vec<String>>()
}

fn lines_after_heading(formatted_text: &str, heading: &str) -> Vec<String> {
    let mut seen = false;
    let mut out = Vec::new();
    for line in formatted_text.lines() {
        let trimmed = line.trim();
        if !seen {
            if trimmed == heading {
                seen = true;
            }
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.ends_with(':') && trimmed != heading {
            break;
        }
        out.push(trimmed.to_string());
    }
    out
}
