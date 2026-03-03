#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::domain_reputation_adjustment;
use crate::web_search_plan::trust::official_detector::TrustTier;
use crate::web_search_plan::trust::spam_signals::SpamSignals;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct TrustScoreBreakdown {
    pub trust_score: f64,
    pub base_score: f64,
    pub spam_penalty: f64,
    pub recency_bonus: f64,
    pub corroboration_bonus: f64,
    pub reputation_adjustment: f64,
    pub recency_available: bool,
}

pub fn compute_trust_score(
    trust_tier: TrustTier,
    spam_signals: &SpamSignals,
    host: Option<&str>,
    published_at_ms: Option<i64>,
    now_ms: i64,
    corroboration_count: usize,
) -> TrustScoreBreakdown {
    let base_score = base_score_for_tier(trust_tier);
    let spam_penalty = round6((spam_signals.spam_risk_score * 0.55).clamp(0.0, 1.0));
    let recency_bonus = recency_bonus(published_at_ms, now_ms);
    let corroboration_bonus = corroboration_bonus(corroboration_count);
    let reputation_adjustment = host
        .map(domain_reputation_adjustment)
        .unwrap_or(0.0);

    let score = (base_score - spam_penalty + recency_bonus + corroboration_bonus + reputation_adjustment)
        .clamp(0.0, 1.0);

    TrustScoreBreakdown {
        trust_score: round6(score),
        base_score: round6(base_score),
        spam_penalty,
        recency_bonus: round6(recency_bonus),
        corroboration_bonus: round6(corroboration_bonus),
        reputation_adjustment: round6(reputation_adjustment),
        recency_available: published_at_ms.is_some(),
    }
}

pub fn parse_published_at_ms(source: &Value) -> Option<i64> {
    if let Some(ms) = source.get("published_at_ms").and_then(Value::as_i64) {
        return Some(ms);
    }
    if let Some(ms) = source.get("published_at").and_then(Value::as_i64) {
        return Some(ms);
    }
    if let Some(raw) = source.get("published_at").and_then(Value::as_str) {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(ms) = trimmed.parse::<i64>() {
            return Some(ms);
        }
    }
    None
}

pub fn parse_corroboration_count(source: &Value) -> usize {
    if let Some(value) = source.get("corroboration_count").and_then(Value::as_u64) {
        return value as usize;
    }
    if let Some(value) = source
        .pointer("/metadata/corroboration_count")
        .and_then(Value::as_u64)
    {
        return value as usize;
    }
    0
}

fn base_score_for_tier(tier: TrustTier) -> f64 {
    match tier {
        TrustTier::Official => 0.95,
        TrustTier::High => 0.80,
        TrustTier::Medium => 0.60,
        TrustTier::Low => 0.30,
        TrustTier::Unknown => 0.15,
    }
}

fn recency_bonus(published_at_ms: Option<i64>, now_ms: i64) -> f64 {
    let Some(published_at_ms) = published_at_ms else {
        return 0.0;
    };
    if now_ms <= 0 || published_at_ms <= 0 {
        return 0.0;
    }
    let age_ms = if now_ms >= published_at_ms {
        now_ms - published_at_ms
    } else {
        0
    };
    let day_ms = 24_i64 * 60_i64 * 60_i64 * 1_000_i64;
    if age_ms <= day_ms {
        0.08
    } else if age_ms <= 7 * day_ms {
        0.05
    } else if age_ms <= 30 * day_ms {
        0.02
    } else {
        0.0
    }
}

fn corroboration_bonus(corroboration_count: usize) -> f64 {
    match corroboration_count {
        0 | 1 => 0.0,
        2 => 0.02,
        3 => 0.03,
        _ => 0.04,
    }
}

fn round6(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}
