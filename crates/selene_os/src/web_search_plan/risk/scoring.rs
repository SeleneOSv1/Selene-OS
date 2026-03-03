#![forbid(unsafe_code)]

use crate::web_search_plan::risk::calibration::{risk_thresholds, ROUNDING_SCALE};
use crate::web_search_plan::risk::factors::FactorScore;
use rust_decimal::{Decimal, RoundingStrategy};

pub const RISK_ROUNDING_STRATEGY: RoundingStrategy = RoundingStrategy::MidpointAwayFromZero;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl RiskLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositeRiskScore {
    pub risk_score: Decimal,
    pub risk_level: RiskLevel,
    pub weight_sum: Decimal,
}

pub fn compute_composite_risk(factors: &[FactorScore]) -> Result<CompositeRiskScore, String> {
    if factors.is_empty() {
        return Err("insufficient_evidence: no factor scores available".to_string());
    }

    let mut numerator = Decimal::ZERO;
    let mut denominator = Decimal::ZERO;
    for factor in factors {
        numerator += factor.score * factor.weight;
        denominator += factor.weight;
    }

    if denominator.is_zero() {
        return Err("insufficient_evidence: no present factor weights".to_string());
    }

    let raw = numerator / denominator;
    let risk_score = clamp_unit(raw).round_dp_with_strategy(ROUNDING_SCALE, RISK_ROUNDING_STRATEGY);
    let risk_level = classify_risk_level(risk_score);

    Ok(CompositeRiskScore {
        risk_score,
        risk_level,
        weight_sum: denominator,
    })
}

pub fn classify_risk_level(risk_score: Decimal) -> RiskLevel {
    let thresholds = risk_thresholds();
    let scaled_bps = (clamp_unit(risk_score) * Decimal::from(10_000u32))
        .round_dp_with_strategy(0, RISK_ROUNDING_STRATEGY)
        .to_u32()
        .unwrap_or(0);

    if scaled_bps <= thresholds.low_max_bps {
        RiskLevel::Low
    } else if scaled_bps <= thresholds.medium_max_bps {
        RiskLevel::Medium
    } else {
        RiskLevel::High
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

trait DecimalExt {
    fn to_u32(self) -> Option<u32>;
}

impl DecimalExt for Decimal {
    fn to_u32(self) -> Option<u32> {
        rust_decimal::prelude::ToPrimitive::to_u32(&self)
    }
}
