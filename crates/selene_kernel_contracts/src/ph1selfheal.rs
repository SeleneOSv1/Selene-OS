#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1d::Ph1dProviderTask;
use crate::ph1feedback::{
    FeedbackConfidenceBucket, FeedbackEventType, FeedbackPathType, FeedbackToolStatus,
};
use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1learn::{LearnArtifactTarget, LearnScope};
use crate::ph1pae::{PaeMode, PaeProviderSlot, PaeRouteDomain};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SELFHEAL_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FailureContainmentAction {
    FailClosedRefuse,
    ClarifyRequired,
    RetryScheduled,
    Escalated,
    ObservedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProblemCardState {
    Open,
    Verifying,
    Resolved,
    EscalatedOpen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixSource {
    Learn,
    Pae,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixKind {
    Artifact,
    RoutingPolicy,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SelfHealValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PromotionDecisionAction {
    Promote,
    Demote,
    Hold,
    Rollback,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureProviderContext {
    pub route_domain: PaeRouteDomain,
    pub provider_slot: PaeProviderSlot,
    pub provider_task: Ph1dProviderTask,
    pub provider_cost_microunits: u64,
    pub provider_latency_ms: u32,
    pub fallback_to_local: bool,
}

impl FailureProviderContext {
    pub fn v1(
        route_domain: PaeRouteDomain,
        provider_slot: PaeProviderSlot,
        provider_task: Ph1dProviderTask,
        provider_cost_microunits: u64,
        provider_latency_ms: u32,
        fallback_to_local: bool,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            route_domain,
            provider_slot,
            provider_task,
            provider_cost_microunits,
            provider_latency_ms,
            fallback_to_local,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for FailureProviderContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.provider_latency_ms > 120_000 {
            return Err(ContractViolation::InvalidValue {
                field: "failure_provider_context.provider_latency_ms",
                reason: "must be <= 120000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailureEvent {
    pub schema_version: SchemaVersion,
    pub failure_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub speaker_id: String,
    pub session_id: String,
    pub device_id: String,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub event_type: FeedbackEventType,
    pub reason_code: ReasonCodeId,
    pub path_type: FeedbackPathType,
    pub evidence_ref: String,
    pub idempotency_key: String,
    pub confidence_bucket: FeedbackConfidenceBucket,
    pub tool_status: FeedbackToolStatus,
    pub latency_ms: u32,
    pub retries: u8,
    pub missing_fields: Vec<String>,
    pub fingerprint: String,
    pub containment_action: FailureContainmentAction,
    pub escalation_required: bool,
    pub unresolved_reason: Option<String>,
    pub provider_context: Option<FailureProviderContext>,
}

impl FailureEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        failure_id: String,
        tenant_id: String,
        user_id: String,
        speaker_id: String,
        session_id: String,
        device_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        event_type: FeedbackEventType,
        reason_code: ReasonCodeId,
        path_type: FeedbackPathType,
        evidence_ref: String,
        idempotency_key: String,
        confidence_bucket: FeedbackConfidenceBucket,
        tool_status: FeedbackToolStatus,
        latency_ms: u32,
        retries: u8,
        missing_fields: Vec<String>,
        fingerprint: String,
        containment_action: FailureContainmentAction,
        escalation_required: bool,
        unresolved_reason: Option<String>,
        provider_context: Option<FailureProviderContext>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1SELFHEAL_CONTRACT_VERSION,
            failure_id,
            tenant_id,
            user_id,
            speaker_id,
            session_id,
            device_id,
            correlation_id,
            turn_id,
            event_type,
            reason_code,
            path_type,
            evidence_ref,
            idempotency_key,
            confidence_bucket,
            tool_status,
            latency_ms,
            retries,
            missing_fields,
            fingerprint,
            containment_action,
            escalation_required,
            unresolved_reason,
            provider_context,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for FailureEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SELFHEAL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "failure_event.schema_version",
                reason: "must match PH1SELFHEAL_CONTRACT_VERSION",
            });
        }
        validate_token("failure_event.failure_id", &self.failure_id, 96)?;
        validate_token("failure_event.tenant_id", &self.tenant_id, 64)?;
        validate_token("failure_event.user_id", &self.user_id, 64)?;
        validate_token("failure_event.speaker_id", &self.speaker_id, 64)?;
        validate_token("failure_event.session_id", &self.session_id, 96)?;
        validate_token("failure_event.device_id", &self.device_id, 128)?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("failure_event.evidence_ref", &self.evidence_ref, 128)?;
        validate_token("failure_event.idempotency_key", &self.idempotency_key, 128)?;
        if self.latency_ms > 600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "failure_event.latency_ms",
                reason: "must be <= 600000",
            });
        }
        if self.retries > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "failure_event.retries",
                reason: "must be <= 32",
            });
        }
        if self.missing_fields.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "failure_event.missing_fields",
                reason: "must be <= 32 entries",
            });
        }
        let mut missing_set = BTreeSet::new();
        for field in &self.missing_fields {
            validate_token("failure_event.missing_fields", field, 64)?;
            if !missing_set.insert(field.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "failure_event.missing_fields",
                    reason: "must be unique",
                });
            }
        }
        validate_token("failure_event.fingerprint", &self.fingerprint, 128)?;
        if self.escalation_required {
            let unresolved_reason =
                self.unresolved_reason
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "failure_event.unresolved_reason",
                        reason: "must be present when escalation_required=true",
                    })?;
            validate_text("failure_event.unresolved_reason", unresolved_reason, 512)?;
        } else if self.unresolved_reason.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "failure_event.unresolved_reason",
                reason: "must be absent when escalation_required=false",
            });
        }
        if let Some(provider_context) = &self.provider_context {
            provider_context.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemCard {
    pub schema_version: SchemaVersion,
    pub problem_id: String,
    pub fingerprint: String,
    pub tenant_id: String,
    pub owner_engine: String,
    pub scope_hint: LearnScope,
    pub scope_ref: Option<String>,
    pub first_seen_at: MonotonicTimeNs,
    pub last_seen_at: MonotonicTimeNs,
    pub recurrence_count: u32,
    pub latest_failure_id: String,
    pub signal_ids: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub quality_impact_bp: i16,
    pub latency_impact_bp: i16,
    pub cost_impact_bp: i16,
    pub state: ProblemCardState,
    pub requires_human: bool,
    pub bcast_id: Option<String>,
    pub unresolved_reason: Option<String>,
    pub idempotency_key: String,
}

impl ProblemCard {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        problem_id: String,
        fingerprint: String,
        tenant_id: String,
        owner_engine: String,
        scope_hint: LearnScope,
        scope_ref: Option<String>,
        first_seen_at: MonotonicTimeNs,
        last_seen_at: MonotonicTimeNs,
        recurrence_count: u32,
        latest_failure_id: String,
        signal_ids: Vec<String>,
        evidence_refs: Vec<String>,
        quality_impact_bp: i16,
        latency_impact_bp: i16,
        cost_impact_bp: i16,
        state: ProblemCardState,
        requires_human: bool,
        bcast_id: Option<String>,
        unresolved_reason: Option<String>,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1SELFHEAL_CONTRACT_VERSION,
            problem_id,
            fingerprint,
            tenant_id,
            owner_engine,
            scope_hint,
            scope_ref,
            first_seen_at,
            last_seen_at,
            recurrence_count,
            latest_failure_id,
            signal_ids,
            evidence_refs,
            quality_impact_bp,
            latency_impact_bp,
            cost_impact_bp,
            state,
            requires_human,
            bcast_id,
            unresolved_reason,
            idempotency_key,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for ProblemCard {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SELFHEAL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.schema_version",
                reason: "must match PH1SELFHEAL_CONTRACT_VERSION",
            });
        }
        validate_token("problem_card.problem_id", &self.problem_id, 96)?;
        validate_token("problem_card.fingerprint", &self.fingerprint, 128)?;
        validate_token("problem_card.tenant_id", &self.tenant_id, 64)?;
        validate_engine_id("problem_card.owner_engine", &self.owner_engine, 64)?;
        if self.last_seen_at.0 < self.first_seen_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.last_seen_at",
                reason: "must be >= first_seen_at",
            });
        }
        if self.recurrence_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.recurrence_count",
                reason: "must be > 0",
            });
        }
        if self.recurrence_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.recurrence_count",
                reason: "must be <= 1000000",
            });
        }
        validate_token(
            "problem_card.latest_failure_id",
            &self.latest_failure_id,
            96,
        )?;
        validate_string_set("problem_card.signal_ids", &self.signal_ids, 128, 96, true)?;
        validate_string_set(
            "problem_card.evidence_refs",
            &self.evidence_refs,
            128,
            128,
            false,
        )?;
        validate_bp(
            "problem_card.quality_impact_bp",
            self.quality_impact_bp,
            20_000,
        )?;
        validate_bp(
            "problem_card.latency_impact_bp",
            self.latency_impact_bp,
            20_000,
        )?;
        validate_bp("problem_card.cost_impact_bp", self.cost_impact_bp, 20_000)?;
        validate_token("problem_card.idempotency_key", &self.idempotency_key, 128)?;

        match self.scope_hint {
            LearnScope::User | LearnScope::Tenant => {
                let scope_ref = self
                    .scope_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "problem_card.scope_ref",
                        reason: "must be present when scope_hint=USER|TENANT",
                    })?;
                validate_token("problem_card.scope_ref", scope_ref, 64)?;
            }
            LearnScope::GlobalDerived => {
                if self.scope_ref.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "problem_card.scope_ref",
                        reason: "must be absent when scope_hint=GLOBAL_DERIVED",
                    });
                }
            }
        }

        if let Some(bcast_id) = &self.bcast_id {
            validate_token("problem_card.bcast_id", bcast_id, 96)?;
        }

        let unresolved_required =
            self.requires_human || matches!(self.state, ProblemCardState::EscalatedOpen);
        if unresolved_required {
            let unresolved_reason =
                self.unresolved_reason
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "problem_card.unresolved_reason",
                        reason: "must be present when requires_human=true or state=ESCALATED_OPEN",
                    })?;
            validate_text("problem_card.unresolved_reason", unresolved_reason, 512)?;
        } else if self.unresolved_reason.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.unresolved_reason",
                reason: "must be absent when unresolved state is not active",
            });
        }

        if matches!(self.state, ProblemCardState::Resolved) && self.requires_human {
            return Err(ContractViolation::InvalidValue {
                field: "problem_card.requires_human",
                reason: "must be false when state=RESOLVED",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FixCard {
    pub schema_version: SchemaVersion,
    pub fix_id: String,
    pub problem_id: String,
    pub fix_source: FixSource,
    pub fix_kind: FixKind,
    pub artifact_id: Option<String>,
    pub artifact_target: Option<LearnArtifactTarget>,
    pub artifact_version: Option<u32>,
    pub expected_effect_bp: Option<i16>,
    pub rollback_to: Option<String>,
    pub provenance_ref: Option<String>,
    pub selected_candidate_id: Option<String>,
    pub selected_mode: Option<PaeMode>,
    pub expected_quality_bp: Option<i16>,
    pub expected_latency_ms: Option<u16>,
    pub expected_cost_bp: Option<i16>,
    pub regression_risk_bp: Option<u16>,
    pub sample_size: Option<u16>,
    pub validation_status: SelfHealValidationStatus,
    pub diagnostics: Vec<String>,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
    pub idempotency_key: String,
}

impl FixCard {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        fix_id: String,
        problem_id: String,
        fix_source: FixSource,
        fix_kind: FixKind,
        artifact_id: Option<String>,
        artifact_target: Option<LearnArtifactTarget>,
        artifact_version: Option<u32>,
        expected_effect_bp: Option<i16>,
        rollback_to: Option<String>,
        provenance_ref: Option<String>,
        selected_candidate_id: Option<String>,
        selected_mode: Option<PaeMode>,
        expected_quality_bp: Option<i16>,
        expected_latency_ms: Option<u16>,
        expected_cost_bp: Option<i16>,
        regression_risk_bp: Option<u16>,
        sample_size: Option<u16>,
        validation_status: SelfHealValidationStatus,
        diagnostics: Vec<String>,
        advisory_only: bool,
        no_execution_authority: bool,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1SELFHEAL_CONTRACT_VERSION,
            fix_id,
            problem_id,
            fix_source,
            fix_kind,
            artifact_id,
            artifact_target,
            artifact_version,
            expected_effect_bp,
            rollback_to,
            provenance_ref,
            selected_candidate_id,
            selected_mode,
            expected_quality_bp,
            expected_latency_ms,
            expected_cost_bp,
            regression_risk_bp,
            sample_size,
            validation_status,
            diagnostics,
            advisory_only,
            no_execution_authority,
            idempotency_key,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for FixCard {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SELFHEAL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card.schema_version",
                reason: "must match PH1SELFHEAL_CONTRACT_VERSION",
            });
        }
        validate_token("fix_card.fix_id", &self.fix_id, 96)?;
        validate_token("fix_card.problem_id", &self.problem_id, 96)?;
        validate_token("fix_card.idempotency_key", &self.idempotency_key, 128)?;

        if let Some(artifact_id) = &self.artifact_id {
            validate_token("fix_card.artifact_id", artifact_id, 128)?;
        }
        if let Some(artifact_version) = self.artifact_version {
            if artifact_version == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "fix_card.artifact_version",
                    reason: "must be > 0",
                });
            }
        }
        if let Some(expected_effect_bp) = self.expected_effect_bp {
            validate_bp("fix_card.expected_effect_bp", expected_effect_bp, 20_000)?;
        }
        if let Some(rollback_to) = &self.rollback_to {
            validate_token("fix_card.rollback_to", rollback_to, 128)?;
        }
        if let Some(provenance_ref) = &self.provenance_ref {
            validate_token("fix_card.provenance_ref", provenance_ref, 128)?;
        }

        if let Some(selected_candidate_id) = &self.selected_candidate_id {
            validate_token("fix_card.selected_candidate_id", selected_candidate_id, 128)?;
        }
        if let Some(expected_quality_bp) = self.expected_quality_bp {
            validate_bp("fix_card.expected_quality_bp", expected_quality_bp, 10_000)?;
        }
        if let Some(expected_latency_ms) = self.expected_latency_ms {
            if expected_latency_ms == 0 || expected_latency_ms > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "fix_card.expected_latency_ms",
                    reason: "must be within 1..=10000",
                });
            }
        }
        if let Some(expected_cost_bp) = self.expected_cost_bp {
            validate_bp("fix_card.expected_cost_bp", expected_cost_bp, 10_000)?;
        }
        if let Some(regression_risk_bp) = self.regression_risk_bp {
            if regression_risk_bp > 10_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "fix_card.regression_risk_bp",
                    reason: "must be <= 10000",
                });
            }
        }
        if let Some(sample_size) = self.sample_size {
            if sample_size == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "fix_card.sample_size",
                    reason: "must be > 0",
                });
            }
        }

        if self.diagnostics.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card.diagnostics",
                reason: "must be <= 32 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_text("fix_card.diagnostics", diagnostic, 256)?;
        }

        let has_artifact_payload = self.artifact_id.is_some()
            || self.artifact_target.is_some()
            || self.artifact_version.is_some()
            || self.expected_effect_bp.is_some()
            || self.provenance_ref.is_some();
        let has_policy_payload = self.selected_candidate_id.is_some()
            || self.selected_mode.is_some()
            || self.expected_quality_bp.is_some()
            || self.expected_latency_ms.is_some()
            || self.expected_cost_bp.is_some()
            || self.regression_risk_bp.is_some()
            || self.sample_size.is_some();

        if !has_artifact_payload && !has_policy_payload {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card",
                reason: "must include artifact payload, policy payload, or both",
            });
        }
        if matches!(self.fix_kind, FixKind::Artifact) && !has_artifact_payload {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card.fix_kind",
                reason: "ARTIFACT fix_kind requires artifact payload fields",
            });
        }
        if matches!(self.fix_kind, FixKind::RoutingPolicy) && !has_policy_payload {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card.fix_kind",
                reason: "ROUTING_POLICY fix_kind requires policy payload fields",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "fix_card.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromotionDecision {
    pub schema_version: SchemaVersion,
    pub decision_id: String,
    pub fix_id: String,
    pub tenant_id: String,
    pub route_domain: PaeRouteDomain,
    pub provider_slot: PaeProviderSlot,
    pub from_mode: PaeMode,
    pub to_mode: PaeMode,
    pub decision_action: PromotionDecisionAction,
    pub minimum_sample_size: u16,
    pub promotion_threshold_bp: i16,
    pub demotion_failure_threshold: u8,
    pub consecutive_threshold_failures: u8,
    pub selected_candidate_id: String,
    pub total_score_bp: i32,
    pub quality_score_bp: i16,
    pub latency_penalty_bp: i16,
    pub cost_penalty_bp: i16,
    pub regression_penalty_bp: i16,
    pub sample_size: u16,
    pub promotion_eligible: bool,
    pub rollback_ready: bool,
    pub reason_code: ReasonCodeId,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
    pub governance_required: bool,
    pub governance_ticket_ref: Option<String>,
    pub approved_by: Option<String>,
    pub idempotency_key: String,
    pub evaluated_at: MonotonicTimeNs,
}

impl PromotionDecision {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        decision_id: String,
        fix_id: String,
        tenant_id: String,
        route_domain: PaeRouteDomain,
        provider_slot: PaeProviderSlot,
        from_mode: PaeMode,
        to_mode: PaeMode,
        decision_action: PromotionDecisionAction,
        minimum_sample_size: u16,
        promotion_threshold_bp: i16,
        demotion_failure_threshold: u8,
        consecutive_threshold_failures: u8,
        selected_candidate_id: String,
        total_score_bp: i32,
        quality_score_bp: i16,
        latency_penalty_bp: i16,
        cost_penalty_bp: i16,
        regression_penalty_bp: i16,
        sample_size: u16,
        promotion_eligible: bool,
        rollback_ready: bool,
        reason_code: ReasonCodeId,
        advisory_only: bool,
        no_execution_authority: bool,
        governance_required: bool,
        governance_ticket_ref: Option<String>,
        approved_by: Option<String>,
        idempotency_key: String,
        evaluated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1SELFHEAL_CONTRACT_VERSION,
            decision_id,
            fix_id,
            tenant_id,
            route_domain,
            provider_slot,
            from_mode,
            to_mode,
            decision_action,
            minimum_sample_size,
            promotion_threshold_bp,
            demotion_failure_threshold,
            consecutive_threshold_failures,
            selected_candidate_id,
            total_score_bp,
            quality_score_bp,
            latency_penalty_bp,
            cost_penalty_bp,
            regression_penalty_bp,
            sample_size,
            promotion_eligible,
            rollback_ready,
            reason_code,
            advisory_only,
            no_execution_authority,
            governance_required,
            governance_ticket_ref,
            approved_by,
            idempotency_key,
            evaluated_at,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for PromotionDecision {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SELFHEAL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.schema_version",
                reason: "must match PH1SELFHEAL_CONTRACT_VERSION",
            });
        }
        validate_token("promotion_decision.decision_id", &self.decision_id, 96)?;
        validate_token("promotion_decision.fix_id", &self.fix_id, 96)?;
        validate_token("promotion_decision.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "promotion_decision.selected_candidate_id",
            &self.selected_candidate_id,
            128,
        )?;
        validate_token(
            "promotion_decision.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        if self.minimum_sample_size < 10 || self.minimum_sample_size > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.minimum_sample_size",
                reason: "must be within 10..=10000",
            });
        }
        validate_bp(
            "promotion_decision.promotion_threshold_bp",
            self.promotion_threshold_bp,
            10_000,
        )?;
        if self.demotion_failure_threshold == 0 || self.demotion_failure_threshold > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.demotion_failure_threshold",
                reason: "must be within 1..=32",
            });
        }
        if self.consecutive_threshold_failures > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.consecutive_threshold_failures",
                reason: "must be <= 64",
            });
        }
        if self.total_score_bp < -50_000 || self.total_score_bp > 50_000 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.total_score_bp",
                reason: "must be within -50000..=50000",
            });
        }
        validate_bp(
            "promotion_decision.quality_score_bp",
            self.quality_score_bp,
            10_000,
        )?;
        validate_bp(
            "promotion_decision.latency_penalty_bp",
            self.latency_penalty_bp,
            10_000,
        )?;
        validate_bp(
            "promotion_decision.cost_penalty_bp",
            self.cost_penalty_bp,
            10_000,
        )?;
        validate_bp(
            "promotion_decision.regression_penalty_bp",
            self.regression_penalty_bp,
            10_000,
        )?;
        if self.sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.sample_size",
                reason: "must be > 0",
            });
        }
        if self.sample_size < self.minimum_sample_size {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.sample_size",
                reason: "must be >= minimum_sample_size",
            });
        }
        let from_rank = pae_mode_rank(self.from_mode);
        let to_rank = pae_mode_rank(self.to_mode);
        if from_rank.abs_diff(to_rank) > 1 {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.to_mode",
                reason: "must be one-step ladder transition from from_mode",
            });
        }
        if matches!(self.to_mode, PaeMode::Lead) && !self.rollback_ready {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.rollback_ready",
                reason: "must be true when to_mode=LEAD",
            });
        }
        if matches!(self.decision_action, PromotionDecisionAction::Promote)
            && !self.promotion_eligible
        {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.promotion_eligible",
                reason: "must be true for decision_action=PROMOTE",
            });
        }
        if matches!(self.decision_action, PromotionDecisionAction::Promote) && to_rank <= from_rank
        {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.decision_action",
                reason: "PROMOTE requires to_mode > from_mode",
            });
        }
        if matches!(self.decision_action, PromotionDecisionAction::Demote) && to_rank >= from_rank {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.decision_action",
                reason: "DEMOTE requires to_mode < from_mode",
            });
        }
        if matches!(self.decision_action, PromotionDecisionAction::Hold) && to_rank != from_rank {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.decision_action",
                reason: "HOLD requires to_mode == from_mode",
            });
        }
        if matches!(self.decision_action, PromotionDecisionAction::Rollback)
            && !(self.from_mode == PaeMode::Lead && self.to_mode == PaeMode::Assist)
        {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.decision_action",
                reason: "ROLLBACK requires LEAD -> ASSIST transition",
            });
        }
        if self.from_mode == PaeMode::Lead && to_rank < from_rank && !self.rollback_ready {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.rollback_ready",
                reason: "lead demotion requires rollback_ready=true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.no_execution_authority",
                reason: "must be true",
            });
        }
        if self.governance_required {
            let ticket =
                self.governance_ticket_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "promotion_decision.governance_ticket_ref",
                        reason: "must be present when governance_required=true",
                    })?;
            validate_token("promotion_decision.governance_ticket_ref", ticket, 128)?;
        } else if self.governance_ticket_ref.is_some() {
            return Err(ContractViolation::InvalidValue {
                field: "promotion_decision.governance_ticket_ref",
                reason: "must be absent when governance_required=false",
            });
        }
        if let Some(approved_by) = &self.approved_by {
            validate_token("promotion_decision.approved_by", approved_by, 96)?;
            if !self.governance_required {
                return Err(ContractViolation::InvalidValue {
                    field: "promotion_decision.approved_by",
                    reason: "must be absent when governance_required=false",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelfHealCardChain {
    pub schema_version: SchemaVersion,
    pub failure_event: FailureEvent,
    pub problem_card: ProblemCard,
    pub fix_card: FixCard,
    pub promotion_decision: PromotionDecision,
}

impl SelfHealCardChain {
    pub fn v1(
        failure_event: FailureEvent,
        problem_card: ProblemCard,
        fix_card: FixCard,
        promotion_decision: PromotionDecision,
    ) -> Result<Self, ContractViolation> {
        let chain = Self {
            schema_version: PH1SELFHEAL_CONTRACT_VERSION,
            failure_event,
            problem_card,
            fix_card,
            promotion_decision,
        };
        chain.validate()?;
        Ok(chain)
    }
}

impl Validate for SelfHealCardChain {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SELFHEAL_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.schema_version",
                reason: "must match PH1SELFHEAL_CONTRACT_VERSION",
            });
        }
        self.failure_event.validate()?;
        self.problem_card.validate()?;
        self.fix_card.validate()?;
        self.promotion_decision.validate()?;

        if self.problem_card.latest_failure_id != self.failure_event.failure_id {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.problem_card.latest_failure_id",
                reason: "must equal failure_event.failure_id",
            });
        }
        if self.problem_card.fingerprint != self.failure_event.fingerprint {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.problem_card.fingerprint",
                reason: "must equal failure_event.fingerprint",
            });
        }
        if self.fix_card.problem_id != self.problem_card.problem_id {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.fix_card.problem_id",
                reason: "must equal problem_card.problem_id",
            });
        }
        if self.promotion_decision.fix_id != self.fix_card.fix_id {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.promotion_decision.fix_id",
                reason: "must equal fix_card.fix_id",
            });
        }
        if self.failure_event.tenant_id != self.problem_card.tenant_id
            || self.problem_card.tenant_id != self.promotion_decision.tenant_id
        {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.tenant_id",
                reason: "must be identical across failure/problem/promotion cards",
            });
        }
        if self.failure_event.escalation_required && !self.problem_card.requires_human {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.problem_card.requires_human",
                reason: "must be true when failure_event.escalation_required=true",
            });
        }
        if matches!(self.promotion_decision.to_mode, PaeMode::Lead)
            && !self.promotion_decision.rollback_ready
        {
            return Err(ContractViolation::InvalidValue {
                field: "self_heal_card_chain.promotion_decision.rollback_ready",
                reason: "to_mode=LEAD requires rollback_ready=true",
            });
        }
        Ok(())
    }
}

pub fn stable_card_id(prefix: &str, parts: &[&str]) -> Result<String, ContractViolation> {
    validate_token("stable_card_id.prefix", prefix, 24)?;
    if parts.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "stable_card_id.parts",
            reason: "must be non-empty",
        });
    }
    let mut buf = String::new();
    for part in parts {
        if part.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "stable_card_id.parts",
                reason: "must not include empty parts",
            });
        }
        buf.push_str(part);
        buf.push('|');
    }
    let hash = fnv1a64(buf.as_bytes());
    Ok(format!("{}_{}", prefix, format!("{hash:016x}")))
}

fn validate_string_set(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_len: usize,
    token_only: bool,
) -> Result<(), ContractViolation> {
    if values.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max item count",
        });
    }
    let mut seen = BTreeSet::new();
    for value in values {
        if token_only {
            validate_token(field, value, max_len)?;
        } else {
            validate_text(field, value, max_len)?;
        }
        if !seen.insert(value.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must be unique",
            });
        }
    }
    Ok(())
}

fn validate_bp(field: &'static str, value: i16, bound: i16) -> Result<(), ContractViolation> {
    if value.abs() > bound {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "basis points out of allowed range",
        });
    }
    Ok(())
}

fn pae_mode_rank(mode: PaeMode) -> u8 {
    match mode {
        PaeMode::Shadow => 0,
        PaeMode::Assist => 1,
        PaeMode::Lead => 2,
    }
}

fn validate_engine_id(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    validate_token(field, value, max_len)?;
    if !value.starts_with("PH1.") && !value.starts_with("PH2.") {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must start with PH1. or PH2.",
        });
    }
    Ok(())
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
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

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not include control chars",
        });
    }
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x100000001b3;
    let mut h = OFFSET;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(PRIME);
    }
    if h == 0 {
        1
    } else {
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_failure_event() -> FailureEvent {
        FailureEvent::v1(
            "failure_1".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(77),
            TurnId(88),
            FeedbackEventType::ToolFail,
            ReasonCodeId(11),
            FeedbackPathType::Defect,
            "evidence:1".to_string(),
            "idem:failure:1".to_string(),
            FeedbackConfidenceBucket::Low,
            FeedbackToolStatus::Fail,
            900,
            1,
            vec!["field_a".to_string()],
            "fp_a".to_string(),
            FailureContainmentAction::FailClosedRefuse,
            true,
            Some("provider timeout unresolved".to_string()),
            Some(
                FailureProviderContext::v1(
                    PaeRouteDomain::Tooling,
                    PaeProviderSlot::Primary,
                    Ph1dProviderTask::OcrTextExtract,
                    14_000,
                    88,
                    false,
                )
                .unwrap(),
            ),
        )
        .unwrap()
    }

    fn sample_problem_card(failure_id: &str, fingerprint: &str) -> ProblemCard {
        ProblemCard::v1(
            "problem_1".to_string(),
            fingerprint.to_string(),
            "tenant_1".to_string(),
            "PH1.OS".to_string(),
            LearnScope::Tenant,
            Some("tenant_1".to_string()),
            MonotonicTimeNs(1),
            MonotonicTimeNs(2),
            3,
            failure_id.to_string(),
            vec!["signal_1".to_string()],
            vec!["evidence:1".to_string()],
            -400,
            220,
            180,
            ProblemCardState::EscalatedOpen,
            true,
            Some("bcast_100".to_string()),
            Some("human review required".to_string()),
            "idem:problem:1".to_string(),
        )
        .unwrap()
    }

    fn sample_fix_card(problem_id: &str) -> FixCard {
        FixCard::v1(
            "fix_1".to_string(),
            problem_id.to_string(),
            FixSource::Hybrid,
            FixKind::Hybrid,
            Some("artifact_1".to_string()),
            Some(LearnArtifactTarget::PaeRoutingWeights),
            Some(2),
            Some(120),
            Some("artifact_prev".to_string()),
            Some("prov:1".to_string()),
            Some("candidate_1".to_string()),
            Some(PaeMode::Assist),
            Some(200),
            Some(120),
            Some(100),
            Some(80),
            Some(120),
            SelfHealValidationStatus::Ok,
            vec!["diag".to_string()],
            true,
            true,
            "idem:fix:1".to_string(),
        )
        .unwrap()
    }

    fn sample_promotion_decision(fix_id: &str) -> PromotionDecision {
        PromotionDecision::v1(
            "decision_1".to_string(),
            fix_id.to_string(),
            "tenant_1".to_string(),
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            PaeMode::Lead,
            PromotionDecisionAction::Promote,
            100,
            800,
            3,
            0,
            "candidate_1".to_string(),
            1900,
            2400,
            180,
            120,
            100,
            180,
            true,
            true,
            ReasonCodeId(44),
            true,
            true,
            true,
            Some("gov_ticket_1".to_string()),
            Some("owner_1".to_string()),
            "idem:decision:1".to_string(),
            MonotonicTimeNs(10),
        )
        .unwrap()
    }

    #[test]
    fn at_selfheal_01_chain_holds_cross_links() {
        let failure = sample_failure_event();
        let problem = sample_problem_card(&failure.failure_id, &failure.fingerprint);
        let fix = sample_fix_card(&problem.problem_id);
        let decision = sample_promotion_decision(&fix.fix_id);
        let chain = SelfHealCardChain::v1(failure, problem, fix, decision).unwrap();
        assert!(chain.validate().is_ok());
    }

    #[test]
    fn at_selfheal_02_chain_fails_closed_on_missing_fix_link() {
        let failure = sample_failure_event();
        let problem = sample_problem_card(&failure.failure_id, &failure.fingerprint);
        let mut fix = sample_fix_card(&problem.problem_id);
        fix.problem_id = "problem_x".to_string();
        let decision = sample_promotion_decision(&fix.fix_id);
        let err = SelfHealCardChain::v1(failure, problem, fix, decision)
            .expect_err("chain mismatch must fail closed");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "self_heal_card_chain.fix_card.problem_id")
            }
            _ => panic!("expected invalid value"),
        }
    }

    #[test]
    fn at_selfheal_03_stable_card_id_is_deterministic() {
        let id_a = stable_card_id("problem", &["tenant_1", "fp_1", "signal_1"]).unwrap();
        let id_b = stable_card_id("problem", &["tenant_1", "fp_1", "signal_1"]).unwrap();
        assert_eq!(id_a, id_b);
    }

    #[test]
    fn at_selfheal_04_promotion_decision_rejects_direct_shadow_to_lead_jump() {
        let err = PromotionDecision::v1(
            "decision_jump_1".to_string(),
            "fix_1".to_string(),
            "tenant_1".to_string(),
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            PaeMode::Shadow,
            PaeMode::Lead,
            PromotionDecisionAction::Promote,
            100,
            800,
            3,
            0,
            "candidate_1".to_string(),
            1900,
            2400,
            180,
            120,
            100,
            180,
            true,
            true,
            ReasonCodeId(44),
            true,
            true,
            true,
            Some("gov_ticket_1".to_string()),
            Some("owner_1".to_string()),
            "idem:decision:jump:1".to_string(),
            MonotonicTimeNs(10),
        )
        .expect_err("direct SHADOW->LEAD jump must fail closed");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "promotion_decision.to_mode")
            }
            other => panic!("expected invalid value, got {other:?}"),
        }
    }

    #[test]
    fn at_selfheal_05_lead_demotion_requires_rollback_ready() {
        let err = PromotionDecision::v1(
            "decision_rollback_1".to_string(),
            "fix_1".to_string(),
            "tenant_1".to_string(),
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            PaeMode::Lead,
            PaeMode::Assist,
            PromotionDecisionAction::Rollback,
            100,
            800,
            3,
            3,
            "candidate_1".to_string(),
            1400,
            1800,
            220,
            180,
            900,
            180,
            false,
            false,
            ReasonCodeId(45),
            true,
            true,
            true,
            Some("gov_ticket_1".to_string()),
            Some("owner_1".to_string()),
            "idem:decision:rollback:1".to_string(),
            MonotonicTimeNs(11),
        )
        .expect_err("lead demotion without rollback pointer must fail closed");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "promotion_decision.rollback_ready")
            }
            other => panic!("expected invalid value, got {other:?}"),
        }
    }

    #[test]
    fn at_selfheal_06_hold_action_requires_no_mode_change() {
        let err = PromotionDecision::v1(
            "decision_hold_1".to_string(),
            "fix_1".to_string(),
            "tenant_1".to_string(),
            PaeRouteDomain::Tooling,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            PaeMode::Shadow,
            PromotionDecisionAction::Hold,
            100,
            800,
            3,
            1,
            "candidate_1".to_string(),
            900,
            1200,
            300,
            200,
            700,
            180,
            false,
            true,
            ReasonCodeId(46),
            true,
            true,
            true,
            Some("gov_ticket_1".to_string()),
            Some("owner_1".to_string()),
            "idem:decision:hold:1".to_string(),
            MonotonicTimeNs(12),
        )
        .expect_err("HOLD with a mode change must fail closed");
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "promotion_decision.decision_action")
            }
            other => panic!("expected invalid value, got {other:?}"),
        }
    }
}
