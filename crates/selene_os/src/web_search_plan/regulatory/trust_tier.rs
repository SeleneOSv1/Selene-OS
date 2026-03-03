#![forbid(unsafe_code)]

use serde_json::Value;
use url::Url;

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

const OFFICIAL_ALLOWLIST: &[&str] = &[
    "sec.gov",
    "mas.gov.sg",
    "europa.eu",
    "bafin.de",
    "gov.sg",
];

const HIGH_ALLOWLIST: &[&str] = &[
    "iso.org",
    "nist.gov",
    "ieee.org",
    "standards.org",
    "oecd.org",
];

const MEDIUM_ALLOWLIST: &[&str] = &[
    "reuters.com",
    "bloomberg.com",
    "ft.com",
    "wsj.com",
    "cnbc.com",
];

const LOW_ALLOWLIST: &[&str] = &[
    "reddit.com",
    "medium.com",
    "blogspot.com",
    "wordpress.com",
];

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
    let host = Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()));
    let Some(host) = host else {
        return TrustTier::Unknown;
    };

    if is_official_host(host.as_str()) {
        return TrustTier::Official;
    }
    if is_high_host(host.as_str()) {
        return TrustTier::High;
    }
    if is_medium_host(host.as_str()) {
        return TrustTier::Medium;
    }
    if is_low_host(host.as_str()) {
        return TrustTier::Low;
    }
    if host.contains("blog") || host.ends_with(".substack.com") {
        return TrustTier::Low;
    }
    TrustTier::Unknown
}

fn is_official_host(host: &str) -> bool {
    OFFICIAL_ALLOWLIST
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)))
        || host.ends_with(".gov")
        || host.contains(".gov.")
}

fn is_high_host(host: &str) -> bool {
    HIGH_ALLOWLIST
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)))
        || host.ends_with(".edu")
        || host.contains(".edu.")
}

fn is_medium_host(host: &str) -> bool {
    MEDIUM_ALLOWLIST
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)))
}

fn is_low_host(host: &str) -> bool {
    LOW_ALLOWLIST
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)))
}

fn map_trust_tier_literal(raw: &str) -> Option<TrustTier> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "official" => Some(TrustTier::Official),
        "high" => Some(TrustTier::High),
        "medium" => Some(TrustTier::Medium),
        "low" => Some(TrustTier::Low),
        "unknown" => Some(TrustTier::Unknown),
        _ => None,
    }
}
