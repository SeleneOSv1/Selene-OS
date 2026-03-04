#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::{
    classify_trust_tier, parse_trust_tier_literal, CanonicalTrustTier,
};
use serde_json::Value;

pub const TRUST_TIER_POLICY_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TrustTier {
    Unknown,
    Low,
    Medium,
    High,
    Official,
}

impl TrustTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "OFFICIAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::Unknown => "UNKNOWN",
        }
    }

    pub const fn rank(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Official => 4,
        }
    }

    pub fn parse_threshold(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "official" => Some(Self::Official),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "unknown" => Some(Self::Unknown),
            _ => None,
        }
    }
}

pub fn classify_source(source: &Value) -> TrustTier {
    if let Some(url) = source.get("url").and_then(Value::as_str) {
        let from_url = classify_url(url);
        if from_url != TrustTier::Unknown {
            return from_url;
        }
    }

    source
        .get("trust_tier")
        .and_then(Value::as_str)
        .and_then(map_trust_tier_literal)
        .unwrap_or(TrustTier::Unknown)
}

pub fn classify_url(url: &str) -> TrustTier {
    map_from_canonical(classify_trust_tier(url))
}

fn map_trust_tier_literal(raw: &str) -> Option<TrustTier> {
    parse_trust_tier_literal(raw).map(map_from_canonical)
}

fn map_from_canonical(tier: CanonicalTrustTier) -> TrustTier {
    match tier {
        CanonicalTrustTier::Official => TrustTier::Official,
        CanonicalTrustTier::High => TrustTier::High,
        CanonicalTrustTier::Medium => TrustTier::Medium,
        CanonicalTrustTier::Low => TrustTier::Low,
        CanonicalTrustTier::Unknown => TrustTier::Unknown,
    }
}
