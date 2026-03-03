#![forbid(unsafe_code)]

use crate::web_search_plan::analytics::decimal::{decimal_to_string, round_decimal};
use crate::web_search_plan::competitive::entity_normalize::EntityIndex;
use crate::web_search_plan::competitive::schema::{
    BillingPeriod, CompetitiveError, CompetitiveErrorKind, CompetitivePrice, TriState,
};
use crate::web_search_plan::structured::types::{StructuredRow, StructuredValue};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PricingBuildResult {
    pub pricing_table: Vec<CompetitivePrice>,
    pub uncertainty_flags: Vec<String>,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone)]
struct PriceAccumulator {
    amount: Option<Decimal>,
    currency: Option<String>,
    billing_period: BillingPeriod,
    tax_included: TriState,
    source_refs: BTreeSet<String>,
}

impl Default for PriceAccumulator {
    fn default() -> Self {
        Self {
            amount: None,
            currency: None,
            billing_period: BillingPeriod::Unknown,
            tax_included: TriState::Unknown,
            source_refs: BTreeSet::new(),
        }
    }
}

pub fn build_pricing_table(
    rows: &[StructuredRow],
    entities: &EntityIndex,
    allowed_source_refs: &BTreeSet<String>,
    evidence_packet: &serde_json::Value,
) -> Result<PricingBuildResult, CompetitiveError> {
    let mut accumulators: BTreeMap<(String, String, BillingPeriod), PriceAccumulator> =
        BTreeMap::new();
    let mut reason_codes = BTreeSet::new();
    let mut uncertainty_flags = BTreeSet::new();

    for row in rows {
        if !is_pricing_row(row) {
            continue;
        }
        let Some(entity_id) = entities.entity_id_for_name(row.entity.as_str()) else {
            continue;
        };

        let source_ref = row.source_ref.trim().to_string();
        if source_ref.is_empty() {
            return Err(CompetitiveError::new(
                CompetitiveErrorKind::PolicyViolation,
                format!(
                    "pricing row {}:{} must include source_ref",
                    row.entity, row.attribute
                ),
            ));
        }
        if !allowed_source_refs.contains(&source_ref) {
            return Err(CompetitiveError::new(
                CompetitiveErrorKind::CitationMismatch,
                format!(
                    "pricing source_ref {} is not present in evidence packet",
                    source_ref
                ),
            ));
        }

        let (amount, currency) = parse_price_value(&row.value);
        let billing_period = billing_period_from_row(row);
        let tax_included = tax_included_from_row(row);

        let currency_value = currency
            .clone()
            .unwrap_or_else(|| "UNKNOWN".to_string())
            .to_ascii_uppercase();
        let key = (entity_id.to_string(), currency_value, billing_period);
        let entry = accumulators.entry(key).or_default();

        entry.billing_period = billing_period;
        entry.tax_included = merge_tristate(entry.tax_included, tax_included);
        if let Some(parsed_amount) = amount {
            entry.amount = match entry.amount {
                Some(existing) => Some(round_decimal((existing + parsed_amount) / Decimal::from(2))),
                None => Some(parsed_amount),
            };
        }
        entry.currency = currency
            .clone()
            .map(|value| value.to_ascii_uppercase())
            .or_else(|| entry.currency.clone());
        entry.source_refs.insert(source_ref);

        if currency.is_none() {
            reason_codes.insert("insufficient_evidence".to_string());
            uncertainty_flags.insert("currency_missing".to_string());
        }
    }

    let mut pricing_table = Vec::with_capacity(accumulators.len());
    for ((entity_id, currency_key, billing_period), accumulator) in accumulators {
        let amount = accumulator.amount.unwrap_or(Decimal::ZERO);
        let normalized_to = if billing_period == BillingPeriod::Yearly {
            Some(decimal_to_string(round_decimal(amount / Decimal::from(12))))
        } else {
            None
        };

        pricing_table.push(CompetitivePrice {
            entity_id,
            price_value: decimal_to_string(amount),
            currency: accumulator
                .currency
                .unwrap_or_else(|| currency_key.to_ascii_uppercase()),
            billing_period,
            tax_included: accumulator.tax_included,
            normalized_to,
            source_refs: accumulator.source_refs.into_iter().collect(),
        });
    }

    pricing_table.sort_by(|left, right| {
        left.entity_id
            .cmp(&right.entity_id)
            .then_with(|| left.billing_period.cmp(&right.billing_period))
            .then_with(|| left.currency.cmp(&right.currency))
            .then_with(|| left.price_value.cmp(&right.price_value))
            .then_with(|| left.source_refs.cmp(&right.source_refs))
    });

    let currencies: BTreeSet<String> = pricing_table
        .iter()
        .map(|entry| entry.currency.clone())
        .filter(|currency| currency != "UNKNOWN")
        .collect();
    if currencies.len() > 1 && !has_currency_rates(evidence_packet) {
        uncertainty_flags.insert("non_comparable_currency".to_string());
        reason_codes.insert("insufficient_evidence".to_string());
    }

    Ok(PricingBuildResult {
        pricing_table,
        uncertainty_flags: uncertainty_flags.into_iter().collect(),
        reason_codes: reason_codes.into_iter().collect(),
    })
}

fn is_pricing_row(row: &StructuredRow) -> bool {
    let attribute = row.attribute.to_lowercase();
    let unit = row.unit.clone().unwrap_or_default().to_lowercase();
    attribute.contains("price")
        || attribute.contains("cost")
        || attribute.contains("billing")
        || unit.contains("month")
        || unit.contains("year")
        || matches!(row.value, StructuredValue::Currency { .. })
}

fn parse_price_value(value: &StructuredValue) -> (Option<Decimal>, Option<String>) {
    match value {
        StructuredValue::Currency {
            amount,
            currency_code,
        } => (
            rust_decimal::Decimal::from_f64_retain(*amount).map(round_decimal),
            Some(currency_code.trim().to_ascii_uppercase()),
        ),
        StructuredValue::Float { value } => (
            rust_decimal::Decimal::from_f64_retain(*value).map(round_decimal),
            None,
        ),
        StructuredValue::Int { value } => (Some(Decimal::from(*value)), None),
        _ => (None, None),
    }
}

fn billing_period_from_row(row: &StructuredRow) -> BillingPeriod {
    let unit = row.unit.clone().unwrap_or_default().to_lowercase();
    let attribute = row.attribute.to_lowercase();
    let combined = format!("{} {}", unit, attribute);
    if combined.contains("month") || combined.contains("monthly") {
        BillingPeriod::Monthly
    } else if combined.contains("year") || combined.contains("annual") {
        BillingPeriod::Yearly
    } else if combined.contains("one time") || combined.contains("one_time") {
        BillingPeriod::OneTime
    } else if combined.contains("usage") || combined.contains("per unit") {
        BillingPeriod::UsageBased
    } else {
        BillingPeriod::Unknown
    }
}

fn tax_included_from_row(row: &StructuredRow) -> TriState {
    let attribute = row.attribute.to_lowercase();
    if attribute.contains("tax included") || attribute.contains("incl tax") {
        TriState::True
    } else if attribute.contains("tax excluded") || attribute.contains("excl tax") {
        TriState::False
    } else {
        TriState::Unknown
    }
}

fn merge_tristate(current: TriState, incoming: TriState) -> TriState {
    match (current, incoming) {
        (TriState::Unknown, next) => next,
        (existing, TriState::Unknown) => existing,
        (TriState::True, TriState::True) => TriState::True,
        (TriState::False, TriState::False) => TriState::False,
        _ => TriState::Unknown,
    }
}

fn has_currency_rates(evidence_packet: &serde_json::Value) -> bool {
    evidence_packet
        .pointer("/trust_metadata/analytics/currency_rates")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|entries| !entries.is_empty())
}
