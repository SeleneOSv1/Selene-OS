#![forbid(unsafe_code)]

use crate::ph1_voice_id::Ph1VoiceIdResponse;
use crate::ph1d::PolicyContextRef;
use crate::ph1l::SessionId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1M_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLayer {
    Working,
    Micro,
    LongTerm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemorySensitivityFlag {
    Low,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryUsePolicy {
    /// Safe: may be used freely (e.g., preferred name).
    AlwaysUsable,
    /// Must be repeated or explicitly confirmed before casual use.
    RepeatedOrConfirmed,
    /// Use only when directly relevant to the current turn.
    ContextRelevantOnly,
    /// Use only when user explicitly requests it.
    UserRequestedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryConsent {
    /// No explicit request/confirmation occurred for this entry.
    NotRequested,
    /// User explicitly said "remember this".
    ExplicitRemember,
    /// User confirmed remembering after being asked.
    Confirmed,
    /// User denied remembering.
    Denied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryConfidence {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MemoryKey(String);

impl MemoryKey {
    pub fn new(key: impl Into<String>) -> Result<Self, ContractViolation> {
        let key = key.into();
        if key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_key",
                reason: "must not be empty",
            });
        }
        if key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(Self(key))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryValue {
    pub verbatim: String,
    pub normalized: Option<String>,
}

impl MemoryValue {
    pub fn v1(verbatim: String, normalized: Option<String>) -> Result<Self, ContractViolation> {
        let v = Self {
            verbatim,
            normalized,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for MemoryValue {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.verbatim.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_value.verbatim",
                reason: "must not be empty",
            });
        }
        if self.verbatim.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_value.verbatim",
                reason: "must be <= 256 chars",
            });
        }
        if let Some(n) = &self.normalized {
            if n.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_value.normalized",
                    reason: "must not be empty when provided",
                });
            }
            if n.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_value.normalized",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProvenance {
    pub schema_version: SchemaVersion,
    pub session_id: Option<SessionId>,
    pub transcript_hash: Option<String>,
}

impl MemoryProvenance {
    pub fn v1(
        session_id: Option<SessionId>,
        transcript_hash: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let p = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            session_id,
            transcript_hash,
        };
        p.validate()?;
        Ok(p)
    }
}

impl Validate for MemoryProvenance {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(h) = &self.transcript_hash {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_provenance.transcript_hash",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_provenance.transcript_hash",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryProposedItem {
    pub schema_version: SchemaVersion,
    pub memory_key: MemoryKey,
    pub memory_value: MemoryValue,
    pub layer: MemoryLayer,
    pub sensitivity_flag: MemorySensitivityFlag,
    pub confidence: MemoryConfidence,
    pub consent: MemoryConsent,
    pub evidence_quote: String,
    pub provenance: MemoryProvenance,
}

impl MemoryProposedItem {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        memory_key: MemoryKey,
        memory_value: MemoryValue,
        layer: MemoryLayer,
        sensitivity_flag: MemorySensitivityFlag,
        confidence: MemoryConfidence,
        consent: MemoryConsent,
        evidence_quote: String,
        provenance: MemoryProvenance,
    ) -> Result<Self, ContractViolation> {
        let p = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_key,
            memory_value,
            layer,
            sensitivity_flag,
            confidence,
            consent,
            evidence_quote,
            provenance,
        };
        p.validate()?;
        Ok(p)
    }
}

impl Validate for MemoryProposedItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.memory_value.validate()?;
        self.provenance.validate()?;
        if self.evidence_quote.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_proposed_item.evidence_quote",
                reason: "must not be empty",
            });
        }
        if self.evidence_quote.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_proposed_item.evidence_quote",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryCommitStatus {
    Stored,
    Updated,
    NeedsConsent,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryCommitDecision {
    pub schema_version: SchemaVersion,
    pub memory_key: MemoryKey,
    pub status: MemoryCommitStatus,
    pub reason_code: ReasonCodeId,
    pub consent_prompt: Option<String>,
}

impl MemoryCommitDecision {
    pub fn v1(
        memory_key: MemoryKey,
        status: MemoryCommitStatus,
        reason_code: ReasonCodeId,
        consent_prompt: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_key,
            status,
            reason_code,
            consent_prompt,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for MemoryCommitDecision {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(p) = &self.consent_prompt {
            if p.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_commit_decision.consent_prompt",
                    reason: "must not be empty when provided",
                });
            }
            if p.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_commit_decision.consent_prompt",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryLedgerEventKind {
    Stored,
    Updated,
    Forgotten,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLedgerEvent {
    pub schema_version: SchemaVersion,
    pub kind: MemoryLedgerEventKind,
    pub t_event: MonotonicTimeNs,
    pub memory_key: MemoryKey,
    pub memory_value: Option<MemoryValue>,
    pub evidence_quote: Option<String>,
    pub provenance: MemoryProvenance,
    pub layer: MemoryLayer,
    pub sensitivity_flag: MemorySensitivityFlag,
    pub confidence: MemoryConfidence,
    pub consent: MemoryConsent,
    pub reason_code: ReasonCodeId,
}

impl MemoryLedgerEvent {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        kind: MemoryLedgerEventKind,
        t_event: MonotonicTimeNs,
        memory_key: MemoryKey,
        memory_value: Option<MemoryValue>,
        evidence_quote: Option<String>,
        provenance: MemoryProvenance,
        layer: MemoryLayer,
        sensitivity_flag: MemorySensitivityFlag,
        confidence: MemoryConfidence,
        consent: MemoryConsent,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            kind,
            t_event,
            memory_key,
            memory_value,
            evidence_quote,
            provenance,
            layer,
            sensitivity_flag,
            confidence,
            consent,
            reason_code,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for MemoryLedgerEvent {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.provenance.validate()?;
        if let Some(v) = &self.memory_value {
            v.validate()?;
        }
        if let Some(q) = &self.evidence_quote {
            if q.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_ledger_event.evidence_quote",
                    reason: "must not be empty when provided",
                });
            }
            if q.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_ledger_event.evidence_quote",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryCandidate {
    pub schema_version: SchemaVersion,
    pub memory_key: MemoryKey,
    pub memory_value: MemoryValue,
    pub confidence: MemoryConfidence,
    pub last_seen_at: MonotonicTimeNs,
    pub evidence_quote: String,
    pub provenance: MemoryProvenance,
    pub sensitivity_flag: MemorySensitivityFlag,
    pub use_policy: MemoryUsePolicy,
    pub expires_at: Option<MonotonicTimeNs>,
}

impl MemoryCandidate {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        memory_key: MemoryKey,
        memory_value: MemoryValue,
        confidence: MemoryConfidence,
        last_seen_at: MonotonicTimeNs,
        evidence_quote: String,
        provenance: MemoryProvenance,
        sensitivity_flag: MemorySensitivityFlag,
        use_policy: MemoryUsePolicy,
        expires_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_key,
            memory_value,
            confidence,
            last_seen_at,
            evidence_quote,
            provenance,
            sensitivity_flag,
            use_policy,
            expires_at,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for MemoryCandidate {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.memory_value.validate()?;
        self.provenance.validate()?;
        if self.evidence_quote.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_candidate.evidence_quote",
                reason: "must not be empty",
            });
        }
        if self.evidence_quote.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_candidate.evidence_quote",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mProposeRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub proposals: Vec<MemoryProposedItem>,
}

impl Ph1mProposeRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        proposals: Vec<MemoryProposedItem>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            proposals,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mProposeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.proposals.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_propose_request.proposals",
                reason: "must be <= 32 entries",
            });
        }
        for p in &self.proposals {
            p.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mProposeResponse {
    pub schema_version: SchemaVersion,
    pub decisions: Vec<MemoryCommitDecision>,
    pub ledger_events: Vec<MemoryLedgerEvent>,
}

impl Ph1mProposeResponse {
    pub fn v1(
        decisions: Vec<MemoryCommitDecision>,
        ledger_events: Vec<MemoryLedgerEvent>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            decisions,
            ledger_events,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mProposeResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.decisions.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_propose_response.decisions",
                reason: "must be <= 32 entries",
            });
        }
        if self.ledger_events.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_propose_response.ledger_events",
                reason: "must be <= 32 entries",
            });
        }
        for d in &self.decisions {
            d.validate()?;
        }
        for e in &self.ledger_events {
            e.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRecallRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub requested_keys: Vec<MemoryKey>,
    pub allow_sensitive: bool,
    pub max_candidates: u8,
}

impl Ph1mRecallRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        requested_keys: Vec<MemoryKey>,
        allow_sensitive: bool,
        max_candidates: u8,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            requested_keys,
            allow_sensitive,
            max_candidates,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRecallRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.requested_keys.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recall_request.requested_keys",
                reason: "must be <= 32 entries",
            });
        }
        if self.max_candidates == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recall_request.max_candidates",
                reason: "must be > 0",
            });
        }
        if self.max_candidates > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recall_request.max_candidates",
                reason: "must be <= 32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRecallResponse {
    pub schema_version: SchemaVersion,
    pub candidates: Vec<MemoryCandidate>,
    pub fail_reason_code: Option<ReasonCodeId>,
}

impl Ph1mRecallResponse {
    pub fn v1(
        candidates: Vec<MemoryCandidate>,
        fail_reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            candidates,
            fail_reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRecallResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recall_response.candidates",
                reason: "must be <= 32 entries",
            });
        }
        for c in &self.candidates {
            c.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mForgetRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub target_key: MemoryKey,
}

impl Ph1mForgetRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        target_key: MemoryKey,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            target_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mForgetRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mForgetResponse {
    pub schema_version: SchemaVersion,
    pub forgotten: bool,
    pub ledger_event: Option<MemoryLedgerEvent>,
    pub fail_reason_code: Option<ReasonCodeId>,
}

impl Ph1mForgetResponse {
    pub fn v1(
        forgotten: bool,
        ledger_event: Option<MemoryLedgerEvent>,
        fail_reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            forgotten,
            ledger_event,
            fail_reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mForgetResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if let Some(e) = &self.ledger_event {
            e.validate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, SpeakerAssertionOk, SpeakerAssertionUnknown,
        SpeakerId, SpeakerLabel, UserId,
    };
    use crate::ph1d::{PolicyContextRef, SafetyTier};

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn ok_speaker() -> Ph1VoiceIdResponse {
        Ph1VoiceIdResponse::SpeakerAssertionOk(
            SpeakerAssertionOk::v1(
                SpeakerId::new("spk").unwrap(),
                Some(UserId::new("user").unwrap()),
                vec![DiarizationSegment::v1(
                    MonotonicTimeNs(0),
                    MonotonicTimeNs(1),
                    Some(SpeakerLabel::speaker_a()),
                )
                .unwrap()],
                SpeakerLabel::speaker_a(),
            )
            .unwrap(),
        )
    }

    #[test]
    fn memory_key_rejects_empty() {
        assert!(MemoryKey::new("").is_err());
    }

    #[test]
    fn memory_candidate_requires_evidence_quote() {
        let c = MemoryCandidate::v1(
            MemoryKey::new("preferred_name").unwrap(),
            MemoryValue::v1("John".to_string(), None).unwrap(),
            MemoryConfidence::High,
            MonotonicTimeNs(0),
            "  ".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
            MemorySensitivityFlag::Low,
            MemoryUsePolicy::AlwaysUsable,
            None,
        );
        assert!(c.is_err());
    }

    #[test]
    fn propose_request_allows_unknown_speaker_but_validates_items() {
        let unknown = Ph1VoiceIdResponse::SpeakerAssertionUnknown(
            SpeakerAssertionUnknown::v1(IdentityConfidence::Medium, ReasonCodeId(1), vec![])
                .unwrap(),
        );
        let item = MemoryProposedItem::v1(
            MemoryKey::new("preferred_name").unwrap(),
            MemoryValue::v1("John".to_string(), None).unwrap(),
            MemoryLayer::LongTerm,
            MemorySensitivityFlag::Low,
            MemoryConfidence::High,
            MemoryConsent::NotRequested,
            "My name is John".to_string(),
            MemoryProvenance::v1(Some(SessionId(1)), None).unwrap(),
        )
        .unwrap();

        let req =
            Ph1mProposeRequest::v1(MonotonicTimeNs(0), unknown, policy_ok(), vec![item]).unwrap();
        assert_eq!(req.proposals.len(), 1);
    }

    #[test]
    fn recall_request_rejects_zero_max_candidates() {
        let r = Ph1mRecallRequest::v1(
            MonotonicTimeNs(0),
            ok_speaker(),
            policy_ok(),
            vec![],
            false,
            0,
        );
        assert!(r.is_err());
    }
}
