#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::structured::types::StructuredRow;
use crate::web_search_plan::temporal::asof::{
    filter_rows_for_window, resolve_asof_windows, AsOfResolutionError, AsOfWindowInput,
    MissingTimestampPolicy,
};
use crate::web_search_plan::temporal::diff::{build_changes, ChangeItem};
use crate::web_search_plan::temporal::timeline::{build_timeline_events, TimelineEvent};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

pub const TEMPORAL_SCHEMA_VERSION: &str = "1.0.0";
pub const TEMPORAL_ENGINE_ID: &str = "PH1.ANALYTICS";

#[derive(Debug, Clone)]
pub struct TemporalRequest {
    pub trace_id: String,
    pub created_at_ms: i64,
    pub intended_consumers: Vec<String>,
    pub now_ms: i64,
    pub baseline_from_ms: Option<i64>,
    pub baseline_to_ms: Option<i64>,
    pub compare_from_ms: Option<i64>,
    pub compare_to_ms: Option<i64>,
    pub allow_default_windows: bool,
    pub policy_snapshot_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporalComparisonPacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub as_of_baseline_from_ms: i64,
    pub as_of_baseline_to_ms: i64,
    pub as_of_compare_from_ms: i64,
    pub as_of_compare_to_ms: i64,
    pub timeline_events: Vec<TimelineEvent>,
    pub changes: Vec<ChangeItem>,
    pub uncertainty_flags: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalBuildOutput {
    pub packet: TemporalComparisonPacket,
    pub unit_mismatch_count: usize,
    pub insufficient_evidence_flag: bool,
    pub policy_snapshot_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemporalBuildError {
    pub reason_code: &'static str,
    pub message: String,
}

impl TemporalBuildError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

pub fn build_temporal_comparison_packet(
    request: &TemporalRequest,
    evidence_packet: &Value,
    structured_rows: &[StructuredRow],
) -> Result<TemporalBuildOutput, TemporalBuildError> {
    let window_resolution = resolve_asof_windows(&AsOfWindowInput {
        baseline_from_ms: request.baseline_from_ms,
        baseline_to_ms: request.baseline_to_ms,
        compare_from_ms: request.compare_from_ms,
        compare_to_ms: request.compare_to_ms,
        now_ms: request.now_ms,
        allow_default_windows: request.allow_default_windows,
        default_window_ms: crate::web_search_plan::temporal::DEFAULT_WINDOW_MS,
    })
    .map_err(map_asof_error)?;

    let baseline_rows = filter_rows_for_window(
        structured_rows,
        window_resolution.baseline,
        MissingTimestampPolicy::Exclude,
    );
    let comparison_rows = filter_rows_for_window(
        structured_rows,
        window_resolution.comparison,
        MissingTimestampPolicy::Exclude,
    );

    let mut reason_codes = Vec::new();
    let mut uncertainty_flags = Vec::new();
    let mut insufficient_evidence_flag = false;

    if baseline_rows.rows.is_empty() || comparison_rows.rows.is_empty() {
        push_unique(&mut reason_codes, "insufficient_evidence");
        push_unique(&mut uncertainty_flags, "insufficient_evidence_window");
        insufficient_evidence_flag = true;
    }

    if baseline_rows.excluded_missing_timestamp_count > 0
        || comparison_rows.excluded_missing_timestamp_count > 0
    {
        push_unique(&mut uncertainty_flags, "missing_timestamps_excluded");
    }

    let timeline_rows = baseline_rows
        .rows
        .iter()
        .cloned()
        .chain(comparison_rows.rows.iter().cloned())
        .collect::<Vec<StructuredRow>>();
    let timeline_events = build_timeline_events(timeline_rows.as_slice(), evidence_packet, true);

    let diff = build_changes(baseline_rows.rows.as_slice(), comparison_rows.rows.as_slice());
    for reason_code in &diff.reason_codes {
        push_unique(&mut reason_codes, reason_code.as_str());
    }
    if diff.unit_mismatch_count > 0 {
        push_unique(&mut uncertainty_flags, "unit_mismatch_detected");
    }

    let packet = TemporalComparisonPacket {
        schema_version: TEMPORAL_SCHEMA_VERSION.to_string(),
        produced_by: TEMPORAL_ENGINE_ID.to_string(),
        intended_consumers: if request.intended_consumers.is_empty() {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        } else {
            request.intended_consumers.clone()
        },
        created_at_ms: request.created_at_ms,
        trace_id: request.trace_id.clone(),
        as_of_baseline_from_ms: window_resolution.baseline.from_ms,
        as_of_baseline_to_ms: window_resolution.baseline.to_ms,
        as_of_compare_from_ms: window_resolution.comparison.from_ms,
        as_of_compare_to_ms: window_resolution.comparison.to_ms,
        timeline_events,
        changes: diff.changes,
        uncertainty_flags,
        reason_codes,
    };

    validate_temporal_packet(&packet)?;

    Ok(TemporalBuildOutput {
        packet,
        unit_mismatch_count: diff.unit_mismatch_count,
        insufficient_evidence_flag,
        policy_snapshot_id: request.policy_snapshot_id.clone(),
    })
}

pub fn append_temporal_audit_metadata(
    audit_packet: &Value,
    output: &TemporalBuildOutput,
) -> Value {
    let mut root = match audit_packet {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };

    let temporal_metadata = json!({
        "as_of_baseline_window": {
            "from_ms": output.packet.as_of_baseline_from_ms,
            "to_ms": output.packet.as_of_baseline_to_ms
        },
        "as_of_compare_window": {
            "from_ms": output.packet.as_of_compare_from_ms,
            "to_ms": output.packet.as_of_compare_to_ms
        },
        "changes_count": output.packet.changes.len(),
        "modified_count": output.packet.changes.iter().filter(|change| change.change_type.as_str() == "modified").count(),
        "unit_mismatch_count": output.unit_mismatch_count,
        "insufficient_evidence_flag": output.insufficient_evidence_flag
    });

    let merged_transition = root
        .get("turn_state_transition")
        .cloned()
        .map(|existing| match existing {
            Value::Object(mut map) => {
                map.insert("temporal".to_string(), temporal_metadata.clone());
                Value::Object(map)
            }
            Value::String(path) => json!({
                "state_path": path,
                "temporal": temporal_metadata.clone()
            }),
            _ => json!({
                "state_path": existing,
                "temporal": temporal_metadata.clone()
            }),
        })
        .unwrap_or_else(|| {
            json!({
                "temporal": temporal_metadata.clone()
            })
        });

    root.insert("turn_state_transition".to_string(), merged_transition);
    Value::Object(root)
}

fn validate_temporal_packet(packet: &TemporalComparisonPacket) -> Result<(), TemporalBuildError> {
    let registry = load_packet_schema_registry()
        .map_err(|err| TemporalBuildError::new("policy_violation", err))?;
    let value = serde_json::to_value(packet)
        .map_err(|err| TemporalBuildError::new("policy_violation", err.to_string()))?;
    validate_packet("ComparisonPacket", &value, &registry)
        .map_err(|err| TemporalBuildError::new("policy_violation", err))
}

fn map_asof_error(error: AsOfResolutionError) -> TemporalBuildError {
    TemporalBuildError::new(error.reason_code, error.message)
}

fn push_unique(entries: &mut Vec<String>, value: &str) {
    if !entries.iter().any(|entry| entry == value) {
        entries.push(value.to_string());
    }
}
