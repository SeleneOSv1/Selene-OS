#![forbid(unsafe_code)]

use crate::web_search_plan::risk::calibration::{bps_to_decimal, factor_calibration};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FactorId {
    FinancialStress,
    LegalEvents,
    RegulatoryEvents,
    NegativeNewsCluster,
    OperationalReliability,
}

impl FactorId {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FinancialStress => "financial_stress",
            Self::LegalEvents => "legal_events",
            Self::RegulatoryEvents => "regulatory_events",
            Self::NegativeNewsCluster => "negative_news_cluster",
            Self::OperationalReliability => "operational_reliability",
        }
    }

    pub const fn all() -> [FactorId; 5] {
        [
            Self::FinancialStress,
            Self::LegalEvents,
            Self::RegulatoryEvents,
            Self::NegativeNewsCluster,
            Self::OperationalReliability,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorScore {
    pub factor_id: FactorId,
    pub score: Decimal,
    pub weight: Decimal,
    pub evidence_refs: Vec<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactorExtractionResult {
    pub factors: Vec<FactorScore>,
    pub missing_factors: Vec<FactorId>,
    pub reason_codes: Vec<String>,
    pub known_refs: BTreeSet<String>,
    pub contradiction_present: bool,
}

pub fn extract_factor_scores(
    evidence_packet: &Value,
    computation_packet: Option<&Value>,
) -> Result<FactorExtractionResult, String> {
    let known_refs = known_evidence_refs(evidence_packet);
    if known_refs.is_empty() {
        return Err("risk factors require evidence refs in sources/content_chunks".to_string());
    }

    let contradiction_present = has_contradiction_group(evidence_packet);
    let mut factors = Vec::new();
    let mut missing_factors = Vec::new();
    let mut reason_codes = Vec::new();

    for factor_id in FactorId::all() {
        let calibration = factor_calibration(factor_id);
        let (keyword_hits, mut refs) = keyword_hits_for_factor(factor_id, evidence_packet);
        let (supplemental_points, supplemental_refs) =
            supplemental_points_for_factor(factor_id, computation_packet);
        refs.extend(supplemental_refs);
        refs.sort();
        refs.dedup();
        refs.retain(|entry| known_refs.contains(entry));

        if refs.len() < calibration.min_evidence_refs {
            missing_factors.push(factor_id);
            continue;
        }

        let total_points = keyword_hits.saturating_add(supplemental_points);
        let saturated_points = total_points.min(calibration.saturation_points as usize);
        let score = if calibration.saturation_points == 0 {
            Decimal::ZERO
        } else {
            Decimal::from(saturated_points as u64) / Decimal::from(calibration.saturation_points)
        };

        factors.push(FactorScore {
            factor_id,
            score,
            weight: bps_to_decimal(calibration.weight_bps),
            evidence_refs: refs.clone(),
            notes: Some(format!(
                "evidence_points={} refs={}",
                total_points,
                refs.len()
            )),
        });
    }

    if contradiction_present {
        push_unique(&mut reason_codes, "conflicting_evidence_detected");
    }

    factors.sort_by_key(|factor| factor.factor_id);
    missing_factors.sort();

    Ok(FactorExtractionResult {
        factors,
        missing_factors,
        reason_codes,
        known_refs,
        contradiction_present,
    })
}

fn keyword_hits_for_factor(factor_id: FactorId, evidence_packet: &Value) -> (usize, Vec<String>) {
    let keywords = keywords_for_factor(factor_id);
    let mut total_hits = 0usize;
    let mut refs = Vec::new();

    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            let text = format!(
                "{}\n{}",
                source
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or_default(),
                source
                    .get("snippet")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
            );
            let hits = count_keyword_hits(text.as_str(), keywords);
            if hits > 0 {
                total_hits = total_hits.saturating_add(hits);
                if let Some(url) = source.get("url").and_then(Value::as_str) {
                    refs.push(url.to_string());
                }
            }
        }
    }

    if let Some(chunks) = evidence_packet.get("content_chunks").and_then(Value::as_array) {
        for chunk in chunks {
            let text = chunk
                .get("text_excerpt")
                .and_then(Value::as_str)
                .or_else(|| chunk.get("normalized_text").and_then(Value::as_str))
                .unwrap_or_default();
            let hits = count_keyword_hits(text, keywords);
            if hits > 0 {
                total_hits = total_hits.saturating_add(hits);
                if let Some(chunk_id) = chunk.get("chunk_id").and_then(Value::as_str) {
                    refs.push(chunk_id.to_string());
                } else if let Some(source_url) = chunk.get("source_url").and_then(Value::as_str) {
                    refs.push(source_url.to_string());
                }
            }
        }
    }

    (total_hits, refs)
}

fn supplemental_points_for_factor(
    factor_id: FactorId,
    computation_packet: Option<&Value>,
) -> (usize, Vec<String>) {
    let Some(packet) = computation_packet else {
        return (0, Vec::new());
    };
    let Some(aggregates) = packet.get("aggregates").and_then(Value::as_array) else {
        return (0, Vec::new());
    };

    let mut points = 0usize;
    let mut refs = Vec::new();
    for aggregate in aggregates {
        let attribute = aggregate
            .get("attribute")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase();
        let value = aggregate.get("value");
        let numeric_value = value.and_then(parse_numeric_value);

        match factor_id {
            FactorId::FinancialStress => {
                let attr_match = attribute.contains("net_income")
                    || attribute.contains("cash_flow")
                    || attribute.contains("debt")
                    || attribute.contains("loss");
                if attr_match {
                    if let Some(number) = numeric_value {
                        if number < Decimal::ZERO {
                            points = points.saturating_add(2);
                            collect_source_refs(aggregate, &mut refs);
                        }
                    }
                }
            }
            FactorId::OperationalReliability => {
                if attribute.contains("uptime") || attribute.contains("availability") {
                    if let Some(number) = numeric_value {
                        if number < Decimal::new(95, 0) {
                            points = points.saturating_add(1);
                            collect_source_refs(aggregate, &mut refs);
                        }
                    }
                }
            }
            _ => {}
        }
    }
    (points, refs)
}

fn collect_source_refs(entry: &Value, out: &mut Vec<String>) {
    if let Some(refs) = entry.get("source_refs").and_then(Value::as_array) {
        for evidence_ref in refs {
            if let Some(raw) = evidence_ref.as_str() {
                out.push(raw.to_string());
            }
        }
    }
}

fn parse_numeric_value(value: &Value) -> Option<Decimal> {
    let type_name = value.get("type").and_then(Value::as_str)?;
    match type_name {
        "int" => value
            .get("value")
            .and_then(Value::as_i64)
            .map(Decimal::from),
        "decimal" => value
            .get("value")
            .and_then(Value::as_str)
            .and_then(|raw| raw.parse::<Decimal>().ok()),
        _ => None,
    }
}

fn keywords_for_factor(factor_id: FactorId) -> &'static [&'static str] {
    match factor_id {
        FactorId::FinancialStress => &[
            "bankruptcy",
            "insolvency",
            "default",
            "cash burn",
            "liquidity",
            "going concern",
            "net loss",
            "debt burden",
        ],
        FactorId::LegalEvents => &[
            "lawsuit",
            "litigation",
            "settlement",
            "class action",
            "subpoena",
            "court filing",
            "legal dispute",
        ],
        FactorId::RegulatoryEvents => &[
            "regulatory",
            "enforcement",
            "penalty",
            "fine",
            "sanction",
            "compliance breach",
            "regulator",
        ],
        FactorId::NegativeNewsCluster => &[
            "scandal",
            "fraud",
            "breach",
            "probe",
            "investigation",
            "downgrade",
            "controversy",
        ],
        FactorId::OperationalReliability => &[
            "outage",
            "downtime",
            "service disruption",
            "degradation",
            "incident",
            "failure",
            "delay",
        ],
    }
}

fn count_keyword_hits(text: &str, keywords: &[&str]) -> usize {
    let normalized = text.to_ascii_lowercase();
    keywords
        .iter()
        .filter(|keyword| normalized.contains(**keyword))
        .count()
}

fn known_evidence_refs(evidence_packet: &Value) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            if let Some(url) = source.get("url").and_then(Value::as_str) {
                refs.insert(url.to_string());
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

fn has_contradiction_group(evidence_packet: &Value) -> bool {
    let source_conflict = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .map(|sources| {
            sources.iter().any(|source| {
                source
                    .get("contradiction_group_id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .is_some()
            })
        })
        .unwrap_or(false);
    let chunk_conflict = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .map(|chunks| {
            chunks.iter().any(|chunk| {
                chunk
                    .get("contradiction_group_id")
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|entry| !entry.is_empty())
                    .is_some()
            })
        })
        .unwrap_or(false);
    source_conflict || chunk_conflict
}

fn push_unique(reason_codes: &mut Vec<String>, reason_code: &str) {
    if !reason_codes.iter().any(|entry| entry == reason_code) {
        reason_codes.push(reason_code.to_string());
    }
}

#[allow(dead_code)]
fn refs_by_domain(refs: &[String]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for entry in refs {
        let domain = url::Url::parse(entry)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|host| host.to_ascii_lowercase()))
            .unwrap_or_else(|| "non_url_ref".to_string());
        *counts.entry(domain).or_insert(0) += 1;
    }
    counts
}
