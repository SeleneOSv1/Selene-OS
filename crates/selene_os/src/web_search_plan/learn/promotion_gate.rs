#![forbid(unsafe_code)]

use crate::web_search_plan::learn::proposal_artifact::{ProposalArtifact, ProposalStatus};
use crate::web_search_plan::learn::GOVERNANCE_ENGINE_ID;

pub const PROMOTION_RECORD_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicySnapshotReference {
    pub policy_snapshot_version: String,
    pub fallback_priority: Vec<String>,
    pub timeout_per_provider_ms: u64,
    pub open_budget_per_query: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionApprovalInput {
    pub approver_engine_id: String,
    pub proposal_id: String,
    pub approved_at_ms: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionRecord {
    pub proposal_id: String,
    pub approved_by: String,
    pub prior_policy_snapshot_version: String,
    pub new_policy_snapshot_version: String,
    pub approved_at_ms: i64,
    pub promotion_status: String,
    pub activation_state: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromotionError {
    ExplicitApprovalRequired,
    InvalidApprover,
    ProposalIdMismatch,
    ProposalNotProposed,
    PolicySnapshotVersionUnchanged,
}

pub fn approve_proposal(
    proposal: &ProposalArtifact,
    approval: &PromotionApprovalInput,
    current_policy: &PolicySnapshotReference,
    promoted_policy: &PolicySnapshotReference,
) -> Result<(ProposalArtifact, PromotionRecord), PromotionError> {
    if approval.approver_engine_id.trim().is_empty() || approval.proposal_id.trim().is_empty() {
        return Err(PromotionError::ExplicitApprovalRequired);
    }
    if approval.approver_engine_id != GOVERNANCE_ENGINE_ID {
        return Err(PromotionError::InvalidApprover);
    }
    if approval.proposal_id != proposal.proposal_id {
        return Err(PromotionError::ProposalIdMismatch);
    }
    if proposal.status != ProposalStatus::Proposed {
        return Err(PromotionError::ProposalNotProposed);
    }
    if current_policy.policy_snapshot_version == promoted_policy.policy_snapshot_version {
        return Err(PromotionError::PolicySnapshotVersionUnchanged);
    }

    let mut approved = proposal.clone();
    approved.status = ProposalStatus::Approved;

    let record = PromotionRecord {
        proposal_id: proposal.proposal_id.clone(),
        approved_by: approval.approver_engine_id.clone(),
        prior_policy_snapshot_version: current_policy.policy_snapshot_version.clone(),
        new_policy_snapshot_version: promoted_policy.policy_snapshot_version.clone(),
        approved_at_ms: approval.approved_at_ms,
        promotion_status: ProposalStatus::Approved.as_str().to_string(),
        activation_state: "activation_pending".to_string(),
        version: PROMOTION_RECORD_VERSION.to_string(),
    };

    Ok((approved, record))
}
