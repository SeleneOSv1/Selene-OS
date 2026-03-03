#![forbid(unsafe_code)]

use crate::web_search_plan::trust::domain_rules::{
    HIGH_ALLOWLIST, LOW_ALLOWLIST, MEDIUM_ALLOWLIST, OFFICIAL_ALLOWLIST,
};
use url::Url;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfficialDetection {
    pub trust_tier: TrustTier,
    pub official_source: bool,
    pub host: Option<String>,
    pub reasons: Vec<String>,
}

pub fn detect_from_url(url: &str) -> OfficialDetection {
    let host = Url::parse(url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|value| value.to_ascii_lowercase()));
    let Some(host) = host else {
        return OfficialDetection {
            trust_tier: TrustTier::Unknown,
            official_source: false,
            host: None,
            reasons: vec!["INVALID_URL".to_string()],
        };
    };

    let mut reasons = Vec::new();
    if is_official_host(host.as_str()) {
        reasons.push("OFFICIAL_DOMAIN".to_string());
        return OfficialDetection {
            trust_tier: TrustTier::Official,
            official_source: true,
            host: Some(host),
            reasons,
        };
    }
    if is_high_host(host.as_str()) {
        reasons.push("HIGH_REPUTATION_DOMAIN".to_string());
        return OfficialDetection {
            trust_tier: TrustTier::High,
            official_source: false,
            host: Some(host),
            reasons,
        };
    }
    if is_medium_host(host.as_str()) {
        reasons.push("MEDIUM_REPUTATION_DOMAIN".to_string());
        return OfficialDetection {
            trust_tier: TrustTier::Medium,
            official_source: false,
            host: Some(host),
            reasons,
        };
    }
    if is_low_host(host.as_str()) || host.contains("blog") || host.ends_with(".substack.com") {
        reasons.push("LOW_REPUTATION_DOMAIN".to_string());
        return OfficialDetection {
            trust_tier: TrustTier::Low,
            official_source: false,
            host: Some(host),
            reasons,
        };
    }

    reasons.push("UNKNOWN_DOMAIN".to_string());
    OfficialDetection {
        trust_tier: TrustTier::Unknown,
        official_source: false,
        host: Some(host),
        reasons,
    }
}

fn is_official_host(host: &str) -> bool {
    OFFICIAL_ALLOWLIST
        .iter()
        .any(|suffix| host == *suffix || host.ends_with(&format!(".{}", suffix)))
        || host.ends_with(".gov")
        || host.contains(".gov.")
        || host.ends_with(".gov.sg")
        || host.ends_with(".gov.uk")
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
