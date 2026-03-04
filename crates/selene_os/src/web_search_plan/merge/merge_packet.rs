#![forbid(unsafe_code)]

use crate::web_search_plan::merge::boundary::{collect_evidence_refs, enforce_evidence_supremacy};
use crate::web_search_plan::merge::conflict::{build_conflict_report, ConflictReport};
use crate::web_search_plan::merge::delta::{
    build_delta, extract_external_findings, DeltaChange, ExternalFinding,
};
use crate::web_search_plan::merge::internal_context::{
    build_internal_view, internal_context_used, InternalContext, InternalView,
};
use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;

pub const MERGE_SCHEMA_VERSION: &str = "1.0.0";
pub const MERGE_ENGINE_ID: &str = "PH1.MERGE";
pub const MERGE_TEMPLATE_VERSION: &str = "1.0.0";

#[derive(Debug, Clone)]
pub struct MergeRequest {
    pub trace_id: String,
    pub created_at_ms: i64,
    pub intended_consumers: Vec<String>,
    pub policy_snapshot_id: String,
    pub evidence_packet: Value,
    pub internal_context: Option<InternalContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExternalView {
    pub top_findings: Vec<ExternalFinding>,
    pub retrieved_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaView {
    pub changes_since_last_time: Vec<DeltaChange>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergePacket {
    pub schema_version: String,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub created_at_ms: i64,
    pub trace_id: String,
    pub policy_snapshot_id: String,
    pub evidence_hash: String,
    pub internal_context_hash: String,
    pub merge_template_version: String,
    pub internal_view: InternalView,
    pub external_view: ExternalView,
    pub delta: DeltaView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_report: Option<ConflictReport>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeBuildOutput {
    pub packet: MergePacket,
    pub internal_context_used: bool,
    pub conflicts_detected_count: usize,
    pub changes_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MergeBuildError {
    pub reason_code: &'static str,
    pub message: String,
}

impl MergeBuildError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

pub fn run_internal_external_merge(request: MergeRequest) -> Result<MergePacket, MergeBuildError> {
    build_merge_packet(request).map(|output| output.packet)
}

pub fn build_merge_packet(request: MergeRequest) -> Result<MergeBuildOutput, MergeBuildError> {
    if request.trace_id.trim().is_empty() {
        return Err(MergeBuildError::new(
            "policy_violation",
            "merge trace_id is required",
        ));
    }
    if request.policy_snapshot_id.trim().is_empty() {
        return Err(MergeBuildError::new(
            "policy_violation",
            "merge policy_snapshot_id is required",
        ));
    }
    if !request.evidence_packet.is_object() {
        return Err(MergeBuildError::new(
            "insufficient_evidence",
            "merge requires object-shaped EvidencePacket input",
        ));
    }

    let internal_view = build_internal_view(request.internal_context.as_ref());
    let internal_context_used = internal_context_used(&internal_view);

    let raw_internal_context = match request.internal_context.as_ref() {
        Some(context) => serde_json::to_value(context).map_err(|error| {
            MergeBuildError::new(
                "policy_violation",
                format!("failed serializing internal context: {}", error),
            )
        })?,
        None => Value::Null,
    };

    let internal_context_hash = hash_canonical_json(&raw_internal_context).map_err(|error| {
        MergeBuildError::new(
            "policy_violation",
            format!("failed computing internal context hash: {}", error),
        )
    })?;
    let evidence_hash = hash_canonical_json(&request.evidence_packet).map_err(|error| {
        MergeBuildError::new(
            "policy_violation",
            format!("failed computing evidence hash: {}", error),
        )
    })?;

    let allowed_refs = collect_evidence_refs(&request.evidence_packet);
    let external_findings = extract_external_findings(&request.evidence_packet, &allowed_refs)
        .map_err(|message| MergeBuildError::new("policy_violation", message))?;
    let delta_result = build_delta(&internal_view.prior_key_points, external_findings.as_slice());
    let conflict_report =
        build_conflict_report(delta_result.changes.as_slice(), external_findings.as_slice());

    enforce_evidence_supremacy(
        &allowed_refs,
        external_findings.as_slice(),
        delta_result.changes.as_slice(),
        conflict_report.as_ref(),
    )
    .map_err(|message| MergeBuildError::new("policy_violation", message))?;

    let mut reason_codes = BTreeSet::new();
    if external_findings.is_empty() {
        reason_codes.insert("insufficient_evidence".to_string());
    }
    for reason_code in delta_result.reason_codes {
        reason_codes.insert(reason_code);
    }
    if conflict_report
        .as_ref()
        .map(|report| !report.conflicts.is_empty())
        .unwrap_or(false)
    {
        reason_codes.insert("conflicting_evidence_detected".to_string());
    }

    let retrieved_at_ms = request
        .evidence_packet
        .get("retrieved_at_ms")
        .and_then(Value::as_i64)
        .unwrap_or(request.created_at_ms);

    let packet = MergePacket {
        schema_version: MERGE_SCHEMA_VERSION.to_string(),
        produced_by: MERGE_ENGINE_ID.to_string(),
        intended_consumers: if request.intended_consumers.is_empty() {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        } else {
            request.intended_consumers
        },
        created_at_ms: request.created_at_ms,
        trace_id: request.trace_id,
        policy_snapshot_id: request.policy_snapshot_id,
        evidence_hash,
        internal_context_hash,
        merge_template_version: MERGE_TEMPLATE_VERSION.to_string(),
        internal_view,
        external_view: ExternalView {
            top_findings: external_findings,
            retrieved_at_ms,
        },
        delta: DeltaView {
            changes_since_last_time: delta_result.changes,
        },
        conflict_report,
        reason_codes: reason_codes.into_iter().collect(),
    };

    validate_merge_packet(&packet)?;

    Ok(MergeBuildOutput {
        internal_context_used,
        conflicts_detected_count: packet
            .conflict_report
            .as_ref()
            .map(|report| report.conflicts.len())
            .unwrap_or(0),
        changes_count: packet.delta.changes_since_last_time.len(),
        packet,
    })
}

pub fn append_merge_audit_metadata(audit_packet: &Value, output: &MergeBuildOutput) -> Value {
    let mut root = match audit_packet {
        Value::Object(map) => map.clone(),
        _ => Map::new(),
    };

    let merge_metadata = json!({
        "internal_context_used": output.internal_context_used,
        "internal_context_hash": output.packet.internal_context_hash,
        "evidence_hash": output.packet.evidence_hash,
        "conflicts_detected_count": output.conflicts_detected_count,
        "changes_count": output.changes_count,
        "merge_template_version": output.packet.merge_template_version,
        "reason_codes": output.packet.reason_codes,
    });

    let merged_transition = root
        .get("turn_state_transition")
        .cloned()
        .map(|existing| match existing {
            Value::Object(mut map) => {
                map.insert("merge".to_string(), merge_metadata.clone());
                Value::Object(map)
            }
            Value::String(path) => json!({
                "state_path": path,
                "merge": merge_metadata.clone()
            }),
            _ => json!({
                "state_path": existing,
                "merge": merge_metadata.clone()
            }),
        })
        .unwrap_or_else(|| {
            json!({
                "merge": merge_metadata.clone()
            })
        });

    root.insert("turn_state_transition".to_string(), merged_transition);
    Value::Object(root)
}

fn validate_merge_packet(packet: &MergePacket) -> Result<(), MergeBuildError> {
    let registry = load_packet_schema_registry()
        .map_err(|error| MergeBuildError::new("policy_violation", error))?;
    let packet_value = serde_json::to_value(packet)
        .map_err(|error| MergeBuildError::new("policy_violation", error.to_string()))?;
    validate_packet("MergePacket", &packet_value, &registry)
        .map_err(|error| MergeBuildError::new("policy_violation", error))
}
