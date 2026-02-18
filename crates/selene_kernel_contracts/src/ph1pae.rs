#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PAE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeCapabilityId {
    PaePolicyScoreBuild,
    PaeAdaptationHintEmit,
}

impl PaeCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PaeCapabilityId::PaePolicyScoreBuild => "PAE_POLICY_SCORE_BUILD",
            PaeCapabilityId::PaeAdaptationHintEmit => "PAE_ADAPTATION_HINT_EMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeRouteDomain {
    Stt,
    Tts,
    Llm,
    Tooling,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PaeMode {
    Shadow,
    Assist,
    Lead,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeProviderSlot {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeSignalSource {
    Listen,
    Feedback,
    Learn,
    RllGoverned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeTargetEngine {
    Ph1C,
    Ph1Tts,
    Ph1Cache,
    Ph1Multi,
}

impl PaeTargetEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            PaeTargetEngine::Ph1C => "PH1.C",
            PaeTargetEngine::Ph1Tts => "PH1.TTS",
            PaeTargetEngine::Ph1Cache => "PH1.CACHE",
            PaeTargetEngine::Ph1Multi => "PH1.MULTI",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PaeValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signals: u8,
    pub max_candidates: u8,
    pub max_scores: u8,
    pub max_hints: u8,
    pub max_diagnostics: u8,
}

impl PaeRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signals: u8,
        max_candidates: u8,
        max_scores: u8,
        max_hints: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signals,
            max_candidates,
            max_scores,
            max_hints,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PaeRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signals == 0 || self.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.max_signals",
                reason: "must be within 1..=64",
            });
        }
        if self.max_candidates == 0 || self.max_candidates > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.max_candidates",
                reason: "must be within 1..=32",
            });
        }
        if self.max_scores == 0 || self.max_scores > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.max_scores",
                reason: "must be within 1..=32",
            });
        }
        if self.max_hints == 0 || self.max_hints > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.max_hints",
                reason: "must be within 1..=16",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeSignalVector {
    pub schema_version: SchemaVersion,
    pub signal_id: String,
    pub source: PaeSignalSource,
    pub route_domain: PaeRouteDomain,
    pub signal_key: String,
    pub signal_value_bp: i16,
    pub confidence_bp: u16,
    pub governed_artifact_active: bool,
    pub evidence_ref: String,
}

impl PaeSignalVector {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        signal_id: String,
        source: PaeSignalSource,
        route_domain: PaeRouteDomain,
        signal_key: String,
        signal_value_bp: i16,
        confidence_bp: u16,
        governed_artifact_active: bool,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let signal = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            signal_id,
            source,
            route_domain,
            signal_key,
            signal_value_bp,
            confidence_bp,
            governed_artifact_active,
            evidence_ref,
        };
        signal.validate()?;
        Ok(signal)
    }
}

impl Validate for PaeSignalVector {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_signal_vector.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        validate_token("pae_signal_vector.signal_id", &self.signal_id, 128)?;
        validate_token("pae_signal_vector.signal_key", &self.signal_key, 96)?;
        if self.signal_value_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_signal_vector.signal_value_bp",
                reason: "must be within -20000..=20000 basis points",
            });
        }
        if self.confidence_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_signal_vector.confidence_bp",
                reason: "must be <= 10000 basis points",
            });
        }
        if self.source == PaeSignalSource::RllGoverned && !self.governed_artifact_active {
            return Err(ContractViolation::InvalidValue {
                field: "pae_signal_vector.governed_artifact_active",
                reason: "must be true for RLL governed signals",
            });
        }
        validate_token("pae_signal_vector.evidence_ref", &self.evidence_ref, 128)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaePolicyCandidate {
    pub schema_version: SchemaVersion,
    pub candidate_id: String,
    pub route_domain: PaeRouteDomain,
    pub provider_slot: PaeProviderSlot,
    pub proposed_mode: PaeMode,
    pub expected_quality_bp: i16,
    pub expected_latency_ms: u16,
    pub expected_cost_bp: i16,
    pub regression_risk_bp: u16,
    pub sample_size: u16,
    pub governed_artifact_ref: Option<String>,
    pub rollback_to: Option<String>,
}

impl PaePolicyCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        candidate_id: String,
        route_domain: PaeRouteDomain,
        provider_slot: PaeProviderSlot,
        proposed_mode: PaeMode,
        expected_quality_bp: i16,
        expected_latency_ms: u16,
        expected_cost_bp: i16,
        regression_risk_bp: u16,
        sample_size: u16,
        governed_artifact_ref: Option<String>,
        rollback_to: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let candidate = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            candidate_id,
            route_domain,
            provider_slot,
            proposed_mode,
            expected_quality_bp,
            expected_latency_ms,
            expected_cost_bp,
            regression_risk_bp,
            sample_size,
            governed_artifact_ref,
            rollback_to,
        };
        candidate.validate()?;
        Ok(candidate)
    }
}

impl Validate for PaePolicyCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        validate_token("pae_policy_candidate.candidate_id", &self.candidate_id, 128)?;
        if self.expected_quality_bp.abs() > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.expected_quality_bp",
                reason: "must be within -10000..=10000 basis points",
            });
        }
        if self.expected_latency_ms == 0 || self.expected_latency_ms > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.expected_latency_ms",
                reason: "must be within 1..=10000 ms",
            });
        }
        if self.expected_cost_bp.abs() > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.expected_cost_bp",
                reason: "must be within -10000..=10000 basis points",
            });
        }
        if self.regression_risk_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.regression_risk_bp",
                reason: "must be <= 10000 basis points",
            });
        }
        if self.sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_candidate.sample_size",
                reason: "must be > 0",
            });
        }

        match self.proposed_mode {
            PaeMode::Shadow => {}
            PaeMode::Assist => {
                let artifact =
                    self.governed_artifact_ref
                        .as_ref()
                        .ok_or(ContractViolation::InvalidValue {
                            field: "pae_policy_candidate.governed_artifact_ref",
                            reason: "must be present when proposed_mode=ASSIST",
                        })?;
                validate_token("pae_policy_candidate.governed_artifact_ref", artifact, 128)?;
            }
            PaeMode::Lead => {
                let artifact =
                    self.governed_artifact_ref
                        .as_ref()
                        .ok_or(ContractViolation::InvalidValue {
                            field: "pae_policy_candidate.governed_artifact_ref",
                            reason: "must be present when proposed_mode=LEAD",
                        })?;
                validate_token("pae_policy_candidate.governed_artifact_ref", artifact, 128)?;
                let rollback =
                    self.rollback_to
                        .as_ref()
                        .ok_or(ContractViolation::InvalidValue {
                            field: "pae_policy_candidate.rollback_to",
                            reason: "must be present when proposed_mode=LEAD",
                        })?;
                validate_token("pae_policy_candidate.rollback_to", rollback, 128)?;
            }
        }

        if let Some(rollback_to) = &self.rollback_to {
            validate_token("pae_policy_candidate.rollback_to", rollback_to, 128)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaePolicyScoreBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PaeRequestEnvelope,
    pub tenant_id: String,
    pub device_profile_ref: String,
    pub current_mode: PaeMode,
    pub signals: Vec<PaeSignalVector>,
    pub candidates: Vec<PaePolicyCandidate>,
    pub require_governed_artifacts: bool,
    pub minimum_sample_size: u16,
    pub promotion_threshold_bp: i16,
    pub demotion_failure_threshold: u8,
    pub consecutive_threshold_failures: u8,
}

impl PaePolicyScoreBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PaeRequestEnvelope,
        tenant_id: String,
        device_profile_ref: String,
        current_mode: PaeMode,
        signals: Vec<PaeSignalVector>,
        candidates: Vec<PaePolicyCandidate>,
        require_governed_artifacts: bool,
        minimum_sample_size: u16,
        promotion_threshold_bp: i16,
        demotion_failure_threshold: u8,
        consecutive_threshold_failures: u8,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            envelope,
            tenant_id,
            device_profile_ref,
            current_mode,
            signals,
            candidates,
            require_governed_artifacts,
            minimum_sample_size,
            promotion_threshold_bp,
            demotion_failure_threshold,
            consecutive_threshold_failures,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PaePolicyScoreBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "pae_policy_score_build_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "pae_policy_score_build_request.device_profile_ref",
            &self.device_profile_ref,
            96,
        )?;

        if self.signals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.signals",
                reason: "must be non-empty",
            });
        }
        if self.signals.len() > self.envelope.max_signals as usize {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.signals",
                reason: "exceeds envelope max_signals",
            });
        }
        let mut signal_ids = BTreeSet::new();
        for signal in &self.signals {
            signal.validate()?;
            if !signal_ids.insert(signal.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_policy_score_build_request.signals",
                    reason: "duplicate signal_id",
                });
            }
        }

        if self.candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.candidates",
                reason: "must be non-empty",
            });
        }
        if self.candidates.len() > self.envelope.max_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.candidates",
                reason: "exceeds envelope max_candidates",
            });
        }
        let mut candidate_ids = BTreeSet::new();
        for candidate in &self.candidates {
            candidate.validate()?;
            if !candidate_ids.insert(candidate.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_policy_score_build_request.candidates",
                    reason: "duplicate candidate_id",
                });
            }
            if self.require_governed_artifacts
                && candidate.proposed_mode != PaeMode::Shadow
                && candidate.governed_artifact_ref.is_none()
            {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_policy_score_build_request.candidates",
                    reason: "governed artifact required for non-shadow candidate",
                });
            }
        }

        if self.minimum_sample_size < 10 || self.minimum_sample_size > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.minimum_sample_size",
                reason: "must be within 10..=10000",
            });
        }
        if self.promotion_threshold_bp < -10_000 || self.promotion_threshold_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.promotion_threshold_bp",
                reason: "must be within -10000..=10000 basis points",
            });
        }
        if self.demotion_failure_threshold == 0 || self.demotion_failure_threshold > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.demotion_failure_threshold",
                reason: "must be within 1..=32",
            });
        }
        if self.consecutive_threshold_failures > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_request.consecutive_threshold_failures",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeScoreEntry {
    pub schema_version: SchemaVersion,
    pub candidate_id: String,
    pub route_domain: PaeRouteDomain,
    pub provider_slot: PaeProviderSlot,
    pub mode_applied: PaeMode,
    pub total_score_bp: i32,
    pub quality_score_bp: i16,
    pub latency_penalty_bp: i16,
    pub cost_penalty_bp: i16,
    pub regression_penalty_bp: i16,
    pub sample_size: u16,
}

impl PaeScoreEntry {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        candidate_id: String,
        route_domain: PaeRouteDomain,
        provider_slot: PaeProviderSlot,
        mode_applied: PaeMode,
        total_score_bp: i32,
        quality_score_bp: i16,
        latency_penalty_bp: i16,
        cost_penalty_bp: i16,
        regression_penalty_bp: i16,
        sample_size: u16,
    ) -> Result<Self, ContractViolation> {
        let score = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            candidate_id,
            route_domain,
            provider_slot,
            mode_applied,
            total_score_bp,
            quality_score_bp,
            latency_penalty_bp,
            cost_penalty_bp,
            regression_penalty_bp,
            sample_size,
        };
        score.validate()?;
        Ok(score)
    }
}

impl Validate for PaeScoreEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_score_entry.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        validate_token("pae_score_entry.candidate_id", &self.candidate_id, 128)?;
        if self.total_score_bp < -50_000 || self.total_score_bp > 50_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_score_entry.total_score_bp",
                reason: "must be within -50000..=50000 basis points",
            });
        }
        if self.sample_size == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_score_entry.sample_size",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaePolicyScoreBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PaeCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_candidate_id: String,
    pub ordered_scores: Vec<PaeScoreEntry>,
    pub selected_mode: PaeMode,
    pub promotion_eligible: bool,
    pub rollback_ready: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl PaePolicyScoreBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_candidate_id: String,
        ordered_scores: Vec<PaeScoreEntry>,
        selected_mode: PaeMode,
        promotion_eligible: bool,
        rollback_ready: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            capability_id: PaeCapabilityId::PaePolicyScoreBuild,
            reason_code,
            selected_candidate_id,
            ordered_scores,
            selected_mode,
            promotion_eligible,
            rollback_ready,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for PaePolicyScoreBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PaeCapabilityId::PaePolicyScoreBuild {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.capability_id",
                reason: "must be PAE_POLICY_SCORE_BUILD",
            });
        }
        validate_token(
            "pae_policy_score_build_ok.selected_candidate_id",
            &self.selected_candidate_id,
            128,
        )?;
        if self.ordered_scores.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.ordered_scores",
                reason: "must be non-empty",
            });
        }
        if self.ordered_scores.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.ordered_scores",
                reason: "must be <= 32",
            });
        }

        let mut seen = BTreeSet::new();
        for score in &self.ordered_scores {
            score.validate()?;
            if !seen.insert(score.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_policy_score_build_ok.ordered_scores",
                    reason: "duplicate candidate_id",
                });
            }
        }

        if self.ordered_scores[0].candidate_id != self.selected_candidate_id {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.selected_candidate_id",
                reason: "must match first ordered score",
            });
        }
        if self.selected_mode == PaeMode::Lead && !self.rollback_ready {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.rollback_ready",
                reason: "must be true when selected_mode=LEAD",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "pae_policy_score_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeAdaptationHint {
    pub schema_version: SchemaVersion,
    pub hint_id: String,
    pub target_engine: PaeTargetEngine,
    pub route_domain: PaeRouteDomain,
    pub hint_key: String,
    pub hint_value: String,
    pub priority_bp: u16,
    pub provenance_ref: String,
}

impl PaeAdaptationHint {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        hint_id: String,
        target_engine: PaeTargetEngine,
        route_domain: PaeRouteDomain,
        hint_key: String,
        hint_value: String,
        priority_bp: u16,
        provenance_ref: String,
    ) -> Result<Self, ContractViolation> {
        let hint = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            hint_id,
            target_engine,
            route_domain,
            hint_key,
            hint_value,
            priority_bp,
            provenance_ref,
        };
        hint.validate()?;
        Ok(hint)
    }
}

impl Validate for PaeAdaptationHint {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        validate_token("pae_adaptation_hint.hint_id", &self.hint_id, 96)?;
        validate_token("pae_adaptation_hint.hint_key", &self.hint_key, 96)?;
        validate_text("pae_adaptation_hint.hint_value", &self.hint_value, 160)?;
        if self.priority_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint.priority_bp",
                reason: "must be <= 10000 basis points",
            });
        }
        validate_token(
            "pae_adaptation_hint.provenance_ref",
            &self.provenance_ref,
            128,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeAdaptationHintEmitRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PaeRequestEnvelope,
    pub tenant_id: String,
    pub device_profile_ref: String,
    pub selected_candidate_id: String,
    pub selected_mode: PaeMode,
    pub ordered_scores: Vec<PaeScoreEntry>,
    pub allowed_targets: Vec<PaeTargetEngine>,
    pub require_no_runtime_authority_drift: bool,
}

impl PaeAdaptationHintEmitRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PaeRequestEnvelope,
        tenant_id: String,
        device_profile_ref: String,
        selected_candidate_id: String,
        selected_mode: PaeMode,
        ordered_scores: Vec<PaeScoreEntry>,
        allowed_targets: Vec<PaeTargetEngine>,
        require_no_runtime_authority_drift: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            envelope,
            tenant_id,
            device_profile_ref,
            selected_candidate_id,
            selected_mode,
            ordered_scores,
            allowed_targets,
            require_no_runtime_authority_drift,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PaeAdaptationHintEmitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "pae_adaptation_hint_emit_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "pae_adaptation_hint_emit_request.device_profile_ref",
            &self.device_profile_ref,
            96,
        )?;
        validate_token(
            "pae_adaptation_hint_emit_request.selected_candidate_id",
            &self.selected_candidate_id,
            128,
        )?;

        if self.ordered_scores.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.ordered_scores",
                reason: "must be non-empty",
            });
        }
        if self.ordered_scores.len() > self.envelope.max_scores as usize {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.ordered_scores",
                reason: "exceeds envelope max_scores",
            });
        }
        let mut score_ids = BTreeSet::new();
        let mut selected_present = false;
        for score in &self.ordered_scores {
            score.validate()?;
            if !score_ids.insert(score.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_request.ordered_scores",
                    reason: "duplicate candidate_id",
                });
            }
            if score.candidate_id == self.selected_candidate_id {
                selected_present = true;
            }
        }
        if !selected_present {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.selected_candidate_id",
                reason: "must exist in ordered_scores",
            });
        }

        if self.allowed_targets.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.allowed_targets",
                reason: "must be non-empty",
            });
        }
        if self.allowed_targets.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_request.allowed_targets",
                reason: "must be <= 4",
            });
        }
        let mut target_set = BTreeSet::new();
        for target in &self.allowed_targets {
            if !target_set.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_request.allowed_targets",
                    reason: "must be unique",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeAdaptationHintEmitOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PaeCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PaeValidationStatus,
    pub diagnostics: Vec<String>,
    pub target_engines: Vec<PaeTargetEngine>,
    pub adaptation_hints: Vec<PaeAdaptationHint>,
    pub no_runtime_authority_drift: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl PaeAdaptationHintEmitOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PaeValidationStatus,
        diagnostics: Vec<String>,
        target_engines: Vec<PaeTargetEngine>,
        adaptation_hints: Vec<PaeAdaptationHint>,
        no_runtime_authority_drift: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            capability_id: PaeCapabilityId::PaeAdaptationHintEmit,
            reason_code,
            validation_status,
            diagnostics,
            target_engines,
            adaptation_hints,
            no_runtime_authority_drift,
            advisory_only,
            no_execution_authority,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for PaeAdaptationHintEmitOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PaeCapabilityId::PaeAdaptationHintEmit {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.capability_id",
                reason: "must be PAE_ADAPTATION_HINT_EMIT",
            });
        }

        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("pae_adaptation_hint_emit_ok.diagnostics", diagnostic, 96)?;
        }

        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.target_engines",
                reason: "must be non-empty",
            });
        }
        if self.target_engines.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.target_engines",
                reason: "must be <= 4",
            });
        }
        let mut target_set = BTreeSet::new();
        for target in &self.target_engines {
            if !target_set.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_ok.target_engines",
                    reason: "must be unique",
                });
            }
        }

        if self.adaptation_hints.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.adaptation_hints",
                reason: "must be non-empty",
            });
        }
        if self.adaptation_hints.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.adaptation_hints",
                reason: "must be <= 16",
            });
        }
        let mut hint_set = BTreeSet::new();
        for hint in &self.adaptation_hints {
            hint.validate()?;
            if !hint_set.insert(hint.hint_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_ok.adaptation_hints",
                    reason: "duplicate hint_id",
                });
            }
            if !target_set.contains(hint.target_engine.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_ok.adaptation_hints",
                    reason: "hint target engine not allowed",
                });
            }
        }

        if self.validation_status == PaeValidationStatus::Ok {
            if !self.diagnostics.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_ok.diagnostics",
                    reason: "must be empty when validation_status=OK",
                });
            }
            if !self.no_runtime_authority_drift {
                return Err(ContractViolation::InvalidValue {
                    field: "pae_adaptation_hint_emit_ok.no_runtime_authority_drift",
                    reason: "must be true when validation_status=OK",
                });
            }
        }

        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "pae_adaptation_hint_emit_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaeRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PaeCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PaeRefuse {
    pub fn v1(
        capability_id: PaeCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1PAE_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for PaeRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PAE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "pae_refuse.schema_version",
                reason: "must match PH1PAE_CONTRACT_VERSION",
            });
        }
        validate_text("pae_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PaeRequest {
    PaePolicyScoreBuild(PaePolicyScoreBuildRequest),
    PaeAdaptationHintEmit(PaeAdaptationHintEmitRequest),
}

impl Validate for Ph1PaeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PaeRequest::PaePolicyScoreBuild(req) => req.validate(),
            Ph1PaeRequest::PaeAdaptationHintEmit(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PaeResponse {
    PaePolicyScoreBuildOk(PaePolicyScoreBuildOk),
    PaeAdaptationHintEmitOk(PaeAdaptationHintEmitOk),
    Refuse(PaeRefuse),
}

impl Validate for Ph1PaeResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PaeResponse::PaePolicyScoreBuildOk(ok) => ok.validate(),
            Ph1PaeResponse::PaeAdaptationHintEmitOk(ok) => ok.validate(),
            Ph1PaeResponse::Refuse(r) => r.validate(),
        }
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

fn validate_text(
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> PaeRequestEnvelope {
        PaeRequestEnvelope::v1(CorrelationId(8201), TurnId(401), 8, 4, 4, 4, 6).unwrap()
    }

    fn signal(id: &str) -> PaeSignalVector {
        PaeSignalVector::v1(
            id.to_string(),
            PaeSignalSource::Feedback,
            PaeRouteDomain::Stt,
            "correction_rate_bp".to_string(),
            180,
            8500,
            true,
            format!("feedback:evidence:{}", id),
        )
        .unwrap()
    }

    fn candidate(id: &str, mode: PaeMode) -> PaePolicyCandidate {
        PaePolicyCandidate::v1(
            id.to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            mode,
            2400,
            220,
            300,
            200,
            140,
            Some(format!("artifact:{}", id)),
            if mode == PaeMode::Lead {
                Some(format!("artifact:{}:rollback", id))
            } else {
                None
            },
        )
        .unwrap()
    }

    #[test]
    fn at_pae_contract_01_score_build_request_is_schema_valid() {
        let req = PaePolicyScoreBuildRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "desktop_profile_v1".to_string(),
            PaeMode::Assist,
            vec![signal("s1")],
            vec![candidate("c1", PaeMode::Assist)],
            true,
            100,
            500,
            3,
            0,
        )
        .unwrap();

        assert!(req.validate().is_ok());
    }

    #[test]
    fn at_pae_contract_02_lead_candidate_requires_governed_artifact_and_rollback() {
        let lead_without_rollback = PaePolicyCandidate::v1(
            "c_lead".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Lead,
            2600,
            200,
            250,
            180,
            200,
            Some("artifact:lead".to_string()),
            None,
        );

        assert!(lead_without_rollback.is_err());
    }

    #[test]
    fn at_pae_contract_03_hint_emit_ok_requires_no_runtime_authority_drift_when_ok() {
        let score = PaeScoreEntry::v1(
            "c1".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            1800,
            2400,
            120,
            180,
            90,
            120,
        )
        .unwrap();

        let hint = PaeAdaptationHint::v1(
            "h1".to_string(),
            PaeTargetEngine::Ph1C,
            PaeRouteDomain::Stt,
            "stt_route_mode".to_string(),
            "ASSIST".to_string(),
            1200,
            "artifact:c1".to_string(),
        )
        .unwrap();

        let req = PaeAdaptationHintEmitRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "desktop_profile_v1".to_string(),
            "c1".to_string(),
            PaeMode::Assist,
            vec![score],
            vec![PaeTargetEngine::Ph1C],
            true,
        )
        .unwrap();
        assert!(req.validate().is_ok());

        let out = PaeAdaptationHintEmitOk::v1(
            ReasonCodeId(1),
            PaeValidationStatus::Ok,
            vec![],
            vec![PaeTargetEngine::Ph1C],
            vec![hint],
            false,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_pae_contract_04_target_list_must_be_unique() {
        let score = PaeScoreEntry::v1(
            "c1".to_string(),
            PaeRouteDomain::Stt,
            PaeProviderSlot::Primary,
            PaeMode::Assist,
            1900,
            2500,
            110,
            160,
            80,
            130,
        )
        .unwrap();

        let req = PaeAdaptationHintEmitRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "desktop_profile_v1".to_string(),
            "c1".to_string(),
            PaeMode::Assist,
            vec![score],
            vec![PaeTargetEngine::Ph1C, PaeTargetEngine::Ph1C],
            true,
        );

        assert!(req.is_err());
    }
}
