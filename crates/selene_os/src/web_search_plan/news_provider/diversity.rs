#![forbid(unsafe_code)]

use crate::web_search_plan::news_provider::recency_policy::ImportanceTier;
use crate::web_search_plan::news_provider::NormalizedNewsResult;
use std::collections::BTreeSet;

pub const HIGH_TIER_MIN_DISTINCT_DOMAINS: usize = 2;

pub fn distinct_domain_count(results: &[NormalizedNewsResult]) -> usize {
    let mut domains = BTreeSet::new();
    for result in results {
        if !result.domain.is_empty() {
            domains.insert(result.domain.clone());
        }
    }
    domains.len()
}

pub fn diversity_threshold_for_tier(tier: ImportanceTier) -> Option<usize> {
    match tier {
        ImportanceTier::High => Some(HIGH_TIER_MIN_DISTINCT_DOMAINS),
        ImportanceTier::Low | ImportanceTier::Medium => None,
    }
}

pub fn diversity_threshold_met(tier: ImportanceTier, domain_count: usize) -> bool {
    match diversity_threshold_for_tier(tier) {
        Some(threshold) => domain_count >= threshold,
        None => true,
    }
}
