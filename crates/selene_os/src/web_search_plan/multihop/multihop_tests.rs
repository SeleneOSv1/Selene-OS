#![forbid(unsafe_code)]

use crate::web_search_plan::multihop::hop_budget::{HopBudget, HopBudgetTracker};
use crate::web_search_plan::multihop::hop_plan::{build_hop_plan, HopMode, HopPlanInput};
use crate::web_search_plan::multihop::hop_runner::{
    execute_hop_plan, HopExecutionError, HopExecutionOutput, HopExecutor, ProviderRunSummary,
};
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
struct MultiHopFixture {
    root_query: String,
    mode: String,
    importance_tier: String,
    sub_queries: Vec<String>,
    #[serde(default)]
    max_hops: Option<usize>,
    #[serde(default)]
    repeated_canonical_url: Option<String>,
    #[serde(default)]
    max_time_per_hop_ms: Option<u64>,
    #[serde(default)]
    max_total_time_ms: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedPlansFixture {
    simple_chain: ExpectedPlanCase,
    cycle_case: ExpectedPlanCase,
}

#[derive(Debug, Clone, Deserialize)]
struct ExpectedPlanCase {
    mode: String,
    expected_hops: Vec<String>,
}

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/web_search_plan/multihop_fixtures")
}

fn load_fixture<T: for<'de> Deserialize<'de>>(name: &str) -> T {
    let text = fs::read_to_string(fixture_dir().join(name)).expect("fixture should load");
    serde_json::from_str::<T>(&text).expect("fixture should parse")
}

fn parse_mode(raw: &str) -> HopMode {
    HopMode::parse(raw).expect("fixture mode should be valid")
}

fn provider_run(provider_id: &str) -> ProviderRunSummary {
    ProviderRunSummary {
        provider_id: provider_id.to_string(),
        endpoint: "web".to_string(),
        success: true,
    }
}

fn success_output(
    provider_id: &str,
    source_urls: Vec<String>,
    elapsed_ms: u64,
    provider_calls: usize,
    url_opens: usize,
) -> HopExecutionOutput {
    HopExecutionOutput {
        provider_runs: vec![provider_run(provider_id)],
        source_urls,
        elapsed_ms,
        provider_calls,
        url_opens,
        reason_code: None,
    }
}

#[derive(Debug, Clone)]
struct DeterministicExecutor {
    scripted: Vec<Result<HopExecutionOutput, HopExecutionError>>,
    cursor: usize,
}

impl DeterministicExecutor {
    fn new(scripted: Vec<Result<HopExecutionOutput, HopExecutionError>>) -> Self {
        Self { scripted, cursor: 0 }
    }

    fn executed_count(&self) -> usize {
        self.cursor
    }
}

impl HopExecutor for DeterministicExecutor {
    fn execute(
        &mut self,
        _hop: &crate::web_search_plan::multihop::hop_plan::Hop,
    ) -> Result<HopExecutionOutput, HopExecutionError> {
        if self.cursor >= self.scripted.len() {
            return Err(HopExecutionError::new(
                "insufficient_evidence",
                "no scripted hop output",
                0,
                0,
                0,
            ));
        }
        let out = self.scripted[self.cursor].clone();
        self.cursor = self.cursor.saturating_add(1);
        out
    }
}

#[test]
fn test_t1_same_root_query_yields_identical_hop_plan() {
    let fixture: MultiHopFixture = load_fixture("simple_chain.json");
    let expected: ExpectedPlansFixture = load_fixture("expected_plans.json");

    let input = HopPlanInput {
        root_query: fixture.root_query.clone(),
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries.clone(),
        max_hops: fixture.max_hops.unwrap_or(5),
    };
    let first = build_hop_plan(&input).expect("first hop plan should build");
    let second = build_hop_plan(&input).expect("second hop plan should build");

    assert_eq!(first, second, "same input must produce identical hop plan");
    assert_eq!(first.plan_id, second.plan_id);
    assert_eq!(first.hops.len(), expected.simple_chain.expected_hops.len());
    assert_eq!(
        first.hops.iter().map(|hop| hop.sub_query.clone()).collect::<Vec<String>>(),
        expected.simple_chain.expected_hops
    );
    assert_eq!(input.mode.as_str(), expected.simple_chain.mode);
}

#[test]
fn test_t2_max_hop_enforced_deterministically() {
    let fixture: MultiHopFixture = load_fixture("simple_chain.json");
    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: 4,
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");

    let mut budget = HopBudget::for_tier(ImportanceTier::Low);
    budget.max_hops = 1;
    let mut executor = DeterministicExecutor::new(vec![
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/a".to_string()],
            25,
            1,
            1,
        )),
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/b".to_string()],
            25,
            1,
            1,
        )),
    ]);
    let result = execute_hop_plan(&plan, budget, &mut executor).expect("run should complete");

    assert_eq!(result.stop_reason, "budget_exhausted");
    assert!(result.reason_codes.iter().any(|code| code == "budget_exhausted"));
    assert!(result.reason_codes.iter().any(|code| code == "insufficient_evidence"));
    assert_eq!(result.hop_records.len(), 1, "only first hop should execute");
}

#[test]
fn test_t3_max_time_enforced_deterministically() {
    let fixture: MultiHopFixture = load_fixture("budget_exhaust_case.json");
    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: fixture.max_hops.unwrap_or(5),
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");

    let mut budget = HopBudget::from_importance_tier(fixture.importance_tier.as_str());
    budget.max_time_per_hop_ms = fixture.max_time_per_hop_ms.unwrap_or(100);
    budget.max_total_time_ms = fixture.max_total_time_ms.unwrap_or(200);
    let mut executor = DeterministicExecutor::new(vec![Ok(success_output(
        "UrlFetch",
        vec!["https://example.com/time".to_string()],
        150,
        1,
        1,
    ))]);

    let result = execute_hop_plan(&plan, budget, &mut executor).expect("run should complete");
    assert_eq!(result.stop_reason, "timeout_exceeded");
    assert!(result.reason_codes.iter().any(|code| code == "timeout_exceeded"));
    assert!(result.reason_codes.iter().any(|code| code == "insufficient_evidence"));
}

#[test]
fn test_t4_cycle_detection_triggers_fail_closed() {
    let fixture: MultiHopFixture = load_fixture("cycle_case.json");
    let expected: ExpectedPlansFixture = load_fixture("expected_plans.json");
    let repeated_url = fixture
        .repeated_canonical_url
        .clone()
        .expect("cycle fixture must define repeated url");

    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: fixture.max_hops.unwrap_or(5),
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");
    assert_eq!(
        plan.hops.iter().map(|hop| hop.sub_query.clone()).collect::<Vec<String>>(),
        expected.cycle_case.expected_hops
    );

    let mut executor = DeterministicExecutor::new(vec![
        Ok(success_output(
            "BraveWebSearch",
            vec![repeated_url.clone()],
            30,
            1,
            1,
        )),
        Ok(success_output(
            "OpenAI_WebSearch",
            vec![repeated_url],
            30,
            1,
            1,
        )),
    ]);
    let budget = HopBudget::for_tier(ImportanceTier::Medium);
    let result = execute_hop_plan(&plan, budget, &mut executor).expect("run should complete");

    assert_eq!(result.stop_reason, "policy_violation");
    assert!(result.cycle_detected);
    assert!(result.reason_codes.iter().any(|code| code == "policy_violation"));
}

#[test]
fn test_t5_hop_proof_chain_is_produced_and_ordered() {
    let fixture: MultiHopFixture = load_fixture("simple_chain.json");
    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: 2,
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");

    let script = vec![
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/1".to_string()],
            20,
            1,
            1,
        )),
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/2".to_string()],
            20,
            1,
            1,
        )),
    ];

    let mut executor_a = DeterministicExecutor::new(script.clone());
    let mut executor_b = DeterministicExecutor::new(script);
    let budget = HopBudget::for_tier(ImportanceTier::Medium);
    let run_a = execute_hop_plan(&plan, budget, &mut executor_a).expect("first run should pass");
    let run_b = execute_hop_plan(&plan, budget, &mut executor_b).expect("second run should pass");

    assert_eq!(run_a.proof_chain.hops_executed, 2);
    assert_eq!(run_a.proof_chain.hops_planned, 2);
    assert_eq!(
        run_a
            .proof_chain
            .hop_proofs
            .iter()
            .map(|proof| proof.hop_index)
            .collect::<Vec<usize>>(),
        vec![0, 1]
    );
    assert_eq!(
        run_a.proof_chain.proof_chain_hash,
        run_b.proof_chain.proof_chain_hash,
        "proof chain hash should be deterministic"
    );
}

#[test]
fn test_t6_stop_reason_is_set_correctly() {
    let fixture: MultiHopFixture = load_fixture("simple_chain.json");
    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: 1,
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");
    let budget = HopBudget::for_tier(ImportanceTier::Medium);

    let mut success_executor = DeterministicExecutor::new(vec![Ok(success_output(
        "BraveWebSearch",
        vec!["https://example.com/success".to_string()],
        20,
        1,
        1,
    ))]);
    let success = execute_hop_plan(&plan, budget, &mut success_executor).expect("success run");
    assert_eq!(success.stop_reason, "success");
    assert!(success.reason_codes.is_empty());

    let mut failure_executor = DeterministicExecutor::new(vec![Err(HopExecutionError::new(
        "insufficient_evidence",
        "no results",
        20,
        1,
        1,
    ))]);
    let failure = execute_hop_plan(&plan, budget, &mut failure_executor).expect("failure run");
    assert_eq!(failure.stop_reason, "insufficient_evidence");
    assert!(failure.reason_codes.iter().any(|code| code == "insufficient_evidence"));
}

#[test]
fn test_t7_no_hidden_hops_executed_count_matches_proof_chain() {
    let fixture: MultiHopFixture = load_fixture("simple_chain.json");
    let input = HopPlanInput {
        root_query: fixture.root_query,
        mode: parse_mode(fixture.mode.as_str()),
        requested_sub_queries: fixture.sub_queries,
        max_hops: 3,
    };
    let plan = build_hop_plan(&input).expect("hop plan should build");

    let mut executor = DeterministicExecutor::new(vec![
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/0".to_string()],
            10,
            1,
            1,
        )),
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/1".to_string()],
            10,
            1,
            1,
        )),
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/2".to_string()],
            10,
            1,
            1,
        )),
        Ok(success_output(
            "BraveWebSearch",
            vec!["https://example.com/unused".to_string()],
            10,
            1,
            1,
        )),
    ]);
    let budget = HopBudget::for_tier(ImportanceTier::High);
    let result = execute_hop_plan(&plan, budget, &mut executor).expect("run should pass");

    assert_eq!(result.hop_records.len(), result.proof_chain.hops_executed);
    assert_eq!(result.hop_records.len(), executor.executed_count());
    assert!(result.hop_records.len() <= plan.hops.len());
}

#[test]
fn test_budget_tracker_guard_rails_are_deterministic() {
    let budget = HopBudget::for_tier(ImportanceTier::Low);
    let mut tracker = HopBudgetTracker::new(budget);
    tracker.check_hop_start(0).expect("hop 0 should be allowed");
    tracker
        .record_hop_usage(10, 1, 1)
        .expect("first usage should be accepted");
    let err = tracker
        .record_hop_usage(budget.max_time_per_hop_ms + 1, 0, 0)
        .expect_err("per-hop timeout must be enforced");
    assert_eq!(err.reason_code, "timeout_exceeded");
}

#[test]
fn test_fixture_json_shape_is_valid() {
    let raw = fs::read_to_string(fixture_dir().join("simple_chain.json")).expect("fixture load");
    let value: Value = serde_json::from_str(&raw).expect("fixture parse");
    assert!(value.get("root_query").is_some());
}
