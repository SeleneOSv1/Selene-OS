#![forbid(unsafe_code)]

use crate::web_search_plan::news_provider::NormalizedNewsResult;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct MergeOutput {
    pub merged_results: Vec<NormalizedNewsResult>,
    pub dedup_count: usize,
    pub corroborated_canonical_urls: BTreeSet<String>,
}

pub fn merge_news_results(
    brave_results: &[NormalizedNewsResult],
    gdelt_results: &[NormalizedNewsResult],
) -> MergeOutput {
    let mut merged_results = Vec::new();
    let mut seen = BTreeSet::new();
    let mut dedup_count = 0usize;
    let mut corroborated_canonical_urls = BTreeSet::new();

    for result in brave_results {
        if seen.insert(result.canonical_url.clone()) {
            merged_results.push(result.clone());
        } else {
            dedup_count = dedup_count.saturating_add(1);
        }
    }

    for result in gdelt_results {
        if seen.insert(result.canonical_url.clone()) {
            merged_results.push(result.clone());
        } else {
            dedup_count = dedup_count.saturating_add(1);
            corroborated_canonical_urls.insert(result.canonical_url.clone());
        }
    }

    MergeOutput {
        merged_results,
        dedup_count,
        corroborated_canonical_urls,
    }
}
