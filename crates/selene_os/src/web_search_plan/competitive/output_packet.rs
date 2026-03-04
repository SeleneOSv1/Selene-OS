#![forbid(unsafe_code)]

use crate::web_search_plan::competitive::schema::{
    CompetitiveComparison, CompetitiveComparisonPacket, CompetitiveRequest, COMPETITIVE_ENGINE_ID,
    COMPETITIVE_SCHEMA_VERSION,
};

pub fn build_comparison_packet(
    request: &CompetitiveRequest,
    comparison: CompetitiveComparison,
) -> CompetitiveComparisonPacket {
    let intended_consumers = if request.intended_consumers.is_empty() {
        vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.J".to_string(),
        ]
    } else {
        request.intended_consumers.clone()
    };

    CompetitiveComparisonPacket {
        schema_version: COMPETITIVE_SCHEMA_VERSION.to_string(),
        produced_by: COMPETITIVE_ENGINE_ID.to_string(),
        intended_consumers,
        created_at_ms: request.created_at_ms,
        trace_id: request.trace_id.clone(),
        target_entity: comparison.target_entity,
        competitors: comparison.competitors,
        pricing_table: comparison.pricing_table,
        feature_matrix: comparison.feature_matrix,
        position_summary: comparison.position_summary,
        swot: comparison.swot,
        uncertainty_flags: comparison.uncertainty_flags,
        reason_codes: comparison.reason_codes,
        source_refs: comparison.source_refs,
    }
}
