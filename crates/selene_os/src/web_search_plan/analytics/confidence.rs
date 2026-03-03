#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::decimal::{decimal_to_string, round_decimal};
use crate::web_search_plan::analytics::types::{
    Aggregate, AggregateGroup, ConfidenceFactors, ConfidenceItem, ConsensusGroup,
};
use rust_decimal::Decimal;
use std::collections::BTreeMap;

const MILLIS_PER_DAY: i64 = 86_400_000;

pub fn build_confidence(
    aggregates: &[Aggregate],
    groups: &[AggregateGroup],
    consensus_groups: &[ConsensusGroup],
    request_as_of_ms: Option<i64>,
) -> Vec<ConfidenceItem> {
    let group_map = groups
        .iter()
        .map(|group| (group_key(group), group))
        .collect::<BTreeMap<String, &AggregateGroup>>();
    let consensus_map = consensus_groups
        .iter()
        .map(|group| (group.topic.clone(), group))
        .collect::<BTreeMap<String, &ConsensusGroup>>();

    let mut output = Vec::new();

    for aggregate in aggregates {
        let key = aggregate_group_key(aggregate);
        let Some(group) = group_map.get(&key).copied() else {
            continue;
        };
        let topic = format!(
            "{}:{}:{}:{}:{}",
            group.metric_id,
            group.entity,
            group.attribute,
            group.unit.as_deref().unwrap_or("none"),
            group.currency.as_deref().unwrap_or("none"),
        );
        let consensus = consensus_map.get(&topic).copied();

        let sample_factor = sample_size_factor(group.samples.len() as u32);
        let trust_factor = trust_mix_factor(group);
        let recency_factor = recency_factor(group.as_of_ms, request_as_of_ms);
        let conflict_factor = if consensus.and_then(|item| item.chosen.as_ref()).is_some() {
            Decimal::ONE
        } else {
            Decimal::new(4, 1)
        };
        let outlier_factor = outlier_factor(consensus);

        let score = round_decimal(
            sample_factor * Decimal::new(30, 2)
                + trust_factor * Decimal::new(25, 2)
                + recency_factor * Decimal::new(20, 2)
                + conflict_factor * Decimal::new(15, 2)
                + outlier_factor * Decimal::new(10, 2),
        );

        output.push(ConfidenceItem {
            claim_key: format!(
                "{}:{}:{}:{}",
                aggregate.metric_id,
                aggregate.entity,
                aggregate.attribute,
                aggregate.method.as_str()
            ),
            confidence_score: decimal_to_string(score),
            factors: ConfidenceFactors {
                sample_size: aggregate.sample_size,
                trust_tier_mix: decimal_to_string(trust_factor),
                recency: decimal_to_string(recency_factor),
                conflict: decimal_to_string(conflict_factor),
                outliers: decimal_to_string(outlier_factor),
            },
        });
    }

    output.sort_by(|left, right| left.claim_key.cmp(&right.claim_key));
    output
}

fn sample_size_factor(sample_size: u32) -> Decimal {
    let capped = sample_size.min(10);
    Decimal::from(capped as i64) / Decimal::from(10)
}

fn trust_mix_factor(group: &AggregateGroup) -> Decimal {
    if group.samples.is_empty() {
        return Decimal::ZERO;
    }
    let total = group
        .samples
        .iter()
        .map(|sample| Decimal::from(sample.trust_weight as i64))
        .sum::<Decimal>();
    let avg = total / Decimal::from(group.samples.len() as i64);
    round_decimal((avg / Decimal::from(3)).min(Decimal::ONE))
}

fn recency_factor(group_as_of: Option<i64>, request_as_of_ms: Option<i64>) -> Decimal {
    let Some(request_now) = request_as_of_ms else {
        return Decimal::new(7, 1);
    };
    let Some(value_as_of) = group_as_of else {
        return Decimal::new(5, 1);
    };

    let age_days = ((request_now - value_as_of).max(0)) / MILLIS_PER_DAY;
    if age_days <= 30 {
        Decimal::ONE
    } else if age_days <= 90 {
        Decimal::new(7, 1)
    } else {
        Decimal::new(4, 1)
    }
}

fn outlier_factor(consensus: Option<&ConsensusGroup>) -> Decimal {
    let outliers = consensus.map(|group| group.outliers.len()).unwrap_or(0);
    if outliers == 0 {
        Decimal::ONE
    } else {
        Decimal::ONE / Decimal::from((outliers + 1) as i64)
    }
}

fn group_key(group: &AggregateGroup) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        group.metric_id,
        group.entity,
        group.attribute,
        group.unit.as_deref().unwrap_or("none"),
        group.currency.as_deref().unwrap_or("none"),
    )
}

fn aggregate_group_key(aggregate: &Aggregate) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        aggregate.metric_id,
        aggregate.entity,
        aggregate.attribute,
        aggregate.unit.as_deref().unwrap_or("none"),
        aggregate.currency.as_deref().unwrap_or("none"),
    )
}
