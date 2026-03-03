#![forbid(unsafe_code)]

pub mod failure_signature;
pub mod promotion_gate;
pub mod proposal;
pub mod rollback;
pub mod session_adaptation;

use serde_json::{json, Map, Value};

pub const LEARNING_ENGINE_ID: &str = "PH1.LEARN";
pub const BUILDER_GOV_ENGINE_ID: &str = "PH1.GOV";
pub const LEARNING_AUDIT_VERSION: &str = "1.0.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryTarget {
    Ph1EExecution,
    EvidencePacket,
    SynthesisPacket,
    WritePacket,
    ScoringWeights,
    ProviderLadderCore,
    PolicyGateBypass,
}

impl BoundaryTarget {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ph1EExecution => "ph1e_execution",
            Self::EvidencePacket => "evidence_packet",
            Self::SynthesisPacket => "synthesis_packet",
            Self::WritePacket => "write_packet",
            Self::ScoringWeights => "scoring_weights",
            Self::ProviderLadderCore => "provider_ladder_core",
            Self::PolicyGateBypass => "policy_gate_bypass",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearningBoundaryError {
    BoundaryViolation(&'static str),
    PacketMutationBlocked(String),
}

pub fn enforce_learning_boundary(target: BoundaryTarget) -> Result<(), LearningBoundaryError> {
    Err(LearningBoundaryError::BoundaryViolation(target.as_str()))
}

pub fn assert_no_packet_mutation(
    original: &Value,
    candidate: &Value,
    packet_name: &str,
) -> Result<(), LearningBoundaryError> {
    if original == candidate {
        Ok(())
    } else {
        Err(LearningBoundaryError::PacketMutationBlocked(
            packet_name.to_string(),
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningAuditMetrics {
    pub failure_signature_id: String,
    pub adaptation_applied: bool,
    pub proposal_generated: bool,
    pub proposal_id: Option<String>,
    pub promotion_status: String,
    pub rollback_status: String,
    pub policy_snapshot_version: String,
}

impl LearningAuditMetrics {
    pub fn new(
        failure_signature_id: impl Into<String>,
        adaptation_applied: bool,
        proposal_generated: bool,
        proposal_id: Option<String>,
        promotion_status: impl Into<String>,
        rollback_status: impl Into<String>,
        policy_snapshot_version: impl Into<String>,
    ) -> Self {
        Self {
            failure_signature_id: failure_signature_id.into(),
            adaptation_applied,
            proposal_generated,
            proposal_id,
            promotion_status: promotion_status.into(),
            rollback_status: rollback_status.into(),
            policy_snapshot_version: policy_snapshot_version.into(),
        }
    }
}

pub fn append_learning_audit_fields(
    audit_packet: &mut Value,
    metrics: &LearningAuditMetrics,
) -> Result<(), String> {
    let obj = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_value = obj
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    let transition_obj = if transition_value.is_object() {
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(state) = transition_value.as_str() {
        *transition_value = json!({"state": state});
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "learning_audit".to_string(),
        json!({
            "version": LEARNING_AUDIT_VERSION,
            "failure_signature_id": metrics.failure_signature_id,
            "adaptation_applied": metrics.adaptation_applied,
            "proposal_generated": metrics.proposal_generated,
            "proposal_id": metrics.proposal_id,
            "promotion_status": metrics.promotion_status,
            "rollback_status": metrics.rollback_status,
            "policy_snapshot_version": metrics.policy_snapshot_version,
        }),
    );

    Ok(())
}

#[cfg(test)]
pub mod learning_tests;
