#![forbid(unsafe_code)]

use crate::web_search_plan::perf_cost::tiers::ImportanceTier;
use crate::web_search_plan::reason_code_validator::{
    validate_reason_code_registry, validate_reason_codes_registered,
};
use crate::web_search_plan::registry_loader::{load_reason_code_registry, ReasonCodeRegistry};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const FAILURE_SIGNATURE_SCHEMA_VERSION: &str = "1.0.0";
pub const FAILURE_SIGNATURE_ID_VERSION: &str = "1.0.0";
pub const FAILURE_LEDGER_IDEMPOTENCY_VERSION: &str = "1.0.0";
pub const FAILURE_LEDGER_WRITE_PATH: &str = "policy_snapshot_persist";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LearningLane {
    Web,
    News,
    Images,
    Video,
    UrlFetch,
    Synthesis,
    Write,
}

impl LearningLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Web => "web",
            Self::News => "news",
            Self::Images => "images",
            Self::Video => "video",
            Self::UrlFetch => "url_fetch",
            Self::Synthesis => "synthesis",
            Self::Write => "write",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureEvent {
    pub lane: LearningLane,
    pub provider_id: Option<String>,
    pub error_kind: String,
    pub reason_code_id: String,
    pub importance_tier: ImportanceTier,
    pub canonical_url: Option<String>,
    pub occurred_at_ms: i64,
    pub ttl_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureSignature {
    pub signature_id: String,
    pub lane: String,
    pub provider_id: Option<String>,
    pub error_kind: String,
    pub reason_code_id: String,
    pub importance_tier: String,
    pub canonical_url: Option<String>,
    pub created_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub count: u64,
    pub ttl_ms: i64,
    pub schema_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureLedgerEntry {
    pub idempotency_key: String,
    pub policy_snapshot_id: String,
    pub write_path_name: String,
    pub signature: FailureSignature,
}

#[derive(Debug, Clone)]
pub struct FailureLedger {
    entries: Vec<FailureLedgerEntry>,
    idempotency_index: BTreeMap<String, usize>,
    latest_by_signature: BTreeMap<String, FailureSignature>,
    reason_registry: ReasonCodeRegistry,
}

impl FailureLedger {
    pub fn new(reason_registry: ReasonCodeRegistry) -> Result<Self, String> {
        validate_reason_code_registry(&reason_registry)?;
        Ok(Self {
            entries: Vec::new(),
            idempotency_index: BTreeMap::new(),
            latest_by_signature: BTreeMap::new(),
            reason_registry,
        })
    }

    pub fn new_from_registry_file() -> Result<Self, String> {
        let reason_registry = load_reason_code_registry()?;
        Self::new(reason_registry)
    }

    pub fn entries(&self) -> &[FailureLedgerEntry] {
        &self.entries
    }

    pub fn latest_signature(&self, signature_id: &str) -> Option<&FailureSignature> {
        self.latest_by_signature.get(signature_id)
    }

    pub fn record_failure(
        &mut self,
        event: &FailureEvent,
        policy_snapshot_id: &str,
    ) -> Result<FailureSignature, String> {
        let normalized_reason_code = normalize_reason_code_id(event.reason_code_id.as_str());
        validate_reason_codes_registered(&[normalized_reason_code.clone()], &self.reason_registry)?;

        let idempotency_key = failure_event_idempotency_key(event, policy_snapshot_id);
        if let Some(index) = self.idempotency_index.get(idempotency_key.as_str()) {
            return self
                .entries
                .get(*index)
                .map(|entry| entry.signature.clone())
                .ok_or_else(|| "idempotency index points to missing entry".to_string());
        }

        let signature_id = compute_signature_id(event);
        let previous = self.latest_by_signature.get(signature_id.as_str());
        let seen_at_ms = previous
            .map(|sig| sig.last_seen_at_ms.max(event.occurred_at_ms))
            .unwrap_or(event.occurred_at_ms);
        let ttl_ms = event.ttl_ms.max(0);

        let next_signature = if let Some(current) = previous {
            if is_signature_expired(current, event.occurred_at_ms) {
                FailureSignature {
                    signature_id: signature_id.clone(),
                    lane: event.lane.as_str().to_string(),
                    provider_id: normalize_provider_id(event.provider_id.as_deref()),
                    error_kind: normalize_error_kind(event.error_kind.as_str()),
                    reason_code_id: normalized_reason_code,
                    importance_tier: event.importance_tier.as_str().to_string(),
                    canonical_url: normalize_canonical_url(event.canonical_url.as_deref()),
                    created_at_ms: event.occurred_at_ms,
                    last_seen_at_ms: event.occurred_at_ms,
                    count: 1,
                    ttl_ms,
                    schema_version: FAILURE_SIGNATURE_SCHEMA_VERSION.to_string(),
                }
            } else {
                FailureSignature {
                    signature_id: signature_id.clone(),
                    lane: event.lane.as_str().to_string(),
                    provider_id: normalize_provider_id(event.provider_id.as_deref()),
                    error_kind: normalize_error_kind(event.error_kind.as_str()),
                    reason_code_id: normalized_reason_code,
                    importance_tier: event.importance_tier.as_str().to_string(),
                    canonical_url: normalize_canonical_url(event.canonical_url.as_deref()),
                    created_at_ms: current.created_at_ms,
                    last_seen_at_ms: seen_at_ms,
                    count: current.count.saturating_add(1),
                    ttl_ms,
                    schema_version: FAILURE_SIGNATURE_SCHEMA_VERSION.to_string(),
                }
            }
        } else {
            FailureSignature {
                signature_id: signature_id.clone(),
                lane: event.lane.as_str().to_string(),
                provider_id: normalize_provider_id(event.provider_id.as_deref()),
                error_kind: normalize_error_kind(event.error_kind.as_str()),
                reason_code_id: normalized_reason_code,
                importance_tier: event.importance_tier.as_str().to_string(),
                canonical_url: normalize_canonical_url(event.canonical_url.as_deref()),
                created_at_ms: event.occurred_at_ms,
                last_seen_at_ms: event.occurred_at_ms,
                count: 1,
                ttl_ms,
                schema_version: FAILURE_SIGNATURE_SCHEMA_VERSION.to_string(),
            }
        };

        let entry = FailureLedgerEntry {
            idempotency_key: idempotency_key.clone(),
            policy_snapshot_id: normalize_policy_snapshot_id(policy_snapshot_id),
            write_path_name: FAILURE_LEDGER_WRITE_PATH.to_string(),
            signature: next_signature.clone(),
        };

        self.entries.push(entry);
        let index = self.entries.len().saturating_sub(1);
        self.idempotency_index.insert(idempotency_key, index);
        self.latest_by_signature
            .insert(signature_id, next_signature.clone());

        Ok(next_signature)
    }
}

pub fn compute_signature_id(event: &FailureEvent) -> String {
    let material = format!(
        "version={}|lane={}|provider_id={}|error_kind={}|reason_code_id={}|importance_tier={}|canonical_url={}",
        FAILURE_SIGNATURE_ID_VERSION,
        event.lane.as_str(),
        normalize_provider_id(event.provider_id.as_deref()).unwrap_or_else(|| "none".to_string()),
        normalize_error_kind(event.error_kind.as_str()),
        normalize_reason_code_id(event.reason_code_id.as_str()),
        event.importance_tier.as_str(),
        normalize_canonical_url(event.canonical_url.as_deref())
            .unwrap_or_else(|| "none".to_string())
    );
    sha256_hex(material.as_bytes())
}

pub fn failure_event_idempotency_key(event: &FailureEvent, policy_snapshot_id: &str) -> String {
    let material = format!(
        "write_path={}|version={}|signature_id={}|occurred_at_ms={}|policy_snapshot_id={}",
        FAILURE_LEDGER_WRITE_PATH,
        FAILURE_LEDGER_IDEMPOTENCY_VERSION,
        compute_signature_id(event),
        event.occurred_at_ms,
        normalize_policy_snapshot_id(policy_snapshot_id),
    );
    sha256_hex(material.as_bytes())
}

pub fn is_signature_expired(signature: &FailureSignature, now_ms: i64) -> bool {
    let expiry = signature
        .last_seen_at_ms
        .saturating_add(signature.ttl_ms.max(0));
    now_ms > expiry
}

fn normalize_provider_id(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn normalize_error_kind(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn normalize_reason_code_id(raw: &str) -> String {
    raw.trim().to_ascii_lowercase()
}

fn normalize_policy_snapshot_id(raw: &str) -> String {
    let normalized = raw.trim();
    if normalized.is_empty() {
        "none".to_string()
    } else {
        normalized.to_string()
    }
}

fn normalize_canonical_url(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
}

fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}
