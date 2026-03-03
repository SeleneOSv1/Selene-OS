#![forbid(unsafe_code)]

use crate::web_search_plan::web_provider::NormalizedSearchResult;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct MergeOutput {
    pub merged_results: Vec<NormalizedSearchResult>,
    pub dedup_count: usize,
}

pub fn merge_results(
    brave_results: &[NormalizedSearchResult],
    openai_results: &[NormalizedSearchResult],
) -> MergeOutput {
    let mut merged = Vec::new();
    let mut seen = BTreeSet::new();
    let mut dedup_count = 0usize;

    for result in brave_results {
        if seen.insert(result.canonical_url.clone()) {
            merged.push(result.clone());
        } else {
            dedup_count = dedup_count.saturating_add(1);
        }
    }

    for result in openai_results {
        if seen.insert(result.canonical_url.clone()) {
            merged.push(result.clone());
        } else {
            dedup_count = dedup_count.saturating_add(1);
        }
    }

    MergeOutput {
        merged_results: merged,
        dedup_count,
    }
}
