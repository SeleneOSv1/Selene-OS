#![forbid(unsafe_code)]

use crate::web_search_plan::learning::proposal::{ProposalArtifact, ProposalStatus};
use crate::web_search_plan::learning::BUILDER_GOV_ENGINE_ID;
use sha2::{Digest, Sha256};

pub const PROMOTION_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshot {
    pub policy_snapshot_version: String,
    pub fallback_priority: Vec<String>,
    pub retry_attempts: u8,
    pub cooldown_failures_before: u8,
    pub open_budget_per_query: u8,
}

impl PolicySnapshot {
    pub fn fingerprint(&self) -> String {
        let material = format!(
            "version={}|fallback={}|retry={}|cooldown={}|open_budget={}",
            self.policy_snapshot_version,
            self.fallback_priority.join(","),
            self.retry_attempts,
            self.cooldown_failures_before,
            self.open_budget_per_query,
        );
        let mut hasher = Sha256::new();
        hasher.update(material.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRecord {
    pub proposal_id: String,
    pub approved_by: String,
    pub prior_policy_snapshot_version: String,
    pub new_policy_snapshot_version: String,
    pub promoted_at_ms: i64,
    pub replay_validation_passed: bool,
    pub promotion_status: String,
    pub promotion_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionError {
    ExplicitApprovalRequired,
    InvalidApprover,
    ProposalNotProposed,
    ReplayValidationRequired,
    PolicySnapshotVersionUnchanged,
}

pub fn approve_proposal(
    proposal: &ProposalArtifact,
    approver_engine_id: &str,
    replay_validation_passed: bool,
    current_policy: &PolicySnapshot,
    promoted_policy: &PolicySnapshot,
    promoted_at_ms: i64,
) -> Result<(ProposalArtifact, PromotionRecord), PromotionError> {
    if approver_engine_id.trim().is_empty() {
        return Err(PromotionError::ExplicitApprovalRequired);
    }
    if approver_engine_id != BUILDER_GOV_ENGINE_ID {
        return Err(PromotionError::InvalidApprover);
    }
    if proposal.status != ProposalStatus::Proposed {
        return Err(PromotionError::ProposalNotProposed);
    }
    if !replay_validation_passed {
        return Err(PromotionError::ReplayValidationRequired);
    }
    if current_policy.policy_snapshot_version == promoted_policy.policy_snapshot_version {
        return Err(PromotionError::PolicySnapshotVersionUnchanged);
    }

    let mut approved = proposal.clone();
    approved.status = ProposalStatus::Approved;

    let record = PromotionRecord {
        proposal_id: proposal.proposal_id.clone(),
        approved_by: approver_engine_id.to_string(),
        prior_policy_snapshot_version: current_policy.policy_snapshot_version.clone(),
        new_policy_snapshot_version: promoted_policy.policy_snapshot_version.clone(),
        promoted_at_ms,
        replay_validation_passed,
        promotion_status: ProposalStatus::Approved.as_str().to_string(),
        promotion_version: PROMOTION_VERSION.to_string(),
    };

    Ok((approved, record))
}
