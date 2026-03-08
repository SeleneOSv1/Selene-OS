#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::domain_reputation_adjustment;
use crate::web_search_plan::trust::official_detector::TrustTier;
use crate::web_search_plan::trust::spam_signals::SpamSignals;
use selene_engines::ph1comp::compute_trust_score_bp;
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
    let posture = compute_trust_score_bp(
        base_score_bp_for_tier(trust_tier),
        ratio_to_bp(spam_signals.spam_risk_score),
        published_at_ms,
        now_ms,
        corroboration_count,
        signed_ratio_to_bp(host.map(domain_reputation_adjustment).unwrap_or(0.0)),
    );

    TrustScoreBreakdown {
        trust_score: bp_to_ratio(posture.trust_score_bp),
        base_score: bp_to_ratio(posture.base_score_bp),
        spam_penalty: bp_to_ratio(posture.spam_penalty_bp),
        recency_bonus: bp_to_ratio(posture.recency_bonus_bp),
        corroboration_bonus: bp_to_ratio(posture.corroboration_bonus_bp),
        reputation_adjustment: signed_bp_to_ratio(posture.reputation_adjustment_bp),
        recency_available: posture.recency_available,
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

fn base_score_bp_for_tier(tier: TrustTier) -> u16 {
    match tier {
        TrustTier::Official => 9_500,
        TrustTier::High => 8_000,
        TrustTier::Medium => 6_000,
        TrustTier::Low => 3_000,
        TrustTier::Unknown => 1_500,
    }
}

fn bp_to_ratio(value_bp: u16) -> f64 {
    (value_bp as f64) / 10_000.0
}

fn signed_bp_to_ratio(value_bp: i16) -> f64 {
    (value_bp as f64) / 10_000.0
}

fn ratio_to_bp(value: f64) -> u16 {
    (value.clamp(0.0, 1.0) * 10_000.0).round() as u16
}

fn signed_ratio_to_bp(value: f64) -> i16 {
    (value.clamp(-1.0, 1.0) * 10_000.0).round() as i16
}
