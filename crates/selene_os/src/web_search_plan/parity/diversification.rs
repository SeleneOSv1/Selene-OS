#![forbid(unsafe_code)]

use std::collections::BTreeSet;

pub const DIVERSIFICATION_POLICY_VERSION: &str = "run33-diversification-v1";
pub const HIGH_TIER_MIN_DISTINCT_DOMAINS: usize = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiversificationOutcome<T> {
    pub reordered: Vec<T>,
    pub distinct_domain_count: usize,
    pub threshold_met: bool,
    pub limitation_flag: bool,
}

pub fn diversify_for_high_tier<T, F>(
    items: &[T],
    importance_tier: &str,
    min_distinct_domains: usize,
    domain_of: F,
) -> DiversificationOutcome<T>
where
    T: Clone,
    F: Fn(&T) -> String,
{
    let normalized_tier = importance_tier.trim().to_ascii_lowercase();
    let all_domain_count = count_distinct_domains(items, &domain_of);
    if normalized_tier != "high" {
        return DiversificationOutcome {
            reordered: items.to_vec(),
            distinct_domain_count: all_domain_count,
            threshold_met: true,
            limitation_flag: false,
        };
    }

    let target = min_distinct_domains.max(1);
    let mut seen_domains = BTreeSet::new();
    let mut selected_indexes = BTreeSet::new();
    let mut reordered = Vec::with_capacity(items.len());

    for (index, item) in items.iter().enumerate() {
        let domain = domain_of(item);
        if seen_domains.len() >= target {
            break;
        }
        if seen_domains.insert(domain) {
            selected_indexes.insert(index);
            reordered.push(item.clone());
        }
    }

    for (index, item) in items.iter().enumerate() {
        if selected_indexes.contains(&index) {
            continue;
        }
        reordered.push(item.clone());
    }

    let threshold_met = all_domain_count >= target;
    DiversificationOutcome {
        reordered,
        distinct_domain_count: all_domain_count,
        threshold_met,
        limitation_flag: !threshold_met,
    }
}

pub fn count_distinct_domains<T, F>(items: &[T], domain_of: F) -> usize
where
    F: Fn(&T) -> String,
{
    let mut domains = BTreeSet::new();
    for item in items {
        let domain = domain_of(item);
        if !domain.trim().is_empty() {
            domains.insert(domain);
        }
    }
    domains.len()
}
