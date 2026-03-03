#![forbid(unsafe_code)]

use crate::web_search_plan::learn::failure_signature::FailureSignature;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeSet;

pub const PROPOSAL_ARTIFACT_VERSION: &str = "1.0.0";
pub const PROPOSAL_ID_VERSION: &str = "1.0.0";
pub const PROPOSAL_WRITE_PATH: &str = "policy_snapshot_persist";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposedChangeType {
    RoutingHint,
    TimeoutTune,
    BudgetTune,
    CooldownTune,
    TemplateTune,
}

impl ProposedChangeType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RoutingHint => "routing_hint",
            Self::TimeoutTune => "timeout_tune",
            Self::BudgetTune => "budget_tune",
            Self::CooldownTune => "cooldown_tune",
            Self::TemplateTune => "template_tune",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalStatus {
    Proposed,
    Approved,
    Rejected,
    Expired,
}

impl ProposalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalEvidence {
    pub occurrence_count: u64,
    pub first_seen_at_ms: i64,
    pub last_seen_at_ms: i64,
    pub reason_code_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalArtifact {
    pub proposal_id: String,
    pub signature_id: String,
    pub proposed_change_type: ProposedChangeType,
    pub proposed_change_payload: Value,
    pub evidence: ProposalEvidence,
    pub created_at_ms: i64,
    pub status: ProposalStatus,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProposalLedgerEntry {
    pub idempotency_key: String,
    pub policy_snapshot_id: String,
    pub write_path_name: String,
    pub artifact: ProposalArtifact,
}

#[derive(Debug, Clone, Default)]
pub struct ProposalLedger {
    entries: Vec<ProposalLedgerEntry>,
    idempotency_keys: BTreeSet<String>,
}

impl ProposalLedger {
    pub fn append_idempotent(
        &mut self,
        artifact: ProposalArtifact,
        policy_snapshot_id: &str,
    ) -> bool {
        let idempotency_key =
            proposal_idempotency_key(artifact.proposal_id.as_str(), policy_snapshot_id);
        if self.idempotency_keys.contains(idempotency_key.as_str()) {
            return false;
        }
        self.idempotency_keys.insert(idempotency_key.clone());
        self.entries.push(ProposalLedgerEntry {
            idempotency_key,
            policy_snapshot_id: normalize_policy_snapshot_id(policy_snapshot_id),
            write_path_name: PROPOSAL_WRITE_PATH.to_string(),
            artifact,
        });
        true
    }

    pub fn entries(&self) -> &[ProposalLedgerEntry] {
        &self.entries
    }
}

pub fn generate_proposal_if_threshold(
    signature: &FailureSignature,
    threshold: u64,
    proposed_change_type: ProposedChangeType,
    proposed_change_payload: &Value,
    created_at_ms: i64,
) -> Option<ProposalArtifact> {
    if threshold == 0 || signature.count < threshold {
        return None;
    }

    let canonical_payload = canonicalize_json(proposed_change_payload);
    let proposal_id = proposal_id(
        signature.signature_id.as_str(),
        proposed_change_type,
        &canonical_payload,
        signature.count,
    );

    Some(ProposalArtifact {
        proposal_id,
        signature_id: signature.signature_id.clone(),
        proposed_change_type,
        proposed_change_payload: canonical_payload,
        evidence: ProposalEvidence {
            occurrence_count: signature.count,
            first_seen_at_ms: signature.created_at_ms,
            last_seen_at_ms: signature.last_seen_at_ms,
            reason_code_id: signature.reason_code_id.clone(),
        },
        created_at_ms,
        status: ProposalStatus::Proposed,
        version: PROPOSAL_ARTIFACT_VERSION.to_string(),
    })
}

pub fn proposal_id(
    signature_id: &str,
    proposed_change_type: ProposedChangeType,
    proposed_change_payload: &Value,
    occurrence_count: u64,
) -> String {
    let payload =
        canonical_json_string(proposed_change_payload).unwrap_or_else(|_| "{}".to_string());
    let material = format!(
        "version={}|signature_id={}|proposed_change_type={}|payload={}|occurrence_count={}",
        PROPOSAL_ID_VERSION,
        signature_id.trim(),
        proposed_change_type.as_str(),
        payload,
        occurrence_count,
    );
    sha256_hex(material.as_bytes())
}

pub fn proposal_idempotency_key(proposal_id: &str, policy_snapshot_id: &str) -> String {
    let material = format!(
        "write_path={}|version={}|proposal_id={}|policy_snapshot_id={}",
        PROPOSAL_WRITE_PATH,
        PROPOSAL_ID_VERSION,
        proposal_id.trim(),
        normalize_policy_snapshot_id(policy_snapshot_id),
    );
    sha256_hex(material.as_bytes())
}

pub fn canonicalize_json(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = map.iter().collect::<Vec<(&String, &Value)>>();
            sorted.sort_by(|left, right| left.0.cmp(right.0));

            let mut object = Map::new();
            for (key, item) in sorted {
                object.insert(key.clone(), canonicalize_json(item));
            }
            Value::Object(object)
        }
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_json).collect()),
        _ => value.clone(),
    }
}

pub fn canonical_json_string(value: &Value) -> Result<String, String> {
    serde_json::to_string(&canonicalize_json(value))
        .map_err(|error| format!("failed to serialize canonical json: {}", error))
}

fn normalize_policy_snapshot_id(raw: &str) -> String {
    let normalized = raw.trim();
    if normalized.is_empty() {
        "none".to_string()
    } else {
        normalized.to_string()
    }
}

fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    format!("{:x}", hasher.finalize())
}
