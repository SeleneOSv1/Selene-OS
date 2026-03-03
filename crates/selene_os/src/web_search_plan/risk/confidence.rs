#![forbid(unsafe_code)]

use crate::web_search_plan::risk::calibration::{bps_to_decimal, CONFIDENCE_LOW_THRESHOLD_BPS, ROUNDING_SCALE};
use crate::web_search_plan::risk::factors::FactorScore;
use rust_decimal::{Decimal, RoundingStrategy};
use serde_json::Value;
use std::collections::BTreeMap;

const ROUNDING: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfidenceOutput {
    pub confidence_score: Decimal,
    pub confidence_low: bool,
    pub sample_component: Decimal,
    pub trust_component: Decimal,
    pub freshness_component: Decimal,
    pub conflict_component: Decimal,
}

pub fn build_confidence(
    factors: &[FactorScore],
    evidence_packet: &Value,
    computation_packet: Option<&Value>,
    contradiction_present: bool,
) -> ConfidenceOutput {
    let evidence_refs = factors
        .iter()
        .flat_map(|factor| factor.evidence_refs.iter().cloned())
        .collect::<Vec<String>>();

    let sample_component = sample_component(evidence_refs.len() as u32);
    let trust_component = trust_component(evidence_packet, evidence_refs.as_slice());
    let freshness_component = freshness_component(evidence_packet, evidence_refs.as_slice());
    let conflict_component = conflict_component(contradiction_present, computation_packet);

    let score = (sample_component * Decimal::new(35, 2)
        + trust_component * Decimal::new(25, 2)
        + freshness_component * Decimal::new(20, 2)
        + conflict_component * Decimal::new(20, 2))
    .round_dp_with_strategy(ROUNDING_SCALE, ROUNDING);

    let threshold = bps_to_decimal(CONFIDENCE_LOW_THRESHOLD_BPS);
    ConfidenceOutput {
        confidence_score: clamp_unit(score),
        confidence_low: score < threshold,
        sample_component,
        trust_component,
        freshness_component,
        conflict_component,
    }
}

fn sample_component(reference_count: u32) -> Decimal {
    let capped = reference_count.min(10);
    (Decimal::from(capped) / Decimal::from(10u32)).round_dp_with_strategy(ROUNDING_SCALE, ROUNDING)
}

fn trust_component(evidence_packet: &Value, evidence_refs: &[String]) -> Decimal {
    let mut trust_by_url = BTreeMap::new();
    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            let Some(url) = source.get("url").and_then(Value::as_str) else {
                continue;
            };
            let tier = source
                .get("trust_tier")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            trust_by_url.insert(url.to_string(), tier_weight(tier));
        }
    }

    let mut total = Decimal::ZERO;
    let mut count = 0u32;
    for evidence_ref in evidence_refs {
        if let Some(weight) = trust_by_url.get(evidence_ref) {
            total += *weight;
            count = count.saturating_add(1);
        }
    }
    if count == 0 {
        Decimal::new(6, 1)
    } else {
        (total / Decimal::from(count)).round_dp_with_strategy(ROUNDING_SCALE, ROUNDING)
    }
}

fn freshness_component(evidence_packet: &Value, evidence_refs: &[String]) -> Decimal {
    let mut freshness_by_url = BTreeMap::new();
    if let Some(sources) = evidence_packet.get("sources").and_then(Value::as_array) {
        for source in sources {
            let Some(url) = source.get("url").and_then(Value::as_str) else {
                continue;
            };
            let freshness = source
                .get("freshness_score")
                .and_then(parse_decimal)
                .unwrap_or(Decimal::new(5, 1));
            freshness_by_url.insert(url.to_string(), clamp_unit(freshness));
        }
    }

    let mut total = Decimal::ZERO;
    let mut count = 0u32;
    for evidence_ref in evidence_refs {
        if let Some(value) = freshness_by_url.get(evidence_ref) {
            total += *value;
            count = count.saturating_add(1);
        }
    }
    if count == 0 {
        Decimal::new(5, 1)
    } else {
        (total / Decimal::from(count)).round_dp_with_strategy(ROUNDING_SCALE, ROUNDING)
    }
}

fn conflict_component(contradiction_present: bool, computation_packet: Option<&Value>) -> Decimal {
    let mut component = if contradiction_present {
        Decimal::new(6, 1)
    } else {
        Decimal::ONE
    };

    if let Some(consensus) = computation_packet
        .and_then(|packet| packet.get("consensus"))
        .and_then(Value::as_array)
    {
        let has_unresolved = consensus.iter().any(|group| {
            group.get("chosen").is_none()
                || group
                    .get("outliers")
                    .and_then(Value::as_array)
                    .map(|outliers| !outliers.is_empty())
                    .unwrap_or(false)
        });
        if has_unresolved {
            component = component.min(Decimal::new(7, 1));
        }
    }

    clamp_unit(component)
}

fn tier_weight(raw: &str) -> Decimal {
    match raw.to_ascii_lowercase().as_str() {
        "official" => Decimal::ONE,
        "high" => Decimal::new(9, 1),
        "medium" => Decimal::new(7, 1),
        "low" => Decimal::new(4, 1),
        _ => Decimal::new(3, 1),
    }
}

fn parse_decimal(value: &Value) -> Option<Decimal> {
    value
        .as_f64()
        .and_then(Decimal::from_f64_retain)
        .or_else(|| value.as_str().and_then(|raw| raw.parse::<Decimal>().ok()))
        .or_else(|| value.as_i64().map(Decimal::from))
        .or_else(|| value.as_u64().map(Decimal::from))
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
