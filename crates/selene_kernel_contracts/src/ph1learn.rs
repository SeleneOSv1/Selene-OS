#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1LEARN_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnCapabilityId {
    LearnSignalAggregate,
    LearnArtifactPackageBuild,
}

impl LearnCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            LearnCapabilityId::LearnSignalAggregate => "LEARN_SIGNAL_AGGREGATE",
            LearnCapabilityId::LearnArtifactPackageBuild => "LEARN_ARTIFACT_PACKAGE_BUILD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnSignalType {
    SttReject,
    UserCorrection,
    ClarifyLoop,
    ToolFail,
    VocabularyRepeat,
    BargeIn,
    DeliverySwitch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnArtifactTarget {
    KnowTenantGlossaryPack,
    PronLexiconPack,
    CacheDecisionSkeleton,
    PruneClarificationOrdering,
    PaeRoutingWeights,
    SearchWebExtractionHints,
    ListenEnvironmentProfile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnScope {
    User,
    Tenant,
    GlobalDerived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnTargetEngine {
    Know,
    Pron,
    Cache,
    Prune,
    Pae,
    Search,
    Listen,
}

impl LearnTargetEngine {
    pub fn as_str(self) -> &'static str {
        match self {
            LearnTargetEngine::Know => "PH1.KNOW",
            LearnTargetEngine::Pron => "PH1.PRON",
            LearnTargetEngine::Cache => "PH1.CACHE",
            LearnTargetEngine::Prune => "PH1.PRUNE",
            LearnTargetEngine::Pae => "PH1.PAE",
            LearnTargetEngine::Search => "PH1.SEARCH",
            LearnTargetEngine::Listen => "PH1.LISTEN",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LearnValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_signals: u8,
    pub max_artifacts: u8,
    pub max_diagnostics: u8,
}

impl LearnRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_signals: u8,
        max_artifacts: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_signals,
            max_artifacts,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for LearnRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_request_envelope.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_signals == 0 || self.max_signals > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_request_envelope.max_signals",
                reason: "must be within 1..=128",
            });
        }
        if self.max_artifacts == 0 || self.max_artifacts > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_request_envelope.max_artifacts",
                reason: "must be within 1..=64",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnSignal {
    pub schema_version: SchemaVersion,
    pub signal_id: String,
    pub tenant_id: String,
    pub signal_type: LearnSignalType,
    pub scope_hint: LearnScope,
    pub scope_ref: Option<String>,
    pub metric_key: String,
    pub metric_value_bp: i16,
    pub occurrence_count: u16,
    pub contains_sensitive_data: bool,
    pub consent_required: bool,
    pub consent_asserted: bool,
    pub evidence_ref: String,
}

impl LearnSignal {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        signal_id: String,
        tenant_id: String,
        signal_type: LearnSignalType,
        scope_hint: LearnScope,
        scope_ref: Option<String>,
        metric_key: String,
        metric_value_bp: i16,
        occurrence_count: u16,
        contains_sensitive_data: bool,
        consent_required: bool,
        consent_asserted: bool,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let signal = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            signal_id,
            tenant_id,
            signal_type,
            scope_hint,
            scope_ref,
            metric_key,
            metric_value_bp,
            occurrence_count,
            contains_sensitive_data,
            consent_required,
            consent_asserted,
            evidence_ref,
        };
        signal.validate()?;
        Ok(signal)
    }
}

impl Validate for LearnSignal {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        validate_token("learn_signal.signal_id", &self.signal_id, 96)?;
        validate_token("learn_signal.tenant_id", &self.tenant_id, 64)?;
        validate_text("learn_signal.metric_key", &self.metric_key, 64)?;
        if self.metric_value_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal.metric_value_bp",
                reason: "must be within -20000..=20000 basis points",
            });
        }
        if self.occurrence_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal.occurrence_count",
                reason: "must be > 0",
            });
        }
        validate_token("learn_signal.evidence_ref", &self.evidence_ref, 128)?;

        match self.scope_hint {
            LearnScope::User => {
                let scope_ref = self
                    .scope_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "learn_signal.scope_ref",
                        reason: "must be present when scope_hint=USER",
                    })?;
                validate_token("learn_signal.scope_ref", scope_ref, 64)?;
            }
            LearnScope::Tenant => {
                let scope_ref = self
                    .scope_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "learn_signal.scope_ref",
                        reason: "must be present when scope_hint=TENANT",
                    })?;
                validate_token("learn_signal.scope_ref", scope_ref, 64)?;
            }
            LearnScope::GlobalDerived => {
                if self.scope_ref.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "learn_signal.scope_ref",
                        reason: "must be absent when scope_hint=GLOBAL_DERIVED",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnArtifactCandidate {
    pub schema_version: SchemaVersion,
    pub artifact_id: String,
    pub target: LearnArtifactTarget,
    pub scope: LearnScope,
    pub scope_ref: Option<String>,
    pub artifact_version: u32,
    pub expected_effect_bp: i16,
    pub provenance_ref: String,
    pub rollback_to: Option<String>,
    pub consent_safe: bool,
}

impl LearnArtifactCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        artifact_id: String,
        target: LearnArtifactTarget,
        scope: LearnScope,
        scope_ref: Option<String>,
        artifact_version: u32,
        expected_effect_bp: i16,
        provenance_ref: String,
        rollback_to: Option<String>,
        consent_safe: bool,
    ) -> Result<Self, ContractViolation> {
        let artifact = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            artifact_id,
            target,
            scope,
            scope_ref,
            artifact_version,
            expected_effect_bp,
            provenance_ref,
            rollback_to,
            consent_safe,
        };
        artifact.validate()?;
        Ok(artifact)
    }
}

impl Validate for LearnArtifactCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_candidate.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        validate_token(
            "learn_artifact_candidate.artifact_id",
            &self.artifact_id,
            128,
        )?;
        if self.artifact_version == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_candidate.artifact_version",
                reason: "must be > 0",
            });
        }
        if self.expected_effect_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_candidate.expected_effect_bp",
                reason: "must be within -20000..=20000 basis points",
            });
        }
        validate_token(
            "learn_artifact_candidate.provenance_ref",
            &self.provenance_ref,
            128,
        )?;

        match self.scope {
            LearnScope::User => {
                let scope_ref = self
                    .scope_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "learn_artifact_candidate.scope_ref",
                        reason: "must be present when scope=USER",
                    })?;
                validate_token("learn_artifact_candidate.scope_ref", scope_ref, 64)?;
            }
            LearnScope::Tenant => {
                let scope_ref = self
                    .scope_ref
                    .as_ref()
                    .ok_or(ContractViolation::InvalidValue {
                        field: "learn_artifact_candidate.scope_ref",
                        reason: "must be present when scope=TENANT",
                    })?;
                validate_token("learn_artifact_candidate.scope_ref", scope_ref, 64)?;
            }
            LearnScope::GlobalDerived => {
                if self.scope_ref.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "learn_artifact_candidate.scope_ref",
                        reason: "must be absent when scope=GLOBAL_DERIVED",
                    });
                }
                if !self.consent_safe {
                    return Err(ContractViolation::InvalidValue {
                        field: "learn_artifact_candidate.consent_safe",
                        reason: "must be true when scope=GLOBAL_DERIVED",
                    });
                }
            }
        }

        if let Some(rollback_to) = &self.rollback_to {
            validate_token("learn_artifact_candidate.rollback_to", rollback_to, 128)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnSignalAggregateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LearnRequestEnvelope,
    pub tenant_id: String,
    pub signals: Vec<LearnSignal>,
    pub require_derived_only_global: bool,
    pub no_runtime_drift_required: bool,
}

impl LearnSignalAggregateRequest {
    pub fn v1(
        envelope: LearnRequestEnvelope,
        tenant_id: String,
        signals: Vec<LearnSignal>,
        require_derived_only_global: bool,
        no_runtime_drift_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            envelope,
            tenant_id,
            signals,
            require_derived_only_global,
            no_runtime_drift_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for LearnSignalAggregateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_request.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "learn_signal_aggregate_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        if self.signals.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_request.signals",
                reason: "must be non-empty",
            });
        }
        if self.signals.len() > self.envelope.max_signals as usize {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_request.signals",
                reason: "must be <= envelope.max_signals",
            });
        }
        if !self.require_derived_only_global {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_request.require_derived_only_global",
                reason: "must be true",
            });
        }
        if !self.no_runtime_drift_required {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_request.no_runtime_drift_required",
                reason: "must be true",
            });
        }

        let mut signal_ids = BTreeSet::new();
        for signal in &self.signals {
            signal.validate()?;
            if signal.tenant_id != self.tenant_id {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_signal_aggregate_request.signals",
                    reason: "signal tenant_id must match request tenant_id",
                });
            }
            if !signal_ids.insert(signal.signal_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_signal_aggregate_request.signals",
                    reason: "signal_id must be unique",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnArtifactPackageBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: LearnRequestEnvelope,
    pub tenant_id: String,
    pub selected_artifact_id: String,
    pub ordered_artifacts: Vec<LearnArtifactCandidate>,
    pub target_engines: Vec<LearnTargetEngine>,
    pub require_versioning: bool,
    pub require_rollback_ptr: bool,
    pub no_runtime_drift_required: bool,
}

impl LearnArtifactPackageBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: LearnRequestEnvelope,
        tenant_id: String,
        selected_artifact_id: String,
        ordered_artifacts: Vec<LearnArtifactCandidate>,
        target_engines: Vec<LearnTargetEngine>,
        require_versioning: bool,
        require_rollback_ptr: bool,
        no_runtime_drift_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            envelope,
            tenant_id,
            selected_artifact_id,
            ordered_artifacts,
            target_engines,
            require_versioning,
            require_rollback_ptr,
            no_runtime_drift_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for LearnArtifactPackageBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "learn_artifact_package_build_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "learn_artifact_package_build_request.selected_artifact_id",
            &self.selected_artifact_id,
            128,
        )?;
        if self.ordered_artifacts.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.ordered_artifacts",
                reason: "must be non-empty",
            });
        }
        if self.ordered_artifacts.len() > self.envelope.max_artifacts as usize {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.ordered_artifacts",
                reason: "must be <= envelope.max_artifacts",
            });
        }
        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.target_engines",
                reason: "must be non-empty",
            });
        }
        if self.target_engines.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.target_engines",
                reason: "must be <= 8",
            });
        }
        if !self.require_versioning {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.require_versioning",
                reason: "must be true",
            });
        }
        if !self.require_rollback_ptr {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.require_rollback_ptr",
                reason: "must be true",
            });
        }
        if !self.no_runtime_drift_required {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.no_runtime_drift_required",
                reason: "must be true",
            });
        }

        let mut artifact_ids = BTreeSet::new();
        let mut selected_present = false;
        for artifact in &self.ordered_artifacts {
            artifact.validate()?;
            if !artifact_ids.insert(artifact.artifact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_artifact_package_build_request.ordered_artifacts",
                    reason: "artifact_id must be unique",
                });
            }
            if artifact.artifact_id == self.selected_artifact_id {
                selected_present = true;
            }

            if artifact.scope == LearnScope::Tenant
                && artifact.scope_ref.as_deref() != Some(self.tenant_id.as_str())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_artifact_package_build_request.ordered_artifacts",
                    reason: "tenant-scoped artifact scope_ref must equal tenant_id",
                });
            }
        }
        if !selected_present {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_request.selected_artifact_id",
                reason: "must exist in ordered_artifacts",
            });
        }

        let mut target_names = BTreeSet::new();
        for target in &self.target_engines {
            if !target_names.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_artifact_package_build_request.target_engines",
                    reason: "must be unique",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LearnRequest {
    LearnSignalAggregate(LearnSignalAggregateRequest),
    LearnArtifactPackageBuild(LearnArtifactPackageBuildRequest),
}

impl Validate for Ph1LearnRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LearnRequest::LearnSignalAggregate(req) => req.validate(),
            Ph1LearnRequest::LearnArtifactPackageBuild(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnSignalAggregateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LearnCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_artifact_id: String,
    pub ordered_artifacts: Vec<LearnArtifactCandidate>,
    pub consent_safe: bool,
    pub derived_only_global_preserved: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl LearnSignalAggregateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_artifact_id: String,
        ordered_artifacts: Vec<LearnArtifactCandidate>,
        consent_safe: bool,
        derived_only_global_preserved: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            capability_id: LearnCapabilityId::LearnSignalAggregate,
            reason_code,
            selected_artifact_id,
            ordered_artifacts,
            consent_safe,
            derived_only_global_preserved,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LearnSignalAggregateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LearnCapabilityId::LearnSignalAggregate {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.capability_id",
                reason: "must be LEARN_SIGNAL_AGGREGATE",
            });
        }
        validate_token(
            "learn_signal_aggregate_ok.selected_artifact_id",
            &self.selected_artifact_id,
            128,
        )?;
        if self.ordered_artifacts.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.ordered_artifacts",
                reason: "must be non-empty",
            });
        }
        if self.ordered_artifacts.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.ordered_artifacts",
                reason: "must be <= 64",
            });
        }
        let mut artifact_ids = BTreeSet::new();
        let mut selected_present = false;
        for artifact in &self.ordered_artifacts {
            artifact.validate()?;
            if !artifact_ids.insert(artifact.artifact_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_signal_aggregate_ok.ordered_artifacts",
                    reason: "artifact_id must be unique",
                });
            }
            if artifact.artifact_id == self.selected_artifact_id {
                selected_present = true;
            }
        }
        if !selected_present {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.selected_artifact_id",
                reason: "must exist in ordered_artifacts",
            });
        }
        if !self.consent_safe {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.consent_safe",
                reason: "must be true",
            });
        }
        if !self.derived_only_global_preserved {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.derived_only_global_preserved",
                reason: "must be true",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "learn_signal_aggregate_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnArtifactPackageBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: LearnCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: LearnValidationStatus,
    pub diagnostics: Vec<String>,
    pub target_engines: Vec<LearnTargetEngine>,
    pub artifacts_versioned: bool,
    pub rollbackable: bool,
    pub no_runtime_drift: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl LearnArtifactPackageBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: LearnValidationStatus,
        diagnostics: Vec<String>,
        target_engines: Vec<LearnTargetEngine>,
        artifacts_versioned: bool,
        rollbackable: bool,
        no_runtime_drift: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            capability_id: LearnCapabilityId::LearnArtifactPackageBuild,
            reason_code,
            validation_status,
            diagnostics,
            target_engines,
            artifacts_versioned,
            rollbackable,
            no_runtime_drift,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LearnArtifactPackageBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        if self.capability_id != LearnCapabilityId::LearnArtifactPackageBuild {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.capability_id",
                reason: "must be LEARN_ARTIFACT_PACKAGE_BUILD",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token(
                "learn_artifact_package_build_ok.diagnostics",
                diagnostic,
                96,
            )?;
        }
        if self.validation_status == LearnValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.diagnostics",
                reason: "must be non-empty when validation_status=FAIL",
            });
        }

        if self.target_engines.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.target_engines",
                reason: "must be non-empty",
            });
        }
        if self.target_engines.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.target_engines",
                reason: "must be <= 8",
            });
        }
        let mut target_names = BTreeSet::new();
        for target in &self.target_engines {
            if !target_names.insert(target.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "learn_artifact_package_build_ok.target_engines",
                    reason: "must be unique",
                });
            }
        }

        if self.validation_status == LearnValidationStatus::Ok
            && (!self.artifacts_versioned || !self.rollbackable || !self.no_runtime_drift)
        {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok",
                reason: "OK status requires versioning + rollback + no_runtime_drift",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "learn_artifact_package_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: LearnCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl LearnRefuse {
    pub fn v1(
        capability_id: LearnCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1LEARN_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for LearnRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1LEARN_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "learn_refuse.schema_version",
                reason: "must match PH1LEARN_CONTRACT_VERSION",
            });
        }
        validate_text("learn_refuse.message", &self.message, 192)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1LearnResponse {
    LearnSignalAggregateOk(LearnSignalAggregateOk),
    LearnArtifactPackageBuildOk(LearnArtifactPackageBuildOk),
    Refuse(LearnRefuse),
}

impl Validate for Ph1LearnResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1LearnResponse::LearnSignalAggregateOk(out) => out.validate(),
            Ph1LearnResponse::LearnArtifactPackageBuildOk(out) => out.validate(),
            Ph1LearnResponse::Refuse(out) => out.validate(),
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
            reason: "must contain only ASCII token characters",
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
    if value.chars().any(char::is_control) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control chars",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> LearnRequestEnvelope {
        LearnRequestEnvelope::v1(CorrelationId(4201), TurnId(401), 8, 6, 8).unwrap()
    }

    fn signal(
        signal_id: &str,
        signal_type: LearnSignalType,
        scope_hint: LearnScope,
        scope_ref: Option<&str>,
        metric_key: &str,
    ) -> LearnSignal {
        LearnSignal::v1(
            signal_id.to_string(),
            "tenant_1".to_string(),
            signal_type,
            scope_hint,
            scope_ref.map(|s| s.to_string()),
            metric_key.to_string(),
            180,
            7,
            false,
            false,
            false,
            format!("learn:evidence:{}", signal_id),
        )
        .unwrap()
    }

    #[test]
    fn at_learn_01_signal_aggregate_contract_is_schema_valid() {
        let req = LearnSignalAggregateRequest::v1(
            envelope(),
            "tenant_1".to_string(),
            vec![
                signal(
                    "sig_1",
                    LearnSignalType::SttReject,
                    LearnScope::Tenant,
                    Some("tenant_1"),
                    "stt_reject_rate",
                ),
                signal(
                    "sig_2",
                    LearnSignalType::UserCorrection,
                    LearnScope::User,
                    Some("user_1"),
                    "correction_rate",
                ),
            ],
            true,
            true,
        )
        .unwrap();
        assert!(req.validate().is_ok());

        let artifact = LearnArtifactCandidate::v1(
            "artifact_1".to_string(),
            LearnArtifactTarget::KnowTenantGlossaryPack,
            LearnScope::Tenant,
            Some("tenant_1".to_string()),
            2,
            210,
            "learn:evidence:artifact_1".to_string(),
            Some("artifact_1.prev".to_string()),
            true,
        )
        .unwrap();

        let out = LearnSignalAggregateOk::v1(
            ReasonCodeId(601),
            "artifact_1".to_string(),
            vec![artifact],
            true,
            true,
            true,
            true,
        )
        .unwrap();
        assert!(out.validate().is_ok());
    }

    #[test]
    fn at_learn_02_user_scope_requires_scope_ref() {
        let req = LearnSignal::v1(
            "sig_user".to_string(),
            "tenant_1".to_string(),
            LearnSignalType::UserCorrection,
            LearnScope::User,
            None,
            "correction_rate".to_string(),
            100,
            3,
            false,
            false,
            false,
            "learn:evidence:sig_user".to_string(),
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_learn_03_global_derived_requires_scope_ref_absent() {
        let req = LearnSignal::v1(
            "sig_global".to_string(),
            "tenant_1".to_string(),
            LearnSignalType::ToolFail,
            LearnScope::GlobalDerived,
            Some("tenant_1".to_string()),
            "tool_fail_rate".to_string(),
            90,
            5,
            false,
            false,
            false,
            "learn:evidence:sig_global".to_string(),
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_learn_04_package_build_fail_status_requires_diagnostics() {
        let out = LearnArtifactPackageBuildOk::v1(
            ReasonCodeId(602),
            LearnValidationStatus::Fail,
            vec![],
            vec![LearnTargetEngine::Know],
            false,
            false,
            false,
            true,
            true,
        );

        assert!(out.is_err());
    }
}
