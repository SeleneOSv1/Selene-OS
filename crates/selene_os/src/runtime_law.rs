#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use selene_kernel_contracts::ph1builder::{
    BuilderApprovalStateStatus, BuilderPostDeployDecisionAction, BuilderReleaseStateStatus,
};
use selene_kernel_contracts::ph1learn::LearnValidationStatus;
use selene_kernel_contracts::ph1selfheal::{PromotionDecisionAction, SelfHealValidationStatus};
use selene_kernel_contracts::runtime_execution::{
    AdmissionState, AuthorityPolicyDecision, ClientCompatibilityStatus, ClientIntegrityStatus,
    DeviceTrustClass, IdentityRecoveryState, IdentityTrustTier, MemoryEligibilityDecision,
    SimulationCertificationState, RuntimeExecutionEnvelope,
};
use selene_kernel_contracts::runtime_governance::GovernanceClusterConsistency;
use selene_kernel_contracts::runtime_law::{
    RuntimeLawBlastRadiusScope, RuntimeLawBuilderInput, RuntimeLawDecisionLogEntry,
    RuntimeLawDryRunEvaluationState, RuntimeLawEvaluationContext, RuntimeLawExecutionState,
    RuntimeLawOverrideState, RuntimeLawPolicyWindow, RuntimeLawResponseClass,
    RuntimeLawRollbackReadinessState, RuntimeLawRuleCategory, RuntimeLawRuleDescriptor,
    RuntimeLawSelfHealInput, RuntimeLawSeverity, RuntimeLawLearningInput,
    RuntimeProtectedActionClass,
};

pub mod reason_codes {
    pub const LAW_ALLOW_BASELINE: &str = "LAW_ALLOW_BASELINE";
    pub const LAW_ENVELOPE_SESSION_REQUIRED: &str = "LAW_ENVELOPE_SESSION_REQUIRED";
    pub const LAW_ENVELOPE_ADMISSION_REQUIRED: &str = "LAW_ENVELOPE_ADMISSION_REQUIRED";
    pub const LAW_AUTHORITY_DENIED: &str = "LAW_AUTHORITY_DENIED";
    pub const LAW_IDENTITY_POSTURE_REQUIRED: &str = "LAW_IDENTITY_POSTURE_REQUIRED";
    pub const LAW_MEMORY_SCOPE_REQUIRED: &str = "LAW_MEMORY_SCOPE_REQUIRED";
    pub const LAW_PERSISTENCE_STALE: &str = "LAW_PERSISTENCE_STALE";
    pub const LAW_PERSISTENCE_QUARANTINE: &str = "LAW_PERSISTENCE_QUARANTINE";
    pub const LAW_PROOF_REQUIRED: &str = "LAW_PROOF_REQUIRED";
    pub const LAW_PROOF_CHAIN_BROKEN: &str = "LAW_PROOF_CHAIN_BROKEN";
    pub const LAW_GOVERNANCE_SAFE_MODE: &str = "LAW_GOVERNANCE_SAFE_MODE";
    pub const LAW_GOVERNANCE_DIVERGENCE: &str = "LAW_GOVERNANCE_DIVERGENCE";
    pub const LAW_PLATFORM_COMPATIBILITY_REQUIRED: &str = "LAW_PLATFORM_COMPATIBILITY_REQUIRED";
    pub const LAW_PLATFORM_TRUST_REQUIRED: &str = "LAW_PLATFORM_TRUST_REQUIRED";
    pub const LAW_BUILDER_APPROVAL_REQUIRED: &str = "LAW_BUILDER_APPROVAL_REQUIRED";
    pub const LAW_BUILDER_ROLLBACK_REQUIRED: &str = "LAW_BUILDER_ROLLBACK_REQUIRED";
    pub const LAW_LEARNING_PROMOTION_DENIED: &str = "LAW_LEARNING_PROMOTION_DENIED";
    pub const LAW_SELF_HEAL_UNSAFE: &str = "LAW_SELF_HEAL_UNSAFE";
    pub const LAW_OVERRIDE_CONTROL_REQUIRED: &str = "LAW_OVERRIDE_CONTROL_REQUIRED";
    pub const LAW_OVERRIDE_APPLIED: &str = "LAW_OVERRIDE_APPLIED";
}

const SUBSYSTEM_RUNTIME_LAW: &str = "RUNTIME_LAW";
const SUBSYSTEM_SESSION_ENGINE: &str = "SESSION_ENGINE";
const SUBSYSTEM_AUTHORITY_LAYER: &str = "AUTHORITY_LAYER";
const SUBSYSTEM_IDENTITY_ENGINE: &str = "IDENTITY_ENGINE";
const SUBSYSTEM_MEMORY_ENGINE: &str = "MEMORY_ENGINE";
const SUBSYSTEM_PERSISTENCE_LAYER: &str = "PERSISTENCE_LAYER";
const SUBSYSTEM_PROOF_ENGINE: &str = "PH1.J";
const SUBSYSTEM_RUNTIME_GOVERNANCE: &str = "RUNTIME_GOVERNANCE";
const SUBSYSTEM_PLATFORM_RUNTIME: &str = "PLATFORM_RUNTIME";
const SUBSYSTEM_BUILDER: &str = "PH1.BUILDER";
const SUBSYSTEM_LEARNING: &str = "PH1.LEARN";
const SUBSYSTEM_SELF_HEAL: &str = "PH1.SELFHEAL";
const SUBSYSTEM_OVERRIDE: &str = "HUMAN_OVERRIDE";

const RULE_ALLOW_BASELINE: &str = "RL-BASE-001";
const RULE_ENV_SESSION_REQUIRED: &str = "RL-ENV-001";
const RULE_ENV_ADMISSION_REQUIRED: &str = "RL-ENV-002";
const RULE_AUTHORITY_REQUIRED: &str = "RL-AUTH-001";
const RULE_IDENTITY_REQUIRED: &str = "RL-ID-001";
const RULE_MEMORY_REQUIRED: &str = "RL-MEM-001";
const RULE_PERSISTENCE_STALE: &str = "RL-PERSIST-001";
const RULE_PERSISTENCE_QUARANTINE: &str = "RL-PERSIST-002";
const RULE_PROOF_REQUIRED: &str = "RL-PROOF-001";
const RULE_PROOF_CHAIN_BROKEN: &str = "RL-PROOF-002";
const RULE_GOVERNANCE_SAFE_MODE: &str = "RL-GOV-001";
const RULE_GOVERNANCE_DIVERGENCE: &str = "RL-GOV-002";
const RULE_PLATFORM_COMPATIBILITY: &str = "RL-PLAT-001";
const RULE_PLATFORM_TRUST: &str = "RL-PLAT-002";
const RULE_BUILDER_APPROVAL: &str = "RL-BUILD-001";
const RULE_BUILDER_ROLLBACK: &str = "RL-BUILD-002";
const RULE_LEARNING_PROMOTION: &str = "RL-LEARN-001";
const RULE_SELF_HEAL_UNSAFE: &str = "RL-SH-001";
const RULE_OVERRIDE_CONTROL: &str = "RL-OVR-001";
const RULE_OVERRIDE_APPLIED: &str = "RL-OVR-002";

#[derive(Debug, Clone)]
pub struct RuntimeLawConfig {
    pub policy_window: RuntimeLawPolicyWindow,
    pub runtime_node_id: String,
}

impl RuntimeLawConfig {
    pub fn mvp_v1() -> Self {
        Self {
            policy_window: RuntimeLawPolicyWindow::v1(
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
                "2026.03.08.law.v1".to_string(),
            )
            .expect("runtime law policy window must validate"),
            runtime_node_id: "runtime-law-node-a".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeLawDecision {
    pub primary_rule_id: String,
    pub response_class: RuntimeLawResponseClass,
    pub severity: RuntimeLawSeverity,
    pub reason_codes: Vec<String>,
    pub law_state: RuntimeLawExecutionState,
}

#[derive(Debug, Clone)]
pub struct RuntimeLawSnapshot {
    pub policy_window: RuntimeLawPolicyWindow,
    pub rule_registry: Vec<RuntimeLawRuleDescriptor>,
    pub decision_log: Vec<RuntimeLawDecisionLogEntry>,
    pub safe_mode_active: bool,
    pub quarantined_scopes: Vec<String>,
}

#[derive(Debug)]
struct RuntimeLawStateStore {
    decision_log: Vec<RuntimeLawDecisionLogEntry>,
    next_sequence: u64,
    safe_mode_active: bool,
    quarantined_scopes: BTreeSet<String>,
}

impl RuntimeLawStateStore {
    fn new() -> Self {
        Self {
            decision_log: Vec::new(),
            next_sequence: 1,
            safe_mode_active: false,
            quarantined_scopes: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeLawRuntime {
    config: RuntimeLawConfig,
    rule_registry: Vec<RuntimeLawRuleDescriptor>,
    state: Arc<Mutex<RuntimeLawStateStore>>,
}

impl Default for RuntimeLawRuntime {
    fn default() -> Self {
        Self::new(RuntimeLawConfig::mvp_v1())
    }
}

impl RuntimeLawRuntime {
    pub fn new(config: RuntimeLawConfig) -> Self {
        Self {
            config,
            rule_registry: default_rule_registry(),
            state: Arc::new(Mutex::new(RuntimeLawStateStore::new())),
        }
    }

    pub fn snapshot(&self) -> RuntimeLawSnapshot {
        let guard = self.state.lock().expect("runtime law state lock poisoned");
        RuntimeLawSnapshot {
            policy_window: self.config.policy_window.clone(),
            rule_registry: self.rule_registry.clone(),
            decision_log: guard.decision_log.clone(),
            safe_mode_active: guard.safe_mode_active,
            quarantined_scopes: guard.quarantined_scopes.iter().cloned().collect(),
        }
    }

    pub fn decision_log_snapshot(&self) -> Vec<RuntimeLawDecisionLogEntry> {
        self.snapshot().decision_log
    }

    pub fn rule_registry_snapshot(&self) -> Vec<RuntimeLawRuleDescriptor> {
        self.rule_registry.clone()
    }

    pub fn policy_version(&self) -> &str {
        &self.config.policy_window.law_policy_version
    }

    pub fn evaluate(
        &self,
        envelope: &RuntimeExecutionEnvelope,
        action_class: RuntimeProtectedActionClass,
        context: &RuntimeLawEvaluationContext,
    ) -> RuntimeLawDecision {
        let safe_mode_already_active = self
            .state
            .lock()
            .expect("runtime law state lock poisoned")
            .safe_mode_active;
        let mut triggered_rules = Vec::new();

        if safe_mode_already_active {
            triggered_rules.push(trigger(
                RULE_GOVERNANCE_SAFE_MODE,
                SUBSYSTEM_RUNTIME_LAW,
                reason_codes::LAW_GOVERNANCE_SAFE_MODE,
                RuntimeLawResponseClass::SafeMode,
                RuntimeLawSeverity::Critical,
                RuntimeLawBlastRadiusScope::GlobalScope,
            ));
        }

        if action_requires_session(action_class) && envelope.session_id.is_none() {
            triggered_rules.push(trigger(
                RULE_ENV_SESSION_REQUIRED,
                SUBSYSTEM_SESSION_ENGINE,
                reason_codes::LAW_ENVELOPE_SESSION_REQUIRED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }
        if action_requires_admission(action_class)
            && envelope.admission_state != AdmissionState::ExecutionAdmitted
        {
            triggered_rules.push(trigger(
                RULE_ENV_ADMISSION_REQUIRED,
                SUBSYSTEM_RUNTIME_LAW,
                reason_codes::LAW_ENVELOPE_ADMISSION_REQUIRED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }

        if let Some(governance_state) = envelope.governance_state.as_ref() {
            if governance_state.safe_mode_active {
                triggered_rules.push(trigger(
                    RULE_GOVERNANCE_SAFE_MODE,
                    SUBSYSTEM_RUNTIME_GOVERNANCE,
                    reason_codes::LAW_GOVERNANCE_SAFE_MODE,
                    RuntimeLawResponseClass::SafeMode,
                    RuntimeLawSeverity::Critical,
                    RuntimeLawBlastRadiusScope::GlobalScope,
                ));
            }
            if governance_state.cluster_consistency != GovernanceClusterConsistency::Consistent {
                triggered_rules.push(trigger(
                    RULE_GOVERNANCE_DIVERGENCE,
                    SUBSYSTEM_RUNTIME_GOVERNANCE,
                    reason_codes::LAW_GOVERNANCE_DIVERGENCE,
                    if action_class == RuntimeProtectedActionClass::InfrastructureCritical {
                        RuntimeLawResponseClass::SafeMode
                    } else {
                        RuntimeLawResponseClass::Degrade
                    },
                    if action_class == RuntimeProtectedActionClass::InfrastructureCritical {
                        RuntimeLawSeverity::Critical
                    } else {
                        RuntimeLawSeverity::Warning
                    },
                    RuntimeLawBlastRadiusScope::ClusterScope,
                ));
            }
        }

        if let Some(persistence_state) = envelope.persistence_state.as_ref() {
            if persistence_quarantine_required(persistence_state) {
                triggered_rules.push(trigger(
                    RULE_PERSISTENCE_QUARANTINE,
                    SUBSYSTEM_PERSISTENCE_LAYER,
                    reason_codes::LAW_PERSISTENCE_QUARANTINE,
                    RuntimeLawResponseClass::Quarantine,
                    RuntimeLawSeverity::QuarantineRequired,
                    RuntimeLawBlastRadiusScope::TenantScope,
                ));
            } else if persistence_stale_rejected(persistence_state) {
                triggered_rules.push(trigger(
                    RULE_PERSISTENCE_STALE,
                    SUBSYSTEM_PERSISTENCE_LAYER,
                    reason_codes::LAW_PERSISTENCE_STALE,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::TenantScope,
                ));
            }
        }

        if action_requires_proof(action_class) {
            match envelope.proof_state.as_ref() {
                Some(proof_state) if proof_state_is_available(proof_state) => {}
                Some(proof_state) if proof_chain_critical(proof_state) => triggered_rules.push(trigger(
                    RULE_PROOF_CHAIN_BROKEN,
                    SUBSYSTEM_PROOF_ENGINE,
                    reason_codes::LAW_PROOF_CHAIN_BROKEN,
                    RuntimeLawResponseClass::Quarantine,
                    RuntimeLawSeverity::QuarantineRequired,
                    RuntimeLawBlastRadiusScope::ClusterScope,
                )),
                _ => triggered_rules.push(trigger(
                    RULE_PROOF_REQUIRED,
                    SUBSYSTEM_PROOF_ENGINE,
                    reason_codes::LAW_PROOF_REQUIRED,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::ClusterScope,
                )),
            }
        }

        if action_requires_identity(action_class) && !identity_posture_satisfied(envelope, action_class)
        {
            triggered_rules.push(trigger(
                RULE_IDENTITY_REQUIRED,
                SUBSYSTEM_IDENTITY_ENGINE,
                reason_codes::LAW_IDENTITY_POSTURE_REQUIRED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }

        if action_class == RuntimeProtectedActionClass::MemoryAuthority
            && !memory_posture_satisfied(envelope)
        {
            triggered_rules.push(trigger(
                RULE_MEMORY_REQUIRED,
                SUBSYSTEM_MEMORY_ENGINE,
                reason_codes::LAW_MEMORY_SCOPE_REQUIRED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }

        if action_requires_authority(action_class) && !authority_posture_satisfied(envelope) {
            triggered_rules.push(trigger(
                RULE_AUTHORITY_REQUIRED,
                SUBSYSTEM_AUTHORITY_LAYER,
                reason_codes::LAW_AUTHORITY_DENIED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }

        if action_requires_platform_posture(action_class) {
            if platform_hard_block_required(envelope) {
                triggered_rules.push(trigger(
                    RULE_PLATFORM_COMPATIBILITY,
                    SUBSYSTEM_PLATFORM_RUNTIME,
                    reason_codes::LAW_PLATFORM_COMPATIBILITY_REQUIRED,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::TenantScope,
                ));
            } else if platform_trust_warning(envelope) {
                triggered_rules.push(trigger(
                    RULE_PLATFORM_TRUST,
                    SUBSYSTEM_PLATFORM_RUNTIME,
                    reason_codes::LAW_PLATFORM_TRUST_REQUIRED,
                    RuntimeLawResponseClass::Degrade,
                    RuntimeLawSeverity::Warning,
                    RuntimeLawBlastRadiusScope::TenantScope,
                ));
            }
        }

        let mut rollback_readiness_state = rollback_readiness_for_action(action_class, context);

        if action_class == RuntimeProtectedActionClass::BuilderDeployment {
            if !builder_posture_satisfied(context.builder_input.as_ref()) {
                triggered_rules.push(trigger(
                    RULE_BUILDER_APPROVAL,
                    SUBSYSTEM_BUILDER,
                    reason_codes::LAW_BUILDER_APPROVAL_REQUIRED,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::SubsystemScope,
                ));
            }
            if rollback_readiness_state != RuntimeLawRollbackReadinessState::Ready {
                triggered_rules.push(trigger(
                    RULE_BUILDER_ROLLBACK,
                    SUBSYSTEM_BUILDER,
                    reason_codes::LAW_BUILDER_ROLLBACK_REQUIRED,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::SubsystemScope,
                ));
            }
        }

        if action_class == RuntimeProtectedActionClass::LearningPromotion
            && !learning_posture_satisfied(context.learning_input.as_ref())
        {
            rollback_readiness_state = RuntimeLawRollbackReadinessState::Unverified;
            triggered_rules.push(trigger(
                RULE_LEARNING_PROMOTION,
                SUBSYSTEM_LEARNING,
                reason_codes::LAW_LEARNING_PROMOTION_DENIED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::SubsystemScope,
            ));
        }

        if action_class == RuntimeProtectedActionClass::SelfHealRemediation {
            if !self_heal_posture_satisfied(context.self_heal_input.as_ref()) {
                triggered_rules.push(trigger(
                    RULE_SELF_HEAL_UNSAFE,
                    SUBSYSTEM_SELF_HEAL,
                    reason_codes::LAW_SELF_HEAL_UNSAFE,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::SubsystemScope,
                ));
            }
            if rollback_readiness_state != RuntimeLawRollbackReadinessState::Ready {
                triggered_rules.push(trigger(
                    RULE_BUILDER_ROLLBACK,
                    SUBSYSTEM_SELF_HEAL,
                    reason_codes::LAW_BUILDER_ROLLBACK_REQUIRED,
                    RuntimeLawResponseClass::Block,
                    RuntimeLawSeverity::Blocking,
                    RuntimeLawBlastRadiusScope::SubsystemScope,
                ));
            }
        }

        let override_validity = context
            .override_state
            .as_ref()
            .map(|state| override_validity(state, action_class, context.evaluated_at));
        if matches!(override_validity, Some(OverrideValidity::Invalid)) {
            triggered_rules.push(trigger(
                RULE_OVERRIDE_CONTROL,
                SUBSYSTEM_OVERRIDE,
                reason_codes::LAW_OVERRIDE_CONTROL_REQUIRED,
                RuntimeLawResponseClass::Block,
                RuntimeLawSeverity::Blocking,
                RuntimeLawBlastRadiusScope::TenantScope,
            ));
        }

        let decision = self.record_decision(
            envelope,
            action_class,
            context,
            triggered_rules,
            rollback_readiness_state,
            override_validity,
        );
        decision
    }

    pub fn govern_completion(
        &self,
        envelope: &RuntimeExecutionEnvelope,
        action_class: RuntimeProtectedActionClass,
        context: &RuntimeLawEvaluationContext,
    ) -> Result<RuntimeExecutionEnvelope, RuntimeLawDecision> {
        let decision = self.evaluate(envelope, action_class, context);
        let result = envelope
            .with_law_state(Some(decision.law_state.clone()))
            .expect("runtime law state must validate");
        if context.dry_run_requested {
            return Ok(result);
        }
        match decision.response_class {
            RuntimeLawResponseClass::Allow
            | RuntimeLawResponseClass::AllowWithWarning
            | RuntimeLawResponseClass::Degrade => Ok(result),
            RuntimeLawResponseClass::Block
            | RuntimeLawResponseClass::Quarantine
            | RuntimeLawResponseClass::SafeMode => Err(decision),
        }
    }

    fn record_decision(
        &self,
        envelope: &RuntimeExecutionEnvelope,
        action_class: RuntimeProtectedActionClass,
        context: &RuntimeLawEvaluationContext,
        mut triggered_rules: Vec<TriggeredRule>,
        rollback_readiness_state: RuntimeLawRollbackReadinessState,
        override_validity: Option<OverrideValidity>,
    ) -> RuntimeLawDecision {
        let override_allowed = matches!(override_validity, Some(OverrideValidity::Valid))
            && !triggered_rules.iter().any(|rule| !rule.overridable)
            && !context.dry_run_requested;
        if override_allowed && !triggered_rules.is_empty() {
            triggered_rules.push(trigger(
                RULE_OVERRIDE_APPLIED,
                SUBSYSTEM_OVERRIDE,
                reason_codes::LAW_OVERRIDE_APPLIED,
                RuntimeLawResponseClass::AllowWithWarning,
                RuntimeLawSeverity::Warning,
                blast_radius_from_rules(&triggered_rules),
            ));
        }

        let (response_class, severity, blast_radius_scope, primary_rule_id, reason_codes) =
            if triggered_rules.is_empty() {
                (
                    RuntimeLawResponseClass::Allow,
                    RuntimeLawSeverity::Info,
                    RuntimeLawBlastRadiusScope::SubsystemScope,
                    RULE_ALLOW_BASELINE.to_string(),
                    vec![reason_codes::LAW_ALLOW_BASELINE.to_string()],
                )
            } else if override_allowed {
                (
                    RuntimeLawResponseClass::AllowWithWarning,
                    RuntimeLawSeverity::Warning,
                    blast_radius_from_rules(&triggered_rules),
                    RULE_OVERRIDE_APPLIED.to_string(),
                    unique_reason_codes(&triggered_rules),
                )
            } else {
                (
                    strongest_response(&triggered_rules),
                    strongest_severity(&triggered_rules),
                    blast_radius_from_rules(&triggered_rules),
                    primary_rule_id(&triggered_rules).to_string(),
                    unique_reason_codes(&triggered_rules),
                )
            };

        let mut evaluated_rule_ids = unique_rule_ids(&triggered_rules);
        if evaluated_rule_ids.is_empty() {
            evaluated_rule_ids.push(RULE_ALLOW_BASELINE.to_string());
        }
        let mut subsystem_inputs = unique_subsystem_inputs(&triggered_rules);
        if subsystem_inputs.is_empty() {
            subsystem_inputs.push(SUBSYSTEM_RUNTIME_LAW.to_string());
        }

        let dry_run_state = if context.dry_run_requested {
            Some(
                RuntimeLawDryRunEvaluationState::v1(
                    response_class,
                    severity,
                    reason_codes.clone(),
                )
                .expect("dry run state must validate"),
            )
        } else {
            None
        };

        let mut guard = self.state.lock().expect("runtime law state lock poisoned");
        let sequence = guard.next_sequence;
        guard.next_sequence += 1;
        let decision_log_ref = format!("LAW-DEC-{sequence:010}");
        if response_class == RuntimeLawResponseClass::SafeMode {
            guard.safe_mode_active = true;
            guard
                .quarantined_scopes
                .insert(RuntimeLawBlastRadiusScope::GlobalScope.as_str().to_string());
        }
        if response_class == RuntimeLawResponseClass::Quarantine {
            guard
                .quarantined_scopes
                .insert(blast_radius_scope.as_str().to_string());
        }

        let law_state = RuntimeLawExecutionState::v1(
            action_class,
            response_class,
            severity,
            reason_codes.clone(),
            self.policy_version().to_string(),
            context.override_state.clone(),
            rollback_readiness_state,
            blast_radius_scope,
            dry_run_state,
            evaluated_rule_ids.clone(),
            subsystem_inputs.clone(),
            decision_log_ref.clone(),
        )
        .expect("runtime law state must validate");

        let entry = RuntimeLawDecisionLogEntry::v1(
            sequence,
            self.policy_version().to_string(),
            action_class,
            evaluated_rule_ids,
            subsystem_inputs,
            response_class,
            severity,
            reason_codes.clone(),
            envelope.session_id.map(|value| value.0),
            Some(envelope.turn_id.0),
            envelope
                .proof_state
                .as_ref()
                .and_then(|value| value.proof_record_ref.clone()),
            builder_proposal_id(context.builder_input.as_ref()),
            learning_capability_id(context.learning_input.as_ref()),
            self_heal_fix_id(context.self_heal_input.as_ref()),
            context.override_state.clone(),
            rollback_readiness_state,
            blast_radius_scope,
            context.dry_run_requested,
            decision_log_ref,
        )
        .expect("runtime law decision log entry must validate");
        guard.decision_log.push(entry);

        RuntimeLawDecision {
            primary_rule_id,
            response_class,
            severity,
            reason_codes,
            law_state,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OverrideValidity {
    Valid,
    Invalid,
}

#[derive(Debug, Clone)]
struct TriggeredRule {
    rule_id: &'static str,
    subsystem_input: &'static str,
    reason_code: &'static str,
    response_class: RuntimeLawResponseClass,
    severity: RuntimeLawSeverity,
    blast_radius_scope: RuntimeLawBlastRadiusScope,
    overridable: bool,
}

fn trigger(
    rule_id: &'static str,
    subsystem_input: &'static str,
    reason_code: &'static str,
    response_class: RuntimeLawResponseClass,
    severity: RuntimeLawSeverity,
    blast_radius_scope: RuntimeLawBlastRadiusScope,
) -> TriggeredRule {
    TriggeredRule {
        rule_id,
        subsystem_input,
        reason_code,
        response_class,
        severity,
        blast_radius_scope,
        overridable: !matches!(
            reason_code,
            reason_codes::LAW_PROOF_CHAIN_BROKEN
                | reason_codes::LAW_GOVERNANCE_SAFE_MODE
                | reason_codes::LAW_PERSISTENCE_QUARANTINE
                | reason_codes::LAW_ENVELOPE_SESSION_REQUIRED
                | reason_codes::LAW_ENVELOPE_ADMISSION_REQUIRED
        ),
    }
}

fn action_requires_session(action_class: RuntimeProtectedActionClass) -> bool {
    action_class != RuntimeProtectedActionClass::LowRisk
}

fn action_requires_admission(action_class: RuntimeProtectedActionClass) -> bool {
    action_class != RuntimeProtectedActionClass::LowRisk
}

fn action_requires_authority(action_class: RuntimeProtectedActionClass) -> bool {
    matches!(
        action_class,
        RuntimeProtectedActionClass::StateMutating
            | RuntimeProtectedActionClass::IdentitySensitive
            | RuntimeProtectedActionClass::MemoryAuthority
            | RuntimeProtectedActionClass::ArtifactAuthority
            | RuntimeProtectedActionClass::Financial
            | RuntimeProtectedActionClass::InfrastructureCritical
    )
}

fn action_requires_identity(action_class: RuntimeProtectedActionClass) -> bool {
    matches!(
        action_class,
        RuntimeProtectedActionClass::IdentitySensitive
            | RuntimeProtectedActionClass::MemoryAuthority
            | RuntimeProtectedActionClass::ArtifactAuthority
            | RuntimeProtectedActionClass::Financial
            | RuntimeProtectedActionClass::InfrastructureCritical
    )
}

fn action_requires_platform_posture(action_class: RuntimeProtectedActionClass) -> bool {
    matches!(
        action_class,
        RuntimeProtectedActionClass::ArtifactAuthority
            | RuntimeProtectedActionClass::Financial
            | RuntimeProtectedActionClass::InfrastructureCritical
            | RuntimeProtectedActionClass::ProofRequired
    )
}

fn action_requires_proof(action_class: RuntimeProtectedActionClass) -> bool {
    matches!(
        action_class,
        RuntimeProtectedActionClass::ProofRequired
            | RuntimeProtectedActionClass::Financial
            | RuntimeProtectedActionClass::InfrastructureCritical
            | RuntimeProtectedActionClass::ArtifactAuthority
    )
}

fn identity_posture_satisfied(
    envelope: &RuntimeExecutionEnvelope,
    action_class: RuntimeProtectedActionClass,
) -> bool {
    let Some(identity_state) = envelope.identity_state.as_ref() else {
        return false;
    };
    let base_ok = match action_class {
        RuntimeProtectedActionClass::IdentitySensitive
        | RuntimeProtectedActionClass::ArtifactAuthority
        | RuntimeProtectedActionClass::ProofRequired => matches!(
            identity_state.trust_tier,
            IdentityTrustTier::Verified | IdentityTrustTier::HighConfidence
        ),
        RuntimeProtectedActionClass::MemoryAuthority
        | RuntimeProtectedActionClass::Financial
        | RuntimeProtectedActionClass::InfrastructureCritical => {
            identity_state.trust_tier == IdentityTrustTier::Verified
        }
        _ => true,
    };
    base_ok
        && !identity_state.step_up_required
        && !matches!(
            identity_state.recovery_state,
            IdentityRecoveryState::RecoveryRestricted
                | IdentityRecoveryState::ReauthRequired
                | IdentityRecoveryState::ReEnrollmentRequired
        )
}

fn memory_posture_satisfied(envelope: &RuntimeExecutionEnvelope) -> bool {
    envelope
        .memory_state
        .as_ref()
        .map(|state| state.eligibility_decision == MemoryEligibilityDecision::Eligible)
        .unwrap_or(false)
}

fn authority_posture_satisfied(envelope: &RuntimeExecutionEnvelope) -> bool {
    envelope
        .authority_state
        .as_ref()
        .map(|state| {
            state.policy_decision == AuthorityPolicyDecision::Allowed
                && state.simulation_certification_state
                    == SimulationCertificationState::CertifiedActive
                && (!state.identity_scope_required || state.identity_scope_satisfied)
        })
        .unwrap_or(false)
}

fn platform_hard_block_required(envelope: &RuntimeExecutionEnvelope) -> bool {
    let context = &envelope.platform_context;
    matches!(
        context.compatibility_status,
        ClientCompatibilityStatus::UnsupportedClient
    ) || matches!(
        context.integrity_status,
        ClientIntegrityStatus::IntegrityFailed
    ) || matches!(context.device_trust_class, DeviceTrustClass::UntrustedDevice)
}

fn platform_trust_warning(envelope: &RuntimeExecutionEnvelope) -> bool {
    let context = &envelope.platform_context;
    matches!(context.device_trust_class, DeviceTrustClass::RestrictedDevice)
        || matches!(
            context.compatibility_status,
            ClientCompatibilityStatus::UpgradeRequired
        )
        || matches!(
            context.integrity_status,
            ClientIntegrityStatus::Unknown
        )
}

fn proof_state_is_available(proof_state: &selene_kernel_contracts::runtime_execution::ProofExecutionState) -> bool {
    matches!(
        proof_state.proof_write_outcome,
        selene_kernel_contracts::ph1j::ProofWriteOutcome::Written
            | selene_kernel_contracts::ph1j::ProofWriteOutcome::ReusedExisting
    ) && proof_state.proof_record_ref.is_some()
}

fn proof_chain_critical(proof_state: &selene_kernel_contracts::runtime_execution::ProofExecutionState) -> bool {
    matches!(
        proof_state.proof_failure_class,
        Some(selene_kernel_contracts::ph1j::ProofFailureClass::ProofChainIntegrityFailure)
            | Some(selene_kernel_contracts::ph1j::ProofFailureClass::ProofSignatureFailure)
    )
}

fn persistence_quarantine_required(
    persistence_state: &selene_kernel_contracts::runtime_execution::PersistenceExecutionState,
) -> bool {
    persistence_state.recovery_mode
        == selene_kernel_contracts::runtime_execution::PersistenceRecoveryMode::QuarantinedLocalState
        || persistence_state.conflict_severity
            == Some(selene_kernel_contracts::runtime_execution::PersistenceConflictSeverity::QuarantineRequired)
}

fn persistence_stale_rejected(
    persistence_state: &selene_kernel_contracts::runtime_execution::PersistenceExecutionState,
) -> bool {
    persistence_state.acknowledgement_state
        == selene_kernel_contracts::runtime_execution::PersistenceAcknowledgementState::StaleRejected
        || persistence_state.conflict_severity
            == Some(selene_kernel_contracts::runtime_execution::PersistenceConflictSeverity::StaleRejected)
}

fn rollback_readiness_for_action(
    action_class: RuntimeProtectedActionClass,
    context: &RuntimeLawEvaluationContext,
) -> RuntimeLawRollbackReadinessState {
    match action_class {
        RuntimeProtectedActionClass::BuilderDeployment => context
            .builder_input
            .as_ref()
            .and_then(|input| input.release_state.as_ref())
            .map(|state| {
                if state.rollback_ready {
                    RuntimeLawRollbackReadinessState::Ready
                } else {
                    RuntimeLawRollbackReadinessState::Missing
                }
            })
            .unwrap_or(RuntimeLawRollbackReadinessState::Unverified),
        RuntimeProtectedActionClass::SelfHealRemediation => context
            .self_heal_input
            .as_ref()
            .and_then(|input| input.promotion_decision.as_ref())
            .map(|state| {
                if state.rollback_ready {
                    RuntimeLawRollbackReadinessState::Ready
                } else {
                    RuntimeLawRollbackReadinessState::Missing
                }
            })
            .unwrap_or(RuntimeLawRollbackReadinessState::Unverified),
        _ => RuntimeLawRollbackReadinessState::NotRequired,
    }
}

fn builder_posture_satisfied(input: Option<&RuntimeLawBuilderInput>) -> bool {
    let Some(input) = input else {
        return false;
    };
    let Some(approval_state) = input.approval_state.as_ref() else {
        return false;
    };
    let Some(release_state) = input.release_state.as_ref() else {
        return false;
    };
    approval_state.status == BuilderApprovalStateStatus::Approved
        && release_state.rollback_ready
        && !matches!(release_state.status, BuilderReleaseStateStatus::Blocked)
        && !matches!(
            input.post_deploy_result.as_ref().map(|value| value.action),
            Some(BuilderPostDeployDecisionAction::Revert)
        )
}

fn learning_posture_satisfied(input: Option<&RuntimeLawLearningInput>) -> bool {
    let Some(input) = input else {
        return false;
    };
    let Some(build) = input.artifact_package_build.as_ref() else {
        return false;
    };
    build.validation_status == LearnValidationStatus::Ok
        && build.artifacts_versioned
        && build.rollbackable
        && build.no_runtime_drift
        && !build.advisory_only
        && !build.no_execution_authority
}

fn self_heal_posture_satisfied(input: Option<&RuntimeLawSelfHealInput>) -> bool {
    let Some(input) = input else {
        return false;
    };
    let Some(fix_card) = input.fix_card.as_ref() else {
        return false;
    };
    let Some(promotion_decision) = input.promotion_decision.as_ref() else {
        return false;
    };
    fix_card.validation_status == SelfHealValidationStatus::Ok
        && !fix_card.advisory_only
        && !fix_card.no_execution_authority
        && promotion_decision.promotion_eligible
        && promotion_decision.rollback_ready
        && matches!(promotion_decision.decision_action, PromotionDecisionAction::Promote)
        && !promotion_decision.advisory_only
        && !promotion_decision.no_execution_authority
        && (!promotion_decision.governance_required || promotion_decision.approved_by.is_some())
        && fix_card.regression_risk_bp.unwrap_or(0) <= 2_500
}

fn override_validity(
    override_state: &RuntimeLawOverrideState,
    action_class: RuntimeProtectedActionClass,
    evaluated_at: selene_kernel_contracts::MonotonicTimeNs,
) -> OverrideValidity {
    if !override_state.authenticated_human_authority || override_state.expires_at.0 <= evaluated_at.0 {
        return OverrideValidity::Invalid;
    }
    let dual_required = override_state.dual_approval_required || action_requires_dual_override(action_class);
    if dual_required && !override_state.dual_approval_satisfied {
        return OverrideValidity::Invalid;
    }
    OverrideValidity::Valid
}

fn action_requires_dual_override(action_class: RuntimeProtectedActionClass) -> bool {
    matches!(
        action_class,
        RuntimeProtectedActionClass::Financial
            | RuntimeProtectedActionClass::InfrastructureCritical
            | RuntimeProtectedActionClass::BuilderDeployment
            | RuntimeProtectedActionClass::SelfHealRemediation
    )
}

fn strongest_response(rules: &[TriggeredRule]) -> RuntimeLawResponseClass {
    rules
        .iter()
        .max_by_key(|rule| response_rank(rule.response_class))
        .map(|rule| rule.response_class)
        .unwrap_or(RuntimeLawResponseClass::Allow)
}

fn strongest_severity(rules: &[TriggeredRule]) -> RuntimeLawSeverity {
    rules
        .iter()
        .max_by_key(|rule| severity_rank(rule.severity))
        .map(|rule| rule.severity)
        .unwrap_or(RuntimeLawSeverity::Info)
}

fn blast_radius_from_rules(rules: &[TriggeredRule]) -> RuntimeLawBlastRadiusScope {
    rules
        .iter()
        .max_by_key(|rule| blast_radius_rank(rule.blast_radius_scope))
        .map(|rule| rule.blast_radius_scope)
        .unwrap_or(RuntimeLawBlastRadiusScope::SubsystemScope)
}

fn primary_rule_id(rules: &[TriggeredRule]) -> &'static str {
    rules
        .iter()
        .max_by_key(|rule| (response_rank(rule.response_class), severity_rank(rule.severity)))
        .map(|rule| rule.rule_id)
        .unwrap_or(RULE_ALLOW_BASELINE)
}

fn unique_reason_codes(rules: &[TriggeredRule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        let value = rule.reason_code.to_string();
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn unique_rule_ids(rules: &[TriggeredRule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        let value = rule.rule_id.to_string();
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn unique_subsystem_inputs(rules: &[TriggeredRule]) -> Vec<String> {
    let mut out = Vec::new();
    for rule in rules {
        let value = rule.subsystem_input.to_string();
        if !out.contains(&value) {
            out.push(value);
        }
    }
    out
}

fn builder_proposal_id(input: Option<&RuntimeLawBuilderInput>) -> Option<String> {
    input
        .and_then(|value| value.release_state.as_ref().map(|state| state.proposal_id.clone()))
        .or_else(|| {
            input.and_then(|value| {
                value.approval_state.as_ref().map(|state| state.proposal_id.clone())
            })
        })
}

fn learning_capability_id(input: Option<&RuntimeLawLearningInput>) -> Option<String> {
    input.and_then(|value| {
        value
            .artifact_package_build
            .as_ref()
            .map(|build| format!("{:?}", build.capability_id))
    })
}

fn self_heal_fix_id(input: Option<&RuntimeLawSelfHealInput>) -> Option<String> {
    input
        .and_then(|value| value.fix_card.as_ref().map(|card| card.fix_id.clone()))
        .or_else(|| {
            input.and_then(|value| {
                value
                    .promotion_decision
                    .as_ref()
                    .map(|decision| decision.fix_id.clone())
            })
        })
}

fn response_rank(value: RuntimeLawResponseClass) -> u8 {
    match value {
        RuntimeLawResponseClass::Allow => 0,
        RuntimeLawResponseClass::AllowWithWarning => 1,
        RuntimeLawResponseClass::Degrade => 2,
        RuntimeLawResponseClass::Block => 3,
        RuntimeLawResponseClass::Quarantine => 4,
        RuntimeLawResponseClass::SafeMode => 5,
    }
}

fn severity_rank(value: RuntimeLawSeverity) -> u8 {
    match value {
        RuntimeLawSeverity::Info => 0,
        RuntimeLawSeverity::Warning => 1,
        RuntimeLawSeverity::Blocking => 2,
        RuntimeLawSeverity::Critical => 3,
        RuntimeLawSeverity::QuarantineRequired => 4,
    }
}

fn blast_radius_rank(value: RuntimeLawBlastRadiusScope) -> u8 {
    match value {
        RuntimeLawBlastRadiusScope::SubsystemScope => 0,
        RuntimeLawBlastRadiusScope::TenantScope => 1,
        RuntimeLawBlastRadiusScope::ClusterScope => 2,
        RuntimeLawBlastRadiusScope::GlobalScope => 3,
    }
}

fn default_rule_registry() -> Vec<RuntimeLawRuleDescriptor> {
    [
        (
            RULE_ALLOW_BASELINE,
            RuntimeLawRuleCategory::Envelope,
            SUBSYSTEM_RUNTIME_LAW,
            "baseline allow path when no law violations are present",
        ),
        (
            RULE_ENV_SESSION_REQUIRED,
            RuntimeLawRuleCategory::Envelope,
            SUBSYSTEM_SESSION_ENGINE,
            "protected execution requires canonical session identity",
        ),
        (
            RULE_ENV_ADMISSION_REQUIRED,
            RuntimeLawRuleCategory::Envelope,
            SUBSYSTEM_RUNTIME_LAW,
            "protected execution requires execution-admitted posture",
        ),
        (
            RULE_AUTHORITY_REQUIRED,
            RuntimeLawRuleCategory::Authority,
            SUBSYSTEM_AUTHORITY_LAYER,
            "authority-denied actions cannot complete",
        ),
        (
            RULE_IDENTITY_REQUIRED,
            RuntimeLawRuleCategory::Identity,
            SUBSYSTEM_IDENTITY_ENGINE,
            "protected identity posture must satisfy action strength requirements",
        ),
        (
            RULE_MEMORY_REQUIRED,
            RuntimeLawRuleCategory::Memory,
            SUBSYSTEM_MEMORY_ENGINE,
            "memory authority actions require eligible governed memory scope",
        ),
        (
            RULE_PERSISTENCE_STALE,
            RuntimeLawRuleCategory::Persistence,
            SUBSYSTEM_PERSISTENCE_LAYER,
            "stale persistence posture blocks final runtime-law completion",
        ),
        (
            RULE_PERSISTENCE_QUARANTINE,
            RuntimeLawRuleCategory::Persistence,
            SUBSYSTEM_PERSISTENCE_LAYER,
            "persistence quarantine escalates into runtime-law quarantine",
        ),
        (
            RULE_PROOF_REQUIRED,
            RuntimeLawRuleCategory::Proof,
            SUBSYSTEM_PROOF_ENGINE,
            "proof-required actions require successful PH1.J completion",
        ),
        (
            RULE_PROOF_CHAIN_BROKEN,
            RuntimeLawRuleCategory::Proof,
            SUBSYSTEM_PROOF_ENGINE,
            "proof chain or signature failures quarantine protected completion",
        ),
        (
            RULE_GOVERNANCE_SAFE_MODE,
            RuntimeLawRuleCategory::Governance,
            SUBSYSTEM_RUNTIME_GOVERNANCE,
            "governance safe mode or existing runtime-law safe mode blocks completion",
        ),
        (
            RULE_GOVERNANCE_DIVERGENCE,
            RuntimeLawRuleCategory::Governance,
            SUBSYSTEM_RUNTIME_GOVERNANCE,
            "governance cluster divergence degrades or safe-modes final law posture",
        ),
        (
            RULE_PLATFORM_COMPATIBILITY,
            RuntimeLawRuleCategory::Platform,
            SUBSYSTEM_PLATFORM_RUNTIME,
            "platform compatibility and integrity violations block protected execution",
        ),
        (
            RULE_PLATFORM_TRUST,
            RuntimeLawRuleCategory::Platform,
            SUBSYSTEM_PLATFORM_RUNTIME,
            "platform trust downgrade can degrade protected execution",
        ),
        (
            RULE_BUILDER_APPROVAL,
            RuntimeLawRuleCategory::Builder,
            SUBSYSTEM_BUILDER,
            "builder deployments require explicit governed builder posture",
        ),
        (
            RULE_BUILDER_ROLLBACK,
            RuntimeLawRuleCategory::Rollback,
            SUBSYSTEM_BUILDER,
            "high-risk actions require rollback readiness when policy demands it",
        ),
        (
            RULE_LEARNING_PROMOTION,
            RuntimeLawRuleCategory::Learning,
            SUBSYSTEM_LEARNING,
            "learning promotion requires explicit runtime-law approval posture",
        ),
        (
            RULE_SELF_HEAL_UNSAFE,
            RuntimeLawRuleCategory::SelfHeal,
            SUBSYSTEM_SELF_HEAL,
            "unsafe self-heal proposals cannot become authoritative runtime actions",
        ),
        (
            RULE_OVERRIDE_CONTROL,
            RuntimeLawRuleCategory::Override,
            SUBSYSTEM_OVERRIDE,
            "human overrides require authenticated and dual-controlled posture where applicable",
        ),
        (
            RULE_OVERRIDE_APPLIED,
            RuntimeLawRuleCategory::Override,
            SUBSYSTEM_OVERRIDE,
            "controlled override may downgrade a block into a recorded warning posture",
        ),
    ]
    .into_iter()
    .map(|(rule_id, category, owner, description)| {
        RuntimeLawRuleDescriptor::v1(
            rule_id.to_string(),
            category,
            true,
            "2026.03.08.law.v1".to_string(),
            owner.to_string(),
            description.to_string(),
        )
        .expect("runtime law rule descriptor must validate")
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{IdentityTierV2, SpoofLivenessStatus, UserId};
    use selene_kernel_contracts::ph1builder::{
        BuilderApprovalState, BuilderChangeClass, BuilderMetricsSnapshot,
        BuilderPostDeployDecisionAction, BuilderPostDeployJudgeResult, BuilderReleaseStage,
        BuilderReleaseState, BuilderApprovalStateStatus,
        BuilderReleaseStateStatus,
    };
    use selene_kernel_contracts::ph1j::{
        DeviceId, ProofChainStatus, ProofFailureClass, ProofVerificationPosture,
        ProofWriteOutcome, TimestampTrustPosture, TurnId,
    };
    use selene_kernel_contracts::ph1l::SessionId;
    use selene_kernel_contracts::ph1learn::{LearnArtifactPackageBuildOk, LearnTargetEngine};
    use selene_kernel_contracts::ph1link::AppPlatform;
    use selene_kernel_contracts::ph1selfheal::{
        FixCard, FixKind, FixSource, PromotionDecision, PromotionDecisionAction,
        SelfHealValidationStatus,
    };
    use selene_kernel_contracts::ph1d::SafetyTier;
    use selene_kernel_contracts::ph1pae::{PaeMode, PaeProviderSlot, PaeRouteDomain};
    use selene_kernel_contracts::runtime_execution::{
        AuthorityExecutionState, ClientCompatibilityStatus, ClientIntegrityStatus,
        DeviceTrustClass, IdentityExecutionState, IdentityRecoveryState,
        IdentityTrustTier, IdentityVerificationConsistencyLevel, MemoryConsistencyLevel,
        MemoryEligibilityDecision, MemoryExecutionState, MemoryTrustLevel,
        OnboardingReadinessState, PlatformRuntimeContext, SimulationCertificationState,
    };
    use selene_kernel_contracts::runtime_governance::{
        GovernanceClusterConsistency, GovernanceExecutionState, GovernancePolicyWindow,
        GovernanceResponseClass, GovernanceSeverity,
    };
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

    fn base_envelope() -> RuntimeExecutionEnvelope {
        RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request_law_1".to_string(),
            "trace_law_1".to_string(),
            "idem_law_1".to_string(),
            UserId::new("tenant_a:user_law_test".to_string()).unwrap(),
            DeviceId::new("device_law_1".to_string()).unwrap(),
            AppPlatform::Desktop,
            PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop).unwrap(),
            Some(SessionId(1)),
            TurnId(1),
            Some(1),
            AdmissionState::ExecutionAdmitted,
            None,
        )
        .unwrap()
        .with_governance_state(Some(
            GovernanceExecutionState::v1(
                GovernancePolicyWindow::v1(
                    "2026.03.08.v1".to_string(),
                    "2026.03.08.v1".to_string(),
                    "2026.03.08.v1".to_string(),
                )
                .unwrap()
                .governance_policy_version,
                GovernanceClusterConsistency::Consistent,
                false,
                vec![],
                vec![],
                vec![],
                Some("RG-SESSION-001".to_string()),
                Some(GovernanceSeverity::Info),
                Some(GovernanceResponseClass::Allow),
                Some("GOV-DEC-0000000001".to_string()),
            )
            .unwrap(),
        ))
        .unwrap()
        .with_identity_state(Some(
            IdentityExecutionState::v1(
                IdentityVerificationConsistencyLevel::StrictVerified,
                IdentityTrustTier::Verified,
                IdentityTierV2::Confirmed,
                SpoofLivenessStatus::Live,
                false,
                IdentityRecoveryState::None,
                false,
                None,
            )
            .unwrap(),
        ))
        .unwrap()
        .with_memory_state(Some(
            MemoryExecutionState::v1(
                true,
                MemoryConsistencyLevel::StrictLedger,
                MemoryTrustLevel::Verified,
                MemoryEligibilityDecision::Eligible,
                None,
                1,
                false,
                0,
                false,
                None,
            )
            .unwrap(),
        ))
        .unwrap()
        .with_authority_state(Some(
            AuthorityExecutionState::v1(
                Some(selene_kernel_contracts::ph1d::PolicyContextRef::v1(false, false, SafetyTier::Standard)),
                SimulationCertificationState::CertifiedActive,
                OnboardingReadinessState::Ready,
                selene_kernel_contracts::runtime_execution::AuthorityPolicyDecision::Allowed,
                true,
                true,
                true,
                None,
            )
            .unwrap(),
        ))
        .unwrap()
        .with_proof_state(Some(
            selene_kernel_contracts::runtime_execution::ProofExecutionState::v1(
                Some("proof_1".to_string()),
                ProofWriteOutcome::Written,
                None,
                ProofChainStatus::ChainLinked,
                ProofVerificationPosture::VerificationReady,
                TimestampTrustPosture::RuntimeMonotonic,
                Some("proof_meta_1".to_string()),
            )
            .unwrap(),
        ))
        .unwrap()
    }

    fn blocked_platform_envelope() -> RuntimeExecutionEnvelope {
        let mut platform_context = PlatformRuntimeContext::default_for_platform(AppPlatform::Desktop)
            .unwrap();
        platform_context.compatibility_status = ClientCompatibilityStatus::UnsupportedClient;
        platform_context.integrity_status = ClientIntegrityStatus::IntegrityFailed;
        platform_context.device_trust_class = DeviceTrustClass::UntrustedDevice;
        RuntimeExecutionEnvelope::v1_with_platform_context_device_turn_sequence_and_attach_outcome(
            "request_law_2".to_string(),
            "trace_law_2".to_string(),
            "idem_law_2".to_string(),
            UserId::new("tenant_a:user_law_test".to_string()).unwrap(),
            DeviceId::new("device_law_2".to_string()).unwrap(),
            AppPlatform::Desktop,
            platform_context,
            Some(SessionId(2)),
            TurnId(2),
            Some(1),
            AdmissionState::ExecutionAdmitted,
            None,
        )
        .unwrap()
    }

    fn builder_input(rollback_ready: bool) -> RuntimeLawBuilderInput {
        let approval = BuilderApprovalState::v1(
            "approval_1".to_string(),
            "proposal_1".to_string(),
            BuilderChangeClass::ClassC,
            2,
            2,
            true,
            true,
            BuilderApprovalStateStatus::Approved,
            ReasonCodeId(7001),
            MonotonicTimeNs(10),
            Some(MonotonicTimeNs(11)),
            Some("builder_approval_1".to_string()),
        )
        .unwrap();
        let release = BuilderReleaseState::v1(
            "release_1".to_string(),
            "proposal_1".to_string(),
            BuilderReleaseStage::Canary,
            5,
            BuilderReleaseStateStatus::Active,
            "rollback_hook_v1".to_string(),
            rollback_ready,
            ReasonCodeId(7002),
            MonotonicTimeNs(12),
            Some("builder_release_1".to_string()),
        )
        .unwrap();
        let before = BuilderMetricsSnapshot::v1(10, 20, 5, 0, 60).unwrap();
        let after = BuilderMetricsSnapshot::v1(10, 20, 5, 0, 60).unwrap();
        let judge = BuilderPostDeployJudgeResult::v1(
            "judge_1".to_string(),
            "proposal_1".to_string(),
            "release_1".to_string(),
            before,
            after,
            BuilderPostDeployDecisionAction::Accept,
            ReasonCodeId(7003),
            MonotonicTimeNs(13),
            Some("builder_judge_1".to_string()),
        )
        .unwrap();
        RuntimeLawBuilderInput::v1(Some(approval), Some(release), Some(judge)).unwrap()
    }

    fn learning_input() -> RuntimeLawLearningInput {
        RuntimeLawLearningInput::v1(Some(
            LearnArtifactPackageBuildOk::v1(
                ReasonCodeId(7101),
                LearnValidationStatus::Ok,
                vec!["LAW_REVIEW_PENDING".to_string()],
                vec![LearnTargetEngine::VoiceId],
                true,
                true,
                true,
                true,
                true,
            )
            .unwrap(),
        ))
        .unwrap()
    }

    fn self_heal_input() -> RuntimeLawSelfHealInput {
        let fix_card = FixCard::v1(
            "fix_1".to_string(),
            "problem_1".to_string(),
            FixSource::Hybrid,
            FixKind::Hybrid,
            Some("artifact_1".to_string()),
            Some(selene_kernel_contracts::ph1learn::LearnArtifactTarget::PaeRoutingWeights),
            Some(1),
            Some(100),
            Some("rollback_1".to_string()),
            Some("prov_1".to_string()),
            Some("candidate_1".to_string()),
            Some(PaeMode::Lead),
            Some(100),
            Some(10),
            Some(10),
            Some(100),
            Some(20),
            SelfHealValidationStatus::Ok,
            vec!["SAFE_POSTURE_PENDING".to_string()],
            true,
            true,
            "selfheal_fix_1".to_string(),
        )
        .unwrap();
        let promotion = PromotionDecision::v1(
            "decision_1".to_string(),
            "fix_1".to_string(),
            "tenant_1".to_string(),
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            PaeMode::Shadow,
            PaeMode::Lead,
            PromotionDecisionAction::Promote,
            10,
            100,
            3,
            1,
            "candidate_1".to_string(),
            300,
            200,
            10,
            10,
            10,
            10,
            true,
            true,
            ReasonCodeId(7201),
            true,
            true,
            true,
            Some("gov_ticket_1".to_string()),
            Some("operator_1".to_string()),
            "selfheal_promote_1".to_string(),
            MonotonicTimeNs(14),
        )
        .unwrap();
        RuntimeLawSelfHealInput::v1(Some(fix_card), Some(promotion)).unwrap()
    }

    #[test]
    fn at_runtime_law_01_conflicting_inputs_resolve_deterministically() {
        let runtime = RuntimeLawRuntime::default();
        let envelope = blocked_platform_envelope()
            .with_governance_state(base_envelope().governance_state.clone())
            .unwrap()
            .with_identity_state(base_envelope().identity_state.clone())
            .unwrap()
            .with_authority_state(base_envelope().authority_state.clone())
            .unwrap()
            .with_proof_state(base_envelope().proof_state.clone())
            .unwrap();
        let decision = runtime.evaluate(
            &envelope,
            RuntimeProtectedActionClass::ProofRequired,
            &RuntimeLawEvaluationContext::default(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Block);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_PLATFORM_COMPATIBILITY_REQUIRED.to_string()));
    }

    #[test]
    fn at_runtime_law_02_proof_failure_can_force_quarantine() {
        let runtime = RuntimeLawRuntime::default();
        let envelope = base_envelope()
            .with_proof_state(Some(
                selene_kernel_contracts::runtime_execution::ProofExecutionState::v1(
                    None,
                    ProofWriteOutcome::Failed,
                    Some(ProofFailureClass::ProofChainIntegrityFailure),
                    ProofChainStatus::ChainBreakDetected,
                    ProofVerificationPosture::VerificationUnavailable,
                    TimestampTrustPosture::RuntimeMonotonic,
                    Some("proof_meta_2".to_string()),
                )
                .unwrap(),
            ))
            .unwrap();
        let decision = runtime.evaluate(
            &envelope,
            RuntimeProtectedActionClass::ProofRequired,
            &RuntimeLawEvaluationContext::default(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Quarantine);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_PROOF_CHAIN_BROKEN.to_string()));
    }

    #[test]
    fn at_runtime_law_02b_proof_failure_under_governance_safe_mode_forces_safe_mode() {
        let runtime = RuntimeLawRuntime::default();
        let envelope = base_envelope()
            .with_governance_state(Some(
                GovernanceExecutionState::v1(
                    "2026.03.08.v1".to_string(),
                    GovernanceClusterConsistency::Consistent,
                    true,
                    vec![],
                    vec![],
                    vec![],
                    Some("RG-GOV-001".to_string()),
                    Some(GovernanceSeverity::Critical),
                    Some(GovernanceResponseClass::SafeMode),
                    Some("GOV-DEC-0000000009".to_string()),
                )
                .unwrap(),
            ))
            .unwrap()
            .with_proof_state(Some(
                selene_kernel_contracts::runtime_execution::ProofExecutionState::v1(
                    None,
                    ProofWriteOutcome::Failed,
                    Some(ProofFailureClass::ProofStorageUnavailable),
                    ProofChainStatus::NotChecked,
                    ProofVerificationPosture::VerificationUnavailable,
                    TimestampTrustPosture::RuntimeMonotonic,
                    Some("proof_meta_3".to_string()),
                )
                .unwrap(),
            ))
            .unwrap();
        let decision = runtime.evaluate(
            &envelope,
            RuntimeProtectedActionClass::ProofRequired,
            &RuntimeLawEvaluationContext::default(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::SafeMode);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_GOVERNANCE_SAFE_MODE.to_string()));
    }

    #[test]
    fn at_runtime_law_03_builder_deployment_without_rollback_ready_is_blocked() {
        let runtime = RuntimeLawRuntime::default();
        let decision = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::BuilderDeployment,
            &RuntimeLawEvaluationContext::v1(
                Some(builder_input(false)),
                None,
                None,
                None,
                MonotonicTimeNs(30),
                false,
            )
            .unwrap(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Block);
        assert_eq!(
            decision.law_state.rollback_readiness_state,
            RuntimeLawRollbackReadinessState::Missing
        );
    }

    #[test]
    fn at_runtime_law_04_learning_promotion_without_law_approval_is_blocked() {
        let runtime = RuntimeLawRuntime::default();
        let decision = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::LearningPromotion,
            &RuntimeLawEvaluationContext::v1(
                None,
                Some(learning_input()),
                None,
                None,
                MonotonicTimeNs(31),
                false,
            )
            .unwrap(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Block);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_LEARNING_PROMOTION_DENIED.to_string()));
    }

    #[test]
    fn at_runtime_law_05_self_heal_without_safe_posture_is_blocked() {
        let runtime = RuntimeLawRuntime::default();
        let decision = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::SelfHealRemediation,
            &RuntimeLawEvaluationContext::v1(
                None,
                None,
                Some(self_heal_input()),
                None,
                MonotonicTimeNs(32),
                false,
            )
            .unwrap(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Block);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_SELF_HEAL_UNSAFE.to_string()));
    }

    #[test]
    fn at_runtime_law_06_dry_run_predicts_without_executing() {
        let runtime = RuntimeLawRuntime::default();
        let out = runtime
            .govern_completion(
                &blocked_platform_envelope()
                    .with_governance_state(base_envelope().governance_state.clone())
                    .unwrap(),
                RuntimeProtectedActionClass::ProofRequired,
                &RuntimeLawEvaluationContext::v1(
                    None,
                    None,
                    None,
                    None,
                    MonotonicTimeNs(33),
                    true,
                )
                .unwrap(),
            )
            .expect("dry run must not block execution path");
        let law_state = out.law_state.expect("dry run must attach law state");
        assert!(law_state.dry_run_evaluation_state.is_some());
        assert_eq!(
            law_state.final_law_response_class,
            RuntimeLawResponseClass::Block
        );
    }

    #[test]
    fn at_runtime_law_07_override_requires_controlled_state() {
        let runtime = RuntimeLawRuntime::default();
        let decision = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::BuilderDeployment,
            &RuntimeLawEvaluationContext::v1(
                Some(builder_input(false)),
                None,
                None,
                Some(
                    RuntimeLawOverrideState::v1(
                        "operator_1".to_string(),
                        None,
                        "EMERGENCY_OVERRIDE".to_string(),
                        MonotonicTimeNs(29),
                        true,
                        false,
                        true,
                    )
                    .unwrap(),
                ),
                MonotonicTimeNs(40),
                false,
            )
            .unwrap(),
        );
        assert_eq!(decision.response_class, RuntimeLawResponseClass::Block);
        assert!(decision
            .reason_codes
            .contains(&reason_codes::LAW_OVERRIDE_CONTROL_REQUIRED.to_string()));
    }

    #[test]
    fn at_runtime_law_08_blast_radius_contains_local_builder_failure() {
        let runtime = RuntimeLawRuntime::default();
        let decision = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::BuilderDeployment,
            &RuntimeLawEvaluationContext::v1(
                Some(builder_input(false)),
                None,
                None,
                None,
                MonotonicTimeNs(41),
                false,
            )
            .unwrap(),
        );
        assert_eq!(
            decision.law_state.blast_radius_scope,
            RuntimeLawBlastRadiusScope::SubsystemScope
        );
        assert_ne!(
            decision.law_state.blast_radius_scope,
            RuntimeLawBlastRadiusScope::GlobalScope
        );
    }

    #[test]
    fn at_runtime_law_09_final_decision_is_recorded_deterministically() {
        let runtime = RuntimeLawRuntime::default();
        let first = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::ProofRequired,
            &RuntimeLawEvaluationContext::default(),
        );
        let second = runtime.evaluate(
            &base_envelope(),
            RuntimeProtectedActionClass::ProofRequired,
            &RuntimeLawEvaluationContext::default(),
        );
        assert_eq!(first.response_class, second.response_class);
        let log = runtime.decision_log_snapshot();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].reason_codes, log[1].reason_codes);
        assert_eq!(log[0].final_response_class, log[1].final_response_class);
    }
}
