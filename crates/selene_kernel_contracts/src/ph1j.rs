#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use crate::ph1_voice_id::UserId;
use crate::ph1l::SessionId;
use crate::ph1simcat::{SimulationId, SimulationVersion};
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1J_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1J_PROOF_SCHEMA_VERSION: SchemaVersion = SchemaVersion(2);

fn validate_opt_id(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(v) = value {
        if v.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not be empty when provided",
            });
        }
        if v.len() > max_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "exceeds max length",
            });
        }
    }
    Ok(())
}

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

fn validate_opt_ascii_token(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value.as_ref() {
        validate_ascii_token(field, value, max_len)?;
    }
    Ok(())
}

fn validate_lower_hex_64(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() != 64
        || !value.is_ascii()
        || !value
            .chars()
            .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase())
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be lowercase hex sha256 (64 chars)",
        });
    }
    Ok(())
}

fn append_canonical_field(buf: &mut String, key: &str, value: &str) {
    buf.push_str(key);
    buf.push('=');
    buf.push_str(&value.len().to_string());
    buf.push(':');
    buf.push_str(value);
    buf.push('\n');
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CorrelationId(pub u128);

impl Validate for CorrelationId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "correlation_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TurnId(pub u64);

impl Validate for TurnId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "turn_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AuditEventId(pub u64);

impl Validate for AuditEventId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(String);

impl DeviceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "device_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "device_id",
                reason: "must be <= 128 chars",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DeviceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "device_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "device_id",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AuditEngine {
    Ph1K,
    Ph1W,
    Ph1C,
    Ph1Nlp,
    Ph1X,
    Ph1D,
    Ph1E,
    Ph1Tts,
    Ph1L,
    Ph1M,
    Ph1Explain,
    Ph1F,
    Ph1J,
    Other(String),
}

impl Validate for AuditEngine {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let AuditEngine::Other(s) = self {
            if s.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_engine.other",
                    reason: "must not be empty",
                });
            }
            if s.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_engine.other",
                    reason: "must be <= 64 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    GatePass,
    GateFail,
    StateTransition,
    TranscriptOk,
    TranscriptReject,
    SttCandidateEval,
    ConversationTurnStored,
    NlpIntentDraft,
    NlpClarify,
    XConfirm,
    XDispatch,
    ToolOk,
    ToolFail,
    MemoryStored,
    MemoryForgotten,
    ExplainEmitted,
    TtsRenderSummary,
    TtsStarted,
    TtsCanceled,
    TtsFailed,
    PerceptionSignalEmitted,
    ArtifactPackApplied,
    ArtifactPackRolledBack,
    RoutingPolicyPromoted,
    RoutingPolicyDemoted,
    JRedactApplied,
    JDeleteExecuted,
    SessionOpen,
    SessionSoftClose,
    SessionClosed,
    SystemSuspend,
    SystemResume,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AuditSeverity {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PayloadKey(String);

fn is_ascii_lower_snake_key(s: &str) -> bool {
    let b = s.as_bytes();
    if b.is_empty() {
        return false;
    }
    if !b[0].is_ascii_lowercase() {
        return false;
    }
    for &c in b.iter().skip(1) {
        if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'_') {
            return false;
        }
    }
    true
}

impl PayloadKey {
    pub fn new(key: impl Into<String>) -> Result<Self, ContractViolation> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must not be empty",
            });
        }
        if key.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be <= 64 chars",
            });
        }
        if !key.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be ASCII",
            });
        }
        if !is_ascii_lower_snake_key(&key) {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be lower_snake_case (a-z0-9_)",
            });
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for PayloadKey {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be <= 64 chars",
            });
        }
        if !self.0.is_ascii() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be ASCII",
            });
        }
        if !is_ascii_lower_snake_key(&self.0) {
            return Err(ContractViolation::InvalidValue {
                field: "payload_key",
                reason: "must be lower_snake_case (a-z0-9_)",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PayloadValue(String);

impl PayloadValue {
    pub fn new(value: impl Into<String>) -> Result<Self, ContractViolation> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_value",
                reason: "must not be empty",
            });
        }
        if value.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "payload_value",
                reason: "must be <= 256 chars",
            });
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for PayloadValue {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "payload_value",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "payload_value",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditPayloadMin {
    pub schema_version: SchemaVersion,
    pub entries: BTreeMap<PayloadKey, PayloadValue>,
}

impl AuditPayloadMin {
    pub fn empty_v1() -> Self {
        Self {
            schema_version: PH1J_CONTRACT_VERSION,
            entries: BTreeMap::new(),
        }
    }

    pub fn v1(entries: BTreeMap<PayloadKey, PayloadValue>) -> Result<Self, ContractViolation> {
        let p = Self {
            schema_version: PH1J_CONTRACT_VERSION,
            entries,
        };
        p.validate()?;
        Ok(p)
    }
}

impl Validate for AuditPayloadMin {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1J_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "audit_payload_min.schema_version",
                reason: "must match PH1J_CONTRACT_VERSION",
            });
        }
        if self.entries.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_payload_min.entries",
                reason: "must be <= 16 entries",
            });
        }
        let mut total_bytes: usize = 0;
        for (k, v) in &self.entries {
            k.validate()?;
            v.validate()?;
            total_bytes = total_bytes.saturating_add(k.as_str().len());
            total_bytes = total_bytes.saturating_add(v.as_str().len());
            if total_bytes > 2048 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_payload_min",
                    reason: "total payload size must be <= 2048 bytes",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvidenceRef {
    pub schema_version: SchemaVersion,
    pub transcript_hash: Option<String>,
    pub memory_ledger_id: Option<u64>,
    pub conversation_turn_id: Option<u64>,
}

impl AuditEvidenceRef {
    pub fn v1(
        transcript_hash: Option<String>,
        memory_ledger_id: Option<u64>,
        conversation_turn_id: Option<u64>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1J_CONTRACT_VERSION,
            transcript_hash,
            memory_ledger_id,
            conversation_turn_id,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for AuditEvidenceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1J_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "audit_evidence_ref.schema_version",
                reason: "must match PH1J_CONTRACT_VERSION",
            });
        }
        if self.transcript_hash.is_none()
            && self.memory_ledger_id.is_none()
            && self.conversation_turn_id.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "audit_evidence_ref",
                reason: "must include at least one reference",
            });
        }
        if let Some(h) = &self.transcript_hash {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_evidence_ref.transcript_hash",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_evidence_ref.transcript_hash",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if let Some(id) = self.memory_ledger_id {
            if id == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_evidence_ref.memory_ledger_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if let Some(id) = self.conversation_turn_id {
            if id == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_evidence_ref.conversation_turn_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventInput {
    pub schema_version: SchemaVersion,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: Option<String>,
    pub work_order_id: Option<String>,
    pub session_id: Option<SessionId>,
    pub user_id: Option<UserId>,
    pub device_id: Option<DeviceId>,
    pub engine: AuditEngine,
    pub event_type: AuditEventType,
    pub reason_code: ReasonCodeId,
    pub severity: AuditSeverity,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub payload_min: AuditPayloadMin,
    pub evidence_ref: Option<AuditEvidenceRef>,
    /// Optional key to detect duplicate emissions deterministically.
    pub idempotency_key: Option<String>,
}

fn allowed_payload_keys_for_event(event_type: AuditEventType) -> Option<&'static [&'static str]> {
    match event_type {
        AuditEventType::GatePass | AuditEventType::GateFail => Some(&["gate"]),
        AuditEventType::StateTransition => Some(&["state_from", "state_to"]),
        AuditEventType::TranscriptOk | AuditEventType::TranscriptReject => {
            Some(&["transcript_hash"])
        }
        AuditEventType::PerceptionSignalEmitted => Some(&[
            "decision",
            "event_kind",
            "event_name",
            "ph1k_event_id",
            "processed_stream_id",
            "raw_stream_id",
            "pre_roll_buffer_id",
            "selected_mic",
            "selected_speaker",
            "device_health",
            "jitter_ms",
            "drift_ppm",
            "buffer_depth_ms",
            "underruns",
            "overruns",
            "tts_playback_active",
            "capture_degraded",
            "aec_unstable",
            "device_changed",
            "stream_gap_detected",
            "phrase_id",
            "trigger_phrase_id",
            "trigger_locale",
            "candidate_confidence_band",
            "vad_decision_confidence_band",
            "risk_context_class",
            "degradation_context",
            "quality_metrics",
            "timing_markers",
            "subject_relation_confidence_bundle",
            "interrupt_profile_refs",
            "adaptive_profile",
        ]),
        _ => None,
    }
}

impl AuditEventInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        created_at: MonotonicTimeNs,
        tenant_id: Option<String>,
        work_order_id: Option<String>,
        session_id: Option<SessionId>,
        user_id: Option<UserId>,
        device_id: Option<DeviceId>,
        engine: AuditEngine,
        event_type: AuditEventType,
        reason_code: ReasonCodeId,
        severity: AuditSeverity,
        correlation_id: CorrelationId,
        turn_id: TurnId,
        payload_min: AuditPayloadMin,
        evidence_ref: Option<AuditEvidenceRef>,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1J_CONTRACT_VERSION,
            created_at,
            tenant_id,
            work_order_id,
            session_id,
            user_id,
            device_id,
            engine,
            event_type,
            reason_code,
            severity,
            correlation_id,
            turn_id,
            payload_min,
            evidence_ref,
            idempotency_key,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for AuditEventInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1J_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event_input.schema_version",
                reason: "must match PH1J_CONTRACT_VERSION",
            });
        }
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event_input.created_at",
                reason: "must be > 0",
            });
        }
        validate_opt_id("audit_event_input.tenant_id", &self.tenant_id, 64)?;
        validate_opt_id("audit_event_input.work_order_id", &self.work_order_id, 128)?;
        if let Some(s) = self.session_id {
            if s.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event_input.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if let Some(u) = &self.user_id {
            if u.as_str().trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event_input.user_id",
                    reason: "must not be empty when provided",
                });
            }
        }
        if let Some(d) = &self.device_id {
            d.validate()?;
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event_input.reason_code",
                reason: "must be > 0",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.engine.validate()?;
        self.payload_min.validate()?;
        if let Some(allowed) = allowed_payload_keys_for_event(self.event_type) {
            for k in self.payload_min.entries.keys() {
                if !allowed.contains(&k.as_str()) {
                    return Err(ContractViolation::InvalidValue {
                        field: "audit_event_input.payload_min.entries",
                        reason: "contains unapproved key for this event_type",
                    });
                }
            }
        }
        if let Some(r) = &self.evidence_ref {
            r.validate()?;
        }
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event_input.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event_input.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEvent {
    pub schema_version: SchemaVersion,
    pub event_id: AuditEventId,
    pub created_at: MonotonicTimeNs,
    pub tenant_id: Option<String>,
    pub work_order_id: Option<String>,
    pub session_id: Option<SessionId>,
    pub user_id: Option<UserId>,
    pub device_id: Option<DeviceId>,
    pub engine: AuditEngine,
    pub event_type: AuditEventType,
    pub reason_code: ReasonCodeId,
    pub severity: AuditSeverity,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub payload_min: AuditPayloadMin,
    pub evidence_ref: Option<AuditEvidenceRef>,
    pub idempotency_key: Option<String>,
}

impl AuditEvent {
    pub fn from_input_v1(
        event_id: AuditEventId,
        input: AuditEventInput,
    ) -> Result<Self, ContractViolation> {
        input.validate()?;
        let e = Self {
            schema_version: PH1J_CONTRACT_VERSION,
            event_id,
            created_at: input.created_at,
            tenant_id: input.tenant_id,
            work_order_id: input.work_order_id,
            session_id: input.session_id,
            user_id: input.user_id,
            device_id: input.device_id,
            engine: input.engine,
            event_type: input.event_type,
            reason_code: input.reason_code,
            severity: input.severity,
            correlation_id: input.correlation_id,
            turn_id: input.turn_id,
            payload_min: input.payload_min,
            evidence_ref: input.evidence_ref,
            idempotency_key: input.idempotency_key,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for AuditEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1J_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event.schema_version",
                reason: "must match PH1J_CONTRACT_VERSION",
            });
        }
        self.event_id.validate()?;
        if self.created_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event.created_at",
                reason: "must be > 0",
            });
        }
        validate_opt_id("audit_event.tenant_id", &self.tenant_id, 64)?;
        validate_opt_id("audit_event.work_order_id", &self.work_order_id, 128)?;
        if let Some(s) = self.session_id {
            if s.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event.session_id",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if let Some(u) = &self.user_id {
            if u.as_str().trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event.user_id",
                    reason: "must not be empty when provided",
                });
            }
        }
        if let Some(d) = &self.device_id {
            d.validate()?;
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "audit_event.reason_code",
                reason: "must be > 0",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.engine.validate()?;
        self.payload_min.validate()?;
        if let Some(allowed) = allowed_payload_keys_for_event(self.event_type) {
            for k in self.payload_min.entries.keys() {
                if !allowed.contains(&k.as_str()) {
                    return Err(ContractViolation::InvalidValue {
                        field: "audit_event.payload_min.entries",
                        reason: "contains unapproved key for this event_type",
                    });
                }
            }
        }
        if let Some(r) = &self.evidence_ref {
            r.validate()?;
        }
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "audit_event.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ProofEventId(pub u64);

impl Validate for ProofEventId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "proof_event_id",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofProtectedActionClass {
    IdentitySensitiveExecution,
    AccessControlledAction,
    SimulationAuthorizedMutation,
    ArtifactAuthoritativeAction,
    ProtectedLinkGeneration,
    MemoryAuthoritativeMutation,
    GovernanceDecision,
    PrimaryDeviceConfirmation,
    VoiceTurnExecution,
}

impl ProofProtectedActionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofProtectedActionClass::IdentitySensitiveExecution => {
                "IDENTITY_SENSITIVE_EXECUTION"
            }
            ProofProtectedActionClass::AccessControlledAction => "ACCESS_CONTROLLED_ACTION",
            ProofProtectedActionClass::SimulationAuthorizedMutation => {
                "SIMULATION_AUTHORIZED_MUTATION"
            }
            ProofProtectedActionClass::ArtifactAuthoritativeAction => {
                "ARTIFACT_AUTHORITATIVE_ACTION"
            }
            ProofProtectedActionClass::ProtectedLinkGeneration => "PROTECTED_LINK_GENERATION",
            ProofProtectedActionClass::MemoryAuthoritativeMutation => {
                "MEMORY_AUTHORITATIVE_MUTATION"
            }
            ProofProtectedActionClass::GovernanceDecision => "GOVERNANCE_DECISION",
            ProofProtectedActionClass::PrimaryDeviceConfirmation => {
                "PRIMARY_DEVICE_CONFIRMATION"
            }
            ProofProtectedActionClass::VoiceTurnExecution => "VOICE_TURN_EXECUTION",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofFailureClass {
    ProofWriteFailure,
    ProofChainIntegrityFailure,
    ProofSignatureFailure,
    ProofCanonicalizationFailure,
    ProofStorageUnavailable,
    ProofVerificationUnavailable,
}

impl ProofFailureClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofFailureClass::ProofWriteFailure => "PROOF_WRITE_FAILURE",
            ProofFailureClass::ProofChainIntegrityFailure => "PROOF_CHAIN_INTEGRITY_FAILURE",
            ProofFailureClass::ProofSignatureFailure => "PROOF_SIGNATURE_FAILURE",
            ProofFailureClass::ProofCanonicalizationFailure => {
                "PROOF_CANONICALIZATION_FAILURE"
            }
            ProofFailureClass::ProofStorageUnavailable => "PROOF_STORAGE_UNAVAILABLE",
            ProofFailureClass::ProofVerificationUnavailable => {
                "PROOF_VERIFICATION_UNAVAILABLE"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofWriteOutcome {
    NotRequired,
    Written,
    ReusedExisting,
    Failed,
}

impl ProofWriteOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofWriteOutcome::NotRequired => "NOT_REQUIRED",
            ProofWriteOutcome::Written => "WRITTEN",
            ProofWriteOutcome::ReusedExisting => "REUSED_EXISTING",
            ProofWriteOutcome::Failed => "FAILED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofChainStatus {
    NotChecked,
    ChainOrigin,
    ChainLinked,
    ChainGapDetected,
    ChainBreakDetected,
}

impl ProofChainStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofChainStatus::NotChecked => "NOT_CHECKED",
            ProofChainStatus::ChainOrigin => "CHAIN_ORIGIN",
            ProofChainStatus::ChainLinked => "CHAIN_LINKED",
            ProofChainStatus::ChainGapDetected => "CHAIN_GAP_DETECTED",
            ProofChainStatus::ChainBreakDetected => "CHAIN_BREAK_DETECTED",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofVerificationPosture {
    NotRequested,
    VerificationReady,
    VerificationUnavailable,
    RedactedVerifiable,
}

impl ProofVerificationPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofVerificationPosture::NotRequested => "NOT_REQUESTED",
            ProofVerificationPosture::VerificationReady => "VERIFICATION_READY",
            ProofVerificationPosture::VerificationUnavailable => "VERIFICATION_UNAVAILABLE",
            ProofVerificationPosture::RedactedVerifiable => "REDACTED_VERIFIABLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimestampTrustPosture {
    RuntimeMonotonic,
    TrustedAttested,
    TrustedTimeUnavailable,
}

impl TimestampTrustPosture {
    pub const fn as_str(self) -> &'static str {
        match self {
            TimestampTrustPosture::RuntimeMonotonic => "RUNTIME_MONOTONIC",
            TimestampTrustPosture::TrustedAttested => "TRUSTED_ATTESTED",
            TimestampTrustPosture::TrustedTimeUnavailable => "TRUSTED_TIME_UNAVAILABLE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProofRetentionClass {
    ShortRetention,
    ComplianceRetention,
    LegalGradeRetention,
    PermanentRetention,
}

impl ProofRetentionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            ProofRetentionClass::ShortRetention => "SHORT_RETENTION",
            ProofRetentionClass::ComplianceRetention => "COMPLIANCE_RETENTION",
            ProofRetentionClass::LegalGradeRetention => "LEGAL_GRADE_RETENTION",
            ProofRetentionClass::PermanentRetention => "PERMANENT_RETENTION",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofSignerIdentityMetadata {
    pub signer_identity: String,
    pub key_id: String,
    pub signature_algorithm: String,
}

impl ProofSignerIdentityMetadata {
    pub fn v1(
        signer_identity: String,
        key_id: String,
        signature_algorithm: String,
    ) -> Result<Self, ContractViolation> {
        let metadata = Self {
            signer_identity,
            key_id,
            signature_algorithm,
        };
        metadata.validate()?;
        Ok(metadata)
    }
}

impl Validate for ProofSignerIdentityMetadata {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_ascii_token(
            "proof_signer_identity_metadata.signer_identity",
            &self.signer_identity,
            128,
        )?;
        validate_ascii_token("proof_signer_identity_metadata.key_id", &self.key_id, 128)?;
        validate_ascii_token(
            "proof_signer_identity_metadata.signature_algorithm",
            &self.signature_algorithm,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalProofRecordInput {
    pub schema_version: SchemaVersion,
    pub request_id: String,
    pub trace_id: String,
    pub session_id: Option<SessionId>,
    pub turn_id: Option<TurnId>,
    pub actor_identity_scope: Option<String>,
    pub device_id: Option<DeviceId>,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub action_class: ProofProtectedActionClass,
    pub authority_decision_reference: Option<String>,
    pub policy_rule_identifiers: Vec<String>,
    pub policy_version: Option<String>,
    pub simulation_id: Option<SimulationId>,
    pub simulation_version: Option<SimulationVersion>,
    pub simulation_certification_state: Option<String>,
    pub execution_outcome: String,
    pub failure_class: Option<String>,
    pub reason_codes: Vec<ReasonCodeId>,
    pub received_at: MonotonicTimeNs,
    pub executed_at: MonotonicTimeNs,
    pub signer_identity_metadata: ProofSignerIdentityMetadata,
    pub proof_retention_class: ProofRetentionClass,
    pub proof_verification_posture: ProofVerificationPosture,
    pub timestamp_trust_posture: TimestampTrustPosture,
    pub verifier_metadata_ref: Option<String>,
}

impl CanonicalProofRecordInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        request_id: String,
        trace_id: String,
        session_id: Option<SessionId>,
        turn_id: Option<TurnId>,
        actor_identity_scope: Option<String>,
        device_id: Option<DeviceId>,
        node_id: String,
        runtime_instance_identity: String,
        environment_identity: String,
        build_version: String,
        git_commit: String,
        action_class: ProofProtectedActionClass,
        authority_decision_reference: Option<String>,
        policy_rule_identifiers: Vec<String>,
        policy_version: Option<String>,
        simulation_id: Option<SimulationId>,
        simulation_version: Option<SimulationVersion>,
        simulation_certification_state: Option<String>,
        execution_outcome: String,
        failure_class: Option<String>,
        reason_codes: Vec<ReasonCodeId>,
        received_at: MonotonicTimeNs,
        executed_at: MonotonicTimeNs,
        signer_identity_metadata: ProofSignerIdentityMetadata,
        proof_retention_class: ProofRetentionClass,
        proof_verification_posture: ProofVerificationPosture,
        timestamp_trust_posture: TimestampTrustPosture,
        verifier_metadata_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let record = Self {
            schema_version: PH1J_PROOF_SCHEMA_VERSION,
            request_id,
            trace_id,
            session_id,
            turn_id,
            actor_identity_scope,
            device_id,
            node_id,
            runtime_instance_identity,
            environment_identity,
            build_version,
            git_commit,
            action_class,
            authority_decision_reference,
            policy_rule_identifiers,
            policy_version,
            simulation_id,
            simulation_version,
            simulation_certification_state,
            execution_outcome,
            failure_class,
            reason_codes,
            received_at,
            executed_at,
            signer_identity_metadata,
            proof_retention_class,
            proof_verification_posture,
            timestamp_trust_posture,
            verifier_metadata_ref,
        };
        record.validate()?;
        Ok(record)
    }

    pub fn canonical_payload(&self) -> String {
        let mut buf = String::new();
        append_canonical_field(&mut buf, "schema_version", &self.schema_version.0.to_string());
        append_canonical_field(&mut buf, "request_id", &self.request_id);
        append_canonical_field(&mut buf, "trace_id", &self.trace_id);
        append_canonical_field(
            &mut buf,
            "session_id",
            &self
                .session_id
                .map(|value| value.0.to_string())
                .unwrap_or_else(|| "-".to_string()),
        );
        append_canonical_field(
            &mut buf,
            "turn_id",
            &self
                .turn_id
                .map(|value| value.0.to_string())
                .unwrap_or_else(|| "-".to_string()),
        );
        append_canonical_field(
            &mut buf,
            "actor_identity_scope",
            self.actor_identity_scope.as_deref().unwrap_or("-"),
        );
        append_canonical_field(
            &mut buf,
            "device_id",
            self.device_id.as_ref().map(DeviceId::as_str).unwrap_or("-"),
        );
        append_canonical_field(&mut buf, "node_id", &self.node_id);
        append_canonical_field(
            &mut buf,
            "runtime_instance_identity",
            &self.runtime_instance_identity,
        );
        append_canonical_field(&mut buf, "environment_identity", &self.environment_identity);
        append_canonical_field(&mut buf, "build_version", &self.build_version);
        append_canonical_field(&mut buf, "git_commit", &self.git_commit);
        append_canonical_field(&mut buf, "action_class", self.action_class.as_str());
        append_canonical_field(
            &mut buf,
            "authority_decision_reference",
            self.authority_decision_reference.as_deref().unwrap_or("-"),
        );
        append_canonical_field(
            &mut buf,
            "policy_rule_identifiers",
            &self
                .policy_rule_identifiers
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(","),
        );
        append_canonical_field(
            &mut buf,
            "policy_version",
            self.policy_version.as_deref().unwrap_or("-"),
        );
        append_canonical_field(
            &mut buf,
            "simulation_id",
            self.simulation_id
                .as_ref()
                .map(SimulationId::as_str)
                .unwrap_or("-"),
        );
        append_canonical_field(
            &mut buf,
            "simulation_version",
            &self
                .simulation_version
                .map(|value| value.0.to_string())
                .unwrap_or_else(|| "-".to_string()),
        );
        append_canonical_field(
            &mut buf,
            "simulation_certification_state",
            self.simulation_certification_state.as_deref().unwrap_or("-"),
        );
        append_canonical_field(&mut buf, "execution_outcome", &self.execution_outcome);
        append_canonical_field(
            &mut buf,
            "failure_class",
            self.failure_class.as_deref().unwrap_or("-"),
        );
        append_canonical_field(
            &mut buf,
            "reason_codes",
            &self
                .reason_codes
                .iter()
                .map(|value| value.0.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );
        append_canonical_field(&mut buf, "received_at", &self.received_at.0.to_string());
        append_canonical_field(&mut buf, "executed_at", &self.executed_at.0.to_string());
        append_canonical_field(
            &mut buf,
            "signer_identity",
            &self.signer_identity_metadata.signer_identity,
        );
        append_canonical_field(&mut buf, "signer_key_id", &self.signer_identity_metadata.key_id);
        append_canonical_field(
            &mut buf,
            "signature_algorithm",
            &self.signer_identity_metadata.signature_algorithm,
        );
        append_canonical_field(
            &mut buf,
            "proof_retention_class",
            self.proof_retention_class.as_str(),
        );
        append_canonical_field(
            &mut buf,
            "proof_verification_posture",
            self.proof_verification_posture.as_str(),
        );
        append_canonical_field(
            &mut buf,
            "timestamp_trust_posture",
            self.timestamp_trust_posture.as_str(),
        );
        append_canonical_field(
            &mut buf,
            "verifier_metadata_ref",
            self.verifier_metadata_ref.as_deref().unwrap_or("-"),
        );
        buf
    }
}

impl Validate for CanonicalProofRecordInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1J_PROOF_SCHEMA_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.schema_version",
                reason: "must match PH1J_PROOF_SCHEMA_VERSION",
            });
        }
        validate_ascii_token(
            "canonical_proof_record_input.request_id",
            &self.request_id,
            256,
        )?;
        validate_ascii_token("canonical_proof_record_input.trace_id", &self.trace_id, 256)?;
        if matches!(self.session_id, Some(SessionId(0))) {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.session_id",
                reason: "must be > 0 when provided",
            });
        }
        if let Some(turn_id) = self.turn_id {
            turn_id.validate()?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record_input.actor_identity_scope",
            &self.actor_identity_scope,
            256,
        )?;
        if let Some(device_id) = self.device_id.as_ref() {
            device_id.validate()?;
        }
        validate_ascii_token("canonical_proof_record_input.node_id", &self.node_id, 128)?;
        validate_ascii_token(
            "canonical_proof_record_input.runtime_instance_identity",
            &self.runtime_instance_identity,
            128,
        )?;
        validate_ascii_token(
            "canonical_proof_record_input.environment_identity",
            &self.environment_identity,
            128,
        )?;
        validate_ascii_token(
            "canonical_proof_record_input.build_version",
            &self.build_version,
            128,
        )?;
        validate_ascii_token("canonical_proof_record_input.git_commit", &self.git_commit, 64)?;
        validate_opt_ascii_token(
            "canonical_proof_record_input.authority_decision_reference",
            &self.authority_decision_reference,
            256,
        )?;
        if self.policy_rule_identifiers.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.policy_rule_identifiers",
                reason: "must be <= 32",
            });
        }
        for rule_id in &self.policy_rule_identifiers {
            validate_ascii_token(
                "canonical_proof_record_input.policy_rule_identifiers",
                rule_id,
                128,
            )?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record_input.policy_version",
            &self.policy_version,
            128,
        )?;
        if let Some(simulation_id) = self.simulation_id.as_ref() {
            simulation_id.validate()?;
        }
        if let Some(simulation_version) = self.simulation_version {
            simulation_version.validate()?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record_input.simulation_certification_state",
            &self.simulation_certification_state,
            64,
        )?;
        validate_ascii_token(
            "canonical_proof_record_input.execution_outcome",
            &self.execution_outcome,
            64,
        )?;
        validate_opt_ascii_token(
            "canonical_proof_record_input.failure_class",
            &self.failure_class,
            64,
        )?;
        if self.reason_codes.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.reason_codes",
                reason: "must be <= 32",
            });
        }
        for reason_code in &self.reason_codes {
            if reason_code.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "canonical_proof_record_input.reason_codes",
                    reason: "must be > 0",
                });
            }
        }
        if self.received_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.received_at",
                reason: "must be > 0",
            });
        }
        if self.executed_at.0 == 0 || self.executed_at.0 < self.received_at.0 {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record_input.executed_at",
                reason: "must be >= received_at",
            });
        }
        self.signer_identity_metadata.validate()?;
        validate_opt_ascii_token(
            "canonical_proof_record_input.verifier_metadata_ref",
            &self.verifier_metadata_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalProofRecord {
    pub proof_schema_version: SchemaVersion,
    pub proof_event_id: ProofEventId,
    pub request_id: String,
    pub trace_id: String,
    pub session_id: Option<SessionId>,
    pub turn_id: Option<TurnId>,
    pub actor_identity_scope: Option<String>,
    pub device_id: Option<DeviceId>,
    pub node_id: String,
    pub runtime_instance_identity: String,
    pub environment_identity: String,
    pub build_version: String,
    pub git_commit: String,
    pub action_class: ProofProtectedActionClass,
    pub authority_decision_reference: Option<String>,
    pub policy_rule_identifiers: Vec<String>,
    pub policy_version: Option<String>,
    pub simulation_id: Option<SimulationId>,
    pub simulation_version: Option<SimulationVersion>,
    pub simulation_certification_state: Option<String>,
    pub execution_outcome: String,
    pub failure_class: Option<String>,
    pub reason_codes: Vec<ReasonCodeId>,
    pub received_at: MonotonicTimeNs,
    pub executed_at: MonotonicTimeNs,
    pub proof_payload_hash: String,
    pub previous_event_hash: Option<String>,
    pub current_event_hash: String,
    pub signer_identity_metadata: ProofSignerIdentityMetadata,
    pub signature: String,
    pub proof_retention_class: ProofRetentionClass,
    pub proof_verification_posture: ProofVerificationPosture,
    pub timestamp_trust_posture: TimestampTrustPosture,
    pub verifier_metadata_ref: Option<String>,
}

impl CanonicalProofRecord {
    pub fn canonical_payload(&self) -> String {
        CanonicalProofRecordInput {
            schema_version: self.proof_schema_version,
            request_id: self.request_id.clone(),
            trace_id: self.trace_id.clone(),
            session_id: self.session_id,
            turn_id: self.turn_id,
            actor_identity_scope: self.actor_identity_scope.clone(),
            device_id: self.device_id.clone(),
            node_id: self.node_id.clone(),
            runtime_instance_identity: self.runtime_instance_identity.clone(),
            environment_identity: self.environment_identity.clone(),
            build_version: self.build_version.clone(),
            git_commit: self.git_commit.clone(),
            action_class: self.action_class,
            authority_decision_reference: self.authority_decision_reference.clone(),
            policy_rule_identifiers: self.policy_rule_identifiers.clone(),
            policy_version: self.policy_version.clone(),
            simulation_id: self.simulation_id.clone(),
            simulation_version: self.simulation_version,
            simulation_certification_state: self.simulation_certification_state.clone(),
            execution_outcome: self.execution_outcome.clone(),
            failure_class: self.failure_class.clone(),
            reason_codes: self.reason_codes.clone(),
            received_at: self.received_at,
            executed_at: self.executed_at,
            signer_identity_metadata: self.signer_identity_metadata.clone(),
            proof_retention_class: self.proof_retention_class,
            proof_verification_posture: self.proof_verification_posture,
            timestamp_trust_posture: self.timestamp_trust_posture,
            verifier_metadata_ref: self.verifier_metadata_ref.clone(),
        }
        .canonical_payload()
    }
}

impl Validate for CanonicalProofRecord {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.proof_schema_version != PH1J_PROOF_SCHEMA_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record.proof_schema_version",
                reason: "must match PH1J_PROOF_SCHEMA_VERSION",
            });
        }
        self.proof_event_id.validate()?;
        validate_ascii_token("canonical_proof_record.request_id", &self.request_id, 256)?;
        validate_ascii_token("canonical_proof_record.trace_id", &self.trace_id, 256)?;
        if matches!(self.session_id, Some(SessionId(0))) {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record.session_id",
                reason: "must be > 0 when provided",
            });
        }
        if let Some(turn_id) = self.turn_id {
            turn_id.validate()?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record.actor_identity_scope",
            &self.actor_identity_scope,
            256,
        )?;
        if let Some(device_id) = self.device_id.as_ref() {
            device_id.validate()?;
        }
        validate_ascii_token("canonical_proof_record.node_id", &self.node_id, 128)?;
        validate_ascii_token(
            "canonical_proof_record.runtime_instance_identity",
            &self.runtime_instance_identity,
            128,
        )?;
        validate_ascii_token(
            "canonical_proof_record.environment_identity",
            &self.environment_identity,
            128,
        )?;
        validate_ascii_token(
            "canonical_proof_record.build_version",
            &self.build_version,
            128,
        )?;
        validate_ascii_token("canonical_proof_record.git_commit", &self.git_commit, 64)?;
        validate_opt_ascii_token(
            "canonical_proof_record.authority_decision_reference",
            &self.authority_decision_reference,
            256,
        )?;
        for rule_id in &self.policy_rule_identifiers {
            validate_ascii_token(
                "canonical_proof_record.policy_rule_identifiers",
                rule_id,
                128,
            )?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record.policy_version",
            &self.policy_version,
            128,
        )?;
        if let Some(simulation_id) = self.simulation_id.as_ref() {
            simulation_id.validate()?;
        }
        if let Some(simulation_version) = self.simulation_version {
            simulation_version.validate()?;
        }
        validate_opt_ascii_token(
            "canonical_proof_record.simulation_certification_state",
            &self.simulation_certification_state,
            64,
        )?;
        validate_ascii_token(
            "canonical_proof_record.execution_outcome",
            &self.execution_outcome,
            64,
        )?;
        validate_opt_ascii_token(
            "canonical_proof_record.failure_class",
            &self.failure_class,
            64,
        )?;
        if self.received_at.0 == 0 || self.executed_at.0 == 0 || self.executed_at.0 < self.received_at.0
        {
            return Err(ContractViolation::InvalidValue {
                field: "canonical_proof_record.executed_at",
                reason: "must be >= received_at",
            });
        }
        validate_lower_hex_64(
            "canonical_proof_record.proof_payload_hash",
            &self.proof_payload_hash,
        )?;
        if let Some(previous_event_hash) = self.previous_event_hash.as_ref() {
            validate_lower_hex_64(
                "canonical_proof_record.previous_event_hash",
                previous_event_hash,
            )?;
        }
        validate_lower_hex_64(
            "canonical_proof_record.current_event_hash",
            &self.current_event_hash,
        )?;
        self.signer_identity_metadata.validate()?;
        validate_lower_hex_64("canonical_proof_record.signature", &self.signature)?;
        validate_opt_ascii_token(
            "canonical_proof_record.verifier_metadata_ref",
            &self.verifier_metadata_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofWriteReceipt {
    pub proof_event_id: ProofEventId,
    pub proof_record_ref: String,
    pub proof_write_outcome: ProofWriteOutcome,
    pub proof_chain_status: ProofChainStatus,
    pub proof_verification_posture: ProofVerificationPosture,
    pub timestamp_trust_posture: TimestampTrustPosture,
    pub verifier_metadata_ref: Option<String>,
}

impl ProofWriteReceipt {
    pub fn v1(
        proof_event_id: ProofEventId,
        proof_record_ref: String,
        proof_write_outcome: ProofWriteOutcome,
        proof_chain_status: ProofChainStatus,
        proof_verification_posture: ProofVerificationPosture,
        timestamp_trust_posture: TimestampTrustPosture,
        verifier_metadata_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let receipt = Self {
            proof_event_id,
            proof_record_ref,
            proof_write_outcome,
            proof_chain_status,
            proof_verification_posture,
            timestamp_trust_posture,
            verifier_metadata_ref,
        };
        receipt.validate()?;
        Ok(receipt)
    }
}

impl Validate for ProofWriteReceipt {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.proof_event_id.validate()?;
        validate_ascii_token(
            "proof_write_receipt.proof_record_ref",
            &self.proof_record_ref,
            128,
        )?;
        validate_opt_ascii_token(
            "proof_write_receipt.verifier_metadata_ref",
            &self.verifier_metadata_ref,
            256,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofVerificationResult {
    pub proof_event_id: ProofEventId,
    pub verified: bool,
    pub failure_class: Option<ProofFailureClass>,
    pub proof_chain_status: ProofChainStatus,
    pub proof_verification_posture: ProofVerificationPosture,
}

impl Validate for ProofVerificationResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.proof_event_id.validate()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_rejects_too_many_entries() {
        let mut m: BTreeMap<PayloadKey, PayloadValue> = BTreeMap::new();
        for i in 0..17 {
            m.insert(
                PayloadKey::new(format!("k{i}")).unwrap(),
                PayloadValue::new("v").unwrap(),
            );
        }
        assert!(AuditPayloadMin::v1(m).is_err());
    }

    #[test]
    fn payload_key_requires_lower_snake_case_ascii() {
        assert!(PayloadKey::new("Gate").is_err());
        assert!(PayloadKey::new("gate-name").is_err());
        assert!(PayloadKey::new("1gate").is_err());
        assert!(PayloadKey::new("gate").is_ok());
        assert!(PayloadKey::new("k1").is_ok());
    }

    #[test]
    fn evidence_ref_requires_at_least_one_reference() {
        let err = AuditEvidenceRef::v1(None, None, None).unwrap_err();
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "audit_evidence_ref")
            }
            _ => panic!("expected InvalidValue"),
        }
    }

    #[test]
    fn perception_signal_payload_rejects_unknown_key() {
        let mut payload = BTreeMap::new();
        payload.insert(
            PayloadKey::new("decision").unwrap(),
            PayloadValue::new("K_RUNTIME_EVENT_COMMIT").unwrap(),
        );
        payload.insert(
            PayloadKey::new("unexpected_key").unwrap(),
            PayloadValue::new("x").unwrap(),
        );
        let payload = AuditPayloadMin::v1(payload).unwrap();
        let err = AuditEventInput::v1(
            MonotonicTimeNs(10),
            Some("tenant_a".to_string()),
            None,
            None,
            None,
            Some(DeviceId::new("dev_1").unwrap()),
            AuditEngine::Ph1K,
            AuditEventType::PerceptionSignalEmitted,
            ReasonCodeId(0x4B00_1001),
            AuditSeverity::Info,
            CorrelationId(1),
            TurnId(1),
            payload,
            None,
            Some("idem_audit_1".to_string()),
        )
        .unwrap_err();
        match err {
            ContractViolation::InvalidValue { field, reason } => {
                assert_eq!(field, "audit_event_input.payload_min.entries");
                assert_eq!(reason, "contains unapproved key for this event_type");
            }
            _ => panic!("expected InvalidValue"),
        }
    }

    #[test]
    fn perception_signal_payload_accepts_ph1k_allowlist_keys() {
        let payload = AuditPayloadMin::v1(BTreeMap::from([
            (
                PayloadKey::new("decision").unwrap(),
                PayloadValue::new("K_RUNTIME_EVENT_COMMIT").unwrap(),
            ),
            (
                PayloadKey::new("event_kind").unwrap(),
                PayloadValue::new("INTERRUPT_CANDIDATE").unwrap(),
            ),
            (
                PayloadKey::new("event_name").unwrap(),
                PayloadValue::new("K_INTERRUPT_CANDIDATE_COMMIT").unwrap(),
            ),
            (
                PayloadKey::new("ph1k_event_id").unwrap(),
                PayloadValue::new("12").unwrap(),
            ),
            (
                PayloadKey::new("candidate_confidence_band").unwrap(),
                PayloadValue::new("HIGH").unwrap(),
            ),
            (
                PayloadKey::new("risk_context_class").unwrap(),
                PayloadValue::new("GUARDED").unwrap(),
            ),
            (
                PayloadKey::new("adaptive_profile").unwrap(),
                PayloadValue::new("BUILTIN|ELEVATED|110|4.000|2.000|0.930").unwrap(),
            ),
        ]))
        .unwrap();
        let ev = AuditEventInput::v1(
            MonotonicTimeNs(11),
            Some("tenant_a".to_string()),
            None,
            None,
            None,
            Some(DeviceId::new("dev_1").unwrap()),
            AuditEngine::Ph1K,
            AuditEventType::PerceptionSignalEmitted,
            ReasonCodeId(0x4B00_1005),
            AuditSeverity::Info,
            CorrelationId(2),
            TurnId(2),
            payload,
            None,
            Some("idem_audit_2".to_string()),
        )
        .expect("allowlisted payload must validate");
        assert_eq!(ev.event_type, AuditEventType::PerceptionSignalEmitted);
    }
}
