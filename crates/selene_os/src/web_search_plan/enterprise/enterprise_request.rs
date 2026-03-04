#![forbid(unsafe_code)]

use crate::web_search_plan::enterprise::mode_router::{route_mode, EnterpriseMode};
use crate::web_search_plan::merge::InternalContext;
use crate::web_search_plan::structured::types::StructuredRow;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnterpriseConstraints {
    pub max_hops: usize,
    pub max_total_time_ms: u64,
    pub max_provider_calls_total: usize,
    pub max_url_opens_total: usize,
    pub require_regulatory_filter: bool,
}

impl Default for EnterpriseConstraints {
    fn default() -> Self {
        Self {
            max_hops: 5,
            max_total_time_ms: 15_000,
            max_provider_calls_total: 16,
            max_url_opens_total: 8,
            require_regulatory_filter: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnterpriseRequest {
    pub trace_id: String,
    pub query: String,
    pub mode: EnterpriseMode,
    pub importance_tier: String,
    pub created_at_ms: i64,
    pub policy_snapshot_id: String,
    pub jurisdiction: Option<String>,
    pub as_of_from_ms: Option<i64>,
    pub as_of_to_ms: Option<i64>,
    pub constraints: EnterpriseConstraints,
    pub target_entity: Option<String>,
    pub tool_request_packet: Option<Value>,
    pub evidence_packet: Option<Value>,
    pub structured_rows: Option<Vec<StructuredRow>>,
    pub computation_packet: Option<Value>,
    pub internal_context: Option<InternalContext>,
}

impl EnterpriseRequest {
    pub fn parse_from_tool_request(tool_request: &Value) -> Result<Self, String> {
        let obj = tool_request
            .as_object()
            .ok_or_else(|| "enterprise tool request must be object".to_string())?;
        let trace_id = obj
            .get("trace_id")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "enterprise trace_id is required".to_string())?
            .to_string();
        let query = obj
            .get("query")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .ok_or_else(|| "enterprise query is required".to_string())?
            .to_string();

        let explicit_mode = obj.get("enterprise_mode").and_then(Value::as_str);
        let mode = route_mode(query.as_str(), explicit_mode)?;
        let importance_tier = obj
            .get("importance_tier")
            .and_then(Value::as_str)
            .unwrap_or("medium")
            .to_ascii_lowercase();
        if !matches!(importance_tier.as_str(), "low" | "medium" | "high") {
            return Err(format!("unsupported importance_tier {}", importance_tier));
        }

        let created_at_ms = obj
            .get("created_at_ms")
            .and_then(Value::as_i64)
            .unwrap_or(0);
        let policy_snapshot_id = obj
            .get("policy_snapshot_id")
            .and_then(Value::as_str)
            .unwrap_or("policy-snapshot-default")
            .to_string();
        let jurisdiction = obj
            .get("jurisdiction")
            .and_then(Value::as_str)
            .map(|entry| entry.trim().to_string())
            .filter(|entry| !entry.is_empty());
        let as_of_from_ms = obj.get("as_of_from_ms").and_then(Value::as_i64);
        let as_of_to_ms = obj.get("as_of_to_ms").and_then(Value::as_i64);

        let constraints = parse_constraints(obj.get("constraints"));
        let target_entity = obj
            .get("target_entity")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(ToString::to_string);

        Ok(Self {
            trace_id,
            query,
            mode,
            importance_tier,
            created_at_ms,
            policy_snapshot_id,
            jurisdiction,
            as_of_from_ms,
            as_of_to_ms,
            constraints,
            target_entity,
            tool_request_packet: Some(tool_request.clone()),
            evidence_packet: None,
            structured_rows: None,
            computation_packet: None,
            internal_context: None,
        })
    }

    pub fn to_regulatory_tool_request(&self) -> Value {
        if let Some(existing) = self.tool_request_packet.as_ref() {
            return existing.clone();
        }
        json!({
            "trace_id": self.trace_id,
            "query": self.query,
            "created_at_ms": self.created_at_ms,
            "importance_tier": self.importance_tier,
            "jurisdiction": self.jurisdiction,
            "budgets": {
                "regulatory_mode": true
            }
        })
    }
}

fn parse_constraints(raw: Option<&Value>) -> EnterpriseConstraints {
    let mut constraints = EnterpriseConstraints::default();
    let Some(value) = raw.and_then(Value::as_object) else {
        return constraints;
    };

    if let Some(max_hops) = value.get("max_hops").and_then(Value::as_u64) {
        constraints.max_hops = max_hops as usize;
    }
    if let Some(total_ms) = value.get("max_total_time_ms").and_then(Value::as_u64) {
        constraints.max_total_time_ms = total_ms;
    }
    if let Some(calls) = value.get("max_provider_calls_total").and_then(Value::as_u64) {
        constraints.max_provider_calls_total = calls as usize;
    }
    if let Some(opens) = value.get("max_url_opens_total").and_then(Value::as_u64) {
        constraints.max_url_opens_total = opens as usize;
    }
    if let Some(required) = value
        .get("require_regulatory_filter")
        .and_then(Value::as_bool)
    {
        constraints.require_regulatory_filter = required;
    }
    constraints
}
