#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1BUILDER_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderChangeClass {
    ClassA,
    ClassB,
    ClassC,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderProposalStatus {
    Draft,
    Validated,
    Approved,
    Released,
    Reverted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderValidationRunStatus {
    Running,
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum BuilderValidationGateId {
    BldG1,
    BldG2,
    BldG3,
    BldG4,
    BldG5,
    BldG6,
    BldG7,
    BldG8,
    BldG9,
    BldG10,
}

impl BuilderValidationGateId {
    pub fn as_str(self) -> &'static str {
        match self {
            BuilderValidationGateId::BldG1 => "BLD-G1",
            BuilderValidationGateId::BldG2 => "BLD-G2",
            BuilderValidationGateId::BldG3 => "BLD-G3",
            BuilderValidationGateId::BldG4 => "BLD-G4",
            BuilderValidationGateId::BldG5 => "BLD-G5",
            BuilderValidationGateId::BldG6 => "BLD-G6",
            BuilderValidationGateId::BldG7 => "BLD-G7",
            BuilderValidationGateId::BldG8 => "BLD-G8",
            BuilderValidationGateId::BldG9 => "BLD-G9",
            BuilderValidationGateId::BldG10 => "BLD-G10",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderSignalWindow {
    pub schema_version: SchemaVersion,
    pub start_at: MonotonicTimeNs,
    pub end_at: MonotonicTimeNs,
    pub signal_count: u32,
}

impl BuilderSignalWindow {
    pub fn v1(
        start_at: MonotonicTimeNs,
        end_at: MonotonicTimeNs,
        signal_count: u32,
    ) -> Result<Self, ContractViolation> {
        let window = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            start_at,
            end_at,
            signal_count,
        };
        window.validate()?;
        Ok(window)
    }
}

impl Validate for BuilderSignalWindow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_signal_window.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        if self.end_at.0 < self.start_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_signal_window.end_at",
                reason: "must be >= start_at",
            });
        }
        if self.signal_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_signal_window.signal_count",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderExpectedEffect {
    pub schema_version: SchemaVersion,
    pub latency_p95_delta_bp: i16,
    pub latency_p99_delta_bp: i16,
    pub quality_delta_bp: i16,
    pub safety_delta_bp: i16,
}

impl BuilderExpectedEffect {
    pub fn v1(
        latency_p95_delta_bp: i16,
        latency_p99_delta_bp: i16,
        quality_delta_bp: i16,
        safety_delta_bp: i16,
    ) -> Result<Self, ContractViolation> {
        let expected = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            latency_p95_delta_bp,
            latency_p99_delta_bp,
            quality_delta_bp,
            safety_delta_bp,
        };
        expected.validate()?;
        Ok(expected)
    }
}

impl Validate for BuilderExpectedEffect {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_expected_effect.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_bp(
            "builder_expected_effect.latency_p95_delta_bp",
            self.latency_p95_delta_bp,
        )?;
        validate_bp(
            "builder_expected_effect.latency_p99_delta_bp",
            self.latency_p99_delta_bp,
        )?;
        validate_bp("builder_expected_effect.quality_delta_bp", self.quality_delta_bp)?;
        validate_bp("builder_expected_effect.safety_delta_bp", self.safety_delta_bp)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderLearningContext {
    pub schema_version: SchemaVersion,
    pub learning_report_id: String,
    pub source_engines: Vec<String>,
    pub learning_signal_count: u32,
    pub evidence_refs: Vec<String>,
}

impl BuilderLearningContext {
    pub fn v1(
        learning_report_id: String,
        source_engines: Vec<String>,
        learning_signal_count: u32,
        evidence_refs: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let context = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            learning_report_id,
            source_engines,
            learning_signal_count,
            evidence_refs,
        };
        context.validate()?;
        Ok(context)
    }
}

impl Validate for BuilderLearningContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token(
            "builder_learning_context.learning_report_id",
            &self.learning_report_id,
            128,
        )?;
        if self.source_engines.is_empty() || self.source_engines.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.source_engines",
                reason: "must be within 1..=16",
            });
        }
        let mut source_set = BTreeSet::new();
        for engine in &self.source_engines {
            validate_engine_id("builder_learning_context.source_engines", engine, 64)?;
            source_set.insert(engine);
        }
        if source_set.len() != self.source_engines.len() {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.source_engines",
                reason: "must not contain duplicates",
            });
        }
        if self.learning_signal_count == 0 || self.learning_signal_count > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.learning_signal_count",
                reason: "must be within 1..=10000",
            });
        }
        if self.evidence_refs.is_empty() || self.evidence_refs.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.evidence_refs",
                reason: "must be within 1..=64",
            });
        }
        let mut evidence_set = BTreeSet::new();
        for evidence_ref in &self.evidence_refs {
            validate_ascii_text(
                "builder_learning_context.evidence_refs",
                evidence_ref,
                256,
            )?;
            evidence_set.insert(evidence_ref);
        }
        if evidence_set.len() != self.evidence_refs.len() {
            return Err(ContractViolation::InvalidValue {
                field: "builder_learning_context.evidence_refs",
                reason: "must not contain duplicates",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderPatchProposal {
    pub schema_version: SchemaVersion,
    pub proposal_id: String,
    pub created_at: MonotonicTimeNs,
    pub source_signal_window: BuilderSignalWindow,
    pub source_signal_hash: String,
    pub learning_context: Option<BuilderLearningContext>,
    pub target_files: Vec<String>,
    pub change_class: BuilderChangeClass,
    pub risk_score_bp: u16,
    pub expected_effect: BuilderExpectedEffect,
    pub validation_plan: String,
    pub rollback_plan: String,
    pub status: BuilderProposalStatus,
}

impl BuilderPatchProposal {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        proposal_id: String,
        created_at: MonotonicTimeNs,
        source_signal_window: BuilderSignalWindow,
        source_signal_hash: String,
        target_files: Vec<String>,
        change_class: BuilderChangeClass,
        risk_score_bp: u16,
        expected_effect: BuilderExpectedEffect,
        validation_plan: String,
        rollback_plan: String,
        status: BuilderProposalStatus,
    ) -> Result<Self, ContractViolation> {
        let proposal = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            proposal_id,
            created_at,
            source_signal_window,
            source_signal_hash,
            learning_context: None,
            target_files,
            change_class,
            risk_score_bp,
            expected_effect,
            validation_plan,
            rollback_plan,
            status,
        };
        proposal.validate()?;
        Ok(proposal)
    }

    pub fn with_learning_context(
        mut self,
        learning_context: BuilderLearningContext,
    ) -> Result<Self, ContractViolation> {
        learning_context.validate()?;
        self.learning_context = Some(learning_context);
        self.validate()?;
        Ok(self)
    }
}

impl Validate for BuilderPatchProposal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_patch_proposal.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token("builder_patch_proposal.proposal_id", &self.proposal_id, 96)?;
        self.source_signal_window.validate()?;
        validate_token(
            "builder_patch_proposal.source_signal_hash",
            &self.source_signal_hash,
            128,
        )?;
        if let Some(learning_context) = &self.learning_context {
            learning_context.validate()?;
            if learning_context.learning_signal_count > self.source_signal_window.signal_count {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_patch_proposal.learning_context.learning_signal_count",
                    reason: "must be <= source_signal_window.signal_count",
                });
            }
        }
        if self.target_files.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "builder_patch_proposal.target_files",
                reason: "must not be empty",
            });
        }
        if self.target_files.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_patch_proposal.target_files",
                reason: "must be <= 256",
            });
        }
        for path in &self.target_files {
            validate_path("builder_patch_proposal.target_files", path, 256)?;
        }
        if self.risk_score_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_patch_proposal.risk_score_bp",
                reason: "must be within 0..=10000",
            });
        }
        self.expected_effect.validate()?;
        validate_ascii_text(
            "builder_patch_proposal.validation_plan",
            &self.validation_plan,
            2048,
        )?;
        validate_ascii_text(
            "builder_patch_proposal.rollback_plan",
            &self.rollback_plan,
            2048,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderValidationRun {
    pub schema_version: SchemaVersion,
    pub run_id: String,
    pub proposal_id: String,
    pub started_at: MonotonicTimeNs,
    pub finished_at: Option<MonotonicTimeNs>,
    pub status: BuilderValidationRunStatus,
    pub gate_count_expected: u8,
    pub gate_count_recorded: u8,
    pub idempotency_key: Option<String>,
}

impl BuilderValidationRun {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        run_id: String,
        proposal_id: String,
        started_at: MonotonicTimeNs,
        finished_at: Option<MonotonicTimeNs>,
        status: BuilderValidationRunStatus,
        gate_count_expected: u8,
        gate_count_recorded: u8,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let run = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            run_id,
            proposal_id,
            started_at,
            finished_at,
            status,
            gate_count_expected,
            gate_count_recorded,
            idempotency_key,
        };
        run.validate()?;
        Ok(run)
    }
}

impl Validate for BuilderValidationRun {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_validation_run.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token("builder_validation_run.run_id", &self.run_id, 96)?;
        validate_token("builder_validation_run.proposal_id", &self.proposal_id, 96)?;
        if self.gate_count_expected == 0 || self.gate_count_expected > 10 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_validation_run.gate_count_expected",
                reason: "must be within 1..=10",
            });
        }
        if self.gate_count_recorded > self.gate_count_expected {
            return Err(ContractViolation::InvalidValue {
                field: "builder_validation_run.gate_count_recorded",
                reason: "must be <= gate_count_expected",
            });
        }
        if let Some(finished_at) = self.finished_at {
            if finished_at.0 < self.started_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_validation_run.finished_at",
                    reason: "must be >= started_at",
                });
            }
        }
        match self.status {
            BuilderValidationRunStatus::Running => {
                if self.finished_at.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_validation_run.finished_at",
                        reason: "must be absent when status=RUNNING",
                    });
                }
            }
            BuilderValidationRunStatus::Passed | BuilderValidationRunStatus::Failed => {
                if self.finished_at.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_validation_run.finished_at",
                        reason: "must be present when status is terminal",
                    });
                }
                if self.gate_count_recorded != self.gate_count_expected {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_validation_run.gate_count_recorded",
                        reason: "must equal gate_count_expected when status is terminal",
                    });
                }
            }
        }
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token(
                "builder_validation_run.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderValidationGateResult {
    pub schema_version: SchemaVersion,
    pub run_id: String,
    pub proposal_id: String,
    pub gate_id: BuilderValidationGateId,
    pub passed: bool,
    pub recorded_at: MonotonicTimeNs,
    pub reason_code: ReasonCodeId,
    pub detail: String,
    pub idempotency_key: Option<String>,
}

impl BuilderValidationGateResult {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        run_id: String,
        proposal_id: String,
        gate_id: BuilderValidationGateId,
        passed: bool,
        recorded_at: MonotonicTimeNs,
        reason_code: ReasonCodeId,
        detail: String,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let result = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            run_id,
            proposal_id,
            gate_id,
            passed,
            recorded_at,
            reason_code,
            detail,
            idempotency_key,
        };
        result.validate()?;
        Ok(result)
    }
}

impl Validate for BuilderValidationGateResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_validation_gate_result.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token("builder_validation_gate_result.run_id", &self.run_id, 96)?;
        validate_token(
            "builder_validation_gate_result.proposal_id",
            &self.proposal_id,
            96,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_validation_gate_result.reason_code",
                reason: "must be non-zero",
            });
        }
        validate_ascii_text(
            "builder_validation_gate_result.detail",
            &self.detail,
            1024,
        )?;
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token(
                "builder_validation_gate_result.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderApprovalStateStatus {
    Pending,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderApprovalState {
    pub schema_version: SchemaVersion,
    pub approval_state_id: String,
    pub proposal_id: String,
    pub change_class: BuilderChangeClass,
    pub required_approvals_total: u8,
    pub approvals_granted: u8,
    pub tech_approved: bool,
    pub product_security_approved: bool,
    pub status: BuilderApprovalStateStatus,
    pub reason_code: ReasonCodeId,
    pub recorded_at: MonotonicTimeNs,
    pub resolved_at: Option<MonotonicTimeNs>,
    pub idempotency_key: Option<String>,
}

impl BuilderApprovalState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        approval_state_id: String,
        proposal_id: String,
        change_class: BuilderChangeClass,
        required_approvals_total: u8,
        approvals_granted: u8,
        tech_approved: bool,
        product_security_approved: bool,
        status: BuilderApprovalStateStatus,
        reason_code: ReasonCodeId,
        recorded_at: MonotonicTimeNs,
        resolved_at: Option<MonotonicTimeNs>,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            approval_state_id,
            proposal_id,
            change_class,
            required_approvals_total,
            approvals_granted,
            tech_approved,
            product_security_approved,
            status,
            reason_code,
            recorded_at,
            resolved_at,
            idempotency_key,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for BuilderApprovalState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_approval_state.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token(
            "builder_approval_state.approval_state_id",
            &self.approval_state_id,
            96,
        )?;
        validate_token("builder_approval_state.proposal_id", &self.proposal_id, 96)?;
        let expected_required = required_approvals_for_change_class(self.change_class);
        if self.required_approvals_total != expected_required {
            return Err(ContractViolation::InvalidValue {
                field: "builder_approval_state.required_approvals_total",
                reason: "must match class-based approval requirement",
            });
        }
        let derived_granted = (u8::from(self.tech_approved)) + (u8::from(self.product_security_approved));
        if self.approvals_granted != derived_granted {
            return Err(ContractViolation::InvalidValue {
                field: "builder_approval_state.approvals_granted",
                reason: "must equal derived approval flags",
            });
        }
        if self.approvals_granted > self.required_approvals_total {
            return Err(ContractViolation::InvalidValue {
                field: "builder_approval_state.approvals_granted",
                reason: "must be <= required_approvals_total",
            });
        }
        match self.change_class {
            BuilderChangeClass::ClassA => {
                if self.tech_approved || self.product_security_approved {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.change_class",
                        reason: "CLASS_A must not carry human approval flags",
                    });
                }
            }
            BuilderChangeClass::ClassB => {
                if self.product_security_approved {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.product_security_approved",
                        reason: "CLASS_B must not require product/security approval",
                    });
                }
            }
            BuilderChangeClass::ClassC => {}
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_approval_state.reason_code",
                reason: "must be non-zero",
            });
        }
        match self.status {
            BuilderApprovalStateStatus::Pending => {
                if self.approvals_granted >= self.required_approvals_total {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.status",
                        reason: "PENDING requires missing approvals",
                    });
                }
                if self.resolved_at.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.resolved_at",
                        reason: "must be absent when status=PENDING",
                    });
                }
            }
            BuilderApprovalStateStatus::Approved => {
                if self.approvals_granted != self.required_approvals_total {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.status",
                        reason: "APPROVED requires all class-based approvals",
                    });
                }
                if self.resolved_at.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.resolved_at",
                        reason: "must be present when status=APPROVED",
                    });
                }
            }
            BuilderApprovalStateStatus::Rejected => {
                if self.resolved_at.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_approval_state.resolved_at",
                        reason: "must be present when status=REJECTED",
                    });
                }
            }
        }
        if let Some(resolved_at) = self.resolved_at {
            if resolved_at.0 < self.recorded_at.0 {
                return Err(ContractViolation::InvalidValue {
                    field: "builder_approval_state.resolved_at",
                    reason: "must be >= recorded_at",
                });
            }
        }
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token(
                "builder_approval_state.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderReleaseStage {
    Staging,
    Canary,
    Ramp25,
    Ramp50,
    Production,
    RolledBack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderReleaseStateStatus {
    Pending,
    Active,
    Blocked,
    Completed,
    Reverted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderReleaseState {
    pub schema_version: SchemaVersion,
    pub release_state_id: String,
    pub proposal_id: String,
    pub stage: BuilderReleaseStage,
    pub stage_rollout_pct: u8,
    pub status: BuilderReleaseStateStatus,
    pub rollback_hook: String,
    pub rollback_ready: bool,
    pub reason_code: ReasonCodeId,
    pub recorded_at: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
}

impl BuilderReleaseState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        release_state_id: String,
        proposal_id: String,
        stage: BuilderReleaseStage,
        stage_rollout_pct: u8,
        status: BuilderReleaseStateStatus,
        rollback_hook: String,
        rollback_ready: bool,
        reason_code: ReasonCodeId,
        recorded_at: MonotonicTimeNs,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            release_state_id,
            proposal_id,
            stage,
            stage_rollout_pct,
            status,
            rollback_hook,
            rollback_ready,
            reason_code,
            recorded_at,
            idempotency_key,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for BuilderReleaseState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_release_state.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token(
            "builder_release_state.release_state_id",
            &self.release_state_id,
            96,
        )?;
        validate_token("builder_release_state.proposal_id", &self.proposal_id, 96)?;
        if self.stage_rollout_pct != rollout_pct_for_stage(self.stage) {
            return Err(ContractViolation::InvalidValue {
                field: "builder_release_state.stage_rollout_pct",
                reason: "must match deterministic rollout percentage for stage",
            });
        }
        validate_ascii_text(
            "builder_release_state.rollback_hook",
            &self.rollback_hook,
            256,
        )?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_release_state.reason_code",
                reason: "must be non-zero",
            });
        }

        match self.status {
            BuilderReleaseStateStatus::Pending => {
                if self.stage != BuilderReleaseStage::Staging {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.status",
                        reason: "PENDING is allowed only at STAGING",
                    });
                }
            }
            BuilderReleaseStateStatus::Active => {
                if matches!(
                    self.stage,
                    BuilderReleaseStage::Production | BuilderReleaseStage::RolledBack
                ) {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.status",
                        reason: "ACTIVE cannot be PRODUCTION or ROLLED_BACK",
                    });
                }
            }
            BuilderReleaseStateStatus::Blocked => {
                if self.stage == BuilderReleaseStage::Production {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.status",
                        reason: "PRODUCTION cannot be BLOCKED",
                    });
                }
            }
            BuilderReleaseStateStatus::Completed => {
                if self.stage != BuilderReleaseStage::Production {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.status",
                        reason: "COMPLETED requires PRODUCTION stage",
                    });
                }
            }
            BuilderReleaseStateStatus::Reverted => {
                if self.stage != BuilderReleaseStage::RolledBack {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.status",
                        reason: "REVERTED requires ROLLED_BACK stage",
                    });
                }
                if !self.rollback_ready {
                    return Err(ContractViolation::InvalidValue {
                        field: "builder_release_state.rollback_ready",
                        reason: "must be true for REVERTED stage",
                    });
                }
            }
        }

        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token(
                "builder_release_state.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuilderPostDeployDecisionAction {
    Accept,
    Revert,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderMetricsSnapshot {
    pub schema_version: SchemaVersion,
    pub latency_p95_ms: u32,
    pub latency_p99_ms: u32,
    pub fail_closed_rate_bp: u16,
    pub critical_reason_spike_bp: i16,
    pub observation_window_minutes: u16,
}

impl BuilderMetricsSnapshot {
    pub fn v1(
        latency_p95_ms: u32,
        latency_p99_ms: u32,
        fail_closed_rate_bp: u16,
        critical_reason_spike_bp: i16,
        observation_window_minutes: u16,
    ) -> Result<Self, ContractViolation> {
        let snapshot = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            latency_p95_ms,
            latency_p99_ms,
            fail_closed_rate_bp,
            critical_reason_spike_bp,
            observation_window_minutes,
        };
        snapshot.validate()?;
        Ok(snapshot)
    }
}

impl Validate for BuilderMetricsSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_metrics_snapshot.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        if self.latency_p95_ms == 0 || self.latency_p99_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_metrics_snapshot.latency",
                reason: "latency values must be > 0",
            });
        }
        if self.fail_closed_rate_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_metrics_snapshot.fail_closed_rate_bp",
                reason: "must be within 0..=10000",
            });
        }
        if !(-10_000..=10_000).contains(&self.critical_reason_spike_bp) {
            return Err(ContractViolation::InvalidValue {
                field: "builder_metrics_snapshot.critical_reason_spike_bp",
                reason: "must be within -10000..=10000",
            });
        }
        if self.observation_window_minutes == 0 || self.observation_window_minutes > 1440 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_metrics_snapshot.observation_window_minutes",
                reason: "must be within 1..=1440",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuilderPostDeployJudgeResult {
    pub schema_version: SchemaVersion,
    pub judge_result_id: String,
    pub proposal_id: String,
    pub release_state_id: String,
    pub before: BuilderMetricsSnapshot,
    pub after: BuilderMetricsSnapshot,
    pub action: BuilderPostDeployDecisionAction,
    pub reason_code: ReasonCodeId,
    pub recorded_at: MonotonicTimeNs,
    pub idempotency_key: Option<String>,
}

impl BuilderPostDeployJudgeResult {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        judge_result_id: String,
        proposal_id: String,
        release_state_id: String,
        before: BuilderMetricsSnapshot,
        after: BuilderMetricsSnapshot,
        action: BuilderPostDeployDecisionAction,
        reason_code: ReasonCodeId,
        recorded_at: MonotonicTimeNs,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let result = Self {
            schema_version: PH1BUILDER_CONTRACT_VERSION,
            judge_result_id,
            proposal_id,
            release_state_id,
            before,
            after,
            action,
            reason_code,
            recorded_at,
            idempotency_key,
        };
        result.validate()?;
        Ok(result)
    }
}

impl Validate for BuilderPostDeployJudgeResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1BUILDER_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "builder_post_deploy_judge_result.schema_version",
                reason: "must match PH1BUILDER_CONTRACT_VERSION",
            });
        }
        validate_token(
            "builder_post_deploy_judge_result.judge_result_id",
            &self.judge_result_id,
            96,
        )?;
        validate_token(
            "builder_post_deploy_judge_result.proposal_id",
            &self.proposal_id,
            96,
        )?;
        validate_token(
            "builder_post_deploy_judge_result.release_state_id",
            &self.release_state_id,
            96,
        )?;
        self.before.validate()?;
        self.after.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "builder_post_deploy_judge_result.reason_code",
                reason: "must be non-zero",
            });
        }
        if let Some(idempotency_key) = &self.idempotency_key {
            validate_token(
                "builder_post_deploy_judge_result.idempotency_key",
                idempotency_key,
                128,
            )?;
        }
        Ok(())
    }
}

pub fn required_approvals_for_change_class(change_class: BuilderChangeClass) -> u8 {
    match change_class {
        BuilderChangeClass::ClassA => 0,
        BuilderChangeClass::ClassB => 1,
        BuilderChangeClass::ClassC => 2,
    }
}

pub fn rollout_pct_for_stage(stage: BuilderReleaseStage) -> u8 {
    match stage {
        BuilderReleaseStage::Staging => 0,
        BuilderReleaseStage::Canary => 5,
        BuilderReleaseStage::Ramp25 => 25,
        BuilderReleaseStage::Ramp50 => 50,
        BuilderReleaseStage::Production => 100,
        BuilderReleaseStage::RolledBack => 0,
    }
}

fn validate_token(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
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
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be token-safe ASCII",
        });
    }
    Ok(())
}

fn validate_path(field: &'static str, value: &str, max_len: usize) -> Result<(), ContractViolation> {
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

fn validate_engine_id(
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
            reason: "must be ASCII engine-id safe",
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

fn validate_bp(field: &'static str, value: i16) -> Result<(), ContractViolation> {
    if !(-10_000..=10_000).contains(&value) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be within -10000..=10000 basis points",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn window() -> BuilderSignalWindow {
        BuilderSignalWindow::v1(MonotonicTimeNs(10), MonotonicTimeNs(20), 8).unwrap()
    }

    fn effect() -> BuilderExpectedEffect {
        BuilderExpectedEffect::v1(-120, -160, 220, 0).unwrap()
    }

    #[test]
    fn at_builder_01_proposal_accepts_mandatory_contract_fields() {
        let proposal = BuilderPatchProposal::v1(
            "proposal_01".to_string(),
            MonotonicTimeNs(100),
            window(),
            "sig_hash_01".to_string(),
            vec![
                "crates/selene_os/src/ph1os.rs".to_string(),
                "crates/selene_storage/src/ph1f.rs".to_string(),
            ],
            BuilderChangeClass::ClassB,
            3200,
            effect(),
            "compile + test + guardrails".to_string(),
            "revert commit and restore previous policy pack".to_string(),
            BuilderProposalStatus::Draft,
        )
        .unwrap();
        assert_eq!(proposal.proposal_id, "proposal_01");
        assert_eq!(proposal.risk_score_bp, 3200);
        assert!(proposal.learning_context.is_none());
    }

    #[test]
    fn at_builder_02_proposal_rejects_out_of_range_risk_score() {
        let res = BuilderPatchProposal::v1(
            "proposal_bad".to_string(),
            MonotonicTimeNs(100),
            window(),
            "sig_hash_02".to_string(),
            vec!["docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md".to_string()],
            BuilderChangeClass::ClassA,
            10_001,
            effect(),
            "validate".to_string(),
            "rollback".to_string(),
            BuilderProposalStatus::Draft,
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_10_proposal_accepts_learning_context_when_evidence_backed() {
        let proposal = BuilderPatchProposal::v1(
            "proposal_learning_01".to_string(),
            MonotonicTimeNs(100),
            BuilderSignalWindow::v1(MonotonicTimeNs(10), MonotonicTimeNs(20), 4).unwrap(),
            "sig_hash_learning_01".to_string(),
            vec!["crates/selene_os/src/ph1builder.rs".to_string()],
            BuilderChangeClass::ClassA,
            1800,
            BuilderExpectedEffect::v1(-80, -100, 120, 0).unwrap(),
            "compile + test".to_string(),
            "revert patch".to_string(),
            BuilderProposalStatus::Draft,
        )
        .unwrap()
        .with_learning_context(
            BuilderLearningContext::v1(
                "learn_report_01".to_string(),
                vec!["PH1.FEEDBACK".to_string(), "PH1.LEARN".to_string()],
                2,
                vec![
                    "evidence_ref:9200:2:PH1.FEEDBACK:STT_REJECT".to_string(),
                    "evidence_ref:9200:2:PH1.LEARN:CLARIFY_LOOP".to_string(),
                ],
            )
            .unwrap(),
        )
        .unwrap();

        assert!(proposal.learning_context.is_some());
    }

    #[test]
    fn at_builder_11_learning_context_rejects_missing_evidence_refs() {
        let res = BuilderLearningContext::v1(
            "learn_report_02".to_string(),
            vec!["PH1.FEEDBACK".to_string()],
            1,
            Vec::new(),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_03_validation_run_terminal_requires_full_gate_count() {
        let res = BuilderValidationRun::v1(
            "run_01".to_string(),
            "proposal_01".to_string(),
            MonotonicTimeNs(100),
            Some(MonotonicTimeNs(150)),
            BuilderValidationRunStatus::Passed,
            10,
            9,
            Some("run_key_01".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_04_gate_result_requires_non_zero_reason_code() {
        let res = BuilderValidationGateResult::v1(
            "run_01".to_string(),
            "proposal_01".to_string(),
            BuilderValidationGateId::BldG1,
            true,
            MonotonicTimeNs(101),
            ReasonCodeId(0),
            "diff applied cleanly".to_string(),
            Some("gate_key_01".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_05_approval_state_enforces_class_requirements() {
        let res = BuilderApprovalState::v1(
            "approval_01".to_string(),
            "proposal_01".to_string(),
            BuilderChangeClass::ClassC,
            2,
            1,
            true,
            false,
            BuilderApprovalStateStatus::Approved,
            ReasonCodeId(0xB1D0_0101),
            MonotonicTimeNs(200),
            Some(MonotonicTimeNs(201)),
            Some("approval_idem_01".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_06_release_state_requires_stage_pct_alignment() {
        let res = BuilderReleaseState::v1(
            "release_01".to_string(),
            "proposal_01".to_string(),
            BuilderReleaseStage::Canary,
            25,
            BuilderReleaseStateStatus::Active,
            "rollback_hook_ref".to_string(),
            true,
            ReasonCodeId(0xB1D0_0102),
            MonotonicTimeNs(300),
            Some("release_idem_01".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_07_release_state_blocks_production_without_completed_status() {
        let res = BuilderReleaseState::v1(
            "release_02".to_string(),
            "proposal_01".to_string(),
            BuilderReleaseStage::Production,
            100,
            BuilderReleaseStateStatus::Active,
            "rollback_hook_ref".to_string(),
            true,
            ReasonCodeId(0xB1D0_0103),
            MonotonicTimeNs(301),
            Some("release_idem_02".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_08_metrics_snapshot_rejects_invalid_fail_closed_rate() {
        let res = BuilderMetricsSnapshot::v1(200, 300, 10_001, 0, 30);
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }

    #[test]
    fn at_builder_09_post_deploy_judge_result_requires_non_zero_reason_code() {
        let before = BuilderMetricsSnapshot::v1(180, 260, 40, 0, 30).unwrap();
        let after = BuilderMetricsSnapshot::v1(184, 268, 45, 10, 30).unwrap();
        let res = BuilderPostDeployJudgeResult::v1(
            "judge_01".to_string(),
            "proposal_01".to_string(),
            "release_01".to_string(),
            before,
            after,
            BuilderPostDeployDecisionAction::Accept,
            ReasonCodeId(0),
            MonotonicTimeNs(400),
            Some("judge_idem_01".to_string()),
        );
        assert!(matches!(res, Err(ContractViolation::InvalidValue { .. })));
    }
}
