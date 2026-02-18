#![forbid(unsafe_code)]

use std::cmp::{max, min};
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1rll::{
    Ph1RllRequest, Ph1RllResponse, RllArtifactCandidate, RllArtifactRecommendOk,
    RllArtifactRecommendRequest, RllCapabilityId, RllOptimizationTarget, RllPolicyRankOfflineOk,
    RllPolicyRankOfflineRequest, RllRecommendationItem, RllRefuse, RllValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.RLL reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_RLL_OK_POLICY_RANK_OFFLINE: ReasonCodeId = ReasonCodeId(0x524C_0001);
    pub const PH1_RLL_OK_ARTIFACT_RECOMMEND: ReasonCodeId = ReasonCodeId(0x524C_0002);

    pub const PH1_RLL_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x524C_00F1);
    pub const PH1_RLL_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x524C_00F2);
    pub const PH1_RLL_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x524C_00F3);
    pub const PH1_RLL_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x524C_00F4);
    pub const PH1_RLL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x524C_00F5);
    pub const PH1_RLL_OFFLINE_ONLY_REQUIRED: ReasonCodeId = ReasonCodeId(0x524C_00F6);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1RllConfig {
    pub max_candidates: u8,
    pub max_recommendations: u8,
    pub max_diagnostics: u8,
}

impl Ph1RllConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_candidates: 32,
            max_recommendations: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1RllRuntime {
    config: Ph1RllConfig,
}

impl Ph1RllRuntime {
    pub fn new(config: Ph1RllConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_RLL_INPUT_SCHEMA_INVALID,
                "rll request failed contract validation",
            );
        }

        match req {
            Ph1RllRequest::RllPolicyRankOffline(r) => self.run_policy_rank_offline(r),
            Ph1RllRequest::RllArtifactRecommend(r) => self.run_artifact_recommend(r),
        }
    }

    fn run_policy_rank_offline(&self, req: &RllPolicyRankOfflineRequest) -> Ph1RllResponse {
        if !req.envelope.offline_pipeline_only {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_OFFLINE_ONLY_REQUIRED,
                "rll capability is offline-only",
            );
        }

        if req.candidates.is_empty() {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_UPSTREAM_INPUT_MISSING,
                "candidates is empty",
            );
        }

        let candidate_budget =
            min(req.envelope.max_candidates, self.config.max_candidates) as usize;
        if req.candidates.len() > candidate_budget {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_BUDGET_EXCEEDED,
                "candidate budget exceeded",
            );
        }

        let recommendation_budget = min(
            req.envelope.max_recommendations,
            self.config.max_recommendations,
        ) as usize;
        if recommendation_budget == 0 {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_BUDGET_EXCEEDED,
                "recommendation budget exceeded",
            );
        }

        if req.candidates.iter().any(|c| c.approval_tier != 3) {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_VALIDATION_FAILED,
                "rll artifacts must require approval_tier=3",
            );
        }

        let mut scored = req
            .candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.artifact_id.clone(),
                    candidate.target,
                    candidate.confidence_pct,
                    candidate.approval_tier,
                    candidate.evidence_ref.clone(),
                    score(candidate),
                )
            })
            .collect::<Vec<_>>();

        scored.sort_by(|a, b| b.5.cmp(&a.5).then(a.0.cmp(&b.0)));

        let mut seen = BTreeSet::new();
        let mut ordered_recommendations = Vec::new();

        for (artifact_id, target, confidence_pct, approval_tier, evidence_ref, _) in scored {
            if !seen.insert(artifact_id.clone()) {
                continue;
            }
            if ordered_recommendations.len() >= recommendation_budget {
                break;
            }

            let item = match RllRecommendationItem::v1(
                artifact_id,
                target,
                (ordered_recommendations.len() + 1) as u8,
                confidence_pct,
                approval_tier,
                evidence_ref,
            ) {
                Ok(item) => item,
                Err(_) => {
                    return self.refuse(
                        RllCapabilityId::RllPolicyRankOffline,
                        reason_codes::PH1_RLL_INTERNAL_PIPELINE_ERROR,
                        "failed to construct recommendation item",
                    )
                }
            };
            ordered_recommendations.push(item);
        }

        if ordered_recommendations.is_empty() {
            return self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_UPSTREAM_INPUT_MISSING,
                "no recommendations could be ranked",
            );
        }

        let selected_artifact_id = ordered_recommendations[0].artifact_id.clone();
        match RllPolicyRankOfflineOk::v1(
            reason_codes::PH1_RLL_OK_POLICY_RANK_OFFLINE,
            selected_artifact_id,
            ordered_recommendations,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1RllResponse::RllPolicyRankOfflineOk(ok),
            Err(_) => self.refuse(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_INTERNAL_PIPELINE_ERROR,
                "failed to construct rank output",
            ),
        }
    }

    fn run_artifact_recommend(&self, req: &RllArtifactRecommendRequest) -> Ph1RllResponse {
        if !req.envelope.offline_pipeline_only {
            return self.refuse(
                RllCapabilityId::RllArtifactRecommend,
                reason_codes::PH1_RLL_OFFLINE_ONLY_REQUIRED,
                "rll capability is offline-only",
            );
        }

        if req.ordered_recommendations.is_empty() {
            return self.refuse(
                RllCapabilityId::RllArtifactRecommend,
                reason_codes::PH1_RLL_UPSTREAM_INPUT_MISSING,
                "ordered_recommendations is empty",
            );
        }

        let recommendation_budget = min(
            req.envelope.max_recommendations,
            self.config.max_recommendations,
        ) as usize;
        if req.ordered_recommendations.len() > recommendation_budget {
            return self.refuse(
                RllCapabilityId::RllArtifactRecommend,
                reason_codes::PH1_RLL_BUDGET_EXCEEDED,
                "recommendation budget exceeded",
            );
        }

        let mut diagnostics = Vec::new();

        if req.ordered_recommendations[0].artifact_id != req.selected_artifact_id {
            diagnostics.push("selected_not_first_in_ordered_recommendations".to_string());
        }
        if !req
            .ordered_recommendations
            .iter()
            .any(|item| item.artifact_id == req.selected_artifact_id)
        {
            diagnostics
                .push("selected_artifact_not_present_in_ordered_recommendations".to_string());
        }
        if req
            .ordered_recommendations
            .windows(2)
            .any(|pair| pair[0].confidence_pct < pair[1].confidence_pct)
        {
            diagnostics.push("confidence_not_sorted_desc".to_string());
        }

        let mut expected_rank = 1u8;
        for item in &req.ordered_recommendations {
            if item.rank != expected_rank {
                diagnostics.push("rank_sequence_gap_detected".to_string());
                break;
            }
            expected_rank = expected_rank.saturating_add(1);
        }

        let mut artifact_ids = BTreeSet::new();
        if req
            .ordered_recommendations
            .iter()
            .any(|item| !artifact_ids.insert(item.artifact_id.as_str()))
        {
            diagnostics.push("duplicate_artifact_id".to_string());
        }

        if req
            .ordered_recommendations
            .iter()
            .any(|item| item.approval_tier != 3)
        {
            diagnostics.push("approval_tier_not_strict_3".to_string());
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                RllValidationStatus::Ok,
                reason_codes::PH1_RLL_OK_ARTIFACT_RECOMMEND,
            )
        } else {
            (
                RllValidationStatus::Fail,
                reason_codes::PH1_RLL_VALIDATION_FAILED,
            )
        };

        match RllArtifactRecommendOk::v1(reason_code, validation_status, diagnostics, true, true) {
            Ok(ok) => Ph1RllResponse::RllArtifactRecommendOk(ok),
            Err(_) => self.refuse(
                RllCapabilityId::RllArtifactRecommend,
                reason_codes::PH1_RLL_INTERNAL_PIPELINE_ERROR,
                "failed to construct recommendation-validation output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: RllCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1RllResponse {
        let out = RllRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("RllRefuse::v1 must construct for static messages");
        Ph1RllResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1RllRequest) -> RllCapabilityId {
    match req {
        Ph1RllRequest::RllPolicyRankOffline(_) => RllCapabilityId::RllPolicyRankOffline,
        Ph1RllRequest::RllArtifactRecommend(_) => RllCapabilityId::RllArtifactRecommend,
    }
}

fn score(candidate: &RllArtifactCandidate) -> u8 {
    let mut score = candidate.confidence_pct as i16;
    score += target_bonus(candidate.target);
    score += effect_bonus(candidate.expected_effect_bp);
    max(0, min(score, 100)) as u8
}

fn target_bonus(target: RllOptimizationTarget) -> i16 {
    match target {
        RllOptimizationTarget::PaeProviderSelectionWeights => 10,
        RllOptimizationTarget::PruneClarificationOrdering => 9,
        RllOptimizationTarget::CachePrefetchHeuristics => 8,
        RllOptimizationTarget::ContextRetrievalScoring => 8,
    }
}

fn effect_bonus(expected_effect_bp: i16) -> i16 {
    // convert basis-point effect estimate into bounded score contribution.
    (expected_effect_bp / 40).clamp(-20, 20)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1rll::{RllRequestEnvelope, RllValidationStatus};

    fn runtime() -> Ph1RllRuntime {
        Ph1RllRuntime::new(Ph1RllConfig::mvp_v1())
    }

    fn envelope(max_candidates: u8, max_recommendations: u8) -> RllRequestEnvelope {
        RllRequestEnvelope::v1(
            CorrelationId(2501),
            TurnId(221),
            max_candidates,
            max_recommendations,
            true,
        )
        .unwrap()
    }

    fn candidate(
        artifact_id: &str,
        target: RllOptimizationTarget,
        expected_effect_bp: i16,
        confidence_pct: u8,
    ) -> RllArtifactCandidate {
        RllArtifactCandidate::v1(
            artifact_id.to_string(),
            target,
            expected_effect_bp,
            confidence_pct,
            3,
            "evidence:offline:2".to_string(),
        )
        .unwrap()
    }

    fn candidates() -> Vec<RllArtifactCandidate> {
        vec![
            candidate(
                "artifact_pae",
                RllOptimizationTarget::PaeProviderSelectionWeights,
                220,
                78,
            ),
            candidate(
                "artifact_prune",
                RllOptimizationTarget::PruneClarificationOrdering,
                190,
                76,
            ),
            candidate(
                "artifact_context",
                RllOptimizationTarget::ContextRetrievalScoring,
                170,
                73,
            ),
        ]
    }

    #[test]
    fn at_rll_01_rank_output_is_schema_valid() {
        let req = Ph1RllRequest::RllPolicyRankOffline(
            RllPolicyRankOfflineRequest::v1(envelope(8, 4), candidates(), 30, 400).unwrap(),
        );
        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1RllResponse::RllPolicyRankOfflineOk(ok) => {
                assert_eq!(
                    ok.selected_artifact_id,
                    ok.ordered_recommendations[0].artifact_id
                );
            }
            _ => panic!("expected RllPolicyRankOfflineOk"),
        }
    }

    #[test]
    fn at_rll_02_rank_order_is_deterministic() {
        let req = Ph1RllRequest::RllPolicyRankOffline(
            RllPolicyRankOfflineRequest::v1(envelope(8, 4), candidates(), 30, 400).unwrap(),
        );

        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);
        let ordered1 = match out1 {
            Ph1RllResponse::RllPolicyRankOfflineOk(ok) => ok.ordered_recommendations,
            _ => panic!("expected RllPolicyRankOfflineOk"),
        };
        let ordered2 = match out2 {
            Ph1RllResponse::RllPolicyRankOfflineOk(ok) => ok.ordered_recommendations,
            _ => panic!("expected RllPolicyRankOfflineOk"),
        };
        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_rll_03_offline_only_required_fails_closed() {
        let envelope = RllRequestEnvelope::v1(CorrelationId(3), TurnId(3), 8, 4, true).unwrap();
        let mut bad_candidates = candidates();
        bad_candidates[0].approval_tier = 2;

        let req = Ph1RllRequest::RllPolicyRankOffline(
            RllPolicyRankOfflineRequest::v1(envelope, bad_candidates, 30, 400).unwrap(),
        );
        let out = runtime().run(&req);
        match out {
            Ph1RllResponse::Refuse(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_RLL_VALIDATION_FAILED)
            }
            _ => panic!("expected Refuse"),
        }
    }

    #[test]
    fn at_rll_04_recommend_validation_detects_selection_drift() {
        let rank = runtime().run(&Ph1RllRequest::RllPolicyRankOffline(
            RllPolicyRankOfflineRequest::v1(envelope(8, 4), candidates(), 30, 400).unwrap(),
        ));
        let rank_ok = match rank {
            Ph1RllResponse::RllPolicyRankOfflineOk(ok) => ok,
            _ => panic!("expected RllPolicyRankOfflineOk"),
        };

        let req = Ph1RllRequest::RllArtifactRecommend(
            RllArtifactRecommendRequest::v1(
                envelope(8, 4),
                rank_ok.ordered_recommendations[1].artifact_id.clone(),
                rank_ok.ordered_recommendations,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1RllResponse::RllArtifactRecommendOk(ok) => {
                assert_eq!(ok.validation_status, RllValidationStatus::Fail);
            }
            _ => panic!("expected RllArtifactRecommendOk"),
        }
    }
}
