#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::audit_fields::{
    append_perf_cost_audit_fields, PerfCostAuditMetrics,
};
use crate::web_search_plan::perf_cost::budgets::{
    budget_plan_for_tier, ProviderCallBudget, Stage, StageBudgetTracker,
};
use crate::web_search_plan::perf_cost::concurrency::{ConcurrencyController, SubmitDecision};
use crate::web_search_plan::perf_cost::degrade::{DegradeController, DegradeStep};
use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};
use crate::web_search_plan::perf_cost::timeouts::{
    clamp_provider_timeout, clamp_url_fetch_total_timeout, timeout_envelope_for_tier,
};
use crate::web_search_plan::perf_cost::{enforce_url_open_cap, PerfCostPolicySnapshot};
use serde_json::{json, Value};
use std::collections::BTreeMap;

#[test]
fn test_t1_tier_selection_returns_correct_caps() {
    let low = caps_for_tier(ImportanceTier::Low);
    let medium = caps_for_tier(ImportanceTier::Medium);
    let high = caps_for_tier(ImportanceTier::High);

    assert_eq!(low.max_results_from_search, 3);
    assert_eq!(low.max_queries, 2);
    assert_eq!(low.max_urls_opened_per_query, 1);
    assert_eq!(low.timeout_per_provider_ms, 1_000);

    assert_eq!(medium.max_results_from_search, 5);
    assert_eq!(medium.max_queries, 3);
    assert_eq!(medium.max_urls_opened_per_query, 2);
    assert_eq!(medium.timeout_per_provider_ms, 2_000);

    assert_eq!(high.max_results_from_search, 10);
    assert_eq!(high.max_queries, 4);
    assert_eq!(high.max_urls_opened_per_query, 3);
    assert_eq!(high.timeout_per_provider_ms, 3_000);

    let snapshot = PerfCostPolicySnapshot::from_importance_tier_str("unknown-tier");
    assert_eq!(snapshot.tier, ImportanceTier::Medium);
}

#[test]
fn test_t2_timeouts_derived_deterministically() {
    let medium = timeout_envelope_for_tier(ImportanceTier::Medium);
    assert_eq!(medium.per_provider_timeout_ms, 2_000);
    assert_eq!(medium.total_timeout_per_turn_ms, 7_000);
    assert_eq!(medium.url_fetch_total_timeout_ms, 4_000);

    assert_eq!(clamp_provider_timeout(4_200, ImportanceTier::Medium), 2_000);
    assert_eq!(clamp_provider_timeout(900, ImportanceTier::Medium), 900);

    assert_eq!(clamp_url_fetch_total_timeout(9_999, ImportanceTier::High), 7_000);
    assert_eq!(clamp_url_fetch_total_timeout(1_500, ImportanceTier::High), 1_500);
}

#[test]
fn test_t3_concurrency_controller_enforces_cap_and_fifo() {
    let mut controller = ConcurrencyController::new(2, 2).expect("controller");
    assert_eq!(controller.submit("a"), SubmitDecision::Started);
    assert_eq!(controller.submit("b"), SubmitDecision::Started);
    assert_eq!(controller.submit("c"), SubmitDecision::Queued);
    assert_eq!(controller.submit("d"), SubmitDecision::Queued);
    assert_eq!(
        controller.submit("e"),
        SubmitDecision::Rejected {
            reason_code: "quota_exceeded"
        }
    );

    let next = controller.complete("a");
    assert_eq!(next.as_deref(), Some("c"));
    assert_eq!(controller.active_count(), 2);
    assert_eq!(controller.queue_len(), 1);
    assert_eq!(controller.concurrency_peak(), 2);
}

#[test]
fn test_t4_degrade_steps_follow_fixed_order() {
    let mut controller = DegradeController::new(ImportanceTier::High);

    let first = controller.advance();
    assert_eq!(
        first.step,
        Some(DegradeStep::ReduceMaxResultsFromSearchToTierMinimum)
    );

    let second = controller.advance();
    assert_eq!(
        second.step,
        Some(DegradeStep::ReduceMaxUrlsOpenedPerQueryToOne)
    );

    let third = controller.advance();
    assert_eq!(third.step, Some(DegradeStep::DisableUrlOpensSnippetOnly));

    let fourth = controller.advance();
    assert_eq!(fourth.step, Some(DegradeStep::FailClosed));
    assert!(fourth.fail_closed);
    assert_eq!(fourth.reason_code, "budget_exhausted");
}

#[test]
fn test_t5_budget_exhaustion_triggers_deterministic_reason_code() {
    let plan = budget_plan_for_tier(ImportanceTier::Low);
    let mut tracker = StageBudgetTracker::new(plan);

    tracker
        .record_stage_timing(Stage::X, 200)
        .expect("stage within budget");

    let violation = tracker
        .record_stage_timing(Stage::Search, plan.stage_deadlines_ms.search + 1)
        .expect_err("must exceed stage budget");
    assert_eq!(violation.reason_code, "budget_exhausted");
    assert_eq!(violation.stage, Stage::Search);
}

#[test]
fn test_t6_snippet_only_degrade_blocks_url_opens() {
    let mut controller = DegradeController::new(ImportanceTier::Medium);
    let _ = controller.advance();
    let _ = controller.advance();
    let third = controller.advance();

    assert!(third.execution_caps.snippet_only_mode);
    assert_eq!(third.execution_caps.max_urls_opened_per_query, 0);

    let cap_check = enforce_url_open_cap(0, Some(third.execution_caps.max_urls_opened_per_query));
    assert_eq!(cap_check, Err("budget_exhausted"));
}

#[test]
fn test_t7_audit_fields_populated_deterministically() {
    let mut audit_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": "PH1.OS",
        "intended_consumers": ["PH1.J"],
        "created_at_ms": 1_700_000_000_000i64,
        "trace_id": "trace-run15",
        "turn_state_transition": "OUTPUT_RENDERED",
        "packet_hashes": {"input": "a"},
        "evidence_hash": "e",
        "response_hash": "r",
        "reason_codes": [],
        "policy_snapshot_id": "policy-default"
    });

    let mut stage_timings = BTreeMap::new();
    stage_timings.insert("X".to_string(), 200);
    stage_timings.insert("SEARCH".to_string(), 700);

    let metrics = PerfCostAuditMetrics {
        importance_tier: "medium".to_string(),
        budgets_applied: json!({"max_results_from_search": 5, "max_urls_opened_per_query": 2}),
        stage_timings_ms: stage_timings,
        degraded: true,
        degrade_step: Some("disable_url_opens_snippet_only_mode".to_string()),
        concurrency_peak: 2,
        total_provider_calls: 3,
        urls_opened_count: 1,
        stop_reason: "budget_exhausted".to_string(),
    };

    append_perf_cost_audit_fields(&mut audit_packet, &metrics).expect("append must pass");

    let serialized = serde_json::to_string(&audit_packet).expect("serialize");
    let reparsed: Value = serde_json::from_str(&serialized).expect("parse");

    let perf = reparsed
        .pointer("/turn_state_transition/perf_cost_audit")
        .expect("perf cost block must exist");
    assert_eq!(perf.get("importance_tier").and_then(Value::as_str), Some("medium"));
    assert_eq!(perf.get("degraded").and_then(Value::as_bool), Some(true));
    assert_eq!(
        perf.get("degrade_step").and_then(Value::as_str),
        Some("disable_url_opens_snippet_only_mode")
    );
}

#[test]
fn test_provider_call_budget_caps_are_enforced() {
    let mut budget = ProviderCallBudget::for_tier(ImportanceTier::Low);

    budget.record_lead_call().expect("lead 1");
    budget.record_fallback_call().expect("fallback 1");
    budget.record_lead_call().expect("lead 2");

    assert_eq!(budget.total_provider_calls(), 3);
    assert_eq!(budget.fallback_invocations(), 1);
    assert_eq!(budget.record_fallback_call(), Err("budget_exhausted"));
    assert_eq!(budget.validate_retry_count(1), Err("budget_exhausted"));
}
