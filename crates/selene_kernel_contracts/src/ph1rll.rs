#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1RLL_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RllCapabilityId {
    RllPolicyRankOffline,
    RllArtifactRecommend,
}

impl RllCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            RllCapabilityId::RllPolicyRankOffline => "RLL_POLICY_RANK_OFFLINE",
            RllCapabilityId::RllArtifactRecommend => "RLL_ARTIFACT_RECOMMEND",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RllOptimizationTarget {
    PaeProviderSelectionWeights,
    PruneClarificationOrdering,
    CachePrefetchHeuristics,
    ContextRetrievalScoring,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RllValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_candidates: u8,
    pub max_recommendations: u8,
    pub offline_pipeline_only: bool,
}

impl RllRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_candidates: u8,
        max_recommendations: u8,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_candidates,
            max_recommendations,
            offline_pipeline_only,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for RllRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_candidates == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.max_candidates",
                reason: "must be > 0",
            });
        }
        if self.max_candidates > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.max_candidates",
                reason: "must be <= 64",
            });
        }
        if self.max_recommendations == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.max_recommendations",
                reason: "must be > 0",
            });
        }
        if self.max_recommendations > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.max_recommendations",
                reason: "must be <= 32",
            });
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "rll_request_envelope.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllArtifactCandidate {
    pub schema_version: SchemaVersion,
    pub artifact_id: String,
    pub target: RllOptimizationTarget,
    pub expected_effect_bp: i16,
    pub confidence_pct: u8,
    pub approval_tier: u8,
    pub evidence_ref: String,
}

impl RllArtifactCandidate {
    pub fn v1(
        artifact_id: String,
        target: RllOptimizationTarget,
        expected_effect_bp: i16,
        confidence_pct: u8,
        approval_tier: u8,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let candidate = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            artifact_id,
            target,
            expected_effect_bp,
            confidence_pct,
            approval_tier,
            evidence_ref,
        };
        candidate.validate()?;
        Ok(candidate)
    }
}

impl Validate for RllArtifactCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_candidate.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        validate_token("rll_artifact_candidate.artifact_id", &self.artifact_id, 96)?;
        if self.expected_effect_bp.abs() > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_candidate.expected_effect_bp",
                reason: "must be within -10000..=10000 basis points",
            });
        }
        if self.approval_tier > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_candidate.approval_tier",
                reason: "must be within 0..=3",
            });
        }
        validate_token(
            "rll_artifact_candidate.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllPolicyRankOfflineRequest {
    pub schema_version: SchemaVersion,
    pub envelope: RllRequestEnvelope,
    pub candidates: Vec<RllArtifactCandidate>,
    pub training_window_days: u16,
    pub minimum_sample_size: u32,
}

impl RllPolicyRankOfflineRequest {
    pub fn v1(
        envelope: RllRequestEnvelope,
        candidates: Vec<RllArtifactCandidate>,
        training_window_days: u16,
        minimum_sample_size: u32,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            envelope,
            candidates,
            training_window_days,
            minimum_sample_size,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for RllPolicyRankOfflineRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_request.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_request.candidates",
                reason: "must not be empty",
            });
        }
        if self.candidates.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_request.candidates",
                reason: "must be <= 64",
            });
        }
        let mut artifact_ids = BTreeSet::new();
        for candidate in &self.candidates {
            candidate.validate()?;
            if !artifact_ids.insert(candidate.artifact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "rll_policy_rank_offline_request.candidates",
                    reason: "artifact_id must be unique",
                });
            }
        }
        if self.training_window_days == 0 || self.training_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_request.training_window_days",
                reason: "must be within 1..=365",
            });
        }
        if self.minimum_sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_request.minimum_sample_size",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllRecommendationItem {
    pub schema_version: SchemaVersion,
    pub artifact_id: String,
    pub target: RllOptimizationTarget,
    pub rank: u8,
    pub confidence_pct: u8,
    pub approval_tier: u8,
    pub evidence_ref: String,
}

impl RllRecommendationItem {
    pub fn v1(
        artifact_id: String,
        target: RllOptimizationTarget,
        rank: u8,
        confidence_pct: u8,
        approval_tier: u8,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let item = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            artifact_id,
            target,
            rank,
            confidence_pct,
            approval_tier,
            evidence_ref,
        };
        item.validate()?;
        Ok(item)
    }
}

impl Validate for RllRecommendationItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_recommendation_item.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        validate_token("rll_recommendation_item.artifact_id", &self.artifact_id, 96)?;
        if self.rank == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_recommendation_item.rank",
                reason: "must be > 0",
            });
        }
        if self.approval_tier > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_recommendation_item.approval_tier",
                reason: "must be within 0..=3",
            });
        }
        validate_token(
            "rll_recommendation_item.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllArtifactRecommendRequest {
    pub schema_version: SchemaVersion,
    pub envelope: RllRequestEnvelope,
    pub selected_artifact_id: String,
    pub ordered_recommendations: Vec<RllRecommendationItem>,
}

impl RllArtifactRecommendRequest {
    pub fn v1(
        envelope: RllRequestEnvelope,
        selected_artifact_id: String,
        ordered_recommendations: Vec<RllRecommendationItem>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            envelope,
            selected_artifact_id,
            ordered_recommendations,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for RllArtifactRecommendRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_request.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "rll_artifact_recommend_request.selected_artifact_id",
            &self.selected_artifact_id,
            96,
        )?;
        if self.ordered_recommendations.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_request.ordered_recommendations",
                reason: "must not be empty",
            });
        }
        if self.ordered_recommendations.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_request.ordered_recommendations",
                reason: "must be <= 32",
            });
        }
        let mut artifact_ids = BTreeSet::new();
        for item in &self.ordered_recommendations {
            item.validate()?;
            if !artifact_ids.insert(item.artifact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "rll_artifact_recommend_request.ordered_recommendations",
                    reason: "artifact_id must be unique",
                });
            }
        }
        if !self
            .ordered_recommendations
            .iter()
            .any(|item| item.artifact_id == self.selected_artifact_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_request.selected_artifact_id",
                reason: "must exist in ordered_recommendations",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1RllRequest {
    RllPolicyRankOffline(RllPolicyRankOfflineRequest),
    RllArtifactRecommend(RllArtifactRecommendRequest),
}

impl Validate for Ph1RllRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1RllRequest::RllPolicyRankOffline(req) => req.validate(),
            Ph1RllRequest::RllArtifactRecommend(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllPolicyRankOfflineOk {
    pub schema_version: SchemaVersion,
    pub capability_id: RllCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_artifact_id: String,
    pub ordered_recommendations: Vec<RllRecommendationItem>,
    pub offline_only: bool,
    pub approval_required: bool,
    pub no_execution_authority: bool,
}

impl RllPolicyRankOfflineOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_artifact_id: String,
        ordered_recommendations: Vec<RllRecommendationItem>,
        offline_only: bool,
        approval_required: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            capability_id: RllCapabilityId::RllPolicyRankOffline,
            reason_code,
            selected_artifact_id,
            ordered_recommendations,
            offline_only,
            approval_required,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for RllPolicyRankOfflineOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        if self.capability_id != RllCapabilityId::RllPolicyRankOffline {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.capability_id",
                reason: "must be RLL_POLICY_RANK_OFFLINE",
            });
        }
        validate_token(
            "rll_policy_rank_offline_ok.selected_artifact_id",
            &self.selected_artifact_id,
            96,
        )?;
        if self.ordered_recommendations.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.ordered_recommendations",
                reason: "must not be empty",
            });
        }
        if self.ordered_recommendations.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.ordered_recommendations",
                reason: "must be <= 32",
            });
        }
        let mut artifact_ids = BTreeSet::new();
        for item in &self.ordered_recommendations {
            item.validate()?;
            if !artifact_ids.insert(item.artifact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "rll_policy_rank_offline_ok.ordered_recommendations",
                    reason: "artifact_id must be unique",
                });
            }
        }
        if !self
            .ordered_recommendations
            .iter()
            .any(|item| item.artifact_id == self.selected_artifact_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.selected_artifact_id",
                reason: "must exist in ordered_recommendations",
            });
        }
        if !self.offline_only {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.offline_only",
                reason: "must be true",
            });
        }
        if !self.approval_required {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.approval_required",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "rll_policy_rank_offline_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllArtifactRecommendOk {
    pub schema_version: SchemaVersion,
    pub capability_id: RllCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: RllValidationStatus,
    pub diagnostics: Vec<String>,
    pub offline_only: bool,
    pub no_execution_authority: bool,
}

impl RllArtifactRecommendOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: RllValidationStatus,
        diagnostics: Vec<String>,
        offline_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            capability_id: RllCapabilityId::RllArtifactRecommend,
            reason_code,
            validation_status,
            diagnostics,
            offline_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for RllArtifactRecommendOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        if self.capability_id != RllCapabilityId::RllArtifactRecommend {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.capability_id",
                reason: "must be RLL_ARTIFACT_RECOMMEND",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("rll_artifact_recommend_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == RllValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.offline_only {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.offline_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "rll_artifact_recommend_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: RllCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl RllRefuse {
    pub fn v1(
        capability_id: RllCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1RLL_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for RllRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1RLL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "rll_refuse.schema_version",
                reason: "must match PH1RLL_CONTRACT_VERSION",
            });
        }
        validate_token("rll_refuse.message", &self.message, 256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1RllResponse {
    RllPolicyRankOfflineOk(RllPolicyRankOfflineOk),
    RllArtifactRecommendOk(RllArtifactRecommendOk),
    Refuse(RllRefuse),
}

impl Validate for Ph1RllResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1RllResponse::RllPolicyRankOfflineOk(out) => out.validate(),
            Ph1RllResponse::RllArtifactRecommendOk(out) => out.validate(),
            Ph1RllResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> RllRequestEnvelope {
        RllRequestEnvelope::v1(CorrelationId(2401), TurnId(211), 8, 4, true).unwrap()
    }

    fn candidate(id: &str, target: RllOptimizationTarget) -> RllArtifactCandidate {
        RllArtifactCandidate::v1(
            id.to_string(),
            target,
            130,
            82,
            3,
            "evidence:offline:1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn rll_policy_rank_request_is_schema_valid() {
        let req = RllPolicyRankOfflineRequest::v1(
            envelope(),
            vec![
                candidate(
                    "artifact_a",
                    RllOptimizationTarget::PaeProviderSelectionWeights,
                ),
                candidate("artifact_b", RllOptimizationTarget::ContextRetrievalScoring),
            ],
            30,
            500,
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn rll_envelope_requires_offline_only_true() {
        let out = RllRequestEnvelope::v1(CorrelationId(1), TurnId(1), 4, 4, false);
        assert!(out.is_err());
    }

    #[test]
    fn rll_policy_rank_ok_rejects_missing_selected_artifact() {
        let item = RllRecommendationItem::v1(
            "artifact_a".to_string(),
            RllOptimizationTarget::PaeProviderSelectionWeights,
            1,
            81,
            3,
            "evidence:offline:1".to_string(),
        )
        .unwrap();
        let out = RllPolicyRankOfflineOk::v1(
            ReasonCodeId(1),
            "missing".to_string(),
            vec![item],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn rll_artifact_recommend_ok_fail_requires_diagnostics() {
        let out = RllArtifactRecommendOk::v1(
            ReasonCodeId(2),
            RllValidationStatus::Fail,
            vec![],
            true,
            true,
        );
        assert!(out.is_err());
    }
}
