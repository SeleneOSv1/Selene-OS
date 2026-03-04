#![forbid(unsafe_code)]

use crate::web_search_plan::merge::conflict::ConflictReport;
use crate::web_search_plan::merge::delta::{DeltaChange, ExternalFinding};
use serde_json::Value;
use std::collections::BTreeSet;

pub fn collect_evidence_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                let trimmed = url.trim();
                if !trimmed.is_empty() {
                    refs.insert(trimmed.to_string());
                }
            }
        }
    }

    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                let trimmed = chunk_id.trim();
                if !trimmed.is_empty() {
                    refs.insert(trimmed.to_string());
                }
            }
            if let Some(source_url) = chunk.get("source_url").and_then(Value::as_str) {
                let trimmed = source_url.trim();
                if !trimmed.is_empty() {
                    refs.insert(trimmed.to_string());
                }
            }
        }
    }

    refs
}

pub fn enforce_evidence_supremacy(
    allowed_refs: &BTreeSet<String>,
    external_findings: &[ExternalFinding],
    delta_changes: &[DeltaChange],
    conflict_report: Option<&ConflictReport>,
) -> Result<(), String> {
    for finding in external_findings {
        if finding.citations.is_empty() {
            return Err(format!(
                "external finding '{}' has no citations",
                finding.statement
            ));
        }
        validate_citations(allowed_refs, finding.citations.as_slice())?;
    }

    for change in delta_changes {
        if change.new_statement.trim().is_empty() {
            return Err(format!(
                "delta item {} has empty new_statement",
                change.topic_key
            ));
        }
        if change.citations.is_empty() {
            return Err(format!(
                "delta item {} has no evidence citations",
                change.topic_key
            ));
        }
        validate_citations(allowed_refs, change.citations.as_slice())?;
    }

    if let Some(report) = conflict_report {
        for conflict in &report.conflicts {
            if conflict.evidence_citations.is_empty() {
                return Err(format!(
                    "conflict item {} has no evidence citations",
                    conflict.topic_key
                ));
            }
            validate_citations(allowed_refs, conflict.evidence_citations.as_slice())?;
        }
    }

    Ok(())
}

fn validate_citations(allowed_refs: &BTreeSet<String>, citations: &[String]) -> Result<(), String> {
    for citation in citations {
        if !allowed_refs.contains(citation.as_str()) {
            return Err(format!(
                "citation {} is not present in EvidencePacket source refs",
                citation
            ));
        }
    }
    Ok(())
}
