#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1FEEDBACK_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackCapabilityId {
    FeedbackEventCollect,
    FeedbackSignalEmit,
}

impl FeedbackCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            FeedbackCapabilityId::FeedbackEventCollect => "FEEDBACK_EVENT_COLLECT",
            FeedbackCapabilityId::FeedbackSignalEmit => "FEEDBACK_SIGNAL_EMIT",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackEventType {
    SttReject,
    SttRetry,
    LanguageMismatch,
    UserCorrection,
    ClarifyLoop,
    ConfirmAbort,
    ToolFail,
    MemoryOverride,
    DeliverySwitch,
    BargeIn,
    // Canonical Voice-ID feedback taxonomy (FDBK-01).
    VoiceIdFalseReject,
    VoiceIdFalseAccept,
    VoiceIdSpoofRisk,
    VoiceIdMultiSpeaker,
    VoiceIdDriftAlert,
    VoiceIdReauthFriction,
    // Backward-compatible Voice-ID event classes kept for migration/cutover.
    VoiceIdConfusionPair,
    VoiceIdDrift,
    VoiceIdLowQuality,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackPathType {
    Defect,
    Improvement,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackGoldProvenanceMethod {
    VerifiedHumanCorrection,
    TrustedGroundTruth,
    HighConfidenceConsensus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackGoldStatus {
    NotRequired,
    Pending,
    Verified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackSignalTarget {
    LearnPackage,
    PaeScorecard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackConfidenceBucket {
    High,
    Med,
    Low,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackToolStatus {
    Ok,
    Fail,
    Conflict,
    Timeout,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedbackValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_events: u8,
    pub max_signals: u8,
}

impl FeedbackRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_events: u8,
        max_signals: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_events,
            max_signals,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for FeedbackRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_request_envelope.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_events == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_request_envelope.max_events",
                reason: "must be > 0",
            });
        }
        if self.max_events > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_request_envelope.max_events",
                reason: "must be <= 64",
            });
        }
        if self.max_signals == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_request_envelope.max_signals",
                reason: "must be > 0",
            });
        }
        if self.max_signals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_request_envelope.max_signals",
                reason: "must be <= 32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackMetrics {
    pub schema_version: SchemaVersion,
    pub latency_ms: u32,
    pub retries: u8,
    pub confidence_bucket: FeedbackConfidenceBucket,
    pub missing_fields: Vec<String>,
    pub tool_status: FeedbackToolStatus,
}

impl FeedbackMetrics {
    pub fn v1(
        latency_ms: u32,
        retries: u8,
        confidence_bucket: FeedbackConfidenceBucket,
        missing_fields: Vec<String>,
        tool_status: FeedbackToolStatus,
    ) -> Result<Self, ContractViolation> {
        let metrics = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            latency_ms,
            retries,
            confidence_bucket,
            missing_fields,
            tool_status,
        };
        metrics.validate()?;
        Ok(metrics)
    }
}

impl Validate for FeedbackMetrics {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_metrics.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        if self.latency_ms > 600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_metrics.latency_ms",
                reason: "must be <= 600000",
            });
        }
        if self.retries > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_metrics.retries",
                reason: "must be <= 32",
            });
        }
        if self.missing_fields.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_metrics.missing_fields",
                reason: "must be <= 16 entries",
            });
        }
        let mut seen = BTreeSet::new();
        for field in &self.missing_fields {
            validate_field_key("feedback_metrics.missing_fields", field)?;
            if !seen.insert(field.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "feedback_metrics.missing_fields",
                    reason: "must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackEventRecord {
    pub schema_version: SchemaVersion,
    pub event_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub speaker_id: String,
    pub session_id: String,
    pub device_id: String,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub event_type: FeedbackEventType,
    pub path_type: FeedbackPathType,
    pub gold_case_id: Option<String>,
    pub gold_provenance_method: Option<FeedbackGoldProvenanceMethod>,
    pub gold_status: FeedbackGoldStatus,
    pub reason_code: ReasonCodeId,
    pub evidence_ref: String,
    pub idempotency_key: String,
    pub metrics: FeedbackMetrics,
}

impl FeedbackEventRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        event_id: String,
        tenant_id: String,
        user_id: String,
        speaker_id: String,
        session_id: String,
        device_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        event_type: FeedbackEventType,
        reason_code: ReasonCodeId,
        evidence_ref: String,
        idempotency_key: String,
        metrics: FeedbackMetrics,
    ) -> Result<Self, ContractViolation> {
        let path_type = classify_feedback_path(event_type);
        let gold_status = default_feedback_gold_status(path_type);
        Self::v2(
            event_id,
            tenant_id,
            user_id,
            speaker_id,
            session_id,
            device_id,
            correlation_id,
            turn_id,
            event_type,
            path_type,
            None,
            None,
            gold_status,
            reason_code,
            evidence_ref,
            idempotency_key,
            metrics,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v2(
        event_id: String,
        tenant_id: String,
        user_id: String,
        speaker_id: String,
        session_id: String,
        device_id: String,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        event_type: FeedbackEventType,
        path_type: FeedbackPathType,
        gold_case_id: Option<String>,
        gold_provenance_method: Option<FeedbackGoldProvenanceMethod>,
        gold_status: FeedbackGoldStatus,
        reason_code: ReasonCodeId,
        evidence_ref: String,
        idempotency_key: String,
        metrics: FeedbackMetrics,
    ) -> Result<Self, ContractViolation> {
        let event = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            event_id,
            tenant_id,
            user_id,
            speaker_id,
            session_id,
            device_id,
            correlation_id,
            turn_id,
            event_type,
            path_type,
            gold_case_id,
            gold_provenance_method,
            gold_status,
            reason_code,
            evidence_ref,
            idempotency_key,
            metrics,
        };
        event.validate()?;
        Ok(event)
    }
}

impl Validate for FeedbackEventRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_record.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        validate_token("feedback_event_record.event_id", &self.event_id, 64)?;
        validate_token("feedback_event_record.tenant_id", &self.tenant_id, 64)?;
        validate_token("feedback_event_record.user_id", &self.user_id, 96)?;
        validate_token("feedback_event_record.speaker_id", &self.speaker_id, 96)?;
        validate_token("feedback_event_record.session_id", &self.session_id, 96)?;
        validate_token("feedback_event_record.device_id", &self.device_id, 96)?;
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if let Some(gold_case_id) = &self.gold_case_id {
            validate_token("feedback_event_record.gold_case_id", gold_case_id, 96)?;
        }
        validate_token(
            "feedback_event_record.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        validate_token(
            "feedback_event_record.idempotency_key",
            &self.idempotency_key,
            128,
        )?;
        self.metrics.validate()?;

        match self.path_type {
            FeedbackPathType::Defect => {
                if self.gold_case_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_event_record.gold_case_id",
                        reason: "must be absent when path_type=DEFECT",
                    });
                }
                if self.gold_provenance_method.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_event_record.gold_provenance_method",
                        reason: "must be absent when path_type=DEFECT",
                    });
                }
                if self.gold_status != FeedbackGoldStatus::NotRequired {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_event_record.gold_status",
                        reason: "must be NOT_REQUIRED when path_type=DEFECT",
                    });
                }
            }
            FeedbackPathType::Improvement => {
                if self.gold_status == FeedbackGoldStatus::NotRequired {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_event_record.gold_status",
                        reason: "must be PENDING or VERIFIED when path_type=IMPROVEMENT",
                    });
                }
                if self.gold_status == FeedbackGoldStatus::Verified {
                    if self.gold_case_id.is_none() {
                        return Err(ContractViolation::InvalidValue {
                            field: "feedback_event_record.gold_case_id",
                            reason: "must be present when gold_status=VERIFIED",
                        });
                    }
                    if self.gold_provenance_method.is_none() {
                        return Err(ContractViolation::InvalidValue {
                            field: "feedback_event_record.gold_provenance_method",
                            reason: "must be present when gold_status=VERIFIED",
                        });
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackSignalCandidate {
    pub schema_version: SchemaVersion,
    pub candidate_id: String,
    pub event_type: FeedbackEventType,
    pub signal_key: String,
    pub target: FeedbackSignalTarget,
    pub path_type: FeedbackPathType,
    pub gold_case_id: Option<String>,
    pub gold_status: FeedbackGoldStatus,
    pub signal_value_bp: i16,
    pub sample_count: u32,
    pub evidence_ref: String,
}

impl FeedbackSignalCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        candidate_id: String,
        event_type: FeedbackEventType,
        signal_key: String,
        target: FeedbackSignalTarget,
        signal_value_bp: i16,
        sample_count: u32,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        Self::v2(
            candidate_id,
            event_type,
            signal_key,
            target,
            FeedbackPathType::Defect,
            None,
            FeedbackGoldStatus::NotRequired,
            signal_value_bp,
            sample_count,
            evidence_ref,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v2(
        candidate_id: String,
        event_type: FeedbackEventType,
        signal_key: String,
        target: FeedbackSignalTarget,
        path_type: FeedbackPathType,
        gold_case_id: Option<String>,
        gold_status: FeedbackGoldStatus,
        signal_value_bp: i16,
        sample_count: u32,
        evidence_ref: String,
    ) -> Result<Self, ContractViolation> {
        let candidate = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            candidate_id,
            event_type,
            signal_key,
            target,
            path_type,
            gold_case_id,
            gold_status,
            signal_value_bp,
            sample_count,
            evidence_ref,
        };
        candidate.validate()?;
        Ok(candidate)
    }
}

impl Validate for FeedbackSignalCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_candidate.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        validate_token(
            "feedback_signal_candidate.candidate_id",
            &self.candidate_id,
            64,
        )?;
        validate_field_key("feedback_signal_candidate.signal_key", &self.signal_key)?;
        if let Some(gold_case_id) = &self.gold_case_id {
            validate_token("feedback_signal_candidate.gold_case_id", gold_case_id, 96)?;
        }
        if self.signal_value_bp.abs() > 20_000 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_candidate.signal_value_bp",
                reason: "must be within -20000..=20000",
            });
        }
        if self.sample_count == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_candidate.sample_count",
                reason: "must be > 0",
            });
        }
        validate_token(
            "feedback_signal_candidate.evidence_ref",
            &self.evidence_ref,
            128,
        )?;
        match self.path_type {
            FeedbackPathType::Defect => {
                if self.gold_case_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_signal_candidate.gold_case_id",
                        reason: "must be absent when path_type=DEFECT",
                    });
                }
                if self.gold_status != FeedbackGoldStatus::NotRequired {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_signal_candidate.gold_status",
                        reason: "must be NOT_REQUIRED when path_type=DEFECT",
                    });
                }
            }
            FeedbackPathType::Improvement => {
                if self.gold_status == FeedbackGoldStatus::NotRequired {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_signal_candidate.gold_status",
                        reason: "must be PENDING or VERIFIED when path_type=IMPROVEMENT",
                    });
                }
                if self.gold_status == FeedbackGoldStatus::Verified && self.gold_case_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "feedback_signal_candidate.gold_case_id",
                        reason: "must be present when gold_status=VERIFIED",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackEventCollectRequest {
    pub schema_version: SchemaVersion,
    pub envelope: FeedbackRequestEnvelope,
    pub events: Vec<FeedbackEventRecord>,
}

impl FeedbackEventCollectRequest {
    pub fn v1(
        envelope: FeedbackRequestEnvelope,
        events: Vec<FeedbackEventRecord>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            envelope,
            events,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for FeedbackEventCollectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_request.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.events.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_request.events",
                reason: "must not be empty",
            });
        }
        if self.events.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_request.events",
                reason: "must be <= 64",
            });
        }
        let mut event_ids = BTreeSet::new();
        for event in &self.events {
            event.validate()?;
            if !event_ids.insert(event.event_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "feedback_event_collect_request.events",
                    reason: "event_id must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackSignalEmitRequest {
    pub schema_version: SchemaVersion,
    pub envelope: FeedbackRequestEnvelope,
    pub selected_candidate_id: String,
    pub ordered_signal_candidates: Vec<FeedbackSignalCandidate>,
}

impl FeedbackSignalEmitRequest {
    pub fn v1(
        envelope: FeedbackRequestEnvelope,
        selected_candidate_id: String,
        ordered_signal_candidates: Vec<FeedbackSignalCandidate>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            envelope,
            selected_candidate_id,
            ordered_signal_candidates,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for FeedbackSignalEmitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_request.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "feedback_signal_emit_request.selected_candidate_id",
            &self.selected_candidate_id,
            64,
        )?;
        if self.ordered_signal_candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_request.ordered_signal_candidates",
                reason: "must not be empty",
            });
        }
        if self.ordered_signal_candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_request.ordered_signal_candidates",
                reason: "must be <= 32",
            });
        }
        let mut candidate_ids = BTreeSet::new();
        for candidate in &self.ordered_signal_candidates {
            candidate.validate()?;
            if !candidate_ids.insert(candidate.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "feedback_signal_emit_request.ordered_signal_candidates",
                    reason: "candidate_id must be unique",
                });
            }
        }
        if !self
            .ordered_signal_candidates
            .iter()
            .any(|candidate| candidate.candidate_id == self.selected_candidate_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_request.selected_candidate_id",
                reason: "must exist in ordered_signal_candidates",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1FeedbackRequest {
    FeedbackEventCollect(FeedbackEventCollectRequest),
    FeedbackSignalEmit(FeedbackSignalEmitRequest),
}

impl Validate for Ph1FeedbackRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1FeedbackRequest::FeedbackEventCollect(req) => req.validate(),
            Ph1FeedbackRequest::FeedbackSignalEmit(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackEventCollectOk {
    pub schema_version: SchemaVersion,
    pub capability_id: FeedbackCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_candidate_id: String,
    pub ordered_signal_candidates: Vec<FeedbackSignalCandidate>,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl FeedbackEventCollectOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_candidate_id: String,
        ordered_signal_candidates: Vec<FeedbackSignalCandidate>,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            capability_id: FeedbackCapabilityId::FeedbackEventCollect,
            reason_code,
            selected_candidate_id,
            ordered_signal_candidates,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for FeedbackEventCollectOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        if self.capability_id != FeedbackCapabilityId::FeedbackEventCollect {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.capability_id",
                reason: "must be FEEDBACK_EVENT_COLLECT",
            });
        }
        validate_token(
            "feedback_event_collect_ok.selected_candidate_id",
            &self.selected_candidate_id,
            64,
        )?;
        if self.ordered_signal_candidates.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.ordered_signal_candidates",
                reason: "must not be empty",
            });
        }
        if self.ordered_signal_candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.ordered_signal_candidates",
                reason: "must be <= 32",
            });
        }
        let mut candidate_ids = BTreeSet::new();
        for candidate in &self.ordered_signal_candidates {
            candidate.validate()?;
            if !candidate_ids.insert(candidate.candidate_id.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "feedback_event_collect_ok.ordered_signal_candidates",
                    reason: "candidate_id must be unique",
                });
            }
        }
        if !self
            .ordered_signal_candidates
            .iter()
            .any(|candidate| candidate.candidate_id == self.selected_candidate_id)
        {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.selected_candidate_id",
                reason: "must exist in ordered_signal_candidates",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_event_collect_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackSignalEmitOk {
    pub schema_version: SchemaVersion,
    pub capability_id: FeedbackCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: FeedbackValidationStatus,
    pub diagnostics: Vec<String>,
    pub emits_learn: bool,
    pub emits_pae: bool,
    pub advisory_only: bool,
    pub no_execution_authority: bool,
}

impl FeedbackSignalEmitOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: FeedbackValidationStatus,
        diagnostics: Vec<String>,
        emits_learn: bool,
        emits_pae: bool,
        advisory_only: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            capability_id: FeedbackCapabilityId::FeedbackSignalEmit,
            reason_code,
            validation_status,
            diagnostics,
            emits_learn,
            emits_pae,
            advisory_only,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for FeedbackSignalEmitOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        if self.capability_id != FeedbackCapabilityId::FeedbackSignalEmit {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.capability_id",
                reason: "must be FEEDBACK_SIGNAL_EMIT",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("feedback_signal_emit_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == FeedbackValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.emits_learn && !self.emits_pae {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok",
                reason: "must emit to LEARN and/or PAE",
            });
        }
        if !self.advisory_only {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.advisory_only",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_signal_emit_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FeedbackRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: FeedbackCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl FeedbackRefuse {
    pub fn v1(
        capability_id: FeedbackCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1FEEDBACK_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for FeedbackRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1FEEDBACK_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "feedback_refuse.schema_version",
                reason: "must match PH1FEEDBACK_CONTRACT_VERSION",
            });
        }
        validate_token("feedback_refuse.message", &self.message, 256)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1FeedbackResponse {
    FeedbackEventCollectOk(FeedbackEventCollectOk),
    FeedbackSignalEmitOk(FeedbackSignalEmitOk),
    Refuse(FeedbackRefuse),
}

impl Validate for Ph1FeedbackResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1FeedbackResponse::FeedbackEventCollectOk(out) => out.validate(),
            Ph1FeedbackResponse::FeedbackSignalEmitOk(out) => out.validate(),
            Ph1FeedbackResponse::Refuse(out) => out.validate(),
        }
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
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

fn validate_field_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII snake_case",
        });
    }
    Ok(())
}

pub fn classify_feedback_path(event_type: FeedbackEventType) -> FeedbackPathType {
    match event_type {
        FeedbackEventType::UserCorrection
        | FeedbackEventType::ClarifyLoop
        | FeedbackEventType::MemoryOverride
        | FeedbackEventType::DeliverySwitch
        | FeedbackEventType::BargeIn
        | FeedbackEventType::VoiceIdDriftAlert
        | FeedbackEventType::VoiceIdReauthFriction
        | FeedbackEventType::VoiceIdDrift
        | FeedbackEventType::VoiceIdLowQuality => FeedbackPathType::Improvement,
        FeedbackEventType::SttReject
        | FeedbackEventType::SttRetry
        | FeedbackEventType::LanguageMismatch
        | FeedbackEventType::ConfirmAbort
        | FeedbackEventType::ToolFail
        | FeedbackEventType::VoiceIdFalseReject
        | FeedbackEventType::VoiceIdFalseAccept
        | FeedbackEventType::VoiceIdMultiSpeaker
        | FeedbackEventType::VoiceIdConfusionPair
        | FeedbackEventType::VoiceIdSpoofRisk => FeedbackPathType::Defect,
    }
}

fn default_feedback_gold_status(path_type: FeedbackPathType) -> FeedbackGoldStatus {
    match path_type {
        FeedbackPathType::Defect => FeedbackGoldStatus::NotRequired,
        FeedbackPathType::Improvement => FeedbackGoldStatus::Pending,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> FeedbackRequestEnvelope {
        FeedbackRequestEnvelope::v1(CorrelationId(3101), TurnId(281), 8, 4).unwrap()
    }

    fn event(event_id: &str, event_type: FeedbackEventType) -> FeedbackEventRecord {
        FeedbackEventRecord::v1(
            event_id.to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3101),
            TurnId(281),
            event_type,
            ReasonCodeId(7),
            "evidence:feedback:1".to_string(),
            "idem:feedback:1".to_string(),
            FeedbackMetrics::v1(
                320,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn candidate(candidate_id: &str) -> FeedbackSignalCandidate {
        FeedbackSignalCandidate::v1(
            candidate_id.to_string(),
            FeedbackEventType::SttReject,
            "stt_reject_rate_by_env".to_string(),
            FeedbackSignalTarget::PaeScorecard,
            1200,
            1,
            "evidence:feedback:1".to_string(),
        )
        .unwrap()
    }

    #[test]
    fn feedback_collect_request_is_schema_valid() {
        let req = FeedbackEventCollectRequest::v1(
            envelope(),
            vec![event("event_1", FeedbackEventType::SttReject)],
        )
        .unwrap();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn feedback_signal_emit_request_requires_selected_candidate() {
        let req = FeedbackSignalEmitRequest::v1(
            envelope(),
            "missing".to_string(),
            vec![candidate("candidate_1")],
        );
        assert!(req.is_err());
    }

    #[test]
    fn feedback_event_collect_ok_requires_selected_candidate_present() {
        let out = FeedbackEventCollectOk::v1(
            ReasonCodeId(1),
            "missing".to_string(),
            vec![candidate("candidate_1")],
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn feedback_signal_emit_ok_fail_requires_diagnostics() {
        let out = FeedbackSignalEmitOk::v1(
            ReasonCodeId(2),
            FeedbackValidationStatus::Fail,
            vec![],
            true,
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn feedback_improvement_event_verified_requires_gold_refs() {
        let out = FeedbackEventRecord::v2(
            "event_improve_1".to_string(),
            "tenant_1".to_string(),
            "user_1".to_string(),
            "speaker_1".to_string(),
            "session_1".to_string(),
            "device_1".to_string(),
            CorrelationId(3101),
            TurnId(281),
            FeedbackEventType::UserCorrection,
            FeedbackPathType::Improvement,
            None,
            None,
            FeedbackGoldStatus::Verified,
            ReasonCodeId(7),
            "evidence:feedback:1".to_string(),
            "idem:feedback:improve".to_string(),
            FeedbackMetrics::v1(
                320,
                1,
                FeedbackConfidenceBucket::Low,
                vec!["when".to_string()],
                FeedbackToolStatus::Fail,
            )
            .unwrap(),
        );
        assert!(out.is_err());
    }

    #[test]
    fn feedback_improvement_signal_candidate_allows_pending_gold() {
        let out = FeedbackSignalCandidate::v2(
            "candidate_improve_1".to_string(),
            FeedbackEventType::UserCorrection,
            "correction_rate_by_intent".to_string(),
            FeedbackSignalTarget::LearnPackage,
            FeedbackPathType::Improvement,
            None,
            FeedbackGoldStatus::Pending,
            900,
            1,
            "evidence:feedback:1".to_string(),
        );
        assert!(out.is_ok());
    }

    #[test]
    fn feedback_voice_taxonomy_classifier_is_deterministic() {
        assert_eq!(
            classify_feedback_path(FeedbackEventType::VoiceIdMultiSpeaker),
            FeedbackPathType::Defect
        );
        assert_eq!(
            classify_feedback_path(FeedbackEventType::VoiceIdSpoofRisk),
            FeedbackPathType::Defect
        );
        assert_eq!(
            classify_feedback_path(FeedbackEventType::VoiceIdDriftAlert),
            FeedbackPathType::Improvement
        );
        assert_eq!(
            classify_feedback_path(FeedbackEventType::VoiceIdReauthFriction),
            FeedbackPathType::Improvement
        );

        let event = event("event_voice_taxonomy", FeedbackEventType::VoiceIdDriftAlert);
        assert_eq!(event.path_type, FeedbackPathType::Improvement);
        assert_eq!(event.gold_status, FeedbackGoldStatus::Pending);
    }
}
