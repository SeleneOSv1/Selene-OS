#![forbid(unsafe_code)]

use crate::web_search_plan::cache::cache_key::CacheMode;
use crate::web_search_plan::perf_cost::tiers::ImportanceTier;

pub fn ttl_ms_for(mode: CacheMode, tier: ImportanceTier) -> i64 {
    match (mode, tier) {
        (CacheMode::UrlFetch, ImportanceTier::Low) => 30_000,
        (CacheMode::UrlFetch, ImportanceTier::Medium) => 60_000,
        (CacheMode::UrlFetch, ImportanceTier::High) => 120_000,
        (CacheMode::Web, ImportanceTier::Low) => 45_000,
        (CacheMode::Web, ImportanceTier::Medium) => 90_000,
        (CacheMode::Web, ImportanceTier::High) => 180_000,
        (CacheMode::News, ImportanceTier::Low) => 20_000,
        (CacheMode::News, ImportanceTier::Medium) => 45_000,
        (CacheMode::News, ImportanceTier::High) => 90_000,
        (CacheMode::Images, ImportanceTier::Low) => 30_000,
        (CacheMode::Images, ImportanceTier::Medium) => 60_000,
        (CacheMode::Images, ImportanceTier::High) => 120_000,
        (CacheMode::Video, ImportanceTier::Low) => 20_000,
        (CacheMode::Video, ImportanceTier::Medium) => 40_000,
        (CacheMode::Video, ImportanceTier::High) => 80_000,
    }
}
