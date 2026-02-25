#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use crate::ph1_voice_id::UserId;
use crate::ph1l::SessionId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1J_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

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
