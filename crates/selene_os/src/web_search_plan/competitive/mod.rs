#![forbid(unsafe_code)]

pub mod compare;
pub mod entity_normalize;
pub mod feature_normalize;
pub mod output_packet;
pub mod pricing_normalize;
pub mod schema;
pub mod swot;

use crate::web_search_plan::competitive::compare::build_competitive_comparison;
use crate::web_search_plan::competitive::entity_normalize::build_entity_index;
use crate::web_search_plan::competitive::feature_normalize::build_feature_matrix;
use crate::web_search_plan::competitive::output_packet::build_comparison_packet;
use crate::web_search_plan::competitive::pricing_normalize::build_pricing_table;
use crate::web_search_plan::competitive::schema::{
    ComparisonPacket, CompetitiveError, CompetitiveErrorKind, CompetitiveRequest,
};
use crate::web_search_plan::structured::normalize::sort_rows_deterministically;
use serde_json::Value;
use std::collections::BTreeSet;

pub fn run_competitive_mode(
    mut request: CompetitiveRequest,
) -> Result<ComparisonPacket, CompetitiveError> {
    if request.trace_id.trim().is_empty() {
        return Err(CompetitiveError::new(
            CompetitiveErrorKind::PolicyViolation,
            "competitive request trace_id is required",
        ));
    }
    if request.target_entity.trim().is_empty() {
        return Err(CompetitiveError::new(
            CompetitiveErrorKind::InsufficientEvidence,
            "competitive request target_entity is required",
        ));
    }
    if request.structured_rows.is_empty() {
        return Err(CompetitiveError::new(
            CompetitiveErrorKind::InsufficientEvidence,
            "competitive request requires structured rows",
        ));
    }

    sort_rows_deterministically(&mut request.structured_rows);
    let allowed_refs = collect_allowed_source_refs(&request.evidence_packet);
    if allowed_refs.is_empty() {
        return Err(CompetitiveError::new(
            CompetitiveErrorKind::InsufficientEvidence,
            "evidence packet does not provide source refs for competitive mode",
        ));
    }

    let entity_index = build_entity_index(&request.structured_rows, request.target_entity.as_str());
    let pricing = build_pricing_table(
        &request.structured_rows,
        &entity_index,
        &allowed_refs,
        &request.evidence_packet,
    )?;
    let feature_matrix = build_feature_matrix(&request.structured_rows, &entity_index, &allowed_refs)?;

    let comparison = build_competitive_comparison(
        request.target_entity.as_str(),
        entity_index,
        pricing.pricing_table,
        feature_matrix,
        request.computation_packet.as_ref(),
        pricing.uncertainty_flags,
        pricing.reason_codes,
    );

    Ok(build_comparison_packet(&request, comparison))
}

pub fn parse_computation_packet(
    computation_packet_json: Option<&Value>,
) -> Result<Option<crate::web_search_plan::analytics::types::ComputationPacket>, CompetitiveError> {
    let Some(raw) = computation_packet_json else {
        return Ok(None);
    };
    serde_json::from_value(raw.clone())
        .map(Some)
        .map_err(|error| {
            CompetitiveError::new(
                CompetitiveErrorKind::PolicyViolation,
                format!("invalid computation packet payload: {}", error),
            )
        })
}

fn collect_allowed_source_refs(evidence_packet: &Value) -> BTreeSet<String> {
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
        }
    }
    refs
}

#[cfg(test)]
pub mod competitive_tests;
