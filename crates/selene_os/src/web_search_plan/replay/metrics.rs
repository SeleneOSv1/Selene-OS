#![forbid(unsafe_code)]

use crate::web_search_plan::eval::metrics::citation_coverage_ratio;
use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier};
use crate::web_search_plan::replay::corpus::ReplayCase;
use crate::web_search_plan::replay::snapshot::ReplaySnapshot;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayMetrics {
    pub citation_coverage_ratio: f64,
    pub refusal_correctness: bool,
    pub latency_budget_compliance: bool,
    pub determinism_ok: bool,
}

pub fn compute_quality_metrics(
    case: &ReplayCase,
    synthesis_packet: Option<&Value>,
    simulated_latency_ms: u64,
    snapshot: &ReplaySnapshot,
    determinism_ok: bool,
) -> Result<ReplayMetrics, String> {
    let citation_coverage_ratio = citation_coverage_ratio(synthesis_packet)?;

    let tier = ImportanceTier::parse(case.importance_tier.as_str())
        .map_err(|e| format!("case {} invalid tier: {}", case.case_id, e))?;
    let tier_caps = caps_for_tier(tier);
    let latency_budget_compliance = simulated_latency_ms <= tier_caps.total_timeout_per_turn_ms;

    let refusal_expected = case.expected_outcome == "refusal";
    let refusal_observed = snapshot
        .reason_codes
        .iter()
        .any(|code| code == "insufficient_evidence");

    let refusal_correctness = if refusal_expected {
        refusal_observed
    } else {
        !refusal_observed && (citation_coverage_ratio - 1.0).abs() < f64::EPSILON
    };

    Ok(ReplayMetrics {
        citation_coverage_ratio,
        refusal_correctness,
        latency_budget_compliance,
        determinism_ok,
    })
}
