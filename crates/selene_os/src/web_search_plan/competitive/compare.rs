#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::types::{AggregateMethod, ComputationPacket, NumericValue};
use crate::web_search_plan::competitive::entity_normalize::EntityIndex;
use crate::web_search_plan::competitive::schema::{
    BillingPeriod, CompetitiveComparison, CompetitiveFeature, CompetitivePrice, PositionClass,
    PositionSummary,
};
use crate::web_search_plan::competitive::swot::build_swot;
use rust_decimal::Decimal;
use std::collections::BTreeSet;
use std::str::FromStr;

pub fn build_competitive_comparison(
    target_entity: &str,
    entity_index: EntityIndex,
    pricing_table: Vec<CompetitivePrice>,
    feature_matrix: Vec<CompetitiveFeature>,
    computation_packet: Option<&ComputationPacket>,
    mut uncertainty_flags: Vec<String>,
    mut reason_codes: Vec<String>,
) -> CompetitiveComparison {
    let target_entity_id = entity_index.target_entity_id().map(ToString::to_string);
    let competitors = if let Some(target_id) = &target_entity_id {
        entity_index
            .entities
            .into_iter()
            .filter(|entity| entity.entity_id != *target_id)
            .collect::<Vec<_>>()
    } else {
        reason_codes.push("insufficient_evidence".to_string());
        uncertainty_flags.push("target_entity_missing".to_string());
        entity_index.entities
    };

    let mut competitor_ids = competitors
        .iter()
        .map(|entity| entity.entity_id.clone())
        .collect::<Vec<String>>();
    competitor_ids.sort();

    let (position_summary, position_uncertainty, position_reason_code) = derive_position_summary(
        target_entity_id.as_deref(),
        &pricing_table,
        computation_packet,
    );
    if let Some(flag) = position_uncertainty {
        uncertainty_flags.push(flag.to_string());
    }
    if let Some(code) = position_reason_code {
        reason_codes.push(code.to_string());
    }

    let swot = target_entity_id
        .as_deref()
        .and_then(|target_id| build_swot(target_id, &competitor_ids, &pricing_table, &feature_matrix));

    let mut all_source_refs = BTreeSet::new();
    for entity in &competitors {
        for source_ref in &entity.sources {
            all_source_refs.insert(source_ref.clone());
        }
    }
    for price in &pricing_table {
        for source_ref in &price.source_refs {
            all_source_refs.insert(source_ref.clone());
        }
    }
    for feature in &feature_matrix {
        for source_ref in &feature.source_refs {
            all_source_refs.insert(source_ref.clone());
        }
    }
    if let Some(summary) = &position_summary {
        for source_ref in &summary.source_refs {
            all_source_refs.insert(source_ref.clone());
        }
    }
    if let Some(swot_value) = &swot {
        for bullet in swot_value
            .strengths
            .iter()
            .chain(swot_value.weaknesses.iter())
            .chain(swot_value.opportunities.iter())
            .chain(swot_value.threats.iter())
        {
            for source_ref in &bullet.source_refs {
                all_source_refs.insert(source_ref.clone());
            }
        }
    }

    dedupe_sort(&mut uncertainty_flags);
    dedupe_sort(&mut reason_codes);

    CompetitiveComparison {
        target_entity: target_entity.trim().to_lowercase(),
        competitors,
        pricing_table,
        feature_matrix,
        position_summary,
        swot,
        uncertainty_flags,
        reason_codes,
        source_refs: all_source_refs.into_iter().collect(),
    }
}

fn dedupe_sort(values: &mut Vec<String>) {
    let mut dedupe = BTreeSet::new();
    for value in values.iter() {
        dedupe.insert(value.clone());
    }
    *values = dedupe.into_iter().collect();
}

fn derive_position_summary(
    target_entity_id: Option<&str>,
    pricing_table: &[CompetitivePrice],
    computation_packet: Option<&ComputationPacket>,
) -> (Option<PositionSummary>, Option<&'static str>, Option<&'static str>) {
    let Some(target_id) = target_entity_id else {
        return (
            None,
            Some("position_unavailable_missing_target"),
            Some("insufficient_evidence"),
        );
    };
    let Some(computation) = computation_packet else {
        return (
            None,
            Some("position_unavailable_missing_computation"),
            Some("insufficient_evidence"),
        );
    };

    let comparable_target = pricing_table
        .iter()
        .filter(|entry| entry.entity_id == target_id)
        .find_map(monthly_equivalent_for_entry);
    let Some((target_value, target_currency, target_refs)) = comparable_target else {
        return (
            None,
            Some("position_unavailable_missing_target_price"),
            Some("insufficient_evidence"),
        );
    };

    let p50 = find_aggregate_value(computation, AggregateMethod::P50, target_currency.as_str());
    let p75 = find_aggregate_value(computation, AggregateMethod::P75, target_currency.as_str());
    let Some((p50_value, mut source_refs)) = p50 else {
        return (
            None,
            Some("position_unavailable_missing_market_p50"),
            Some("insufficient_evidence"),
        );
    };
    let Some((p75_value, p75_refs)) = p75 else {
        return (
            None,
            Some("position_unavailable_missing_market_p75"),
            Some("insufficient_evidence"),
        );
    };
    source_refs.extend(p75_refs);
    source_refs.extend(target_refs);
    let mut source_set = BTreeSet::new();
    for source_ref in source_refs {
        source_set.insert(source_ref);
    }

    let classification = if target_value > (p75_value * Decimal::from_str("1.50").unwrap_or(Decimal::ONE)) {
        PositionClass::Outlier
    } else if target_value > p75_value {
        PositionClass::Premium
    } else if target_value > p50_value {
        PositionClass::UpperTier
    } else {
        PositionClass::WithinMarket
    };

    (
        Some(PositionSummary {
            classification,
            basis_metric: "market_price_monthly".to_string(),
            target_value: Some(target_value.normalize().to_string()),
            comparator_value: Some(p50_value.normalize().to_string()),
            source_refs: source_set.into_iter().collect(),
        }),
        None,
        None,
    )
}

fn monthly_equivalent_for_entry(entry: &CompetitivePrice) -> Option<(Decimal, String, Vec<String>)> {
    if entry.currency == "UNKNOWN" {
        return None;
    }
    let amount = match entry.billing_period {
        BillingPeriod::Monthly | BillingPeriod::Unknown => Decimal::from_str(&entry.price_value).ok(),
        BillingPeriod::Yearly => entry
            .normalized_to
            .as_ref()
            .and_then(|value| Decimal::from_str(value).ok())
            .or_else(|| Decimal::from_str(&entry.price_value).ok().map(|value| value / Decimal::from(12))),
        BillingPeriod::OneTime | BillingPeriod::UsageBased => None,
    }?;
    Some((amount, entry.currency.clone(), entry.source_refs.clone()))
}

fn find_aggregate_value(
    computation_packet: &ComputationPacket,
    method: AggregateMethod,
    currency: &str,
) -> Option<(Decimal, Vec<String>)> {
    computation_packet
        .aggregates
        .iter()
        .find(|aggregate| {
            aggregate.method == method
                && aggregate
                    .attribute
                    .to_lowercase()
                    .contains("price")
                && aggregate.currency.as_deref().unwrap_or_default() == currency
        })
        .and_then(|aggregate| numeric_value_to_decimal(&aggregate.value).map(|value| (value, aggregate.source_refs.clone())))
}

fn numeric_value_to_decimal(value: &NumericValue) -> Option<Decimal> {
    match value {
        NumericValue::Int { value } => Some(Decimal::from(*value)),
        NumericValue::Decimal { value } => Decimal::from_str(value).ok(),
    }
}
