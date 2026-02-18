#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1rll::{
    Ph1RllRequest, Ph1RllResponse, RllArtifactCandidate, RllArtifactRecommendOk,
    RllArtifactRecommendRequest, RllCapabilityId, RllPolicyRankOfflineOk,
    RllPolicyRankOfflineRequest, RllRefuse, RllRequestEnvelope, RllValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.RLL OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_RLL_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x524C_0101);
    pub const PH1_RLL_OFFLINE_ONLY_REQUIRED: ReasonCodeId = ReasonCodeId(0x524C_0102);
    pub const PH1_RLL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x524C_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1RllWiringConfig {
    pub rll_enabled: bool,
    pub max_candidates: u8,
    pub max_recommendations: u8,
    pub offline_pipeline_only: bool,
}

impl Ph1RllWiringConfig {
    pub fn mvp_v1(rll_enabled: bool) -> Self {
        Self {
            rll_enabled,
            max_candidates: 32,
            max_recommendations: 16,
            offline_pipeline_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllOfflineInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub candidates: Vec<RllArtifactCandidate>,
    pub training_window_days: u16,
    pub minimum_sample_size: u32,
    pub offline_pipeline_only: bool,
}

impl RllOfflineInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        candidates: Vec<RllArtifactCandidate>,
        training_window_days: u16,
        minimum_sample_size: u32,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            candidates,
            training_window_days,
            minimum_sample_size,
            offline_pipeline_only,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for RllOfflineInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.candidates.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_offline_input.candidates",
                reason: "must be <= 64",
            });
        }
        for candidate in &self.candidates {
            candidate.validate()?;
        }
        if self.training_window_days == 0 || self.training_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_offline_input.training_window_days",
                reason: "must be within 1..=365",
            });
        }
        if self.minimum_sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "rll_offline_input.minimum_sample_size",
                reason: "must be > 0",
            });
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "rll_offline_input.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RllForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_rank: RllPolicyRankOfflineOk,
    pub artifact_recommend: RllArtifactRecommendOk,
}

impl RllForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_rank: RllPolicyRankOfflineOk,
        artifact_recommend: RllArtifactRecommendOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            policy_rank,
            artifact_recommend,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for RllForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_rank.validate()?;
        self.artifact_recommend.validate()?;
        if self.artifact_recommend.validation_status != RllValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "rll_forward_bundle.artifact_recommend.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RllWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoCandidates,
    Refused(RllRefuse),
    Forwarded(RllForwardBundle),
}

pub trait Ph1RllEngine {
    fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1RllWiring<E>
where
    E: Ph1RllEngine,
{
    config: Ph1RllWiringConfig,
    engine: E,
}

impl<E> Ph1RllWiring<E>
where
    E: Ph1RllEngine,
{
    pub fn new(config: Ph1RllWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_candidates == 0 || config.max_candidates > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rll_wiring_config.max_candidates",
                reason: "must be within 1..=64",
            });
        }
        if config.max_recommendations == 0 || config.max_recommendations > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1rll_wiring_config.max_recommendations",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_offline(
        &self,
        input: &RllOfflineInput,
    ) -> Result<RllWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.rll_enabled {
            return Ok(RllWiringOutcome::NotInvokedDisabled);
        }
        if input.candidates.is_empty() {
            return Ok(RllWiringOutcome::NotInvokedNoCandidates);
        }

        if !self.config.offline_pipeline_only || !input.offline_pipeline_only {
            return Ok(RllWiringOutcome::Refused(RllRefuse::v1(
                RllCapabilityId::RllPolicyRankOffline,
                reason_codes::PH1_RLL_OFFLINE_ONLY_REQUIRED,
                "rll wiring requires offline pipeline".to_string(),
            )?));
        }

        let envelope = RllRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_candidates, 64),
            min(self.config.max_recommendations, 32),
            true,
        )?;

        let rank_req = Ph1RllRequest::RllPolicyRankOffline(RllPolicyRankOfflineRequest::v1(
            envelope.clone(),
            input.candidates.clone(),
            input.training_window_days,
            input.minimum_sample_size,
        )?);
        let rank_resp = self.engine.run(&rank_req);
        rank_resp.validate()?;

        let rank_ok = match rank_resp {
            Ph1RllResponse::Refuse(refuse) => return Ok(RllWiringOutcome::Refused(refuse)),
            Ph1RllResponse::RllPolicyRankOfflineOk(ok) => ok,
            Ph1RllResponse::RllArtifactRecommendOk(_) => {
                return Ok(RllWiringOutcome::Refused(RllRefuse::v1(
                    RllCapabilityId::RllPolicyRankOffline,
                    reason_codes::PH1_RLL_INTERNAL_PIPELINE_ERROR,
                    "unexpected recommend response for rank request".to_string(),
                )?))
            }
        };

        let recommend_req = Ph1RllRequest::RllArtifactRecommend(RllArtifactRecommendRequest::v1(
            envelope,
            rank_ok.selected_artifact_id.clone(),
            rank_ok.ordered_recommendations.clone(),
        )?);
        let recommend_resp = self.engine.run(&recommend_req);
        recommend_resp.validate()?;

        let recommend_ok = match recommend_resp {
            Ph1RllResponse::Refuse(refuse) => return Ok(RllWiringOutcome::Refused(refuse)),
            Ph1RllResponse::RllArtifactRecommendOk(ok) => ok,
            Ph1RllResponse::RllPolicyRankOfflineOk(_) => {
                return Ok(RllWiringOutcome::Refused(RllRefuse::v1(
                    RllCapabilityId::RllArtifactRecommend,
                    reason_codes::PH1_RLL_INTERNAL_PIPELINE_ERROR,
                    "unexpected rank response for recommend request".to_string(),
                )?))
            }
        };

        if recommend_ok.validation_status != RllValidationStatus::Ok {
            return Ok(RllWiringOutcome::Refused(RllRefuse::v1(
                RllCapabilityId::RllArtifactRecommend,
                reason_codes::PH1_RLL_VALIDATION_FAILED,
                "rll recommendation validation failed".to_string(),
            )?));
        }

        let bundle =
            RllForwardBundle::v1(input.correlation_id, input.turn_id, rank_ok, recommend_ok)?;
        Ok(RllWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1rll::{
        RllArtifactCandidate, RllArtifactRecommendOk, RllOptimizationTarget,
        RllPolicyRankOfflineOk, RllRecommendationItem,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicRllEngine;

    impl Ph1RllEngine for DeterministicRllEngine {
        fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse {
            match req {
                Ph1RllRequest::RllPolicyRankOffline(r) => {
                    let mut items = r
                        .candidates
                        .iter()
                        .enumerate()
                        .map(|(idx, c)| {
                            RllRecommendationItem::v1(
                                c.artifact_id.clone(),
                                c.target,
                                (idx + 1) as u8,
                                c.confidence_pct,
                                c.approval_tier,
                                c.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    items.sort_by(|a, b| {
                        b.confidence_pct
                            .cmp(&a.confidence_pct)
                            .then(a.artifact_id.cmp(&b.artifact_id))
                    });
                    for (idx, item) in items.iter_mut().enumerate() {
                        item.rank = (idx + 1) as u8;
                    }

                    Ph1RllResponse::RllPolicyRankOfflineOk(
                        RllPolicyRankOfflineOk::v1(
                            ReasonCodeId(31),
                            items[0].artifact_id.clone(),
                            items,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1RllRequest::RllArtifactRecommend(_r) => Ph1RllResponse::RllArtifactRecommendOk(
                    RllArtifactRecommendOk::v1(
                        ReasonCodeId(32),
                        RllValidationStatus::Ok,
                        vec![],
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftRllEngine;

    impl Ph1RllEngine for DriftRllEngine {
        fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse {
            match req {
                Ph1RllRequest::RllPolicyRankOffline(r) => {
                    let item = RllRecommendationItem::v1(
                        r.candidates[0].artifact_id.clone(),
                        r.candidates[0].target,
                        1,
                        r.candidates[0].confidence_pct,
                        3,
                        r.candidates[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1RllResponse::RllPolicyRankOfflineOk(
                        RllPolicyRankOfflineOk::v1(
                            ReasonCodeId(41),
                            item.artifact_id.clone(),
                            vec![item],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1RllRequest::RllArtifactRecommend(_r) => Ph1RllResponse::RllArtifactRecommendOk(
                    RllArtifactRecommendOk::v1(
                        ReasonCodeId(42),
                        RllValidationStatus::Fail,
                        vec!["selected_not_first_in_ordered_recommendations".to_string()],
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn candidate(
        id: &str,
        target: RllOptimizationTarget,
        confidence_pct: u8,
    ) -> RllArtifactCandidate {
        RllArtifactCandidate::v1(
            id.to_string(),
            target,
            200,
            confidence_pct,
            3,
            "evidence:offline:3".to_string(),
        )
        .unwrap()
    }

    fn candidates() -> Vec<RllArtifactCandidate> {
        vec![
            candidate(
                "artifact_pae",
                RllOptimizationTarget::PaeProviderSelectionWeights,
                81,
            ),
            candidate(
                "artifact_prune",
                RllOptimizationTarget::PruneClarificationOrdering,
                78,
            ),
            candidate(
                "artifact_cache",
                RllOptimizationTarget::CachePrefetchHeuristics,
                75,
            ),
        ]
    }

    #[test]
    fn at_rll_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1RllWiring::new(Ph1RllWiringConfig::mvp_v1(true), DeterministicRllEngine).unwrap();
        let input = RllOfflineInput::v1(
            CorrelationId(2601),
            TurnId(231),
            candidates(),
            30,
            500,
            true,
        )
        .unwrap();

        let out = wiring.run_offline(&input).unwrap();
        match out {
            RllWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(bundle.policy_rank.selected_artifact_id, "artifact_pae");
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_rll_02_os_order_is_deterministic_for_same_input() {
        let wiring =
            Ph1RllWiring::new(Ph1RllWiringConfig::mvp_v1(true), DeterministicRllEngine).unwrap();
        let input = RllOfflineInput::v1(
            CorrelationId(2602),
            TurnId(232),
            candidates(),
            30,
            500,
            true,
        )
        .unwrap();

        let out1 = wiring.run_offline(&input).unwrap();
        let out2 = wiring.run_offline(&input).unwrap();

        let ordered1 = match out1 {
            RllWiringOutcome::Forwarded(bundle) => bundle.policy_rank.ordered_recommendations,
            _ => panic!("expected Forwarded"),
        };
        let ordered2 = match out2 {
            RllWiringOutcome::Forwarded(bundle) => bundle.policy_rank.ordered_recommendations,
            _ => panic!("expected Forwarded"),
        };

        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_rll_03_os_does_not_invoke_when_disabled() {
        let wiring =
            Ph1RllWiring::new(Ph1RllWiringConfig::mvp_v1(false), DeterministicRllEngine).unwrap();
        let input = RllOfflineInput::v1(
            CorrelationId(2603),
            TurnId(233),
            candidates(),
            30,
            500,
            true,
        )
        .unwrap();

        let out = wiring.run_offline(&input).unwrap();
        assert_eq!(out, RllWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_rll_04_os_fails_closed_on_recommend_validation_drift() {
        let wiring = Ph1RllWiring::new(Ph1RllWiringConfig::mvp_v1(true), DriftRllEngine).unwrap();
        let input = RllOfflineInput::v1(
            CorrelationId(2604),
            TurnId(234),
            candidates(),
            30,
            500,
            true,
        )
        .unwrap();

        let out = wiring.run_offline(&input).unwrap();
        match out {
            RllWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.capability_id, RllCapabilityId::RllArtifactRecommend);
                assert_eq!(refuse.reason_code, reason_codes::PH1_RLL_VALIDATION_FAILED);
            }
            _ => panic!("expected Refused"),
        }
    }
}
