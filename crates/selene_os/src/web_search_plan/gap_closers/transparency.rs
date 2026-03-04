#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeSet;

pub const TRACE_REPORT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReasoningTag {
    DirectQuote,
    MultiSourceAgree,
    OfficialSource,
    ComputedAggregate,
    ConflictPresent,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimTrace {
    pub claim_index: usize,
    pub claim_text: String,
    pub citations: Vec<String>,
    pub reasoning_tags: Vec<ReasoningTag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TraceReport {
    pub trace_report_version: String,
    pub claims: Vec<ClaimTrace>,
}

pub fn build_trace_report(
    synthesis_packet: &Value,
    evidence_packet: &Value,
    computation_packet: Option<&Value>,
    comparison_packet: Option<&Value>,
    temporal_packet: Option<&Value>,
    risk_packet: Option<&Value>,
) -> Result<TraceReport, String> {
    let claims = synthesis_packet
        .get("bullet_evidence")
        .and_then(Value::as_array)
        .ok_or_else(|| "synthesis packet missing bullet_evidence".to_string())?;
    let synthesis_citations = synthesis_packet
        .get("citations")
        .and_then(Value::as_array)
        .ok_or_else(|| "synthesis packet missing citations".to_string())?;
    let reason_codes = synthesis_packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let uncertainty_flags = synthesis_packet
        .get("uncertainty_flags")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let known_refs = collect_known_evidence_refs(evidence_packet);
    let ordered_citations = ordered_valid_citations(synthesis_citations, &known_refs);

    let supports_computed_context =
        computation_packet.is_some() || comparison_packet.is_some() || temporal_packet.is_some() || risk_packet.is_some();

    let has_conflict = reason_codes
        .iter()
        .filter_map(Value::as_str)
        .any(|code| code == "conflicting_evidence_detected")
        || uncertainty_flags
            .iter()
            .filter_map(Value::as_str)
            .any(|flag| flag == "conflicting_evidence_detected");

    let mut output_claims = Vec::with_capacity(claims.len());
    for (index, entry) in claims.iter().enumerate() {
        let claim_text = entry
            .as_str()
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .ok_or_else(|| format!("bullet_evidence entry {} must be non-empty string", index))?
            .to_string();

        let mut tags = Vec::new();
        if ordered_citations.is_empty() {
            tags.push(ReasoningTag::Unknown);
        } else {
            tags.push(ReasoningTag::DirectQuote);
            if ordered_citations.len() > 1 {
                tags.push(ReasoningTag::MultiSourceAgree);
            }
            if ordered_citations.iter().any(|citation| is_official_citation(citation)) {
                tags.push(ReasoningTag::OfficialSource);
            }
            if supports_computed_context && claim_contains_numeric_signal(claim_text.as_str()) {
                tags.push(ReasoningTag::ComputedAggregate);
            }
            if has_conflict {
                tags.push(ReasoningTag::ConflictPresent);
            }
        }

        output_claims.push(ClaimTrace {
            claim_index: index,
            claim_text,
            citations: ordered_citations.clone(),
            reasoning_tags: dedup_reasoning_tags(tags),
        });
    }

    Ok(TraceReport {
        trace_report_version: TRACE_REPORT_VERSION.to_string(),
        claims: output_claims,
    })
}

pub fn trace_report_to_json(report: &TraceReport) -> Value {
    serde_json::to_value(report).unwrap_or(Value::Null)
}

fn ordered_valid_citations(citations: &[Value], known_refs: &BTreeSet<String>) -> Vec<String> {
    let mut ordered = Vec::new();
    let mut seen = BTreeSet::new();
    for entry in citations {
        let Some(value) = entry.get("value").and_then(Value::as_str) else {
            continue;
        };
        if !known_refs.contains(value) {
            continue;
        }
        if seen.insert(value.to_string()) {
            ordered.push(value.to_string());
        }
    }
    ordered
}

fn collect_known_evidence_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                refs.insert(url.to_string());
            }
            if let Some(canonical) = source.get("canonical_url").and_then(Value::as_str) {
                refs.insert(canonical.to_string());
            }
        }
    }

    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                refs.insert(chunk_id.to_string());
            }
            if let Some(source_url) = chunk.get("source_url").and_then(Value::as_str) {
                refs.insert(source_url.to_string());
            }
        }
    }

    refs
}

fn dedup_reasoning_tags(tags: Vec<ReasoningTag>) -> Vec<ReasoningTag> {
    let mut out = Vec::new();
    for tag in tags {
        if !out.contains(&tag) {
            out.push(tag);
        }
    }
    out
}

fn is_official_citation(citation: &str) -> bool {
    let lower = citation.to_ascii_lowercase();
    lower.contains(".gov")
        || lower.contains(".edu")
        || lower.contains("sec.gov")
        || lower.contains("europa.eu")
        || lower.contains("who.int")
}

fn claim_contains_numeric_signal(claim: &str) -> bool {
    let lower = claim.to_ascii_lowercase();
    lower.contains('%')
        || lower.contains("percent")
        || lower.contains("average")
        || lower.contains("median")
        || claim.chars().any(|ch| ch.is_ascii_digit())
}
