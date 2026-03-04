#![forbid(unsafe_code)]

use crate::web_search_plan::realtime::freshness::evaluate as evaluate_realtime_freshness;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const FRESHNESS_WATCHDOG_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryFreshnessClass {
    Realtime,
    Recent,
    Evergreen,
}

impl QueryFreshnessClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Realtime => "realtime",
            Self::Recent => "recent",
            Self::Evergreen => "evergreen",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StaleCitation {
    pub citation_ref: String,
    pub age_ms: i64,
    pub threshold_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FreshnessWatchdogReport {
    pub watchdog_version: String,
    pub query_class: String,
    pub threshold_ms: i64,
    pub refresh_required: bool,
    pub stale_citations: Vec<StaleCitation>,
}

pub fn evaluate_freshness_watchdog(
    query: &str,
    importance_tier: &str,
    retrieved_at_ms: i64,
    sources: &[Value],
) -> FreshnessWatchdogReport {
    let query_class = classify_query_freshness_class(query);
    let threshold_ms = stale_threshold_ms(query_class, importance_tier);

    let mut stale_citations = Vec::new();
    for source in sources {
        let citation_ref = source
            .get("canonical_url")
            .and_then(Value::as_str)
            .or_else(|| source.get("url").and_then(Value::as_str))
            .unwrap_or("unknown")
            .to_string();

        let source_timestamp_ms = source
            .get("published_at")
            .and_then(Value::as_i64)
            .or_else(|| source.get("retrieved_at_ms").and_then(Value::as_i64));
        let Some(source_timestamp_ms) = source_timestamp_ms else {
            continue;
        };

        let Ok(assessment) =
            evaluate_realtime_freshness(retrieved_at_ms, source_timestamp_ms, threshold_ms as u64)
        else {
            continue;
        };
        if assessment.stale {
            stale_citations.push(StaleCitation {
                citation_ref,
                age_ms: assessment.age_ms,
                threshold_ms,
            });
        }
    }

    stale_citations.sort_by(|left, right| {
        (left.citation_ref.as_str(), left.age_ms).cmp(&(right.citation_ref.as_str(), right.age_ms))
    });

    FreshnessWatchdogReport {
        watchdog_version: FRESHNESS_WATCHDOG_VERSION.to_string(),
        query_class: query_class.as_str().to_string(),
        threshold_ms,
        refresh_required: !stale_citations.is_empty(),
        stale_citations,
    }
}

pub fn append_watchdog_audit_metadata(
    audit_packet: &mut Value,
    report: &FreshnessWatchdogReport,
) -> Result<(), String> {
    let root = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_entry = root
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let transition_obj = if transition_entry.is_object() {
        transition_entry
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(raw) = transition_entry.as_str() {
        *transition_entry = json!({ "state_path": raw });
        transition_entry
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    let gap_entry = transition_obj
        .entry("gap_closers".to_string())
        .or_insert_with(|| Value::Object(Map::new()));
    let gap_obj = gap_entry
        .as_object_mut()
        .ok_or_else(|| "gap_closers metadata must be object".to_string())?;
    gap_obj.insert(
        "freshness_watchdog".to_string(),
        serde_json::to_value(report).map_err(|err| err.to_string())?,
    );
    Ok(())
}

pub fn report_to_json(report: &FreshnessWatchdogReport) -> Value {
    serde_json::to_value(report).unwrap_or(Value::Null)
}

pub fn classify_query_freshness_class(query: &str) -> QueryFreshnessClass {
    let normalized = query.to_ascii_lowercase();
    if contains_any(
        normalized.as_str(),
        &[
            "today",
            "latest",
            "current",
            "now",
            "stock",
            "price",
            "weather",
            "flight",
            "breaking",
            "live",
        ],
    ) {
        return QueryFreshnessClass::Realtime;
    }
    if contains_any(
        normalized.as_str(),
        &["recent", "this week", "new update", "last week"],
    ) {
        return QueryFreshnessClass::Recent;
    }
    QueryFreshnessClass::Evergreen
}

fn stale_threshold_ms(class: QueryFreshnessClass, importance_tier: &str) -> i64 {
    let tier = importance_tier.trim().to_ascii_lowercase();
    match class {
        QueryFreshnessClass::Realtime => match tier.as_str() {
            "high" => 15 * 60 * 1000,
            "low" => 60 * 60 * 1000,
            _ => 30 * 60 * 1000,
        },
        QueryFreshnessClass::Recent => match tier.as_str() {
            "high" => 24 * 60 * 60 * 1000,
            "low" => 7 * 24 * 60 * 60 * 1000,
            _ => 3 * 24 * 60 * 60 * 1000,
        },
        QueryFreshnessClass::Evergreen => match tier.as_str() {
            "high" => 30 * 24 * 60 * 60 * 1000,
            "low" => 90 * 24 * 60 * 60 * 1000,
            _ => 60 * 24 * 60 * 60 * 1000,
        },
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}
