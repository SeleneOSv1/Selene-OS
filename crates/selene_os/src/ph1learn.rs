#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1learn::{
    LearnArtifactPackageBuildOk, LearnArtifactPackageBuildRequest, LearnArtifactTarget,
    LearnCapabilityId, LearnRefuse, LearnRequestEnvelope, LearnSignal, LearnSignalAggregateOk,
    LearnSignalAggregateRequest, LearnSignalType, LearnTargetEngine, LearnValidationStatus,
    Ph1LearnRequest, Ph1LearnResponse,
};
use selene_kernel_contracts::ph1selfheal::{
    stable_card_id, FailureEvent, FixCard, FixKind, FixSource, ProblemCard, ProblemCardState,
    SelfHealValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LEARN OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LEARN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C45_0101);
    pub const PH1_LEARN_FROZEN_EVAL_LEAKAGE: ReasonCodeId = ReasonCodeId(0x4C45_0102);
    pub const PH1_LEARN_CALIBRATION_MIN_SAMPLES: ReasonCodeId = ReasonCodeId(0x4C45_0103);
    pub const PH1_LEARN_DRIFT_RATE_LIMITED: ReasonCodeId = ReasonCodeId(0x4C45_0104);
    pub const PH1_LEARN_ROLLBACK_METADATA_MISSING: ReasonCodeId = ReasonCodeId(0x4C45_0105);
    pub const PH1_LEARN_GOV_HANDOFF_BLOCKED: ReasonCodeId = ReasonCodeId(0x4C45_0106);
    pub const PH1_LEARN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C45_01F1);
}

const LEARN_VOICE_CALIBRATION_MIN_SAMPLES: u32 = 24;
const LEARN_DRIFT_MAX_ADAPTATIONS_24H: u8 = 1;
const LEARN_DRIFT_MAX_REENROLL_PROMPTS_72H: u8 = 1;
const LEARN_DRIFT_MAX_BREACHES_14D: u8 = 3;

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
    pub recent_user_adaptations_24h: u8,
    pub recent_user_reenroll_prompts_72h: u8,
    pub recent_user_drift_breaches_14d: u8,
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
        Self::v2(
            correlation_id,
            turn_id,
            tenant_id,
            signals,
            requested_target_engines,
            require_derived_only_global,
            no_runtime_drift_required,
            0,
            0,
            0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v2(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        signals: Vec<LearnSignal>,
        requested_target_engines: Vec<LearnTargetEngine>,
        require_derived_only_global: bool,
        no_runtime_drift_required: bool,
        recent_user_adaptations_24h: u8,
        recent_user_reenroll_prompts_72h: u8,
        recent_user_drift_breaches_14d: u8,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            signals,
            requested_target_engines,
            require_derived_only_global,
            no_runtime_drift_required,
            recent_user_adaptations_24h,
            recent_user_reenroll_prompts_72h,
            recent_user_drift_breaches_14d,
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
        if self.recent_user_adaptations_24h > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_turn_input.recent_user_adaptations_24h",
                reason: "must be <= 16",
            });
        }
        if self.recent_user_reenroll_prompts_72h > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_turn_input.recent_user_reenroll_prompts_72h",
                reason: "must be <= 16",
            });
        }
        if self.recent_user_drift_breaches_14d > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_turn_input.recent_user_drift_breaches_14d",
                reason: "must be <= 32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd)]
pub enum VoiceArtifactFamily {
    ThresholdPack,
    ConfusionPairPack,
    SpoofPolicyPack,
    ProfileAdaptationPack,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VoiceCalibrationSnapshot {
    pub sample_count: u32,
    pub tar_bp: u16,
    pub far_bp: u16,
    pub frr_bp: u16,
    pub roc_auc_bp: u16,
    pub ci_low_bp: u16,
    pub ci_high_bp: u16,
}

impl VoiceCalibrationSnapshot {
    pub fn v1(
        sample_count: u32,
        tar_bp: u16,
        far_bp: u16,
        frr_bp: u16,
        roc_auc_bp: u16,
        ci_low_bp: u16,
        ci_high_bp: u16,
    ) -> Result<Self, ContractViolation> {
        let snapshot = Self {
            sample_count,
            tar_bp,
            far_bp,
            frr_bp,
            roc_auc_bp,
            ci_low_bp,
            ci_high_bp,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

impl Validate for VoiceCalibrationSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.sample_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "voice_calibration_snapshot.sample_count",
                reason: "must be > 0",
            });
        }
        if self.tar_bp > 10_000
            || self.far_bp > 10_000
            || self.frr_bp > 10_000
            || self.roc_auc_bp > 10_000
            || self.ci_low_bp > 10_000
            || self.ci_high_bp > 10_000
        {
            return Err(ContractViolation::InvalidValue {
                field: "voice_calibration_snapshot",
                reason: "all basis-point values must be within 0..=10000",
            });
        }
        if self.ci_low_bp > self.ci_high_bp {
            return Err(ContractViolation::InvalidValue {
                field: "voice_calibration_snapshot.ci",
                reason: "ci_low_bp must be <= ci_high_bp",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnGovArtifactProposal {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub artifact_id: String,
    pub artifact_family: VoiceArtifactFamily,
    pub artifact_version: u32,
    pub expected_effect_bp: i16,
    pub rollback_to: String,
    pub prior_version_compatible: bool,
    pub no_train_eval_leakage: bool,
    pub drift_guard_applied: bool,
    pub calibration: VoiceCalibrationSnapshot,
}

impl LearnGovArtifactProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        artifact_id: String,
        artifact_family: VoiceArtifactFamily,
        artifact_version: u32,
        expected_effect_bp: i16,
        rollback_to: String,
        prior_version_compatible: bool,
        no_train_eval_leakage: bool,
        drift_guard_applied: bool,
        calibration: VoiceCalibrationSnapshot,
    ) -> Result<Self, ContractViolation> {
        let proposal = Self {
            correlation_id,
            turn_id,
            tenant_id,
            artifact_id,
            artifact_family,
            artifact_version,
            expected_effect_bp,
            rollback_to,
            prior_version_compatible,
            no_train_eval_leakage,
            drift_guard_applied,
            calibration,
        };
        proposal.validate()?;
        Ok(proposal)
    }
}

impl Validate for LearnGovArtifactProposal {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("learn_gov_artifact_proposal.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "learn_gov_artifact_proposal.artifact_id",
            &self.artifact_id,
            128,
        )?;
        if self.artifact_version == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_gov_artifact_proposal.artifact_version",
                reason: "must be > 0",
            });
        }
        if self.expected_effect_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_gov_artifact_proposal.expected_effect_bp",
                reason: "must be within -20000..=20000 basis points",
            });
        }
        validate_token(
            "learn_gov_artifact_proposal.rollback_to",
            &self.rollback_to,
            128,
        )?;
        if !self.prior_version_compatible {
            return Err(ContractViolation::InvalidValue {
                field: "learn_gov_artifact_proposal.prior_version_compatible",
                reason: "must be true",
            });
        }
        if !self.no_train_eval_leakage {
            return Err(ContractViolation::InvalidValue {
                field: "learn_gov_artifact_proposal.no_train_eval_leakage",
                reason: "must be true",
            });
        }
        self.calibration.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_aggregate: LearnSignalAggregateOk,
    pub artifact_package_build: LearnArtifactPackageBuildOk,
    pub gov_artifact_proposals: Vec<LearnGovArtifactProposal>,
}

impl LearnForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_aggregate: LearnSignalAggregateOk,
        artifact_package_build: LearnArtifactPackageBuildOk,
        gov_artifact_proposals: Vec<LearnGovArtifactProposal>,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            signal_aggregate,
            artifact_package_build,
            gov_artifact_proposals,
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
        if self.gov_artifact_proposals.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_forward_bundle.gov_artifact_proposals",
                reason: "must be <= 32",
            });
        }
        for proposal in &self.gov_artifact_proposals {
            proposal.validate()?;
            if proposal.correlation_id != self.correlation_id {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_forward_bundle.gov_artifact_proposals",
                    reason: "proposal correlation_id must match bundle correlation_id",
                });
            }
            if proposal.turn_id != self.turn_id {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_forward_bundle.gov_artifact_proposals",
                    reason: "proposal turn_id must match bundle turn_id",
                });
            }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemCardEscalationState {
    pub state: ProblemCardState,
    pub requires_human: bool,
    pub bcast_id: Option<String>,
    pub unresolved_reason: Option<String>,
}

impl ProblemCardEscalationState {
    pub fn open() -> Self {
        Self {
            state: ProblemCardState::Open,
            requires_human: false,
            bcast_id: None,
            unresolved_reason: None,
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn map_learn_bundle_to_problem_card(
    failure_event: &FailureEvent,
    turn_input: &LearnTurnInput,
    bundle: &LearnForwardBundle,
    owner_engine: String,
    first_seen_at: MonotonicTimeNs,
    last_seen_at: MonotonicTimeNs,
    escalation: ProblemCardEscalationState,
) -> Result<ProblemCard, ContractViolation> {
    failure_event.validate()?;
    turn_input.validate()?;
    bundle.validate()?;
    validate_token(
        "map_learn_bundle_to_problem_card.owner_engine",
        &owner_engine,
        64,
    )?;

    if failure_event.tenant_id != turn_input.tenant_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_learn_bundle_to_problem_card.turn_input.tenant_id",
            reason: "must match failure_event.tenant_id",
        });
    }
    if failure_event.correlation_id != bundle.correlation_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_learn_bundle_to_problem_card.bundle.correlation_id",
            reason: "must match failure_event.correlation_id",
        });
    }
    if failure_event.turn_id != bundle.turn_id {
        return Err(ContractViolation::InvalidValue {
            field: "map_learn_bundle_to_problem_card.bundle.turn_id",
            reason: "must match failure_event.turn_id",
        });
    }

    let selected_artifact =
        find_selected_artifact(bundle).ok_or(ContractViolation::InvalidValue {
            field: "map_learn_bundle_to_problem_card.bundle.signal_aggregate.selected_artifact_id",
            reason: "must resolve to an artifact in ordered_artifacts",
        })?;

    let signal_ids = turn_input
        .signals
        .iter()
        .map(|signal| signal.signal_id.clone())
        .collect::<Vec<_>>();
    let mut evidence_refs = turn_input
        .signals
        .iter()
        .map(|signal| signal.evidence_ref.clone())
        .collect::<Vec<_>>();
    if !evidence_refs
        .iter()
        .any(|v| v == &failure_event.evidence_ref)
    {
        evidence_refs.push(failure_event.evidence_ref.clone());
    }
    if !evidence_refs
        .iter()
        .any(|v| v == &selected_artifact.provenance_ref)
    {
        evidence_refs.push(selected_artifact.provenance_ref.clone());
    }

    let recurrence_count = turn_input
        .signals
        .iter()
        .map(|signal| signal.occurrence_count as u32)
        .sum::<u32>()
        .max(1);
    let quality_impact_sum = turn_input
        .signals
        .iter()
        .map(|signal| i32::from(signal.metric_value_bp))
        .sum::<i32>();
    let quality_impact_bp =
        (quality_impact_sum / turn_input.signals.len() as i32).clamp(-20_000, 20_000) as i16;

    let problem_id = stable_card_id(
        "problem",
        &[
            turn_input.tenant_id.as_str(),
            failure_event.fingerprint.as_str(),
            selected_artifact.artifact_id.as_str(),
        ],
    )?;
    let idempotency_key = stable_card_id(
        "idem_problem",
        &[problem_id.as_str(), failure_event.idempotency_key.as_str()],
    )?;

    ProblemCard::v1(
        problem_id,
        failure_event.fingerprint.clone(),
        turn_input.tenant_id.clone(),
        owner_engine,
        selected_artifact.scope,
        selected_artifact.scope_ref.clone(),
        first_seen_at,
        last_seen_at,
        recurrence_count,
        failure_event.failure_id.clone(),
        signal_ids,
        evidence_refs,
        quality_impact_bp,
        0,
        0,
        escalation.state,
        escalation.requires_human,
        escalation.bcast_id,
        escalation.unresolved_reason,
        idempotency_key,
    )
}

pub fn map_learn_bundle_to_fix_card(
    problem_card: &ProblemCard,
    bundle: &LearnForwardBundle,
) -> Result<FixCard, ContractViolation> {
    problem_card.validate()?;
    bundle.validate()?;

    let selected_artifact =
        find_selected_artifact(bundle).ok_or(ContractViolation::InvalidValue {
            field: "map_learn_bundle_to_fix_card.bundle.signal_aggregate.selected_artifact_id",
            reason: "must resolve to an artifact in ordered_artifacts",
        })?;

    let fix_id = stable_card_id(
        "fix",
        &[
            problem_card.problem_id.as_str(),
            selected_artifact.artifact_id.as_str(),
            &selected_artifact.artifact_version.to_string(),
        ],
    )?;
    let idempotency_key = stable_card_id(
        "idem_fix",
        &[fix_id.as_str(), problem_card.idempotency_key.as_str()],
    )?;

    FixCard::v1(
        fix_id,
        problem_card.problem_id.clone(),
        FixSource::Learn,
        FixKind::Artifact,
        Some(selected_artifact.artifact_id.clone()),
        Some(selected_artifact.target),
        Some(selected_artifact.artifact_version),
        Some(selected_artifact.expected_effect_bp),
        selected_artifact.rollback_to.clone(),
        Some(selected_artifact.provenance_ref.clone()),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        if bundle.artifact_package_build.validation_status == LearnValidationStatus::Ok {
            SelfHealValidationStatus::Ok
        } else {
            SelfHealValidationStatus::Fail
        },
        bundle.artifact_package_build.diagnostics.clone(),
        bundle.signal_aggregate.advisory_only && bundle.artifact_package_build.advisory_only,
        bundle.signal_aggregate.no_execution_authority
            && bundle.artifact_package_build.no_execution_authority,
        idempotency_key,
    )
}

fn find_selected_artifact(
    bundle: &LearnForwardBundle,
) -> Option<&selene_kernel_contracts::ph1learn::LearnArtifactCandidate> {
    bundle
        .signal_aggregate
        .ordered_artifacts
        .iter()
        .find(|artifact| artifact.artifact_id == bundle.signal_aggregate.selected_artifact_id)
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
        if has_frozen_eval_evidence_leakage(&input.signals) {
            return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                LearnCapabilityId::LearnSignalAggregate,
                reason_codes::PH1_LEARN_FROZEN_EVAL_LEAKAGE,
                "frozen-eval evidence rows must never enter learn training path".to_string(),
            )?));
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

        let required_families = required_voice_artifact_families(&input.signals);
        let voice_artifacts = aggregate_ok
            .ordered_artifacts
            .iter()
            .filter(|artifact| voice_artifact_family_for_target(artifact.target).is_some())
            .cloned()
            .collect::<Vec<_>>();
        if !required_families.is_empty() {
            let built_families = voice_artifacts
                .iter()
                .filter_map(|artifact| voice_artifact_family_for_target(artifact.target))
                .collect::<BTreeSet<_>>();
            if !required_families.is_subset(&built_families) {
                return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                    LearnCapabilityId::LearnArtifactPackageBuild,
                    reason_codes::PH1_LEARN_VALIDATION_FAILED,
                    "missing required voice artifact family in learn package".to_string(),
                )?));
            }
        }

        let drift_adaptation_required =
            required_families.contains(&VoiceArtifactFamily::ProfileAdaptationPack);
        if drift_adaptation_required
            && (input.recent_user_adaptations_24h >= LEARN_DRIFT_MAX_ADAPTATIONS_24H
                || input.recent_user_reenroll_prompts_72h >= LEARN_DRIFT_MAX_REENROLL_PROMPTS_72H
                || input.recent_user_drift_breaches_14d >= LEARN_DRIFT_MAX_BREACHES_14D)
        {
            return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_DRIFT_RATE_LIMITED,
                "drift adaptation or re-enrollment is rate limited to prevent thrash".to_string(),
            )?));
        }

        if voice_artifacts
            .iter()
            .any(|artifact| artifact.rollback_to.is_none())
        {
            return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_ROLLBACK_METADATA_MISSING,
                "voice learn artifacts must include rollback pointer metadata".to_string(),
            )?));
        }

        let calibration = if voice_artifacts.is_empty() {
            None
        } else {
            let snapshot = voice_calibration_snapshot_from_signals(&input.signals)?;
            if snapshot.sample_count < LEARN_VOICE_CALIBRATION_MIN_SAMPLES {
                return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                    LearnCapabilityId::LearnArtifactPackageBuild,
                    reason_codes::PH1_LEARN_CALIBRATION_MIN_SAMPLES,
                    "voice calibration requires cohort sample minimum before governance handoff"
                        .to_string(),
                )?));
            }
            Some(snapshot)
        };

        let gov_artifact_proposals = if let Some(calibration_snapshot) = calibration {
            voice_artifacts
                .iter()
                .filter_map(|artifact| {
                    let family = voice_artifact_family_for_target(artifact.target)?;
                    let rollback_to = artifact.rollback_to.as_ref()?;
                    Some(LearnGovArtifactProposal::v1(
                        input.correlation_id,
                        input.turn_id,
                        input.tenant_id.clone(),
                        artifact.artifact_id.clone(),
                        family,
                        artifact.artifact_version,
                        artifact.expected_effect_bp,
                        rollback_to.clone(),
                        true,
                        true,
                        drift_adaptation_required,
                        calibration_snapshot.clone(),
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };

        if !required_families.is_empty() && gov_artifact_proposals.is_empty() {
            return Ok(LearnWiringOutcome::Refused(LearnRefuse::v1(
                LearnCapabilityId::LearnArtifactPackageBuild,
                reason_codes::PH1_LEARN_GOV_HANDOFF_BLOCKED,
                "voice learn package could not produce governance-ready proposals".to_string(),
            )?));
        }

        let bundle = LearnForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            aggregate_ok,
            package_ok,
            gov_artifact_proposals,
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
        LearnArtifactTarget::VoiceIdThresholdPack => vec![LearnTargetEngine::VoiceId],
        LearnArtifactTarget::VoiceIdConfusionPairPack => vec![LearnTargetEngine::VoiceId],
        LearnArtifactTarget::VoiceIdSpoofPolicyPack => vec![LearnTargetEngine::VoiceId],
        LearnArtifactTarget::VoiceIdProfileDeltaPack => vec![LearnTargetEngine::VoiceId],
    }
}

fn has_frozen_eval_evidence_leakage(signals: &[LearnSignal]) -> bool {
    signals.iter().any(|signal| {
        signal.evidence_ref.starts_with("frozen_eval:")
            || signal.evidence_ref.contains(":frozen_eval:")
    })
}

fn is_voice_signal(signal_type: LearnSignalType) -> bool {
    matches!(
        signal_type,
        LearnSignalType::VoiceIdFalseReject
            | LearnSignalType::VoiceIdFalseAccept
            | LearnSignalType::VoiceIdSpoofRisk
            | LearnSignalType::VoiceIdMultiSpeaker
            | LearnSignalType::VoiceIdDriftAlert
            | LearnSignalType::VoiceIdReauthFriction
            | LearnSignalType::VoiceIdConfusionPair
            | LearnSignalType::VoiceIdDrift
            | LearnSignalType::VoiceIdLowQuality
    )
}

fn required_voice_artifact_families(signals: &[LearnSignal]) -> BTreeSet<VoiceArtifactFamily> {
    let mut families = BTreeSet::new();
    for signal in signals {
        match signal.signal_type {
            LearnSignalType::VoiceIdFalseReject | LearnSignalType::VoiceIdFalseAccept => {
                families.insert(VoiceArtifactFamily::ThresholdPack);
                families.insert(VoiceArtifactFamily::ConfusionPairPack);
            }
            LearnSignalType::VoiceIdSpoofRisk => {
                families.insert(VoiceArtifactFamily::SpoofPolicyPack);
            }
            LearnSignalType::VoiceIdMultiSpeaker | LearnSignalType::VoiceIdConfusionPair => {
                families.insert(VoiceArtifactFamily::ConfusionPairPack);
            }
            LearnSignalType::VoiceIdDriftAlert
            | LearnSignalType::VoiceIdReauthFriction
            | LearnSignalType::VoiceIdDrift
            | LearnSignalType::VoiceIdLowQuality => {
                families.insert(VoiceArtifactFamily::ProfileAdaptationPack);
            }
            LearnSignalType::SttReject
            | LearnSignalType::UserCorrection
            | LearnSignalType::ClarifyLoop
            | LearnSignalType::ToolFail
            | LearnSignalType::VocabularyRepeat
            | LearnSignalType::BargeIn
            | LearnSignalType::DeliverySwitch => {}
        }
    }
    families
}

fn voice_artifact_family_for_target(target: LearnArtifactTarget) -> Option<VoiceArtifactFamily> {
    match target {
        LearnArtifactTarget::VoiceIdThresholdPack => Some(VoiceArtifactFamily::ThresholdPack),
        LearnArtifactTarget::VoiceIdConfusionPairPack => {
            Some(VoiceArtifactFamily::ConfusionPairPack)
        }
        LearnArtifactTarget::VoiceIdSpoofPolicyPack => Some(VoiceArtifactFamily::SpoofPolicyPack),
        LearnArtifactTarget::VoiceIdProfileDeltaPack => {
            Some(VoiceArtifactFamily::ProfileAdaptationPack)
        }
        LearnArtifactTarget::KnowTenantGlossaryPack
        | LearnArtifactTarget::PronLexiconPack
        | LearnArtifactTarget::CacheDecisionSkeleton
        | LearnArtifactTarget::PruneClarificationOrdering
        | LearnArtifactTarget::PaeRoutingWeights
        | LearnArtifactTarget::SearchWebExtractionHints
        | LearnArtifactTarget::ListenEnvironmentProfile => None,
    }
}

fn voice_calibration_snapshot_from_signals(
    signals: &[LearnSignal],
) -> Result<VoiceCalibrationSnapshot, ContractViolation> {
    let voice_signals = signals
        .iter()
        .filter(|signal| is_voice_signal(signal.signal_type))
        .collect::<Vec<_>>();
    if voice_signals.is_empty() {
        return VoiceCalibrationSnapshot::v1(1, 10_000, 0, 0, 10_000, 10_000, 10_000);
    }

    let sample_count = voice_signals
        .iter()
        .map(|signal| signal.occurrence_count as u32)
        .sum::<u32>();
    let false_accept_events = voice_signals
        .iter()
        .filter(|signal| {
            matches!(
                signal.signal_type,
                LearnSignalType::VoiceIdFalseAccept | LearnSignalType::VoiceIdSpoofRisk
            )
        })
        .map(|signal| signal.occurrence_count as u32)
        .sum::<u32>();
    let false_reject_events = voice_signals
        .iter()
        .filter(|signal| {
            matches!(
                signal.signal_type,
                LearnSignalType::VoiceIdFalseReject
                    | LearnSignalType::VoiceIdMultiSpeaker
                    | LearnSignalType::VoiceIdDriftAlert
                    | LearnSignalType::VoiceIdReauthFriction
                    | LearnSignalType::VoiceIdDrift
                    | LearnSignalType::VoiceIdLowQuality
            )
        })
        .map(|signal| signal.occurrence_count as u32)
        .sum::<u32>();
    let total = sample_count.max(1);
    let far_bp = ((false_accept_events * 10_000) / total).min(10_000) as u16;
    let frr_bp = ((false_reject_events * 10_000) / total).min(10_000) as u16;
    let tar_bp = 10_000u16.saturating_sub(frr_bp);
    let roc_auc_bp = 10_000u16.saturating_sub(((far_bp as u32 + frr_bp as u32) / 2) as u16);
    let ci_margin_bp = ((4_000u32 / total).max(50)).min(2_000) as u16;
    let ci_low_bp = tar_bp.saturating_sub(ci_margin_bp);
    let ci_high_bp = tar_bp.saturating_add(ci_margin_bp).min(10_000);

    VoiceCalibrationSnapshot::v1(
        sample_count,
        tar_bp,
        far_bp,
        frr_bp,
        roc_auc_bp,
        ci_low_bp,
        ci_high_bp,
    )
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
    use selene_kernel_contracts::ph1feedback::{
        FeedbackConfidenceBucket, FeedbackEventType, FeedbackPathType, FeedbackToolStatus,
    };
    use selene_kernel_contracts::ph1learn::{
        LearnArtifactCandidate, LearnArtifactPackageBuildOk, LearnScope, LearnSignalAggregateOk,
        LearnSignalType,
    };
    use selene_kernel_contracts::ph1selfheal::{
        FailureContainmentAction, FailureEvent, ProblemCardState,
    };
    use selene_kernel_contracts::MonotonicTimeNs;
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

    struct DeterministicVoiceLearnEngine;

    impl Ph1LearnEngine for DeterministicVoiceLearnEngine {
        fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
            match req {
                Ph1LearnRequest::LearnSignalAggregate(r) => {
                    let artifacts = vec![
                        LearnArtifactCandidate::v1(
                            "vid_threshold".to_string(),
                            LearnArtifactTarget::VoiceIdThresholdPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            7,
                            320,
                            "vid:threshold:prov".to_string(),
                            Some("vid_threshold.prev".to_string()),
                            true,
                        )
                        .unwrap(),
                        LearnArtifactCandidate::v1(
                            "vid_confusion".to_string(),
                            LearnArtifactTarget::VoiceIdConfusionPairPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            5,
                            280,
                            "vid:confusion:prov".to_string(),
                            Some("vid_confusion.prev".to_string()),
                            true,
                        )
                        .unwrap(),
                        LearnArtifactCandidate::v1(
                            "vid_spoof".to_string(),
                            LearnArtifactTarget::VoiceIdSpoofPolicyPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            3,
                            150,
                            "vid:spoof:prov".to_string(),
                            Some("vid_spoof.prev".to_string()),
                            true,
                        )
                        .unwrap(),
                        LearnArtifactCandidate::v1(
                            "vid_profile".to_string(),
                            LearnArtifactTarget::VoiceIdProfileDeltaPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            4,
                            220,
                            "vid:profile:prov".to_string(),
                            Some("vid_profile.prev".to_string()),
                            true,
                        )
                        .unwrap(),
                    ];
                    Ph1LearnResponse::LearnSignalAggregateOk(
                        LearnSignalAggregateOk::v1(
                            ReasonCodeId(721),
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
                            ReasonCodeId(722),
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

    struct MissingRollbackVoiceLearnEngine;

    impl Ph1LearnEngine for MissingRollbackVoiceLearnEngine {
        fn run(&self, req: &Ph1LearnRequest) -> Ph1LearnResponse {
            match req {
                Ph1LearnRequest::LearnSignalAggregate(r) => {
                    let artifact = vec![
                        LearnArtifactCandidate::v1(
                            "vid_threshold".to_string(),
                            LearnArtifactTarget::VoiceIdThresholdPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            3,
                            120,
                            "vid:threshold:prov".to_string(),
                            None,
                            true,
                        )
                        .unwrap(),
                        LearnArtifactCandidate::v1(
                            "vid_confusion".to_string(),
                            LearnArtifactTarget::VoiceIdConfusionPairPack,
                            LearnScope::Tenant,
                            Some(r.tenant_id.clone()),
                            2,
                            90,
                            "vid:confusion:prov".to_string(),
                            Some("vid_confusion.prev".to_string()),
                            true,
                        )
                        .unwrap(),
                    ];
                    Ph1LearnResponse::LearnSignalAggregateOk(
                        LearnSignalAggregateOk::v1(
                            ReasonCodeId(731),
                            artifact[0].artifact_id.clone(),
                            artifact,
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
                            ReasonCodeId(732),
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

    fn voice_signal(signal_id: &str, signal_type: LearnSignalType, occ: u16) -> LearnSignal {
        LearnSignal::v1(
            signal_id.to_string(),
            "tenant_1".to_string(),
            signal_type,
            LearnScope::Tenant,
            Some("tenant_1".to_string()),
            "voice_metric".to_string(),
            150,
            occ,
            false,
            false,
            false,
            format!("learn:evidence:{}", signal_id),
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

    #[test]
    fn at_learn_05_voice_artifact_handoff_includes_metrics_ci_and_rollback_metadata() {
        let wiring = Ph1LearnWiring::new(
            Ph1LearnWiringConfig::mvp_v1(true),
            DeterministicVoiceLearnEngine,
        )
        .unwrap();
        let input = LearnTurnInput::v1(
            CorrelationId(5302),
            TurnId(502),
            "tenant_1".to_string(),
            vec![
                voice_signal("v1", LearnSignalType::VoiceIdFalseReject, 8),
                voice_signal("v2", LearnSignalType::VoiceIdFalseAccept, 8),
                voice_signal("v3", LearnSignalType::VoiceIdSpoofRisk, 8),
                voice_signal("v4", LearnSignalType::VoiceIdDriftAlert, 8),
            ],
            vec![],
            true,
            true,
        )
        .unwrap();

        let outcome = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded");
        };
        assert_eq!(bundle.gov_artifact_proposals.len(), 4);
        for proposal in &bundle.gov_artifact_proposals {
            assert!(proposal.calibration.sample_count >= LEARN_VOICE_CALIBRATION_MIN_SAMPLES);
            assert!(proposal.calibration.ci_low_bp <= proposal.calibration.ci_high_bp);
            assert!(!proposal.rollback_to.is_empty());
        }
    }

    #[test]
    fn at_learn_06_frozen_eval_signal_leakage_is_refused() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DeterministicLearnEngine)
                .unwrap();
        let mut bad = signal("sig_frozen", LearnSignalType::SttReject);
        bad.evidence_ref = "frozen_eval:lang_en:row_11".to_string();
        let input = LearnTurnInput::v1(
            CorrelationId(5303),
            TurnId(503),
            "tenant_1".to_string(),
            vec![bad],
            vec![],
            true,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_LEARN_FROZEN_EVAL_LEAKAGE
        );
    }

    #[test]
    fn at_learn_07_drift_rate_limit_blocks_profile_adaptation() {
        let wiring = Ph1LearnWiring::new(
            Ph1LearnWiringConfig::mvp_v1(true),
            DeterministicVoiceLearnEngine,
        )
        .unwrap();
        let input = LearnTurnInput::v2(
            CorrelationId(5304),
            TurnId(504),
            "tenant_1".to_string(),
            vec![voice_signal("v5", LearnSignalType::VoiceIdDriftAlert, 25)],
            vec![],
            true,
            true,
            LEARN_DRIFT_MAX_ADAPTATIONS_24H,
            0,
            0,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_LEARN_DRIFT_RATE_LIMITED
        );
    }

    #[test]
    fn at_learn_08_missing_rollback_metadata_is_refused() {
        let wiring = Ph1LearnWiring::new(
            Ph1LearnWiringConfig::mvp_v1(true),
            MissingRollbackVoiceLearnEngine,
        )
        .unwrap();
        let input = LearnTurnInput::v1(
            CorrelationId(5305),
            TurnId(505),
            "tenant_1".to_string(),
            vec![voice_signal("v6", LearnSignalType::VoiceIdFalseReject, 24)],
            vec![],
            true,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_LEARN_ROLLBACK_METADATA_MISSING
        );
    }

    fn mapped_failure_event(correlation_id: CorrelationId, turn_id: TurnId) -> FailureEvent {
        FailureEvent::v1(
            "failure_map_1".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            correlation_id,
            turn_id,
            FeedbackEventType::ToolFail,
            ReasonCodeId(0x4C45_9001),
            FeedbackPathType::Defect,
            "evidence:learn_map".to_string(),
            "idem:failure:learn_map".to_string(),
            FeedbackConfidenceBucket::Low,
            FeedbackToolStatus::Fail,
            90,
            1,
            vec!["field_a".to_string()],
            "fingerprint_1".to_string(),
            FailureContainmentAction::FailClosedRefuse,
            false,
            None,
            None,
        )
        .unwrap()
    }

    #[test]
    fn at_learn_09_mapper_builds_problem_and_fix_cards_deterministically() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DeterministicLearnEngine)
                .unwrap();
        let input = LearnTurnInput::v1(
            CorrelationId(5309),
            TurnId(509),
            "tenant_1".to_string(),
            vec![
                signal("sig_91", LearnSignalType::SttReject),
                signal("sig_92", LearnSignalType::UserCorrection),
            ],
            vec![],
            true,
            true,
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded learn bundle");
        };
        let failure_event = mapped_failure_event(CorrelationId(5309), TurnId(509));
        let escalation = ProblemCardEscalationState::open();

        let problem_a = map_learn_bundle_to_problem_card(
            &failure_event,
            &input,
            &bundle,
            "PH1.LEARN".to_string(),
            MonotonicTimeNs(100),
            MonotonicTimeNs(120),
            escalation.clone(),
        )
        .unwrap();
        let problem_b = map_learn_bundle_to_problem_card(
            &failure_event,
            &input,
            &bundle,
            "PH1.LEARN".to_string(),
            MonotonicTimeNs(100),
            MonotonicTimeNs(120),
            escalation,
        )
        .unwrap();
        assert_eq!(problem_a.problem_id, problem_b.problem_id);
        assert_eq!(problem_a.state, ProblemCardState::Open);
        assert_eq!(problem_a.latest_failure_id, failure_event.failure_id);

        let fix_a = map_learn_bundle_to_fix_card(&problem_a, &bundle).unwrap();
        let fix_b = map_learn_bundle_to_fix_card(&problem_a, &bundle).unwrap();
        assert_eq!(fix_a.fix_id, fix_b.fix_id);
        assert_eq!(fix_a.problem_id, problem_a.problem_id);
    }

    #[test]
    fn at_learn_10_mapper_fails_closed_on_tenant_mismatch() {
        let wiring =
            Ph1LearnWiring::new(Ph1LearnWiringConfig::mvp_v1(true), DeterministicLearnEngine)
                .unwrap();
        let input = LearnTurnInput::v1(
            CorrelationId(5310),
            TurnId(510),
            "tenant_1".to_string(),
            vec![signal("sig_101", LearnSignalType::SttReject)],
            vec![],
            true,
            true,
        )
        .unwrap();
        let out = wiring.run_turn(&input).unwrap();
        let LearnWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded learn bundle");
        };
        let mut failure_event = mapped_failure_event(CorrelationId(5310), TurnId(510));
        failure_event.tenant_id = "tenant_2".to_string();

        let err = map_learn_bundle_to_problem_card(
            &failure_event,
            &input,
            &bundle,
            "PH1.LEARN".to_string(),
            MonotonicTimeNs(100),
            MonotonicTimeNs(120),
            ProblemCardEscalationState::open(),
        )
        .expect_err("tenant mismatch must fail closed");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(
                    field,
                    "map_learn_bundle_to_problem_card.turn_input.tenant_id"
                );
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }
}
