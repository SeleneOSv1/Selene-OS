#![forbid(unsafe_code)]

pub mod aggregates;
pub mod confidence;
pub mod consensus;
pub mod currency_normalize;
pub mod decimal;
pub mod packet_builder;
pub mod types;
pub mod unit_normalize;

use crate::ph1comp::Ph1CompRuntime;
use crate::web_search_plan::analytics::types::{
    AnalyticsError, AnalyticsErrorKind, AnalyticsRequest, ComputationPacket,
};
use crate::web_search_plan::structured::types::StructuredRow;
use serde_json::Value;

pub fn run_numeric_consensus(
    trace_id: impl Into<String>,
    created_at_ms: i64,
    policy_snapshot_id: impl Into<String>,
    as_of_ms: Option<i64>,
    evidence_packet: Value,
    structured_rows: Vec<StructuredRow>,
) -> Result<ComputationPacket, AnalyticsError> {
    Ph1CompRuntime.build_analytics_computation_packet(AnalyticsRequest {
        trace_id: trace_id.into(),
        created_at_ms,
        intended_consumers: vec![
            "PH1.D".to_string(),
            "PH1.WRITE".to_string(),
            "PH1.LAW".to_string(),
            "PH1.J".to_string(),
        ],
        policy_snapshot_id: policy_snapshot_id.into(),
        as_of_ms,
        evidence_packet,
        structured_rows,
    })
}

pub fn structured_rows_from_evidence(
    evidence_packet: &Value,
) -> Result<Vec<StructuredRow>, AnalyticsError> {
    let raw = evidence_packet
        .pointer("/trust_metadata/structured/rows")
        .cloned()
        .ok_or_else(|| {
            AnalyticsError::new(
                AnalyticsErrorKind::InsufficientEvidence,
                "evidence packet does not include trust_metadata.structured.rows",
            )
        })?;

    serde_json::from_value(raw).map_err(|error| {
        AnalyticsError::new(
            AnalyticsErrorKind::PolicyViolation,
            format!("failed parsing structured rows from evidence: {}", error),
        )
    })
}

#[cfg(test)]
pub mod analytics_tests;
