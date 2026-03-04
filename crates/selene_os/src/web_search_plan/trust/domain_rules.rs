#![forbid(unsafe_code)]

use url::Url;

pub const TRUST_MODEL_VERSION: &str = "1.0.0";
pub const SPAM_SIGNAL_VERSION: &str = "1.0.0";

pub const OFFICIAL_ALLOWLIST: &[&str] = &[
    "sec.gov",
    "mas.gov.sg",
    "europa.eu",
    "bafin.de",
    "gov.sg",
];

pub const HIGH_ALLOWLIST: &[&str] = &[
    "iso.org",
    "nist.gov",
    "ieee.org",
    "standards.org",
    "oecd.org",
];

pub const MEDIUM_ALLOWLIST: &[&str] = &[
    "reuters.com",
    "bloomberg.com",
    "ft.com",
    "wsj.com",
    "cnbc.com",
];

pub const LOW_ALLOWLIST: &[&str] = &[
    "reddit.com",
    "medium.com",
    "blogspot.com",
    "wordpress.com",
];

pub const CLICKBAIT_KEYWORDS: &[&str] = &[
    "shocking",
    "unbelievable",
    "you won't believe",
    "you wont believe",
    "must see",
    "click here",
    "viral",
    "breaking!!!",
];

pub const TRACKING_QUERY_PARAMS: &[&str] = &[
    "utm_source",
    "utm_medium",
    "utm_campaign",
    "utm_term",
    "utm_content",
    "gclid",
    "fbclid",
    "msclkid",
    "dclid",
    "yclid",
    "mc_cid",
    "mc_eid",
    "ref",
    "ref_src",
    "cmpid",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CanonicalTrustTier {
    Unknown,
    Low,
    Medium,
    High,
    Official,
}

impl CanonicalTrustTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Official => "OFFICIAL",
            Self::High => "HIGH",
            Self::Medium => "MEDIUM",
            Self::Low => "LOW",
            Self::Unknown => "UNKNOWN",
        }
    }
}

pub fn domain_reputation_adjustment(host: &str) -> f64 {
    match host {
        "sec.gov" | "mas.gov.sg" | "europa.eu" | "bafin.de" => 0.06,
        "reuters.com" | "bloomberg.com" | "wsj.com" | "ft.com" => 0.04,
        "reddit.com" | "medium.com" | "blogspot.com" | "wordpress.com" => -0.05,
        _ => 0.0,
    }
}

pub fn classify_trust_tier(url: &str) -> CanonicalTrustTier {
    let Some(host) = parse_host(url) else {
        return CanonicalTrustTier::Unknown;
    };
    classify_host_trust_tier(host.as_str())
}

pub fn classify_host_trust_tier(host: &str) -> CanonicalTrustTier {
    if is_official_host(host) {
        return CanonicalTrustTier::Official;
    }
    if is_high_host(host) {
        return CanonicalTrustTier::High;
    }
    if is_medium_host(host) {
        return CanonicalTrustTier::Medium;
    }
    if is_low_host(host) || host.contains("blog") || host.ends_with(".substack.com") {
        return CanonicalTrustTier::Low;
    }
    CanonicalTrustTier::Unknown
}

pub fn parse_trust_tier_literal(raw: &str) -> Option<CanonicalTrustTier> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "official" => Some(CanonicalTrustTier::Official),
        "high" => Some(CanonicalTrustTier::High),
        "medium" => Some(CanonicalTrustTier::Medium),
        "low" => Some(CanonicalTrustTier::Low),
        "unknown" => Some(CanonicalTrustTier::Unknown),
        _ => None,
    }
}

pub fn parse_host(url: &str) -> Option<String> {
    Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()))
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
