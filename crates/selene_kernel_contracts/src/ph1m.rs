#![forbid(unsafe_code)]

use crate::ph1_voice_id::Ph1VoiceIdResponse;
use crate::ph1d::PolicyContextRef;
use crate::ph1l::SessionId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1M_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const MEMORY_RESUME_HOT_WINDOW_MS: u64 = 72 * 60 * 60 * 1000;
pub const MEMORY_RESUME_WARM_WINDOW_MS: u64 = 30 * 24 * 60 * 60 * 1000;
pub const MEMORY_UNRESOLVED_DECAY_WINDOW_MS: u64 = 90 * 24 * 60 * 60 * 1000;
pub const MEMORY_CONTEXT_BUNDLE_MAX_BYTES: u32 = 32 * 1024;
pub const MEMORY_CONTEXT_BUNDLE_MAX_ATOMS: u8 = 20;
pub const MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS: u8 = 2;
pub const MEMORY_RECENT_ARCHIVE_RECALL_MAX_MATCHES: u8 = 8;
pub const MEMORY_SAFE_SUMMARY_MAX_BYTES: u16 = 1024;
pub const MEMORY_SAFE_SUMMARY_MAX_ITEMS: u8 = 10;
pub const MEMORY_CANONICAL_TEXT_MAX_CHARS: usize = 1024;
pub const MEMORY_CANONICAL_LABEL_MAX_CHARS: usize = 128;
pub const MEMORY_CANONICAL_MAX_REFS: usize = 32;

fn validate_optional_memory_text(
    field: &'static str,
    value: &Option<String>,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if let Some(value) = value {
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not be empty when provided",
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
    }
    Ok(())
}

fn validate_memory_text_list(
    field: &'static str,
    values: &[String],
    max_items: usize,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max items",
        });
    }
    for value in values {
        if value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain empty entries",
            });
        }
        if value.len() > max_len {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "entry exceeds max length",
            });
        }
        if value.chars().any(char::is_control) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "entries must not contain control chars",
            });
        }
    }
    Ok(())
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryEvidenceType {
    Fresh,
    Today,
    Topic,
    Deep,
    Permanent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryAgeLabel {
    JustNow,
    EarlierThisConversation,
    BeforeSleep,
    EarlierToday,
    Yesterday,
    PastFewDays,
    RecentWeeks,
    OlderTopic,
    Permanent,
}

impl Default for MemoryAgeLabel {
    fn default() -> Self {
        Self::JustNow
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryRecallStyle {
    IRemember,
    WeSpokeAbout,
    EarlierToday,
    Yesterday,
    AWhileBack,
    NoRecord,
}

impl Default for MemoryRecallStyle {
    fn default() -> Self {
        Self::IRemember
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryTrustLevel {
    VerbatimUserInstruction,
    CodexReport,
    InferredSummary,
    AssistantSuggestion,
    UnverifiedIdea,
    OutdatedNote,
}

impl Default for MemoryTrustLevel {
    fn default() -> Self {
        Self::InferredSummary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryPrivacyStatus {
    Allowed,
    Restricted,
    Sensitive,
    RequiresIdentity,
    Unavailable,
}

impl Default for MemoryPrivacyStatus {
    fn default() -> Self {
        Self::Allowed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryConflictStatus {
    Current,
    Superseded,
    Conflicts,
    Stale,
    Unverified,
}

impl Default for MemoryConflictStatus {
    fn default() -> Self {
        Self::Current
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FreshMemoryHandoffReason {
    SessionSleep,
    SessionClose,
    WakeReady,
    TopicShift,
    ManualBoundary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryContinuationDecisionKind {
    ContinueAutomatically,
    AskClarification,
    AnswerNormally,
    NoMemoryMatch,
    BlockedByPrivacy,
    BlockedByStaleness,
    BlockedByLowConfidence,
}

impl Default for MemoryContinuationDecisionKind {
    fn default() -> Self {
        Self::AnswerNormally
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEvidencePacket {
    pub schema_version: SchemaVersion,
    pub memory_type: MemoryEvidenceType,
    pub topic_label: Option<String>,
    pub age_label: MemoryAgeLabel,
    pub confidence: MemoryConfidence,
    pub evidence_refs: Vec<String>,
    pub continuation_allowed: bool,
    pub clarification_needed: bool,
    pub user_facing_summary: Option<String>,
    pub active_context_allowed: bool,
    pub user_facing_recall_style: MemoryRecallStyle,
    pub trust_level: MemoryTrustLevel,
    pub privacy_status: MemoryPrivacyStatus,
    pub conflict_status: MemoryConflictStatus,
}

impl MemoryEvidencePacket {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        memory_type: MemoryEvidenceType,
        topic_label: Option<String>,
        age_label: MemoryAgeLabel,
        confidence: MemoryConfidence,
        evidence_refs: Vec<String>,
        continuation_allowed: bool,
        clarification_needed: bool,
        user_facing_summary: Option<String>,
        active_context_allowed: bool,
        user_facing_recall_style: MemoryRecallStyle,
        trust_level: MemoryTrustLevel,
        privacy_status: MemoryPrivacyStatus,
        conflict_status: MemoryConflictStatus,
    ) -> Result<Self, ContractViolation> {
        let packet = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_type,
            topic_label,
            age_label,
            confidence,
            evidence_refs,
            continuation_allowed,
            clarification_needed,
            user_facing_summary,
            active_context_allowed,
            user_facing_recall_style,
            trust_level,
            privacy_status,
            conflict_status,
        };
        packet.validate()?;
        Ok(packet)
    }
}

impl Validate for MemoryEvidencePacket {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1M_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "memory_evidence_packet.schema_version",
                reason: "must match PH1M_CONTRACT_VERSION",
            });
        }
        validate_optional_memory_text(
            "memory_evidence_packet.topic_label",
            &self.topic_label,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_memory_text_list(
            "memory_evidence_packet.evidence_refs",
            &self.evidence_refs,
            MEMORY_CANONICAL_MAX_REFS,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_evidence_packet.user_facing_summary",
            &self.user_facing_summary,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        if matches!(self.privacy_status, MemoryPrivacyStatus::Unavailable)
            && self.active_context_allowed
        {
            return Err(ContractViolation::InvalidValue {
                field: "memory_evidence_packet.active_context_allowed",
                reason: "must be false when privacy_status is Unavailable",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MemoryRecallRequest {
    pub schema_version: SchemaVersion,
    /// Compatibility wrapper around the existing PH1.M recall request owner.
    pub base_request: Ph1mRecallRequest,
    pub query_text: Option<String>,
    pub current_ph1x_context_ref: Option<String>,
    pub recall_scope: Option<String>,
    pub freshness_preference: Option<MemoryAgeLabel>,
    pub topic_hint: Option<String>,
    pub identity_scope_ref: Option<String>,
    pub privacy_status: MemoryPrivacyStatus,
    pub request_reason: Option<String>,
}

impl MemoryRecallRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        base_request: Ph1mRecallRequest,
        query_text: Option<String>,
        current_ph1x_context_ref: Option<String>,
        recall_scope: Option<String>,
        freshness_preference: Option<MemoryAgeLabel>,
        topic_hint: Option<String>,
        identity_scope_ref: Option<String>,
        privacy_status: MemoryPrivacyStatus,
        request_reason: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let request = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            base_request,
            query_text,
            current_ph1x_context_ref,
            recall_scope,
            freshness_preference,
            topic_hint,
            identity_scope_ref,
            privacy_status,
            request_reason,
        };
        request.validate()?;
        Ok(request)
    }

    pub fn from_ph1m_recall_request(
        base_request: Ph1mRecallRequest,
    ) -> Result<Self, ContractViolation> {
        Self::v1(
            base_request,
            None,
            None,
            None,
            None,
            None,
            None,
            MemoryPrivacyStatus::Allowed,
            None,
        )
    }

    pub fn as_ph1m_recall_request(&self) -> &Ph1mRecallRequest {
        &self.base_request
    }
}

impl Validate for MemoryRecallRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1M_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recall_request.schema_version",
                reason: "must match PH1M_CONTRACT_VERSION",
            });
        }
        self.base_request.validate()?;
        validate_optional_memory_text(
            "memory_recall_request.query_text",
            &self.query_text,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_recall_request.current_ph1x_context_ref",
            &self.current_ph1x_context_ref,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_recall_request.recall_scope",
            &self.recall_scope,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_recall_request.topic_hint",
            &self.topic_hint,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_recall_request.identity_scope_ref",
            &self.identity_scope_ref,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_recall_request.request_reason",
            &self.request_reason,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FreshMemoryHandoff {
    pub schema_version: SchemaVersion,
    pub handoff_id: String,
    pub source_session_id: Option<SessionId>,
    pub source_thread_key: Option<String>,
    pub source_turn_ref: Option<String>,
    pub last_topic: Option<String>,
    pub last_intent: Option<String>,
    pub last_tool_family: Option<String>,
    pub last_entity_focus: Vec<String>,
    pub last_answer_type: Option<String>,
    pub freshness_label: MemoryAgeLabel,
    pub confidence: MemoryConfidence,
    pub evidence_refs: Vec<String>,
    pub continuation_allowed: bool,
    pub handoff_reason: FreshMemoryHandoffReason,
    pub expires_at: Option<MonotonicTimeNs>,
}

impl FreshMemoryHandoff {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        handoff_id: String,
        source_session_id: Option<SessionId>,
        source_thread_key: Option<String>,
        source_turn_ref: Option<String>,
        last_topic: Option<String>,
        last_intent: Option<String>,
        last_tool_family: Option<String>,
        last_entity_focus: Vec<String>,
        last_answer_type: Option<String>,
        freshness_label: MemoryAgeLabel,
        confidence: MemoryConfidence,
        evidence_refs: Vec<String>,
        continuation_allowed: bool,
        handoff_reason: FreshMemoryHandoffReason,
        expires_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let handoff = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            handoff_id,
            source_session_id,
            source_thread_key,
            source_turn_ref,
            last_topic,
            last_intent,
            last_tool_family,
            last_entity_focus,
            last_answer_type,
            freshness_label,
            confidence,
            evidence_refs,
            continuation_allowed,
            handoff_reason,
            expires_at,
        };
        handoff.validate()?;
        Ok(handoff)
    }
}

impl Validate for FreshMemoryHandoff {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1M_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "fresh_memory_handoff.schema_version",
                reason: "must match PH1M_CONTRACT_VERSION",
            });
        }
        if self.handoff_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "fresh_memory_handoff.handoff_id",
                reason: "must not be empty",
            });
        }
        if self.handoff_id.len() > MEMORY_CANONICAL_LABEL_MAX_CHARS {
            return Err(ContractViolation::InvalidValue {
                field: "fresh_memory_handoff.handoff_id",
                reason: "exceeds max length",
            });
        }
        validate_optional_memory_text(
            "fresh_memory_handoff.source_thread_key",
            &self.source_thread_key,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "fresh_memory_handoff.source_turn_ref",
            &self.source_turn_ref,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "fresh_memory_handoff.last_topic",
            &self.last_topic,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "fresh_memory_handoff.last_intent",
            &self.last_intent,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "fresh_memory_handoff.last_tool_family",
            &self.last_tool_family,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_memory_text_list(
            "fresh_memory_handoff.last_entity_focus",
            &self.last_entity_focus,
            MEMORY_CANONICAL_MAX_REFS,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "fresh_memory_handoff.last_answer_type",
            &self.last_answer_type,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_memory_text_list(
            "fresh_memory_handoff.evidence_refs",
            &self.evidence_refs,
            MEMORY_CANONICAL_MAX_REFS,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        if let Some(expires_at) = self.expires_at {
            if expires_at.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "fresh_memory_handoff.expires_at",
                    reason: "must be > 0 when provided",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContinuationDecision {
    pub schema_version: SchemaVersion,
    pub decision: MemoryContinuationDecisionKind,
    pub confidence: MemoryConfidence,
    pub reason_code: ReasonCodeId,
    pub evidence_packet_ref: Option<String>,
    pub user_facing_summary: Option<String>,
    pub clarification_prompt: Option<String>,
}

impl MemoryContinuationDecision {
    pub fn v1(
        decision: MemoryContinuationDecisionKind,
        confidence: MemoryConfidence,
        reason_code: ReasonCodeId,
        evidence_packet_ref: Option<String>,
        user_facing_summary: Option<String>,
        clarification_prompt: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            decision,
            confidence,
            reason_code,
            evidence_packet_ref,
            user_facing_summary,
            clarification_prompt,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for MemoryContinuationDecision {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1M_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "memory_continuation_decision.schema_version",
                reason: "must match PH1M_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_continuation_decision.reason_code",
                reason: "must be > 0",
            });
        }
        validate_optional_memory_text(
            "memory_continuation_decision.evidence_packet_ref",
            &self.evidence_packet_ref,
            MEMORY_CANONICAL_LABEL_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_continuation_decision.user_facing_summary",
            &self.user_facing_summary,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        validate_optional_memory_text(
            "memory_continuation_decision.clarification_prompt",
            &self.clarification_prompt,
            MEMORY_CANONICAL_TEXT_MAX_CHARS,
        )?;
        if matches!(
            self.decision,
            MemoryContinuationDecisionKind::AskClarification
        ) && self.clarification_prompt.is_none()
        {
            return Err(ContractViolation::InvalidValue {
                field: "memory_continuation_decision.clarification_prompt",
                reason: "must be provided when decision is AskClarification",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryExposureLevel {
    SafeToSpeak,
    SafeToText,
    InternalOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryItemTag {
    Confirmed,
    Tentative,
    Stale,
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemoryProvenanceTier {
    Verified,
    UserStated,
    ToolDerived,
    Inferred,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemorySuppressionRuleKind {
    DoNotMention,
    DoNotRepeat,
    DoNotStore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MemorySuppressionTargetType {
    ThreadId,
    WorkOrderId,
    TopicKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryGraphNodeKind {
    Entity,
    Project,
    Vendor,
    Decision,
    Thread,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryGraphEdgeKind {
    MentionedWith,
    DependsOn,
    DecidedIn,
    BlockedBy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryResumeDeliveryMode {
    Voice,
    Text,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PendingWorkStatus {
    Draft,
    Clarify,
    Confirm,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryRetentionMode {
    Default,
    RememberEverything,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryResumeTier {
    Hot,
    Warm,
    Cold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MemoryResumeAction {
    AutoLoad,
    Suggest,
    None,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryContextFact {
    pub memory_key: MemoryKey,
    pub memory_value: MemoryValue,
}

impl MemoryContextFact {
    pub fn v1(memory_key: MemoryKey, memory_value: MemoryValue) -> Result<Self, ContractViolation> {
        let f = Self {
            memory_key,
            memory_value,
        };
        f.validate()?;
        Ok(f)
    }
}

impl Validate for MemoryContextFact {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.memory_value.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryHintEntry {
    pub schema_version: SchemaVersion,
    pub key: String,
    pub value: String,
}

impl MemoryHintEntry {
    pub fn v1(key: String, value: String) -> Result<Self, ContractViolation> {
        let h = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            key,
            value,
        };
        h.validate()?;
        Ok(h)
    }
}

impl Validate for MemoryHintEntry {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_hint_entry.key",
                reason: "must not be empty",
            });
        }
        if self.key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_hint_entry.key",
                reason: "must be <= 128 chars",
            });
        }
        if self.value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_hint_entry.value",
                reason: "must not be empty",
            });
        }
        if self.value.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_hint_entry.value",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryBundleItem {
    pub schema_version: SchemaVersion,
    pub item_id: String,
    pub memory_key: MemoryKey,
    pub memory_value: MemoryValue,
    pub tag: MemoryItemTag,
    pub exposure_level: MemoryExposureLevel,
    pub confidence: MemoryConfidence,
    pub provenance_tier: MemoryProvenanceTier,
    pub pinned: bool,
    pub last_used_at: MonotonicTimeNs,
    pub use_count: u32,
}

impl MemoryBundleItem {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        item_id: String,
        memory_key: MemoryKey,
        memory_value: MemoryValue,
        tag: MemoryItemTag,
        exposure_level: MemoryExposureLevel,
        confidence: MemoryConfidence,
        provenance_tier: MemoryProvenanceTier,
        pinned: bool,
        last_used_at: MonotonicTimeNs,
        use_count: u32,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            item_id,
            memory_key,
            memory_value,
            tag,
            exposure_level,
            confidence,
            provenance_tier,
            pinned,
            last_used_at,
            use_count,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for MemoryBundleItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.memory_value.validate()?;
        if self.item_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_bundle_item.item_id",
                reason: "must not be empty",
            });
        }
        if self.item_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_bundle_item.item_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.use_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_bundle_item.use_count",
                reason: "must be <= 1000000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryArchiveExcerpt {
    pub schema_version: SchemaVersion,
    pub archive_ref_id: String,
    pub excerpt_text: String,
}

impl MemoryArchiveExcerpt {
    pub fn v1(archive_ref_id: String, excerpt_text: String) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            archive_ref_id,
            excerpt_text,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for MemoryArchiveExcerpt {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.archive_ref_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_archive_excerpt.archive_ref_id",
                reason: "must not be empty",
            });
        }
        if self.archive_ref_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_archive_excerpt.archive_ref_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.excerpt_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_archive_excerpt.excerpt_text",
                reason: "must not be empty",
            });
        }
        if self.excerpt_text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_archive_excerpt.excerpt_text",
                reason: "must be <= 512 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryRecentArchiveMatch {
    pub schema_version: SchemaVersion,
    pub archive_ref_id: String,
    pub thread_id: Option<String>,
    pub matched_at: MonotonicTimeNs,
    pub excerpt_text: String,
    pub match_reason: String,
    pub score: u16,
}

impl MemoryRecentArchiveMatch {
    pub fn v1(
        archive_ref_id: String,
        thread_id: Option<String>,
        matched_at: MonotonicTimeNs,
        excerpt_text: String,
        match_reason: String,
        score: u16,
    ) -> Result<Self, ContractViolation> {
        let m = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            archive_ref_id,
            thread_id,
            matched_at,
            excerpt_text,
            match_reason,
            score,
        };
        m.validate()?;
        Ok(m)
    }
}

impl Validate for MemoryRecentArchiveMatch {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.archive_ref_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.archive_ref_id",
                reason: "must not be empty",
            });
        }
        if self.archive_ref_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.archive_ref_id",
                reason: "must be <= 128 chars",
            });
        }
        if let Some(thread_id) = &self.thread_id {
            if thread_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_recent_archive_match.thread_id",
                    reason: "must not be empty when provided",
                });
            }
            if thread_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_recent_archive_match.thread_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.excerpt_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.excerpt_text",
                reason: "must not be empty",
            });
        }
        if self.excerpt_text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.excerpt_text",
                reason: "must be <= 512 chars",
            });
        }
        if self.match_reason.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.match_reason",
                reason: "must not be empty",
            });
        }
        if self.match_reason.len() > 192 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.match_reason",
                reason: "must be <= 192 chars",
            });
        }
        if self.score == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_recent_archive_match.score",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRecentArchiveRecallRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub query_text: String,
    pub window_ms: u64,
    pub max_matches: u8,
}

impl Ph1mRecentArchiveRecallRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        query_text: String,
        window_ms: u64,
        max_matches: u8,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            query_text,
            window_ms,
            max_matches,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRecentArchiveRecallRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.query_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_request.query_text",
                reason: "must not be empty",
            });
        }
        if self.query_text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_request.query_text",
                reason: "must be <= 512 chars",
            });
        }
        if self.window_ms == 0 || self.window_ms > MEMORY_RESUME_WARM_WINDOW_MS {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_request.window_ms",
                reason: "must be within 1..=MEMORY_RESUME_WARM_WINDOW_MS",
            });
        }
        if self.max_matches == 0 || self.max_matches > MEMORY_RECENT_ARCHIVE_RECALL_MAX_MATCHES {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_request.max_matches",
                reason: "must be within 1..=MEMORY_RECENT_ARCHIVE_RECALL_MAX_MATCHES",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRecentArchiveRecallResponse {
    pub schema_version: SchemaVersion,
    pub matches: Vec<MemoryRecentArchiveMatch>,
    pub reason_code: ReasonCodeId,
}

impl Ph1mRecentArchiveRecallResponse {
    pub fn v1(
        matches: Vec<MemoryRecentArchiveMatch>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            matches,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRecentArchiveRecallResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.matches.len() > MEMORY_RECENT_ARCHIVE_RECALL_MAX_MATCHES as usize {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_response.matches",
                reason: "must be <= MEMORY_RECENT_ARCHIVE_RECALL_MAX_MATCHES",
            });
        }
        for item in &self.matches {
            item.validate()?;
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_recent_archive_recall_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingWorkItem {
    pub schema_version: SchemaVersion,
    pub work_order_id: String,
    pub status: PendingWorkStatus,
    pub thread_id: Option<String>,
    pub summary_bullets: Vec<String>,
    pub last_updated_at: MonotonicTimeNs,
    pub use_count: u32,
}

impl PendingWorkItem {
    pub fn v1(
        work_order_id: String,
        status: PendingWorkStatus,
        thread_id: Option<String>,
        summary_bullets: Vec<String>,
        last_updated_at: MonotonicTimeNs,
        use_count: u32,
    ) -> Result<Self, ContractViolation> {
        let p = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            work_order_id,
            status,
            thread_id,
            summary_bullets,
            last_updated_at,
            use_count,
        };
        p.validate()?;
        Ok(p)
    }
}

impl Validate for PendingWorkItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.work_order_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "pending_work_item.work_order_id",
                reason: "must not be empty",
            });
        }
        if self.work_order_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "pending_work_item.work_order_id",
                reason: "must be <= 128 chars",
            });
        }
        if let Some(thread_id) = &self.thread_id {
            if thread_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_work_item.thread_id",
                    reason: "must not be empty when provided",
                });
            }
            if thread_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_work_item.thread_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.summary_bullets.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "pending_work_item.summary_bullets",
                reason: "must contain <= 3 entries",
            });
        }
        for bullet in &self.summary_bullets {
            if bullet.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_work_item.summary_bullets[]",
                    reason: "must not contain empty entries",
                });
            }
            if bullet.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_work_item.summary_bullets[]",
                    reason: "must be <= 256 chars",
                });
            }
        }
        if self.use_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "pending_work_item.use_count",
                reason: "must be <= 1000000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySuppressionRule {
    pub schema_version: SchemaVersion,
    pub target_type: MemorySuppressionTargetType,
    pub target_id: String,
    pub rule_kind: MemorySuppressionRuleKind,
    pub active: bool,
    pub reason_code: ReasonCodeId,
    pub updated_at: MonotonicTimeNs,
}

impl MemorySuppressionRule {
    pub fn v1(
        target_type: MemorySuppressionTargetType,
        target_id: String,
        rule_kind: MemorySuppressionRuleKind,
        active: bool,
        reason_code: ReasonCodeId,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            target_type,
            target_id,
            rule_kind,
            active,
            reason_code,
            updated_at,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for MemorySuppressionRule {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.target_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_suppression_rule.target_id",
                reason: "must not be empty",
            });
        }
        if self.target_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_suppression_rule.target_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_suppression_rule.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemorySafeSummaryItem {
    pub schema_version: SchemaVersion,
    pub memory_key: MemoryKey,
    pub summary_text: String,
    pub exposure_level: MemoryExposureLevel,
}

impl MemorySafeSummaryItem {
    pub fn v1(
        memory_key: MemoryKey,
        summary_text: String,
        exposure_level: MemoryExposureLevel,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_key,
            summary_text,
            exposure_level,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for MemorySafeSummaryItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.summary_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_safe_summary_item.summary_text",
                reason: "must not be empty",
            });
        }
        if self.summary_text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_safe_summary_item.summary_text",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryEmotionalThreadState {
    pub schema_version: SchemaVersion,
    pub thread_key: String,
    pub tone_tags: Vec<String>,
    pub summary: Option<String>,
    pub updated_at: MonotonicTimeNs,
}

impl MemoryEmotionalThreadState {
    pub fn v1(
        thread_key: String,
        tone_tags: Vec<String>,
        summary: Option<String>,
        updated_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            thread_key,
            tone_tags,
            summary,
            updated_at,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for MemoryEmotionalThreadState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.thread_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_emotional_thread_state.thread_key",
                reason: "must not be empty",
            });
        }
        if self.thread_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_emotional_thread_state.thread_key",
                reason: "must be <= 128 chars",
            });
        }
        if self.tone_tags.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_emotional_thread_state.tone_tags",
                reason: "must contain <= 8 entries",
            });
        }
        for tag in &self.tone_tags {
            if tag.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_emotional_thread_state.tone_tags[]",
                    reason: "must not contain empty strings",
                });
            }
            if tag.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_emotional_thread_state.tone_tags[]",
                    reason: "must be <= 64 chars",
                });
            }
        }
        if let Some(summary) = &self.summary {
            if summary.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_emotional_thread_state.summary",
                    reason: "must not be empty when provided",
                });
            }
            if summary.len() > 512 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_emotional_thread_state.summary",
                    reason: "must be <= 512 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryMetricPayload {
    pub schema_version: SchemaVersion,
    pub context_bundle_bytes: u32,
    pub atoms_selected_count: u8,
    pub excerpts_selected_count: u8,
    pub confirmed_count: u8,
    pub tentative_count: u8,
    pub stale_count: u8,
    pub conflict_count: u8,
    pub conflict_trigger_count: u16,
    pub clarification_due_to_memory_count: u16,
    pub do_not_mention_hits_count: u16,
}

impl MemoryMetricPayload {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        context_bundle_bytes: u32,
        atoms_selected_count: u8,
        excerpts_selected_count: u8,
        confirmed_count: u8,
        tentative_count: u8,
        stale_count: u8,
        conflict_count: u8,
        conflict_trigger_count: u16,
        clarification_due_to_memory_count: u16,
        do_not_mention_hits_count: u16,
    ) -> Result<Self, ContractViolation> {
        let m = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            context_bundle_bytes,
            atoms_selected_count,
            excerpts_selected_count,
            confirmed_count,
            tentative_count,
            stale_count,
            conflict_count,
            conflict_trigger_count,
            clarification_due_to_memory_count,
            do_not_mention_hits_count,
        };
        m.validate()?;
        Ok(m)
    }
}

impl Validate for MemoryMetricPayload {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.context_bundle_bytes > MEMORY_CONTEXT_BUNDLE_MAX_BYTES {
            return Err(ContractViolation::InvalidValue {
                field: "memory_metric_payload.context_bundle_bytes",
                reason: "must be <= MEMORY_CONTEXT_BUNDLE_MAX_BYTES",
            });
        }
        if self.atoms_selected_count > MEMORY_CONTEXT_BUNDLE_MAX_ATOMS {
            return Err(ContractViolation::InvalidValue {
                field: "memory_metric_payload.atoms_selected_count",
                reason: "must be <= MEMORY_CONTEXT_BUNDLE_MAX_ATOMS",
            });
        }
        if self.excerpts_selected_count > MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS {
            return Err(ContractViolation::InvalidValue {
                field: "memory_metric_payload.excerpts_selected_count",
                reason: "must be <= MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryGraphNodeInput {
    pub schema_version: SchemaVersion,
    pub node_id: String,
    pub kind: MemoryGraphNodeKind,
    pub confidence: MemoryConfidence,
    pub last_used_at: MonotonicTimeNs,
    pub use_count: u32,
}

impl MemoryGraphNodeInput {
    pub fn v1(
        node_id: String,
        kind: MemoryGraphNodeKind,
        confidence: MemoryConfidence,
        last_used_at: MonotonicTimeNs,
        use_count: u32,
    ) -> Result<Self, ContractViolation> {
        let n = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            node_id,
            kind,
            confidence,
            last_used_at,
            use_count,
        };
        n.validate()?;
        Ok(n)
    }
}

impl Validate for MemoryGraphNodeInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.node_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_node_input.node_id",
                reason: "must not be empty",
            });
        }
        if self.node_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_node_input.node_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.use_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_node_input.use_count",
                reason: "must be <= 1000000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryGraphEdgeInput {
    pub schema_version: SchemaVersion,
    pub edge_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub kind: MemoryGraphEdgeKind,
    pub confidence: MemoryConfidence,
    pub last_used_at: MonotonicTimeNs,
    pub use_count: u32,
}

impl MemoryGraphEdgeInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        edge_id: String,
        from_node_id: String,
        to_node_id: String,
        kind: MemoryGraphEdgeKind,
        confidence: MemoryConfidence,
        last_used_at: MonotonicTimeNs,
        use_count: u32,
    ) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            edge_id,
            from_node_id,
            to_node_id,
            kind,
            confidence,
            last_used_at,
            use_count,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for MemoryGraphEdgeInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.edge_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_edge_input.edge_id",
                reason: "must not be empty",
            });
        }
        if self.edge_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_edge_input.edge_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.from_node_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_edge_input.from_node_id",
                reason: "must not be empty",
            });
        }
        if self.to_node_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_edge_input.to_node_id",
                reason: "must not be empty",
            });
        }
        if self.use_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_graph_edge_input.use_count",
                reason: "must be <= 1000000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryThreadDigest {
    pub schema_version: SchemaVersion,
    pub thread_id: String,
    pub thread_title: String,
    pub summary_bullets: Vec<String>,
    pub pinned: bool,
    pub unresolved: bool,
    pub last_updated_at: MonotonicTimeNs,
    pub use_count: u32,
}

impl MemoryThreadDigest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        thread_id: String,
        thread_title: String,
        summary_bullets: Vec<String>,
        pinned: bool,
        unresolved: bool,
        last_updated_at: MonotonicTimeNs,
        use_count: u32,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            thread_id,
            thread_title,
            summary_bullets,
            pinned,
            unresolved,
            last_updated_at,
            use_count,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for MemoryThreadDigest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.thread_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.thread_id",
                reason: "must not be empty",
            });
        }
        if self.thread_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.thread_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.thread_title.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.thread_title",
                reason: "must not be empty",
            });
        }
        if self.thread_title.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.thread_title",
                reason: "must be <= 256 chars",
            });
        }
        if self.summary_bullets.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.summary_bullets",
                reason: "must contain 1..=3 entries",
            });
        }
        if self.summary_bullets.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.summary_bullets",
                reason: "must contain <= 3 entries",
            });
        }
        for bullet in &self.summary_bullets {
            if bullet.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_thread_digest.summary_bullets[]",
                    reason: "must not contain empty strings",
                });
            }
            if bullet.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "memory_thread_digest.summary_bullets[]",
                    reason: "must be <= 256 chars",
                });
            }
        }
        if self.use_count > 1_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "memory_thread_digest.use_count",
                reason: "must be <= 1000000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mThreadDigestUpsertRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub memory_retention_mode: MemoryRetentionMode,
    pub thread_digest: MemoryThreadDigest,
    pub idempotency_key: String,
}

impl Ph1mThreadDigestUpsertRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        memory_retention_mode: MemoryRetentionMode,
        thread_digest: MemoryThreadDigest,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            memory_retention_mode,
            thread_digest,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mThreadDigestUpsertRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.thread_digest.validate()?;
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_thread_digest_upsert_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_thread_digest_upsert_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mThreadDigestUpsertResponse {
    pub schema_version: SchemaVersion,
    pub stored: bool,
    pub thread_id: String,
    pub reason_code: ReasonCodeId,
}

impl Ph1mThreadDigestUpsertResponse {
    pub fn v1(
        stored: bool,
        thread_id: String,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            stored,
            thread_id,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mThreadDigestUpsertResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.thread_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_thread_digest_upsert_response.thread_id",
                reason: "must not be empty",
            });
        }
        if self.thread_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_thread_digest_upsert_response.thread_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_thread_digest_upsert_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mResumeSelectRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub memory_retention_mode: MemoryRetentionMode,
    pub allow_auto_resume: bool,
    pub allow_suggest: bool,
    pub voice_delivery_allowed: bool,
    pub allow_text_delivery: bool,
    pub allow_pending_work_resume: bool,
    pub auto_resume_disabled_by_user: bool,
    pub max_summary_bullets: u8,
    pub topic_hint: Option<String>,
    pub pending_work_orders: Vec<PendingWorkItem>,
    pub suppressed_thread_ids: Vec<String>,
    pub suppressed_work_order_ids: Vec<String>,
}

impl Ph1mResumeSelectRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        memory_retention_mode: MemoryRetentionMode,
        allow_auto_resume: bool,
        allow_suggest: bool,
        voice_delivery_allowed: bool,
        auto_resume_disabled_by_user: bool,
        max_summary_bullets: u8,
        topic_hint: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            memory_retention_mode,
            allow_auto_resume,
            allow_suggest,
            voice_delivery_allowed,
            allow_text_delivery: true,
            allow_pending_work_resume: true,
            auto_resume_disabled_by_user,
            max_summary_bullets,
            topic_hint,
            pending_work_orders: vec![],
            suppressed_thread_ids: vec![],
            suppressed_work_order_ids: vec![],
        };
        r.validate()?;
        Ok(r)
    }

    pub fn with_pending_work_context(
        mut self,
        pending_work_orders: Vec<PendingWorkItem>,
        suppressed_thread_ids: Vec<String>,
        suppressed_work_order_ids: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        self.pending_work_orders = pending_work_orders;
        self.suppressed_thread_ids = suppressed_thread_ids;
        self.suppressed_work_order_ids = suppressed_work_order_ids;
        self.validate()?;
        Ok(self)
    }

    pub fn with_text_delivery(
        mut self,
        allow_text_delivery: bool,
    ) -> Result<Self, ContractViolation> {
        self.allow_text_delivery = allow_text_delivery;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for Ph1mResumeSelectRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.max_summary_bullets == 0 || self.max_summary_bullets > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.max_summary_bullets",
                reason: "must be within 1..=3",
            });
        }
        if !self.allow_auto_resume && !self.allow_suggest {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.allow_auto_resume",
                reason: "allow_auto_resume and allow_suggest cannot both be false",
            });
        }
        if !self.voice_delivery_allowed && !self.allow_text_delivery {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.voice_delivery_allowed",
                reason: "voice_delivery_allowed and allow_text_delivery cannot both be false",
            });
        }
        if let Some(topic_hint) = &self.topic_hint {
            if topic_hint.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.topic_hint",
                    reason: "must not be empty when provided",
                });
            }
            if topic_hint.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.topic_hint",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.pending_work_orders.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.pending_work_orders",
                reason: "must contain <= 16 entries",
            });
        }
        for pending in &self.pending_work_orders {
            pending.validate()?;
        }
        if self.suppressed_thread_ids.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.suppressed_thread_ids",
                reason: "must contain <= 64 entries",
            });
        }
        for thread_id in &self.suppressed_thread_ids {
            if thread_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.suppressed_thread_ids[]",
                    reason: "must not contain empty strings",
                });
            }
            if thread_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.suppressed_thread_ids[]",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.suppressed_work_order_ids.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_request.suppressed_work_order_ids",
                reason: "must contain <= 64 entries",
            });
        }
        for work_order_id in &self.suppressed_work_order_ids {
            if work_order_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.suppressed_work_order_ids[]",
                    reason: "must not contain empty strings",
                });
            }
            if work_order_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_request.suppressed_work_order_ids[]",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mResumeSelectResponse {
    pub schema_version: SchemaVersion,
    pub selected_thread_id: Option<String>,
    pub selected_thread_title: Option<String>,
    pub pending_work_order_id: Option<String>,
    pub resume_tier: Option<MemoryResumeTier>,
    pub resume_action: MemoryResumeAction,
    pub resume_delivery_mode: MemoryResumeDeliveryMode,
    pub resume_summary_bullets: Vec<String>,
    pub reason_code: ReasonCodeId,
}

impl Ph1mResumeSelectResponse {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        selected_thread_id: Option<String>,
        selected_thread_title: Option<String>,
        resume_tier: Option<MemoryResumeTier>,
        resume_action: MemoryResumeAction,
        resume_summary_bullets: Vec<String>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let resume_delivery_mode = if matches!(resume_action, MemoryResumeAction::None) {
            MemoryResumeDeliveryMode::None
        } else {
            MemoryResumeDeliveryMode::Voice
        };
        Self::v1_with_delivery(
            selected_thread_id,
            selected_thread_title,
            None,
            resume_tier,
            resume_action,
            resume_delivery_mode,
            resume_summary_bullets,
            reason_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v1_with_delivery(
        selected_thread_id: Option<String>,
        selected_thread_title: Option<String>,
        pending_work_order_id: Option<String>,
        resume_tier: Option<MemoryResumeTier>,
        resume_action: MemoryResumeAction,
        resume_delivery_mode: MemoryResumeDeliveryMode,
        resume_summary_bullets: Vec<String>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            selected_thread_id,
            selected_thread_title,
            pending_work_order_id,
            resume_tier,
            resume_action,
            resume_delivery_mode,
            resume_summary_bullets,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mResumeSelectResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.resume_summary_bullets.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_response.resume_summary_bullets",
                reason: "must contain <= 3 entries",
            });
        }
        for bullet in &self.resume_summary_bullets {
            if bullet.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_response.resume_summary_bullets[]",
                    reason: "must not contain empty strings",
                });
            }
            if bullet.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_response.resume_summary_bullets[]",
                    reason: "must be <= 256 chars",
                });
            }
        }
        match self.resume_action {
            MemoryResumeAction::AutoLoad | MemoryResumeAction::Suggest => {
                if self.selected_thread_id.is_none()
                    || self.selected_thread_title.is_none()
                    || self.resume_tier.is_none()
                {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1m_resume_select_response.selected_thread_id",
                        reason: "must be present when resume_action is AutoLoad or Suggest",
                    });
                }
            }
            MemoryResumeAction::None => {
                if self.resume_delivery_mode != MemoryResumeDeliveryMode::None {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1m_resume_select_response.resume_delivery_mode",
                        reason: "must be None when resume_action is None",
                    });
                }
            }
        }
        if let Some(work_order_id) = &self.pending_work_order_id {
            if work_order_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_response.pending_work_order_id",
                    reason: "must not be empty when provided",
                });
            }
            if work_order_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_resume_select_response.pending_work_order_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_resume_select_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mHintBundleBuildRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub max_hints: u8,
}

impl Ph1mHintBundleBuildRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        max_hints: u8,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            max_hints,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mHintBundleBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.max_hints == 0 || self.max_hints > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_hint_bundle_build_request.max_hints",
                reason: "must be within 1..=32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mHintBundleBuildResponse {
    pub schema_version: SchemaVersion,
    pub hints: Vec<MemoryHintEntry>,
    pub reason_code: ReasonCodeId,
}

impl Ph1mHintBundleBuildResponse {
    pub fn v1(
        hints: Vec<MemoryHintEntry>,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            hints,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mHintBundleBuildResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.hints.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_hint_bundle_build_response.hints",
                reason: "must be <= 32 entries",
            });
        }
        for hint in &self.hints {
            hint.validate()?;
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_hint_bundle_build_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mContextBundleBuildRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub requested_keys: Vec<MemoryKey>,
    pub current_state_facts: Vec<MemoryContextFact>,
    pub topic_hint: Option<String>,
    pub thread_id: Option<String>,
    pub work_order_id: Option<String>,
    pub allow_sensitive: bool,
    pub max_bundle_bytes: u32,
    pub max_atoms: u8,
    pub max_excerpts: u8,
}

impl Ph1mContextBundleBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        requested_keys: Vec<MemoryKey>,
        current_state_facts: Vec<MemoryContextFact>,
        topic_hint: Option<String>,
        thread_id: Option<String>,
        work_order_id: Option<String>,
        allow_sensitive: bool,
        max_bundle_bytes: u32,
        max_atoms: u8,
        max_excerpts: u8,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            requested_keys,
            current_state_facts,
            topic_hint,
            thread_id,
            work_order_id,
            allow_sensitive,
            max_bundle_bytes,
            max_atoms,
            max_excerpts,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mContextBundleBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.requested_keys.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_request.requested_keys",
                reason: "must contain <= 64 entries",
            });
        }
        if self.current_state_facts.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_request.current_state_facts",
                reason: "must contain <= 64 entries",
            });
        }
        for fact in &self.current_state_facts {
            fact.validate()?;
        }
        if let Some(topic_hint) = &self.topic_hint {
            if topic_hint.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.topic_hint",
                    reason: "must not be empty when provided",
                });
            }
            if topic_hint.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.topic_hint",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if let Some(thread_id) = &self.thread_id {
            if thread_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.thread_id",
                    reason: "must not be empty when provided",
                });
            }
            if thread_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.thread_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if let Some(work_order_id) = &self.work_order_id {
            if work_order_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.work_order_id",
                    reason: "must not be empty when provided",
                });
            }
            if work_order_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1m_context_bundle_build_request.work_order_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.max_bundle_bytes == 0 || self.max_bundle_bytes > MEMORY_CONTEXT_BUNDLE_MAX_BYTES {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_request.max_bundle_bytes",
                reason: "must be within 1..=MEMORY_CONTEXT_BUNDLE_MAX_BYTES",
            });
        }
        if self.max_atoms == 0 || self.max_atoms > MEMORY_CONTEXT_BUNDLE_MAX_ATOMS {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_request.max_atoms",
                reason: "must be within 1..=MEMORY_CONTEXT_BUNDLE_MAX_ATOMS",
            });
        }
        if self.max_excerpts > MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_request.max_excerpts",
                reason: "must be <= MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mContextBundleBuildResponse {
    pub schema_version: SchemaVersion,
    pub push_items: Vec<MemoryBundleItem>,
    pub pull_items: Vec<MemoryBundleItem>,
    pub archive_excerpts: Vec<MemoryArchiveExcerpt>,
    pub metric_payload: MemoryMetricPayload,
    pub reason_code: ReasonCodeId,
}

impl Ph1mContextBundleBuildResponse {
    pub fn v1(
        push_items: Vec<MemoryBundleItem>,
        pull_items: Vec<MemoryBundleItem>,
        archive_excerpts: Vec<MemoryArchiveExcerpt>,
        metric_payload: MemoryMetricPayload,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            push_items,
            pull_items,
            archive_excerpts,
            metric_payload,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mContextBundleBuildResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.push_items.len() > MEMORY_CONTEXT_BUNDLE_MAX_ATOMS as usize {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_response.push_items",
                reason: "must contain <= MEMORY_CONTEXT_BUNDLE_MAX_ATOMS entries",
            });
        }
        if self.pull_items.len() > MEMORY_CONTEXT_BUNDLE_MAX_ATOMS as usize {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_response.pull_items",
                reason: "must contain <= MEMORY_CONTEXT_BUNDLE_MAX_ATOMS entries",
            });
        }
        if self.archive_excerpts.len() > MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS as usize {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_response.archive_excerpts",
                reason: "must contain <= MEMORY_CONTEXT_BUNDLE_MAX_EXCERPTS entries",
            });
        }
        for item in &self.push_items {
            item.validate()?;
        }
        for item in &self.pull_items {
            item.validate()?;
        }
        for excerpt in &self.archive_excerpts {
            excerpt.validate()?;
        }
        self.metric_payload.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_context_bundle_build_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mSuppressionSetRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub rule: MemorySuppressionRule,
    pub idempotency_key: String,
}

impl Ph1mSuppressionSetRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        rule: MemorySuppressionRule,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            rule,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mSuppressionSetRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.rule.validate()?;
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_suppression_set_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_suppression_set_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mSuppressionSetResponse {
    pub schema_version: SchemaVersion,
    pub applied: bool,
    pub rule: MemorySuppressionRule,
    pub reason_code: ReasonCodeId,
}

impl Ph1mSuppressionSetResponse {
    pub fn v1(
        applied: bool,
        rule: MemorySuppressionRule,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            applied,
            rule,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mSuppressionSetResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.rule.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_suppression_set_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mSafeSummaryRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub max_items: u8,
    pub max_bytes: u16,
}

impl Ph1mSafeSummaryRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        max_items: u8,
        max_bytes: u16,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            max_items,
            max_bytes,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mSafeSummaryRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.max_items == 0 || self.max_items > MEMORY_SAFE_SUMMARY_MAX_ITEMS {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_safe_summary_request.max_items",
                reason: "must be within 1..=MEMORY_SAFE_SUMMARY_MAX_ITEMS",
            });
        }
        if self.max_bytes == 0 || self.max_bytes > MEMORY_SAFE_SUMMARY_MAX_BYTES {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_safe_summary_request.max_bytes",
                reason: "must be within 1..=MEMORY_SAFE_SUMMARY_MAX_BYTES",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mSafeSummaryResponse {
    pub schema_version: SchemaVersion,
    pub summary_items: Vec<MemorySafeSummaryItem>,
    pub output_bytes: u16,
    pub reason_code: ReasonCodeId,
}

impl Ph1mSafeSummaryResponse {
    pub fn v1(
        summary_items: Vec<MemorySafeSummaryItem>,
        output_bytes: u16,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            summary_items,
            output_bytes,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mSafeSummaryResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.summary_items.len() > MEMORY_SAFE_SUMMARY_MAX_ITEMS as usize {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_safe_summary_response.summary_items",
                reason: "must contain <= MEMORY_SAFE_SUMMARY_MAX_ITEMS entries",
            });
        }
        for item in &self.summary_items {
            item.validate()?;
        }
        if self.output_bytes > MEMORY_SAFE_SUMMARY_MAX_BYTES {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_safe_summary_response.output_bytes",
                reason: "must be <= MEMORY_SAFE_SUMMARY_MAX_BYTES",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_safe_summary_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mEmotionalThreadUpdateRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub thread_state: MemoryEmotionalThreadState,
    pub idempotency_key: String,
}

impl Ph1mEmotionalThreadUpdateRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        thread_state: MemoryEmotionalThreadState,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            thread_state,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mEmotionalThreadUpdateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.thread_state.validate()?;
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_emotional_thread_update_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_emotional_thread_update_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mEmotionalThreadUpdateResponse {
    pub schema_version: SchemaVersion,
    pub state: MemoryEmotionalThreadState,
    pub reason_code: ReasonCodeId,
}

impl Ph1mEmotionalThreadUpdateResponse {
    pub fn v1(
        state: MemoryEmotionalThreadState,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            state,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mEmotionalThreadUpdateResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.state.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_emotional_thread_update_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mMetricsEmitRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub payload: MemoryMetricPayload,
    pub idempotency_key: String,
}

impl Ph1mMetricsEmitRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        payload: MemoryMetricPayload,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            payload,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mMetricsEmitRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.payload.validate()?;
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_metrics_emit_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_metrics_emit_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mMetricsEmitResponse {
    pub schema_version: SchemaVersion,
    pub emitted: bool,
    pub reason_code: ReasonCodeId,
}

impl Ph1mMetricsEmitResponse {
    pub fn v1(emitted: bool, reason_code: ReasonCodeId) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            emitted,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mMetricsEmitResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_metrics_emit_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mGraphUpdateRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub nodes: Vec<MemoryGraphNodeInput>,
    pub edges: Vec<MemoryGraphEdgeInput>,
    pub idempotency_key: String,
}

impl Ph1mGraphUpdateRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        nodes: Vec<MemoryGraphNodeInput>,
        edges: Vec<MemoryGraphEdgeInput>,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            nodes,
            edges,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mGraphUpdateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.nodes.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_graph_update_request.nodes",
                reason: "must contain <= 128 entries",
            });
        }
        if self.edges.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_graph_update_request.edges",
                reason: "must contain <= 256 entries",
            });
        }
        for node in &self.nodes {
            node.validate()?;
        }
        for edge in &self.edges {
            edge.validate()?;
        }
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_graph_update_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_graph_update_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mGraphUpdateResponse {
    pub schema_version: SchemaVersion,
    pub graph_update_count: u16,
    pub reason_code: ReasonCodeId,
}

impl Ph1mGraphUpdateResponse {
    pub fn v1(
        graph_update_count: u16,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            graph_update_count,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mGraphUpdateResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_graph_update_response.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRetentionModeSetRequest {
    pub schema_version: SchemaVersion,
    pub now: MonotonicTimeNs,
    pub speaker_assertion: Ph1VoiceIdResponse,
    pub policy_context_ref: PolicyContextRef,
    pub memory_retention_mode: MemoryRetentionMode,
    pub idempotency_key: String,
}

impl Ph1mRetentionModeSetRequest {
    pub fn v1(
        now: MonotonicTimeNs,
        speaker_assertion: Ph1VoiceIdResponse,
        policy_context_ref: PolicyContextRef,
        memory_retention_mode: MemoryRetentionMode,
        idempotency_key: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            now,
            speaker_assertion,
            policy_context_ref,
            memory_retention_mode,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRetentionModeSetRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.idempotency_key.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_retention_mode_set_request.idempotency_key",
                reason: "must not be empty",
            });
        }
        if self.idempotency_key.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_retention_mode_set_request.idempotency_key",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1mRetentionModeSetResponse {
    pub schema_version: SchemaVersion,
    pub memory_retention_mode: MemoryRetentionMode,
    pub effective_at: MonotonicTimeNs,
    pub reason_code: ReasonCodeId,
}

impl Ph1mRetentionModeSetResponse {
    pub fn v1(
        memory_retention_mode: MemoryRetentionMode,
        effective_at: MonotonicTimeNs,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1M_CONTRACT_VERSION,
            memory_retention_mode,
            effective_at,
            reason_code,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1mRetentionModeSetResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1m_retention_mode_set_response.reason_code",
                reason: "must be > 0",
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

    fn base_recall_request() -> Ph1mRecallRequest {
        Ph1mRecallRequest::v1(
            MonotonicTimeNs(10),
            ok_speaker(),
            policy_ok(),
            vec![MemoryKey::new("synthetic_topic").unwrap()],
            false,
            4,
        )
        .unwrap()
    }

    #[test]
    fn ph1m_canonical_memory_evidence_packet_carries_memory_styles() {
        for (memory_type, age_label) in [
            (MemoryEvidenceType::Fresh, MemoryAgeLabel::JustNow),
            (MemoryEvidenceType::Today, MemoryAgeLabel::EarlierToday),
            (MemoryEvidenceType::Topic, MemoryAgeLabel::OlderTopic),
            (MemoryEvidenceType::Deep, MemoryAgeLabel::RecentWeeks),
            (MemoryEvidenceType::Permanent, MemoryAgeLabel::Permanent),
        ] {
            let packet = MemoryEvidencePacket::v1(
                memory_type,
                Some("synthetic_topic".to_string()),
                age_label,
                MemoryConfidence::High,
                vec!["evidence_ref_1".to_string()],
                true,
                false,
                Some("Synthetic user-facing summary.".to_string()),
                true,
                MemoryRecallStyle::IRemember,
                MemoryTrustLevel::InferredSummary,
                MemoryPrivacyStatus::Allowed,
                MemoryConflictStatus::Current,
            )
            .unwrap();
            assert_eq!(packet.topic_label.as_deref(), Some("synthetic_topic"));
            assert!(packet.continuation_allowed);
        }
    }

    #[test]
    fn ph1m_canonical_memory_recall_request_wraps_existing_ph1m_recall_request() {
        let base = base_recall_request();
        let canonical = MemoryRecallRequest::v1(
            base.clone(),
            Some("continue synthetic topic".to_string()),
            Some("active_context_ref_1".to_string()),
            Some("fresh".to_string()),
            Some(MemoryAgeLabel::BeforeSleep),
            Some("synthetic_topic".to_string()),
            Some("speaker_scope_1".to_string()),
            MemoryPrivacyStatus::Allowed,
            Some("fresh_memory_continuation".to_string()),
        )
        .unwrap();

        assert_eq!(canonical.as_ph1m_recall_request(), &base);
        assert_eq!(
            canonical.current_ph1x_context_ref.as_deref(),
            Some("active_context_ref_1")
        );

        MemoryRecallRequest::from_ph1m_recall_request(base).unwrap();
    }

    #[test]
    fn ph1m_canonical_fresh_memory_handoff_represents_post_sleep_followup_evidence() {
        let handoff = FreshMemoryHandoff::v1(
            "handoff_1".to_string(),
            Some(SessionId(42)),
            Some("thread_alpha".to_string()),
            Some("turn_7".to_string()),
            Some("synthetic_time_lookup".to_string()),
            Some("answer_current_value_for_place".to_string()),
            Some("time_lookup".to_string()),
            vec!["city_alpha".to_string()],
            Some("direct_answer".to_string()),
            MemoryAgeLabel::BeforeSleep,
            MemoryConfidence::High,
            vec!["turn_ref_7".to_string()],
            true,
            FreshMemoryHandoffReason::SessionSleep,
            Some(MonotonicTimeNs(30_000_000_000)),
        )
        .unwrap();

        assert_eq!(handoff.last_tool_family.as_deref(), Some("time_lookup"));
        assert!(handoff.continuation_allowed);
    }

    #[test]
    fn ph1m_canonical_memory_continuation_decision_variants_validate() {
        let continue_decision = MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::ContinueAutomatically,
            MemoryConfidence::High,
            ReasonCodeId(1),
            Some("memory_packet_ref_1".to_string()),
            Some("Continue the synthetic topic.".to_string()),
            None,
        )
        .unwrap();
        assert_eq!(
            continue_decision.decision,
            MemoryContinuationDecisionKind::ContinueAutomatically
        );

        MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::AskClarification,
            MemoryConfidence::Med,
            ReasonCodeId(2),
            Some("memory_packet_ref_2".to_string()),
            None,
            Some("Which synthetic topic do you mean?".to_string()),
        )
        .unwrap();

        MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::AnswerNormally,
            MemoryConfidence::Low,
            ReasonCodeId(3),
            None,
            None,
            None,
        )
        .unwrap();

        MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::NoMemoryMatch,
            MemoryConfidence::Low,
            ReasonCodeId(4),
            None,
            None,
            None,
        )
        .unwrap();

        assert!(MemoryContinuationDecision::v1(
            MemoryContinuationDecisionKind::BlockedByLowConfidence,
            MemoryConfidence::Low,
            ReasonCodeId(5),
            None,
            None,
            None,
        )
        .is_ok());
    }
}
