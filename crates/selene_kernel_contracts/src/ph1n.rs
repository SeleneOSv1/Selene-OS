#![forbid(unsafe_code)]

use crate::ph1c::{SessionStateRef, TranscriptOk};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1N_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OverallConfidence {
    High,
    Med,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntentType {
    CreateCalendarEvent,
    SetReminder,
    BookTable,
    SendMoney,
    /// Memory control: explicit "remember this" request.
    MemoryRememberRequest,
    /// Memory control: explicit "forget this" request.
    MemoryForgetRequest,
    /// Memory control: explicit recall/query request.
    MemoryQuery,
    /// Governance/control: generate an onboarding/invite link (simulation-gated via PH1.LINK.001).
    CreateInviteLink,
    /// Governance/control: manage capability-request lifecycle (simulation-gated via PH1.CAPREQ).
    CapreqManage,
    /// Governance/control: manage access profile schema lifecycle (simulation-gated via PH1.ACCESS.001_PH2.ACCESS.002).
    AccessSchemaManage,
    /// Governance/control: cast/resolve escalation votes (simulation-gated via PH1.ACCESS.001_PH2.ACCESS.002).
    AccessEscalationVote,
    /// Governance/control: compile/refresh per-user access instance lineage (simulation-gated via PH1.ACCESS.001_PH2.ACCESS.002).
    AccessInstanceCompileRefresh,
    TimeQuery,
    WeatherQuery,
    WebSearchQuery,
    NewsQuery,
    UrlFetchAndCiteQuery,
    DocumentUnderstandQuery,
    PhotoUnderstandQuery,
    /// Conversation-control: resume an interrupted answer (Resume Buffer).
    Continue,
    /// Conversation-control: a clear follow-up that attaches to the interrupted answer (Combine).
    MoreDetail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldKey {
    When,
    Task,
    Person,
    Place,
    PartySize,
    Amount,
    Recipient,
    /// Link/onboarding: invitee type (company/customer/employee/family_member/friend/associate).
    InviteeType,
    /// Link/onboarding: delivery method (sms/email/etc).
    DeliveryMethod,
    /// Link/onboarding: recipient contact (phone/email/handle).
    RecipientContact,
    /// Link/onboarding: tenant scope identifier (company/tenant).
    TenantId,
    /// CAPREQ: requested capability identifier (for example: `position.activate`).
    RequestedCapabilityId,
    /// CAPREQ: target scope reference (for example: `store_17`, `team.finance`).
    TargetScopeRef,
    /// CAPREQ: human-entered justification text (bounded and evidence-backed).
    Justification,
    /// CAPREQ: lifecycle action (`create_draft|submit_for_approval|approve|reject|fulfill|cancel`).
    CapreqAction,
    /// CAPREQ: existing request identifier for non-create lifecycle actions.
    CapreqId,
    /// ACCESS schema: profile identifier (for example: `AP_DRIVER`).
    AccessProfileId,
    /// ACCESS schema: lifecycle version reference (for example: `v3`).
    SchemaVersionId,
    /// ACCESS schema: scope selector (`GLOBAL|TENANT`).
    ApScope,
    /// ACCESS schema: lifecycle action (`CREATE_DRAFT|UPDATE|ACTIVATE|RETIRE`).
    ApAction,
    /// ACCESS schema: bounded payload descriptor reference.
    ProfilePayloadJson,
    /// ACCESS schema authoring review channel (`PHONE_DESKTOP | READ_OUT_LOUD`).
    AccessReviewChannel,
    /// ACCESS schema authoring per-rule action (`AGREE | DISAGREE | EDIT | DELETE | DISABLE | ADD_CUSTOM_RULE`).
    AccessRuleAction,
    /// ACCESS voting: escalation case identifier.
    EscalationCaseId,
    /// ACCESS voting: board policy identifier.
    BoardPolicyId,
    /// ACCESS voting/compile: target user identifier.
    TargetUserId,
    /// ACCESS voting: target access instance identifier.
    AccessInstanceId,
    /// ACCESS voting: vote action (`CAST_VOTE|RESOLVE`).
    VoteAction,
    /// ACCESS voting: vote value (`APPROVE|REJECT`).
    VoteValue,
    /// ACCESS voting: override result (`ONE_SHOT|TEMPORARY|TIME_WINDOW|PERMANENT|DENY`).
    OverrideResult,
    /// ACCESS compile: optional position identifier.
    PositionId,
    /// ACCESS compile: optional overlay set reference.
    OverlayIdList,
    /// ACCESS compile: deterministic compile reason code.
    CompileReason,
    /// Used when the user expressed multiple actionable intents in one turn.
    IntentChoice,
    /// Used when the user said "that/it/there/this" without a confirmed referent.
    ReferenceTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensitivityLevel {
    Public,
    Private,
    Confidential,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AmbiguityFlag {
    ReferenceAmbiguous,
    RecipientAmbiguous,
    DateAmbiguous,
    AmountAmbiguous,
    MultiIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoutingHint {
    OnboardingStart,
    OnboardingConfirmIdentity,
    OnboardingComplete,
    OnboardingLanguageDetect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TranscriptHash(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TimeExpressionKind {
    DateKeyword,
    DateTimeLocal,
    PartOfDay,
    Duration,
    RangeLocal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeExpression {
    pub kind: TimeExpressionKind,
    /// Deterministic, bounded representation (may be relative like "tomorrow 15:00").
    pub normalized: String,
}

impl Validate for TimeExpression {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.normalized.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "time_expression.normalized",
                reason: "must not be empty",
            });
        }
        if self.normalized.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "time_expression.normalized",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldValue {
    /// The exact span from `transcript_text` used as source-of-truth.
    pub original_span: String,
    /// Optional deterministic normalization (slang mapping, explicit numeric normalization, etc).
    pub normalized_value: Option<String>,
    /// Optional structured time expression (only when deterministic).
    pub normalized_time: Option<TimeExpression>,
}

impl FieldValue {
    pub fn verbatim(original_span: String) -> Result<Self, ContractViolation> {
        if original_span.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "field_value.original_span",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            original_span,
            normalized_value: None,
            normalized_time: None,
        })
    }

    pub fn normalized(
        original_span: String,
        normalized_value: String,
    ) -> Result<Self, ContractViolation> {
        if original_span.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "field_value.original_span",
                reason: "must not be empty",
            });
        }
        if normalized_value.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "field_value.normalized_value",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            original_span,
            normalized_value: Some(normalized_value),
            normalized_time: None,
        })
    }

    pub fn time(original_span: String, time: TimeExpression) -> Result<Self, ContractViolation> {
        if original_span.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "field_value.original_span",
                reason: "must not be empty",
            });
        }
        time.validate()?;
        Ok(Self {
            original_span,
            normalized_value: Some(time.normalized.clone()),
            normalized_time: Some(time),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentField {
    pub key: FieldKey,
    pub value: FieldValue,
    pub confidence: OverallConfidence,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSpan {
    pub field: FieldKey,
    pub transcript_hash: TranscriptHash,
    /// Byte offsets into `transcript_text` (UTF-8). Used for UI highlight + replay verification.
    pub start_byte: u32,
    pub end_byte: u32,
    pub verbatim_excerpt: String,
}

impl Validate for EvidenceSpan {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.verbatim_excerpt.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "evidence_span.verbatim_excerpt",
                reason: "must not be empty",
            });
        }
        if self.end_byte <= self.start_byte {
            return Err(ContractViolation::InvalidValue {
                field: "evidence_span.end_byte",
                reason: "must be > start_byte",
            });
        }
        // Keep evidence bounded (bytes, not chars).
        let width = (self.end_byte - self.start_byte) as usize;
        if width > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "evidence_span",
                reason: "span width must be <= 32768 bytes",
            });
        }
        if self.verbatim_excerpt.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "evidence_span.verbatim_excerpt",
                reason: "must be <= 32768 bytes",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct IntentDraft {
    pub schema_version: SchemaVersion,
    pub intent_type: IntentType,
    /// Version of the intent schema/taxonomy used (separate from contract schema_version).
    pub intent_schema_version: SchemaVersion,
    pub fields: Vec<IntentField>,
    pub required_fields_missing: Vec<FieldKey>,
    pub overall_confidence: OverallConfidence,
    pub evidence_spans: Vec<EvidenceSpan>,
    pub reason_code: ReasonCodeId,
    /// Enterprise metadata (non-authoritative). Used for downstream UX/audit; never grants authority.
    pub sensitivity_level: SensitivityLevel,
    /// Enterprise metadata (non-authoritative). Downstream may require explicit confirmation for risky intents.
    pub requires_confirmation: bool,
    /// Bounded uncertainty flags (non-authoritative). Prefer clarify when these block HIGH confidence.
    pub ambiguity_flags: Vec<AmbiguityFlag>,
    /// Bounded routing hints (non-authoritative). Must never bypass Access/Simulation gates.
    pub routing_hints: Vec<RoutingHint>,
}

impl IntentDraft {
    pub fn v1(
        intent_type: IntentType,
        intent_schema_version: SchemaVersion,
        fields: Vec<IntentField>,
        required_fields_missing: Vec<FieldKey>,
        overall_confidence: OverallConfidence,
        evidence_spans: Vec<EvidenceSpan>,
        reason_code: ReasonCodeId,
        sensitivity_level: SensitivityLevel,
        requires_confirmation: bool,
        ambiguity_flags: Vec<AmbiguityFlag>,
        routing_hints: Vec<RoutingHint>,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            intent_type,
            intent_schema_version,
            fields,
            required_fields_missing,
            overall_confidence,
            evidence_spans,
            reason_code,
            sensitivity_level,
            requires_confirmation,
            ambiguity_flags,
            routing_hints,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for IntentDraft {
    fn validate(&self) -> Result<(), ContractViolation> {
        for e in &self.evidence_spans {
            e.validate()?;
        }
        if self.ambiguity_flags.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "intent_draft.ambiguity_flags",
                reason: "must be <= 8 entries",
            });
        }
        if self.routing_hints.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "intent_draft.routing_hints",
                reason: "must be <= 8 entries",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Clarify {
    pub schema_version: SchemaVersion,
    pub question: String,
    pub what_is_missing: Vec<FieldKey>,
    pub accepted_answer_formats: Vec<String>,
    pub reason_code: ReasonCodeId,
    /// Enterprise metadata (non-authoritative). Used for downstream UX/audit; never grants authority.
    pub sensitivity_level: SensitivityLevel,
    /// Enterprise metadata (non-authoritative). Downstream may require explicit confirmation after clarify.
    pub requires_confirmation: bool,
    /// Bounded uncertainty flags (non-authoritative). Helps UI/audit explain "why we asked".
    pub ambiguity_flags: Vec<AmbiguityFlag>,
    /// Bounded routing hints (non-authoritative). Must never bypass Access/Simulation gates.
    pub routing_hints: Vec<RoutingHint>,
}

impl Clarify {
    pub fn v1(
        question: String,
        what_is_missing: Vec<FieldKey>,
        accepted_answer_formats: Vec<String>,
        reason_code: ReasonCodeId,
        sensitivity_level: SensitivityLevel,
        requires_confirmation: bool,
        ambiguity_flags: Vec<AmbiguityFlag>,
        routing_hints: Vec<RoutingHint>,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            question,
            what_is_missing,
            accepted_answer_formats,
            reason_code,
            sensitivity_level,
            requires_confirmation,
            ambiguity_flags,
            routing_hints,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for Clarify {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.question.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.question",
                reason: "must not be empty",
            });
        }
        if self.what_is_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.what_is_missing",
                reason: "must not be empty",
            });
        }
        // Hard rule: one question => one missing field (no "two things at once").
        if self.what_is_missing.len() != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.what_is_missing",
                reason: "must contain exactly 1 entry",
            });
        }
        if !(2..=3).contains(&self.accepted_answer_formats.len()) {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.accepted_answer_formats",
                reason: "must contain 2–3 entries",
            });
        }
        for f in &self.accepted_answer_formats {
            if f.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "clarify.accepted_answer_formats[]",
                    reason: "must not contain empty strings",
                });
            }
            if f.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "clarify.accepted_answer_formats[]",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if self.ambiguity_flags.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.ambiguity_flags",
                reason: "must be <= 8 entries",
            });
        }
        if self.routing_hints.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify.routing_hints",
                reason: "must be <= 8 entries",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chat {
    pub schema_version: SchemaVersion,
    pub response_text: String,
    pub reason_code: ReasonCodeId,
}

impl Chat {
    pub fn v1(response_text: String, reason_code: ReasonCodeId) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            response_text,
            reason_code,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for Chat {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "chat.response_text",
                reason: "must not be empty",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1nResponse {
    IntentDraft(IntentDraft),
    Clarify(Clarify),
    Chat(Chat),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1nRequest {
    pub schema_version: SchemaVersion,
    pub transcript_ok: TranscriptOk,
    pub session_state_ref: SessionStateRef,
    /// Optional time context supplied by Selene OS (used to resolve relative phrases deterministically).
    pub time_context: Option<TimeContext>,
    /// Optional uncertain spans from PH1.C. PH1.NLP uses these to ask targeted questions (no guessing).
    pub uncertain_spans: Vec<UncertainSpan>,
    /// Optional confirmed context from the last confirmed WorkOrder (used for safe reference resolution).
    pub confirmed_context: Option<ConfirmedContext>,
    /// Optional runtime tenant scope supplied by Selene OS/app ingress; never extracted from transcript.
    pub runtime_tenant_id: Option<String>,
}

impl Ph1nRequest {
    pub fn v1(
        transcript_ok: TranscriptOk,
        session_state_ref: SessionStateRef,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            transcript_ok,
            session_state_ref,
            time_context: None,
            uncertain_spans: vec![],
            confirmed_context: None,
            runtime_tenant_id: None,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn with_runtime_tenant_id(
        mut self,
        runtime_tenant_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        self.runtime_tenant_id = runtime_tenant_id
            .map(|tenant| tenant.trim().to_string())
            .filter(|tenant| !tenant.is_empty());
        self.validate()?;
        Ok(self)
    }
}

impl Validate for Ph1nRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1N_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1n_request.schema_version",
                reason: "must match PH1N_CONTRACT_VERSION",
            });
        }
        self.transcript_ok.validate()?;
        self.session_state_ref.validate()?;
        if self.uncertain_spans.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1n_request.uncertain_spans",
                reason: "must be <= 8 entries",
            });
        }
        for s in &self.uncertain_spans {
            s.validate()?;
            if (s.end_byte as usize) > self.transcript_ok.transcript_text.len() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1n_request.uncertain_spans.end_byte",
                    reason: "must be <= transcript_ok.transcript_text byte length",
                });
            }
            if !self
                .transcript_ok
                .transcript_text
                .is_char_boundary(s.start_byte as usize)
                || !self
                    .transcript_ok
                    .transcript_text
                    .is_char_boundary(s.end_byte as usize)
            {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1n_request.uncertain_spans",
                    reason: "start/end must align to UTF-8 char boundaries",
                });
            }
        }
        if let Some(t) = &self.time_context {
            t.validate()?;
        }
        if let Some(c) = &self.confirmed_context {
            c.validate()?;
        }
        if let Some(tenant_id) = &self.runtime_tenant_id {
            validate_runtime_tenant_id(tenant_id)?;
        }
        Ok(())
    }
}

fn validate_runtime_tenant_id(tenant_id: &str) -> Result<(), ContractViolation> {
    let trimmed = tenant_id.trim();
    if trimmed.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.runtime_tenant_id",
            reason: "must not be empty when present",
        });
    }
    if trimmed.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.runtime_tenant_id",
            reason: "must be <= 128 chars",
        });
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
    {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.runtime_tenant_id",
            reason: "must contain only [A-Za-z0-9_.-]",
        });
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UncertainSpanKind {
    NumberLike,
    NameLike,
    DateTimeLike,
    AmountLike,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UncertainSpan {
    pub schema_version: SchemaVersion,
    pub kind: UncertainSpanKind,
    pub field_hint: Option<FieldKey>,
    pub start_byte: u32,
    pub end_byte: u32,
}

impl UncertainSpan {
    pub fn v1(
        kind: UncertainSpanKind,
        field_hint: Option<FieldKey>,
        start_byte: u32,
        end_byte: u32,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            kind,
            field_hint,
            start_byte,
            end_byte,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for UncertainSpan {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1N_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "uncertain_span.schema_version",
                reason: "must match PH1N_CONTRACT_VERSION",
            });
        }
        if self.end_byte <= self.start_byte {
            return Err(ContractViolation::InvalidValue {
                field: "uncertain_span.end_byte",
                reason: "must be > start_byte",
            });
        }
        // Bounded width to keep downstream safe.
        let width = (self.end_byte - self.start_byte) as usize;
        if width > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "uncertain_span",
                reason: "span width must be <= 256 bytes",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeContext {
    pub schema_version: SchemaVersion,
    /// Unix epoch milliseconds (injected by Selene OS).
    pub now_unix_ms: u64,
    /// Timezone offset from UTC in minutes (injected by Selene OS).
    pub tz_offset_minutes: i16,
}

impl TimeContext {
    pub fn v1(now_unix_ms: u64, tz_offset_minutes: i16) -> Self {
        Self {
            schema_version: PH1N_CONTRACT_VERSION,
            now_unix_ms,
            tz_offset_minutes,
        }
    }
}

impl Validate for TimeContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1N_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "time_context.schema_version",
                reason: "must match PH1N_CONTRACT_VERSION",
            });
        }
        // Keep bounds practical.
        if self.tz_offset_minutes < -14 * 60 || self.tz_offset_minutes > 14 * 60 {
            return Err(ContractViolation::InvalidValue {
                field: "time_context.tz_offset_minutes",
                reason: "must be within [-840, +840]",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConfirmedContext {
    pub schema_version: SchemaVersion,
    pub last_confirmed_intent: Option<IntentType>,
    pub last_confirmed_fields: Vec<IntentField>,
}

impl ConfirmedContext {
    pub fn v1(
        last_confirmed_intent: Option<IntentType>,
        last_confirmed_fields: Vec<IntentField>,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1N_CONTRACT_VERSION,
            last_confirmed_intent,
            last_confirmed_fields,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for ConfirmedContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1N_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "confirmed_context.schema_version",
                reason: "must match PH1N_CONTRACT_VERSION",
            });
        }
        if self.last_confirmed_fields.len() > 24 {
            return Err(ContractViolation::InvalidValue {
                field: "confirmed_context.last_confirmed_fields",
                reason: "must be <= 24 entries",
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1c::{ConfidenceBucket, LanguageTag};
    use crate::ph1w::SessionState;
    use crate::ReasonCodeId;

    #[test]
    fn clarify_requires_two_or_three_formats() {
        let bad = Clarify::v1(
            "When?".to_string(),
            vec![FieldKey::When],
            vec!["tomorrow at 3pm".to_string()],
            ReasonCodeId(1),
            SensitivityLevel::Public,
            false,
            vec![],
            vec![],
        );
        assert!(bad.is_err());
    }

    #[test]
    fn request_requires_transcript_ok() {
        let ok = TranscriptOk::v1(
            "hello".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let req = Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false));
        assert!(req.is_ok());
    }

    #[test]
    fn request_rejects_schema_drift() {
        let ok = TranscriptOk::v1(
            "hello".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let mut req = Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false))
            .expect("request must construct");
        req.schema_version = SchemaVersion(999);
        assert!(req.validate().is_err());
    }

    #[test]
    fn request_rejects_uncertain_span_outside_transcript_bounds() {
        let ok = TranscriptOk::v1(
            "hello".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let mut req = Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false))
            .expect("request must construct");
        req.uncertain_spans.push(
            UncertainSpan::v1(UncertainSpanKind::Unknown, Some(FieldKey::Task), 1, 3).unwrap(),
        );
        req.uncertain_spans[0].end_byte = 99;
        assert!(req.validate().is_err());
    }

    #[test]
    fn request_accepts_runtime_tenant_id_context() {
        let ok = TranscriptOk::v1(
            "hello".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let req = Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false))
            .expect("request must construct")
            .with_runtime_tenant_id(Some("tenant_1".to_string()))
            .expect("runtime tenant context should validate");
        assert_eq!(req.runtime_tenant_id.as_deref(), Some("tenant_1"));
    }
}
