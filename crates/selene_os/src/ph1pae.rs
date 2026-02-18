#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1pae::{
    PaeAdaptationHintEmitOk, PaeAdaptationHintEmitRequest, PaeCapabilityId, PaeMode,
    PaePolicyCandidate, PaePolicyScoreBuildOk, PaePolicyScoreBuildRequest, PaeRefuse,
    PaeRequestEnvelope, PaeRouteDomain, PaeSignalVector, PaeTargetEngine, PaeValidationStatus,
    Ph1PaeRequest, Ph1PaeResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PAE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PAE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5041_0101);
    pub const PH1_PAE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5041_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PaeWiringConfig {
    pub pae_enabled: bool,
    pub max_signals: u8,
    pub max_candidates: u8,
    pub max_scores: u8,
    pub max_hints: u8,
    pub max_diagnostics: u8,
}

impl Ph1PaeWiringConfig {
    pub fn mvp_v1(pae_enabled: bool) -> Self {
        Self {
            pae_enabled,
            max_signals: 24,
            max_candidates: 8,
            max_scores: 8,
            max_hints: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub device_profile_ref: String,
    pub current_mode: PaeMode,
    pub signals: Vec<PaeSignalVector>,
    pub candidates: Vec<PaePolicyCandidate>,
    pub allowed_targets: Vec<PaeTargetEngine>,
    pub require_governed_artifacts: bool,
    pub minimum_sample_size: u16,
    pub promotion_threshold_bp: i16,
    pub demotion_failure_threshold: u8,
    pub consecutive_threshold_failures: u8,
    pub require_no_runtime_authority_drift: bool,
}

impl PaeTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        device_profile_ref: String,
        current_mode: PaeMode,
        signals: Vec<PaeSignalVector>,
        candidates: Vec<PaePolicyCandidate>,
        allowed_targets: Vec<PaeTargetEngine>,
        require_governed_artifacts: bool,
        minimum_sample_size: u16,
        promotion_threshold_bp: i16,
        demotion_failure_threshold: u8,
        consecutive_threshold_failures: u8,
        require_no_runtime_authority_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            device_profile_ref,
            current_mode,
            signals,
            candidates,
            allowed_targets,
            require_governed_artifacts,
            minimum_sample_size,
            promotion_threshold_bp,
            demotion_failure_threshold,
            consecutive_threshold_failures,
            require_no_runtime_authority_drift,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for PaeTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("pae_turn_input.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "pae_turn_input.device_profile_ref",
            &self.device_profile_ref,
            96,
        )?;

        if self.signals.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_turn_input.signals",
                reason: "must be <= 64",
            });
        }
        for signal in &self.signals {
            signal.validate()?;
        }

        if self.candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_turn_input.candidates",
                reason: "must be <= 32",
            });
        }
        for candidate in &self.candidates {
            candidate.validate()?;
        }

        if self.allowed_targets.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_turn_input.allowed_targets",
                reason: "must be <= 4",
            });
        }
        let mut target_set = BTreeSet::new();
        for target in &self.allowed_targets {
            if !target_set.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_turn_input.allowed_targets",
                    reason: "must be unique",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub score_build: PaePolicyScoreBuildOk,
    pub hint_emit: PaeAdaptationHintEmitOk,
}

impl PaeForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        score_build: PaePolicyScoreBuildOk,
        hint_emit: PaeAdaptationHintEmitOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            score_build,
            hint_emit,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for PaeForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.score_build.validate()?;
        self.hint_emit.validate()?;

        if self.hint_emit.validation_status != PaeValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "pae_forward_bundle.hint_emit.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaeWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoCandidates,
    Refused(PaeRefuse),
    Forwarded(PaeForwardBundle),
}

pub trait Ph1PaeEngine {
    fn run(&self, req: &Ph1PaeRequest) -> Ph1PaeResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PaeWiring<E>
where
    E: Ph1PaeEngine,
{
    config: Ph1PaeWiringConfig,
    engine: E,
}

impl<E> Ph1PaeWiring<E>
where
    E: Ph1PaeEngine,
{
    pub fn new(config: Ph1PaeWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signals == 0 || config.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pae_wiring_config.max_signals",
                reason: "must be within 1..=64",
            });
        }
        if config.max_candidates == 0 || config.max_candidates > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pae_wiring_config.max_candidates",
                reason: "must be within 1..=32",
            });
        }
        if config.max_scores == 0 || config.max_scores > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pae_wiring_config.max_scores",
                reason: "must be within 1..=32",
            });
        }
        if config.max_hints == 0 || config.max_hints > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pae_wiring_config.max_hints",
                reason: "must be within 1..=16",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pae_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }

        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &PaeTurnInput) -> Result<PaeWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.pae_enabled {
            return Ok(PaeWiringOutcome::NotInvokedDisabled);
        }
        if input.candidates.is_empty() {
            return Ok(PaeWiringOutcome::NotInvokedNoCandidates);
        }

        let envelope = PaeRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signals, 64),
            min(self.config.max_candidates, 32),
            min(self.config.max_scores, 32),
            min(self.config.max_hints, 16),
            min(self.config.max_diagnostics, 16),
        )?;

        let score_req = Ph1PaeRequest::PaePolicyScoreBuild(PaePolicyScoreBuildRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.device_profile_ref.clone(),
            input.current_mode,
            input.signals.clone(),
            input.candidates.clone(),
            input.require_governed_artifacts,
            input.minimum_sample_size,
            input.promotion_threshold_bp,
            input.demotion_failure_threshold,
            input.consecutive_threshold_failures,
        )?);
        let score_resp = self.engine.run(&score_req);
        score_resp.validate()?;

        let score_ok = match score_resp {
            Ph1PaeResponse::Refuse(refuse) => return Ok(PaeWiringOutcome::Refused(refuse)),
            Ph1PaeResponse::PaePolicyScoreBuildOk(ok) => ok,
            Ph1PaeResponse::PaeAdaptationHintEmitOk(_) => {
                return Ok(PaeWiringOutcome::Refused(PaeRefuse::v1(
                    PaeCapabilityId::PaePolicyScoreBuild,
                    reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                    "unexpected adaptation-hint response for score-build request".to_string(),
                )?));
            }
        };

        let allowed_targets = if input.allowed_targets.is_empty() {
            infer_targets(score_ok.ordered_scores[0].route_domain)
        } else {
            input.allowed_targets.clone()
        };

        let emit_req = Ph1PaeRequest::PaeAdaptationHintEmit(PaeAdaptationHintEmitRequest::v1(
            envelope,
            input.tenant_id.clone(),
            input.device_profile_ref.clone(),
            score_ok.selected_candidate_id.clone(),
            score_ok.selected_mode,
            score_ok.ordered_scores.clone(),
            allowed_targets,
            input.require_no_runtime_authority_drift,
        )?);
        let emit_resp = self.engine.run(&emit_req);
        emit_resp.validate()?;

        let emit_ok = match emit_resp {
            Ph1PaeResponse::Refuse(refuse) => return Ok(PaeWiringOutcome::Refused(refuse)),
            Ph1PaeResponse::PaeAdaptationHintEmitOk(ok) => ok,
            Ph1PaeResponse::PaePolicyScoreBuildOk(_) => {
                return Ok(PaeWiringOutcome::Refused(PaeRefuse::v1(
                    PaeCapabilityId::PaeAdaptationHintEmit,
                    reason_codes::PH1_PAE_INTERNAL_PIPELINE_ERROR,
                    "unexpected score-build response for adaptation-hint request".to_string(),
                )?));
            }
        };

        if emit_ok.validation_status != PaeValidationStatus::Ok {
            return Ok(PaeWiringOutcome::Refused(PaeRefuse::v1(
                PaeCapabilityId::PaeAdaptationHintEmit,
                reason_codes::PH1_PAE_VALIDATION_FAILED,
                "pae adaptation hint validation failed".to_string(),
            )?));
        }

        let bundle = PaeForwardBundle::v1(input.correlation_id, input.turn_id, score_ok, emit_ok)?;
        Ok(PaeWiringOutcome::Forwarded(bundle))
    }
}

fn infer_targets(route_domain: PaeRouteDomain) -> Vec<PaeTargetEngine> {
    match route_domain {
        PaeRouteDomain::Stt => vec![PaeTargetEngine::Ph1C, PaeTargetEngine::Ph1Cache],
        PaeRouteDomain::Tts => vec![PaeTargetEngine::Ph1Tts, PaeTargetEngine::Ph1Cache],
        PaeRouteDomain::Llm => vec![PaeTargetEngine::Ph1Cache, PaeTargetEngine::Ph1Multi],
        PaeRouteDomain::Tooling => vec![PaeTargetEngine::Ph1Cache],
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1pae::{
        PaeAdaptationHint, PaePolicyScoreBuildOk, PaeProviderSlot, PaeScoreEntry, PaeSignalSource,
    };
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Clone)]
    struct DeterministicPaeEngine {
        force_emit_fail: bool,
    }

    impl Ph1PaeEngine for DeterministicPaeEngine {
        fn run(&self, req: &Ph1PaeRequest) -> Ph1PaeResponse {
            match req {
                Ph1PaeRequest::PaePolicyScoreBuild(r) => {
                    let selected = &r.candidates[0];
                    let score = PaeScoreEntry::v1(
                        selected.candidate_id.clone(),
                        selected.route_domain,
                        selected.provider_slot,
                        selected.proposed_mode,
                        1900,
                        2400,
                        180,
                        220,
                        100,
                        selected.sample_size,
                    )
                    .unwrap();

                    Ph1PaeResponse::PaePolicyScoreBuildOk(
                        PaePolicyScoreBuildOk::v1(
                            ReasonCodeId(1),
                            selected.candidate_id.clone(),
                            vec![score],
                            r.current_mode,
                            false,
                            selected.rollback_to.is_some(),
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PaeRequest::PaeAdaptationHintEmit(r) => {
                    let hint = PaeAdaptationHint::v1(
                        "hint_01".to_string(),
                        r.allowed_targets[0],
                        r.ordered_scores[0].route_domain,
                        "stt_route_plan".to_string(),
                        "mode=ASSIST;slot=PRIMARY;score_bp=1900".to_string(),
                        1900,
                        "pae:selected:c1".to_string(),
                    )
                    .unwrap();

                    Ph1PaeResponse::PaeAdaptationHintEmitOk(
                        PaeAdaptationHintEmitOk::v1(
                            if self.force_emit_fail {
                                ReasonCodeId(2)
                            } else {
                                ReasonCodeId(1)
                            },
                            if self.force_emit_fail {
                                PaeValidationStatus::Fail
                            } else {
                                PaeValidationStatus::Ok
                            },
                            if self.force_emit_fail {
                                vec!["selected_not_first".to_string()]
                            } else {
                                vec![]
                            },
                            r.allowed_targets.clone(),
                            vec![hint],
                            !self.force_emit_fail,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn sample_input() -> PaeTurnInput {
        let signal = PaeSignalVector::v1(
            "sig_01".to_string(),
            PaeSignalSource::Feedback,
            PaeRouteDomain::Stt,
            "quality_trend".to_string(),
            220,
            8600,
            true,
            "feedback:evidence:1".to_string(),
        )
        .unwrap();

        let candidate = PaePolicyCandidate::v1(
            "c1".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            2500,
            190,
            250,
            200,
            180,
            Some("artifact:c1".to_string()),
            None,
        )
        .unwrap();

        PaeTurnInput::v1(
            CorrelationId(9901),
            TurnId(5501),
            "tenant_demo".to_string(),
            "desktop_profile_v1".to_string(),
            PaeMode::Assist,
            vec![signal],
            vec![candidate],
            vec![PaeTargetEngine::Ph1C, PaeTargetEngine::Ph1Cache],
            true,
            120,
            1200,
            3,
            0,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_pae_01_os_invokes_and_returns_forward_bundle() {
        let wiring = Ph1PaeWiring::new(
            Ph1PaeWiringConfig::mvp_v1(true),
            DeterministicPaeEngine {
                force_emit_fail: false,
            },
        )
        .unwrap();

        let out = wiring.run_turn(&sample_input()).unwrap();
        match out {
            PaeWiringOutcome::Forwarded(bundle) => {
                assert_eq!(
                    bundle.score_build.selected_candidate_id,
                    bundle.hint_emit.adaptation_hints[0]
                        .provenance_ref
                        .replace("pae:selected:", "")
                );
            }
            _ => panic!("expected forwarded bundle"),
        }
    }

    #[test]
    fn at_pae_02_disabled_returns_not_invoked() {
        let wiring = Ph1PaeWiring::new(
            Ph1PaeWiringConfig::mvp_v1(false),
            DeterministicPaeEngine {
                force_emit_fail: false,
            },
        )
        .unwrap();

        let out = wiring.run_turn(&sample_input()).unwrap();
        assert_eq!(out, PaeWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_pae_03_missing_candidates_returns_not_invoked() {
        let mut input = sample_input();
        input.candidates.clear();

        let wiring = Ph1PaeWiring::new(
            Ph1PaeWiringConfig::mvp_v1(true),
            DeterministicPaeEngine {
                force_emit_fail: false,
            },
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, PaeWiringOutcome::NotInvokedNoCandidates);
    }

    #[test]
    fn at_pae_04_validation_fail_is_refused() {
        let wiring = Ph1PaeWiring::new(
            Ph1PaeWiringConfig::mvp_v1(true),
            DeterministicPaeEngine {
                force_emit_fail: true,
            },
        )
        .unwrap();

        let out = wiring.run_turn(&sample_input()).unwrap();
        match out {
            PaeWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_PAE_VALIDATION_FAILED);
                assert_eq!(refuse.capability_id, PaeCapabilityId::PaeAdaptationHintEmit);
            }
            _ => panic!("expected refused outcome"),
        }
    }
}
