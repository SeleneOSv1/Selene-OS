#![forbid(unsafe_code)]

use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceSufficiencyPolicy {
    pub min_distinct_sources: usize,
    pub min_chunk_support: usize,
}

impl Default for EvidenceSufficiencyPolicy {
    fn default() -> Self {
        Self {
            min_distinct_sources: 2,
            min_chunk_support: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EvidenceSufficiency {
    pub distinct_sources: usize,
    pub chunk_support: usize,
    pub is_sufficient: bool,
}

pub fn assess_evidence_sufficiency(
    evidence_packet: &Value,
    policy: EvidenceSufficiencyPolicy,
) -> EvidenceSufficiency {
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut source_urls: BTreeSet<String> = BTreeSet::new();
    for source in sources {
        if let Some(url) = source.get("url").and_then(Value::as_str) {
            let url = url.trim();
            if !url.is_empty() {
                source_urls.insert(url.to_string());
            }
        }
    }

    for chunk in &chunks {
        if let Some(url) = chunk.get("source_url").and_then(Value::as_str) {
            let url = url.trim();
            if !url.is_empty() {
                source_urls.insert(url.to_string());
            }
        }
    }

    let distinct_sources = source_urls.len();
    let chunk_support = chunks.len();

    let is_sufficient = distinct_sources >= policy.min_distinct_sources
        && chunk_support >= policy.min_chunk_support;

    EvidenceSufficiency {
        distinct_sources,
        chunk_support,
        is_sufficient,
    }
}
