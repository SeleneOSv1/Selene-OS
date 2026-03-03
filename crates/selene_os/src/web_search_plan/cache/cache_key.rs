#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const CACHE_KEY_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CacheMode {
    Web,
    News,
    Images,
    Video,
    UrlFetch,
}

impl CacheMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::News => "news",
            Self::Images => "images",
            Self::Video => "video",
            Self::UrlFetch => "url_fetch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    pub cache_key_version: String,
    pub mode: CacheMode,
    pub canonical_query: String,
    pub canonical_url: Option<String>,
    pub provider_id: Option<String>,
    pub importance_tier: String,
    pub policy_snapshot_id: String,
}

impl CacheKey {
    pub fn new(
        mode: CacheMode,
        query: &str,
        canonical_url: Option<&str>,
        provider_id: Option<&str>,
        tier: ImportanceTier,
        policy_snapshot_id: Option<&str>,
    ) -> Self {
        Self {
            cache_key_version: CACHE_KEY_VERSION.to_string(),
            mode,
            canonical_query: canonicalize_query(query),
            canonical_url: canonical_url
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_ascii_lowercase()),
            provider_id: provider_id
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| v.to_ascii_lowercase()),
            importance_tier: tier.as_str().to_string(),
            policy_snapshot_id: policy_snapshot_id
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .unwrap_or("none")
                .to_string(),
        }
    }

    pub fn stable_key_string(&self) -> String {
        format!(
            "{}|{}|{}|{}|{}|{}|{}",
            self.cache_key_version,
            self.mode.as_str(),
            self.canonical_query,
            self.canonical_url.as_deref().unwrap_or("none"),
            self.provider_id.as_deref().unwrap_or("none"),
            self.importance_tier,
            self.policy_snapshot_id,
        )
    }

    pub fn stable_key_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self.stable_key_string().as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

pub fn canonicalize_query(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_ascii_lowercase()
}
