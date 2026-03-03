#![forbid(unsafe_code)]

use crate::web_search_plan::competitive::schema::{
    BillingPeriod, CompetitiveFeature, CompetitivePrice, CompetitiveSwot, SwotBullet, TriState,
};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};
use std::str::FromStr;

pub fn build_swot(
    target_entity_id: &str,
    competitor_ids: &[String],
    pricing_table: &[CompetitivePrice],
    feature_matrix: &[CompetitiveFeature],
) -> Option<CompetitiveSwot> {
    let strengths = build_strengths(target_entity_id, competitor_ids, feature_matrix);
    let weaknesses = build_weaknesses(target_entity_id, competitor_ids, feature_matrix);
    let (opportunities, threats) = build_price_opportunities_threats(
        target_entity_id,
        competitor_ids,
        pricing_table,
    );

    if strengths.is_empty() && weaknesses.is_empty() && opportunities.is_empty() && threats.is_empty() {
        None
    } else {
        Some(CompetitiveSwot {
            strengths,
            weaknesses,
            opportunities,
            threats,
        })
    }
}

fn build_strengths(
    target_entity_id: &str,
    competitor_ids: &[String],
    feature_matrix: &[CompetitiveFeature],
) -> Vec<SwotBullet> {
    let by_key = feature_map(feature_matrix);
    let mut bullets = Vec::new();
    for (feature_key, by_entity) in by_key {
        let Some(target_feature) = by_entity.get(target_entity_id) else {
            continue;
        };
        if target_feature.presence != TriState::True {
            continue;
        }
        let has_gap = competitor_ids.iter().any(|competitor_id| {
            by_entity
                .get(competitor_id)
                .map(|entry| entry.presence != TriState::True)
                .unwrap_or(true)
        });
        if !has_gap {
            continue;
        }

        bullets.push(SwotBullet {
            text: format!("Target provides feature {} with stronger coverage", feature_key),
            source_refs: dedupe_refs(vec![target_feature.source_refs.clone()]),
        });
    }
    sort_and_cap_bullets(bullets)
}

fn build_weaknesses(
    target_entity_id: &str,
    competitor_ids: &[String],
    feature_matrix: &[CompetitiveFeature],
) -> Vec<SwotBullet> {
    let by_key = feature_map(feature_matrix);
    let mut bullets = Vec::new();
    for (feature_key, by_entity) in by_key {
        let target_presence = by_entity
            .get(target_entity_id)
            .map(|entry| entry.presence)
            .unwrap_or(TriState::Unknown);
        if target_presence == TriState::True {
            continue;
        }
        let Some(competitor_feature) = competitor_ids
            .iter()
            .filter_map(|competitor_id| by_entity.get(competitor_id))
            .find(|entry| entry.presence == TriState::True)
        else {
            continue;
        };
        bullets.push(SwotBullet {
            text: format!(
                "Competitor coverage appears stronger for feature {}",
                feature_key
            ),
            source_refs: dedupe_refs(vec![competitor_feature.source_refs.clone()]),
        });
    }
    sort_and_cap_bullets(bullets)
}

fn build_price_opportunities_threats(
    target_entity_id: &str,
    competitor_ids: &[String],
    pricing_table: &[CompetitivePrice],
) -> (Vec<SwotBullet>, Vec<SwotBullet>) {
    let mut opportunities = Vec::new();
    let mut threats = Vec::new();

    let Some((target_price, target_currency, target_sources)) =
        representative_monthly_price(target_entity_id, pricing_table)
    else {
        return (opportunities, threats);
    };

    let mut lower_competitor_sources = Vec::new();
    let mut higher_competitor_sources = Vec::new();

    for competitor_id in competitor_ids {
        let Some((competitor_price, competitor_currency, competitor_sources)) =
            representative_monthly_price(competitor_id, pricing_table)
        else {
            continue;
        };
        if competitor_currency != target_currency {
            continue;
        }
        if competitor_price < target_price {
            lower_competitor_sources.push(competitor_sources);
        } else if competitor_price > target_price {
            higher_competitor_sources.push(competitor_sources);
        }
    }

    if !higher_competitor_sources.is_empty() {
        opportunities.push(SwotBullet {
            text: "Target price appears lower than at least one competitor in comparable currency".to_string(),
            source_refs: dedupe_refs(
                std::iter::once(target_sources.clone())
                    .chain(higher_competitor_sources)
                    .collect(),
            ),
        });
    }
    if !lower_competitor_sources.is_empty() {
        threats.push(SwotBullet {
            text: "At least one competitor appears lower-priced in comparable currency".to_string(),
            source_refs: dedupe_refs(
                std::iter::once(target_sources)
                    .chain(lower_competitor_sources)
                    .collect(),
            ),
        });
    }

    (sort_and_cap_bullets(opportunities), sort_and_cap_bullets(threats))
}

fn representative_monthly_price(
    entity_id: &str,
    pricing_table: &[CompetitivePrice],
) -> Option<(Decimal, String, Vec<String>)> {
    let candidates = pricing_table
        .iter()
        .filter(|entry| entry.entity_id == entity_id)
        .filter_map(|entry| {
            if entry.currency == "UNKNOWN" {
                return None;
            }
            let amount = monthly_equivalent(entry)?;
            Some((amount, entry.currency.clone(), entry.source_refs.clone()))
        })
        .collect::<Vec<(Decimal, String, Vec<String>)>>();

    let mut sorted = candidates;
    sorted.sort_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| left.1.cmp(&right.1))
            .then_with(|| left.2.cmp(&right.2))
    });
    sorted.into_iter().next()
}

fn monthly_equivalent(entry: &CompetitivePrice) -> Option<Decimal> {
    match entry.billing_period {
        BillingPeriod::Monthly | BillingPeriod::Unknown => Decimal::from_str(&entry.price_value).ok(),
        BillingPeriod::Yearly => {
            if let Some(normalized) = &entry.normalized_to {
                Decimal::from_str(normalized).ok()
            } else {
                Decimal::from_str(&entry.price_value)
                    .ok()
                    .map(|value| value / Decimal::from(12))
            }
        }
        BillingPeriod::OneTime | BillingPeriod::UsageBased => None,
    }
}

fn feature_map(
    feature_matrix: &[CompetitiveFeature],
) -> BTreeMap<String, BTreeMap<String, CompetitiveFeature>> {
    let mut by_key: BTreeMap<String, BTreeMap<String, CompetitiveFeature>> = BTreeMap::new();
    for feature in feature_matrix {
        by_key
            .entry(feature.feature_key.clone())
            .or_default()
            .insert(feature.entity_id.clone(), feature.clone());
    }
    by_key
}

fn dedupe_refs(ref_groups: Vec<Vec<String>>) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for group in ref_groups {
        for source_ref in group {
            if !source_ref.trim().is_empty() {
                refs.insert(source_ref);
            }
        }
    }
    refs.into_iter().collect()
}

fn sort_and_cap_bullets(mut bullets: Vec<SwotBullet>) -> Vec<SwotBullet> {
    bullets.sort_by(|left, right| left.text.cmp(&right.text));
    bullets.truncate(3);
    bullets
}
