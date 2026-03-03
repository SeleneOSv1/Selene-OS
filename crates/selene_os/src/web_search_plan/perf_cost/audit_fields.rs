#![forbid(unsafe_code)]

use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerfCostAuditMetrics {
    pub importance_tier: String,
    pub budgets_applied: Value,
    pub stage_timings_ms: BTreeMap<String, u64>,
    pub degraded: bool,
    pub degrade_step: Option<String>,
    pub concurrency_peak: usize,
    pub total_provider_calls: usize,
    pub urls_opened_count: usize,
    pub stop_reason: String,
}

pub fn append_perf_cost_audit_fields(
    audit_packet: &mut Value,
    metrics: &PerfCostAuditMetrics,
) -> Result<(), String> {
    let obj = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_value = obj
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    let transition_obj = if transition_value.is_object() {
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(state) = transition_value.as_str() {
        *transition_value = json!({"state": state});
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    let stage_timings_json: Map<String, Value> = metrics
        .stage_timings_ms
        .iter()
        .map(|(key, value)| (key.clone(), json!(value)))
        .collect();

    transition_obj.insert(
        "perf_cost_audit".to_string(),
        json!({
            "importance_tier": metrics.importance_tier,
            "budgets_applied": metrics.budgets_applied,
            "stage_timings_ms": stage_timings_json,
            "degraded": metrics.degraded,
            "degrade_step": metrics.degrade_step,
            "concurrency_peak": metrics.concurrency_peak,
            "total_provider_calls": metrics.total_provider_calls,
            "urls_opened_count": metrics.urls_opened_count,
            "stop_reason": metrics.stop_reason,
        }),
    );

    Ok(())
}
