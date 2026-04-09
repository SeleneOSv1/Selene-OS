#![forbid(unsafe_code)]

use crate::ph1builder::{BuilderApprovalState, BuilderPostDeployJudgeResult, BuilderReleaseState};
use crate::ph1learn::LearnArtifactPackageBuildOk;
use crate::ph1selfheal::{FixCard, PromotionDecision};
use crate::{ContractViolation, MonotonicTimeNs, Validate};

fn validate_ascii_token(
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

fn validate_optional_ascii_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value.as_ref() {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

fn validate_ascii_token_vec(
    field: &'static str,
    values: &[String],
    max_len: usize,
    max_items: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "too many entries",
        });
    }
    for value in values {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

fn rule_registry_covers_triggered_rules(
    rule_registry: &[RuntimeLawRuleDescriptor],
    triggered_rule_ids: &[String],
) -> bool {
    triggered_rule_ids.iter().all(|triggered_rule_id| {
        rule_registry
            .iter()
            .any(|descriptor| descriptor.rule_id == *triggered_rule_id)
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeProtectedActionClass {
    LowRisk,
    StateMutating,
    IdentitySensitive,
    MemoryAuthority,
    ArtifactAuthority,
    Financial,
    InfrastructureCritical,
    ProofRequired,
    LearningPromotion,
    BuilderDeployment,
    SelfHealRemediation,
}

impl RuntimeProtectedActionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeProtectedActionClass::LowRisk => "LOW_RISK",
            RuntimeProtectedActionClass::StateMutating => "STATE_MUTATING",
            RuntimeProtectedActionClass::IdentitySensitive => "IDENTITY_SENSITIVE",
            RuntimeProtectedActionClass::MemoryAuthority => "MEMORY_AUTHORITY",
            RuntimeProtectedActionClass::ArtifactAuthority => "ARTIFACT_AUTHORITY",
            RuntimeProtectedActionClass::Financial => "FINANCIAL",
            RuntimeProtectedActionClass::InfrastructureCritical => "INFRASTRUCTURE_CRITICAL",
            RuntimeProtectedActionClass::ProofRequired => "PROOF_REQUIRED",
            RuntimeProtectedActionClass::LearningPromotion => "LEARNING_PROMOTION",
            RuntimeProtectedActionClass::BuilderDeployment => "BUILDER_DEPLOYMENT",
            RuntimeProtectedActionClass::SelfHealRemediation => "SELF_HEAL_REMEDIATION",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeLawResponseClass {
    Allow,
    AllowWithWarning,
    Degrade,
    Block,
    Quarantine,
    SafeMode,
}

impl RuntimeLawResponseClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeLawResponseClass::Allow => "ALLOW",
            RuntimeLawResponseClass::AllowWithWarning => "ALLOW_WITH_WARNING",
            RuntimeLawResponseClass::Degrade => "DEGRADE",
            RuntimeLawResponseClass::Block => "BLOCK",
            RuntimeLawResponseClass::Quarantine => "QUARANTINE",
            RuntimeLawResponseClass::SafeMode => "SAFE_MODE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeLawSeverity {
    Info,
    Warning,
    Blocking,
    Critical,
    QuarantineRequired,
}

impl RuntimeLawSeverity {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeLawSeverity::Info => "INFO",
            RuntimeLawSeverity::Warning => "WARNING",
            RuntimeLawSeverity::Blocking => "BLOCKING",
            RuntimeLawSeverity::Critical => "CRITICAL",
            RuntimeLawSeverity::QuarantineRequired => "QUARANTINE_REQUIRED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeLawBlastRadiusScope {
    SubsystemScope,
    TenantScope,
    ClusterScope,
    GlobalScope,
}

impl RuntimeLawBlastRadiusScope {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeLawBlastRadiusScope::SubsystemScope => "SUBSYSTEM_SCOPE",
            RuntimeLawBlastRadiusScope::TenantScope => "TENANT_SCOPE",
            RuntimeLawBlastRadiusScope::ClusterScope => "CLUSTER_SCOPE",
            RuntimeLawBlastRadiusScope::GlobalScope => "GLOBAL_SCOPE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeLawRollbackReadinessState {
    NotRequired,
    Ready,
    Missing,
    Unverified,
}

impl RuntimeLawRollbackReadinessState {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeLawRollbackReadinessState::NotRequired => "NOT_REQUIRED",
            RuntimeLawRollbackReadinessState::Ready => "READY",
            RuntimeLawRollbackReadinessState::Missing => "MISSING",
            RuntimeLawRollbackReadinessState::Unverified => "UNVERIFIED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RuntimeLawRuleCategory {
    Envelope,
    Authority,
    Identity,
    Memory,
    Persistence,
    Proof,
    Governance,
    Platform,
    Learning,
    Builder,
    SelfHeal,
    Override,
    Rollback,
    BlastRadius,
}

impl RuntimeLawRuleCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            RuntimeLawRuleCategory::Envelope => "ENVELOPE",
            RuntimeLawRuleCategory::Authority => "AUTHORITY",
            RuntimeLawRuleCategory::Identity => "IDENTITY",
            RuntimeLawRuleCategory::Memory => "MEMORY",
            RuntimeLawRuleCategory::Persistence => "PERSISTENCE",
            RuntimeLawRuleCategory::Proof => "PROOF",
            RuntimeLawRuleCategory::Governance => "GOVERNANCE",
            RuntimeLawRuleCategory::Platform => "PLATFORM",
            RuntimeLawRuleCategory::Learning => "LEARNING",
            RuntimeLawRuleCategory::Builder => "BUILDER",
            RuntimeLawRuleCategory::SelfHeal => "SELF_HEAL",
            RuntimeLawRuleCategory::Override => "OVERRIDE",
            RuntimeLawRuleCategory::Rollback => "ROLLBACK",
            RuntimeLawRuleCategory::BlastRadius => "BLAST_RADIUS",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawPolicyWindow {
    pub law_policy_version: String,
    pub min_compatible_policy_version: String,
    pub max_compatible_policy_version: String,
}

impl RuntimeLawPolicyWindow {
    pub fn v1(
        law_policy_version: String,
        min_compatible_policy_version: String,
        max_compatible_policy_version: String,
    ) -> Result<Self, ContractViolation> {
        let window = Self {
            law_policy_version,
            min_compatible_policy_version,
            max_compatible_policy_version,
        };
        window.validate()?;
        Ok(window)
    }
}

impl Validate for RuntimeLawPolicyWindow {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "runtime_law_policy_window.law_policy_version",
            &self.law_policy_version,
            64,
        )?;
        validate_ascii_token(
            "runtime_law_policy_window.min_compatible_policy_version",
            &self.min_compatible_policy_version,
            64,
        )?;
        validate_ascii_token(
            "runtime_law_policy_window.max_compatible_policy_version",
            &self.max_compatible_policy_version,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawRuleDescriptor {
    pub rule_id: String,
    pub category: RuntimeLawRuleCategory,
    pub enabled: bool,
    pub law_policy_version: String,
    pub owner_subsystem_id: String,
    pub description: String,
}

impl RuntimeLawRuleDescriptor {
    pub fn v1(
        rule_id: String,
        category: RuntimeLawRuleCategory,
        enabled: bool,
        law_policy_version: String,
        owner_subsystem_id: String,
        description: String,
    ) -> Result<Self, ContractViolation> {
        let descriptor = Self {
            rule_id,
            category,
            enabled,
            law_policy_version,
            owner_subsystem_id,
            description,
        };
        descriptor.validate()?;
        Ok(descriptor)
    }
}

impl Validate for RuntimeLawRuleDescriptor {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token("runtime_law_rule_descriptor.rule_id", &self.rule_id, 64)?;
        validate_ascii_token(
            "runtime_law_rule_descriptor.law_policy_version",
            &self.law_policy_version,
            64,
        )?;
        validate_ascii_token(
            "runtime_law_rule_descriptor.owner_subsystem_id",
            &self.owner_subsystem_id,
            64,
        )?;
        validate_ascii_token(
            "runtime_law_rule_descriptor.description",
            &self.description,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawIndependentVerificationSupport {
    pub runtime_node_id: String,
    pub policy_window: RuntimeLawPolicyWindow,
    pub rule_registry: Vec<RuntimeLawRuleDescriptor>,
    pub safe_mode_active: bool,
    pub quarantined_scopes: Vec<String>,
}

impl RuntimeLawIndependentVerificationSupport {
    pub fn v1(
        runtime_node_id: String,
        policy_window: RuntimeLawPolicyWindow,
        rule_registry: Vec<RuntimeLawRuleDescriptor>,
        safe_mode_active: bool,
        quarantined_scopes: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let support = Self {
            runtime_node_id,
            policy_window,
            rule_registry,
            safe_mode_active,
            quarantined_scopes,
        };
        support.validate()?;
        Ok(support)
    }
}

impl Validate for RuntimeLawIndependentVerificationSupport {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "runtime_law_independent_verification_support.runtime_node_id",
            &self.runtime_node_id,
            128,
        )?;
        self.policy_window.validate()?;
        if self.rule_registry.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_law_independent_verification_support.rule_registry",
                reason: "must not be empty",
            });
        }
        if self.rule_registry.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_law_independent_verification_support.rule_registry",
                reason: "too many entries",
            });
        }
        for descriptor in &self.rule_registry {
            descriptor.validate()?;
        }
        validate_ascii_token_vec(
            "runtime_law_independent_verification_support.quarantined_scopes",
            &self.quarantined_scopes,
            64,
            32,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawOverrideState {
    pub operator_identity_ref: String,
    pub second_approver_identity_ref: Option<String>,
    pub override_reason_code: String,
    pub expires_at: MonotonicTimeNs,
    pub dual_approval_required: bool,
    pub dual_approval_satisfied: bool,
    pub authenticated_human_authority: bool,
}

impl RuntimeLawOverrideState {
    pub fn v1(
        operator_identity_ref: String,
        second_approver_identity_ref: Option<String>,
        override_reason_code: String,
        expires_at: MonotonicTimeNs,
        dual_approval_required: bool,
        dual_approval_satisfied: bool,
        authenticated_human_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            operator_identity_ref,
            second_approver_identity_ref,
            override_reason_code,
            expires_at,
            dual_approval_required,
            dual_approval_satisfied,
            authenticated_human_authority,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for RuntimeLawOverrideState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "runtime_law_override_state.operator_identity_ref",
            &self.operator_identity_ref,
            128,
        )?;
        validate_optional_ascii_token(
            "runtime_law_override_state.second_approver_identity_ref",
            &self.second_approver_identity_ref,
            128,
        )?;
        validate_ascii_token(
            "runtime_law_override_state.override_reason_code",
            &self.override_reason_code,
            64,
        )?;
        if self.dual_approval_required && !self.dual_approval_satisfied {
            return Ok(());
        }
        if self.dual_approval_satisfied && self.second_approver_identity_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_law_override_state.second_approver_identity_ref",
                reason: "must be present when dual approval is satisfied",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawDryRunEvaluationState {
    pub predicted_response_class: RuntimeLawResponseClass,
    pub predicted_severity: RuntimeLawSeverity,
    pub predicted_reason_codes: Vec<String>,
}

impl RuntimeLawDryRunEvaluationState {
    pub fn v1(
        predicted_response_class: RuntimeLawResponseClass,
        predicted_severity: RuntimeLawSeverity,
        predicted_reason_codes: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            predicted_response_class,
            predicted_severity,
            predicted_reason_codes,
        };
        state.validate()?;
        Ok(state)
    }
}

impl Validate for RuntimeLawDryRunEvaluationState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token_vec(
            "runtime_law_dry_run_evaluation_state.predicted_reason_codes",
            &self.predicted_reason_codes,
            64,
            16,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawBuilderInput {
    pub approval_state: Option<BuilderApprovalState>,
    pub release_state: Option<BuilderReleaseState>,
    pub post_deploy_result: Option<BuilderPostDeployJudgeResult>,
}

impl RuntimeLawBuilderInput {
    pub fn v1(
        approval_state: Option<BuilderApprovalState>,
        release_state: Option<BuilderReleaseState>,
        post_deploy_result: Option<BuilderPostDeployJudgeResult>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            approval_state,
            release_state,
            post_deploy_result,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for RuntimeLawBuilderInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(value) = self.approval_state.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.release_state.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.post_deploy_result.as_ref() {
            value.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawLearningInput {
    pub artifact_package_build: Option<LearnArtifactPackageBuildOk>,
}

impl RuntimeLawLearningInput {
    pub fn v1(
        artifact_package_build: Option<LearnArtifactPackageBuildOk>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            artifact_package_build,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for RuntimeLawLearningInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(value) = self.artifact_package_build.as_ref() {
            value.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawSelfHealInput {
    pub fix_card: Option<FixCard>,
    pub promotion_decision: Option<PromotionDecision>,
}

impl RuntimeLawSelfHealInput {
    pub fn v1(
        fix_card: Option<FixCard>,
        promotion_decision: Option<PromotionDecision>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            fix_card,
            promotion_decision,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for RuntimeLawSelfHealInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(value) = self.fix_card.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.promotion_decision.as_ref() {
            value.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawEvaluationContext {
    pub builder_input: Option<RuntimeLawBuilderInput>,
    pub learning_input: Option<RuntimeLawLearningInput>,
    pub self_heal_input: Option<RuntimeLawSelfHealInput>,
    pub override_state: Option<RuntimeLawOverrideState>,
    pub evaluated_at: MonotonicTimeNs,
    pub dry_run_requested: bool,
}

impl RuntimeLawEvaluationContext {
    pub fn v1(
        builder_input: Option<RuntimeLawBuilderInput>,
        learning_input: Option<RuntimeLawLearningInput>,
        self_heal_input: Option<RuntimeLawSelfHealInput>,
        override_state: Option<RuntimeLawOverrideState>,
        evaluated_at: MonotonicTimeNs,
        dry_run_requested: bool,
    ) -> Result<Self, ContractViolation> {
        let context = Self {
            builder_input,
            learning_input,
            self_heal_input,
            override_state,
            evaluated_at,
            dry_run_requested,
        };
        context.validate()?;
        Ok(context)
    }
}

impl Default for RuntimeLawEvaluationContext {
    fn default() -> Self {
        Self {
            builder_input: None,
            learning_input: None,
            self_heal_input: None,
            override_state: None,
            evaluated_at: MonotonicTimeNs(0),
            dry_run_requested: false,
        }
    }
}

impl Validate for RuntimeLawEvaluationContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(value) = self.builder_input.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.learning_input.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.self_heal_input.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.override_state.as_ref() {
            value.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawExecutionState {
    pub protected_action_class: RuntimeProtectedActionClass,
    pub final_law_response_class: RuntimeLawResponseClass,
    pub final_law_severity: RuntimeLawSeverity,
    pub law_reason_codes: Vec<String>,
    pub law_policy_version: String,
    pub override_state: Option<RuntimeLawOverrideState>,
    pub rollback_readiness_state: RuntimeLawRollbackReadinessState,
    pub blast_radius_scope: RuntimeLawBlastRadiusScope,
    pub dry_run_evaluation_state: Option<RuntimeLawDryRunEvaluationState>,
    pub triggered_rule_ids: Vec<String>,
    pub subsystem_inputs: Vec<String>,
    pub decision_log_ref: String,
    pub independent_verification_support: Option<RuntimeLawIndependentVerificationSupport>,
    pub artifact_trust_decision_ids: Vec<String>,
    pub artifact_trust_proof_entry_refs: Vec<String>,
    pub artifact_trust_proof_record_ref: Option<String>,
    pub artifact_trust_policy_snapshot_refs: Vec<String>,
    pub artifact_trust_set_snapshot_refs: Vec<String>,
    pub artifact_trust_basis_fingerprints: Vec<String>,
    pub artifact_trust_negative_result_refs: Vec<String>,
}

impl RuntimeLawExecutionState {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        protected_action_class: RuntimeProtectedActionClass,
        final_law_response_class: RuntimeLawResponseClass,
        final_law_severity: RuntimeLawSeverity,
        law_reason_codes: Vec<String>,
        law_policy_version: String,
        override_state: Option<RuntimeLawOverrideState>,
        rollback_readiness_state: RuntimeLawRollbackReadinessState,
        blast_radius_scope: RuntimeLawBlastRadiusScope,
        dry_run_evaluation_state: Option<RuntimeLawDryRunEvaluationState>,
        triggered_rule_ids: Vec<String>,
        subsystem_inputs: Vec<String>,
        decision_log_ref: String,
    ) -> Result<Self, ContractViolation> {
        let state = Self {
            protected_action_class,
            final_law_response_class,
            final_law_severity,
            law_reason_codes,
            law_policy_version,
            override_state,
            rollback_readiness_state,
            blast_radius_scope,
            dry_run_evaluation_state,
            triggered_rule_ids,
            subsystem_inputs,
            decision_log_ref,
            independent_verification_support: None,
            artifact_trust_decision_ids: Vec::new(),
            artifact_trust_proof_entry_refs: Vec::new(),
            artifact_trust_proof_record_ref: None,
            artifact_trust_policy_snapshot_refs: Vec::new(),
            artifact_trust_set_snapshot_refs: Vec::new(),
            artifact_trust_basis_fingerprints: Vec::new(),
            artifact_trust_negative_result_refs: Vec::new(),
        };
        state.validate()?;
        Ok(state)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_artifact_trust_linkage(
        mut self,
        artifact_trust_decision_ids: Vec<String>,
        artifact_trust_proof_entry_refs: Vec<String>,
        artifact_trust_proof_record_ref: Option<String>,
        artifact_trust_policy_snapshot_refs: Vec<String>,
        artifact_trust_set_snapshot_refs: Vec<String>,
        artifact_trust_basis_fingerprints: Vec<String>,
        artifact_trust_negative_result_refs: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        self.artifact_trust_decision_ids = artifact_trust_decision_ids;
        self.artifact_trust_proof_entry_refs = artifact_trust_proof_entry_refs;
        self.artifact_trust_proof_record_ref = artifact_trust_proof_record_ref;
        self.artifact_trust_policy_snapshot_refs = artifact_trust_policy_snapshot_refs;
        self.artifact_trust_set_snapshot_refs = artifact_trust_set_snapshot_refs;
        self.artifact_trust_basis_fingerprints = artifact_trust_basis_fingerprints;
        self.artifact_trust_negative_result_refs = artifact_trust_negative_result_refs;
        self.validate()?;
        Ok(self)
    }

    pub fn with_independent_verification_support(
        mut self,
        independent_verification_support: Option<RuntimeLawIndependentVerificationSupport>,
    ) -> Result<Self, ContractViolation> {
        self.independent_verification_support = independent_verification_support;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for RuntimeLawExecutionState {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token_vec(
            "runtime_law_execution_state.law_reason_codes",
            &self.law_reason_codes,
            64,
            16,
        )?;
        validate_ascii_token(
            "runtime_law_execution_state.law_policy_version",
            &self.law_policy_version,
            64,
        )?;
        if let Some(value) = self.override_state.as_ref() {
            value.validate()?;
        }
        if let Some(value) = self.dry_run_evaluation_state.as_ref() {
            value.validate()?;
        }
        validate_ascii_token_vec(
            "runtime_law_execution_state.triggered_rule_ids",
            &self.triggered_rule_ids,
            64,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.subsystem_inputs",
            &self.subsystem_inputs,
            64,
            32,
        )?;
        validate_ascii_token(
            "runtime_law_execution_state.decision_log_ref",
            &self.decision_log_ref,
            64,
        )?;
        if let Some(value) = self.independent_verification_support.as_ref() {
            value.validate()?;
            if value.policy_window.law_policy_version != self.law_policy_version {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_law_execution_state.independent_verification_support.policy_window.law_policy_version",
                    reason: "must match runtime_law_execution_state.law_policy_version",
                });
            }
            if !rule_registry_covers_triggered_rules(&value.rule_registry, &self.triggered_rule_ids)
            {
                return Err(ContractViolation::InvalidValue {
                    field:
                        "runtime_law_execution_state.independent_verification_support.rule_registry",
                    reason: "must cover runtime_law_execution_state.triggered_rule_ids",
                });
            }
            if self.final_law_response_class == RuntimeLawResponseClass::SafeMode
                && !value.safe_mode_active
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_law_execution_state.independent_verification_support.safe_mode_active",
                    reason: "must be true when final_law_response_class=SAFE_MODE",
                });
            }
            if self.final_law_response_class == RuntimeLawResponseClass::Quarantine
                && !value
                    .quarantined_scopes
                    .iter()
                    .any(|scope| scope == self.blast_radius_scope.as_str())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "runtime_law_execution_state.independent_verification_support.quarantined_scopes",
                    reason: "must include blast_radius_scope when final_law_response_class=QUARANTINE",
                });
            }
        }
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_decision_ids",
            &self.artifact_trust_decision_ids,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_proof_entry_refs",
            &self.artifact_trust_proof_entry_refs,
            128,
            32,
        )?;
        validate_optional_ascii_token(
            "runtime_law_execution_state.artifact_trust_proof_record_ref",
            &self.artifact_trust_proof_record_ref,
            128,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_policy_snapshot_refs",
            &self.artifact_trust_policy_snapshot_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_set_snapshot_refs",
            &self.artifact_trust_set_snapshot_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_basis_fingerprints",
            &self.artifact_trust_basis_fingerprints,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_execution_state.artifact_trust_negative_result_refs",
            &self.artifact_trust_negative_result_refs,
            128,
            32,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawDecisionLogEntry {
    pub sequence: u64,
    pub law_policy_version: String,
    pub protected_action_class: RuntimeProtectedActionClass,
    pub evaluated_rule_ids: Vec<String>,
    pub subsystem_inputs: Vec<String>,
    pub final_response_class: RuntimeLawResponseClass,
    pub final_severity: RuntimeLawSeverity,
    pub reason_codes: Vec<String>,
    pub session_id: Option<u128>,
    pub turn_id: Option<u64>,
    pub proof_record_ref: Option<String>,
    pub builder_proposal_id: Option<String>,
    pub learning_capability_id: Option<String>,
    pub self_heal_fix_id: Option<String>,
    pub override_state: Option<RuntimeLawOverrideState>,
    pub rollback_readiness_state: RuntimeLawRollbackReadinessState,
    pub blast_radius_scope: RuntimeLawBlastRadiusScope,
    pub dry_run_requested: bool,
    pub decision_log_ref: String,
    pub artifact_trust_decision_ids: Vec<String>,
    pub artifact_trust_proof_entry_refs: Vec<String>,
    pub artifact_trust_policy_snapshot_refs: Vec<String>,
    pub artifact_trust_set_snapshot_refs: Vec<String>,
    pub artifact_trust_basis_fingerprints: Vec<String>,
    pub artifact_trust_negative_result_refs: Vec<String>,
}

impl RuntimeLawDecisionLogEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        sequence: u64,
        law_policy_version: String,
        protected_action_class: RuntimeProtectedActionClass,
        evaluated_rule_ids: Vec<String>,
        subsystem_inputs: Vec<String>,
        final_response_class: RuntimeLawResponseClass,
        final_severity: RuntimeLawSeverity,
        reason_codes: Vec<String>,
        session_id: Option<u128>,
        turn_id: Option<u64>,
        proof_record_ref: Option<String>,
        builder_proposal_id: Option<String>,
        learning_capability_id: Option<String>,
        self_heal_fix_id: Option<String>,
        override_state: Option<RuntimeLawOverrideState>,
        rollback_readiness_state: RuntimeLawRollbackReadinessState,
        blast_radius_scope: RuntimeLawBlastRadiusScope,
        dry_run_requested: bool,
        decision_log_ref: String,
    ) -> Result<Self, ContractViolation> {
        let entry = Self {
            sequence,
            law_policy_version,
            protected_action_class,
            evaluated_rule_ids,
            subsystem_inputs,
            final_response_class,
            final_severity,
            reason_codes,
            session_id,
            turn_id,
            proof_record_ref,
            builder_proposal_id,
            learning_capability_id,
            self_heal_fix_id,
            override_state,
            rollback_readiness_state,
            blast_radius_scope,
            dry_run_requested,
            decision_log_ref,
            artifact_trust_decision_ids: Vec::new(),
            artifact_trust_proof_entry_refs: Vec::new(),
            artifact_trust_policy_snapshot_refs: Vec::new(),
            artifact_trust_set_snapshot_refs: Vec::new(),
            artifact_trust_basis_fingerprints: Vec::new(),
            artifact_trust_negative_result_refs: Vec::new(),
        };
        entry.validate()?;
        Ok(entry)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn with_artifact_trust_linkage(
        mut self,
        artifact_trust_decision_ids: Vec<String>,
        artifact_trust_proof_entry_refs: Vec<String>,
        artifact_trust_policy_snapshot_refs: Vec<String>,
        artifact_trust_set_snapshot_refs: Vec<String>,
        artifact_trust_basis_fingerprints: Vec<String>,
        artifact_trust_negative_result_refs: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        self.artifact_trust_decision_ids = artifact_trust_decision_ids;
        self.artifact_trust_proof_entry_refs = artifact_trust_proof_entry_refs;
        self.artifact_trust_policy_snapshot_refs = artifact_trust_policy_snapshot_refs;
        self.artifact_trust_set_snapshot_refs = artifact_trust_set_snapshot_refs;
        self.artifact_trust_basis_fingerprints = artifact_trust_basis_fingerprints;
        self.artifact_trust_negative_result_refs = artifact_trust_negative_result_refs;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for RuntimeLawDecisionLogEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.sequence == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "runtime_law_decision_log_entry.sequence",
                reason: "must be > 0",
            });
        }
        validate_ascii_token(
            "runtime_law_decision_log_entry.law_policy_version",
            &self.law_policy_version,
            64,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.evaluated_rule_ids",
            &self.evaluated_rule_ids,
            64,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.subsystem_inputs",
            &self.subsystem_inputs,
            64,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.reason_codes",
            &self.reason_codes,
            64,
            16,
        )?;
        validate_optional_ascii_token(
            "runtime_law_decision_log_entry.proof_record_ref",
            &self.proof_record_ref,
            128,
        )?;
        validate_optional_ascii_token(
            "runtime_law_decision_log_entry.builder_proposal_id",
            &self.builder_proposal_id,
            96,
        )?;
        validate_optional_ascii_token(
            "runtime_law_decision_log_entry.learning_capability_id",
            &self.learning_capability_id,
            96,
        )?;
        validate_optional_ascii_token(
            "runtime_law_decision_log_entry.self_heal_fix_id",
            &self.self_heal_fix_id,
            96,
        )?;
        if let Some(value) = self.override_state.as_ref() {
            value.validate()?;
        }
        validate_ascii_token(
            "runtime_law_decision_log_entry.decision_log_ref",
            &self.decision_log_ref,
            64,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_decision_ids",
            &self.artifact_trust_decision_ids,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_proof_entry_refs",
            &self.artifact_trust_proof_entry_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_policy_snapshot_refs",
            &self.artifact_trust_policy_snapshot_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_set_snapshot_refs",
            &self.artifact_trust_set_snapshot_refs,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_basis_fingerprints",
            &self.artifact_trust_basis_fingerprints,
            128,
            32,
        )?;
        validate_ascii_token_vec(
            "runtime_law_decision_log_entry.artifact_trust_negative_result_refs",
            &self.artifact_trust_negative_result_refs,
            128,
            32,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_rule_descriptor(rule_id: &str) -> RuntimeLawRuleDescriptor {
        RuntimeLawRuleDescriptor::v1(
            rule_id.to_string(),
            RuntimeLawRuleCategory::Governance,
            true,
            "2026.03.08.law.v1".to_string(),
            "RUNTIME_GOVERNANCE".to_string(),
            "runtime law verification rule".to_string(),
        )
        .expect("runtime law rule descriptor must validate")
    }

    fn sample_verification_support(
        rule_ids: Vec<String>,
    ) -> RuntimeLawIndependentVerificationSupport {
        RuntimeLawIndependentVerificationSupport::v1(
            "runtime-law-node-a".to_string(),
            RuntimeLawPolicyWindow::v1(
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
            )
            .expect("runtime law policy window must validate"),
            rule_ids
                .iter()
                .map(|rule_id| sample_rule_descriptor(rule_id))
                .collect(),
            false,
            Vec::new(),
        )
        .expect("runtime law independent verification support must validate")
    }

    #[test]
    fn runtime_law_independent_verification_support_requires_rule_registry_entries() {
        let err = RuntimeLawIndependentVerificationSupport::v1(
            "runtime-law-node-a".to_string(),
            RuntimeLawPolicyWindow::v1(
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
            )
            .expect("runtime law policy window must validate"),
            Vec::new(),
            false,
            Vec::new(),
        )
        .expect_err("independent verification support must require rule registry entries");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "runtime_law_independent_verification_support.rule_registry"
                );
                assert_eq!(reason, "must not be empty");
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }

    #[test]
    fn runtime_law_execution_state_support_must_cover_triggered_rule_ids() {
        let err = RuntimeLawExecutionState::v1(
            RuntimeProtectedActionClass::ArtifactAuthority,
            RuntimeLawResponseClass::Degrade,
            RuntimeLawSeverity::Warning,
            vec!["LAW_GOVERNANCE_POLICY_DRIFT".to_string()],
            "2026.03.08.law.v1".to_string(),
            None,
            RuntimeLawRollbackReadinessState::NotRequired,
            RuntimeLawBlastRadiusScope::ClusterScope,
            None,
            vec!["RL-GOV-003".to_string()],
            vec!["RUNTIME_GOVERNANCE".to_string()],
            "LAW-DEC-0000000001".to_string(),
        )
        .expect("runtime law execution state must validate before support attachment")
        .with_independent_verification_support(Some(sample_verification_support(vec![
            "RL-GOV-002".to_string(),
        ])))
        .expect_err("support rule registry must cover triggered rule ids");

        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(
                    field,
                    "runtime_law_execution_state.independent_verification_support.rule_registry"
                );
                assert_eq!(
                    reason,
                    "must cover runtime_law_execution_state.triggered_rule_ids"
                );
            }
            _ => panic!("expected invalid-value contract violation"),
        }
    }
}
