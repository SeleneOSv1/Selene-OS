#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::currency_normalize::CurrencyRateTable;
use crate::web_search_plan::analytics::decimal::{
    decimal_to_numeric_value, max, mean, median, min, percentile, stddev, trimmed_mean,
    weighted_mean,
};
use crate::web_search_plan::analytics::types::{
    Aggregate, AggregateBuildResult, AggregateGroup, AggregateMethod, AggregateWindow,
    NumericSample,
};
use crate::web_search_plan::analytics::unit_normalize::UnitConversionTable;
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

const METHOD_ORDER: [AggregateMethod; 10] = [
    AggregateMethod::Mean,
    AggregateMethod::Median,
    AggregateMethod::TrimmedMean,
    AggregateMethod::WeightedMean,
    AggregateMethod::Min,
    AggregateMethod::Max,
    AggregateMethod::P25,
    AggregateMethod::P50,
    AggregateMethod::P75,
    AggregateMethod::Stddev,
];

type BaseGroupKey = (String, String, String);
type FinalGroupKey = (String, String, String, Option<String>, Option<String>);

pub fn compute_aggregates(
    samples: Vec<NumericSample>,
    unit_table: &UnitConversionTable,
    currency_table: &CurrencyRateTable,
) -> AggregateBuildResult {
    let mut grouped: BTreeMap<BaseGroupKey, Vec<NumericSample>> = BTreeMap::new();
    for sample in samples {
        grouped
            .entry((
                sample.metric_id.clone(),
                sample.entity.clone(),
                sample.attribute.clone(),
            ))
            .or_default()
            .push(sample);
    }

    let mut reason_codes = BTreeSet::new();
    let mut final_groups: Vec<AggregateGroup> = Vec::new();

    for ((metric_id, entity, attribute), entries) in grouped {
        let (normalized, had_unit_mismatch, had_currency_mismatch) =
            normalize_group(entries, unit_table, currency_table);
        if had_unit_mismatch || had_currency_mismatch {
            reason_codes.insert("policy_violation".to_string());
        }

        let mut subgrouped: BTreeMap<FinalGroupKey, Vec<NumericSample>> = BTreeMap::new();
        for sample in normalized {
            subgrouped
                .entry((
                    metric_id.clone(),
                    entity.clone(),
                    attribute.clone(),
                    sample.unit.clone(),
                    sample.currency.clone(),
                ))
                .or_default()
                .push(sample);
        }

        for ((metric_id, entity, attribute, unit, currency), mut group_samples) in subgrouped {
            group_samples.sort_by(|left, right| {
                (
                    left.source_ref.as_str(),
                    left.source_url.as_str(),
                    left.value_decimal,
                )
                    .cmp(&(
                        right.source_ref.as_str(),
                        right.source_url.as_str(),
                        right.value_decimal,
                    ))
            });

            let as_of = uniform_as_of_ms(group_samples.as_slice());
            final_groups.push(AggregateGroup {
                metric_id,
                entity,
                attribute,
                unit,
                currency,
                as_of_ms: as_of,
                samples: group_samples,
            });
        }
    }

    final_groups.sort_by(|left, right| {
        (
            left.metric_id.as_str(),
            left.entity.as_str(),
            left.attribute.as_str(),
            left.unit.as_deref().unwrap_or(""),
            left.currency.as_deref().unwrap_or(""),
        )
            .cmp(&(
                right.metric_id.as_str(),
                right.entity.as_str(),
                right.attribute.as_str(),
                right.unit.as_deref().unwrap_or(""),
                right.currency.as_deref().unwrap_or(""),
            ))
    });

    let mut aggregates = Vec::new();
    for group in &final_groups {
        let mut values = group
            .samples
            .iter()
            .map(|sample| sample.value_decimal)
            .collect::<Vec<Decimal>>();
        values.sort();

        let weighted = group
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.value_decimal,
                    Decimal::from(sample.trust_weight as i64),
                )
            })
            .collect::<Vec<(Decimal, Decimal)>>();

        let source_refs = group
            .samples
            .iter()
            .map(|sample| sample.source_ref.clone())
            .collect::<BTreeSet<String>>()
            .into_iter()
            .collect::<Vec<String>>();

        for method in METHOD_ORDER {
            let value = match method {
                AggregateMethod::Mean => mean(values.as_slice()),
                AggregateMethod::Median => median(values.as_slice()),
                AggregateMethod::TrimmedMean => trimmed_mean(values.as_slice()),
                AggregateMethod::WeightedMean => weighted_mean(weighted.as_slice()),
                AggregateMethod::Min => min(values.as_slice()),
                AggregateMethod::Max => max(values.as_slice()),
                AggregateMethod::P25 => percentile(values.as_slice(), 25),
                AggregateMethod::P50 => percentile(values.as_slice(), 50),
                AggregateMethod::P75 => percentile(values.as_slice(), 75),
                AggregateMethod::Stddev => stddev(values.as_slice()),
            };

            aggregates.push(Aggregate {
                metric_id: group.metric_id.clone(),
                entity: group.entity.clone(),
                attribute: group.attribute.clone(),
                unit: group.unit.clone(),
                currency: group.currency.clone(),
                window: Some(AggregateWindow {
                    as_of_ms: group.as_of_ms,
                    from_ms: None,
                    to_ms: None,
                }),
                method,
                value: decimal_to_numeric_value(value),
                sample_size: group.samples.len() as u32,
                source_refs: source_refs.clone(),
            });
        }
    }

    aggregates.sort_by(|left, right| {
        (
            left.metric_id.as_str(),
            left.entity.as_str(),
            left.attribute.as_str(),
            left.unit.as_deref().unwrap_or(""),
            left.currency.as_deref().unwrap_or(""),
            left.method,
        )
            .cmp(&(
                right.metric_id.as_str(),
                right.entity.as_str(),
                right.attribute.as_str(),
                right.unit.as_deref().unwrap_or(""),
                right.currency.as_deref().unwrap_or(""),
                right.method,
            ))
    });

    AggregateBuildResult {
        aggregates,
        groups: final_groups,
        reason_codes: reason_codes.into_iter().collect(),
    }
}

fn uniform_as_of_ms(samples: &[NumericSample]) -> Option<i64> {
    let mut values = samples.iter().filter_map(|sample| sample.as_of_ms);
    let first = values.next()?;
    if values.all(|value| value == first) {
        Some(first)
    } else {
        None
    }
}

fn normalize_group(
    mut samples: Vec<NumericSample>,
    unit_table: &UnitConversionTable,
    currency_table: &CurrencyRateTable,
) -> (Vec<NumericSample>, bool, bool) {
    let mut unit_mismatch = false;
    let mut currency_mismatch = false;

    let unit_values = samples
        .iter()
        .filter_map(|sample| sample.unit.as_ref().map(|u| u.to_ascii_lowercase()))
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();

    if unit_values.len() > 1 {
        let target = unit_values.iter().find(|candidate| {
            samples.iter().all(|sample| {
                sample
                    .unit
                    .as_deref()
                    .map(|unit| unit.eq_ignore_ascii_case(candidate.as_str()))
                    .unwrap_or(true)
                    || sample
                        .unit
                        .as_deref()
                        .and_then(|unit| {
                            unit_table.convert(sample.value_decimal, unit, candidate.as_str())
                        })
                        .is_some()
            })
        });

        if let Some(target) = target {
            for sample in &mut samples {
                if let Some(unit) = sample.unit.clone() {
                    if !unit.eq_ignore_ascii_case(target.as_str()) {
                        if let Some(converted) =
                            unit_table.convert(sample.value_decimal, unit.as_str(), target.as_str())
                        {
                            sample.value_decimal = converted;
                            sample.unit = Some(target.clone());
                        }
                    } else {
                        sample.unit = Some(target.clone());
                    }
                }
            }
        } else {
            unit_mismatch = true;
        }
    }

    let currency_values = samples
        .iter()
        .filter_map(|sample| sample.currency.as_ref().map(|c| c.to_ascii_uppercase()))
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect::<Vec<String>>();

    if currency_values.len() > 1 {
        let target = currency_values.iter().find(|candidate| {
            samples.iter().all(|sample| {
                sample
                    .currency
                    .as_deref()
                    .map(|currency| currency.eq_ignore_ascii_case(candidate.as_str()))
                    .unwrap_or(true)
                    || sample
                        .currency
                        .as_deref()
                        .and_then(|currency| {
                            currency_table.convert(
                                sample.value_decimal,
                                currency,
                                candidate.as_str(),
                            )
                        })
                        .is_some()
            })
        });

        if let Some(target) = target {
            for sample in &mut samples {
                if let Some(currency) = sample.currency.clone() {
                    if !currency.eq_ignore_ascii_case(target.as_str()) {
                        if let Some(converted) = currency_table.convert(
                            sample.value_decimal,
                            currency.as_str(),
                            target.as_str(),
                        ) {
                            sample.value_decimal = converted;
                            sample.currency = Some(target.clone());
                        }
                    } else {
                        sample.currency = Some(target.clone());
                    }
                }
            }
        } else {
            currency_mismatch = true;
        }
    }

    (samples, unit_mismatch, currency_mismatch)
}
