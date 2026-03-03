#![forbid(unsafe_code)]

use crate::web_search_plan::learning::failure_signature::FailureSignature;
use sha2::{Digest, Sha256};

pub const PROPOSAL_VERSION: &str = "1.0.0";

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
pub struct ProposalArtifact {
    pub proposal_id: String,
    pub related_signature_id: String,
    pub suggested_change: String,
    pub impact_estimate: String,
    pub reproducibility_count: u64,
    pub created_at: i64,
    pub proposal_version: String,
    pub status: ProposalStatus,
}

pub fn proposal_id(
    related_signature_id: &str,
    suggested_change: &str,
    reproducibility_count: u64,
) -> String {
    let material = format!(
        "v={}|related_signature_id={}|suggested_change={}|reproducibility_count={}",
        PROPOSAL_VERSION,
        related_signature_id.trim(),
        suggested_change.trim(),
        reproducibility_count,
    );

    let mut hasher = Sha256::new();
    hasher.update(material.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn generate_proposal_if_threshold(
    signature: &FailureSignature,
    threshold: u64,
    suggested_change: &str,
    impact_estimate: &str,
    created_at: i64,
) -> Option<ProposalArtifact> {
    if threshold == 0 || signature.occurrence_count < threshold {
        return None;
    }

    let normalized_change = suggested_change.trim();
    let normalized_impact = impact_estimate.trim();
    if normalized_change.is_empty() || normalized_impact.is_empty() {
        return None;
    }

    Some(ProposalArtifact {
        proposal_id: proposal_id(
            &signature.signature_id,
            normalized_change,
            signature.occurrence_count,
        ),
        related_signature_id: signature.signature_id.clone(),
        suggested_change: normalized_change.to_string(),
        impact_estimate: normalized_impact.to_string(),
        reproducibility_count: signature.occurrence_count,
        created_at,
        proposal_version: PROPOSAL_VERSION.to_string(),
        status: ProposalStatus::Proposed,
    })
}

#[derive(Debug, Clone, Default)]
pub struct LearningProposalLedger {
    entries: Vec<ProposalArtifact>,
}

impl LearningProposalLedger {
    pub fn append(&mut self, artifact: ProposalArtifact) {
        self.entries.push(artifact);
    }

    pub fn entries(&self) -> &[ProposalArtifact] {
        &self.entries
    }

    pub fn latest_for_signature(&self, signature_id: &str) -> Option<&ProposalArtifact> {
        self.entries
            .iter()
            .rev()
            .find(|entry| entry.related_signature_id == signature_id)
    }
}
