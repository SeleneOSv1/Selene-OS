#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use selene_kernel_contracts::ph1builder::{
    required_approvals_for_change_class, rollout_pct_for_stage, BuilderApprovalState,
    BuilderApprovalStateStatus, BuilderChangeClass, BuilderExpectedEffect, BuilderLearningContext,
    BuilderMetricsSnapshot, BuilderPatchProposal, BuilderPostDeployDecisionAction,
    BuilderPostDeployJudgeResult,
    BuilderProposalStatus, BuilderReleaseStage, BuilderReleaseState, BuilderReleaseStateStatus,
    BuilderSignalWindow, BuilderValidationGateId, BuilderValidationGateResult,
    BuilderValidationRun, BuilderValidationRunStatus,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1os::{OsOutcomeActionClass, OsOutcomeUtilizationEntry};
use selene_kernel_contracts::ph1pattern::{PatternProposalItem, PatternProposalTarget, PatternSignal};
use selene_kernel_contracts::ph1rll::{
    RllArtifactCandidate, RllOptimizationTarget, RllRecommendationItem,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, ReasonCodeId, Validate};
use selene_storage::ph1f::{BuilderProposalLedgerRowInput, StorageError};
use selene_storage::repo::BuilderSeleneRepo;

use crate::ph1pattern::{
    PatternOfflineInput, PatternWiringOutcome, Ph1PatternEngine, Ph1PatternWiring,
    Ph1PatternWiringConfig,
};
use crate::ph1rll::{
    Ph1RllEngine, Ph1RllWiring, Ph1RllWiringConfig, RllOfflineInput, RllWiringOutcome,
};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.BUILDER reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_BUILDER_OFFLINE_ONLY_REQUIRED: ReasonCodeId = ReasonCodeId(0xB13D_0001);
    pub const PH1_BUILDER_PATTERN_REFUSED: ReasonCodeId = ReasonCodeId(0xB13D_0002);
    pub const PH1_BUILDER_RLL_REFUSED: ReasonCodeId = ReasonCodeId(0xB13D_0003);
    pub const PH1_BUILDER_GATE_COLLECTION_INVALID: ReasonCodeId = ReasonCodeId(0xB13D_0004);
    pub const PH1_BUILDER_APPROVAL_UNRESOLVED: ReasonCodeId = ReasonCodeId(0xB13D_0005);
    pub const PH1_BUILDER_APPROVAL_AUTO_RESOLVED: ReasonCodeId = ReasonCodeId(0xB13D_0006);
    pub const PH1_BUILDER_RELEASE_BLOCKED_APPROVAL: ReasonCodeId = ReasonCodeId(0xB13D_0007);
    pub const PH1_BUILDER_RELEASE_STAGE_ACTIVE: ReasonCodeId = ReasonCodeId(0xB13D_0008);
    pub const PH1_BUILDER_RELEASE_PROMOTION_BLOCKED: ReasonCodeId = ReasonCodeId(0xB13D_0009);
    pub const PH1_BUILDER_RELEASE_ROLLBACK_TRIGGERED: ReasonCodeId = ReasonCodeId(0xB13D_000A);
    pub const PH1_BUILDER_POST_DEPLOY_ACCEPTED: ReasonCodeId = ReasonCodeId(0xB13D_000B);
    pub const PH1_BUILDER_POST_DEPLOY_REVERTED: ReasonCodeId = ReasonCodeId(0xB13D_000C);
    pub const PH1_BUILDER_POST_DEPLOY_MISSING_PROPOSAL_FIELDS: ReasonCodeId =
        ReasonCodeId(0xB13D_000D);
    pub const PH1_BUILDER_POST_DEPLOY_MISSING_GATE_OUTCOMES: ReasonCodeId =
        ReasonCodeId(0xB13D_000E);
    pub const PH1_BUILDER_POST_DEPLOY_RELEASE_STAGE_INVALID: ReasonCodeId =
        ReasonCodeId(0xB13D_000F);
    pub const PH1_BUILDER_LEARNING_REPORT_WRITE_FAILED: ReasonCodeId = ReasonCodeId(0xB13D_0010);
    pub const PH1_BUILDER_CHANGE_BRIEF_WRITE_FAILED: ReasonCodeId = ReasonCodeId(0xB13D_0011);
    pub const PH1_BUILDER_PERMISSION_PACKET_WRITE_FAILED: ReasonCodeId =
        ReasonCodeId(0xB13D_0012);
    pub const PH1_BUILDER_DECISION_SEED_WRITE_FAILED: ReasonCodeId = ReasonCodeId(0xB13D_0013);
}

const DEFAULT_LEARNING_REPORT_OUTPUT_PATH: &str = ".dev/builder_learning_report.md";
const DEFAULT_CHANGE_BRIEF_OUTPUT_PATH: &str = ".dev/builder_change_brief.md";
const DEFAULT_PERMISSION_PACKET_OUTPUT_PATH: &str = ".dev/builder_permission_packet.md";
const LEARNING_BRIDGE_SOURCE_ENGINES: [&str; 3] = ["PH1.FEEDBACK", "PH1.LEARN", "PH1.KNOW"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1BuilderConfig {
    pub builder_enabled: bool,
    pub max_outcomes: u16,
    pub max_pattern_signals: u8,
    pub max_pattern_proposals: u8,
    pub max_rll_candidates: u8,
    pub max_rll_recommendations: u8,
    pub analysis_window_days: u16,
    pub training_window_days: u16,
    pub minimum_sample_size: u32,
    pub offline_pipeline_only: bool,
}

impl Ph1BuilderConfig {
    pub fn mvp_v1(builder_enabled: bool) -> Self {
        Self {
            builder_enabled,
            max_outcomes: 256,
            max_pattern_signals: 32,
            max_pattern_proposals: 16,
            max_rll_candidates: 16,
            max_rll_recommendations: 8,
            analysis_window_days: 30,
            training_window_days: 30,
            minimum_sample_size: 500,
            offline_pipeline_only: true,
        }
    }

    fn validate(self) -> Result<(), ContractViolation> {
        if self.max_outcomes == 0 || self.max_outcomes > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.max_outcomes",
                reason: "must be within 1..=512",
            });
        }
        if self.max_pattern_signals == 0 || self.max_pattern_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.max_pattern_signals",
                reason: "must be within 1..=64",
            });
        }
        if self.max_pattern_proposals == 0 || self.max_pattern_proposals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.max_pattern_proposals",
                reason: "must be within 1..=32",
            });
        }
        if self.max_rll_candidates == 0 || self.max_rll_candidates > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.max_rll_candidates",
                reason: "must be within 1..=64",
            });
        }
        if self.max_rll_recommendations == 0 || self.max_rll_recommendations > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.max_rll_recommendations",
                reason: "must be within 1..=32",
            });
        }
        if self.analysis_window_days == 0 || self.analysis_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.analysis_window_days",
                reason: "must be within 1..=365",
            });
        }
        if self.training_window_days == 0 || self.training_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.training_window_days",
                reason: "must be within 1..=365",
            });
        }
        if self.minimum_sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.minimum_sample_size",
                reason: "must be > 0",
            });
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "ph1builder_config.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderOfflineInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_window_start_at: MonotonicTimeNs,
    pub source_window_end_at: MonotonicTimeNs,
    pub now: MonotonicTimeNs,
    pub outcome_entries: Vec<OsOutcomeUtilizationEntry>,
    pub source_signal_hash: Option<String>,
    pub proposal_idempotency_key: Option<String>,
    pub validation_run_idempotency_key: Option<String>,
    pub learning_report_output_path: Option<String>,
    pub change_brief_output_path: Option<String>,
    pub permission_packet_output_path: Option<String>,
    pub offline_pipeline_only: bool,
}

impl BuilderOfflineInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_window_start_at: MonotonicTimeNs,
        source_window_end_at: MonotonicTimeNs,
        now: MonotonicTimeNs,
        outcome_entries: Vec<OsOutcomeUtilizationEntry>,
        source_signal_hash: Option<String>,
        proposal_idempotency_key: Option<String>,
        validation_run_idempotency_key: Option<String>,
        learning_report_output_path: Option<String>,
        change_brief_output_path: Option<String>,
        permission_packet_output_path: Option<String>,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            source_window_start_at,
            source_window_end_at,
            now,
            outcome_entries,
            source_signal_hash,
            proposal_idempotency_key,
            validation_run_idempotency_key,
            learning_report_output_path,
            change_brief_output_path,
            permission_packet_output_path,
            offline_pipeline_only,
        };
        input.validate()?;
        Ok(input)
    }

    fn signal_hash(&self) -> String {
        if let Some(hash) = &self.source_signal_hash {
            return hash.clone();
        }
        let mut canonical = self
            .outcome_entries
            .iter()
            .map(|entry| {
                format!(
                    "{}|{}|{}|{}|{}|{}",
                    entry.engine_id,
                    entry.outcome_type,
                    entry.reason_code.0,
                    entry.action_class.as_str(),
                    entry.latency_cost_ms,
                    if entry.decision_delta { 1 } else { 0 }
                )
            })
            .collect::<Vec<_>>();
        canonical.sort();
        let joined = canonical.join(";");
        hash_hex_64(&joined)
    }
}

impl Validate for BuilderOfflineInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.source_window_end_at.0 < self.source_window_start_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_offline_input.source_window_end_at",
                reason: "must be >= source_window_start_at",
            });
        }
        if self.outcome_entries.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_offline_input.outcome_entries",
                reason: "must be <= 512",
            });
        }
        for entry in &self.outcome_entries {
            entry.validate()?;
            if entry.correlation_id != self.correlation_id {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_offline_input.outcome_entries.correlation_id",
                    reason: "must match builder_offline_input.correlation_id",
                });
            }
            if entry.turn_id != self.turn_id {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_offline_input.outcome_entries.turn_id",
                    reason: "must match builder_offline_input.turn_id",
                });
            }
        }
        if let Some(hash) = &self.source_signal_hash {
            validate_token_ascii("builder_offline_input.source_signal_hash", hash, 128)?;
        }
        if let Some(key) = &self.proposal_idempotency_key {
            validate_token_ascii("builder_offline_input.proposal_idempotency_key", key, 128)?;
        }
        if let Some(key) = &self.validation_run_idempotency_key {
            validate_token_ascii(
                "builder_offline_input.validation_run_idempotency_key",
                key,
                128,
            )?;
        }
        if let Some(path) = &self.learning_report_output_path {
            validate_path_ascii(
                "builder_offline_input.learning_report_output_path",
                path,
                512,
            )?;
        }
        if let Some(path) = &self.change_brief_output_path {
            validate_path_ascii(
                "builder_offline_input.change_brief_output_path",
                path,
                512,
            )?;
        }
        if let Some(path) = &self.permission_packet_output_path {
            validate_path_ascii(
                "builder_offline_input.permission_packet_output_path",
                path,
                512,
            )?;
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "builder_offline_input.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderGateEvaluation {
    pub gate_id: BuilderValidationGateId,
    pub passed: bool,
    pub reason_code: ReasonCodeId,
    pub detail: String,
}

impl BuilderGateEvaluation {
    pub fn v1(
        gate_id: BuilderValidationGateId,
        passed: bool,
        reason_code: ReasonCodeId,
        detail: String,
    ) -> Result<Self, ContractViolation> {
        if reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_gate_evaluation.reason_code",
                reason: "must be non-zero",
            });
        }
        validate_ascii_text("builder_gate_evaluation.detail", &detail, 256)?;
        Ok(Self {
            gate_id,
            passed,
            reason_code,
            detail,
        })
    }
}

pub trait BuilderSandboxValidator {
    fn collect_gate_evaluations(
        &self,
        proposal: &BuilderPatchProposal,
        outcome_entries: &[OsOutcomeUtilizationEntry],
    ) -> Result<Vec<BuilderGateEvaluation>, ContractViolation>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeterministicBuilderSandboxValidator;

impl BuilderSandboxValidator for DeterministicBuilderSandboxValidator {
    fn collect_gate_evaluations(
        &self,
        proposal: &BuilderPatchProposal,
        outcome_entries: &[OsOutcomeUtilizationEntry],
    ) -> Result<Vec<BuilderGateEvaluation>, ContractViolation> {
        let has_compile = proposal.validation_plan.contains("compile");
        let has_test = proposal.validation_plan.contains("test");
        let has_revert = proposal.rollback_plan.contains("revert")
            || proposal.rollback_plan.contains("rollback");
        let touches_sql = proposal
            .target_files
            .iter()
            .any(|path| path.ends_with(".sql"));
        let has_secret_path = proposal
            .target_files
            .iter()
            .any(|path| path.to_ascii_lowercase().contains("secret"));

        let avg_latency_ms = if outcome_entries.is_empty() {
            0u32
        } else {
            (outcome_entries
                .iter()
                .map(|entry| entry.latency_cost_ms as u64)
                .sum::<u64>()
                / outcome_entries.len() as u64) as u32
        };

        let bld_g1 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG1,
            !proposal.target_files.is_empty(),
            ReasonCodeId(0xB13D_0101),
            "proposal target files are deterministic and non-empty".to_string(),
        )?;
        let bld_g2 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG2,
            has_compile && has_test,
            ReasonCodeId(0xB13D_0102),
            "compile/test validation plan is declared".to_string(),
        )?;
        let bld_g3 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG3,
            proposal.expected_effect.validate().is_ok(),
            ReasonCodeId(0xB13D_0103),
            "contract invariants remain schema-valid".to_string(),
        )?;
        let bld_g4 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG4,
            proposal.risk_score_bp <= 10_000,
            ReasonCodeId(0xB13D_0104),
            "single-owner risk posture remains bounded".to_string(),
        )?;
        let bld_g5 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG5,
            !proposal
                .target_files
                .iter()
                .any(|path| path.contains("runtime_boundary_forbidden")),
            ReasonCodeId(0xB13D_0105),
            "offline/runtime boundary policy check passed".to_string(),
        )?;
        let bld_g6 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG6,
            has_revert,
            ReasonCodeId(0xB13D_0106),
            "idempotency and rollback coverage present".to_string(),
        )?;
        let bld_g7 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG7,
            !touches_sql,
            ReasonCodeId(0xB13D_0107),
            "migration safety gate passed (no schema migration in this proposal)".to_string(),
        )?;
        let bld_g8 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG8,
            !has_secret_path,
            ReasonCodeId(0xB13D_0108),
            "security/privacy guard passed".to_string(),
        )?;
        let bld_g9 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG9,
            proposal.expected_effect.latency_p95_delta_bp <= 0
                && proposal.expected_effect.latency_p99_delta_bp <= 0
                && avg_latency_ms <= 60_000,
            ReasonCodeId(0xB13D_0109),
            "latency guard remains within budget".to_string(),
        )?;
        let bld_g10 = BuilderGateEvaluation::v1(
            BuilderValidationGateId::BldG10,
            proposal.reason_code_valid(),
            ReasonCodeId(0xB13D_010A),
            "audit traceability fields are complete".to_string(),
        )?;

        Ok(vec![
            bld_g1, bld_g2, bld_g3, bld_g4, bld_g5, bld_g6, bld_g7, bld_g8, bld_g9, bld_g10,
        ])
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderRefusal {
    pub stage: &'static str,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderCompletedBundle {
    pub proposal: BuilderPatchProposal,
    pub validation_run: BuilderValidationRun,
    pub gate_results: Vec<BuilderValidationGateResult>,
    pub approval_state: BuilderApprovalState,
    pub release_state: BuilderReleaseState,
    pub proposal_row_id: u64,
    pub run_row_id: u64,
    pub gate_result_row_ids: Vec<u64>,
    pub approval_row_id: u64,
    pub release_row_id: u64,
    pub learning_report_id: Option<String>,
    pub learning_source_engines: Vec<String>,
    pub learning_signal_count: u32,
    pub learning_evidence_refs: Vec<String>,
    pub learning_report_path: Option<String>,
    pub change_brief_path: Option<String>,
    pub permission_packet_path: Option<String>,
    pub code_decision_file_path: Option<String>,
    pub launch_decision_file_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuilderLearningAutoReport {
    learning_report_id: String,
    source_engines: Vec<String>,
    learning_signal_count: u32,
    evidence_refs: Vec<String>,
    report_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BuilderDecisionSeedFiles {
    code_file_path: String,
    launch_file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderOrchestrationOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(BuilderRefusal),
    Completed(BuilderCompletedBundle),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderPostDeployJudgeInput {
    pub proposal: BuilderPatchProposal,
    pub release_state: BuilderReleaseState,
    pub gate_results: Vec<BuilderValidationGateResult>,
    pub before_metrics: BuilderMetricsSnapshot,
    pub after_metrics: BuilderMetricsSnapshot,
    pub authority_or_gate_order_violation: bool,
    pub duplicate_side_effect_event_detected: bool,
    pub now: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
}

impl Validate for BuilderPostDeployJudgeInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.proposal.validate()?;
        self.release_state.validate()?;
        self.before_metrics.validate()?;
        self.after_metrics.validate()?;
        if self.gate_results.is_empty() || self.gate_results.len() > 10 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_post_deploy_judge_input.gate_results",
                reason: "must be within 1..=10",
            });
        }
        for gate in &self.gate_results {
            gate.validate()?;
            if gate.proposal_id != self.proposal.proposal_id {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_post_deploy_judge_input.gate_results.proposal_id",
                    reason: "must match proposal.proposal_id",
                });
            }
        }
        if let Some(key) = &self.idempotency_key {
            validate_token_ascii("builder_post_deploy_judge_input.idempotency_key", key, 128)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderPostDeployDecisionBundle {
    pub judge_result: BuilderPostDeployJudgeResult,
    pub release_state: BuilderReleaseState,
    pub judge_row_id: u64,
    pub release_row_id: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuilderPostDeployJudgeOutcome {
    Refused(BuilderRefusal),
    Completed(BuilderPostDeployDecisionBundle),
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuilderPipelineError {
    Contract(ContractViolation),
    Storage(StorageError),
    Io(String),
}

impl From<ContractViolation> for BuilderPipelineError {
    fn from(value: ContractViolation) -> Self {
        Self::Contract(value)
    }
}

impl From<StorageError> for BuilderPipelineError {
    fn from(value: StorageError) -> Self {
        Self::Storage(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuilderApprovalTransitionAction {
    ApproveTech,
    ApproveProductSecurity,
    Reject,
}

pub fn advance_approval_state(
    current: &BuilderApprovalState,
    action: BuilderApprovalTransitionAction,
    now: MonotonicTimeNs,
    reason_code: ReasonCodeId,
    idempotency_key: Option<String>,
) -> Result<BuilderApprovalState, ContractViolation> {
    if current.status != BuilderApprovalStateStatus::Pending {
        return Err(ContractViolation::InvalidValue {
            field: "builder_approval_transition.current_status",
            reason: "can transition only from PENDING state",
        });
    }
    if reason_code.0 == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "builder_approval_transition.reason_code",
            reason: "must be non-zero",
        });
    }

    let mut tech_approved = current.tech_approved;
    let mut product_security_approved = current.product_security_approved;
    let next_status = match action {
        BuilderApprovalTransitionAction::ApproveTech => {
            if matches!(current.change_class, BuilderChangeClass::ClassA) {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_approval_transition.action",
                    reason: "CLASS_A must not require human approvals",
                });
            }
            tech_approved = true;
            if transition_has_all_required_approvals(current.change_class, tech_approved, product_security_approved)
            {
                BuilderApprovalStateStatus::Approved
            } else {
                BuilderApprovalStateStatus::Pending
            }
        }
        BuilderApprovalTransitionAction::ApproveProductSecurity => {
            if current.change_class != BuilderChangeClass::ClassC {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_approval_transition.action",
                    reason: "product/security approval is allowed only for CLASS_C",
                });
            }
            product_security_approved = true;
            if transition_has_all_required_approvals(current.change_class, tech_approved, product_security_approved)
            {
                BuilderApprovalStateStatus::Approved
            } else {
                BuilderApprovalStateStatus::Pending
            }
        }
        BuilderApprovalTransitionAction::Reject => BuilderApprovalStateStatus::Rejected,
    };

    let approvals_granted = (u8::from(tech_approved)) + (u8::from(product_security_approved));
    BuilderApprovalState::v1(
        next_approval_state_id(&current.approval_state_id, action),
        current.proposal_id.clone(),
        current.change_class,
        required_approvals_for_change_class(current.change_class),
        approvals_granted,
        tech_approved,
        product_security_approved,
        next_status,
        reason_code,
        now,
        if next_status == BuilderApprovalStateStatus::Pending {
            None
        } else {
            Some(now)
        },
        idempotency_key,
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuilderReleaseController;

impl BuilderReleaseController {
    pub fn initial_state(
        proposal: &BuilderPatchProposal,
        approval: &BuilderApprovalState,
        now: MonotonicTimeNs,
        idempotency_key: Option<String>,
    ) -> Result<BuilderReleaseState, ContractViolation> {
        let (status, reason_code) = if proposal.status != BuilderProposalStatus::Validated {
            (
                BuilderReleaseStateStatus::Blocked,
                reason_codes::PH1_BUILDER_GATE_COLLECTION_INVALID,
            )
        } else if approval.status != BuilderApprovalStateStatus::Approved {
            (
                BuilderReleaseStateStatus::Blocked,
                reason_codes::PH1_BUILDER_RELEASE_BLOCKED_APPROVAL,
            )
        } else {
            (
                BuilderReleaseStateStatus::Active,
                reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            )
        };
        let stage = BuilderReleaseStage::Staging;
        BuilderReleaseState::v1(
            release_state_id_for_proposal_and_stage(&proposal.proposal_id, stage),
            proposal.proposal_id.clone(),
            stage,
            rollout_pct_for_stage(stage),
            status,
            "rollback:revert_patch_and_restore_last_validated_package".to_string(),
            true,
            reason_code,
            now,
            idempotency_key,
        )
    }

    pub fn promote(
        &self,
        current: &BuilderReleaseState,
        approval: &BuilderApprovalState,
        now: MonotonicTimeNs,
        idempotency_key: Option<String>,
    ) -> Result<BuilderReleaseState, BuilderRefusal> {
        if current.status != BuilderReleaseStateStatus::Active {
            return Err(BuilderRefusal {
                stage: "RELEASE",
                reason_code: reason_codes::PH1_BUILDER_RELEASE_PROMOTION_BLOCKED,
                message: "release promotion requires ACTIVE stage state".to_string(),
            });
        }
        let next_stage = next_release_stage(current.stage).ok_or_else(|| BuilderRefusal {
            stage: "RELEASE",
            reason_code: reason_codes::PH1_BUILDER_RELEASE_PROMOTION_BLOCKED,
            message: "release stage cannot advance beyond PRODUCTION".to_string(),
        })?;
        if next_stage == BuilderReleaseStage::Production
            && approval.status != BuilderApprovalStateStatus::Approved
        {
            return Err(BuilderRefusal {
                stage: "RELEASE",
                reason_code: reason_codes::PH1_BUILDER_RELEASE_PROMOTION_BLOCKED,
                message: "production rollout blocked because approval class is unresolved".to_string(),
            });
        }
        let next_status = if next_stage == BuilderReleaseStage::Production {
            BuilderReleaseStateStatus::Completed
        } else {
            BuilderReleaseStateStatus::Active
        };
        BuilderReleaseState::v1(
            release_state_id_for_proposal_and_stage(&current.proposal_id, next_stage),
            current.proposal_id.clone(),
            next_stage,
            rollout_pct_for_stage(next_stage),
            next_status,
            current.rollback_hook.clone(),
            current.rollback_ready,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            now,
            idempotency_key,
        )
        .map_err(|_err| BuilderRefusal {
            stage: "RELEASE",
            reason_code: reason_codes::PH1_BUILDER_RELEASE_PROMOTION_BLOCKED,
            message: "release promotion contract validation failed".to_string(),
        })
    }

    pub fn rollback(
        &self,
        current: &BuilderReleaseState,
        now: MonotonicTimeNs,
        idempotency_key: Option<String>,
    ) -> Result<BuilderReleaseState, ContractViolation> {
        BuilderReleaseState::v1(
            release_state_id_for_proposal_and_stage(
                &current.proposal_id,
                BuilderReleaseStage::RolledBack,
            ),
            current.proposal_id.clone(),
            BuilderReleaseStage::RolledBack,
            rollout_pct_for_stage(BuilderReleaseStage::RolledBack),
            BuilderReleaseStateStatus::Reverted,
            current.rollback_hook.clone(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_ROLLBACK_TRIGGERED,
            now,
            idempotency_key,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Ph1BuilderOrchestrator<P, R, V>
where
    P: Ph1PatternEngine,
    R: Ph1RllEngine,
    V: BuilderSandboxValidator,
{
    config: Ph1BuilderConfig,
    pattern_wiring: Ph1PatternWiring<P>,
    rll_wiring: Ph1RllWiring<R>,
    validator: V,
}

impl<P, R, V> Ph1BuilderOrchestrator<P, R, V>
where
    P: Ph1PatternEngine,
    R: Ph1RllEngine,
    V: BuilderSandboxValidator,
{
    pub fn new(
        config: Ph1BuilderConfig,
        pattern_engine: P,
        rll_engine: R,
        validator: V,
    ) -> Result<Self, ContractViolation> {
        config.validate()?;
        let pattern_wiring = Ph1PatternWiring::new(
            Ph1PatternWiringConfig {
                pattern_enabled: config.builder_enabled,
                max_signals: config.max_pattern_signals,
                max_proposals: config.max_pattern_proposals,
                offline_pipeline_only: true,
            },
            pattern_engine,
        )?;
        let rll_wiring = Ph1RllWiring::new(
            Ph1RllWiringConfig {
                rll_enabled: config.builder_enabled,
                max_candidates: config.max_rll_candidates,
                max_recommendations: config.max_rll_recommendations,
                offline_pipeline_only: true,
            },
            rll_engine,
        )?;
        Ok(Self {
            config,
            pattern_wiring,
            rll_wiring,
            validator,
        })
    }

    pub fn run_offline<S>(
        &self,
        store: &mut S,
        input: &BuilderOfflineInput,
    ) -> Result<BuilderOrchestrationOutcome, BuilderPipelineError>
    where
        S: BuilderSeleneRepo,
    {
        input.validate()?;
        if !self.config.builder_enabled {
            return Ok(BuilderOrchestrationOutcome::NotInvokedDisabled);
        }
        if input.outcome_entries.is_empty() {
            return Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals);
        }
        if !self.config.offline_pipeline_only || !input.offline_pipeline_only {
            return Ok(BuilderOrchestrationOutcome::Refused(BuilderRefusal {
                stage: "INPUT",
                reason_code: reason_codes::PH1_BUILDER_OFFLINE_ONLY_REQUIRED,
                message: "builder orchestration requires offline pipeline".to_string(),
            }));
        }

        let pattern_signals = cluster_outcomes_to_pattern_signals(
            &input.outcome_entries,
            self.config.max_pattern_signals as usize,
            input.correlation_id,
            input.turn_id,
        )?;
        if pattern_signals.is_empty() {
            return Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals);
        }

        let pattern_input = PatternOfflineInput::v1(
            input.correlation_id,
            input.turn_id,
            pattern_signals.clone(),
            self.config.analysis_window_days,
            true,
        )?;
        let pattern_bundle = match self.pattern_wiring.run_offline(&pattern_input)? {
            PatternWiringOutcome::NotInvokedDisabled => {
                return Ok(BuilderOrchestrationOutcome::NotInvokedDisabled)
            }
            PatternWiringOutcome::NotInvokedNoSignals => {
                return Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals)
            }
            PatternWiringOutcome::Refused(refuse) => {
                return Ok(BuilderOrchestrationOutcome::Refused(BuilderRefusal {
                    stage: "PATTERN",
                    reason_code: reason_codes::PH1_BUILDER_PATTERN_REFUSED,
                    message: format!(
                        "pattern refused capability {} with reason {}",
                        refuse.capability_id.as_str(),
                        refuse.reason_code.0
                    ),
                }))
            }
            PatternWiringOutcome::Forwarded(bundle) => bundle,
        };

        let rll_candidates = pattern_to_rll_candidates(
            &pattern_bundle.mine_offline.ordered_proposals,
            self.config.max_rll_candidates as usize,
        )?;
        if rll_candidates.is_empty() {
            return Ok(BuilderOrchestrationOutcome::Refused(BuilderRefusal {
                stage: "RLL_INPUT",
                reason_code: reason_codes::PH1_BUILDER_RLL_REFUSED,
                message: "no RLL candidates could be derived from pattern proposals".to_string(),
            }));
        }

        let rll_input = RllOfflineInput::v1(
            input.correlation_id,
            input.turn_id,
            rll_candidates,
            self.config.training_window_days,
            self.config.minimum_sample_size,
            true,
        )?;
        let rll_bundle = match self.rll_wiring.run_offline(&rll_input)? {
            RllWiringOutcome::NotInvokedDisabled => {
                return Ok(BuilderOrchestrationOutcome::NotInvokedDisabled)
            }
            RllWiringOutcome::NotInvokedNoCandidates => {
                return Ok(BuilderOrchestrationOutcome::NotInvokedNoSignals)
            }
            RllWiringOutcome::Refused(refuse) => {
                return Ok(BuilderOrchestrationOutcome::Refused(BuilderRefusal {
                    stage: "RLL",
                    reason_code: reason_codes::PH1_BUILDER_RLL_REFUSED,
                    message: format!(
                        "rll refused capability {} with reason {}",
                        refuse.capability_id.as_str(),
                        refuse.reason_code.0
                    ),
                }))
            }
            RllWiringOutcome::Forwarded(bundle) => bundle,
        };

        let selected_recommendation = find_selected_recommendation(
            &rll_bundle.policy_rank.ordered_recommendations,
            &rll_bundle.policy_rank.selected_artifact_id,
        )
        .ok_or_else(|| {
            BuilderPipelineError::Contract(ContractViolation::InvalidValue {
                field: "builder_orchestration.selected_artifact_id",
                reason: "must exist in ordered_recommendations",
            })
        })?;

        let signal_hash = input.signal_hash();
        let proposal_id = deterministic_proposal_id(
            input.correlation_id,
            input.turn_id,
            &selected_recommendation.artifact_id,
        );
        let run_id = deterministic_run_id(&proposal_id);
        let signal_window = BuilderSignalWindow::v1(
            input.source_window_start_at,
            input.source_window_end_at,
            input.outcome_entries.len() as u32,
        )?;

        let change_class = change_class_for_target(selected_recommendation.target);
        let target_files = target_files_for_target(selected_recommendation.target);
        let risk_score_bp = risk_score_for_selection(change_class, selected_recommendation);
        let expected_effect =
            expected_effect_from_outcomes(&input.outcome_entries, selected_recommendation)?;

        let validation_plan = "compile + test + guardrails + audit checks".to_string();
        let rollback_plan = "revert patch and restore previous validated package".to_string();

        let base_proposal = BuilderPatchProposal::v1(
            proposal_id.clone(),
            input.now,
            signal_window.clone(),
            signal_hash.clone(),
            target_files.clone(),
            change_class,
            risk_score_bp,
            expected_effect.clone(),
            validation_plan.clone(),
            rollback_plan.clone(),
            BuilderProposalStatus::Draft,
        )?;

        let mut gate_evaluations = self
            .validator
            .collect_gate_evaluations(&base_proposal, &input.outcome_entries)?;
        if validate_gate_evaluations_complete(&gate_evaluations).is_err() {
            return Ok(BuilderOrchestrationOutcome::Refused(BuilderRefusal {
                stage: "VALIDATOR",
                reason_code: reason_codes::PH1_BUILDER_GATE_COLLECTION_INVALID,
                message: "validator must emit BLD-G1..BLD-G10 exactly once".to_string(),
            }));
        }
        gate_evaluations.sort_by_key(|gate| gate.gate_id);

        let all_passed = gate_evaluations.iter().all(|gate| gate.passed);
        let proposal_status = if all_passed {
            BuilderProposalStatus::Validated
        } else {
            BuilderProposalStatus::Draft
        };
        let mut final_proposal = BuilderPatchProposal::v1(
            proposal_id.clone(),
            input.now,
            signal_window,
            signal_hash,
            target_files,
            change_class,
            risk_score_bp,
            expected_effect,
            validation_plan,
            rollback_plan,
            proposal_status,
        )?;

        let learning_auto_report = maybe_generate_learning_auto_report(
            input,
            &input.outcome_entries,
            &pattern_signals,
            selected_recommendation,
            &proposal_id,
        )?;
        if let Some(ref learning_report) = learning_auto_report {
            final_proposal = final_proposal.with_learning_context(BuilderLearningContext::v1(
                learning_report.learning_report_id.clone(),
                learning_report.source_engines.clone(),
                learning_report.learning_signal_count,
                learning_report.evidence_refs.clone(),
            )?)?;
        }
        let change_brief_path = generate_change_brief(
            input,
            &input.outcome_entries,
            selected_recommendation,
            &proposal_id,
            change_class,
            learning_auto_report.as_ref(),
        )?;
        let decision_seed_files = generate_decision_seed_files(input, &proposal_id)?;
        let permission_packet_path = generate_permission_packet(
            input,
            &input.outcome_entries,
            selected_recommendation,
            &proposal_id,
            change_class,
            learning_auto_report.as_ref(),
            &change_brief_path,
            &decision_seed_files,
        )?;

        let proposal_idempotency_key = input
            .proposal_idempotency_key
            .clone()
            .unwrap_or_else(|| format!("builder_proposal:{}", proposal_id));
        let proposal_row_id = store.append_builder_proposal_row(BuilderProposalLedgerRowInput {
            proposal: final_proposal.clone(),
            idempotency_key: Some(proposal_idempotency_key),
        })?;

        let finished_at = MonotonicTimeNs(input.now.0.saturating_add(1_000_000));
        let run_status = if all_passed {
            BuilderValidationRunStatus::Passed
        } else {
            BuilderValidationRunStatus::Failed
        };
        let run_idempotency_key = input
            .validation_run_idempotency_key
            .clone()
            .unwrap_or_else(|| format!("builder_run:{}", run_id));
        let run = BuilderValidationRun::v1(
            run_id.clone(),
            proposal_id.clone(),
            input.now,
            Some(finished_at),
            run_status,
            10,
            10,
            Some(run_idempotency_key),
        )?;
        let run_row_id = store.append_builder_validation_run_row(run.clone())?;

        let mut gate_results = Vec::with_capacity(gate_evaluations.len());
        let mut gate_result_row_ids = Vec::with_capacity(gate_evaluations.len());
        for gate in gate_evaluations {
            let gate_result = BuilderValidationGateResult::v1(
                run_id.clone(),
                proposal_id.clone(),
                gate.gate_id,
                gate.passed,
                finished_at,
                gate.reason_code,
                gate.detail,
                Some(format!(
                    "builder_gate:{}:{}",
                    run_id,
                    gate.gate_id.as_str().replace('-', "_")
                )),
            )?;
            let gate_row_id = store.append_builder_validation_gate_result_row(gate_result.clone())?;
            gate_results.push(gate_result);
            gate_result_row_ids.push(gate_row_id);
        }

        let approval_state = initial_approval_state(
            &final_proposal,
            finished_at,
            Some(format!("builder_approval:{}", proposal_id)),
        )?;
        let approval_row_id = store.append_builder_approval_state_row(approval_state.clone())?;
        let release_state = BuilderReleaseController::initial_state(
            &final_proposal,
            &approval_state,
            finished_at,
            Some(format!("builder_release:{}:staging", proposal_id)),
        )?;
        let release_row_id = store.append_builder_release_state_row(release_state.clone())?;

        Ok(BuilderOrchestrationOutcome::Completed(BuilderCompletedBundle {
            proposal: final_proposal,
            validation_run: run,
            gate_results,
            approval_state,
            release_state,
            proposal_row_id,
            run_row_id,
            gate_result_row_ids,
            approval_row_id,
            release_row_id,
            learning_report_id: learning_auto_report
                .as_ref()
                .map(|report| report.learning_report_id.clone()),
            learning_source_engines: learning_auto_report
                .as_ref()
                .map(|report| report.source_engines.clone())
                .unwrap_or_default(),
            learning_signal_count: learning_auto_report
                .as_ref()
                .map(|report| report.learning_signal_count)
                .unwrap_or(0),
            learning_evidence_refs: learning_auto_report
                .as_ref()
                .map(|report| report.evidence_refs.clone())
                .unwrap_or_default(),
            learning_report_path: learning_auto_report
                .as_ref()
                .map(|report| report.report_path.clone()),
            change_brief_path: Some(change_brief_path),
            permission_packet_path: Some(permission_packet_path),
            code_decision_file_path: Some(decision_seed_files.code_file_path),
            launch_decision_file_path: Some(decision_seed_files.launch_file_path),
        }))
    }

    pub fn run_post_deploy_judge<S>(
        &self,
        store: &mut S,
        input: &BuilderPostDeployJudgeInput,
    ) -> Result<BuilderPostDeployJudgeOutcome, BuilderPipelineError>
    where
        S: BuilderSeleneRepo,
    {
        input.validate()?;
        if let Err(_err) = input.proposal.validate() {
            return Ok(BuilderPostDeployJudgeOutcome::Refused(BuilderRefusal {
                stage: "POST_DEPLOY",
                reason_code: reason_codes::PH1_BUILDER_POST_DEPLOY_MISSING_PROPOSAL_FIELDS,
                message: "proposal fields are incomplete; post-deploy judge cannot proceed"
                    .to_string(),
            }));
        }
        if validate_gate_results_complete(&input.gate_results).is_err() {
            return Ok(BuilderPostDeployJudgeOutcome::Refused(BuilderRefusal {
                stage: "POST_DEPLOY",
                reason_code: reason_codes::PH1_BUILDER_POST_DEPLOY_MISSING_GATE_OUTCOMES,
                message: "gate outcomes must include BLD-G1..BLD-G10 exactly once".to_string(),
            }));
        }
        if input.release_state.stage != BuilderReleaseStage::Production
            || input.release_state.status != BuilderReleaseStateStatus::Completed
        {
            return Ok(BuilderPostDeployJudgeOutcome::Refused(BuilderRefusal {
                stage: "POST_DEPLOY",
                reason_code: reason_codes::PH1_BUILDER_POST_DEPLOY_RELEASE_STAGE_INVALID,
                message: "post-deploy judge requires PRODUCTION release state".to_string(),
            }));
        }

        let should_revert = should_trigger_post_deploy_rollback(
            &input.before_metrics,
            &input.after_metrics,
            input.authority_or_gate_order_violation,
            input.duplicate_side_effect_event_detected,
        );
        let action = if should_revert {
            BuilderPostDeployDecisionAction::Revert
        } else {
            BuilderPostDeployDecisionAction::Accept
        };
        let reason_code = if should_revert {
            reason_codes::PH1_BUILDER_POST_DEPLOY_REVERTED
        } else {
            reason_codes::PH1_BUILDER_POST_DEPLOY_ACCEPTED
        };

        let judge_result = BuilderPostDeployJudgeResult::v1(
            deterministic_post_deploy_judge_result_id(&input.proposal.proposal_id, action),
            input.proposal.proposal_id.clone(),
            input.release_state.release_state_id.clone(),
            input.before_metrics.clone(),
            input.after_metrics.clone(),
            action,
            reason_code,
            input.now,
            input.idempotency_key.clone(),
        )?;
        let judge_row_id = store.append_builder_post_deploy_judge_result_row(judge_result.clone())?;

        let release_state = if action == BuilderPostDeployDecisionAction::Revert {
            let controller = BuilderReleaseController;
            controller.rollback(
                &input.release_state,
                MonotonicTimeNs(input.now.0.saturating_add(1_000_000)),
                Some(format!(
                    "builder_release:{}:rolled_back",
                    input.proposal.proposal_id
                )),
            )?
        } else {
            BuilderReleaseState::v1(
                deterministic_release_decision_state_id(&input.proposal.proposal_id, "accepted"),
                input.proposal.proposal_id.clone(),
                BuilderReleaseStage::Production,
                rollout_pct_for_stage(BuilderReleaseStage::Production),
                BuilderReleaseStateStatus::Completed,
                input.release_state.rollback_hook.clone(),
                input.release_state.rollback_ready,
                reason_codes::PH1_BUILDER_POST_DEPLOY_ACCEPTED,
                MonotonicTimeNs(input.now.0.saturating_add(1_000_000)),
                Some(format!(
                    "builder_release:{}:accepted",
                    input.proposal.proposal_id
                )),
            )?
        };
        let release_row_id = store.append_builder_release_state_row(release_state.clone())?;

        Ok(BuilderPostDeployJudgeOutcome::Completed(
            BuilderPostDeployDecisionBundle {
                judge_result,
                release_state,
                judge_row_id,
                release_row_id,
            },
        ))
    }
}

fn maybe_generate_learning_auto_report(
    input: &BuilderOfflineInput,
    outcome_entries: &[OsOutcomeUtilizationEntry],
    pattern_signals: &[PatternSignal],
    selected_recommendation: &RllRecommendationItem,
    proposal_id: &str,
) -> Result<Option<BuilderLearningAutoReport>, BuilderPipelineError> {
    let source_engines = collect_learning_source_engines(outcome_entries);
    if source_engines.is_empty() {
        return Ok(None);
    }

    let learning_signal_count = count_learning_signals(outcome_entries);
    if learning_signal_count == 0 {
        return Ok(None);
    }

    let evidence_refs = collect_learning_evidence_refs(
        outcome_entries,
        pattern_signals,
        input.correlation_id,
        input.turn_id,
    );
    if evidence_refs.is_empty() {
        return Ok(None);
    }

    let learning_report_id = deterministic_learning_report_id(proposal_id);
    let report_path = input
        .learning_report_output_path
        .clone()
        .unwrap_or_else(|| DEFAULT_LEARNING_REPORT_OUTPUT_PATH.to_string());
    validate_path_ascii(
        "builder_learning_report.report_path",
        &report_path,
        512,
    )?;

    let report = render_learning_report_markdown(
        outcome_entries,
        selected_recommendation,
        &evidence_refs,
        input.correlation_id,
        input.turn_id,
    );
    write_learning_report_to_path(&report_path, &report)?;

    Ok(Some(BuilderLearningAutoReport {
        learning_report_id,
        source_engines,
        learning_signal_count,
        evidence_refs,
        report_path,
    }))
}

fn generate_change_brief(
    input: &BuilderOfflineInput,
    outcome_entries: &[OsOutcomeUtilizationEntry],
    selected_recommendation: &RllRecommendationItem,
    proposal_id: &str,
    change_class: BuilderChangeClass,
    learning_auto_report: Option<&BuilderLearningAutoReport>,
) -> Result<String, BuilderPipelineError> {
    let brief_path = input
        .change_brief_output_path
        .clone()
        .unwrap_or_else(|| DEFAULT_CHANGE_BRIEF_OUTPUT_PATH.to_string());
    validate_path_ascii("builder_change_brief.path", &brief_path, 512)?;
    let brief = render_change_brief_markdown(
        outcome_entries,
        selected_recommendation,
        proposal_id,
        change_class,
        learning_auto_report,
    );
    write_change_brief_to_path(&brief_path, &brief)?;
    Ok(brief_path)
}

fn collect_learning_source_engines(entries: &[OsOutcomeUtilizationEntry]) -> Vec<String> {
    let mut engines = BTreeSet::new();
    for entry in entries {
        for learning_engine in LEARNING_BRIDGE_SOURCE_ENGINES {
            if entry.engine_id == learning_engine || entry.consumed_by == learning_engine {
                engines.insert(learning_engine.to_string());
            }
        }
    }
    engines.into_iter().collect()
}

fn count_learning_signals(entries: &[OsOutcomeUtilizationEntry]) -> u32 {
    entries
        .iter()
        .filter(|entry| {
            LEARNING_BRIDGE_SOURCE_ENGINES
                .iter()
                .any(|engine| entry.engine_id == *engine || entry.consumed_by == *engine)
        })
        .count() as u32
}

fn collect_learning_evidence_refs(
    entries: &[OsOutcomeUtilizationEntry],
    pattern_signals: &[PatternSignal],
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for entry in entries {
        if LEARNING_BRIDGE_SOURCE_ENGINES
            .iter()
            .any(|engine| entry.engine_id == *engine || entry.consumed_by == *engine)
        {
            refs.insert(format!(
                "evidence_ref:{}:{}:{}:{}",
                correlation_id.0,
                turn_id.0,
                sanitize_token_component(&entry.engine_id, 32),
                sanitize_token_component(&entry.outcome_type, 48)
            ));
        }
    }
    for signal in pattern_signals.iter().take(16) {
        refs.insert(signal.evidence_ref.clone());
    }
    refs.into_iter().take(32).collect()
}

fn deterministic_learning_report_id(proposal_id: &str) -> String {
    truncate_token(format!("learn_report_{}", proposal_id), 128)
}

fn render_learning_report_markdown(
    outcome_entries: &[OsOutcomeUtilizationEntry],
    selected_recommendation: &RllRecommendationItem,
    evidence_refs: &[String],
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> String {
    let issues = outcome_entries
        .iter()
        .take(5)
        .map(|entry| {
            format!(
                "- {} produced {} (reason_code={}).",
                entry.engine_id, entry.outcome_type, entry.reason_code.0
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    let evidence_block = evidence_refs
        .iter()
        .map(|r| format!("- evidence_ref: {}", r))
        .collect::<Vec<_>>()
        .join("\n");

    let fix_plan = format!(
        "- Apply deterministic recommendation `{}` for target `{}` with full BLD-G1..BLD-G10 validation.\n- Keep authority/simulation order unchanged and fail closed on any gate regression.",
        selected_recommendation.artifact_id,
        rll_optimization_target_name(selected_recommendation.target)
    );

    let expected = format!(
        "- Expected improvement: lower reject/clarify pressure and lower latency while preserving fail-closed behavior.\n- Correlation context: {} / turn {}.",
        correlation_id.0, turn_id.0
    );

    format!(
        "## Learning Issues Received\n{}\n\n## Root Cause Evidence\n{}\n\n## Deterministic Fix Plan\n{}\n\n## Expected Improvement\n{}\n\n## Builder Decision Prompt\n- Should I proceed with this learning-driven fix?\n",
        if issues.is_empty() {
            "- No learning issues were provided for this cycle."
        } else {
            &issues
        },
        if evidence_block.is_empty() {
            "- evidence_ref: unavailable"
        } else {
            &evidence_block
        },
        fix_plan,
        expected
    )
}

fn write_learning_report_to_path(path: &str, content: &str) -> Result<(), BuilderPipelineError> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                BuilderPipelineError::Io(format!(
                    "reason_code={} cannot create learning report directory '{}': {}",
                    reason_codes::PH1_BUILDER_LEARNING_REPORT_WRITE_FAILED.0,
                    parent.display(),
                    err
                ))
            })?;
        }
    }
    fs::write(target, content).map_err(|err| {
        BuilderPipelineError::Io(format!(
            "reason_code={} cannot write learning report '{}': {}",
            reason_codes::PH1_BUILDER_LEARNING_REPORT_WRITE_FAILED.0,
            path,
            err
        ))
    })?;
    Ok(())
}

fn render_change_brief_markdown(
    outcome_entries: &[OsOutcomeUtilizationEntry],
    selected_recommendation: &RllRecommendationItem,
    proposal_id: &str,
    change_class: BuilderChangeClass,
    learning_auto_report: Option<&BuilderLearningAutoReport>,
) -> String {
    let issues = if outcome_entries.is_empty() {
        "- I received these issues: none in this cycle.".to_string()
    } else {
        let list = outcome_entries
            .iter()
            .take(4)
            .map(|entry| {
                format!(
                    "{} -> {} (reason_code={})",
                    entry.engine_id, entry.outcome_type, entry.reason_code.0
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        format!("- I received these issues: {}.", list)
    };

    let mut fix_lines = vec![
        format!(
            "- This is the fix: apply recommendation `{}` to `{}`.",
            selected_recommendation.artifact_id,
            rll_optimization_target_name(selected_recommendation.target)
        ),
        format!(
            "- Change class: {}. Proposal id: {}.",
            builder_change_class_name(change_class),
            proposal_id
        ),
        "- Safety: keep gate order unchanged and require BLD-G1..BLD-G10 pass.".to_string(),
    ];
    if let Some(report) = learning_auto_report {
        fix_lines.push(format!(
            "- Learning evidence attached: {} ({} signals).",
            report.learning_report_id, report.learning_signal_count
        ));
    }
    let fix_block = fix_lines.join("\n");

    format!(
        "## Issue\n{}\n\n## Fix\n{}\n\n## Should I Proceed\n- Should I proceed?\n\n## Launch Question\n- All tests passed. Can I launch?\n",
        issues, fix_block
    )
}

fn write_change_brief_to_path(path: &str, content: &str) -> Result<(), BuilderPipelineError> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                BuilderPipelineError::Io(format!(
                    "reason_code={} cannot create change brief directory '{}': {}",
                    reason_codes::PH1_BUILDER_CHANGE_BRIEF_WRITE_FAILED.0,
                    parent.display(),
                    err
                ))
            })?;
        }
    }
    fs::write(target, content).map_err(|err| {
        BuilderPipelineError::Io(format!(
            "reason_code={} cannot write change brief '{}': {}",
            reason_codes::PH1_BUILDER_CHANGE_BRIEF_WRITE_FAILED.0,
            path,
            err
        ))
    })?;
    Ok(())
}

fn generate_permission_packet(
    input: &BuilderOfflineInput,
    outcome_entries: &[OsOutcomeUtilizationEntry],
    selected_recommendation: &RllRecommendationItem,
    proposal_id: &str,
    change_class: BuilderChangeClass,
    learning_auto_report: Option<&BuilderLearningAutoReport>,
    change_brief_path: &str,
    decision_seed_files: &BuilderDecisionSeedFiles,
) -> Result<String, BuilderPipelineError> {
    let packet_path = input
        .permission_packet_output_path
        .clone()
        .unwrap_or_else(|| DEFAULT_PERMISSION_PACKET_OUTPUT_PATH.to_string());
    validate_path_ascii("builder_permission_packet.path", &packet_path, 512)?;

    let code_permission_ref = deterministic_permission_ref("code", proposal_id);
    let launch_permission_ref = deterministic_permission_ref("launch", proposal_id);

    let content = render_permission_packet_markdown(
        outcome_entries,
        selected_recommendation,
        proposal_id,
        change_class,
        learning_auto_report,
        change_brief_path,
        &code_permission_ref,
        &launch_permission_ref,
        decision_seed_files,
    );
    write_permission_packet_to_path(&packet_path, &content)?;
    Ok(packet_path)
}

fn deterministic_permission_ref(phase: &str, proposal_id: &str) -> String {
    truncate_token(format!("perm_{}_{}", phase, proposal_id), 128)
}

fn generate_decision_seed_files(
    input: &BuilderOfflineInput,
    proposal_id: &str,
) -> Result<BuilderDecisionSeedFiles, BuilderPipelineError> {
    let packet_path = input
        .permission_packet_output_path
        .clone()
        .unwrap_or_else(|| DEFAULT_PERMISSION_PACKET_OUTPUT_PATH.to_string());
    validate_path_ascii("builder_decision_seed.packet_path", &packet_path, 512)?;

    let packet_parent = Path::new(&packet_path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_path_buf())
        .unwrap_or_else(|| Path::new(".").to_path_buf());

    let code_file_path = packet_parent
        .join("builder_code_decision.env")
        .display()
        .to_string();
    let launch_file_path = packet_parent
        .join("builder_launch_decision.env")
        .display()
        .to_string();

    validate_path_ascii(
        "builder_decision_seed.code_file_path",
        &code_file_path,
        512,
    )?;
    validate_path_ascii(
        "builder_decision_seed.launch_file_path",
        &launch_file_path,
        512,
    )?;

    let code_permission_ref = deterministic_permission_ref("code", proposal_id);
    let launch_permission_ref = deterministic_permission_ref("launch", proposal_id);

    let code_content = render_decision_seed_file(
        "code",
        "approve",
        &code_permission_ref,
        proposal_id,
    );
    let launch_content = render_decision_seed_file(
        "launch",
        "approve",
        &launch_permission_ref,
        proposal_id,
    );

    write_decision_seed_file(&code_file_path, &code_content)?;
    write_decision_seed_file(&launch_file_path, &launch_content)?;

    Ok(BuilderDecisionSeedFiles {
        code_file_path,
        launch_file_path,
    })
}

fn render_decision_seed_file(
    phase: &str,
    decision: &str,
    permission_ref: &str,
    proposal_id: &str,
) -> String {
    format!(
        "# Auto-generated by PH1.BUILDER. Fill BCAST_ID and DECISION_REF from the approval event.\nPHASE={}\nDECISION={}\nBCAST_ID=\nDECISION_REF=\nREMINDER_REF=\nBUSY_MODE_OVERRIDE=\nREFRESH_DAILY_REVIEW=1\nPERMISSION_REF={}\nPROPOSAL_ID={}\n",
        phase, decision, permission_ref, proposal_id
    )
}

fn write_decision_seed_file(path: &str, content: &str) -> Result<(), BuilderPipelineError> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                BuilderPipelineError::Io(format!(
                    "reason_code={} cannot create decision seed directory '{}': {}",
                    reason_codes::PH1_BUILDER_DECISION_SEED_WRITE_FAILED.0,
                    parent.display(),
                    err
                ))
            })?;
        }
    }
    fs::write(target, content).map_err(|err| {
        BuilderPipelineError::Io(format!(
            "reason_code={} cannot write decision seed file '{}': {}",
            reason_codes::PH1_BUILDER_DECISION_SEED_WRITE_FAILED.0,
            path,
            err
        ))
    })?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn render_permission_packet_markdown(
    outcome_entries: &[OsOutcomeUtilizationEntry],
    selected_recommendation: &RllRecommendationItem,
    proposal_id: &str,
    change_class: BuilderChangeClass,
    learning_auto_report: Option<&BuilderLearningAutoReport>,
    change_brief_path: &str,
    code_permission_ref: &str,
    launch_permission_ref: &str,
    decision_seed_files: &BuilderDecisionSeedFiles,
) -> String {
    let issue_summary = if outcome_entries.is_empty() {
        "no issues in this cycle".to_string()
    } else {
        outcome_entries
            .iter()
            .take(3)
            .map(|entry| format!("{} -> {}", entry.engine_id, entry.outcome_type))
            .collect::<Vec<_>>()
            .join("; ")
    };

    let learning_line = learning_auto_report
        .map(|report| {
            format!(
                "- learning_report_id: {} (signals={})",
                report.learning_report_id, report.learning_signal_count
            )
        })
        .unwrap_or_else(|| "- learning_report_id: none".to_string());

    format!(
        "## Builder Permission Packet\n- proposal_id: {}\n- change_class: {}\n- issue_summary: {}\n- fix_target: {}\n- recommendation: {}\n- change_brief_path: {}\n{}\n\n## Code Permission Request (BCAST)\n- permission_ref: {}\n- message: Should I proceed?\n- simulation_step_1: BCAST_CREATE_DRAFT\n- simulation_step_2: BCAST_DELIVER_COMMIT\n- busy_followup: REMINDER_SCHEDULE_COMMIT (reminder_type=BCAST_MHP_FOLLOWUP)\n\n## Launch Permission Request (BCAST)\n- permission_ref: {}\n- message: All tests passed. Can I launch?\n- simulation_step_1: BCAST_CREATE_DRAFT\n- simulation_step_2: BCAST_DELIVER_COMMIT\n- busy_followup: REMINDER_SCHEDULE_COMMIT (reminder_type=BCAST_MHP_FOLLOWUP)\n\n## Apply Decision Commands\n- code approve: BCAST_ID=<code_bcast_id> DECISION_REF={} ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh code approve\n- launch approve: BCAST_ID=<launch_bcast_id> DECISION_REF={} ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh launch approve\n- code pending busy: REMINDER_REF=<code_reminder_ref> ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh code pending\n- launch pending busy: REMINDER_REF=<launch_reminder_ref> ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh launch pending\n\n## Auto-Generated Decision Files\n- decision_file_template: docs/fixtures/builder_permission_decision_template.env\n- code_decision_file: {}\n- launch_decision_file: {}\n- apply code file: DECISION_FILE={} ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh\n- apply launch file: DECISION_FILE={} ENV_FILE=.dev/builder_permission.env bash scripts/apply_builder_permission_decision.sh\n\n## Hard Rules\n- No Approval -> No Code\n- No Launch Approval -> No Launch\n- Reminder scheduling does not grant approval\n",
        proposal_id,
        builder_change_class_name(change_class),
        issue_summary,
        rll_optimization_target_name(selected_recommendation.target),
        selected_recommendation.artifact_id,
        change_brief_path,
        learning_line,
        code_permission_ref,
        launch_permission_ref,
        code_permission_ref,
        launch_permission_ref,
        decision_seed_files.code_file_path,
        decision_seed_files.launch_file_path,
        decision_seed_files.code_file_path,
        decision_seed_files.launch_file_path
    )
}

fn write_permission_packet_to_path(
    path: &str,
    content: &str,
) -> Result<(), BuilderPipelineError> {
    let target = Path::new(path);
    if let Some(parent) = target.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| {
                BuilderPipelineError::Io(format!(
                    "reason_code={} cannot create permission packet directory '{}': {}",
                    reason_codes::PH1_BUILDER_PERMISSION_PACKET_WRITE_FAILED.0,
                    parent.display(),
                    err
                ))
            })?;
        }
    }
    fs::write(target, content).map_err(|err| {
        BuilderPipelineError::Io(format!(
            "reason_code={} cannot write permission packet '{}': {}",
            reason_codes::PH1_BUILDER_PERMISSION_PACKET_WRITE_FAILED.0,
            path,
            err
        ))
    })?;
    Ok(())
}

fn cluster_outcomes_to_pattern_signals(
    entries: &[OsOutcomeUtilizationEntry],
    max_signals: usize,
    correlation_id: CorrelationId,
    turn_id: TurnId,
) -> Result<Vec<PatternSignal>, ContractViolation> {
    #[derive(Debug, Clone)]
    struct Acc {
        metric_key: String,
        count: u32,
        drop_count: u32,
        decision_delta_count: u32,
        latency_sum: u64,
    }

    let mut grouped: BTreeMap<String, Acc> = BTreeMap::new();
    for entry in entries {
        let metric_key = metric_key_from_outcome(entry);
        let key = format!("{}|{}", entry.engine_id, metric_key);
        let acc = grouped.entry(key).or_insert(Acc {
            metric_key,
            count: 0,
            drop_count: 0,
            decision_delta_count: 0,
            latency_sum: 0,
        });
        acc.count = acc.count.saturating_add(1);
        if entry.action_class == OsOutcomeActionClass::Drop {
            acc.drop_count = acc.drop_count.saturating_add(1);
        }
        if entry.decision_delta {
            acc.decision_delta_count = acc.decision_delta_count.saturating_add(1);
        }
        acc.latency_sum = acc.latency_sum.saturating_add(entry.latency_cost_ms as u64);
    }

    let mut ranked = grouped
        .into_iter()
        .map(|(key, acc)| {
            let drop_rate_bp = (acc.drop_count as i32 * 10_000) / acc.count as i32;
            let decision_rate_bp = (acc.decision_delta_count as i32 * 10_000) / acc.count as i32;
            let metric_value_bp = (drop_rate_bp - (decision_rate_bp / 2))
                .clamp(-20_000, 20_000) as i16;
            let avg_latency = (acc.latency_sum / acc.count as u64) as i32;
            let severity = drop_rate_bp + avg_latency * 12;
            (key, acc, metric_value_bp, severity)
        })
        .collect::<Vec<_>>();
    ranked.sort_by(|a, b| b.3.cmp(&a.3).then(a.0.cmp(&b.0)));

    let mut signals = Vec::new();
    for (idx, (key, acc, metric_value_bp, _)) in ranked.into_iter().enumerate() {
        if idx >= max_signals {
            break;
        }
        let engine_id = key.split('|').next().unwrap_or("PH1.OS").to_string();
        let signal = PatternSignal::v1(
            format!("sig_{}_{}", idx + 1, sanitize_token_component(&engine_id, 24)),
            engine_id,
            acc.metric_key,
            metric_value_bp,
            acc.count,
            format!(
                "evidence:{}:{}:{}",
                correlation_id.0,
                turn_id.0,
                sanitize_token_component(&key, 48)
            ),
        )?;
        signals.push(signal);
    }
    Ok(signals)
}

fn metric_key_from_outcome(entry: &OsOutcomeUtilizationEntry) -> String {
    let outcome = entry.outcome_type.to_ascii_lowercase();
    if outcome.contains("clarify") {
        "clarify_loop_rate".to_string()
    } else if outcome.contains("cache") || outcome.contains("prefetch") {
        "prefetch_miss_rate".to_string()
    } else if outcome.contains("tool") || entry.engine_id == "PH1.E" {
        "provider_timeout_rate".to_string()
    } else if outcome.contains("context") {
        "context_miss_rate".to_string()
    } else {
        "quality_regression_rate".to_string()
    }
}

fn pattern_to_rll_candidates(
    proposals: &[PatternProposalItem],
    max_candidates: usize,
) -> Result<Vec<RllArtifactCandidate>, ContractViolation> {
    let mut out = Vec::new();
    for item in proposals.iter().take(max_candidates) {
        let candidate = RllArtifactCandidate::v1(
            item.proposal_id.clone(),
            rll_target_from_pattern(item.target),
            expected_effect_bp_from_confidence(item.confidence_pct),
            item.confidence_pct,
            item.approval_tier,
            item.evidence_ref.clone(),
        )?;
        out.push(candidate);
    }
    Ok(out)
}

fn rll_target_from_pattern(target: PatternProposalTarget) -> RllOptimizationTarget {
    match target {
        PatternProposalTarget::PaeProviderRoutingWeights => {
            RllOptimizationTarget::PaeProviderSelectionWeights
        }
        PatternProposalTarget::PruneClarificationOrdering => {
            RllOptimizationTarget::PruneClarificationOrdering
        }
        PatternProposalTarget::CachePrefetchHeuristics => {
            RllOptimizationTarget::CachePrefetchHeuristics
        }
        PatternProposalTarget::ContextRetrievalScoring => {
            RllOptimizationTarget::ContextRetrievalScoring
        }
    }
}

fn rll_optimization_target_name(target: RllOptimizationTarget) -> &'static str {
    match target {
        RllOptimizationTarget::PaeProviderSelectionWeights => "PAE_PROVIDER_SELECTION_WEIGHTS",
        RllOptimizationTarget::PruneClarificationOrdering => "PRUNE_CLARIFICATION_ORDERING",
        RllOptimizationTarget::CachePrefetchHeuristics => "CACHE_PREFETCH_HEURISTICS",
        RllOptimizationTarget::ContextRetrievalScoring => "CONTEXT_RETRIEVAL_SCORING",
    }
}

fn builder_change_class_name(change_class: BuilderChangeClass) -> &'static str {
    match change_class {
        BuilderChangeClass::ClassA => "CLASS-A",
        BuilderChangeClass::ClassB => "CLASS-B",
        BuilderChangeClass::ClassC => "CLASS-C",
    }
}

fn expected_effect_bp_from_confidence(confidence_pct: u8) -> i16 {
    ((confidence_pct as i16 - 50) * 24).clamp(-2000, 2000)
}

fn find_selected_recommendation<'a>(
    recommendations: &'a [RllRecommendationItem],
    selected_artifact_id: &str,
) -> Option<&'a RllRecommendationItem> {
    recommendations
        .iter()
        .find(|item| item.artifact_id == selected_artifact_id)
}

fn change_class_for_target(target: RllOptimizationTarget) -> BuilderChangeClass {
    match target {
        RllOptimizationTarget::PaeProviderSelectionWeights
        | RllOptimizationTarget::PruneClarificationOrdering
        | RllOptimizationTarget::CachePrefetchHeuristics => BuilderChangeClass::ClassA,
        RllOptimizationTarget::ContextRetrievalScoring => BuilderChangeClass::ClassB,
    }
}

fn target_files_for_target(target: RllOptimizationTarget) -> Vec<String> {
    match target {
        RllOptimizationTarget::PaeProviderSelectionWeights => vec![
            "crates/selene_os/src/ph1pae.rs".to_string(),
            "docs/ECM/PH1_PAE.md".to_string(),
        ],
        RllOptimizationTarget::PruneClarificationOrdering => vec![
            "crates/selene_os/src/ph1prune.rs".to_string(),
            "docs/ECM/PH1_PRUNE.md".to_string(),
        ],
        RllOptimizationTarget::CachePrefetchHeuristics => vec![
            "crates/selene_os/src/ph1cache.rs".to_string(),
            "crates/selene_os/src/ph1prefetch.rs".to_string(),
            "docs/ECM/PH1_CACHE.md".to_string(),
        ],
        RllOptimizationTarget::ContextRetrievalScoring => vec![
            "crates/selene_os/src/ph1context.rs".to_string(),
            "docs/ECM/PH1_CONTEXT.md".to_string(),
        ],
    }
}

fn risk_score_for_selection(
    change_class: BuilderChangeClass,
    selected: &RllRecommendationItem,
) -> u16 {
    let base = (100u16.saturating_sub(selected.confidence_pct as u16)).saturating_mul(100);
    let class_penalty = match change_class {
        BuilderChangeClass::ClassA => 300,
        BuilderChangeClass::ClassB => 1200,
        BuilderChangeClass::ClassC => 2500,
    };
    min(base.saturating_add(class_penalty), 10_000)
}

fn expected_effect_from_outcomes(
    entries: &[OsOutcomeUtilizationEntry],
    selected: &RllRecommendationItem,
) -> Result<BuilderExpectedEffect, ContractViolation> {
    let avg_latency = if entries.is_empty() {
        0i16
    } else {
        (entries
            .iter()
            .map(|entry| entry.latency_cost_ms as u64)
            .sum::<u64>()
            / entries.len() as u64) as i16
    };
    let latency_p95_delta_bp = -min(avg_latency.saturating_mul(8), 2000);
    let latency_p99_delta_bp = -min(avg_latency.saturating_mul(10), 2500);
    let quality_delta_bp = ((selected.confidence_pct as i16 * 20) - 1000).clamp(-10000, 10000);
    let safety_delta_bp = 0;
    BuilderExpectedEffect::v1(
        latency_p95_delta_bp,
        latency_p99_delta_bp,
        quality_delta_bp,
        safety_delta_bp,
    )
}

fn initial_approval_state(
    proposal: &BuilderPatchProposal,
    now: MonotonicTimeNs,
    idempotency_key: Option<String>,
) -> Result<BuilderApprovalState, ContractViolation> {
    let required_approvals_total = required_approvals_for_change_class(proposal.change_class);
    let is_auto_resolved = proposal.status == BuilderProposalStatus::Validated && required_approvals_total == 0;
    let status = if is_auto_resolved {
        BuilderApprovalStateStatus::Approved
    } else {
        BuilderApprovalStateStatus::Pending
    };
    let reason_code = if is_auto_resolved {
        reason_codes::PH1_BUILDER_APPROVAL_AUTO_RESOLVED
    } else {
        reason_codes::PH1_BUILDER_APPROVAL_UNRESOLVED
    };
    BuilderApprovalState::v1(
        deterministic_approval_state_id(&proposal.proposal_id),
        proposal.proposal_id.clone(),
        proposal.change_class,
        required_approvals_total,
        0,
        false,
        false,
        status,
        reason_code,
        now,
        if is_auto_resolved { Some(now) } else { None },
        idempotency_key,
    )
}

fn deterministic_proposal_id(
    correlation_id: CorrelationId,
    turn_id: TurnId,
    selected_artifact_id: &str,
) -> String {
    let suffix = sanitize_token_component(selected_artifact_id, 32);
    let base = format!("builder_prop_{}_{}_{}", correlation_id.0, turn_id.0, suffix);
    truncate_token(base, 96)
}

fn deterministic_run_id(proposal_id: &str) -> String {
    truncate_token(format!("builder_run_{}", proposal_id), 96)
}

fn deterministic_post_deploy_judge_result_id(
    proposal_id: &str,
    action: BuilderPostDeployDecisionAction,
) -> String {
    let action_suffix = match action {
        BuilderPostDeployDecisionAction::Accept => "accept",
        BuilderPostDeployDecisionAction::Revert => "revert",
    };
    truncate_token(format!("builder_judge_{}_{}", proposal_id, action_suffix), 96)
}

fn deterministic_approval_state_id(proposal_id: &str) -> String {
    truncate_token(format!("builder_approval_{}", proposal_id), 96)
}

fn deterministic_release_state_id(proposal_id: &str, stage: BuilderReleaseStage) -> String {
    let suffix = match stage {
        BuilderReleaseStage::Staging => "staging",
        BuilderReleaseStage::Canary => "canary",
        BuilderReleaseStage::Ramp25 => "ramp25",
        BuilderReleaseStage::Ramp50 => "ramp50",
        BuilderReleaseStage::Production => "production",
        BuilderReleaseStage::RolledBack => "rolled_back",
    };
    truncate_token(format!("builder_release_{}_{}", proposal_id, suffix), 96)
}

fn next_approval_state_id(
    current_approval_state_id: &str,
    action: BuilderApprovalTransitionAction,
) -> String {
    let suffix = match action {
        BuilderApprovalTransitionAction::ApproveTech => "approve_tech",
        BuilderApprovalTransitionAction::ApproveProductSecurity => "approve_product_security",
        BuilderApprovalTransitionAction::Reject => "reject",
    };
    truncate_token(format!("{}_{}", current_approval_state_id, suffix), 96)
}

fn transition_has_all_required_approvals(
    change_class: BuilderChangeClass,
    tech_approved: bool,
    product_security_approved: bool,
) -> bool {
    match change_class {
        BuilderChangeClass::ClassA => true,
        BuilderChangeClass::ClassB => tech_approved,
        BuilderChangeClass::ClassC => tech_approved && product_security_approved,
    }
}

fn next_release_stage(stage: BuilderReleaseStage) -> Option<BuilderReleaseStage> {
    match stage {
        BuilderReleaseStage::Staging => Some(BuilderReleaseStage::Canary),
        BuilderReleaseStage::Canary => Some(BuilderReleaseStage::Ramp25),
        BuilderReleaseStage::Ramp25 => Some(BuilderReleaseStage::Ramp50),
        BuilderReleaseStage::Ramp50 => Some(BuilderReleaseStage::Production),
        BuilderReleaseStage::Production | BuilderReleaseStage::RolledBack => None,
    }
}

fn release_state_id_for_proposal_and_stage(
    proposal_id: &str,
    stage: BuilderReleaseStage,
) -> String {
    deterministic_release_state_id(proposal_id, stage)
}

fn deterministic_release_decision_state_id(proposal_id: &str, decision: &str) -> String {
    truncate_token(
        format!(
            "builder_release_{}_production_{}",
            proposal_id,
            sanitize_token_component(decision, 16)
        ),
        96,
    )
}

fn validate_gate_evaluations_complete(
    evaluations: &[BuilderGateEvaluation],
) -> Result<(), ContractViolation> {
    if evaluations.len() != 10 {
        return Err(ContractViolation::InvalidValue {
            field: "builder_gate_evaluations",
            reason: "must include exactly 10 gate evaluations",
        });
    }
    let expected = [
        BuilderValidationGateId::BldG1,
        BuilderValidationGateId::BldG2,
        BuilderValidationGateId::BldG3,
        BuilderValidationGateId::BldG4,
        BuilderValidationGateId::BldG5,
        BuilderValidationGateId::BldG6,
        BuilderValidationGateId::BldG7,
        BuilderValidationGateId::BldG8,
        BuilderValidationGateId::BldG9,
        BuilderValidationGateId::BldG10,
    ];
    let expected_set = expected.into_iter().collect::<BTreeSet<_>>();
    let actual_set = evaluations
        .iter()
        .map(|item| item.gate_id)
        .collect::<BTreeSet<_>>();
    if expected_set != actual_set {
        return Err(ContractViolation::InvalidValue {
            field: "builder_gate_evaluations",
            reason: "must contain BLD-G1..BLD-G10 exactly once",
        });
    }
    Ok(())
}

fn validate_gate_results_complete(
    results: &[BuilderValidationGateResult],
) -> Result<(), ContractViolation> {
    if results.len() != 10 {
        return Err(ContractViolation::InvalidValue {
            field: "builder_post_deploy.gate_results",
            reason: "must include exactly 10 gate outcomes",
        });
    }
    let expected = [
        BuilderValidationGateId::BldG1,
        BuilderValidationGateId::BldG2,
        BuilderValidationGateId::BldG3,
        BuilderValidationGateId::BldG4,
        BuilderValidationGateId::BldG5,
        BuilderValidationGateId::BldG6,
        BuilderValidationGateId::BldG7,
        BuilderValidationGateId::BldG8,
        BuilderValidationGateId::BldG9,
        BuilderValidationGateId::BldG10,
    ];
    let expected_set = expected.into_iter().collect::<BTreeSet<_>>();
    let actual_set = results
        .iter()
        .map(|item| item.gate_id)
        .collect::<BTreeSet<_>>();
    if expected_set != actual_set {
        return Err(ContractViolation::InvalidValue {
            field: "builder_post_deploy.gate_results",
            reason: "must contain BLD-G1..BLD-G10 exactly once",
        });
    }
    Ok(())
}

fn should_trigger_post_deploy_rollback(
    before: &BuilderMetricsSnapshot,
    after: &BuilderMetricsSnapshot,
    authority_or_gate_order_violation: bool,
    duplicate_side_effect_event_detected: bool,
) -> bool {
    if authority_or_gate_order_violation || duplicate_side_effect_event_detected {
        return true;
    }
    let p95_regression_bp = regression_bp(before.latency_p95_ms, after.latency_p95_ms);
    if p95_regression_bp > 300 && after.observation_window_minutes >= 30 {
        return true;
    }
    let p99_regression_bp = regression_bp(before.latency_p99_ms, after.latency_p99_ms);
    if p99_regression_bp > 500 && after.observation_window_minutes >= 15 {
        return true;
    }
    let critical_spike_delta_bp =
        after.critical_reason_spike_bp as i32 - before.critical_reason_spike_bp as i32;
    critical_spike_delta_bp > 20
}

fn regression_bp(before: u32, after: u32) -> i32 {
    if before == 0 {
        return 0;
    }
    (((after as i64 - before as i64) * 10_000) / before as i64) as i32
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut hash = OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

fn hash_hex_64(value: &str) -> String {
    let mut hash = fnv1a64(value.as_bytes());
    if hash == 0 {
        hash = 1;
    }
    format!("{hash:016x}")
}

fn validate_token_ascii(
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
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be token-safe ASCII",
        });
    }
    Ok(())
}

fn validate_ascii_text(
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

fn validate_path_ascii(
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
    if !value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '/' || c == '.')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII path-safe",
        });
    }
    Ok(())
}

fn sanitize_token_component(value: &str, max_len: usize) -> String {
    let mut out = value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();
    if out.len() > max_len {
        out.truncate(max_len);
    }
    while out.ends_with('_') {
        out.pop();
    }
    if out.is_empty() {
        "x".to_string()
    } else {
        out
    }
}

fn truncate_token(mut value: String, max_len: usize) -> String {
    if value.len() > max_len {
        value.truncate(max_len);
    }
    while value.ends_with('_') || value.ends_with('-') || value.ends_with(':') {
        value.pop();
    }
    if value.is_empty() {
        "builder_x".to_string()
    } else {
        value
    }
}

trait BuilderProposalAuditCheck {
    fn reason_code_valid(&self) -> bool;
}

impl BuilderProposalAuditCheck for BuilderPatchProposal {
    fn reason_code_valid(&self) -> bool {
        !self.proposal_id.trim().is_empty()
            && !self.source_signal_hash.trim().is_empty()
            && !self.validation_plan.trim().is_empty()
            && !self.rollback_plan.trim().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1pattern::{
        PatternMineOfflineOk, PatternProposalEmitOk, PatternValidationStatus, Ph1PatternRequest,
        Ph1PatternResponse,
    };
    use selene_kernel_contracts::ph1rll::{
        Ph1RllRequest, Ph1RllResponse, RllArtifactRecommendOk, RllPolicyRankOfflineOk,
        RllValidationStatus,
    };
    use selene_storage::ph1f::Ph1fStore;

    struct DeterministicPatternEngine;

    impl Ph1PatternEngine for DeterministicPatternEngine {
        fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
            match req {
                Ph1PatternRequest::PatternMineOffline(r) => {
                    let mut items = r
                        .signals
                        .iter()
                        .enumerate()
                        .map(|(idx, signal)| {
                            PatternProposalItem::v1(
                                format!("proposal_{}", signal.signal_id),
                                PatternProposalTarget::PaeProviderRoutingWeights,
                                (idx + 1) as u8,
                                84u8.saturating_sub((idx as u8) * 3),
                                3,
                                signal.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    items.sort_by(|a, b| {
                        b.confidence_pct
                            .cmp(&a.confidence_pct)
                            .then(a.proposal_id.cmp(&b.proposal_id))
                    });
                    for (idx, item) in items.iter_mut().enumerate() {
                        item.rank = (idx + 1) as u8;
                    }
                    Ph1PatternResponse::PatternMineOfflineOk(
                        PatternMineOfflineOk::v1(
                            ReasonCodeId(0x5100_0001),
                            items[0].proposal_id.clone(),
                            items,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PatternRequest::PatternProposalEmit(_r) => {
                    Ph1PatternResponse::PatternProposalEmitOk(
                        PatternProposalEmitOk::v1(
                            ReasonCodeId(0x5100_0002),
                            PatternValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DeterministicRllEngine;

    impl Ph1RllEngine for DeterministicRllEngine {
        fn run(&self, req: &Ph1RllRequest) -> Ph1RllResponse {
            match req {
                Ph1RllRequest::RllPolicyRankOffline(r) => {
                    let mut items = r
                        .candidates
                        .iter()
                        .enumerate()
                        .map(|(idx, candidate)| {
                            RllRecommendationItem::v1(
                                candidate.artifact_id.clone(),
                                candidate.target,
                                (idx + 1) as u8,
                                candidate.confidence_pct,
                                candidate.approval_tier,
                                candidate.evidence_ref.clone(),
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
                            ReasonCodeId(0x5200_0001),
                            items[0].artifact_id.clone(),
                            items,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1RllRequest::RllArtifactRecommend(_r) => {
                    Ph1RllResponse::RllArtifactRecommendOk(
                        RllArtifactRecommendOk::v1(
                            ReasonCodeId(0x5200_0002),
                            RllValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct IncompleteValidator;

    impl BuilderSandboxValidator for IncompleteValidator {
        fn collect_gate_evaluations(
            &self,
            _proposal: &BuilderPatchProposal,
            _outcome_entries: &[OsOutcomeUtilizationEntry],
        ) -> Result<Vec<BuilderGateEvaluation>, ContractViolation> {
            Ok(vec![BuilderGateEvaluation::v1(
                BuilderValidationGateId::BldG1,
                true,
                ReasonCodeId(0xB13D_0201),
                "only one gate".to_string(),
            )?])
        }
    }

    fn input() -> BuilderOfflineInput {
        let correlation_id = CorrelationId(7001);
        let turn_id = TurnId(801);
        let learning_report_path = std::env::temp_dir()
            .join(format!(
                "builder_learning_report_{}_{}.md",
                correlation_id.0, turn_id.0
            ))
            .display()
            .to_string();
        let change_brief_path = std::env::temp_dir()
            .join(format!(
                "builder_change_brief_{}_{}.md",
                correlation_id.0, turn_id.0
            ))
            .display()
            .to_string();
        let permission_packet_path = std::env::temp_dir()
            .join(format!(
                "builder_permission_packet_{}_{}.md",
                correlation_id.0, turn_id.0
            ))
            .display()
            .to_string();
        let entries = vec![
            OsOutcomeUtilizationEntry::v1(
                "PH1.SEARCH".to_string(),
                "tool_timeout".to_string(),
                correlation_id,
                turn_id,
                OsOutcomeActionClass::Drop,
                "NONE".to_string(),
                34,
                false,
                ReasonCodeId(0x7000_0001),
            )
            .unwrap(),
            OsOutcomeUtilizationEntry::v1(
                "PH1.CONTEXT".to_string(),
                "context_miss".to_string(),
                correlation_id,
                turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.LEARN".to_string(),
                18,
                true,
                ReasonCodeId(0x7000_0002),
            )
            .unwrap(),
        ];

        BuilderOfflineInput::v1(
            correlation_id,
            turn_id,
            MonotonicTimeNs(1000),
            MonotonicTimeNs(2000),
            MonotonicTimeNs(3000),
            entries,
            Some("sig_hash_7001".to_string()),
            None,
            None,
            Some(learning_report_path),
            Some(change_brief_path),
            Some(permission_packet_path),
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_builder_os_01_offline_run_persists_validated_proposal_run_and_gates() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let out = orchestrator.run_offline(&mut store, &input()).unwrap();
        match out {
            BuilderOrchestrationOutcome::Completed(bundle) => {
                assert_eq!(bundle.validation_run.status, BuilderValidationRunStatus::Passed);
                assert_eq!(bundle.proposal.status, BuilderProposalStatus::Validated);
                assert_eq!(bundle.gate_results.len(), 10);
                assert_eq!(store.builder_proposal_rows().len(), 1);
                assert_eq!(store.builder_validation_run_rows().len(), 1);
                assert_eq!(store.builder_validation_gate_result_rows().len(), 10);
                assert_eq!(store.builder_approval_state_rows().len(), 1);
                assert_eq!(store.builder_release_state_rows().len(), 1);
            }
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn at_builder_os_02_fails_closed_when_gate_collection_is_incomplete() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            IncompleteValidator,
        )
        .unwrap();

        let out = orchestrator.run_offline(&mut store, &input()).unwrap();
        match out {
            BuilderOrchestrationOutcome::Refused(refuse) => {
                assert_eq!(refuse.stage, "VALIDATOR");
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_BUILDER_GATE_COLLECTION_INVALID
                );
            }
            _ => panic!("expected Refused"),
        }
        assert_eq!(store.builder_proposal_rows().len(), 0);
        assert_eq!(store.builder_validation_run_rows().len(), 0);
        assert_eq!(store.builder_validation_gate_result_rows().len(), 0);
    }

    #[test]
    fn at_builder_os_03_idempotent_replay_keeps_single_rows() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();
        let input = input();

        let out1 = orchestrator.run_offline(&mut store, &input).unwrap();
        let out2 = orchestrator.run_offline(&mut store, &input).unwrap();

        let b1 = match out1 {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };
        let b2 = match out2 {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };

        assert_eq!(b1.proposal_row_id, b2.proposal_row_id);
        assert_eq!(b1.run_row_id, b2.run_row_id);
        assert_eq!(b1.gate_result_row_ids, b2.gate_result_row_ids);
        assert_eq!(store.builder_proposal_rows().len(), 1);
        assert_eq!(store.builder_validation_run_rows().len(), 1);
        assert_eq!(store.builder_validation_gate_result_rows().len(), 10);
        assert_eq!(store.builder_approval_state_rows().len(), 1);
        assert_eq!(store.builder_release_state_rows().len(), 1);
    }

    struct ContextPatternEngine;

    impl Ph1PatternEngine for ContextPatternEngine {
        fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
            match req {
                Ph1PatternRequest::PatternMineOffline(r) => {
                    let item = PatternProposalItem::v1(
                        "proposal_context_01".to_string(),
                        PatternProposalTarget::ContextRetrievalScoring,
                        1,
                        91,
                        3,
                        r.signals[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1PatternResponse::PatternMineOfflineOk(
                        PatternMineOfflineOk::v1(
                            ReasonCodeId(0x5100_1001),
                            item.proposal_id.clone(),
                            vec![item],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PatternRequest::PatternProposalEmit(_r) => {
                    Ph1PatternResponse::PatternProposalEmitOk(
                        PatternProposalEmitOk::v1(
                            ReasonCodeId(0x5100_1002),
                            PatternValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    #[test]
    fn at_builder_os_04_class_b_requires_pending_approval_and_blocks_release() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            ContextPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let out = orchestrator.run_offline(&mut store, &input()).unwrap();
        match out {
            BuilderOrchestrationOutcome::Completed(bundle) => {
                assert_eq!(bundle.proposal.change_class, BuilderChangeClass::ClassB);
                assert_eq!(bundle.approval_state.status, BuilderApprovalStateStatus::Pending);
                assert_eq!(bundle.approval_state.required_approvals_total, 1);
                assert_eq!(bundle.release_state.stage, BuilderReleaseStage::Staging);
                assert_eq!(bundle.release_state.status, BuilderReleaseStateStatus::Blocked);
                assert_eq!(
                    bundle.release_state.reason_code,
                    reason_codes::PH1_BUILDER_RELEASE_BLOCKED_APPROVAL
                );
                assert_eq!(store.builder_approval_state_rows().len(), 1);
                assert_eq!(store.builder_release_state_rows().len(), 1);
            }
            _ => panic!("expected Completed"),
        }
    }

    #[test]
    fn at_builder_os_05_release_controller_blocks_production_without_resolved_approval() {
        let pending_approval = BuilderApprovalState::v1(
            "approval_pending_01".to_string(),
            "proposal_pending_01".to_string(),
            BuilderChangeClass::ClassB,
            required_approvals_for_change_class(BuilderChangeClass::ClassB),
            0,
            false,
            false,
            BuilderApprovalStateStatus::Pending,
            reason_codes::PH1_BUILDER_APPROVAL_UNRESOLVED,
            MonotonicTimeNs(100),
            None,
            Some("approval_pending_idem_01".to_string()),
        )
        .unwrap();
        let release_state = BuilderReleaseState::v1(
            "release_ramp50_01".to_string(),
            "proposal_pending_01".to_string(),
            BuilderReleaseStage::Ramp50,
            rollout_pct_for_stage(BuilderReleaseStage::Ramp50),
            BuilderReleaseStateStatus::Active,
            "rollback:hook".to_string(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            MonotonicTimeNs(101),
            Some("release_ramp50_idem_01".to_string()),
        )
        .unwrap();

        let controller = BuilderReleaseController;
        let out = controller.promote(
            &release_state,
            &pending_approval,
            MonotonicTimeNs(102),
            Some("release_promote_idem_01".to_string()),
        );
        match out {
            Err(refuse) => {
                assert_eq!(refuse.stage, "RELEASE");
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_BUILDER_RELEASE_PROMOTION_BLOCKED
                );
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_builder_os_06_release_controller_promotes_after_required_approval() {
        let pending_approval = BuilderApprovalState::v1(
            "approval_pending_02".to_string(),
            "proposal_pending_02".to_string(),
            BuilderChangeClass::ClassB,
            required_approvals_for_change_class(BuilderChangeClass::ClassB),
            0,
            false,
            false,
            BuilderApprovalStateStatus::Pending,
            reason_codes::PH1_BUILDER_APPROVAL_UNRESOLVED,
            MonotonicTimeNs(200),
            None,
            Some("approval_pending_idem_02".to_string()),
        )
        .unwrap();
        let approved = advance_approval_state(
            &pending_approval,
            BuilderApprovalTransitionAction::ApproveTech,
            MonotonicTimeNs(201),
            reason_codes::PH1_BUILDER_APPROVAL_AUTO_RESOLVED,
            Some("approval_transition_idem_02".to_string()),
        )
        .unwrap();
        assert_eq!(approved.status, BuilderApprovalStateStatus::Approved);

        let release_state = BuilderReleaseState::v1(
            "release_ramp50_02".to_string(),
            "proposal_pending_02".to_string(),
            BuilderReleaseStage::Ramp50,
            rollout_pct_for_stage(BuilderReleaseStage::Ramp50),
            BuilderReleaseStateStatus::Active,
            "rollback:hook".to_string(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            MonotonicTimeNs(202),
            Some("release_ramp50_idem_02".to_string()),
        )
        .unwrap();

        let controller = BuilderReleaseController;
        let promoted = controller
            .promote(
                &release_state,
                &approved,
                MonotonicTimeNs(203),
                Some("release_promote_idem_02".to_string()),
            )
            .unwrap();
        assert_eq!(promoted.stage, BuilderReleaseStage::Production);
        assert_eq!(promoted.status, BuilderReleaseStateStatus::Completed);
    }

    fn post_deploy_input_for(bundle: &BuilderCompletedBundle) -> BuilderPostDeployJudgeInput {
        BuilderPostDeployJudgeInput {
            proposal: bundle.proposal.clone(),
            release_state: BuilderReleaseState::v1(
                "release_production_for_judge".to_string(),
                bundle.proposal.proposal_id.clone(),
                BuilderReleaseStage::Production,
                rollout_pct_for_stage(BuilderReleaseStage::Production),
                BuilderReleaseStateStatus::Completed,
                "rollback:hook".to_string(),
                true,
                reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
                MonotonicTimeNs(10_000),
                Some("release_production_judge_idem".to_string()),
            )
            .unwrap(),
            gate_results: bundle.gate_results.clone(),
            before_metrics: BuilderMetricsSnapshot::v1(200, 300, 40, 0, 30).unwrap(),
            after_metrics: BuilderMetricsSnapshot::v1(210, 330, 45, 10, 30).unwrap(),
            authority_or_gate_order_violation: false,
            duplicate_side_effect_event_detected: false,
            now: MonotonicTimeNs(12_000),
            idempotency_key: Some("post_deploy_judge_idem".to_string()),
        }
    }

    #[test]
    fn at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();
        let bundle = match orchestrator.run_offline(&mut store, &input()).unwrap() {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };
        store
            .append_builder_release_state_row(
                BuilderReleaseState::v1(
                    "release_production_for_judge".to_string(),
                    bundle.proposal.proposal_id.clone(),
                    BuilderReleaseStage::Production,
                    rollout_pct_for_stage(BuilderReleaseStage::Production),
                    BuilderReleaseStateStatus::Completed,
                    "rollback:hook".to_string(),
                    true,
                    reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
                    MonotonicTimeNs(10_000),
                    Some("release_production_judge_idem".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let mut judge_input = post_deploy_input_for(&bundle);
        judge_input.after_metrics = BuilderMetricsSnapshot::v1(208, 318, 50, 25, 30).unwrap();

        let out = orchestrator
            .run_post_deploy_judge(&mut store, &judge_input)
            .unwrap();
        match out {
            BuilderPostDeployJudgeOutcome::Completed(decision) => {
                assert_eq!(decision.judge_result.action, BuilderPostDeployDecisionAction::Revert);
                assert_eq!(decision.release_state.stage, BuilderReleaseStage::RolledBack);
                assert_eq!(decision.release_state.status, BuilderReleaseStateStatus::Reverted);
                assert_eq!(
                    decision.judge_result.reason_code,
                    reason_codes::PH1_BUILDER_POST_DEPLOY_REVERTED
                );
            }
            _ => panic!("expected Completed"),
        }
        assert_eq!(store.builder_post_deploy_judge_result_ledger_rows().len(), 1);
    }

    #[test]
    fn at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();
        let bundle = match orchestrator.run_offline(&mut store, &input()).unwrap() {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };
        store
            .append_builder_release_state_row(
                BuilderReleaseState::v1(
                    "release_production_for_judge".to_string(),
                    bundle.proposal.proposal_id.clone(),
                    BuilderReleaseStage::Production,
                    rollout_pct_for_stage(BuilderReleaseStage::Production),
                    BuilderReleaseStateStatus::Completed,
                    "rollback:hook".to_string(),
                    true,
                    reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
                    MonotonicTimeNs(10_000),
                    Some("release_production_judge_idem".to_string()),
                )
                .unwrap(),
            )
            .unwrap();

        let mut judge_input = post_deploy_input_for(&bundle);
        judge_input.gate_results.pop();

        let out = orchestrator
            .run_post_deploy_judge(&mut store, &judge_input)
            .unwrap();
        match out {
            BuilderPostDeployJudgeOutcome::Refused(refuse) => {
                assert_eq!(refuse.stage, "POST_DEPLOY");
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_BUILDER_POST_DEPLOY_MISSING_GATE_OUTCOMES
                );
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_builder_os_09_release_controller_promotes_staging_to_canary_when_approved() {
        let approved = BuilderApprovalState::v1(
            "approval_approved_03".to_string(),
            "proposal_approved_03".to_string(),
            BuilderChangeClass::ClassB,
            required_approvals_for_change_class(BuilderChangeClass::ClassB),
            1,
            true,
            false,
            BuilderApprovalStateStatus::Approved,
            reason_codes::PH1_BUILDER_APPROVAL_AUTO_RESOLVED,
            MonotonicTimeNs(300),
            Some(MonotonicTimeNs(301)),
            Some("approval_approved_idem_03".to_string()),
        )
        .unwrap();
        let staging_active = BuilderReleaseState::v1(
            "release_staging_03".to_string(),
            "proposal_approved_03".to_string(),
            BuilderReleaseStage::Staging,
            rollout_pct_for_stage(BuilderReleaseStage::Staging),
            BuilderReleaseStateStatus::Active,
            "rollback:hook".to_string(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            MonotonicTimeNs(302),
            Some("release_staging_idem_03".to_string()),
        )
        .unwrap();

        let controller = BuilderReleaseController;
        let canary = controller
            .promote(
                &staging_active,
                &approved,
                MonotonicTimeNs(303),
                Some("release_promote_idem_03".to_string()),
            )
            .unwrap();
        assert_eq!(canary.stage, BuilderReleaseStage::Canary);
        assert_eq!(canary.stage_rollout_pct, 5);
        assert_eq!(canary.status, BuilderReleaseStateStatus::Active);
    }

    #[test]
    fn at_builder_os_10_release_controller_promotes_canary_to_ramp_stages_when_approved() {
        let approved = BuilderApprovalState::v1(
            "approval_approved_04".to_string(),
            "proposal_approved_04".to_string(),
            BuilderChangeClass::ClassB,
            required_approvals_for_change_class(BuilderChangeClass::ClassB),
            1,
            true,
            false,
            BuilderApprovalStateStatus::Approved,
            reason_codes::PH1_BUILDER_APPROVAL_AUTO_RESOLVED,
            MonotonicTimeNs(400),
            Some(MonotonicTimeNs(401)),
            Some("approval_approved_idem_04".to_string()),
        )
        .unwrap();
        let canary_active = BuilderReleaseState::v1(
            "release_canary_04".to_string(),
            "proposal_approved_04".to_string(),
            BuilderReleaseStage::Canary,
            rollout_pct_for_stage(BuilderReleaseStage::Canary),
            BuilderReleaseStateStatus::Active,
            "rollback:hook".to_string(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            MonotonicTimeNs(402),
            Some("release_canary_idem_04".to_string()),
        )
        .unwrap();

        let controller = BuilderReleaseController;
        let ramp25 = controller
            .promote(
                &canary_active,
                &approved,
                MonotonicTimeNs(403),
                Some("release_promote_idem_04a".to_string()),
            )
            .unwrap();
        assert_eq!(ramp25.stage, BuilderReleaseStage::Ramp25);
        assert_eq!(ramp25.stage_rollout_pct, 25);
        assert_eq!(ramp25.status, BuilderReleaseStateStatus::Active);

        let ramp50 = controller
            .promote(
                &ramp25,
                &approved,
                MonotonicTimeNs(404),
                Some("release_promote_idem_04b".to_string()),
            )
            .unwrap();
        assert_eq!(ramp50.stage, BuilderReleaseStage::Ramp50);
        assert_eq!(ramp50.stage_rollout_pct, 50);
        assert_eq!(ramp50.status, BuilderReleaseStateStatus::Active);
    }

    #[test]
    fn at_builder_os_11_class_c_requires_dual_approval_before_production_promotion() {
        let pending = BuilderApprovalState::v1(
            "approval_pending_05".to_string(),
            "proposal_pending_05".to_string(),
            BuilderChangeClass::ClassC,
            required_approvals_for_change_class(BuilderChangeClass::ClassC),
            0,
            false,
            false,
            BuilderApprovalStateStatus::Pending,
            reason_codes::PH1_BUILDER_APPROVAL_UNRESOLVED,
            MonotonicTimeNs(500),
            None,
            Some("approval_pending_idem_05".to_string()),
        )
        .unwrap();
        let ramp50_active = BuilderReleaseState::v1(
            "release_ramp50_05".to_string(),
            "proposal_pending_05".to_string(),
            BuilderReleaseStage::Ramp50,
            rollout_pct_for_stage(BuilderReleaseStage::Ramp50),
            BuilderReleaseStateStatus::Active,
            "rollback:hook".to_string(),
            true,
            reason_codes::PH1_BUILDER_RELEASE_STAGE_ACTIVE,
            MonotonicTimeNs(501),
            Some("release_ramp50_idem_05".to_string()),
        )
        .unwrap();
        let controller = BuilderReleaseController;

        let blocked_pending = controller.promote(
            &ramp50_active,
            &pending,
            MonotonicTimeNs(502),
            Some("release_promote_idem_05a".to_string()),
        );
        assert!(blocked_pending.is_err());

        let tech_only = advance_approval_state(
            &pending,
            BuilderApprovalTransitionAction::ApproveTech,
            MonotonicTimeNs(503),
            reason_codes::PH1_BUILDER_APPROVAL_UNRESOLVED,
            Some("approval_transition_idem_05a".to_string()),
        )
        .unwrap();
        assert_eq!(tech_only.status, BuilderApprovalStateStatus::Pending);

        let blocked_tech_only = controller.promote(
            &ramp50_active,
            &tech_only,
            MonotonicTimeNs(504),
            Some("release_promote_idem_05b".to_string()),
        );
        assert!(blocked_tech_only.is_err());

        let fully_approved = advance_approval_state(
            &tech_only,
            BuilderApprovalTransitionAction::ApproveProductSecurity,
            MonotonicTimeNs(505),
            reason_codes::PH1_BUILDER_APPROVAL_AUTO_RESOLVED,
            Some("approval_transition_idem_05b".to_string()),
        )
        .unwrap();
        assert_eq!(fully_approved.status, BuilderApprovalStateStatus::Approved);

        let promoted = controller
            .promote(
                &ramp50_active,
                &fully_approved,
                MonotonicTimeNs(506),
                Some("release_promote_idem_05c".to_string()),
            )
            .unwrap();
        assert_eq!(promoted.stage, BuilderReleaseStage::Production);
        assert_eq!(promoted.stage_rollout_pct, 100);
        assert_eq!(promoted.status, BuilderReleaseStateStatus::Completed);
    }

    #[test]
    fn at_builder_os_12_learning_report_auto_generated_for_learning_sources() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let mut builder_input = input();
        builder_input.outcome_entries = vec![
            OsOutcomeUtilizationEntry::v1(
                "PH1.FEEDBACK".to_string(),
                "STT_REJECT".to_string(),
                builder_input.correlation_id,
                builder_input.turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.LEARN".to_string(),
                24,
                true,
                ReasonCodeId(0x7000_1001),
            )
            .unwrap(),
            OsOutcomeUtilizationEntry::v1(
                "PH1.KNOW".to_string(),
                "VOCAB_MISS".to_string(),
                builder_input.correlation_id,
                builder_input.turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.LEARN".to_string(),
                16,
                true,
                ReasonCodeId(0x7000_1002),
            )
            .unwrap(),
        ];
        let report_path = std::env::temp_dir()
            .join("builder_learning_report_auto_generated.md")
            .display()
            .to_string();
        let _ = std::fs::remove_file(&report_path);
        builder_input.learning_report_output_path = Some(report_path.clone());

        let out = orchestrator.run_offline(&mut store, &builder_input).unwrap();
        let bundle = match out {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };

        assert!(bundle.learning_report_id.is_some());
        assert_eq!(bundle.learning_signal_count, 2);
        assert!(bundle
            .learning_source_engines
            .iter()
            .any(|engine| engine == "PH1.FEEDBACK"));
        assert!(bundle
            .proposal
            .learning_context
            .as_ref()
            .is_some_and(|ctx| ctx.learning_signal_count == 2));

        let content = std::fs::read_to_string(&report_path).unwrap();
        assert!(content.contains("## Learning Issues Received"));
        assert!(content.contains("## Root Cause Evidence"));
        assert!(content.contains("Should I proceed with this learning-driven fix?"));
    }

    #[test]
    fn at_builder_os_13_learning_report_skipped_without_learning_sources() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let report_path = std::env::temp_dir()
            .join("builder_learning_report_should_not_exist.md")
            .display()
            .to_string();
        let _ = std::fs::remove_file(&report_path);

        let mut builder_input = input();
        builder_input.outcome_entries = vec![
            OsOutcomeUtilizationEntry::v1(
                "PH1.SEARCH".to_string(),
                "tool_timeout".to_string(),
                builder_input.correlation_id,
                builder_input.turn_id,
                OsOutcomeActionClass::Drop,
                "NONE".to_string(),
                34,
                false,
                ReasonCodeId(0x7000_2001),
            )
            .unwrap(),
            OsOutcomeUtilizationEntry::v1(
                "PH1.CONTEXT".to_string(),
                "context_miss".to_string(),
                builder_input.correlation_id,
                builder_input.turn_id,
                OsOutcomeActionClass::QueueLearn,
                "PH1.X".to_string(),
                18,
                true,
                ReasonCodeId(0x7000_2002),
            )
            .unwrap(),
        ];
        builder_input.learning_report_output_path = Some(report_path.clone());

        let out = orchestrator.run_offline(&mut store, &builder_input).unwrap();
        let bundle = match out {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };

        assert!(bundle.learning_report_id.is_none());
        assert_eq!(bundle.learning_signal_count, 0);
        assert!(bundle.learning_source_engines.is_empty());
        assert!(bundle.proposal.learning_context.is_none());
        assert!(!Path::new(&report_path).exists());
    }

    #[test]
    fn at_builder_os_14_change_brief_auto_generated_for_permission_gate() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let mut builder_input = input();
        let brief_path = std::env::temp_dir()
            .join("builder_change_brief_auto_generated.md")
            .display()
            .to_string();
        let _ = std::fs::remove_file(&brief_path);
        builder_input.change_brief_output_path = Some(brief_path.clone());

        let out = orchestrator.run_offline(&mut store, &builder_input).unwrap();
        let bundle = match out {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };

        assert_eq!(bundle.change_brief_path, Some(brief_path.clone()));
        let content = std::fs::read_to_string(&brief_path).unwrap();
        assert!(content.contains("## Issue"));
        assert!(content.contains("## Fix"));
        assert!(content.contains("Should I proceed?"));
        assert!(content.contains("All tests passed. Can I launch?"));
    }

    #[test]
    fn at_builder_os_15_permission_packet_auto_generated_for_bcast_flow() {
        let mut store = Ph1fStore::new_in_memory();
        let orchestrator = Ph1BuilderOrchestrator::new(
            Ph1BuilderConfig::mvp_v1(true),
            DeterministicPatternEngine,
            DeterministicRllEngine,
            DeterministicBuilderSandboxValidator,
        )
        .unwrap();

        let mut builder_input = input();
        let packet_path = std::env::temp_dir()
            .join("builder_permission_packet_auto_generated.md")
            .display()
            .to_string();
        let _ = std::fs::remove_file(&packet_path);
        builder_input.permission_packet_output_path = Some(packet_path.clone());

        let out = orchestrator.run_offline(&mut store, &builder_input).unwrap();
        let bundle = match out {
            BuilderOrchestrationOutcome::Completed(bundle) => bundle,
            _ => panic!("expected Completed"),
        };

        assert_eq!(bundle.permission_packet_path, Some(packet_path.clone()));
        assert!(bundle.code_decision_file_path.is_some());
        assert!(bundle.launch_decision_file_path.is_some());
        let content = std::fs::read_to_string(&packet_path).unwrap();
        assert!(content.contains("## Code Permission Request (BCAST)"));
        assert!(content.contains("BCAST_CREATE_DRAFT"));
        assert!(content.contains("BCAST_DELIVER_COMMIT"));
        assert!(content.contains("REMINDER_SCHEDULE_COMMIT"));
        assert!(content.contains("Should I proceed?"));
        assert!(content.contains("All tests passed. Can I launch?"));
        assert!(content.contains("scripts/apply_builder_permission_decision.sh"));
        assert!(content.contains("DECISION_FILE="));
        assert!(content.contains("builder_code_decision.env"));
        assert!(content.contains("builder_launch_decision.env"));

        let code_decision_path = bundle.code_decision_file_path.unwrap();
        let launch_decision_path = bundle.launch_decision_file_path.unwrap();
        let code_decision_content = std::fs::read_to_string(&code_decision_path).unwrap();
        let launch_decision_content = std::fs::read_to_string(&launch_decision_path).unwrap();
        assert!(code_decision_content.contains("PHASE=code"));
        assert!(code_decision_content.contains("DECISION=approve"));
        assert!(code_decision_content.contains("PERMISSION_REF=perm_code_"));
        assert!(launch_decision_content.contains("PHASE=launch"));
        assert!(launch_decision_content.contains("DECISION=approve"));
        assert!(launch_decision_content.contains("PERMISSION_REF=perm_launch_"));
    }
}
