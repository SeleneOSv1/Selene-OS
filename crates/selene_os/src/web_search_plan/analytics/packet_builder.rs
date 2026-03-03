#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::aggregates::compute_aggregates;
use crate::web_search_plan::analytics::confidence::build_confidence;
use crate::web_search_plan::analytics::consensus::build_consensus;
use crate::web_search_plan::analytics::currency_normalize::CurrencyRateTable;
use crate::web_search_plan::analytics::decimal::{
    decimal_to_numeric_value, structured_value_to_decimal,
};
use crate::web_search_plan::analytics::types::{
    AnalyticsError, AnalyticsErrorKind, AnalyticsRequest, ComputationInputs, ComputationPacket,
    NumericSample, ANALYTICS_ENGINE_ID, ANALYTICS_SCHEMA_VERSION,
};
use crate::web_search_plan::analytics::unit_normalize::UnitConversionTable;
use crate::web_search_plan::replay::snapshot::hash_canonical_json;
use crate::web_search_plan::structured::normalize::sort_rows_deterministically;
use crate::web_search_plan::structured::types::StructuredValue;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_computation_packet(
    mut request: AnalyticsRequest,
) -> Result<ComputationPacket, AnalyticsError> {
    if request.trace_id.trim().is_empty() {
        return Err(AnalyticsError::new(
            AnalyticsErrorKind::InvalidInput,
            "analytics trace_id is required",
        ));
    }
    if request.policy_snapshot_id.trim().is_empty() {
        return Err(AnalyticsError::new(
            AnalyticsErrorKind::InvalidInput,
            "analytics policy_snapshot_id is required",
        ));
    }

    sort_rows_deterministically(&mut request.structured_rows);

    let evidence_hash = hash_canonical_json(&request.evidence_packet).map_err(|error| {
        AnalyticsError::new(
            AnalyticsErrorKind::PolicyViolation,
            format!("failed computing evidence hash: {}", error),
        )
    })?;

    let known_refs = known_source_refs(&request.evidence_packet);
    let trust_weights = trust_weights_by_source_url(&request.evidence_packet);

    let mut samples = Vec::new();
    for row in &request.structured_rows {
        let Some(decimal_value) = structured_value_to_decimal(&row.value) else {
            continue;
        };

        if !known_refs.contains(row.source_ref.as_str()) {
            return Err(AnalyticsError::new(
                AnalyticsErrorKind::CitationMismatch,
                format!(
                    "source_ref {} is not present in evidence packet references",
                    row.source_ref
                ),
            ));
        }

        let source_url_key = row.source_url.to_ascii_lowercase();
        let trust_weight = trust_weights
            .get(source_url_key.as_str())
            .copied()
            .unwrap_or(1);

        let currency = match &row.value {
            StructuredValue::Currency { currency_code, .. } => {
                Some(currency_code.to_ascii_uppercase())
            }
            _ => None,
        };

        let unit = row.unit.as_ref().map(|value| value.to_ascii_lowercase());

        samples.push(NumericSample {
            metric_id: metric_id(row.entity.as_str(), row.attribute.as_str()),
            entity: row.entity.clone(),
            attribute: row.attribute.clone(),
            unit,
            currency,
            as_of_ms: row.as_of_ms,
            source_ref: row.source_ref.clone(),
            source_url: row.source_url.clone(),
            value_repr: decimal_to_numeric_value(decimal_value),
            trust_weight,
            value_decimal: decimal_value,
        });
    }

    if samples.is_empty() {
        return Err(AnalyticsError::new(
            AnalyticsErrorKind::InsufficientEvidence,
            "no numeric structured rows available for analytics",
        ));
    }

    let unit_table = UnitConversionTable::from_evidence_packet(&request.evidence_packet);
    let currency_table = CurrencyRateTable::from_evidence_packet(&request.evidence_packet);

    let aggregate_result = compute_aggregates(samples, &unit_table, &currency_table);
    if aggregate_result.aggregates.is_empty() {
        return Err(AnalyticsError::new(
            AnalyticsErrorKind::InsufficientEvidence,
            "numeric aggregation produced no outputs",
        ));
    }

    let consensus_result = build_consensus(aggregate_result.groups.as_slice());
    let confidence = build_confidence(
        aggregate_result.aggregates.as_slice(),
        aggregate_result.groups.as_slice(),
        consensus_result.groups.as_slice(),
        request.as_of_ms,
    );

    let mut reason_codes = BTreeSet::new();
    for code in aggregate_result.reason_codes {
        reason_codes.insert(code);
    }
    for code in consensus_result.reason_codes {
        reason_codes.insert(code);
    }

    Ok(ComputationPacket {
        schema_version: ANALYTICS_SCHEMA_VERSION.to_string(),
        produced_by: ANALYTICS_ENGINE_ID.to_string(),
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
        inputs: ComputationInputs {
            evidence_hash,
            policy_snapshot_id: request.policy_snapshot_id,
            as_of_ms: request.as_of_ms,
        },
        aggregates: aggregate_result.aggregates,
        consensus: consensus_result.groups,
        confidence,
        reason_codes: reason_codes.into_iter().collect(),
    })
}

fn metric_id(entity: &str, attribute: &str) -> String {
    format!(
        "{}.{}",
        normalize_identifier(entity),
        normalize_identifier(attribute)
    )
}

fn normalize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

fn known_source_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                refs.insert(url.to_string());
            }
        }
    }

    if let Some(chunks) = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
    {
        for chunk in chunks {
            if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                refs.insert(chunk_id.to_string());
            }
        }
    }

    refs
}

fn trust_weights_by_source_url(evidence_packet: &Value) -> BTreeMap<String, u32> {
    let mut weights = BTreeMap::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            let Some(url) = source.get("url").and_then(Value::as_str) else {
                continue;
            };
            let trust_tier = source
                .get("trust_tier")
                .and_then(Value::as_str)
                .unwrap_or("medium");
            weights.insert(url.to_ascii_lowercase(), trust_weight(trust_tier));
        }
    }

    weights
}

fn trust_weight(trust_tier: &str) -> u32 {
    match trust_tier.to_ascii_lowercase().as_str() {
        "high" | "tier_1" => 3,
        "medium" | "tier_2" => 2,
        "low" | "tier_3" => 1,
        _ => 1,
    }
}
