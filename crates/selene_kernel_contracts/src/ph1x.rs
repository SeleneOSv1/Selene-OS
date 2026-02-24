#![forbid(unsafe_code)]

use crate::ph1_voice_id::Ph1VoiceIdResponse;
use crate::ph1d::PolicyContextRef;
use crate::ph1e::{ToolRequest, ToolRequestId, ToolResponse};
use crate::ph1k::{InterruptCandidate, PH1K_CONTRACT_VERSION};
use crate::ph1m::MemoryCandidate;
use crate::ph1n::{FieldKey, IntentDraft, OverallConfidence, Ph1nResponse};
use crate::ph1tts::{AnswerId, TtsControl};
use crate::ph1w::SessionState;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1X_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);
pub const PH1X_UNKNOWN_ACTIVE_SPEAKER_USER_ID: &str = "unknown";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeliveryHint {
    /// OS may render text and also request speech from PH1.TTS.
    AudibleAndText,
    /// OS must render text only (do not request speech from PH1.TTS).
    TextOnly,
    /// No output. Used for listening posture (e.g., interruption stop).
    Silent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfirmAnswer {
    Yes,
    No,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepUpActionClass {
    Payments,
    CapabilityGovernance,
    AccessGovernance,
}

impl StepUpActionClass {
    pub const fn as_str(self) -> &'static str {
        match self {
            StepUpActionClass::Payments => "PAYMENTS",
            StepUpActionClass::CapabilityGovernance => "CAPABILITY_GOVERNANCE",
            StepUpActionClass::AccessGovernance => "ACCESS_GOVERNANCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepUpChallengeMethod {
    DeviceBiometric,
    DevicePasscode,
}

impl StepUpChallengeMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            StepUpChallengeMethod::DeviceBiometric => "DEVICE_BIOMETRIC",
            StepUpChallengeMethod::DevicePasscode => "DEVICE_PASSCODE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StepUpOutcome {
    Continue,
    Refuse,
    Defer,
}

impl StepUpOutcome {
    pub const fn as_str(self) -> &'static str {
        match self {
            StepUpOutcome::Continue => "CONTINUE",
            StepUpOutcome::Refuse => "REFUSE",
            StepUpOutcome::Defer => "DEFER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StepUpCapabilities {
    pub supports_biometric: bool,
    pub supports_passcode: bool,
}

impl StepUpCapabilities {
    pub const fn v1(supports_biometric: bool, supports_passcode: bool) -> Self {
        Self {
            supports_biometric,
            supports_passcode,
        }
    }
}

impl Validate for StepUpCapabilities {
    fn validate(&self) -> Result<(), ContractViolation> {
        if !self.supports_biometric && !self.supports_passcode {
            return Err(ContractViolation::InvalidValue {
                field: "step_up_capabilities",
                reason: "must support at least one challenge method",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepUpResult {
    pub schema_version: SchemaVersion,
    pub outcome: StepUpOutcome,
    pub challenge_method: StepUpChallengeMethod,
    pub reason_code: ReasonCodeId,
}

impl StepUpResult {
    pub fn v1(
        outcome: StepUpOutcome,
        challenge_method: StepUpChallengeMethod,
        reason_code: ReasonCodeId,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            outcome,
            challenge_method,
            reason_code,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for StepUpResult {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "step_up_result.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "step_up_result.reason_code",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdentityContext {
    /// Voice modality: identity and diarization-lite results from PH1.VOICE.ID.
    Voice(Ph1VoiceIdResponse),
    /// Text modality: UI-authenticated user identity (no speaker binding required).
    TextUserId(String),
}

impl Validate for IdentityContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            IdentityContext::Voice(v) => v.validate(),
            IdentityContext::TextUserId(u) => {
                if u.trim().is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "identity_context.text_user_id",
                        reason: "must not be empty",
                    });
                }
                if u.len() > 128 {
                    return Err(ContractViolation::InvalidValue {
                        field: "identity_context.text_user_id",
                        reason: "must be <= 128 chars",
                    });
                }
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeBuffer {
    pub schema_version: SchemaVersion,
    /// Interrupted answer identifier (ties Resume Buffer to TTS playback).
    pub answer_id: AnswerId,
    /// Optional stable topic label for the interrupted answer (audit/UX only).
    pub topic_hint: Option<String>,
    /// What Selene already spoke out loud.
    pub spoken_prefix: String,
    /// The exact text Selene did not speak yet.
    pub unsaid_remainder: String,
    /// Hard expiry (short TTL). After this time, the buffer is ignored.
    pub expires_at: MonotonicTimeNs,
}

impl ResumeBuffer {
    pub fn v1(
        answer_id: AnswerId,
        topic_hint: Option<String>,
        spoken_prefix: String,
        unsaid_remainder: String,
        expires_at: MonotonicTimeNs,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            answer_id,
            topic_hint,
            spoken_prefix,
            unsaid_remainder,
            expires_at,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for ResumeBuffer {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "resume_buffer.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if let Some(h) = &self.topic_hint {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "resume_buffer.topic_hint",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "resume_buffer.topic_hint",
                    reason: "must be <= 64 chars",
                });
            }
        }
        if self.spoken_prefix.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "resume_buffer.spoken_prefix",
                reason: "must be <= 32768 chars",
            });
        }
        if self.unsaid_remainder.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "resume_buffer.unsaid_remainder",
                reason: "must not be empty",
            });
        }
        if self.unsaid_remainder.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "resume_buffer.unsaid_remainder",
                reason: "must be <= 32768 chars",
            });
        }
        if self.expires_at.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "resume_buffer.expires_at",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum PendingState {
    Clarify {
        missing_field: FieldKey,
        attempts: u8,
    },
    Confirm {
        intent_draft: IntentDraft,
        attempts: u8,
    },
    /// Pending decision: PH1.X asked for permission to use potentially-sensitive memory.
    /// Holds a deferred response so the permission question can be answered without re-running upstream engines.
    MemoryPermission {
        deferred_response_text: String,
        attempts: u8,
    },
    /// Deterministic step-up challenge handoff for high-stakes intents.
    StepUp {
        intent_draft: IntentDraft,
        requested_action: String,
        challenge_method: StepUpChallengeMethod,
        attempts: u8,
    },
    Tool {
        request_id: ToolRequestId,
        attempts: u8,
    },
}

impl Validate for PendingState {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            PendingState::Clarify { attempts, .. }
            | PendingState::Confirm { attempts, .. }
            | PendingState::MemoryPermission { attempts, .. }
            | PendingState::StepUp { attempts, .. }
            | PendingState::Tool { attempts, .. } => {
                if *attempts == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "pending_state.attempts",
                        reason: "must be >= 1",
                    });
                }
                if *attempts > 10 {
                    return Err(ContractViolation::InvalidValue {
                        field: "pending_state.attempts",
                        reason: "must be <= 10",
                    });
                }
            }
        }
        if let PendingState::Confirm { intent_draft, .. } = self {
            intent_draft.validate()?;
            if intent_draft.overall_confidence != OverallConfidence::High {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.confirm.intent_draft.overall_confidence",
                    reason: "must be High for confirmation snapshot",
                });
            }
            if !intent_draft.required_fields_missing.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.confirm.intent_draft.required_fields_missing",
                    reason: "must be empty for confirmation snapshot",
                });
            }
            // Keep thread_state lightweight: confirmation snapshots must not carry verbatim transcript excerpts.
            if !intent_draft.evidence_spans.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.confirm.intent_draft.evidence_spans",
                    reason: "must be empty for confirmation snapshot",
                });
            }
        }
        if let PendingState::StepUp {
            intent_draft,
            requested_action,
            ..
        } = self
        {
            intent_draft.validate()?;
            if intent_draft.overall_confidence != OverallConfidence::High {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.step_up.intent_draft.overall_confidence",
                    reason: "must be High for step-up snapshot",
                });
            }
            if !intent_draft.required_fields_missing.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.step_up.intent_draft.required_fields_missing",
                    reason: "must be empty for step-up snapshot",
                });
            }
            // Keep thread_state lightweight and deterministic.
            if !intent_draft.evidence_spans.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.step_up.intent_draft.evidence_spans",
                    reason: "must be empty for step-up snapshot",
                });
            }
            if requested_action.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.step_up.requested_action",
                    reason: "must not be empty",
                });
            }
            if requested_action.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.step_up.requested_action",
                    reason: "must be <= 128 chars",
                });
            }
        }
        if let PendingState::MemoryPermission {
            deferred_response_text,
            ..
        } = self
        {
            if deferred_response_text.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.memory_permission.deferred_response_text",
                    reason: "must not be empty",
                });
            }
            if deferred_response_text.len() > 32_768 {
                return Err(ContractViolation::InvalidValue {
                    field: "pending_state.memory_permission.deferred_response_text",
                    reason: "must be <= 32768 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ThreadState {
    pub schema_version: SchemaVersion,
    pub pending: Option<PendingState>,
    pub resume_buffer: Option<ResumeBuffer>,
    /// Deterministic ask-once identity prompt tracking for voice confidence ladders.
    pub identity_prompt_state: Option<IdentityPromptState>,
    /// Optional continuity topic carried across turns in one correlation chain.
    pub active_subject_ref: Option<String>,
    /// Optional active speaker user identity carried across turns.
    pub active_speaker_user_id: Option<String>,
}

impl ThreadState {
    pub fn empty_v1() -> Self {
        Self {
            schema_version: PH1X_CONTRACT_VERSION,
            pending: None,
            resume_buffer: None,
            identity_prompt_state: None,
            active_subject_ref: None,
            active_speaker_user_id: None,
        }
    }

    pub fn v1(pending: Option<PendingState>, resume_buffer: Option<ResumeBuffer>) -> Self {
        Self {
            schema_version: PH1X_CONTRACT_VERSION,
            pending,
            resume_buffer,
            identity_prompt_state: None,
            active_subject_ref: None,
            active_speaker_user_id: None,
        }
    }

    pub fn with_continuity(
        mut self,
        active_subject_ref: Option<String>,
        active_speaker_user_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        self.active_subject_ref = active_subject_ref;
        self.active_speaker_user_id = active_speaker_user_id;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for ThreadState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "thread_state.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if let Some(p) = &self.pending {
            p.validate()?;
        }
        if let Some(b) = &self.resume_buffer {
            b.validate()?;
        }
        if let Some(p) = &self.identity_prompt_state {
            p.validate()?;
        }
        if let Some(subject_ref) = &self.active_subject_ref {
            if subject_ref.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "thread_state.active_subject_ref",
                    reason: "must not be empty when provided",
                });
            }
            if subject_ref.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "thread_state.active_subject_ref",
                    reason: "must be <= 256 chars",
                });
            }
        }
        if let Some(active_speaker_user_id) = &self.active_speaker_user_id {
            if active_speaker_user_id.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "thread_state.active_speaker_user_id",
                    reason: "must not be empty when provided",
                });
            }
            if active_speaker_user_id.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "thread_state.active_speaker_user_id",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdentityPromptState {
    pub schema_version: SchemaVersion,
    pub prompted_in_session: bool,
    pub last_prompt_at: Option<MonotonicTimeNs>,
    /// Deterministic identity prompt scope key (tenant/user/device/voice-branch lineage).
    /// Optional for backward compatibility; when missing, scope is treated as global.
    pub prompt_scope_key: Option<String>,
    /// Prompt attempts observed in the current cooldown window for `prompt_scope_key`.
    pub prompts_in_scope: u8,
}

impl IdentityPromptState {
    pub fn v1(
        prompted_in_session: bool,
        last_prompt_at: Option<MonotonicTimeNs>,
    ) -> Result<Self, ContractViolation> {
        let default_prompts_in_scope = if prompted_in_session { 1 } else { 0 };
        Self::v1_with_scope(
            prompted_in_session,
            last_prompt_at,
            None,
            default_prompts_in_scope,
        )
    }

    pub fn v1_with_scope(
        prompted_in_session: bool,
        last_prompt_at: Option<MonotonicTimeNs>,
        prompt_scope_key: Option<String>,
        prompts_in_scope: u8,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            prompted_in_session,
            last_prompt_at,
            prompt_scope_key,
            prompts_in_scope,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for IdentityPromptState {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "identity_prompt_state.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.prompted_in_session && self.last_prompt_at.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "identity_prompt_state.last_prompt_at",
                reason: "must be Some(...) when prompted_in_session=true",
            });
        }
        if let Some(t) = self.last_prompt_at {
            if t.0 == 0 {
                return Err(ContractViolation::InvalidValue {
                    field: "identity_prompt_state.last_prompt_at",
                    reason: "must be > 0 when provided",
                });
            }
        }
        if self.prompts_in_scope > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "identity_prompt_state.prompts_in_scope",
                reason: "must be <= 8",
            });
        }
        if self.prompts_in_scope > 0 && self.last_prompt_at.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "identity_prompt_state.last_prompt_at",
                reason: "must be Some(...) when prompts_in_scope > 0",
            });
        }
        if let Some(scope_key) = &self.prompt_scope_key {
            if scope_key.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "identity_prompt_state.prompt_scope_key",
                    reason: "must not be empty when provided",
                });
            }
            if scope_key.len() > 192 {
                return Err(ContractViolation::InvalidValue {
                    field: "identity_prompt_state.prompt_scope_key",
                    reason: "must be <= 192 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TtsResumeSnapshot {
    pub schema_version: SchemaVersion,
    pub answer_id: AnswerId,
    pub topic_hint: Option<String>,
    pub response_text: String,
    pub spoken_cursor_byte: u32,
}

impl TtsResumeSnapshot {
    pub fn v1(
        answer_id: AnswerId,
        topic_hint: Option<String>,
        response_text: String,
        spoken_cursor_byte: u32,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            answer_id,
            topic_hint,
            response_text,
            spoken_cursor_byte,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for TtsResumeSnapshot {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tts_resume_snapshot.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if let Some(h) = &self.topic_hint {
            if h.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "tts_resume_snapshot.topic_hint",
                    reason: "must not be empty when provided",
                });
            }
            if h.len() > 64 {
                return Err(ContractViolation::InvalidValue {
                    field: "tts_resume_snapshot.topic_hint",
                    reason: "must be <= 64 chars",
                });
            }
        }
        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "tts_resume_snapshot.response_text",
                reason: "must not be empty",
            });
        }
        if self.response_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "tts_resume_snapshot.response_text",
                reason: "must be <= 32768 chars",
            });
        }
        if (self.spoken_cursor_byte as usize) > self.response_text.len() {
            return Err(ContractViolation::InvalidValue {
                field: "tts_resume_snapshot.spoken_cursor_byte",
                reason: "must be <= response_text byte length",
            });
        }
        if !self
            .response_text
            .is_char_boundary(self.spoken_cursor_byte as usize)
        {
            return Err(ContractViolation::InvalidValue {
                field: "tts_resume_snapshot.spoken_cursor_byte",
                reason: "must align to a UTF-8 char boundary",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1xRequest {
    pub schema_version: SchemaVersion,
    /// Audit correlation for the current WorkOrder thread.
    pub correlation_id: u128,
    /// Monotonic turn counter within `correlation_id`.
    pub turn_id: u64,
    /// Monotonic time supplied by Selene OS (used for deterministic expiry windows).
    pub now: MonotonicTimeNs,
    pub thread_state: ThreadState,
    pub session_state: SessionState,
    /// Identity context (voice or text). Used for privacy-safe personalization decisions.
    pub identity_context: IdentityContext,
    /// Topic continuity key for this turn.
    pub subject_ref: String,
    /// Active speaker user identity for this turn ("unknown" when not user-bound).
    pub active_speaker_user_id: String,
    /// Optional deterministic scope key for ask-once/cooldown identity prompts.
    pub identity_prompt_scope_key: Option<String>,
    /// Device-supported step-up methods. Selection is deterministic:
    /// biometric first, passcode fallback.
    pub step_up_capabilities: Option<StepUpCapabilities>,
    pub policy_context_ref: PolicyContextRef,
    /// Optional memory candidates proposed/returned by PH1.M (bounded, evidence-backed).
    /// PH1.X decides whether to use silently, ask permission, or ignore.
    pub memory_candidates: Vec<MemoryCandidate>,
    /// Optional confirmation answer extracted by Selene OS when PH1.X is awaiting a confirm response.
    pub confirm_answer: Option<ConfirmAnswer>,
    /// NLP output for the current user turn (if present).
    pub nlp_output: Option<Ph1nResponse>,
    /// Tool response (if this PH1.X call is completing a prior dispatch).
    pub tool_response: Option<ToolResponse>,
    /// Interrupt candidate (barge-in). If present, PH1.X may cancel speech immediately.
    pub interruption: Option<InterruptCandidate>,
    /// Optional active TTS snapshot used to build Resume Buffer when barge-in cancels speech.
    pub tts_resume_snapshot: Option<TtsResumeSnapshot>,
    /// Optional deterministic step-up outcome from PH1.ACCESS/CAPREQ.
    pub step_up_result: Option<StepUpResult>,
    pub locale: Option<String>,
    pub last_failure_reason_code: Option<ReasonCodeId>,
}

impl Ph1xRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: u128,
        turn_id: u64,
        now: MonotonicTimeNs,
        thread_state: ThreadState,
        session_state: SessionState,
        identity_context: IdentityContext,
        policy_context_ref: PolicyContextRef,
        memory_candidates: Vec<MemoryCandidate>,
        confirm_answer: Option<ConfirmAnswer>,
        nlp_output: Option<Ph1nResponse>,
        tool_response: Option<ToolResponse>,
        interruption: Option<InterruptCandidate>,
        locale: Option<String>,
        last_failure_reason_code: Option<ReasonCodeId>,
    ) -> Result<Self, ContractViolation> {
        let subject_ref = derive_subject_ref(
            &thread_state,
            &nlp_output,
            &tool_response,
            &interruption,
            confirm_answer,
            last_failure_reason_code,
        );
        let active_speaker_user_id = derive_active_speaker_user_id(&identity_context);
        let r = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            now,
            thread_state,
            session_state,
            identity_context,
            subject_ref,
            active_speaker_user_id,
            identity_prompt_scope_key: None,
            step_up_capabilities: Some(StepUpCapabilities::v1(false, true)),
            policy_context_ref,
            memory_candidates,
            confirm_answer,
            nlp_output,
            tool_response,
            interruption,
            tts_resume_snapshot: None,
            step_up_result: None,
            locale,
            last_failure_reason_code,
        };
        r.validate()?;
        Ok(r)
    }

    pub fn with_continuity_context(
        mut self,
        subject_ref: String,
        active_speaker_user_id: String,
    ) -> Result<Self, ContractViolation> {
        self.subject_ref = subject_ref;
        self.active_speaker_user_id = active_speaker_user_id;
        self.validate()?;
        Ok(self)
    }

    pub fn with_tts_resume_snapshot(
        mut self,
        tts_resume_snapshot: Option<TtsResumeSnapshot>,
    ) -> Result<Self, ContractViolation> {
        self.tts_resume_snapshot = tts_resume_snapshot;
        self.validate()?;
        Ok(self)
    }

    pub fn with_identity_prompt_scope_key(
        mut self,
        identity_prompt_scope_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        self.identity_prompt_scope_key = identity_prompt_scope_key;
        self.validate()?;
        Ok(self)
    }

    pub fn with_step_up_capabilities(
        mut self,
        step_up_capabilities: Option<StepUpCapabilities>,
    ) -> Result<Self, ContractViolation> {
        self.step_up_capabilities = step_up_capabilities;
        self.validate()?;
        Ok(self)
    }

    pub fn with_step_up_result(
        mut self,
        step_up_result: Option<StepUpResult>,
    ) -> Result<Self, ContractViolation> {
        self.step_up_result = step_up_result;
        self.validate()?;
        Ok(self)
    }
}

impl Validate for Ph1xRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.turn_id",
                reason: "must be > 0",
            });
        }
        self.thread_state.validate()?;
        self.identity_context.validate()?;
        if self.subject_ref.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.subject_ref",
                reason: "must not be empty",
            });
        }
        if self.subject_ref.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.subject_ref",
                reason: "must be <= 256 chars",
            });
        }
        if self.active_speaker_user_id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.active_speaker_user_id",
                reason: "must not be empty",
            });
        }
        if self.active_speaker_user_id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.active_speaker_user_id",
                reason: "must be <= 128 chars",
            });
        }
        if let Some(scope_key) = &self.identity_prompt_scope_key {
            if scope_key.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.identity_prompt_scope_key",
                    reason: "must not be empty when provided",
                });
            }
            if scope_key.len() > 192 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.identity_prompt_scope_key",
                    reason: "must be <= 192 chars",
                });
            }
        }
        if let Some(capabilities) = &self.step_up_capabilities {
            capabilities.validate()?;
        }
        validate_active_speaker_matches_identity(
            &self.identity_context,
            &self.active_speaker_user_id,
        )?;
        self.policy_context_ref.validate()?;
        if self.memory_candidates.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.memory_candidates",
                reason: "must be <= 32 entries",
            });
        }
        for c in &self.memory_candidates {
            c.validate()?;
        }

        if self.confirm_answer.is_some() {
            match &self.thread_state.pending {
                Some(PendingState::Confirm { .. })
                | Some(PendingState::MemoryPermission { .. }) => {}
                _ => {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1x_request.confirm_answer",
                        reason: "confirm_answer is only valid when thread_state.pending is Confirm or MemoryPermission",
                    });
                }
            }
        }
        if let Some(step_up_result) = &self.step_up_result {
            step_up_result.validate()?;
            match &self.thread_state.pending {
                Some(PendingState::StepUp { .. }) => {}
                _ => {
                    return Err(ContractViolation::InvalidValue {
                        field: "ph1x_request.step_up_result",
                        reason: "step_up_result is only valid when thread_state.pending is StepUp",
                    });
                }
            }
            if self.confirm_answer.is_some() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request",
                    reason: "confirm_answer and step_up_result cannot be present together",
                });
            }
        }

        // At least one signal must be provided, except transient StepUp pending state
        // where a caller may construct then attach `step_up_result` via builder.
        let has_signal = self.nlp_output.is_some()
            || self.tool_response.is_some()
            || self.interruption.is_some()
            || self.confirm_answer.is_some()
            || self.step_up_result.is_some()
            || self.last_failure_reason_code.is_some();
        if !has_signal && !matches!(self.thread_state.pending, Some(PendingState::StepUp { .. })) {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request",
                reason:
                    "must include nlp_output, tool_response, interruption, confirm_answer, step_up_result, or last_failure_reason_code",
            });
        }
        if let Some(out) = &self.nlp_output {
            match out {
                Ph1nResponse::IntentDraft(d) => d.validate()?,
                Ph1nResponse::Clarify(c) => c.validate()?,
                Ph1nResponse::Chat(ch) => ch.validate()?,
            }
        }
        if let Some(tr) = &self.tool_response {
            tr.validate()?;
        }
        if let Some(c) = &self.interruption {
            validate_interrupt_candidate(c)?;
        }
        if let Some(s) = &self.tts_resume_snapshot {
            s.validate()?;
            if self.interruption.is_none() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.tts_resume_snapshot",
                    reason: "is only valid when interruption is present",
                });
            }
        }
        if let Some(loc) = &self.locale {
            if loc.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.locale",
                    reason: "must not be empty when provided",
                });
            }
            if loc.len() > 32 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.locale",
                    reason: "must be <= 32 chars",
                });
            }
        }
        Ok(())
    }
}

fn derive_subject_ref(
    thread_state: &ThreadState,
    nlp_output: &Option<Ph1nResponse>,
    tool_response: &Option<ToolResponse>,
    interruption: &Option<InterruptCandidate>,
    confirm_answer: Option<ConfirmAnswer>,
    last_failure_reason_code: Option<ReasonCodeId>,
) -> String {
    if let Some(subject_ref) = &thread_state.active_subject_ref {
        return subject_ref.clone();
    }

    if let Some(out) = nlp_output {
        return match out {
            Ph1nResponse::IntentDraft(d) => {
                format!(
                    "intent_{}",
                    format!("{:?}", d.intent_type).to_ascii_lowercase()
                )
            }
            Ph1nResponse::Clarify(_) => "clarify".to_string(),
            Ph1nResponse::Chat(_) => "chat".to_string(),
        };
    }
    if tool_response.is_some() {
        return "tool_followup".to_string();
    }
    if interruption.is_some() {
        return "interruption".to_string();
    }
    if confirm_answer.is_some() {
        return "confirmation".to_string();
    }
    if last_failure_reason_code.is_some() {
        return "failure_recovery".to_string();
    }
    "general".to_string()
}

fn derive_active_speaker_user_id(identity_context: &IdentityContext) -> String {
    match identity_context {
        IdentityContext::TextUserId(user_id) => user_id.clone(),
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionOk(ok)) => ok
            .user_id
            .as_ref()
            .map(|u| u.as_str().to_string())
            .unwrap_or_else(|| PH1X_UNKNOWN_ACTIVE_SPEAKER_USER_ID.to_string()),
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)) => {
            PH1X_UNKNOWN_ACTIVE_SPEAKER_USER_ID.to_string()
        }
    }
}

fn validate_active_speaker_matches_identity(
    identity_context: &IdentityContext,
    active_speaker_user_id: &str,
) -> Result<(), ContractViolation> {
    match identity_context {
        IdentityContext::TextUserId(user_id) => {
            if user_id != active_speaker_user_id {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.active_speaker_user_id",
                    reason: "must match identity_context.text_user_id",
                });
            }
        }
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionOk(ok)) => {
            let expected = ok
                .user_id
                .as_ref()
                .map(|u| u.as_str())
                .unwrap_or(PH1X_UNKNOWN_ACTIVE_SPEAKER_USER_ID);
            if active_speaker_user_id != expected {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.active_speaker_user_id",
                    reason: "must match identity_context.voice.speaker_assertion_ok.user_id (or unknown)",
                });
            }
        }
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionUnknown(_)) => {
            if active_speaker_user_id != PH1X_UNKNOWN_ACTIVE_SPEAKER_USER_ID {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.active_speaker_user_id",
                    reason:
                        "must be unknown when identity_context.voice is speaker_assertion_unknown",
                });
            }
        }
    }
    Ok(())
}

fn validate_interrupt_candidate(c: &InterruptCandidate) -> Result<(), ContractViolation> {
    if c.schema_version != PH1K_CONTRACT_VERSION {
        return Err(ContractViolation::InvalidValue {
            field: "ph1x_request.interruption.schema_version",
            reason: "must match PH1K_CONTRACT_VERSION",
        });
    }
    if c.phrase_text.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field: "ph1x_request.interruption.phrase_text",
            reason: "must not be empty",
        });
    }
    if c.phrase_text.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field: "ph1x_request.interruption.phrase_text",
            reason: "must be <= 128 chars",
        });
    }
    if c.phrase_set_version.0 == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "ph1x_request.interruption.phrase_set_version",
            reason: "must be > 0",
        });
    }
    let conf = c.phrase_confidence.0;
    if !conf.is_finite() {
        return Err(ContractViolation::NotFinite {
            field: "ph1x_request.interruption.phrase_confidence",
        });
    }
    if !(0.0..=1.0).contains(&conf) {
        return Err(ContractViolation::InvalidRange {
            field: "ph1x_request.interruption.phrase_confidence",
            min: 0.0,
            max: 1.0,
            got: conf as f64,
        });
    }
    for (field, v) in [
        (
            "ph1x_request.interruption.gate_confidences.vad_confidence",
            c.gate_confidences.vad_confidence.0,
        ),
        (
            "ph1x_request.interruption.gate_confidences.speech_likeness",
            c.gate_confidences.speech_likeness.0,
        ),
        (
            "ph1x_request.interruption.gate_confidences.echo_safe_confidence",
            c.gate_confidences.echo_safe_confidence.0,
        ),
        (
            "ph1x_request.interruption.gate_confidences.phrase_confidence",
            c.gate_confidences.phrase_confidence.0,
        ),
    ] {
        if !v.is_finite() {
            return Err(ContractViolation::NotFinite { field });
        }
        if !(0.0..=1.0).contains(&v) {
            return Err(ContractViolation::InvalidRange {
                field,
                min: 0.0,
                max: 1.0,
                got: v as f64,
            });
        }
    }
    if let Some(v) = c.gate_confidences.nearfield_confidence {
        if !v.0.is_finite() {
            return Err(ContractViolation::NotFinite {
                field: "ph1x_request.interruption.gate_confidences.nearfield_confidence",
            });
        }
        if !(0.0..=1.0).contains(&v.0) {
            return Err(ContractViolation::InvalidRange {
                field: "ph1x_request.interruption.gate_confidences.nearfield_confidence",
                min: 0.0,
                max: 1.0,
                got: v.0 as f64,
            });
        }
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmDirective {
    pub schema_version: SchemaVersion,
    pub text: String,
}

impl ConfirmDirective {
    pub fn v1(text: String) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            text,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for ConfirmDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "confirm_directive.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "confirm_directive.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "confirm_directive.text",
                reason: "must be <= 2048 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClarifyDirective {
    pub schema_version: SchemaVersion,
    pub question: String,
    pub accepted_answer_formats: Vec<String>,
    pub what_is_missing: Vec<FieldKey>,
}

impl ClarifyDirective {
    pub fn v1(
        question: String,
        accepted_answer_formats: Vec<String>,
        what_is_missing: Vec<FieldKey>,
    ) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            question,
            accepted_answer_formats,
            what_is_missing,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for ClarifyDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_directive.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.question.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_directive.question",
                reason: "must not be empty",
            });
        }
        if self.what_is_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_directive.what_is_missing",
                reason: "must not be empty",
            });
        }
        // Hard rule: one question => one missing field (no "two things at once").
        if self.what_is_missing.len() != 1 {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_directive.what_is_missing",
                reason: "must contain exactly 1 entry",
            });
        }
        if !(2..=3).contains(&self.accepted_answer_formats.len()) {
            return Err(ContractViolation::InvalidValue {
                field: "clarify_directive.accepted_answer_formats",
                reason: "must contain 2–3 entries",
            });
        }
        for f in &self.accepted_answer_formats {
            if f.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "clarify_directive.accepted_answer_formats[]",
                    reason: "must not contain empty strings",
                });
            }
            if f.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "clarify_directive.accepted_answer_formats[]",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RespondDirective {
    pub schema_version: SchemaVersion,
    pub response_text: String,
}

impl RespondDirective {
    pub fn v1(response_text: String) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            response_text,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for RespondDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "respond_directive.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.response_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "respond_directive.response_text",
                reason: "must not be empty",
            });
        }
        if self.response_text.len() > 32_768 {
            return Err(ContractViolation::InvalidValue {
                field: "respond_directive.response_text",
                reason: "must be <= 32768 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimulationCandidateDispatch {
    pub schema_version: SchemaVersion,
    pub intent_draft: IntentDraft,
}

impl SimulationCandidateDispatch {
    pub fn v1(intent_draft: IntentDraft) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            intent_draft,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for SimulationCandidateDispatch {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        self.intent_draft.validate()?;
        if self.intent_draft.overall_confidence != OverallConfidence::High {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.overall_confidence",
                reason: "must be High",
            });
        }
        if !self.intent_draft.required_fields_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.required_fields_missing",
                reason: "must be empty",
            });
        }
        // Disallow conversation-control and read-only tool intents here; those have their own paths.
        if matches!(
            self.intent_draft.intent_type,
            crate::ph1n::IntentType::TimeQuery
                | crate::ph1n::IntentType::WeatherQuery
                | crate::ph1n::IntentType::Continue
                | crate::ph1n::IntentType::MoreDetail
        ) {
            return Err(ContractViolation::InvalidValue {
                field: "simulation_candidate_dispatch.intent_draft.intent_type",
                reason: "must not be a tool query or conversation-control intent",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccessStepUpDispatch {
    pub schema_version: SchemaVersion,
    pub intent_draft: IntentDraft,
    pub action_class: StepUpActionClass,
    pub requested_action: String,
    pub challenge_method: StepUpChallengeMethod,
}

impl AccessStepUpDispatch {
    pub fn v1(
        intent_draft: IntentDraft,
        action_class: StepUpActionClass,
        requested_action: String,
        challenge_method: StepUpChallengeMethod,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            intent_draft,
            action_class,
            requested_action,
            challenge_method,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for AccessStepUpDispatch {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        self.intent_draft.validate()?;
        if self.intent_draft.overall_confidence != OverallConfidence::High {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.intent_draft.overall_confidence",
                reason: "must be High",
            });
        }
        if !self.intent_draft.required_fields_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.intent_draft.required_fields_missing",
                reason: "must be empty",
            });
        }
        // Keep thread payload deterministic and bounded.
        if !self.intent_draft.evidence_spans.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.intent_draft.evidence_spans",
                reason: "must be empty",
            });
        }
        if self.requested_action.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.requested_action",
                reason: "must not be empty",
            });
        }
        if self.requested_action.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "access_step_up_dispatch.requested_action",
                reason: "must be <= 128 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DispatchRequest {
    Tool(ToolRequest),
    SimulationCandidate(SimulationCandidateDispatch),
    AccessStepUp(AccessStepUpDispatch),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispatchDirective {
    pub schema_version: SchemaVersion,
    pub dispatch_request: DispatchRequest,
}

impl DispatchDirective {
    pub fn tool_v1(tool_request: ToolRequest) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            dispatch_request: DispatchRequest::Tool(tool_request),
        };
        d.validate()?;
        Ok(d)
    }

    pub fn simulation_candidate_v1(intent_draft: IntentDraft) -> Result<Self, ContractViolation> {
        let cand = SimulationCandidateDispatch::v1(intent_draft)?;
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            dispatch_request: DispatchRequest::SimulationCandidate(cand),
        };
        d.validate()?;
        Ok(d)
    }

    pub fn access_step_up_v1(
        intent_draft: IntentDraft,
        action_class: StepUpActionClass,
        requested_action: String,
        challenge_method: StepUpChallengeMethod,
    ) -> Result<Self, ContractViolation> {
        let dispatch = AccessStepUpDispatch::v1(
            intent_draft,
            action_class,
            requested_action,
            challenge_method,
        )?;
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            dispatch_request: DispatchRequest::AccessStepUp(dispatch),
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for DispatchDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "dispatch_directive.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        match &self.dispatch_request {
            DispatchRequest::Tool(t) => t.validate(),
            DispatchRequest::SimulationCandidate(c) => c.validate(),
            DispatchRequest::AccessStepUp(c) => c.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitDirective {
    pub schema_version: SchemaVersion,
    pub reason: Option<String>,
}

impl WaitDirective {
    pub fn v1(reason: Option<String>) -> Result<Self, ContractViolation> {
        let d = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            reason,
        };
        d.validate()?;
        Ok(d)
    }
}

impl Validate for WaitDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "wait_directive.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if let Some(r) = &self.reason {
            if r.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "wait_directive.reason",
                    reason: "must not be empty when provided",
                });
            }
            if r.len() > 256 {
                return Err(ContractViolation::InvalidValue {
                    field: "wait_directive.reason",
                    reason: "must be <= 256 chars",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1xDirective {
    Confirm(ConfirmDirective),
    Clarify(ClarifyDirective),
    Respond(RespondDirective),
    Dispatch(DispatchDirective),
    Wait(WaitDirective),
}

impl Validate for Ph1xDirective {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1xDirective::Confirm(d) => d.validate(),
            Ph1xDirective::Clarify(d) => d.validate(),
            Ph1xDirective::Respond(d) => d.validate(),
            Ph1xDirective::Dispatch(d) => d.validate(),
            Ph1xDirective::Wait(d) => d.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ph1xResponse {
    pub schema_version: SchemaVersion,
    pub correlation_id: u128,
    pub turn_id: u64,
    pub directive: Ph1xDirective,
    pub thread_state: ThreadState,
    /// Optional TTS control hint. When Some(Cancel), OS must cancel speech immediately.
    pub tts_control: Option<TtsControl>,
    pub delivery_hint: DeliveryHint,
    pub reason_code: ReasonCodeId,
    pub idempotency_key: Option<String>,
}

impl Ph1xResponse {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: u128,
        turn_id: u64,
        directive: Ph1xDirective,
        thread_state: ThreadState,
        tts_control: Option<TtsControl>,
        delivery_hint: DeliveryHint,
        reason_code: ReasonCodeId,
        idempotency_key: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1X_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            directive,
            thread_state,
            tts_control,
            delivery_hint,
            reason_code,
            idempotency_key,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for Ph1xResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1X_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_response.schema_version",
                reason: "must match PH1X_CONTRACT_VERSION",
            });
        }
        if self.correlation_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_response.correlation_id",
                reason: "must be > 0",
            });
        }
        if self.turn_id == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_response.turn_id",
                reason: "must be > 0",
            });
        }
        self.directive.validate()?;
        self.thread_state.validate()?;
        if self.reason_code.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_response.reason_code",
                reason: "must be > 0",
            });
        }
        if let Some(k) = &self.idempotency_key {
            if k.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_response.idempotency_key",
                    reason: "must not be empty when provided",
                });
            }
            if k.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_response.idempotency_key",
                    reason: "must be <= 128 chars",
                });
            }
        }
        Ok(())
    }
}

pub fn requires_clarify(draft: &IntentDraft) -> bool {
    draft.overall_confidence != OverallConfidence::High || !draft.required_fields_missing.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ph1d::SafetyTier;
    use crate::ph1k::{
        Confidence, InterruptCandidate, InterruptGateConfidences, InterruptGates,
        InterruptPhraseId, InterruptPhraseSetVersion, SpeechLikeness,
    };
    use crate::ph1n::{EvidenceSpan, IntentType, SensitivityLevel, TranscriptHash};

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn id_text() -> IdentityContext {
        IdentityContext::TextUserId("user-1".to_string())
    }

    #[test]
    fn continuity_defaults_are_set_for_text_identity() {
        let req = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                crate::ph1n::Chat::v1("hello".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        assert!(!req.subject_ref.trim().is_empty());
        assert_eq!(req.active_speaker_user_id, "user-1");
    }

    #[test]
    fn continuity_context_rejects_active_speaker_mismatch_for_text_identity() {
        let err = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                crate::ph1n::Chat::v1("hello".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_continuity_context("chat".to_string(), "user-2".to_string())
        .unwrap_err();

        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1x_request.active_speaker_user_id");
            }
            _ => panic!("expected InvalidValue for active speaker mismatch"),
        }
    }

    #[test]
    fn request_allows_thread_state_speaker_mismatch_for_runtime_gate() {
        let thread = ThreadState::empty_v1()
            .with_continuity(Some("chat".to_string()), Some("user-2".to_string()))
            .unwrap();

        let req = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                crate::ph1n::Chat::v1("hello".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        assert_eq!(req.active_speaker_user_id, "user-1");
    }

    fn intent_draft_with_evidence() -> IntentDraft {
        IntentDraft::v1(
            IntentType::SendMoney,
            SchemaVersion(1),
            vec![],
            vec![],
            OverallConfidence::High,
            vec![EvidenceSpan {
                field: FieldKey::Task,
                transcript_hash: TranscriptHash(1),
                start_byte: 0,
                end_byte: 1,
                verbatim_excerpt: "x".to_string(),
            }],
            ReasonCodeId(1),
            SensitivityLevel::Public,
            true,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn intent_draft_snapshot() -> IntentDraft {
        IntentDraft::v1(
            IntentType::SendMoney,
            SchemaVersion(1),
            vec![],
            vec![],
            OverallConfidence::High,
            vec![],
            ReasonCodeId(1),
            SensitivityLevel::Public,
            true,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn interrupt_wait() -> InterruptCandidate {
        InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(1),
            "wait".to_string(),
            Confidence::new(0.9).unwrap(),
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            InterruptGateConfidences {
                vad_confidence: Confidence::new(0.9).unwrap(),
                speech_likeness: SpeechLikeness::new(0.9).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                phrase_confidence: Confidence::new(0.9).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
        )
        .unwrap()
    }

    #[test]
    fn confirm_answer_requires_pending_confirm_or_memory_permission() {
        let err = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            Some(ConfirmAnswer::Yes),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();

        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1x_request.confirm_answer");
            }
            _ => panic!("expected InvalidValue for confirm_answer"),
        }
    }

    #[test]
    fn confirm_answer_rejects_non_confirm_pending() {
        let thread = ThreadState::v1(
            Some(PendingState::Clarify {
                missing_field: FieldKey::Task,
                attempts: 1,
            }),
            None,
        );
        let err = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            Some(ConfirmAnswer::No),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();

        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1x_request.confirm_answer");
            }
            _ => panic!("expected InvalidValue for confirm_answer"),
        }
    }

    #[test]
    fn confirm_answer_allows_pending_confirm() {
        let thread = ThreadState::v1(
            Some(PendingState::Confirm {
                intent_draft: intent_draft_snapshot(),
                attempts: 1,
            }),
            None,
        );
        Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            Some(ConfirmAnswer::Yes),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    }

    #[test]
    fn confirm_answer_allows_pending_memory_permission() {
        let thread = ThreadState::v1(
            Some(PendingState::MemoryPermission {
                deferred_response_text: "Okay.".to_string(),
                attempts: 1,
            }),
            None,
        );
        Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            Some(ConfirmAnswer::No),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap();
    }

    #[test]
    fn confirm_pending_requires_empty_evidence_spans() {
        let p = PendingState::Confirm {
            intent_draft: intent_draft_with_evidence(),
            attempts: 1,
        };
        let err = p.validate().unwrap_err();
        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "pending_state.confirm.intent_draft.evidence_spans");
            }
            _ => panic!("expected InvalidValue for confirm evidence_spans"),
        }
    }

    #[test]
    fn step_up_result_requires_pending_step_up() {
        let err = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                crate::ph1n::Chat::v1("hello".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_step_up_result(Some(
            StepUpResult::v1(
                StepUpOutcome::Continue,
                StepUpChallengeMethod::DevicePasscode,
                ReasonCodeId(1),
            )
            .unwrap(),
        ))
        .unwrap_err();

        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1x_request.step_up_result");
            }
            _ => panic!("expected InvalidValue for step_up_result"),
        }
    }

    #[test]
    fn step_up_result_allows_pending_step_up() {
        let thread = ThreadState::v1(
            Some(PendingState::StepUp {
                intent_draft: intent_draft_snapshot(),
                requested_action: "CAPREQ_MANAGE".to_string(),
                challenge_method: StepUpChallengeMethod::DevicePasscode,
                attempts: 1,
            }),
            None,
        );
        let req = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_step_up_result(Some(
            StepUpResult::v1(
                StepUpOutcome::Continue,
                StepUpChallengeMethod::DevicePasscode,
                ReasonCodeId(1),
            )
            .unwrap(),
        ));
        assert!(req.is_ok());
    }

    #[test]
    fn tts_resume_snapshot_requires_interruption() {
        let err = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                crate::ph1n::Chat::v1("hi".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_tts_resume_snapshot(Some(
                TtsResumeSnapshot::v1(
                    AnswerId(1),
                    None,
                    "First. Second.".to_string(),
                    "First.".len() as u32,
                )
                .unwrap(),
            ))
        })
        .unwrap_err();

        match err {
            ContractViolation::InvalidValue { field, .. } => {
                assert_eq!(field, "ph1x_request.tts_resume_snapshot");
            }
            _ => panic!("expected InvalidValue for tts_resume_snapshot"),
        }
    }

    #[test]
    fn tts_resume_snapshot_allows_interruption() {
        let req = Ph1xRequest::v1(
            1,
            1,
            MonotonicTimeNs(1),
            ThreadState::empty_v1(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            None,
            Some(interrupt_wait()),
            None,
            None,
        )
        .and_then(|r| {
            r.with_tts_resume_snapshot(Some(
                TtsResumeSnapshot::v1(
                    AnswerId(1),
                    Some("topic".to_string()),
                    "First. Second.".to_string(),
                    "First.".len() as u32,
                )
                .unwrap(),
            ))
        });
        assert!(req.is_ok());
    }
}
