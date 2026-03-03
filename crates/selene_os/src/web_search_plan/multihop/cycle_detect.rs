#![forbid(unsafe_code)]

use crate::web_search_plan::contract_hash::sha256_hex;
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CycleDetectionError {
    pub reason_code: &'static str,
    pub message: String,
    pub cycle_detected: bool,
}

impl CycleDetectionError {
    fn duplicate_sub_query(sub_query: &str) -> Self {
        Self {
            reason_code: "policy_violation",
            message: format!("cycle detected for sub_query {}", sub_query),
            cycle_detected: true,
        }
    }

    fn duplicate_url(canonical_url: &str) -> Self {
        Self {
            reason_code: "policy_violation",
            message: format!("cycle detected for canonical_url {}", canonical_url),
            cycle_detected: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CycleDetector {
    visited_sub_query_hashes: BTreeSet<String>,
    visited_url_hashes: BTreeSet<String>,
}

impl CycleDetector {
    pub fn register_sub_query(&mut self, sub_query: &str) -> Result<(), CycleDetectionError> {
        let canonical = canonicalize_sub_query(sub_query);
        let key = sha256_hex(canonical.as_bytes());
        if !self.visited_sub_query_hashes.insert(key) {
            return Err(CycleDetectionError::duplicate_sub_query(canonical.as_str()));
        }
        Ok(())
    }

    pub fn register_canonical_url(&mut self, canonical_url: &str) -> Result<(), CycleDetectionError> {
        let normalized = canonical_url.trim().to_ascii_lowercase();
        if normalized.is_empty() {
            return Ok(());
        }
        let key = sha256_hex(normalized.as_bytes());
        if !self.visited_url_hashes.insert(key) {
            return Err(CycleDetectionError::duplicate_url(normalized.as_str()));
        }
        Ok(())
    }

    pub fn visited_sub_queries(&self) -> usize {
        self.visited_sub_query_hashes.len()
    }

    pub fn visited_urls(&self) -> usize {
        self.visited_url_hashes.len()
    }
}

fn canonicalize_sub_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<&str>>()
        .join(" ")
        .to_ascii_lowercase()
}
