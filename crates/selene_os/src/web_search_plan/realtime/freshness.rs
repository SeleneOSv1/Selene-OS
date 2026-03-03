#![forbid(unsafe_code)]

use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;

#[derive(Debug, Clone, PartialEq)]
pub struct FreshnessAssessment {
    pub age_ms: i64,
    pub ttl_ms: u64,
    pub freshness_score: f64,
    pub stale: bool,
}

pub fn evaluate(now_ms: i64, retrieved_at_ms: i64, ttl_ms: u64) -> Result<FreshnessAssessment, String> {
    if ttl_ms == 0 {
        return Err("ttl_ms must be > 0".to_string());
    }
    if retrieved_at_ms <= 0 {
        return Err("retrieved_at_ms must be > 0".to_string());
    }

    let age_ms = if now_ms >= retrieved_at_ms {
        now_ms - retrieved_at_ms
    } else {
        0
    };

    let ttl_decimal = Decimal::from(ttl_ms as i64);
    let age_decimal = Decimal::from(age_ms);
    let raw_ratio = age_decimal / ttl_decimal;
    let mut score = Decimal::ONE - raw_ratio;
    if score < Decimal::ZERO {
        score = Decimal::ZERO;
    }
    if score > Decimal::ONE {
        score = Decimal::ONE;
    }
    let rounded = score.round_dp(6);
    let freshness_score = rounded
        .to_f64()
        .ok_or_else(|| "failed converting freshness score to f64".to_string())?;

    Ok(FreshnessAssessment {
        age_ms,
        ttl_ms,
        freshness_score,
        stale: (age_ms as u64) > ttl_ms,
    })
}
