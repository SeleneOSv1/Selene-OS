#![forbid(unsafe_code)]

use crate::web_search_plan::gap_closers::transparency::TraceReport;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const CLAIM_CONFIDENCE_VERSION: &str = "1.0.0";
pub const CLAIM_CONFIDENCE_SCALE: u32 = 4;
pub const CLAIM_CONFIDENCE_ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimConfidenceItem {
    pub claim_index: usize,
    pub claim_text: String,
    pub confidence_score: String,
    pub supporting_citation_count: usize,
    pub trust_component: String,
    pub freshness_component: String,
    pub conflict_penalty: String,
    pub outlier_penalty: String,
    pub confidence_version: String,
}

pub fn calibrate_claim_confidence(
    trace_report: &TraceReport,
    evidence_packet: &Value,
    synthesis_packet: &Value,
    computation_packet: Option<&Value>,
) -> Vec<ClaimConfidenceItem> {
    let source_index = build_source_index(evidence_packet);
    let conflict_present = synthesis_packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .map(|codes| {
            codes
                .iter()
                .filter_map(Value::as_str)
                .any(|code| code == "conflicting_evidence_detected")
        })
        .unwrap_or(false)
        || synthesis_packet
            .get("uncertainty_flags")
            .and_then(Value::as_array)
            .map(|flags| {
                flags
                    .iter()
                    .filter_map(Value::as_str)
                    .any(|flag| flag == "conflicting_evidence_detected")
            })
            .unwrap_or(false);

    let outlier_penalty = if has_consensus_outliers(computation_packet) {
        Decimal::new(10, 2)
    } else {
        Decimal::ZERO
    };

    let mut out = Vec::with_capacity(trace_report.claims.len());
    for claim in &trace_report.claims {
        let support_component =
            decimal_ratio(claim.citations.len().min(3) as i64, 3).round_dp_with_strategy(
                CLAIM_CONFIDENCE_SCALE,
                CLAIM_CONFIDENCE_ROUNDING,
            );
        let trust_component = average_component(
            claim.citations.as_slice(),
            &source_index,
            |meta| meta.trust_component,
            Decimal::new(50, 2),
        );
        let freshness_component = average_component(
            claim.citations.as_slice(),
            &source_index,
            |meta| meta.freshness_component,
            Decimal::new(50, 2),
        );
        let conflict_penalty = if conflict_present {
            Decimal::new(15, 2)
        } else {
            Decimal::ZERO
        };

        let raw = support_component * Decimal::new(40, 2)
            + trust_component * Decimal::new(30, 2)
            + freshness_component * Decimal::new(20, 2)
            + corroboration_component(claim.citations.len()) * Decimal::new(10, 2)
            - conflict_penalty
            - outlier_penalty;

        let confidence = clamp_unit(raw).round_dp_with_strategy(
            CLAIM_CONFIDENCE_SCALE,
            CLAIM_CONFIDENCE_ROUNDING,
        );

        out.push(ClaimConfidenceItem {
            claim_index: claim.claim_index,
            claim_text: claim.claim_text.clone(),
            confidence_score: decimal_to_string(confidence),
            supporting_citation_count: claim.citations.len(),
            trust_component: decimal_to_string(trust_component),
            freshness_component: decimal_to_string(freshness_component),
            conflict_penalty: decimal_to_string(conflict_penalty),
            outlier_penalty: decimal_to_string(outlier_penalty),
            confidence_version: CLAIM_CONFIDENCE_VERSION.to_string(),
        });
    }
    out
}

fn corroboration_component(citation_count: usize) -> Decimal {
    match citation_count {
        0 | 1 => Decimal::ZERO,
        2 => Decimal::new(50, 2),
        _ => Decimal::ONE,
    }
}

#[derive(Debug, Clone, Copy)]
struct SourceMeta {
    trust_component: Decimal,
    freshness_component: Decimal,
}

fn build_source_index(evidence_packet: &Value) -> BTreeMap<String, SourceMeta> {
    let mut index = BTreeMap::new();
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for source in sources {
        let trust_component = parse_trust_component(&source);
        let freshness_component = parse_freshness_component(&source);

        for key in [
            source.get("url").and_then(Value::as_str),
            source.get("canonical_url").and_then(Value::as_str),
        ]
        .into_iter()
        .flatten()
        {
            index.insert(
                key.to_ascii_lowercase(),
                SourceMeta {
                    trust_component,
                    freshness_component,
                },
            );
        }
    }

    let chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for chunk in chunks {
        let key = chunk
            .get("chunk_id")
            .and_then(Value::as_str)
            .map(|chunk_id| chunk_id.to_ascii_lowercase());
        let source_url = chunk
            .get("source_url")
            .and_then(Value::as_str)
            .map(|url| url.to_ascii_lowercase());

        if let Some(chunk_key) = key {
            let inherited = source_url
                .as_ref()
                .and_then(|source| index.get(source).copied())
                .unwrap_or(SourceMeta {
                    trust_component: Decimal::new(50, 2),
                    freshness_component: Decimal::new(50, 2),
                });
            index.insert(chunk_key, inherited);
        }
    }

    index
}

fn parse_trust_component(source: &Value) -> Decimal {
    if let Some(raw) = source.get("trust_tier_score").and_then(Value::as_i64) {
        return clamp_unit(decimal_ratio(raw, 100));
    }
    if let Some(raw) = source.get("trust_score").and_then(Value::as_f64) {
        return clamp_unit(decimal_from_f64(raw));
    }
    if let Some(raw) = source.get("trust_tier").and_then(Value::as_str) {
        return match raw.to_ascii_lowercase().as_str() {
            "official" => Decimal::ONE,
            "high" => Decimal::new(85, 2),
            "medium" => Decimal::new(65, 2),
            "low" => Decimal::new(35, 2),
            _ => Decimal::new(50, 2),
        };
    }
    Decimal::new(50, 2)
}

fn parse_freshness_component(source: &Value) -> Decimal {
    if let Some(raw) = source.get("freshness_score").and_then(Value::as_f64) {
        if raw > 1.0 {
            return clamp_unit(decimal_from_f64(raw / 100.0));
        }
        return clamp_unit(decimal_from_f64(raw));
    }
    Decimal::new(50, 2)
}

fn average_component<F>(
    citations: &[String],
    index: &BTreeMap<String, SourceMeta>,
    selector: F,
    default: Decimal,
) -> Decimal
where
    F: Fn(SourceMeta) -> Decimal,
{
    if citations.is_empty() {
        return default;
    }
    let mut total = Decimal::ZERO;
    let mut count = 0usize;
    for citation in citations {
        let normalized = citation.to_ascii_lowercase();
        if let Some(meta) = index.get(&normalized).copied() {
            total += selector(meta);
            count = count.saturating_add(1);
        }
    }
    if count == 0 {
        return default;
    }
    (total / Decimal::from(count as u64)).round_dp_with_strategy(
        CLAIM_CONFIDENCE_SCALE,
        CLAIM_CONFIDENCE_ROUNDING,
    )
}

fn has_consensus_outliers(computation_packet: Option<&Value>) -> bool {
    let Some(packet) = computation_packet else {
        return false;
    };
    let Some(consensus) = packet.get("consensus").and_then(Value::as_array) else {
        return false;
    };
    consensus.iter().any(|group| {
        group
            .get("outliers")
            .and_then(Value::as_array)
            .map(|outliers| !outliers.is_empty())
            .unwrap_or(false)
    })
}

fn decimal_ratio(numerator: i64, denominator: i64) -> Decimal {
    if denominator <= 0 {
        return Decimal::ZERO;
    }
    Decimal::from(numerator) / Decimal::from(denominator)
}

fn decimal_from_f64(raw: f64) -> Decimal {
    if raw.is_finite() {
        Decimal::from_f64_retain(raw).unwrap_or(Decimal::ZERO)
    } else {
        Decimal::ZERO
    }
}

fn clamp_unit(value: Decimal) -> Decimal {
    if value < Decimal::ZERO {
        Decimal::ZERO
    } else if value > Decimal::ONE {
        Decimal::ONE
    } else {
        value
    }
}

fn decimal_to_string(value: Decimal) -> String {
    value
        .round_dp_with_strategy(CLAIM_CONFIDENCE_SCALE, CLAIM_CONFIDENCE_ROUNDING)
        .normalize()
        .to_string()
}
