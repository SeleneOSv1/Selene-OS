#![forbid(unsafe_code)]

use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const FAILURE_SIGNATURE_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SearchMode {
    Web,
    News,
    Image,
    Video,
    UrlFetch,
}

impl SearchMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::News => "news",
            Self::Image => "image",
            Self::Video => "video",
            Self::UrlFetch => "url_fetch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureEventInput {
    pub provider_id: String,
    pub error_kind: String,
    pub mode: SearchMode,
    pub importance_tier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureSignature {
    pub signature_id: String,
    pub provider_id: String,
    pub error_kind: String,
    pub mode: String,
    pub importance_tier: String,
    pub timestamp_ms: i64,
    pub occurrence_count: u64,
    pub ttl_ms: i64,
    pub version: String,
}

pub fn compute_signature_id(input: &FailureEventInput) -> String {
    let material = format!(
        "v={}|provider_id={}|error_kind={}|mode={}|importance_tier={}",
        FAILURE_SIGNATURE_VERSION,
        normalize_provider_id(&input.provider_id),
        normalize_error_kind(&input.error_kind),
        input.mode.as_str(),
        normalize_importance_tier(&input.importance_tier)
    );

    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    format!("{:x}", hasher.finalize())
}

impl FailureSignature {
    pub fn from_input(
        input: &FailureEventInput,
        timestamp_ms: i64,
        occurrence_count: u64,
        ttl_ms: i64,
    ) -> Self {
        Self {
            signature_id: compute_signature_id(input),
            provider_id: normalize_provider_id(&input.provider_id),
            error_kind: normalize_error_kind(&input.error_kind),
            mode: input.mode.as_str().to_string(),
            importance_tier: normalize_importance_tier(&input.importance_tier),
            timestamp_ms,
            occurrence_count,
            ttl_ms: ttl_ms.max(0),
            version: FAILURE_SIGNATURE_VERSION.to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct FailureLedger {
    events: Vec<FailureSignature>,
    occurrence_counts: BTreeMap<String, u64>,
    latest_by_signature: BTreeMap<String, usize>,
}

impl FailureLedger {
    pub fn record_failure(
        &mut self,
        input: &FailureEventInput,
        timestamp_ms: i64,
        ttl_ms: i64,
    ) -> FailureSignature {
        let signature_id = compute_signature_id(input);
        let next_count = self
            .occurrence_counts
            .get(signature_id.as_str())
            .copied()
            .unwrap_or(0)
            .saturating_add(1);

        let signature = FailureSignature::from_input(input, timestamp_ms, next_count, ttl_ms);
        self.events.push(signature.clone());
        self.occurrence_counts
            .insert(signature_id.clone(), next_count);
        self.latest_by_signature
            .insert(signature_id, self.events.len().saturating_sub(1));

        signature
    }

    pub fn events(&self) -> &[FailureSignature] {
        &self.events
    }

    pub fn latest_signature(&self, signature_id: &str) -> Option<&FailureSignature> {
        self.latest_by_signature
            .get(signature_id)
            .and_then(|idx| self.events.get(*idx))
    }

    pub fn occurrence_count(&self, signature_id: &str) -> u64 {
        self.occurrence_counts
            .get(signature_id)
            .copied()
            .unwrap_or(0)
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

fn normalize_provider_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn normalize_error_kind(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

pub fn normalize_importance_tier(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "low" => "low".to_string(),
        "high" => "high".to_string(),
        _ => "medium".to_string(),
    }
}
