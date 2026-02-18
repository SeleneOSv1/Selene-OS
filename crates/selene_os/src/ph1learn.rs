#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1learn::{
    LearnArtifactPackageBuildOk, LearnArtifactPackageBuildRequest, LearnArtifactTarget,
    LearnCapabilityId, LearnRefuse, LearnRequestEnvelope, LearnSignal, LearnSignalAggregateOk,
    LearnSignalAggregateRequest, LearnTargetEngine, LearnValidationStatus, Ph1LearnRequest,
    Ph1LearnResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LEARN OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LEARN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C45_0101);
    pub const PH1_LEARN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C45_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LearnWiringConfig {
    pub learn_enabled: bool,
    pub max_signals: u8,
    pub max_artifacts: u8,
    pub max_diagnostics: u8,
}

impl Ph1LearnWiringConfig {
    pub fn mvp_v1(learn_enabled: bool) -> Self {
        Self {
            learn_enabled,
            max_signals: 24,
            max_artifacts: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub signals: Vec<LearnSignal>,
    pub requested_target_engines: Vec<LearnTargetEngine>,
    pub require_derived_only_global: bool,
    pub no_runtime_drift_required: bool,
}

impl LearnTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        signals: Vec<LearnSignal>,
        requested_target_engines: Vec<LearnTargetEngine>,
        require_derived_only_global: bool,
        no_runtime_drift_required: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            signals,
            requested_target_engines,
            require_derived_only_global,
            no_runtime_drift_required,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for LearnTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("learn_turn_input.tenant_id", &self.tenant_id, 64)?;
        if self.signals.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_turn_input.signals",
                reason: "must be <= 128",
            });
        }
        for signal in &self.signals {
            signal.validate()?;
            if signal.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_turn_input.signals",
                    reason: "signal tenant_id must match input tenant_id",
                });
            }
        }
        if self.requested_target_engines.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_turn_input.requested_target_engines",
                reason: "must be <= 8",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_aggregate: LearnSignalAggregateOk,
    pub artifact_package_build: LearnArtifactPackageBuildOk,
}

impl LearnForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_aggregate: LearnSignalAggregateOk,
        artifact_package_build: LearnArtifactPackageBuildOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            signal_aggregate,
            artifact_package_build,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for LearnForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.signal_aggregate.validate()?;
        self.artifact_package_build.validate()?;
        if self.artifact_package_build.validation_status != LearnValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "learn_forward_bundle.artifact_package_build.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LearnWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(LearnRefuse),
    Forwarded(LearnForwardBundle),
}

pub trait Ph1LearnEngine {
    fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1LearnWiring<E>
where
    E: Ph1LearnEngine,
{
    config: Ph1LearnWiringConfig,
    engine: E,
}

impl<E> Ph1LearnWiring<E>
where
    E: Ph1LearnEngine,
{
    pub fn new(config: Ph1LearnWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signals == 0 || config.max_signals > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1learn_wiring_config.max_signals",
                reason: "must be within 1..=128",
            });
        }
        if config.max_artifacts == 0 || config.max_artifacts > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1learn_wiring_config.max_artifacts",
                reason: "must be within 1..=64",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1learn_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }

        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &LearnTurnInput,
    ) -> Result<LearnWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.learn_enabled {
            return Ok(LearnWiringOutcome::NotInvokedDisabled);
        }
        if input.signals.is_empty() {
            return Ok(LearnWiringOutcome::NotInvokedNoSignals);
        }

        let envelope = LearnRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signals, 128),
            min(self.config.max_artifacts, 64),
            min(self.config.max_diagnostics, 16),
        )?;

        let aggregate_req = Ph1LearnRequest::LearnSignalAggregate(LearnSignalAggregateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.signals.clone(),
            input.require_derived_only_global,
            input.no_runtime_drift_required,
        )?);
        let aggregate_resp = self.engine.run(&aggregate_req);
        aggregate_resp.validate()?;

        let aggregate_ok = match aggregate_resp {
            Ph1LearnResponse::Refuse(refuse) => return Ok(LearnWiringOutcome::Refused(refuse)),
            Ph1LearnResponse::LearnSignalAggregateOk(ok) => ok,
            Ph1LearnResponse::LearnArtifactPackageBuildOk(_) => {
                return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                    LearnCapabilityId::LearnSignalAggregate,
                    reason_codes::PH1_LEARN_INTERNAL_PIPELINE_ERROR,
                    "unexpected package-build response for signal-aggregate request".to_string(),
                )?));
            }
        };

        let target_engines = if input.requested_target_engines.is_empty() {
            infer_targets_from_artifacts(&aggregate_ok.ordered_artifacts)
        } else {
            input.requested_target_engines.clone()
        };

        let package_req =
            Ph1LearnRequest::LearnArtifactPackageBuild(LearnArtifactPackageBuildRequest::v1(
                envelope,
                input.tenant_id.clone(),
                aggregate_ok.selected_artifact_id.clone(),
                aggregate_ok.ordered_artifacts.clone(),
                target_engines,
                true,
                true,
                input.no_runtime_drift_required,
            )?);
        let package_resp = self.engine.run(&package_req);
        package_resp.validate()?;

        let package_ok = match package_resp {
            Ph1LearnResponse::Refuse(refuse) => return Ok(LearnWiringOutcome::Refused(refuse)),
            Ph1LearnResponse::LearnArtifactPackageBuildOk(ok) => ok,
            Ph1LearnResponse::LearnSignalAggregateOk(_) => {
                return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                    LearnCapabilityId::LearnArtifactPackageBuild,
                    reason_codes::PH1_LEARN_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-aggregate response for package-build request".to_string(),
                )?));
            }
        };

        if package_ok.validation_status != LearnValidationStatus::Ok {
            return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_VALIDATION_FAILED,
                "learn artifact package validation failed".to_string(),
            )?));
        }

        let bundle = LearnForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            aggregate_ok,
            package_ok,
        )?;
        Ok(LearnWiringOutcome::Forwarded(bundle))
    }
}

fn infer_targets_from_artifacts(
    artifacts: &[selene_kernel_contracts::ph1learn::LearnArtifactCandidate],
) -> Vec<LearnTargetEngine> {
    let mut targets = Vec::new();
    let mut seen = BTreeSet::new();

    for artifact in artifacts {
        for target in targets_for_artifact(artifact.target) {
            if seen.insert(target.as_str()) {
                targets.push(target);
            }
        }
    }

    if targets.is_empty() {
        targets.push(LearnTargetEngine::Pae);
    }

    targets
}

fn targets_for_artifact(target: LearnArtifactTarget) -> Vec<LearnTargetEngine> {
    match target {
        LearnArtifactTarget::KnowTenantGlossaryPack => vec![LearnTargetEngine::Know],
        LearnArtifactTarget::PronLexiconPack => vec![LearnTargetEngine::Pron],
        LearnArtifactTarget::CacheDecisionSkeleton => vec![LearnTargetEngine::Cache],
        LearnArtifactTarget::PruneClarificationOrdering => vec![LearnTargetEngine::Prune],
        LearnArtifactTarget::PaeRoutingWeights => vec![LearnTargetEngine::Pae],
        LearnArtifactTarget::SearchWebExtractionHints => vec![LearnTargetEngine::Search],
        LearnArtifactTarget::ListenEnvironmentProfile => vec![LearnTargetEngine::Listen],
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
    if value
        .chars()
        .any(|c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'))
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1learn::{
        LearnArtifactCandidate, LearnArtifactPackageBuildOk, LearnScope, LearnSignalAggregateOk,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicLearnEngine;

    impl Ph1LearnEngine for DeterministicLearnEngine {
        fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
            match req {
                Ph1LearnRequest::LearnSignalAggregate(r) => {
                    let artifacts = r
                        .signals
                        .iter()
                        .enumerate()
                        .map(|(idx, signal)| {
                            LearnArtifactCandidate::v1(
                                format!("artifact_{}", signal.signal_id),
                                if idx == 0 {
                                    LearnArtifactTarget::PaeRoutingWeights
                                } else {
                                    LearnArtifactTarget::KnowTenantGlossaryPack
                                },
                                LearnScope::Tenant,
                                Some(r.tenant_id.clone()),
                                8 - idx as u32,
                                900 - idx as i16 * 20,
                                signal.evidence_ref.clone(),
                                Some(format!("artifact_{}.prev", signal.signal_id)),
                                true,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();

                    Ph1LearnResponse::LearnSignalAggregateOk(
                        LearnSignalAggregateOk::v1(
                            ReasonCodeId(701),
                            artifacts[0].artifact_id.clone(),
                            artifacts,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1LearnRequest::LearnArtifactPackageBuild(r) => {
                    Ph1LearnResponse::LearnArtifactPackageBuildOk(
                        LearnArtifactPackageBuildOk::v1(
                            ReasonCodeId(702),
                            LearnValidationStatus::Ok,
                            vec![],
                            r.target_engines.clone(),
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftLearnEngine;

    impl Ph1LearnEngine for DriftLearnEngine {
        fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
            match req {
                Ph1LearnRequest::LearnSignalAggregate(r) => {
                    let artifact = LearnArtifactCandidate::v1(
                        format!("artifact_{}", r.signals[0].signal_id),
                        LearnArtifactTarget::PaeRoutingWeights,
                        LearnScope::Tenant,
                        Some(r.tenant_id.clone()),
                        6,
                        760,
                        r.signals[0].evidence_ref.clone(),
                        Some(format!("artifact_{}.prev", r.signals[0].signal_id)),
                        true,
                    )
                    .unwrap();

                    Ph1LearnResponse::LearnSignalAggregateOk(
                        LearnSignalAggregateOk::v1(
                            ReasonCodeId(711),
                            artifact.artifact_id.clone(),
                            vec![artifact],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1LearnRequest::LearnArtifactPackageBuild(r) => {
                    Ph1LearnResponse::LearnArtifactPackageBuildOk(
                        LearnArtifactPackageBuildOk::v1(
                            ReasonCodeId(712),
                            LearnValidationStatus::Fail,
                            vec!["artifact_order_not_canonical".to_string()],
                            r.target_engines.clone(),
                            false,
                            false,
                            false,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn signal(
        signal_id: &str,
        signal_type: selene_kernel_contracts::ph1learn::LearnSignalType,
    ) -> LearnSignal {
        LearnSignal::v1(
            signal_id.to_string(),
            "tenant_1".to_string(),
            signal_type,
            LearnScope::Tenant,
            Some("tenant_1".to_string()),
            "metric_key".to_string(),
            150,
            6,
            false,
            false,
            false,
            format!("learn:evidence:{}", signal_id),
        )
        .unwrap()
    }

    fn input() -> LearnTurnInput {
        LearnTurnInput::v1(
            CorrelationId(5301),
            TurnId(501),
            "tenant_1".to_string(),
            vec![
                signal(
                    "sig_1",
                    selene_kernel_contracts::ph1learn::LearnSignalType::SttReject,
                ),
                signal(
                    "sig_2",
                    selene_kernel_contracts::ph1learn::LearnSignalType::UserCorrection,
                ),
            ],
            vec![],
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_learn_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DeterministicLearnEngine)
                .unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        match outcome {
            LearnWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.signal_aggregate.selected_artifact_id,
                    bundle.signal_aggregate.ordered_artifacts[0].artifact_id
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_learn_02_validation_fail_is_refused() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DriftLearnEngine).unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        match outcome {
            LearnWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_LEARN_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_learn_03_disabled_returns_not_invoked() {
        let wiring = Ph1LearnWiring::new(
            Ph1LearnWiringConfig::mvp_v1(false),
            DeterministicLearnEngine,
        )
        .unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        assert_eq!(outcome, LearnWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_learn_04_empty_signal_input_returns_not_invoked() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DeterministicLearnEngine)
                .unwrap();

        let empty_input = LearnTurnInput::v1(
            CorrelationId(5301),
            TurnId(501),
            "tenant_1".to_string(),
            vec![],
            vec![],
            true,
            true,
        )
        .unwrap();

        let outcome = wiring.run_turn(&empty_input).unwrap();
        assert_eq!(outcome, LearnWiringOutcome::NotInvokedNoSignals);
    }
}
