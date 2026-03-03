#![forbid(unsafe_code)]

use crate::web_search_plan::planning::budget_control::OpenBudgetPolicy;
use crate::web_search_plan::planning::scoring::{ScoringPolicy, ScoringWeights};
use crate::web_search_plan::planning::{
    execute_search_topk_pipeline_with_opener, OpenFailure, OpenSuccess, PlanningInput,
    PlanningPolicy, SearchCandidate, StopReason, PLANNING_ENGINE_ID,
};
use serde_json::Value;

fn candidate(
    title: &str,
    url: &str,
    canonical_url: &str,
    provider_rank: usize,
    relevance: i32,
    trust_tier: i32,
    freshness_score: i32,
    corroboration_count: i32,
    spam_risk: i32,
) -> SearchCandidate {
    SearchCandidate {
        title: title.to_string(),
        url: url.to_string(),
        snippet: format!("{} snippet", title),
        canonical_url: canonical_url.to_string(),
        provider_id: "brave_web_search".to_string(),
        provider_rank,
        relevance,
        trust_tier,
        freshness_score,
        corroboration_count,
        spam_risk,
    }
}

fn planning_input(candidates: Vec<SearchCandidate>) -> PlanningInput {
    PlanningInput {
        trace_id: "trace-run9".to_string(),
        query: "test query".to_string(),
        created_at_ms: 1_772_496_500_000i64,
        retrieved_at_ms: 1_772_496_500_000i64,
        produced_by: PLANNING_ENGINE_ID.to_string(),
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ],
        importance_tier: "medium".to_string(),
        rewrite_attempts: vec!["rewrite-a".to_string()],
        sub_queries: vec!["test query".to_string(), "test sub-query".to_string()],
        candidates,
    }
}

fn success_for(candidate: &SearchCandidate, extracted_chars: usize) -> OpenSuccess {
    OpenSuccess {
        final_url: candidate.url.clone(),
        title: candidate.title.clone(),
        extracted_text: format!("extracted body for {}", candidate.canonical_url),
        extracted_chars,
    }
}

#[test]
fn test_t1_same_input_identical_top_k_selection() {
    let candidates = vec![
        candidate(
            "A",
            "https://a.example.com/1",
            "https://a.example.com/1",
            1,
            9,
            9,
            8,
            2,
            1,
        ),
        candidate(
            "B",
            "https://b.example.com/1",
            "https://b.example.com/1",
            2,
            8,
            8,
            8,
            1,
            1,
        ),
        candidate(
            "C",
            "https://c.example.com/1",
            "https://c.example.com/1",
            3,
            7,
            7,
            7,
            1,
            1,
        ),
    ];

    let input = planning_input(candidates.clone());
    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 2;

    let run_a =
        execute_search_topk_pipeline_with_opener(&input, &policy, |c| Ok(success_for(c, 300)))
            .expect("run A should pass");
    let run_b =
        execute_search_topk_pipeline_with_opener(&input, &policy, |c| Ok(success_for(c, 300)))
            .expect("run B should pass");

    assert_eq!(run_a.top_k_selected, run_b.top_k_selected);
    assert_eq!(run_a.stop_reason, StopReason::KReached);
    assert_eq!(
        run_a.evidence_packet.pointer("/sources"),
        run_b.evidence_packet.pointer("/sources")
    );
}

#[test]
fn test_t2_tie_break_stability() {
    let candidates = vec![
        candidate(
            "HighTrust",
            "https://z.example.com/1",
            "https://z.example.com/1",
            1,
            1,
            9,
            3,
            0,
            0,
        ),
        candidate(
            "HigherFreshness",
            "https://y.example.com/1",
            "https://y.example.com/1",
            2,
            1,
            8,
            9,
            0,
            0,
        ),
        candidate(
            "LexicalFirst",
            "https://a.example.com/1",
            "https://a.example.com/1",
            3,
            1,
            8,
            9,
            0,
            0,
        ),
    ];

    let input = planning_input(candidates);
    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 3;
    policy.scoring_policy = ScoringPolicy::new(
        "snapshot-tie-break",
        ScoringWeights {
            w_relevance: 0,
            w_trust_tier: 0,
            w_freshness: 0,
            w_corroboration: 0,
            w_spam_risk: 0,
        },
    );

    let result =
        execute_search_topk_pipeline_with_opener(&input, &policy, |c| Ok(success_for(c, 120)))
            .expect("tie-break run should pass");

    assert_eq!(
        result.top_k_selected,
        vec![
            "https://z.example.com/1".to_string(),
            "https://a.example.com/1".to_string(),
            "https://y.example.com/1".to_string(),
        ]
    );
}

#[test]
fn test_t3_per_domain_cap_enforced() {
    let candidates = vec![
        candidate(
            "DomainA-1",
            "https://same.example.com/1",
            "https://same.example.com/1",
            1,
            9,
            9,
            9,
            1,
            0,
        ),
        candidate(
            "DomainA-2",
            "https://same.example.com/2",
            "https://same.example.com/2",
            2,
            8,
            8,
            8,
            1,
            0,
        ),
        candidate(
            "DomainB",
            "https://other.example.com/1",
            "https://other.example.com/1",
            3,
            7,
            7,
            7,
            1,
            0,
        ),
    ];

    let input = planning_input(candidates);
    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 3;
    policy.open_budget.per_domain_cap = 1;

    let result =
        execute_search_topk_pipeline_with_opener(&input, &policy, |c| Ok(success_for(c, 100)))
            .expect("per-domain cap run should pass");

    assert_eq!(result.top_k_selected.len(), 2);
    assert!(
        result
            .top_k_selected
            .iter()
            .filter(|url| url.contains("same.example.com"))
            .count()
            == 1
    );
}

#[test]
fn test_t4_dead_link_deterministic_replacement() {
    let candidates = vec![
        candidate(
            "Bad",
            "https://bad.example.com/1",
            "https://bad.example.com/1",
            1,
            9,
            9,
            9,
            1,
            0,
        ),
        candidate(
            "Good1",
            "https://good.example.com/1",
            "https://good.example.com/1",
            2,
            8,
            8,
            8,
            1,
            0,
        ),
        candidate(
            "Good2",
            "https://good2.example.com/1",
            "https://good2.example.com/1",
            3,
            7,
            7,
            7,
            1,
            0,
        ),
    ];

    let input = planning_input(candidates);
    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 2;

    let result = execute_search_topk_pipeline_with_opener(&input, &policy, |c| {
        if c.canonical_url == "https://bad.example.com/1" {
            Err(OpenFailure {
                canonical_url: c.canonical_url.clone(),
                reason_code: "provider_upstream_failed".to_string(),
                error_kind: "http_non_200".to_string(),
                message: "dead link".to_string(),
            })
        } else {
            Ok(success_for(c, 180))
        }
    })
    .expect("dead-link replacement should pass");

    assert_eq!(result.stop_reason, StopReason::KReached);
    assert_eq!(result.open_failures.len(), 1);
    assert_eq!(result.top_k_selected.len(), 3);
}

#[test]
fn test_t5_stop_condition_correctness() {
    let mut input = planning_input(Vec::new());
    let mut policy = PlanningPolicy::default();
    policy.max_rewrite_attempts = 1;

    input.rewrite_attempts = vec!["rewrite-1".to_string()];
    let exhausted = execute_search_topk_pipeline_with_opener(&input, &policy, |_c| {
        Ok(OpenSuccess {
            final_url: String::new(),
            title: String::new(),
            extracted_text: String::new(),
            extracted_chars: 0,
        })
    })
    .expect("rewrite exhausted run should pass");
    assert_eq!(exhausted.stop_reason, StopReason::RewriteAttemptsExhausted);

    input.rewrite_attempts.clear();
    let provider_exhausted = execute_search_topk_pipeline_with_opener(&input, &policy, |_c| {
        Ok(OpenSuccess {
            final_url: String::new(),
            title: String::new(),
            extracted_text: String::new(),
            extracted_chars: 0,
        })
    })
    .expect("provider exhausted run should pass");
    assert_eq!(
        provider_exhausted.stop_reason,
        StopReason::ProviderResultsExhausted
    );
}

#[test]
fn test_t6_snippet_only_fallback_behavior() {
    let candidates = vec![candidate(
        "OnlyOne",
        "https://fallback.example.com/1",
        "https://fallback.example.com/1",
        1,
        9,
        9,
        9,
        1,
        0,
    )];

    let input = planning_input(candidates);
    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 1;
    policy.snippet_fallback_min_sources = 2;

    let result = execute_search_topk_pipeline_with_opener(&input, &policy, |c| {
        Err(OpenFailure {
            canonical_url: c.canonical_url.clone(),
            reason_code: "provider_upstream_failed".to_string(),
            error_kind: "timeout_exceeded".to_string(),
            message: "timeout".to_string(),
        })
    })
    .expect("snippet fallback run should pass");

    assert!(result.degraded_evidence_mode);
    assert_eq!(
        result
            .evidence_packet
            .pointer("/trust_metadata/planning/reason_codes")
            .and_then(Value::as_array)
            .expect("reason codes should exist")
            .iter()
            .any(|v| v.as_str() == Some("insufficient_evidence")),
        true
    );
    assert_eq!(
        result
            .evidence_packet
            .pointer("/sources/0/snippet_only")
            .and_then(Value::as_bool),
        Some(true)
    );
}

#[test]
fn test_t7_provenance_fields_populated() {
    let candidates = vec![
        candidate(
            "A",
            "https://a.example.com/1",
            "https://a.example.com/1",
            1,
            9,
            9,
            9,
            2,
            0,
        ),
        candidate(
            "B",
            "https://b.example.com/1",
            "https://b.example.com/1",
            2,
            8,
            8,
            8,
            1,
            0,
        ),
    ];

    let mut input = planning_input(candidates);
    input.importance_tier = "high".to_string();
    input.rewrite_attempts = vec!["rw1".to_string(), "rw2".to_string()];
    input.sub_queries = vec!["q1".to_string(), "q2".to_string()];

    let mut policy = PlanningPolicy::default();
    policy.open_budget.max_urls_opened_per_query = 1;

    let result = execute_search_topk_pipeline_with_opener(&input, &policy, |c| {
        if c.canonical_url == "https://a.example.com/1" {
            Err(OpenFailure {
                canonical_url: c.canonical_url.clone(),
                reason_code: "provider_upstream_failed".to_string(),
                error_kind: "http_non_200".to_string(),
                message: "404".to_string(),
            })
        } else {
            Ok(success_for(c, 140))
        }
    })
    .expect("provenance run should pass");

    let planning = result
        .evidence_packet
        .pointer("/trust_metadata/planning")
        .expect("planning metadata must exist");

    assert_eq!(
        planning.get("importance_tier").and_then(Value::as_str),
        Some("high")
    );
    assert!(planning
        .get("rewrite_attempts")
        .and_then(Value::as_array)
        .is_some());
    assert!(planning
        .get("sub_queries")
        .and_then(Value::as_array)
        .is_some());
    assert!(planning
        .get("top_k_selected")
        .and_then(Value::as_array)
        .is_some());
    assert!(planning
        .get("stop_reason")
        .and_then(Value::as_str)
        .is_some());
    assert!(planning
        .get("open_failures")
        .and_then(Value::as_array)
        .is_some());
}

#[test]
fn test_t8_budget_exhaustion_deterministic() {
    let candidates = vec![
        candidate(
            "A",
            "https://a.example.com/1",
            "https://a.example.com/1",
            1,
            9,
            9,
            9,
            1,
            0,
        ),
        candidate(
            "B",
            "https://b.example.com/1",
            "https://b.example.com/1",
            2,
            8,
            8,
            8,
            1,
            0,
        ),
    ];

    let input = planning_input(candidates.clone());
    let mut policy = PlanningPolicy::default();
    policy.open_budget = OpenBudgetPolicy {
        max_urls_opened_per_query: 2,
        per_domain_cap: 2,
        max_total_extracted_chars: 50,
        max_chunks_total: 64,
    };

    let run_once = |input: &PlanningInput| {
        execute_search_topk_pipeline_with_opener(input, &policy, |c| {
            if c.canonical_url == "https://a.example.com/1" {
                Ok(success_for(c, 90))
            } else {
                Ok(success_for(c, 20))
            }
        })
        .expect("budget run should pass")
    };

    let first = run_once(&input);
    let second = run_once(&input);

    assert_eq!(first.stop_reason, StopReason::BudgetExhausted);
    assert_eq!(second.stop_reason, StopReason::BudgetExhausted);
    assert_eq!(first.top_k_selected, second.top_k_selected);
    assert_eq!(
        first
            .evidence_packet
            .pointer("/trust_metadata/planning/reason_codes")
            .and_then(Value::as_array)
            .expect("reason codes should exist")
            .iter()
            .any(|v| v.as_str() == Some("budget_exhausted")),
        true
    );
}
