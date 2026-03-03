#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::decimal::{
    decimal_to_numeric_value, decimal_to_string, is_outlier, mad, median,
};
use crate::web_search_plan::analytics::types::{
    AggregateGroup, ConsensusBuildResult, ConsensusCandidate, ConsensusGroup, ConsensusOutlier,
    NumericValue,
};
use crate::web_search_plan::contract_hash::sha256_hex;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

pub fn build_consensus(groups: &[AggregateGroup]) -> ConsensusBuildResult {
    let mut out = Vec::new();
    let mut reason_codes = BTreeSet::new();

    for group in groups {
        let topic = format!(
            "{}:{}:{}:{}:{}",
            group.metric_id,
            group.entity,
            group.attribute,
            group.unit.as_deref().unwrap_or("none"),
            group.currency.as_deref().unwrap_or("none"),
        );

        let mut candidate_map: BTreeMap<String, CandidateAccumulator> = BTreeMap::new();
        let mut total_weight = Decimal::ZERO;

        for sample in &group.samples {
            let numeric = decimal_to_numeric_value(sample.value_decimal);
            let key = numeric_key(&numeric);
            let entry = candidate_map
                .entry(key)
                .or_insert_with(|| CandidateAccumulator {
                    value: numeric,
                    sources: BTreeSet::new(),
                    total_weight: Decimal::ZERO,
                });
            entry.sources.insert(sample.source_ref.clone());
            entry.total_weight += Decimal::from(sample.trust_weight as i64);
            total_weight += Decimal::from(sample.trust_weight as i64);
        }

        let mut candidates = candidate_map
            .into_values()
            .map(|entry| ConsensusCandidate {
                value: entry.value,
                sources: entry.sources.into_iter().collect(),
            })
            .collect::<Vec<ConsensusCandidate>>();

        candidates.sort_by(|left, right| {
            (numeric_key(&left.value), left.sources.clone())
                .cmp(&(numeric_key(&right.value), right.sources.clone()))
        });

        let mut weighted = candidates
            .iter()
            .map(|candidate| {
                let weight = group
                    .samples
                    .iter()
                    .filter(|sample| {
                        decimal_to_numeric_value(sample.value_decimal) == candidate.value
                    })
                    .map(|sample| Decimal::from(sample.trust_weight as i64))
                    .sum::<Decimal>();
                (candidate.value.clone(), weight)
            })
            .collect::<Vec<(NumericValue, Decimal)>>();

        weighted.sort_by(|left, right| {
            let left_key = numeric_key(&left.0);
            let right_key = numeric_key(&right.0);
            right.1.cmp(&left.1).then_with(|| left_key.cmp(&right_key))
        });

        let chosen = choose_consensus(weighted.as_slice(), total_weight);
        let agreement_score = chosen
            .as_ref()
            .and_then(|value| {
                weighted
                    .iter()
                    .find(|(candidate, _)| candidate == value)
                    .map(|(_, weight)| {
                        if total_weight.is_zero() {
                            Decimal::ZERO
                        } else {
                            *weight / total_weight
                        }
                    })
            })
            .unwrap_or(Decimal::ZERO);

        if chosen.is_none() && !weighted.is_empty() {
            reason_codes.insert("conflicting_evidence_detected".to_string());
        }

        let outliers = detect_outliers(group);

        let group_id = stable_group_id(topic.as_str(), candidates.as_slice());
        out.push(ConsensusGroup {
            group_id,
            topic,
            candidates,
            chosen,
            agreement_score: decimal_to_string(agreement_score),
            outliers,
        });
    }

    out.sort_by(|left, right| {
        (left.topic.as_str(), left.group_id.as_str())
            .cmp(&(right.topic.as_str(), right.group_id.as_str()))
    });

    ConsensusBuildResult {
        groups: out,
        reason_codes: reason_codes.into_iter().collect(),
    }
}

fn stable_group_id(topic: &str, candidates: &[ConsensusCandidate]) -> String {
    let serialized = candidates
        .iter()
        .map(|candidate| {
            format!(
                "{}|{}",
                numeric_key(&candidate.value),
                candidate.sources.join(",")
            )
        })
        .collect::<Vec<String>>()
        .join(";");
    sha256_hex(format!("{}::{}", topic, serialized).as_bytes())
}

fn choose_consensus(
    weighted: &[(NumericValue, Decimal)],
    total_weight: Decimal,
) -> Option<NumericValue> {
    if weighted.is_empty() || total_weight.is_zero() {
        return None;
    }

    let best = &weighted[0];
    let best_ratio = best.1 / total_weight;

    let tied = weighted.len() > 1 && weighted[1].1 == best.1;
    if tied || best_ratio <= Decimal::new(5, 1) {
        None
    } else {
        Some(best.0.clone())
    }
}

fn detect_outliers(group: &AggregateGroup) -> Vec<ConsensusOutlier> {
    let values = group
        .samples
        .iter()
        .map(|sample| sample.value_decimal)
        .collect::<Vec<Decimal>>();
    if values.is_empty() {
        return Vec::new();
    }

    let mut sorted = values.clone();
    sorted.sort();
    let center = median(sorted.as_slice());
    let mad_value = mad(values.as_slice());

    let mut bucketed: BTreeMap<String, (NumericValue, BTreeSet<String>)> = BTreeMap::new();

    for sample in &group.samples {
        if is_outlier(sample.value_decimal, center, mad_value) {
            let value = decimal_to_numeric_value(sample.value_decimal);
            let key = numeric_key(&value);
            let entry = bucketed
                .entry(key)
                .or_insert_with(|| (value.clone(), BTreeSet::new()));
            entry.1.insert(sample.source_ref.clone());
        }
    }

    let mut outliers = bucketed
        .into_values()
        .map(|(value, sources)| ConsensusOutlier {
            value,
            sources: sources.into_iter().collect(),
        })
        .collect::<Vec<ConsensusOutlier>>();

    outliers.sort_by(|left, right| {
        (numeric_key(&left.value), left.sources.clone())
            .cmp(&(numeric_key(&right.value), right.sources.clone()))
    });

    outliers
}

fn numeric_key(value: &NumericValue) -> String {
    match value {
        NumericValue::Int { value } => format!("int:{}", value),
        NumericValue::Decimal { value } => format!("decimal:{}", value),
    }
}

#[derive(Debug, Clone)]
struct CandidateAccumulator {
    value: NumericValue,
    sources: BTreeSet<String>,
    total_weight: Decimal,
}
