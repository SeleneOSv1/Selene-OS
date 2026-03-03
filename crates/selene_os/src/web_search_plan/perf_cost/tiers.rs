#![forbid(unsafe_code)]

pub const PERF_COST_POLICY_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImportanceTier {
    Low,
    Medium,
    High,
}

impl ImportanceTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }

    pub fn parse(raw: &str) -> Result<Self, String> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            other => Err(format!("unsupported importance_tier {}", other)),
        }
    }

    pub fn parse_or_default(raw: &str) -> Self {
        Self::parse(raw).unwrap_or_default()
    }
}

impl Default for ImportanceTier {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TierCaps {
    pub max_results_from_search: usize,
    pub max_queries: usize,
    pub max_urls_opened_per_query: usize,
    pub max_total_extracted_chars: usize,
    pub max_chunks_total: usize,
    pub timeout_per_provider_ms: u64,
    pub total_timeout_per_turn_ms: u64,
    pub url_fetch_total_timeout_ms: u64,
    pub max_concurrent_fetches: usize,
    pub max_fallback_invocations_per_turn: usize,
    pub max_total_provider_calls_per_turn: usize,
    pub max_retries_per_provider: usize,
}

impl TierCaps {
    pub const fn minimum_search_results() -> usize {
        3
    }
}

pub const LOW_TIER_CAPS: TierCaps = TierCaps {
    max_results_from_search: 3,
    max_queries: 2,
    max_urls_opened_per_query: 1,
    max_total_extracted_chars: 60_000,
    max_chunks_total: 96,
    timeout_per_provider_ms: 1_000,
    total_timeout_per_turn_ms: 3_500,
    url_fetch_total_timeout_ms: 1_800,
    max_concurrent_fetches: 1,
    max_fallback_invocations_per_turn: 1,
    max_total_provider_calls_per_turn: 3,
    max_retries_per_provider: 0,
};

pub const MEDIUM_TIER_CAPS: TierCaps = TierCaps {
    max_results_from_search: 5,
    max_queries: 3,
    max_urls_opened_per_query: 2,
    max_total_extracted_chars: 120_000,
    max_chunks_total: 192,
    timeout_per_provider_ms: 2_000,
    total_timeout_per_turn_ms: 7_000,
    url_fetch_total_timeout_ms: 4_000,
    max_concurrent_fetches: 2,
    max_fallback_invocations_per_turn: 1,
    max_total_provider_calls_per_turn: 5,
    max_retries_per_provider: 0,
};

pub const HIGH_TIER_CAPS: TierCaps = TierCaps {
    max_results_from_search: 10,
    max_queries: 4,
    max_urls_opened_per_query: 3,
    max_total_extracted_chars: 240_000,
    max_chunks_total: 320,
    timeout_per_provider_ms: 3_000,
    total_timeout_per_turn_ms: 12_000,
    url_fetch_total_timeout_ms: 7_000,
    max_concurrent_fetches: 3,
    max_fallback_invocations_per_turn: 1,
    max_total_provider_calls_per_turn: 7,
    max_retries_per_provider: 0,
};

pub const fn caps_for_tier(tier: ImportanceTier) -> TierCaps {
    match tier {
        ImportanceTier::Low => LOW_TIER_CAPS,
        ImportanceTier::Medium => MEDIUM_TIER_CAPS,
        ImportanceTier::High => HIGH_TIER_CAPS,
    }
}
