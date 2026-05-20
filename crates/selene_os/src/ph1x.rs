#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::{IdentityTierV2, Ph1VoiceIdResponse, VoiceIdentityV2};
use selene_kernel_contracts::ph1e::{
    StrictBudget, StructuredAmbiguity, ToolName, ToolRequest, ToolRequestOrigin, ToolResponse,
    ToolResult, ToolStatus, WebAnswerVerificationPacket,
};
use selene_kernel_contracts::ph1m::{
    MemoryCandidate, MemoryConfidence, MemorySensitivityFlag, MemoryUsePolicy,
};
use selene_kernel_contracts::ph1n::{
    AmbiguityFlag, FieldKey, FieldValue, IntentDraft, IntentField, IntentType, OverallConfidence,
    Ph1nResponse,
};
use selene_kernel_contracts::ph1tts::{AnswerId, TtsControl};
use selene_kernel_contracts::ph1x::{
    ActiveContextPacket, AmbiguityLevel, ClarifyDirective, ConfirmDirective, ContinuationType,
    ConversationRhythm, DeliveryHint, DispatchDirective, HumanConversationDirective,
    IdentityContext, IdentityPromptState, InteractionPosture, InterruptContinuityOutcome,
    InterruptResumePolicy, InterruptSubjectRelation, LastTurnRouteClass, PendingState,
    Ph1xCandidateRejection, Ph1xCandidateRejectionLedger, Ph1xCandidateRejectionReasonCode,
    Ph1xCandidateScoreFactors, Ph1xContextCandidate, Ph1xContextCandidateKind, Ph1xDirective,
    Ph1xOwnerOutputContract, Ph1xRequest, Ph1xResponse, ProtectedRisk, ResponseShape, ResumeBuffer,
    StepUpActionClass, StepUpCapabilities, StepUpChallengeMethod, StepUpOutcome, StepUpResult,
    SuggestedNextEngine, ThreadState, UniversalActiveFrameFields, WaitDirective,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};
use std::hash::{Hash, Hasher};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.X reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const X_SESSION_NOT_ACTIVE: ReasonCodeId = ReasonCodeId(0x5800_0001);
    pub const X_INTERRUPT_CANCEL: ReasonCodeId = ReasonCodeId(0x5800_0002);
    pub const X_LAST_FAILURE: ReasonCodeId = ReasonCodeId(0x5800_0003);
    pub const X_NLP_CLARIFY: ReasonCodeId = ReasonCodeId(0x5800_0004);
    pub const X_NLP_CHAT: ReasonCodeId = ReasonCodeId(0x5800_0005);
    pub const X_CONFIRM_REQUIRED: ReasonCodeId = ReasonCodeId(0x5800_0006);
    pub const X_DISPATCH_TOOL: ReasonCodeId = ReasonCodeId(0x5800_0007);
    pub const X_TOOL_OK: ReasonCodeId = ReasonCodeId(0x5800_0008);
    pub const X_TOOL_FAIL: ReasonCodeId = ReasonCodeId(0x5800_0009);
    pub const X_TOOL_AMBIGUOUS: ReasonCodeId = ReasonCodeId(0x5800_000A);
    pub const X_RESUME_CONTINUE: ReasonCodeId = ReasonCodeId(0x5800_000B);
    pub const X_RESUME_MORE_DETAIL: ReasonCodeId = ReasonCodeId(0x5800_000C);
    pub const X_RESUME_EXPIRED: ReasonCodeId = ReasonCodeId(0x5800_000D);
    pub const X_MEMORY_PERMISSION_REQUIRED: ReasonCodeId = ReasonCodeId(0x5800_000E);
    pub const X_CONFIRM_YES_DISPATCH: ReasonCodeId = ReasonCodeId(0x5800_000F);
    pub const X_CONFIRM_NO_ABORT: ReasonCodeId = ReasonCodeId(0x5800_0010);
    pub const X_CONFIRM_ANSWER_INVALID: ReasonCodeId = ReasonCodeId(0x5800_0011);
    pub const X_DISPATCH_SIMULATION_CANDIDATE: ReasonCodeId = ReasonCodeId(0x5800_0012);
    pub const X_MEMORY_PERMISSION_YES: ReasonCodeId = ReasonCodeId(0x5800_0013);
    pub const X_MEMORY_PERMISSION_NO: ReasonCodeId = ReasonCodeId(0x5800_0014);
    pub const X_CONTINUITY_SPEAKER_MISMATCH: ReasonCodeId = ReasonCodeId(0x5800_0015);
    pub const X_CONTINUITY_SUBJECT_MISMATCH: ReasonCodeId = ReasonCodeId(0x5800_0016);
    pub const X_STEPUP_REQUIRED_DISPATCH: ReasonCodeId = ReasonCodeId(0x5800_0017);
    pub const X_STEPUP_CONTINUE_DISPATCH: ReasonCodeId = ReasonCodeId(0x5800_0018);
    pub const X_STEPUP_REFUSED: ReasonCodeId = ReasonCodeId(0x5800_0019);
    pub const X_STEPUP_DEFERRED: ReasonCodeId = ReasonCodeId(0x5800_001A);
    pub const X_STEPUP_CHALLENGE_UNAVAILABLE: ReasonCodeId = ReasonCodeId(0x5800_001B);
    pub const X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY: ReasonCodeId = ReasonCodeId(0x5800_001C);
    pub const X_INTERRUPT_SAME_SUBJECT_APPEND: ReasonCodeId = ReasonCodeId(0x5800_001D);
    pub const X_INTERRUPT_SWITCH_TOPIC: ReasonCodeId = ReasonCodeId(0x5800_001E);
    pub const X_INTERRUPT_RETURN_CHECK_ASKED: ReasonCodeId = ReasonCodeId(0x5800_001F);
    pub const X_INTERRUPT_RESUME_NOW: ReasonCodeId = ReasonCodeId(0x5800_0020);
    pub const X_INTERRUPT_DISCARD: ReasonCodeId = ReasonCodeId(0x5800_0021);
    pub const X_STAGE8_5C_CONTINUE_TOOL: ReasonCodeId = ReasonCodeId(0x5800_0870);
    pub const X_STAGE8_5C_CONTINUE_PLAN: ReasonCodeId = ReasonCodeId(0x5800_0871);
    pub const X_STAGE8_5C_MODIFY_ARTIFACT: ReasonCodeId = ReasonCodeId(0x5800_0872);
    pub const X_STAGE8_5C_CORRECT_TOOL: ReasonCodeId = ReasonCodeId(0x5800_0873);
    pub const X_STAGE8_5C_ANSWER_NEW_TOPIC: ReasonCodeId = ReasonCodeId(0x5800_0874);
    pub const X_STAGE8_5C_ASK_CLARIFICATION: ReasonCodeId = ReasonCodeId(0x5800_0875);
    pub const X_STAGE8_5C_FAIL_CLOSED_PROTECTED: ReasonCodeId = ReasonCodeId(0x5800_0876);
    pub const X_STAGE8_5C_HANDOFF_MEMORY: ReasonCodeId = ReasonCodeId(0x5800_0877);
}

const IDENTITY_PROMPT_COOLDOWN_NS: u64 = 600_000_000_000;
const IDENTITY_PROMPT_RETRY_BUDGET: u8 = 1;
const INTERRUPT_RELATION_CONFIDENCE_MIN: f32 = 0.70;
pub const DETERMINISTIC_TIME_CLARIFICATION_TOPIC: &str = "deterministic_time_clarification";
pub const DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC: &str = "deterministic_weather_clarification";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1xConfig {
    pub tool_timeout_ms: u32,
    pub tool_max_results: u8,
    pub resume_buffer_ttl_ms: u32,
}

impl Ph1xConfig {
    pub fn mvp_v1() -> Self {
        Self {
            tool_timeout_ms: 2_000,
            tool_max_results: 5,
            resume_buffer_ttl_ms: 60_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1xRuntime {
    config: Ph1xConfig,
}

impl Ph1xRuntime {
    pub fn new(config: Ph1xConfig) -> Self {
        Self { config }
    }

    pub fn decide(&self, req: &Ph1xRequest) -> Result<Ph1xResponse, ContractViolation> {
        // PH1.X is deterministic and must fail closed on contract failures.
        req.validate()?;

        // Always clear expired resume state deterministically, even if we don't use it this turn.
        let base_thread_state = clear_expired_resume_buffer(req.thread_state.clone(), req.now);

        let delivery_base =
            if req.policy_context_ref.privacy_mode || req.policy_context_ref.do_not_disturb {
                DeliveryHint::TextOnly
            } else {
                DeliveryHint::AudibleAndText
            };

        // Fail closed: never speak/dispatch when suspended/closed.
        if matches!(
            req.session_state,
            SessionState::Suspended | SessionState::Closed
        ) {
            return self.out_wait(
                req,
                base_thread_state,
                reason_codes::X_SESSION_NOT_ACTIVE,
                Some("session_not_active".to_string()),
                None,
            );
        }

        // Interruption is time-critical: cancel speech immediately and adopt a listening posture.
        if req.interruption.is_some() {
            let mut next_state = clear_pending(base_thread_state);
            let mut interrupted_subject_ref = next_state
                .active_subject_ref
                .clone()
                .or_else(|| Some(req.subject_ref.clone()));
            if let Some(snapshot) = &req.tts_resume_snapshot {
                let split_at = snapshot.spoken_cursor_byte as usize;
                let spoken_prefix = snapshot.response_text[..split_at].to_string();
                let unsaid_remainder = snapshot.response_text[split_at..].trim_start().to_string();
                if let Some(topic_hint) = &snapshot.topic_hint {
                    interrupted_subject_ref = Some(topic_hint.clone());
                }
                if !unsaid_remainder.is_empty() {
                    let expires_at = MonotonicTimeNs(
                        req.now
                            .0
                            .saturating_add((self.config.resume_buffer_ttl_ms as u64) * 1_000_000),
                    );
                    next_state.resume_buffer = Some(ResumeBuffer::v1(
                        snapshot.answer_id,
                        snapshot.topic_hint.clone(),
                        spoken_prefix,
                        unsaid_remainder,
                        expires_at,
                    )?);
                }
            }
            next_state = mark_interrupt_captured_topic(next_state, interrupted_subject_ref);
            match (
                req.interrupt_subject_relation,
                req.interrupt_subject_relation_confidence,
            ) {
                (Some(InterruptSubjectRelation::Same), Some(conf))
                | (Some(InterruptSubjectRelation::Switch), Some(conf))
                    if conf >= INTERRUPT_RELATION_CONFIDENCE_MIN => {}
                _ => {
                    let attempts =
                        bump_attempts(&next_state.pending, PendingKind::Clarify(FieldKey::Task));
                    next_state.pending = Some(PendingState::Clarify {
                        missing_field: FieldKey::Task,
                        attempts,
                    });
                    return self.out_clarify_with_tts_control(
                        req,
                        next_state,
                        reason_codes::X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY,
                        "Should I continue the previous topic or switch to your new topic?"
                            .to_string(),
                        vec![
                            "Continue previous topic".to_string(),
                            "Switch to new topic".to_string(),
                            "Not sure yet".to_string(),
                        ],
                        vec![FieldKey::Task],
                        delivery_base,
                        Some(TtsControl::Cancel),
                    );
                }
            }
            return self.out_wait(
                req,
                next_state,
                reason_codes::X_INTERRUPT_CANCEL,
                Some("interrupted".to_string()),
                Some(TtsControl::Cancel),
            );
        }

        if let Some(prev_active_speaker_user_id) = &base_thread_state.active_speaker_user_id {
            if prev_active_speaker_user_id != &req.active_speaker_user_id {
                let attempts = bump_attempts(
                    &base_thread_state.pending,
                    PendingKind::Clarify(FieldKey::ReferenceTarget),
                );
                let next_state = inherit_thread_scope(
                    ThreadState::v1(
                        Some(PendingState::Clarify {
                            missing_field: FieldKey::ReferenceTarget,
                            attempts,
                        }),
                        base_thread_state.resume_buffer.clone(),
                    ),
                    &base_thread_state,
                )
                .with_continuity(
                    Some(req.subject_ref.clone()),
                    Some(req.active_speaker_user_id.clone()),
                )?;
                return self.out_clarify(
                    req,
                    next_state,
                    reason_codes::X_CONTINUITY_SPEAKER_MISMATCH,
                    "I need to confirm who is speaking before I continue. Please say your name."
                        .to_string(),
                    vec![
                        "It is JD".to_string(),
                        "This is <your name>".to_string(),
                        "Typed: I am JD".to_string(),
                    ],
                    vec![FieldKey::ReferenceTarget],
                    delivery_base,
                );
            }
        }

        if let Some(prev_subject_ref) = &base_thread_state.active_subject_ref {
            if prev_subject_ref != &req.subject_ref && base_thread_state.pending.is_some() {
                let attempts = bump_attempts(
                    &base_thread_state.pending,
                    PendingKind::Clarify(FieldKey::ReferenceTarget),
                );
                let next_state = inherit_thread_scope(
                    ThreadState::v1(
                        Some(PendingState::Clarify {
                            missing_field: FieldKey::ReferenceTarget,
                            attempts,
                        }),
                        base_thread_state.resume_buffer.clone(),
                    ),
                    &base_thread_state,
                )
                .with_continuity(
                    Some(req.subject_ref.clone()),
                    Some(req.active_speaker_user_id.clone()),
                )?;
                return self.out_clarify(
                    req,
                    next_state,
                    reason_codes::X_CONTINUITY_SUBJECT_MISMATCH,
                    format!(
                        "Should I continue '{prev_subject_ref}' or switch to '{}'? ",
                        req.subject_ref
                    ),
                    vec![
                        format!("Continue {prev_subject_ref}"),
                        format!("Switch to {}", req.subject_ref),
                    ],
                    vec![FieldKey::ReferenceTarget],
                    delivery_base,
                );
            }
        }

        // If we are completing a prior tool dispatch, handle the ToolResponse deterministically.
        if let Some(tr) = &req.tool_response {
            return self.decide_from_tool_response(req, tr, base_thread_state, delivery_base);
        }

        // Confirmation answers are handled before NLP, using pending confirm snapshot.
        if let Some(ans) = req.confirm_answer {
            if base_thread_state.return_check_pending {
                return self.decide_from_return_check_answer(
                    req,
                    ans,
                    base_thread_state,
                    delivery_base,
                );
            }
            return self.decide_from_confirm_answer(req, ans, base_thread_state, delivery_base);
        }

        if let Some(step_up_result) = req.step_up_result.as_ref() {
            return self.decide_from_step_up_result(
                req,
                step_up_result,
                base_thread_state,
                delivery_base,
            );
        }

        if let Some(rc) = req.last_failure_reason_code {
            return self.out_respond(
                req,
                clear_pending(base_thread_state),
                reason_codes::X_LAST_FAILURE,
                retry_message_for_failure(rc, None),
                delivery_base,
            );
        }

        let nlp = req
            .nlp_output
            .as_ref()
            .ok_or(ContractViolation::InvalidValue {
                field: "ph1x_request.nlp_output",
                reason:
                    "must be Some(...) when no tool_response/interrupt/last_failure is provided",
            })?;

        match nlp {
            Ph1nResponse::Clarify(c) => {
                let missing_field = c.what_is_missing.first().copied().unwrap_or(FieldKey::Task);
                let attempts = bump_attempts(
                    &base_thread_state.pending,
                    PendingKind::Clarify(missing_field),
                );
                let next_state = inherit_thread_scope(
                    ThreadState::v1(
                        Some(PendingState::Clarify {
                            missing_field,
                            attempts,
                        }),
                        base_thread_state.resume_buffer.clone(),
                    ),
                    &base_thread_state,
                );

                self.out_clarify(
                    req,
                    next_state,
                    reason_codes::X_NLP_CLARIFY,
                    c.question.clone(),
                    c.accepted_answer_formats.clone(),
                    vec![missing_field],
                    delivery_base,
                )
            }
            Ph1nResponse::Chat(ch) => {
                if should_interrupt_relation_clarify(req, &base_thread_state, None) {
                    return self.out_interrupt_relation_uncertain_clarify(
                        req,
                        base_thread_state,
                        delivery_base,
                    );
                }
                let identity_v2 = identity_v2_for_context(&req.identity_context);
                let may_prompt_identity = identity_may_prompt(req, &base_thread_state, identity_v2);
                let allow_personalization =
                    identity_allows_personalization(&req.identity_context, identity_v2);
                let safe_memory = if allow_personalization {
                    filter_fresh_low_risk_candidates(&req.memory_candidates, req.now)
                } else {
                    vec![]
                };

                let mut text = ch.response_text.clone();
                // Minimal, deterministic silent personalization: greeting + preferred_name.
                if let Some(name) = preferred_name(&safe_memory) {
                    if is_greeting_text(&text) {
                        text = format!("Hello, {name}.");
                    }
                }

                let mut next_thread_state = clear_pending(base_thread_state.clone());
                if may_prompt_identity {
                    if let Some(prompt) = identity_confirmation_prompt(&req.identity_context) {
                        text = append_identity_prompt(text, &prompt);
                    }
                    next_thread_state = mark_identity_prompted(
                        next_thread_state,
                        req.now,
                        req.identity_prompt_scope_key.as_deref(),
                    );
                }
                let mut same_subject_merge_applied = false;
                let same_subject_confident = matches!(
                    (req.interrupt_subject_relation, req.interrupt_subject_relation_confidence),
                    (Some(InterruptSubjectRelation::Same), Some(conf))
                        if conf >= INTERRUPT_RELATION_CONFIDENCE_MIN
                );
                let switch_subject_confident = matches!(
                    (req.interrupt_subject_relation, req.interrupt_subject_relation_confidence),
                    (Some(InterruptSubjectRelation::Switch), Some(conf))
                        if conf >= INTERRUPT_RELATION_CONFIDENCE_MIN
                );
                if same_subject_confident {
                    if let Some(rb) = next_thread_state.resume_buffer.take() {
                        text = merge_same_subject_response_text(&rb.unsaid_remainder, &text);
                        same_subject_merge_applied = true;
                        next_thread_state = clear_interrupt_continuity_state(next_thread_state);
                    }
                }
                let mut switch_topic_return_check_applied = false;
                if !same_subject_merge_applied && switch_subject_confident {
                    if next_thread_state.resume_buffer.is_some() {
                        text = append_switch_topic_return_check(text);
                        switch_topic_return_check_applied = true;
                        next_thread_state = mark_return_check_pending(
                            next_thread_state,
                            req.now,
                            self.config.resume_buffer_ttl_ms,
                        );
                    }
                }

                // Sensitive memory requires permission before it is used or cited.
                // When triggered, defer the already-generated response text and ask one permission question.
                if allow_personalization
                    && contains_sensitive_candidate(&req.memory_candidates, req.now)
                {
                    let attempts =
                        bump_attempts(&base_thread_state.pending, PendingKind::MemoryPermission);
                    let next_state = inherit_thread_scope(
                        ThreadState::v1(
                            Some(PendingState::MemoryPermission {
                                deferred_response_text: truncate_to_char_boundary(text, 32_768),
                                attempts,
                            }),
                            next_thread_state.resume_buffer.clone(),
                        ),
                        &base_thread_state,
                    );
                    return self.out_respond(
                        req,
                        next_state,
                        reason_codes::X_MEMORY_PERMISSION_REQUIRED,
                        "This may be sensitive. Do you want me to use it to answer? (Yes / No)"
                            .to_string(),
                        delivery_base,
                    );
                }

                if same_subject_merge_applied {
                    return self.out_respond_with_interrupt_metadata(
                        req,
                        next_thread_state,
                        reason_codes::X_INTERRUPT_SAME_SUBJECT_APPEND,
                        text,
                        delivery_base,
                        Some(InterruptContinuityOutcome::SameSubjectAppend),
                        Some(InterruptResumePolicy::ResumeNow),
                    );
                }
                if switch_topic_return_check_applied {
                    return self.out_respond_with_interrupt_metadata(
                        req,
                        next_thread_state,
                        reason_codes::X_INTERRUPT_RETURN_CHECK_ASKED,
                        text,
                        delivery_base,
                        Some(InterruptContinuityOutcome::SwitchTopicThenReturnCheck),
                        Some(InterruptResumePolicy::ResumeLater),
                    );
                }

                self.out_respond(
                    req,
                    next_thread_state,
                    reason_codes::X_NLP_CHAT,
                    text,
                    delivery_base,
                )
            }
            Ph1nResponse::IntentDraft(d) => {
                self.decide_from_intent(req, d, base_thread_state, delivery_base)
            }
        }
    }

    fn decide_from_intent(
        &self,
        req: &Ph1xRequest,
        d: &IntentDraft,
        mut base_thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        // Resume/Combine (only while the Resume Buffer is valid).
        if matches!(d.intent_type, IntentType::Continue | IntentType::MoreDetail) {
            if let Some(rb) = base_thread_state.resume_buffer.take() {
                base_thread_state.pending = None;
                base_thread_state = clear_interrupt_continuity_state(base_thread_state);

                match d.intent_type {
                    IntentType::Continue => {
                        return self.out_respond_with_resume_policy(
                            req,
                            base_thread_state,
                            reason_codes::X_INTERRUPT_RESUME_NOW,
                            rb.unsaid_remainder,
                            delivery_base,
                            InterruptResumePolicy::ResumeNow,
                        );
                    }
                    IntentType::MoreDetail => {
                        // Deterministic: resume the remainder and (when size allows) acknowledge the request.
                        let mut text = String::new();
                        let prefix = "Sure. Here's more detail.\n";
                        if prefix.len() + rb.unsaid_remainder.len() <= 32_768 {
                            text.push_str(prefix);
                        }
                        text.push_str(&rb.unsaid_remainder);
                        return self.out_respond_with_resume_policy(
                            req,
                            base_thread_state,
                            reason_codes::X_INTERRUPT_RESUME_NOW,
                            text,
                            delivery_base,
                            InterruptResumePolicy::ResumeNow,
                        );
                    }
                    _ => {}
                }
            }

            // Conversation-control with no Resume Buffer: fail closed into a single clarify.
            let missing_field = FieldKey::ReferenceTarget;
            let attempts = bump_attempts(
                &base_thread_state.pending,
                PendingKind::Clarify(missing_field),
            );
            let next_state = inherit_thread_scope(
                ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field,
                        attempts,
                    }),
                    None,
                ),
                &base_thread_state,
            );
            return self.out_clarify(
                req,
                next_state,
                reason_codes::X_RESUME_EXPIRED,
                "What should I continue or add detail to?".to_string(),
                vec![
                    "The last answer".to_string(),
                    "The meeting".to_string(),
                    "The reminder".to_string(),
                ],
                vec![missing_field],
                delivery_base,
            );
        }

        if should_interrupt_relation_clarify(req, &base_thread_state, Some(d)) {
            return self.out_interrupt_relation_uncertain_clarify(
                req,
                base_thread_state,
                delivery_base,
            );
        }

        let identity_v2 = identity_v2_for_context(&req.identity_context);
        let allow_personalization =
            identity_allows_personalization(&req.identity_context, identity_v2);
        let draft_with_memory_context = if allow_personalization {
            hydrate_invite_link_contact_from_memory(d, &req.memory_candidates, req.now)?
        } else {
            d.clone()
        };
        let d = &draft_with_memory_context;

        if let Some(clarify) = clarify_for_invite_link_recipient_resolution(d)? {
            let missing_field = clarify.what_is_missing[0];
            let attempts = bump_attempts(
                &base_thread_state.pending,
                PendingKind::Clarify(missing_field),
            );
            let next_state = inherit_thread_scope(
                ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
                ),
                &base_thread_state,
            );
            return self.out_clarify(
                req,
                next_state,
                reason_codes::X_NLP_CLARIFY,
                clarify.question,
                clarify.accepted_answer_formats,
                clarify.what_is_missing,
                delivery_base,
            );
        }

        if d.overall_confidence != OverallConfidence::High || !d.required_fields_missing.is_empty()
        {
            let clarify = clarify_for_missing(d.intent_type, &d.required_fields_missing)?;
            let missing_field = clarify.what_is_missing[0];
            let attempts = bump_attempts(
                &base_thread_state.pending,
                PendingKind::Clarify(missing_field),
            );
            let next_state = inherit_thread_scope(
                ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
                ),
                &base_thread_state,
            );
            return self.out_clarify(
                req,
                next_state,
                reason_codes::X_NLP_CLARIFY,
                clarify.question,
                clarify.accepted_answer_formats,
                clarify.what_is_missing,
                delivery_base,
            );
        }

        // Read-only tool dispatch (PH1.E).
        if matches!(
            d.intent_type,
            IntentType::TimeQuery
                | IntentType::WeatherQuery
                | IntentType::WebSearchQuery
                | IntentType::NewsQuery
                | IntentType::UrlFetchAndCiteQuery
                | IntentType::DocumentUnderstandQuery
                | IntentType::PhotoUnderstandQuery
                | IntentType::DataAnalysisQuery
                | IntentType::DeepResearchQuery
                | IntentType::RecordModeQuery
                | IntentType::ConnectorQuery
                | IntentType::ListReminders
        ) {
            let (tool_name, query) = match d.intent_type {
                IntentType::TimeQuery => (
                    ToolName::Time,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::WeatherQuery => (
                    ToolName::Weather,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::WebSearchQuery => (
                    ToolName::WebSearch,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::NewsQuery => (
                    ToolName::News,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::UrlFetchAndCiteQuery => (
                    ToolName::UrlFetchAndCite,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::DocumentUnderstandQuery => (
                    ToolName::DocumentUnderstand,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::PhotoUnderstandQuery => (
                    ToolName::PhotoUnderstand,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::DataAnalysisQuery => (
                    ToolName::DataAnalysis,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::DeepResearchQuery => (
                    ToolName::DeepResearch,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::RecordModeQuery => (
                    ToolName::RecordMode,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::ConnectorQuery => (
                    ToolName::ConnectorQuery,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                IntentType::ListReminders => (
                    ToolName::ConnectorQuery,
                    intent_query_text_with_thread_context(d, &base_thread_state),
                ),
                _ => unreachable!("match guarded above"),
            };

            let tool_request = self.tool_request(req, tool_name, query)?;
            let request_id = tool_request.request_id;
            let attempts = bump_attempts(&base_thread_state.pending, PendingKind::Tool(request_id));
            let next_state = inherit_thread_scope(
                ThreadState::v1(
                    Some(PendingState::Tool {
                        request_id,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
                ),
                &base_thread_state,
            );
            return self.out_dispatch_tool(
                req,
                next_state,
                reason_codes::X_DISPATCH_TOOL,
                tool_request,
                delivery_base,
            );
        }

        // Memory control: remember/query are low-impact and dispatch directly;
        // forget remains confirm-gated.
        if matches!(
            d.intent_type,
            IntentType::MemoryRememberRequest | IntentType::MemoryQuery
        ) {
            return self.out_dispatch_simulation_candidate(
                req,
                clear_pending(base_thread_state),
                reason_codes::X_DISPATCH_SIMULATION_CANDIDATE,
                d.clone(),
                delivery_base,
            );
        }

        // v1 rule: impactful intents require a confirm snapshot, then a later confirm_answer drives dispatch.
        let attempts = bump_attempts(
            &base_thread_state.pending,
            PendingKind::Confirm(d.intent_type),
        );
        let next_state = inherit_thread_scope(
            ThreadState::v1(
                Some(PendingState::Confirm {
                    intent_draft: confirm_snapshot_intent_draft(d),
                    attempts,
                }),
                base_thread_state.resume_buffer.clone(),
            ),
            &base_thread_state,
        );
        self.out_confirm(
            req,
            next_state,
            reason_codes::X_CONFIRM_REQUIRED,
            confirm_text(d),
            delivery_base,
        )
    }

    fn decide_from_confirm_answer(
        &self,
        req: &Ph1xRequest,
        ans: selene_kernel_contracts::ph1x::ConfirmAnswer,
        base_thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        if base_thread_state.return_check_pending {
            return self.decide_from_return_check_answer(
                req,
                ans,
                base_thread_state,
                delivery_base,
            );
        }
        let pending = base_thread_state.pending.clone();
        match pending {
            Some(PendingState::Confirm { intent_draft, attempts }) => match ans {
                selene_kernel_contracts::ph1x::ConfirmAnswer::Yes => {
                    if let Some((action_class, requested_action)) =
                        high_stakes_policy_binding(&intent_draft)
                    {
                        let challenge_method = select_step_up_challenge(
                            req.step_up_capabilities
                                .unwrap_or(StepUpCapabilities::v1(false, true)),
                        );
                        if let Some(challenge_method) = challenge_method {
                            let step_up_snapshot = confirm_snapshot_intent_draft(&intent_draft);
                            let next_attempts = bump_attempts(
                                &base_thread_state.pending,
                                PendingKind::StepUp(requested_action),
                            );
                            let next_state = inherit_thread_scope(
                                ThreadState::v1(
                                    Some(PendingState::StepUp {
                                        intent_draft: step_up_snapshot.clone(),
                                        requested_action: requested_action.to_string(),
                                        challenge_method,
                                        attempts: next_attempts,
                                    }),
                                    base_thread_state.resume_buffer.clone(),
                                ),
                                &base_thread_state,
                            );
                            return self.out_dispatch_access_step_up(
                                req,
                                next_state,
                                reason_codes::X_STEPUP_REQUIRED_DISPATCH,
                                step_up_snapshot,
                                action_class,
                                requested_action.to_string(),
                                challenge_method,
                                delivery_base,
                            );
                        }
                        return self.out_wait(
                            req,
                            clear_pending(base_thread_state),
                            reason_codes::X_STEPUP_CHALLENGE_UNAVAILABLE,
                            Some("step_up_challenge_unavailable".to_string()),
                            None,
                        );
                    }

                    let next_state = clear_pending(base_thread_state);
                    self.out_dispatch_simulation_candidate(
                        req,
                        next_state,
                        reason_codes::X_CONFIRM_YES_DISPATCH,
                        intent_draft,
                        delivery_base,
                    )
                }
                selene_kernel_contracts::ph1x::ConfirmAnswer::No => {
                    let next_state = clear_pending(base_thread_state);
                    let msg = if attempts <= 1 {
                        "Okay — I won’t do that.".to_string()
                    } else {
                        "Okay — I won’t do it.".to_string()
                    };
                    self.out_respond(
                        req,
                        next_state,
                        reason_codes::X_CONFIRM_NO_ABORT,
                        msg,
                        delivery_base,
                    )
                }
            },
            Some(PendingState::MemoryPermission {
                deferred_response_text,
                attempts,
            }) => {
                let next_state = clear_pending(base_thread_state);
                match ans {
                    selene_kernel_contracts::ph1x::ConfirmAnswer::Yes => self.out_respond(
                        req,
                        next_state,
                        reason_codes::X_MEMORY_PERMISSION_YES,
                        deferred_response_text.clone(),
                        delivery_base,
                    ),
                    selene_kernel_contracts::ph1x::ConfirmAnswer::No => {
                        let prefix = if attempts <= 1 {
                            "Okay — I won’t use it. "
                        } else {
                            "Okay — I won’t. "
                        };
                        let mut text = String::new();
                        text.push_str(prefix);
                        text.push_str(&deferred_response_text);
                        self.out_respond(
                            req,
                            next_state,
                            reason_codes::X_MEMORY_PERMISSION_NO,
                            truncate_to_char_boundary(text, 32_768),
                            delivery_base,
                        )
                    }
                }
            }
            _ => Err(ContractViolation::InvalidValue {
                field: "ph1x_request.confirm_answer",
                reason:
                    "confirm_answer is only valid when thread_state.pending is Confirm or MemoryPermission",
            }),
        }
    }

    fn decide_from_return_check_answer(
        &self,
        req: &Ph1xRequest,
        ans: selene_kernel_contracts::ph1x::ConfirmAnswer,
        mut base_thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        if !base_thread_state.return_check_pending {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.thread_state.return_check_pending",
                reason: "must be true when handling return-check confirm answer",
            });
        }
        base_thread_state.pending = None;
        match ans {
            selene_kernel_contracts::ph1x::ConfirmAnswer::Yes => {
                let resume_text = match base_thread_state.resume_buffer.take() {
                    Some(rb) => rb.unsaid_remainder,
                    None => {
                        return self.out_clarify(
                            req,
                            clear_interrupt_continuity_state(base_thread_state),
                            reason_codes::X_RESUME_EXPIRED,
                            "What should I continue from the previous topic?".to_string(),
                            vec![
                                "Continue previous topic".to_string(),
                                "Stay on new topic".to_string(),
                            ],
                            vec![FieldKey::ReferenceTarget],
                            delivery_base,
                        );
                    }
                };
                let next_state = clear_interrupt_continuity_state(base_thread_state);
                self.out_respond_with_resume_policy(
                    req,
                    next_state,
                    reason_codes::X_INTERRUPT_RESUME_NOW,
                    resume_text,
                    delivery_base,
                    InterruptResumePolicy::ResumeNow,
                )
            }
            selene_kernel_contracts::ph1x::ConfirmAnswer::No => {
                base_thread_state.resume_buffer = None;
                let next_state = clear_interrupt_continuity_state(base_thread_state);
                self.out_respond_with_resume_policy(
                    req,
                    next_state,
                    reason_codes::X_INTERRUPT_DISCARD,
                    "Okay. I will keep focus on the new topic only.".to_string(),
                    delivery_base,
                    InterruptResumePolicy::Discard,
                )
            }
        }
    }

    fn decide_from_tool_response(
        &self,
        req: &Ph1xRequest,
        tr: &ToolResponse,
        base_thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let expected = match &base_thread_state.pending {
            Some(PendingState::Tool { request_id, .. }) => *request_id,
            _ => {
                return Err(ContractViolation::InvalidValue {
                    field: "ph1x_request.thread_state.pending",
                    reason: "must be PendingState::Tool when tool_response is provided",
                });
            }
        };

        if tr.request_id != expected {
            return Err(ContractViolation::InvalidValue {
                field: "ph1x_request.tool_response.request_id",
                reason: "must match the pending tool request_id",
            });
        }

        if let Some(a) = &tr.ambiguity {
            if is_deterministic_weather_request(req) || is_deterministic_weather_tool_response(tr) {
                return self.out_deterministic_weather_clarification(
                    req,
                    base_thread_state,
                    a.clone(),
                    delivery_base,
                );
            }
            return self.out_tool_ambiguity(req, base_thread_state, a.clone(), delivery_base);
        }

        match tr.tool_status {
            ToolStatus::Ok => {
                let text = tool_ok_text_for_request(req, tr);
                self.out_respond(
                    req,
                    clear_completed_public_deterministic_clarification(clear_pending(
                        base_thread_state,
                    )),
                    reason_codes::X_TOOL_OK,
                    text,
                    delivery_base,
                )
            }
            ToolStatus::Fail => {
                let text = retry_message_for_failure(
                    tr.fail_reason_code.unwrap_or(tr.reason_code),
                    tr.fail_detail.as_deref(),
                );
                if is_deterministic_time_clarification_fail_detail(tr.fail_detail.as_deref()) {
                    return self.out_deterministic_time_clarification(
                        req,
                        base_thread_state,
                        text,
                        delivery_base,
                    );
                }
                if is_deterministic_weather_clarification_fail_detail(tr.fail_detail.as_deref()) {
                    return self.out_deterministic_weather_missing_clarification(
                        req,
                        base_thread_state,
                        text,
                        delivery_base,
                    );
                }
                self.out_respond(
                    req,
                    clear_pending(base_thread_state),
                    reason_codes::X_TOOL_FAIL,
                    text,
                    delivery_base,
                )
            }
        }
    }

    fn out_deterministic_time_clarification(
        &self,
        req: &Ph1xRequest,
        mut thread_state: ThreadState,
        question: String,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let attempts = bump_attempts(&thread_state.pending, PendingKind::Clarify(FieldKey::Place));
        thread_state.pending = Some(PendingState::Clarify {
            missing_field: FieldKey::Place,
            attempts,
        });
        thread_state.resume_buffer = Some(ResumeBuffer::v1(
            deterministic_time_clarification_answer_id(req),
            Some(DETERMINISTIC_TIME_CLARIFICATION_TOPIC.to_string()),
            question.clone(),
            "Awaiting a place, city, country, or region for the pending deterministic time query."
                .to_string(),
            MonotonicTimeNs(
                req.now
                    .0
                    .saturating_add((self.config.resume_buffer_ttl_ms as u64) * 1_000_000),
            ),
        )?);
        self.out_clarify(
            req,
            thread_state,
            reason_codes::X_TOOL_FAIL,
            question,
            vec![
                "A city, e.g. Madrid".to_string(),
                "A region, e.g. Canary Islands".to_string(),
                "A local place, e.g. Hobart".to_string(),
            ],
            vec![FieldKey::Place],
            delivery_base,
        )
    }

    fn out_deterministic_weather_missing_clarification(
        &self,
        req: &Ph1xRequest,
        mut thread_state: ThreadState,
        question: String,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let attempts = bump_attempts(&thread_state.pending, PendingKind::Clarify(FieldKey::Place));
        thread_state.pending = Some(PendingState::Clarify {
            missing_field: FieldKey::Place,
            attempts,
        });
        thread_state.resume_buffer = Some(ResumeBuffer::v1(
            deterministic_weather_clarification_answer_id(req),
            Some(DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC.to_string()),
            question.clone(),
            "Awaiting a city or exact local place for the pending weather query.".to_string(),
            MonotonicTimeNs(
                req.now
                    .0
                    .saturating_add((self.config.resume_buffer_ttl_ms as u64) * 1_000_000),
            ),
        )?);
        self.out_clarify(
            req,
            thread_state,
            reason_codes::X_TOOL_FAIL,
            question,
            vec![
                "A city, e.g. Lisbon".to_string(),
                "A city and region, e.g. Springfield, Illinois".to_string(),
                "A local place, e.g. Madrid".to_string(),
            ],
            vec![FieldKey::Place],
            delivery_base,
        )
    }

    fn out_deterministic_weather_clarification(
        &self,
        req: &Ph1xRequest,
        mut thread_state: ThreadState,
        amb: StructuredAmbiguity,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let attempts = bump_attempts(&thread_state.pending, PendingKind::Clarify(FieldKey::Place));
        let formats: Vec<String> = amb
            .alternatives
            .iter()
            .take(3)
            .map(|a| a.to_string())
            .collect();
        let accepted = if formats.is_empty() {
            vec![
                "A city, e.g. Lisbon".to_string(),
                "A city and region, e.g. Springfield, Illinois".to_string(),
                "A local place, e.g. Madrid".to_string(),
            ]
        } else {
            formats
        };
        let option_text = weather_clarification_options(&accepted);
        let question = if option_text.is_empty() {
            format!("{} Which place should I use?", amb.summary)
        } else {
            format!("{} Which place should I use? {}.", amb.summary, option_text)
        };
        thread_state.pending = Some(PendingState::Clarify {
            missing_field: FieldKey::Place,
            attempts,
        });
        thread_state.resume_buffer = Some(ResumeBuffer::v1(
            deterministic_weather_clarification_answer_id(req),
            Some(DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC.to_string()),
            question.clone(),
            "Awaiting a city or exact local place for the pending weather query.".to_string(),
            MonotonicTimeNs(
                req.now
                    .0
                    .saturating_add((self.config.resume_buffer_ttl_ms as u64) * 1_000_000),
            ),
        )?);
        self.out_clarify(
            req,
            thread_state,
            reason_codes::X_TOOL_AMBIGUOUS,
            question,
            accepted,
            vec![FieldKey::Place],
            delivery_base,
        )
    }

    fn decide_from_step_up_result(
        &self,
        req: &Ph1xRequest,
        result: &StepUpResult,
        base_thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let pending = base_thread_state.pending.clone();
        match pending {
            Some(PendingState::StepUp { intent_draft, .. }) => match result.outcome {
                StepUpOutcome::Continue => self.out_dispatch_simulation_candidate(
                    req,
                    clear_pending(base_thread_state),
                    reason_codes::X_STEPUP_CONTINUE_DISPATCH,
                    intent_draft,
                    delivery_base,
                ),
                StepUpOutcome::Refuse => self.out_respond(
                    req,
                    clear_pending(base_thread_state),
                    reason_codes::X_STEPUP_REFUSED,
                    "I can’t continue with that action without verification.".to_string(),
                    delivery_base,
                ),
                StepUpOutcome::Defer => self.out_wait(
                    req,
                    clear_pending(base_thread_state),
                    reason_codes::X_STEPUP_DEFERRED,
                    Some("step_up_deferred".to_string()),
                    None,
                ),
            },
            _ => Err(ContractViolation::InvalidValue {
                field: "ph1x_request.step_up_result",
                reason: "step_up_result is only valid when thread_state.pending is StepUp",
            }),
        }
    }

    fn tool_request(
        &self,
        req: &Ph1xRequest,
        tool_name: ToolName,
        query: String,
    ) -> Result<ToolRequest, ContractViolation> {
        let budget = StrictBudget::new(self.config.tool_timeout_ms, self.config.tool_max_results)?;
        ToolRequest::v1(
            ToolRequestOrigin::Ph1X,
            tool_name,
            query,
            req.locale.clone(),
            budget,
            req.policy_context_ref,
        )
    }

    fn out_confirm(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        text: String,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Confirm(ConfirmDirective::v1(text)?);
        self.out(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
        )
    }

    fn out_respond(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        response_text: String,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Respond(
            selene_kernel_contracts::ph1x::RespondDirective::v1(response_text)?,
        );
        self.out_with_interrupt_metadata(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
            None,
            None,
        )
    }

    fn out_respond_with_interrupt_metadata(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        response_text: String,
        delivery_base: DeliveryHint,
        interrupt_continuity_outcome: Option<InterruptContinuityOutcome>,
        interrupt_resume_policy: Option<InterruptResumePolicy>,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Respond(
            selene_kernel_contracts::ph1x::RespondDirective::v1(response_text)?,
        );
        self.out_with_interrupt_metadata(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
            interrupt_continuity_outcome,
            interrupt_resume_policy,
        )
    }

    fn out_respond_with_resume_policy(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        response_text: String,
        delivery_base: DeliveryHint,
        interrupt_resume_policy: InterruptResumePolicy,
    ) -> Result<Ph1xResponse, ContractViolation> {
        self.out_respond_with_interrupt_metadata(
            req,
            thread_state,
            reason_code,
            response_text,
            delivery_base,
            None,
            Some(interrupt_resume_policy),
        )
    }

    fn out_clarify(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        question: String,
        accepted_answer_formats: Vec<String>,
        what_is_missing: Vec<FieldKey>,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        self.out_clarify_with_tts_control(
            req,
            thread_state,
            reason_code,
            question,
            accepted_answer_formats,
            what_is_missing,
            delivery_base,
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn out_clarify_with_tts_control(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        question: String,
        accepted_answer_formats: Vec<String>,
        what_is_missing: Vec<FieldKey>,
        delivery_base: DeliveryHint,
        tts_control: Option<TtsControl>,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Clarify(ClarifyDirective::v1(
            question,
            accepted_answer_formats,
            what_is_missing,
        )?);
        self.out(
            req,
            directive,
            thread_state,
            tts_control,
            delivery_base,
            reason_code,
        )
    }

    fn out_interrupt_relation_uncertain_clarify(
        &self,
        req: &Ph1xRequest,
        mut thread_state: ThreadState,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let attempts = bump_attempts(&thread_state.pending, PendingKind::Clarify(FieldKey::Task));
        thread_state.pending = Some(PendingState::Clarify {
            missing_field: FieldKey::Task,
            attempts,
        });
        self.out_clarify(
            req,
            thread_state,
            reason_codes::X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY,
            "Should I continue the previous topic or switch to your new topic?".to_string(),
            vec![
                "Continue previous topic".to_string(),
                "Switch to new topic".to_string(),
                "Not sure yet".to_string(),
            ],
            vec![FieldKey::Task],
            delivery_base,
        )
    }

    fn out_dispatch_tool(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        tool_request: ToolRequest,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Dispatch(DispatchDirective::tool_v1(tool_request)?);
        self.out(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
        )
    }

    fn out_dispatch_simulation_candidate(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        intent_draft: IntentDraft,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive =
            Ph1xDirective::Dispatch(DispatchDirective::simulation_candidate_v1(intent_draft)?);
        self.out(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn out_dispatch_access_step_up(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        intent_draft: IntentDraft,
        action_class: StepUpActionClass,
        requested_action: String,
        challenge_method: StepUpChallengeMethod,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Dispatch(DispatchDirective::access_step_up_v1(
            intent_draft,
            action_class,
            requested_action,
            challenge_method,
        )?);
        self.out(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
        )
    }

    fn out_wait(
        &self,
        req: &Ph1xRequest,
        thread_state: ThreadState,
        reason_code: ReasonCodeId,
        reason: Option<String>,
        tts_control: Option<TtsControl>,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let directive = Ph1xDirective::Wait(WaitDirective::v1(reason)?);
        // Wait is silent by definition; do not request audible output.
        self.out(
            req,
            directive,
            thread_state,
            tts_control,
            DeliveryHint::Silent,
            reason_code,
        )
    }

    fn out_tool_ambiguity(
        &self,
        req: &Ph1xRequest,
        base_thread_state: ThreadState,
        amb: StructuredAmbiguity,
        delivery_base: DeliveryHint,
    ) -> Result<Ph1xResponse, ContractViolation> {
        // Deterministic: ask one question and offer up to 3 stable options.
        let formats: Vec<String> = amb
            .alternatives
            .iter()
            .take(3)
            .map(|a| a.to_string())
            .collect();

        let accepted = if formats.len() >= 2 {
            formats
        } else {
            vec!["Option A".to_string(), "Option B".to_string()]
        };

        let attempts = bump_attempts(
            &base_thread_state.pending,
            PendingKind::Clarify(FieldKey::IntentChoice),
        );
        let next_state = inherit_thread_scope(
            ThreadState::v1(
                Some(PendingState::Clarify {
                    missing_field: FieldKey::IntentChoice,
                    attempts,
                }),
                base_thread_state.resume_buffer.clone(),
            ),
            &base_thread_state,
        );

        self.out_clarify(
            req,
            next_state,
            reason_codes::X_TOOL_AMBIGUOUS,
            format!("{} Which one should I use?", amb.summary),
            accepted,
            vec![FieldKey::IntentChoice],
            delivery_base,
        )
    }

    fn out(
        &self,
        req: &Ph1xRequest,
        directive: Ph1xDirective,
        thread_state: ThreadState,
        tts_control: Option<TtsControl>,
        delivery_hint: DeliveryHint,
        reason_code: ReasonCodeId,
    ) -> Result<Ph1xResponse, ContractViolation> {
        self.out_with_interrupt_metadata(
            req,
            directive,
            thread_state,
            tts_control,
            delivery_hint,
            reason_code,
            None,
            None,
        )
    }

    fn out_with_interrupt_metadata(
        &self,
        req: &Ph1xRequest,
        directive: Ph1xDirective,
        thread_state: ThreadState,
        tts_control: Option<TtsControl>,
        delivery_hint: DeliveryHint,
        reason_code: ReasonCodeId,
        interrupt_continuity_outcome: Option<InterruptContinuityOutcome>,
        interrupt_resume_policy: Option<InterruptResumePolicy>,
    ) -> Result<Ph1xResponse, ContractViolation> {
        let delivery = match delivery_hint {
            DeliveryHint::Silent => DeliveryHint::Silent,
            _ => delivery_hint_from_base(directive_kind(&directive), delivery_hint),
        };

        let thread_state = thread_state.with_continuity(
            Some(req.subject_ref.clone()),
            Some(req.active_speaker_user_id.clone()),
        )?;

        let idempotency_key = Some(make_idempotency_key(req, directive_kind(&directive)));
        let out = Ph1xResponse::v1(
            req.correlation_id,
            req.turn_id,
            directive,
            thread_state,
            tts_control,
            delivery,
            reason_code,
            idempotency_key,
        )?
        .with_interrupt_continuity_outcome(interrupt_continuity_outcome)?
        .with_interrupt_resume_policy(interrupt_resume_policy)?;
        out.validate()?;
        Ok(out)
    }
}

fn delivery_hint_from_base(kind: DirectiveKind, base: DeliveryHint) -> DeliveryHint {
    match kind {
        DirectiveKind::Wait => DeliveryHint::Silent,
        _ => base,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirectiveKind {
    Confirm,
    Clarify,
    Respond,
    Dispatch,
    Wait,
}

fn directive_kind(d: &Ph1xDirective) -> DirectiveKind {
    match d {
        Ph1xDirective::Confirm(_) => DirectiveKind::Confirm,
        Ph1xDirective::Clarify(_) => DirectiveKind::Clarify,
        Ph1xDirective::Respond(_) => DirectiveKind::Respond,
        Ph1xDirective::Dispatch(_) => DirectiveKind::Dispatch,
        Ph1xDirective::Wait(_) => DirectiveKind::Wait,
    }
}

fn make_idempotency_key(req: &Ph1xRequest, kind: DirectiveKind) -> String {
    let k = match kind {
        DirectiveKind::Confirm => "confirm",
        DirectiveKind::Clarify => "clarify",
        DirectiveKind::Respond => "respond",
        DirectiveKind::Dispatch => "dispatch",
        DirectiveKind::Wait => "wait",
    };
    // Deterministic and bounded.
    format!("x:{:032x}:{:016x}:{k}", req.correlation_id, req.turn_id)
}

fn deterministic_time_clarification_answer_id(req: &Ph1xRequest) -> AnswerId {
    AnswerId(
        req.correlation_id
            .wrapping_add(u128::from(req.turn_id))
            .max(1),
    )
}

fn deterministic_weather_clarification_answer_id(req: &Ph1xRequest) -> AnswerId {
    AnswerId(
        req.correlation_id
            .wrapping_add(u128::from(req.turn_id))
            .wrapping_add(364)
            .max(1),
    )
}

fn is_deterministic_weather_request(req: &Ph1xRequest) -> bool {
    matches!(
        req.nlp_output.as_ref(),
        Some(Ph1nResponse::IntentDraft(d)) if d.intent_type == IntentType::WeatherQuery
    )
}

fn is_deterministic_weather_tool_response(tr: &ToolResponse) -> bool {
    matches!(tr.tool_result.as_ref(), Some(ToolResult::Weather { .. }))
}

fn is_deterministic_time_clarification_fail_detail(fail_detail: Option<&str>) -> bool {
    fail_detail.is_some_and(|detail| {
        detail.contains("missing_time_location")
            || detail.contains("ambiguous_time_location")
            || detail.contains("unsupported_time_location")
    })
}

fn is_deterministic_weather_clarification_fail_detail(fail_detail: Option<&str>) -> bool {
    fail_detail.is_some_and(|detail| detail.contains("weather_query_missing_place"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Clarify(FieldKey),
    Confirm(IntentType),
    MemoryPermission,
    StepUp(&'static str),
    Tool(selene_kernel_contracts::ph1e::ToolRequestId),
}

fn bump_attempts(prev: &Option<PendingState>, next: PendingKind) -> u8 {
    match (prev, next) {
        (
            Some(PendingState::Clarify {
                missing_field,
                attempts,
            }),
            PendingKind::Clarify(k),
        ) if *missing_field == k => attempts.saturating_add(1).min(10),
        (
            Some(PendingState::Confirm {
                intent_draft,
                attempts,
            }),
            PendingKind::Confirm(it),
        ) if intent_draft.intent_type == it => attempts.saturating_add(1).min(10),
        (Some(PendingState::MemoryPermission { attempts, .. }), PendingKind::MemoryPermission) => {
            attempts.saturating_add(1).min(10)
        }
        (
            Some(PendingState::StepUp {
                requested_action,
                attempts,
                ..
            }),
            PendingKind::StepUp(next_action),
        ) if requested_action == next_action => attempts.saturating_add(1).min(10),
        (
            Some(PendingState::Tool {
                request_id,
                attempts,
            }),
            PendingKind::Tool(id),
        ) if *request_id == id => attempts.saturating_add(1).min(10),
        _ => 1,
    }
}

fn clear_pending(mut s: ThreadState) -> ThreadState {
    s.pending = None;
    s
}

fn clear_completed_public_deterministic_clarification(s: ThreadState) -> ThreadState {
    clear_completed_weather_clarification(clear_completed_time_clarification(s))
}

fn clear_completed_time_clarification(mut s: ThreadState) -> ThreadState {
    if has_deterministic_time_clarification_resume(&s) {
        s.resume_buffer = None;
    }
    s
}

fn clear_completed_weather_clarification(mut s: ThreadState) -> ThreadState {
    if has_deterministic_weather_clarification_resume(&s) {
        s.resume_buffer = None;
    }
    s
}

fn has_deterministic_time_clarification_resume(s: &ThreadState) -> bool {
    s.resume_buffer
        .as_ref()
        .and_then(|buffer| buffer.topic_hint.as_deref())
        == Some(DETERMINISTIC_TIME_CLARIFICATION_TOPIC)
}

fn has_deterministic_weather_clarification_resume(s: &ThreadState) -> bool {
    s.resume_buffer
        .as_ref()
        .and_then(|buffer| buffer.topic_hint.as_deref())
        == Some(DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC)
}

fn inherit_thread_scope(mut next: ThreadState, base: &ThreadState) -> ThreadState {
    next.project_id = base.project_id.clone();
    next.pinned_context_refs = base.pinned_context_refs.clone();
    next.thread_policy_flags = base.thread_policy_flags;
    next
}

fn clear_expired_resume_buffer(mut s: ThreadState, now: MonotonicTimeNs) -> ThreadState {
    if let Some(b) = &s.resume_buffer {
        if now.0 >= b.expires_at.0 {
            s.resume_buffer = None;
            s = clear_interrupt_continuity_state(s);
        }
    }
    if let Some(expires_at) = s.return_check_expires_at {
        if now.0 >= expires_at.0 {
            s.return_check_pending = false;
            s.return_check_expires_at = None;
        }
    }
    s
}

fn clear_interrupt_continuity_state(mut s: ThreadState) -> ThreadState {
    s.interrupted_subject_ref = None;
    s.return_check_pending = false;
    s.return_check_expires_at = None;
    s
}

fn mark_interrupt_captured_topic(
    mut s: ThreadState,
    interrupted_subject_ref: Option<String>,
) -> ThreadState {
    s.interrupted_subject_ref = interrupted_subject_ref;
    s.return_check_pending = false;
    s.return_check_expires_at = None;
    s
}

fn mark_return_check_pending(mut s: ThreadState, now: MonotonicTimeNs, ttl_ms: u32) -> ThreadState {
    if s.interrupted_subject_ref.is_none() {
        s.interrupted_subject_ref = s
            .resume_buffer
            .as_ref()
            .and_then(|rb| rb.topic_hint.clone())
            .or_else(|| s.active_subject_ref.clone());
    }
    if s.resume_buffer.is_some() {
        s.return_check_pending = true;
        s.return_check_expires_at = Some(MonotonicTimeNs(
            now.0.saturating_add((ttl_ms as u64) * 1_000_000),
        ));
    } else {
        s.return_check_pending = false;
        s.return_check_expires_at = None;
    }
    s
}

fn identity_allows_personalization(id: &IdentityContext, identity_v2: VoiceIdentityV2) -> bool {
    match id {
        IdentityContext::TextUserId(_) => true,
        IdentityContext::Voice(_) => identity_v2.identity_tier_v2 == IdentityTierV2::Confirmed,
    }
}

fn identity_v2_for_context(id: &IdentityContext) -> VoiceIdentityV2 {
    match id {
        IdentityContext::TextUserId(_) => VoiceIdentityV2 {
            identity_tier_v2: IdentityTierV2::Confirmed,
            may_prompt_identity: false,
        },
        IdentityContext::Voice(v) => v.identity_v2(),
    }
}

fn identity_may_prompt(
    req: &Ph1xRequest,
    base_thread_state: &ThreadState,
    identity_v2: VoiceIdentityV2,
) -> bool {
    if !matches!(req.identity_context, IdentityContext::Voice(_)) {
        return false;
    }
    if !identity_v2.may_prompt_identity {
        return false;
    }
    if identity_v2.identity_tier_v2 == IdentityTierV2::Confirmed {
        return false;
    }
    if matches!(base_thread_state.pending, Some(PendingState::StepUp { .. })) {
        return false;
    }
    if let Some(prompt_state) = base_thread_state.identity_prompt_state.as_ref() {
        let scope_matches = identity_prompt_scope_matches(
            prompt_state.prompt_scope_key.as_deref(),
            req.identity_prompt_scope_key.as_deref(),
        );
        if scope_matches {
            if let Some(last_prompt_at) = prompt_state.last_prompt_at {
                let cooldown_until = last_prompt_at.0.saturating_add(IDENTITY_PROMPT_COOLDOWN_NS);
                if req.now.0 < cooldown_until {
                    if prompt_state.prompted_in_session {
                        return false;
                    }
                    if prompt_state.prompts_in_scope >= IDENTITY_PROMPT_RETRY_BUDGET {
                        return false;
                    }
                    return false;
                }
            }
        }
    }
    true
}

fn mark_identity_prompted(
    mut thread_state: ThreadState,
    now: MonotonicTimeNs,
    scope_key: Option<&str>,
) -> ThreadState {
    let scope_key = scope_key.map(str::to_string);
    let mut prompts_in_scope = 1u8;
    if let Some(prev) = thread_state.identity_prompt_state.clone() {
        let scope_matches =
            identity_prompt_scope_matches(prev.prompt_scope_key.as_deref(), scope_key.as_deref());
        let cooldown_active = prev
            .last_prompt_at
            .map(|last| now.0 < last.0.saturating_add(IDENTITY_PROMPT_COOLDOWN_NS))
            .unwrap_or(false);
        if scope_matches && cooldown_active {
            prompts_in_scope = prev.prompts_in_scope.saturating_add(1);
        }
    }
    if let Ok(state) =
        IdentityPromptState::v1_with_scope(true, Some(now), scope_key, prompts_in_scope)
    {
        thread_state.identity_prompt_state = Some(state);
    }
    thread_state
}

fn identity_prompt_scope_matches(previous: Option<&str>, current: Option<&str>) -> bool {
    match (previous, current) {
        (Some(a), Some(b)) => a == b,
        (None, None) => true,
        _ => false,
    }
}

fn identity_confirmation_prompt(id: &IdentityContext) -> Option<String> {
    match id {
        IdentityContext::TextUserId(_) => None,
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionOk(ok)) => ok
            .user_id
            .as_ref()
            .map(|u| format!("Quick check: is this {}?", u.as_str()))
            .or_else(|| Some("Quick check: can you confirm this is you?".to_string())),
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionUnknown(u)) => u
            .candidate_user_id
            .as_ref()
            .map(|cand| format!("Quick check: is this {}?", cand.as_str()))
            .or_else(|| Some("Quick check: can you confirm this is you?".to_string())),
    }
}

fn append_identity_prompt(text: String, prompt: &str) -> String {
    if text.trim().is_empty() {
        return prompt.to_string();
    }
    if text.len().saturating_add(prompt.len()).saturating_add(1) > 32_768 {
        return text;
    }
    format!("{text} {prompt}")
}

fn high_stakes_policy_binding(
    intent_draft: &IntentDraft,
) -> Option<(StepUpActionClass, &'static str)> {
    match intent_draft.intent_type {
        IntentType::SendMoney => Some((StepUpActionClass::Payments, "PAYMENT_EXECUTE")),
        IntentType::CapreqManage => {
            Some((StepUpActionClass::CapabilityGovernance, "CAPREQ_MANAGE"))
        }
        IntentType::AccessSchemaManage => {
            Some((StepUpActionClass::AccessGovernance, "ACCESS_SCHEMA_MANAGE"))
        }
        IntentType::AccessEscalationVote => Some((
            StepUpActionClass::AccessGovernance,
            "ACCESS_ESCALATION_VOTE",
        )),
        IntentType::AccessInstanceCompileRefresh => Some((
            StepUpActionClass::AccessGovernance,
            "ACCESS_INSTANCE_COMPILE_REFRESH",
        )),
        _ => None,
    }
}

fn select_step_up_challenge(capabilities: StepUpCapabilities) -> Option<StepUpChallengeMethod> {
    if capabilities.supports_biometric {
        return Some(StepUpChallengeMethod::DeviceBiometric);
    }
    if capabilities.supports_passcode {
        return Some(StepUpChallengeMethod::DevicePasscode);
    }
    None
}

fn candidate_is_fresh(c: &MemoryCandidate, now: MonotonicTimeNs) -> bool {
    match c.expires_at {
        Some(t) => t.0 > now.0,
        None => true,
    }
}

fn filter_fresh_low_risk_candidates<'a>(
    candidates: &'a [MemoryCandidate],
    now: MonotonicTimeNs,
) -> Vec<&'a MemoryCandidate> {
    candidates
        .iter()
        .filter(|c| candidate_is_fresh(c, now))
        .filter(|c| c.sensitivity_flag == MemorySensitivityFlag::Low)
        .filter(|c| c.confidence == MemoryConfidence::High)
        .collect()
}

fn contains_sensitive_candidate(candidates: &[MemoryCandidate], now: MonotonicTimeNs) -> bool {
    candidates.iter().any(|c| {
        candidate_is_fresh(c, now) && c.sensitivity_flag == MemorySensitivityFlag::Sensitive
    })
}

fn preferred_name<'a>(candidates: &'a [&'a MemoryCandidate]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .find(|c| c.memory_key.as_str() == "preferred_name")
        .filter(|c| c.use_policy == MemoryUsePolicy::AlwaysUsable)
        .map(|c| c.memory_value.verbatim.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
}

fn is_greeting_text(s: &str) -> bool {
    let t = s.trim();
    // Keep this intentionally narrow and deterministic.
    t == "Hello." || t == "Hello"
}

fn truncate_to_char_boundary(mut s: String, max_bytes: usize) -> String {
    if s.len() <= max_bytes {
        return s;
    }
    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    s.truncate(end);
    s
}

fn merge_same_subject_response_text(unsaid_remainder: &str, new_response_text: &str) -> String {
    let unsaid = unsaid_remainder.trim();
    let new_text = new_response_text.trim();
    if unsaid.is_empty() {
        return truncate_to_char_boundary(new_text.to_string(), 32_768);
    }
    if new_text.is_empty() {
        return truncate_to_char_boundary(unsaid.to_string(), 32_768);
    }
    if new_text.eq_ignore_ascii_case(unsaid) || new_text.contains(unsaid) {
        return truncate_to_char_boundary(new_text.to_string(), 32_768);
    }
    if unsaid.contains(new_text) {
        return truncate_to_char_boundary(unsaid.to_string(), 32_768);
    }
    truncate_to_char_boundary(
        format!("{unsaid}\n\nAlso, on your new point: {new_text}"),
        32_768,
    )
}

fn append_switch_topic_return_check(new_response_text: String) -> String {
    let trimmed = new_response_text.trim();
    if trimmed.is_empty() {
        return "Do you still want to continue the previous topic?".to_string();
    }
    if trimmed.contains("Do you still want to continue the previous topic?") {
        return truncate_to_char_boundary(trimmed.to_string(), 32_768);
    }
    truncate_to_char_boundary(
        format!("{trimmed}\n\nDo you still want to continue the previous topic?"),
        32_768,
    )
}

fn should_interrupt_relation_clarify(
    req: &Ph1xRequest,
    thread_state: &ThreadState,
    intent_draft: Option<&IntentDraft>,
) -> bool {
    if thread_state.resume_buffer.is_none() {
        return false;
    }
    if has_deterministic_time_clarification_resume(thread_state) {
        return false;
    }
    if has_deterministic_weather_clarification_resume(thread_state) {
        return false;
    }
    if matches!(
        intent_draft.map(|d| d.intent_type),
        Some(IntentType::Continue | IntentType::MoreDetail)
    ) {
        return false;
    }
    match (
        req.interrupt_subject_relation,
        req.interrupt_subject_relation_confidence,
    ) {
        (Some(InterruptSubjectRelation::Uncertain), Some(_)) => true,
        (Some(InterruptSubjectRelation::Same), Some(conf))
        | (Some(InterruptSubjectRelation::Switch), Some(conf)) => {
            conf < INTERRUPT_RELATION_CONFIDENCE_MIN
        }
        _ => true,
    }
}

fn confirm_snapshot_intent_draft(d: &IntentDraft) -> IntentDraft {
    // Keep pending thread_state lightweight: store only extracted fields, not verbatim evidence excerpts.
    let mut snap = d.clone();
    snap.evidence_spans.clear();
    snap
}

fn tool_ok_text_for_request(req: &Ph1xRequest, tr: &ToolResponse) -> String {
    if let Some(verification) = tr
        .source_metadata
        .as_ref()
        .and_then(|metadata| metadata.web_answer_verification.as_ref())
    {
        return stage5_web_presentation_text_for_request(req, verification);
    }
    if req
        .language_packet
        .as_ref()
        .is_some_and(|packet| packet.output_language_is_chinese())
    {
        if let Some(result) = &tr.tool_result {
            match result {
                ToolResult::Time { local_time_iso } => {
                    return chinese_time_tool_answer_text(local_time_iso);
                }
                ToolResult::Weather { summary } => {
                    return chinese_weather_tool_answer_text(summary);
                }
                _ => {}
            }
        }
    }
    tool_ok_text(tr)
}

fn stage5_web_presentation_text_for_request(
    req: &Ph1xRequest,
    verification: &WebAnswerVerificationPacket,
) -> String {
    if req
        .language_packet
        .as_ref()
        .is_some_and(|packet| packet.output_language_is_chinese())
    {
        if let Some(chinese) = stage5_chinese_web_presentation_text(verification) {
            return chinese;
        }
    }
    verification.response_text.clone()
}

fn stage5_chinese_web_presentation_text(
    verification: &WebAnswerVerificationPacket,
) -> Option<String> {
    match verification.final_answer_class.as_str() {
        "VERIFIED_DIRECT_ANSWER" => {
            let claim = verification.claim_verifications.first()?;
            if claim.claim_type != "leadership_role" || !claim.safe_for_direct_answer {
                return None;
            }
            let value = claim.selected_answer_value.as_deref()?.trim();
            let entity = claim.requested_entity.trim();
            let role = stage5_role_from_claim_text(&claim.claim_text).unwrap_or("CEO");
            if value.is_empty() || entity.is_empty() {
                return None;
            }
            Some(format!("{value} 是 {entity} 的 {role}。"))
        }
        "SOURCE_DISCOVERY_ONLY" => {
            let entity = verification.requested_entity.captured_text.trim();
            if entity.is_empty() {
                Some("我找到了可引用的网页结果。".to_string())
            } else {
                Some(format!("我找到了关于 {entity} 的可引用网页结果。"))
            }
        }
        "PARTIAL_UNCERTAIN_ANSWER" => {
            Some("我找到部分证据，但无法有把握地验证当前答案。".to_string())
        }
        "MIXED_EVIDENCE_ANSWER" => {
            let claim = verification.claim_verifications.first()?;
            let value = claim.selected_answer_value.as_deref()?.trim();
            let entity = claim.requested_entity.trim();
            let role = stage5_role_from_claim_text(&claim.claim_text).unwrap_or("信息");
            if value.is_empty() || entity.is_empty() {
                return None;
            }
            Some(format!(
                "我找到的最清楚来源列出 {value} 是 {entity} 的 {role}，但另一个已接受来源有不同说法，所以不应把这个称谓视为完全确定。"
            ))
        }
        "CLOSEST_SOURCE_BACKED_ANSWER" => {
            let claim = verification.claim_verifications.first()?;
            let value = claim.selected_answer_value.as_deref()?.trim();
            let entity = claim.requested_entity.trim();
            let requested_role = stage5_role_from_claim_text(&claim.claim_text).unwrap_or("信息");
            let closest_role = claim
                .evidence_links
                .first()
                .map(|link| link.role_or_value_match.trim())
                .filter(|role| !role.is_empty())
                .unwrap_or("相关职务");
            if value.is_empty() || entity.is_empty() {
                return None;
            }
            Some(format!(
                "我没有找到来源列出 {entity} 的 {requested_role}。最接近的来源列出 {value} 是 {closest_role}。"
            ))
        }
        "STALE_SOURCE_BACKED_ANSWER" => {
            let claim = verification.claim_verifications.first()?;
            let value = claim.selected_answer_value.as_deref()?.trim();
            let entity = claim.requested_entity.trim();
            let role = claim
                .evidence_links
                .first()
                .map(|link| link.role_or_value_match.trim())
                .filter(|role| !role.is_empty())
                .or_else(|| stage5_role_from_claim_text(&claim.claim_text))
                .unwrap_or("信息");
            if value.is_empty() || entity.is_empty() {
                return None;
            }
            Some(format!(
                "我找到的已接受来源似乎已经过时；它曾列出 {value} 是 {entity} 的 {role}，所以不应把它当作当前信息。"
            ))
        }
        "UNSUPPORTED_SAFE_DEGRADE" => {
            let claim = verification.claim_verifications.first();
            let role = claim
                .and_then(|claim| stage5_role_from_claim_text(&claim.claim_text))
                .unwrap_or("信息");
            let entity = verification.requested_entity.captured_text.trim();
            if role == "信息" {
                Some(format!("我无法从已接受来源验证关于 {entity} 的信息。"))
            } else {
                Some(format!("我无法从已接受来源验证 {entity} 的 {role}。"))
            }
        }
        "CONTRADICTED_SAFE_DEGRADE" => {
            Some("我发现证据相互冲突，无法有把握地验证这个答案。".to_string())
        }
        "STALE_UNCERTAIN_SAFE_DEGRADE" => {
            Some("现有证据似乎已经过时，因此我无法验证当前答案。".to_string())
        }
        "PROTECTED_FAIL_CLOSED" => Some("没有授权的模拟和权限检查，我不能批准薪资。".to_string()),
        _ => None,
    }
}

fn stage5_role_from_claim_text(claim_text: &str) -> Option<&str> {
    let trimmed = claim_text.trim();
    let rest = trimmed.strip_prefix("Verify the ")?;
    let (role, _) = rest.split_once(" of ")?;
    let role = role.trim();
    if role.is_empty() {
        None
    } else {
        Some(role)
    }
}

fn tool_ok_text(tr: &ToolResponse) -> String {
    // Deterministic shaping. Never mention providers here.
    let mut out = String::new();
    if let Some(verification) = tr
        .source_metadata
        .as_ref()
        .and_then(|metadata| metadata.web_answer_verification.as_ref())
    {
        return verification.response_text.clone();
    }
    if let Some(answer) = h414_public_leadership_answer(tr) {
        return answer;
    }
    if let Some(r) = &tr.tool_result {
        match r {
            ToolResult::Time { local_time_iso } => {
                out.push_str(&time_tool_answer_text(local_time_iso));
            }
            ToolResult::Weather { summary } => {
                out.push_str(summary);
            }
            ToolResult::WebSearch { items } | ToolResult::News { items } => {
                out.push_str(&web_search_without_verification_safe_degrade(items));
            }
            ToolResult::UrlFetchAndCite { citations } => {
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
            ToolResult::DocumentUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&field.key),
                        h409_public_answer_fragment(&field.value)
                    ));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
            ToolResult::PhotoUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&field.key),
                        h409_public_answer_fragment(&field.value)
                    ));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
            ToolResult::DataAnalysis {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&field.key),
                        h409_public_answer_fragment(&field.value)
                    ));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
            ToolResult::DeepResearch {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&field.key),
                        h409_public_answer_fragment(&field.value)
                    ));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
            ToolResult::RecordMode {
                summary,
                action_items,
                evidence_refs,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Action items:\n");
                for item in action_items.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&item.key),
                        h409_public_answer_fragment(&item.value)
                    ));
                }
                out.push_str("Recording evidence refs:\n");
                for evidence in evidence_refs.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&evidence.key),
                        h409_public_answer_fragment(&evidence.value)
                    ));
                }
            }
            ToolResult::ConnectorQuery {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(&h409_public_answer_fragment(summary));
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!(
                        "- {}: {}\n",
                        h409_public_answer_fragment(&field.key),
                        h409_public_answer_fragment(&field.value)
                    ));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!(
                        "{}. {} ({})\n",
                        i + 1,
                        h409_public_answer_fragment(&it.title),
                        it.url
                    ));
                }
            }
        }
    }
    let should_append_sources = !matches!(
        tr.tool_result,
        Some(
            ToolResult::Time { .. }
                | ToolResult::Weather { .. }
                | ToolResult::WebSearch { .. }
                | ToolResult::News { .. }
        )
    );
    if should_append_sources {
        if let Some(meta) = &tr.source_metadata {
            if !out.ends_with('\n') && !out.is_empty() {
                out.push('\n');
            }
            out.push_str("Sources:\n");
            for (i, s) in meta.sources.iter().enumerate().take(5) {
                out.push_str(&format!(
                    "{}. {} ({})\n",
                    i + 1,
                    h409_public_answer_fragment(&s.title),
                    s.url
                ));
            }
        }
    }
    if out.trim().is_empty() {
        // Defensive fallback; shouldn't happen due to ToolResponse validation.
        "Done.".to_string()
    } else {
        out
    }
}

fn web_search_without_verification_safe_degrade(
    items: &[selene_kernel_contracts::ph1e::ToolTextSnippet],
) -> String {
    if items.is_empty() {
        "I did not find a usable source-backed answer.".to_string()
    } else {
        let title = items
            .first()
            .map(|item| h409_public_answer_fragment(&item.title))
            .filter(|title| !title.trim().is_empty())
            .unwrap_or_else(|| "a relevant search result".to_string());
        format!("The closest search result I found is {title}.")
    }
}

#[derive(Debug, Clone)]
struct H414SearchEvidence {
    title: String,
    snippet: String,
    url: String,
}

fn h414_public_leadership_answer(tr: &ToolResponse) -> Option<String> {
    let evidence = h414_collect_search_evidence(tr);
    if evidence.is_empty() {
        return None;
    }
    let combined = evidence
        .iter()
        .map(|item| format!("{} {} {}", item.title, item.snippet, item.url))
        .collect::<Vec<_>>()
        .join("\n")
        .to_ascii_lowercase();
    let leadership_context = combined.contains("ceo")
        || combined.contains("director")
        || combined.contains("founder")
        || combined.contains("owner")
        || combined.contains("head of grape")
        || combined.contains("head of wine")
        || combined.contains("leadership");
    if !leadership_context {
        return None;
    }

    if let Some((_source, role, person)) = evidence.iter().find_map(|item| {
        let role = h414_verified_non_ceo_role(item)?;
        let person = h414_extract_public_person_near_role(item)
            .unwrap_or_else(|| "a public leadership profile".to_string());
        Some((item, role, person))
    }) {
        return Some(format!(
            "I did not find a reliable source naming a CEO. The closest source I found lists {person} as {role}."
        ));
    }

    if let Some((_source, person)) = evidence
        .iter()
        .filter(|item| h414_verified_ceo_evidence(item))
        .find_map(|item| {
            let person = h414_extract_public_person_near_role(item)
                .unwrap_or_else(|| "a public CEO listing".to_string());
            Some((item, person))
        })
    {
        return Some(format!(
            "I found {person} listed as CEO in an accepted source."
        ));
    }

    if combined.contains("ceo") {
        return Some("I did not find a reliable public source naming a CEO.".to_string());
    }

    None
}

fn h414_collect_search_evidence(tr: &ToolResponse) -> Vec<H414SearchEvidence> {
    let mut evidence = Vec::new();
    if let Some(result) = &tr.tool_result {
        match result {
            ToolResult::WebSearch { items } | ToolResult::News { items } => {
                for item in items {
                    evidence.push(H414SearchEvidence {
                        title: item.title.clone(),
                        snippet: item.snippet.clone(),
                        url: item.url.clone(),
                    });
                }
            }
            ToolResult::DeepResearch {
                summary,
                extracted_fields,
                citations,
            }
            | ToolResult::DocumentUnderstand {
                summary,
                extracted_fields,
                citations,
            }
            | ToolResult::PhotoUnderstand {
                summary,
                extracted_fields,
                citations,
            }
            | ToolResult::DataAnalysis {
                summary,
                extracted_fields,
                citations,
            }
            | ToolResult::ConnectorQuery {
                summary,
                extracted_fields,
                citations,
            } => {
                for citation in citations {
                    evidence.push(H414SearchEvidence {
                        title: citation.title.clone(),
                        snippet: citation.snippet.clone(),
                        url: citation.url.clone(),
                    });
                }
                for field in extracted_fields {
                    evidence.push(H414SearchEvidence {
                        title: field.key.clone(),
                        snippet: field.value.clone(),
                        url: String::new(),
                    });
                }
                evidence.push(H414SearchEvidence {
                    title: "summary".to_string(),
                    snippet: summary.clone(),
                    url: String::new(),
                });
            }
            ToolResult::UrlFetchAndCite { citations } => {
                for citation in citations {
                    evidence.push(H414SearchEvidence {
                        title: citation.title.clone(),
                        snippet: citation.snippet.clone(),
                        url: citation.url.clone(),
                    });
                }
            }
            ToolResult::Time { .. }
            | ToolResult::Weather { .. }
            | ToolResult::RecordMode { .. } => {}
        }
    }
    if let Some(meta) = &tr.source_metadata {
        for source in &meta.sources {
            evidence.push(H414SearchEvidence {
                title: source.title.clone(),
                snippet: String::new(),
                url: source.url.clone(),
            });
        }
    }
    evidence
}

fn h414_verified_non_ceo_role(item: &H414SearchEvidence) -> Option<&'static str> {
    let combined = format!("{} {}", item.title, item.snippet).to_ascii_lowercase();
    if item.url.trim().is_empty() || h414_weak_or_wrong_ceo_source(item) {
        return None;
    }
    if combined.contains("managing director")
        && (combined.contains("head of grape") || combined.contains("head of wine"))
    {
        Some("Managing Director / Head of Grape and Wine Production")
    } else if combined.contains("managing director") {
        Some("Managing Director")
    } else if combined.contains("director") {
        Some("Director")
    } else if combined.contains("founder") {
        Some("Founder")
    } else if combined.contains("owner") {
        Some("Owner")
    } else {
        None
    }
}

fn h414_verified_ceo_evidence(item: &H414SearchEvidence) -> bool {
    let combined = format!("{} {} {}", item.title, item.snippet, item.url).to_ascii_lowercase();
    !item.url.trim().is_empty()
        && combined.contains("ceo")
        && !h414_weak_or_wrong_ceo_source(item)
        && (combined.contains("source: primary_official")
            || combined.contains("trust: high_confidence")
            || combined.contains("official"))
        && (combined.contains("listed as ceo")
            || combined.contains("ceo at")
            || combined.contains("chief executive officer"))
        && (combined.contains("official")
            || combined.contains("leadership")
            || combined.contains("management")
            || combined.contains("about us")
            || combined.contains("company profile"))
}

fn h414_weak_or_wrong_ceo_source(item: &H414SearchEvidence) -> bool {
    let combined = format!("{} {} {}", item.title, item.snippet, item.url).to_ascii_lowercase();
    combined.contains("ranking")
        || combined.contains("rankings")
        || combined.contains("top ceo")
        || combined.contains("seo")
        || combined.contains("directory")
        || combined.contains("store")
        || combined.contains("shop")
        || combined.contains("marketplace")
        || combined.contains("generic profile")
}

fn h414_extract_public_person_near_role(item: &H414SearchEvidence) -> Option<String> {
    let text = format!("{} {}", item.title, item.snippet);
    let lower = text.to_ascii_lowercase();
    for marker in [
        " is listed as ",
        " is the ",
        " is ",
        " led by ",
        " led by managing director",
        " managing director",
        " director",
        " ceo",
    ] {
        if let Some(index) = lower.find(marker) {
            if let Some(name) = h414_last_capitalized_name(&text[..index]) {
                return Some(name);
            }
        }
    }
    None
}

fn h414_last_capitalized_name(prefix: &str) -> Option<String> {
    let mut parts = Vec::new();
    for token in prefix.split_whitespace().rev() {
        let cleaned = token.trim_matches(|ch: char| ch.is_ascii_punctuation());
        let lower = cleaned.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "by" | "as" | "the" | "and" | "with" | "company" | "profile" | "official"
        ) {
            continue;
        }
        if cleaned
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            parts.push(cleaned.to_string());
            if parts.len() == 2 {
                break;
            }
        } else if !parts.is_empty() {
            break;
        }
    }
    if parts.is_empty() {
        None
    } else {
        parts.reverse();
        Some(parts.join(" "))
    }
}

fn h409_public_answer_fragment(value: &str) -> String {
    let without_tags = h409_strip_html_tags(value);
    let lower = without_tags.to_ascii_lowercase();
    let mut cut_at = without_tags.len();
    for marker in [
        " — source:",
        " – source:",
        " -- source:",
        "; source:",
        " source: unknown",
        " trust: unverified",
        " freshness:",
        " retention:",
        " citation_verified",
        " audit_metadata_only",
        " retrieved at (unix_ms)",
    ] {
        if let Some(index) = lower.find(marker) {
            cut_at = cut_at.min(index);
        }
    }
    without_tags[..cut_at]
        .replace("AUDIT_METADATA_ONLY", "")
        .replace("citation_verified", "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn h409_strip_html_tags(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut inside_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => out.push(ch),
            _ => {}
        }
    }
    out
}

fn time_tool_answer_text(local_time_iso: &str) -> String {
    if let Some((display, label)) = local_time_display_and_label(local_time_iso) {
        return format!("It's {display} in {label}.");
    }

    format!("It's {local_time_iso}.")
}

fn chinese_time_tool_answer_text(local_time_iso: &str) -> String {
    if let Some((display, label)) = local_time_display_and_label(local_time_iso) {
        return format!(
            "{}现在是{}。",
            chinese_time_zone_display_label(&label),
            display
        );
    }

    format!("现在是{local_time_iso}。")
}

fn chinese_weather_tool_answer_text(summary: &str) -> String {
    let trimmed = summary.trim();
    if trimmed.chars().any(is_cjk_char) {
        return trimmed.to_string();
    }
    if trimmed.to_ascii_lowercase().contains("provider")
        || trimmed.to_ascii_lowercase().contains("unavailable")
        || trimmed.to_ascii_lowercase().contains("failed")
    {
        return "天气服务暂时不可用，所以我现在不能核验实时天气。".to_string();
    }
    let sentence = trimmed.trim_end_matches('.');
    if let Some(prefix) =
        sentence.strip_suffix(" right now, so current conditions do not indicate rain")
    {
        if let Some((place, rest)) = prefix.split_once(" is ") {
            if let Some((temp, condition)) = rest.split_once(" and ") {
                return format!(
                    "{}现在是{}，天气{}；当前条件不显示下雨。",
                    chinese_time_zone_display_label(place),
                    temp.trim(),
                    chinese_weather_condition_label(condition)
                );
            }
        }
    }
    if let Some((place, rest)) = sentence.split_once(" is ") {
        if let Some((temp, condition)) = rest.split_once(" and ") {
            return format!(
                "{}现在是{}，天气{}。",
                chinese_time_zone_display_label(place),
                temp.trim(),
                chinese_weather_condition_label(condition)
            );
        }
        if let Some((temp, condition)) = rest.split_once(" with ") {
            return format!(
                "{}现在是{}，正在{}。",
                chinese_time_zone_display_label(place),
                temp.trim(),
                chinese_weather_condition_label(condition)
            );
        }
    }
    format!("天气结果：{}。", trimmed.trim_end_matches('.'))
}

fn chinese_time_zone_display_label(label: &str) -> String {
    match label.trim().to_ascii_lowercase().as_str() {
        "tokyo" | "japan" => "东京".to_string(),
        "new york" => "纽约".to_string(),
        "sydney" => "悉尼".to_string(),
        "barcelona" => "巴塞罗那".to_string(),
        "madrid" => "马德里".to_string(),
        other if !other.is_empty() => label.trim().to_string(),
        _ => "当地".to_string(),
    }
}

fn chinese_weather_condition_label(condition: &str) -> String {
    let normalized = condition.trim().trim_matches('.').to_ascii_lowercase();
    if normalized.contains("partly cloudy") {
        "局部多云".to_string()
    } else if normalized.contains("mostly clear") {
        "大多晴朗".to_string()
    } else if normalized.contains("cloudy") {
        "多云".to_string()
    } else if normalized.contains("clear") || normalized.contains("sunny") {
        "晴朗".to_string()
    } else if normalized.contains("drizzle") {
        "毛毛雨".to_string()
    } else if normalized.contains("heavy rain") {
        "大雨".to_string()
    } else if normalized.contains("rain") {
        "下雨".to_string()
    } else if normalized.contains("snow") {
        "下雪".to_string()
    } else if normalized.contains("thunderstorm") {
        "雷雨".to_string()
    } else {
        condition.trim().trim_matches('.').to_string()
    }
}

fn is_cjk_char(ch: char) -> bool {
    ('\u{4e00}'..='\u{9fff}').contains(&ch)
        || ('\u{3400}'..='\u{4dbf}').contains(&ch)
        || ('\u{f900}'..='\u{faff}').contains(&ch)
}

fn local_time_display_and_label(local_time_iso: &str) -> Option<(String, String)> {
    let (timestamp, zone_part) = local_time_iso.rsplit_once('[')?;
    let zone_and_label = zone_part.strip_suffix(']')?;
    let (zone, explicit_label) = zone_and_label
        .split_once('|')
        .map(|(zone, label)| (zone, Some(label)))
        .unwrap_or((zone_and_label, None));
    let time = timestamp.split_once('T')?.1;
    let hour: u32 = time.get(0..2)?.parse().ok()?;
    let minute = time.get(3..5)?;
    let suffix = if hour >= 12 { "PM" } else { "AM" };
    let display_hour = match hour % 12 {
        0 => 12,
        value => value,
    };
    Some((
        format!("{display_hour}:{minute} {suffix}"),
        explicit_label
            .map(str::trim)
            .filter(|label| !label.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| time_zone_display_label(zone)),
    ))
}

fn time_zone_display_label(zone: &str) -> String {
    match zone {
        "America/New_York" => "New York".to_string(),
        "Asia/Tokyo" => "Japan".to_string(),
        "Australia/Sydney" => "Sydney".to_string(),
        "Europe/Berlin" => "Germany".to_string(),
        "Europe/Lisbon" => "Lisbon".to_string(),
        "Europe/Rome" => "Italy".to_string(),
        "UTC" => "UTC".to_string(),
        other => other.rsplit('/').next().unwrap_or(other).replace('_', " "),
    }
}

fn intent_query_text(d: &IntentDraft) -> String {
    // Deterministic: prefer a task evidence excerpt if present, else use a stable fallback token.
    if let Some(field) = d.fields.iter().find(|field| field.key == FieldKey::Task) {
        if let Some(normalized) = field.value.normalized_value.as_ref() {
            return normalized.clone();
        }
    }
    d.evidence_spans
        .iter()
        .find(|e| e.field == FieldKey::Task)
        .map(|e| e.verbatim_excerpt.clone())
        .unwrap_or_else(|| "query".to_string())
}

fn intent_query_text_with_thread_context(d: &IntentDraft, thread_state: &ThreadState) -> String {
    let mut query = intent_query_text(d);
    if let Some(project_id) = &thread_state.project_id {
        query.push_str(" | project_id=");
        query.push_str(project_id);
    }
    if !thread_state.pinned_context_refs.is_empty() {
        query.push_str(" | pinned_context_refs=");
        query.push_str(&thread_state.pinned_context_refs.join(","));
    }
    query
}

fn confirm_text(d: &IntentDraft) -> String {
    // Deterministic restatement using only already-extracted field spans (no reinterpretation).
    match d.intent_type {
        IntentType::SendMoney => {
            let amount = field_original(d, FieldKey::Amount).unwrap_or("an amount");
            let to = field_original(d, FieldKey::Recipient).unwrap_or("a recipient");
            format!("You want to send {amount} to {to}. Is that right?")
        }
        IntentType::BookTable => {
            let place = field_original(d, FieldKey::Place).unwrap_or("a place");
            let when = field_original(d, FieldKey::When).unwrap_or("a time");
            let party = field_original(d, FieldKey::PartySize).unwrap_or("a party size");
            format!("You want to book a table at {place} for {party} on {when}. Is that right?")
        }
        IntentType::CreateCalendarEvent => {
            let when = field_original(d, FieldKey::When).unwrap_or("a time");
            let who = field_original(d, FieldKey::Person);
            match who {
                Some(w) => {
                    format!("You want to schedule a meeting {when} with {w}. Is that right?")
                }
                None => format!("You want to schedule a meeting {when}. Is that right?"),
            }
        }
        IntentType::SetReminder => {
            let task = field_original(d, FieldKey::Task).unwrap_or("a task");
            let when = field_original(d, FieldKey::When).unwrap_or("a time");
            format!("You want a reminder {when}: {task}. Is that right?")
        }
        IntentType::UpdateBcastWaitPolicy => {
            let amount = field_original(d, FieldKey::Amount).unwrap_or("300 seconds");
            format!(
                "You want to change the non-urgent follow-up wait time to {amount}. Is that right?"
            )
        }
        IntentType::UpdateBcastUrgentFollowupPolicy => {
            let behavior = field_original(d, FieldKey::Task).unwrap_or("immediate");
            format!("You want urgent follow-up behavior to be {behavior}. Is that right?")
        }
        IntentType::UpdateReminder => {
            let reminder_id = field_original(d, FieldKey::ReminderId).unwrap_or("that reminder");
            let when = field_original(d, FieldKey::When).unwrap_or("a new time");
            format!("You want to update {reminder_id} to {when}. Is that right?")
        }
        IntentType::CancelReminder => {
            let reminder_id = field_original(d, FieldKey::ReminderId).unwrap_or("that reminder");
            format!("You want to cancel {reminder_id}. Is that right?")
        }
        IntentType::ListReminders => {
            "You want me to list your reminders. Is that right?".to_string()
        }
        IntentType::MemoryRememberRequest => {
            let subject = field_original(d, FieldKey::Task).unwrap_or("that detail");
            format!("You want me to remember this: {subject}. Is that right?")
        }
        IntentType::MemoryForgetRequest => {
            let subject = field_original(d, FieldKey::Task).unwrap_or("that memory");
            format!("You want me to forget this: {subject}. Is that right?")
        }
        IntentType::MemoryQuery => {
            let subject = field_original(d, FieldKey::Task).unwrap_or("your memory");
            format!("You want me to recall what I know about {subject}. Is that right?")
        }
        IntentType::CreateInviteLink => {
            let invitee_type = field_original(d, FieldKey::InviteeType).unwrap_or("a person");
            let contact = field_original(d, FieldKey::RecipientContact)
                .or_else(|| field_original(d, FieldKey::Recipient))
                .unwrap_or("the recipient");
            let delivery =
                field_original(d, FieldKey::DeliveryMethod).unwrap_or("a delivery method");
            format!("You want to create an invite link for {contact} ({invitee_type}) via {delivery}. Is that right?")
        }
        IntentType::CapreqManage => {
            let action = field_original(d, FieldKey::CapreqAction)
                .unwrap_or("create_draft")
                .to_ascii_lowercase();
            let capreq_id = field_original(d, FieldKey::CapreqId);
            let capability =
                field_original(d, FieldKey::RequestedCapabilityId).unwrap_or("a capability");
            let scope = field_original(d, FieldKey::TargetScopeRef).unwrap_or("a scope");
            let tenant = field_original(d, FieldKey::TenantId).unwrap_or("a tenant");
            let justification = field_original(d, FieldKey::Justification).unwrap_or("a reason");
            match action.as_str() {
                "submit_for_approval" => match capreq_id {
                    Some(id) => {
                        format!("You want to submit capability request {id} for approval. Is that right?")
                    }
                    None => format!(
                        "You want to submit the capability request for {capability} in {scope} for {tenant} because \"{justification}\". Is that right?"
                    ),
                },
                "approve" => {
                    let id = capreq_id.unwrap_or("this capability request");
                    format!("You want to approve {id}. Is that right?")
                }
                "reject" => {
                    let id = capreq_id.unwrap_or("this capability request");
                    format!("You want to reject {id}. Is that right?")
                }
                "fulfill" => {
                    let id = capreq_id.unwrap_or("this capability request");
                    format!("You want to mark {id} as fulfilled. Is that right?")
                }
                "cancel" => {
                    let id = capreq_id.unwrap_or("this capability request");
                    format!("You want to cancel {id}. Is that right?")
                }
                _ => format!(
                    "You want to create a capability request for {capability} in {scope} for {tenant} because \"{justification}\". Is that right?"
                ),
            }
        }
        IntentType::AccessSchemaManage => {
            let action = field_original(d, FieldKey::ApAction)
                .unwrap_or("CREATE_DRAFT")
                .to_ascii_uppercase();
            let profile_id =
                field_original(d, FieldKey::AccessProfileId).unwrap_or("an access profile");
            let version =
                field_original(d, FieldKey::SchemaVersionId).unwrap_or("a schema version");
            let scope = field_original(d, FieldKey::ApScope).unwrap_or("TENANT");
            let tenant = field_original(d, FieldKey::TenantId).unwrap_or("the tenant");
            let review_channel =
                field_original(d, FieldKey::AccessReviewChannel).unwrap_or("PHONE_DESKTOP");
            let rule_action = field_original(d, FieldKey::AccessRuleAction).unwrap_or("AGREE");
            format!(
                "You are requesting {action} for access profile {profile_id} ({version}) in {scope} scope for {tenant}, using review channel {review_channel} with rule action {rule_action}. Please confirm."
            )
        }
        IntentType::AccessEscalationVote => {
            let vote_action = field_original(d, FieldKey::VoteAction).unwrap_or("CAST_VOTE");
            let case_id =
                field_original(d, FieldKey::EscalationCaseId).unwrap_or("an escalation case");
            let vote_value = field_original(d, FieldKey::VoteValue).unwrap_or("APPROVE");
            format!(
                "You want to run {vote_action} on escalation case {case_id} with vote {vote_value}. Is that correct?"
            )
        }
        IntentType::AccessInstanceCompileRefresh => {
            let target_user =
                field_original(d, FieldKey::TargetUserId).unwrap_or("the target user");
            let profile_id =
                field_original(d, FieldKey::AccessProfileId).unwrap_or("an access profile");
            let tenant = field_original(d, FieldKey::TenantId).unwrap_or("the tenant");
            format!(
                "You want to compile or refresh access for {target_user} using profile {profile_id} in {tenant}. Is that correct?"
            )
        }
        IntentType::TimeQuery
        | IntentType::WeatherQuery
        | IntentType::WebSearchQuery
        | IntentType::NewsQuery
        | IntentType::UrlFetchAndCiteQuery
        | IntentType::DocumentUnderstandQuery
        | IntentType::PhotoUnderstandQuery
        | IntentType::DataAnalysisQuery
        | IntentType::DeepResearchQuery
        | IntentType::RecordModeQuery
        | IntentType::ConnectorQuery => "Is that right?".to_string(),
        IntentType::Continue | IntentType::MoreDetail => "Is that right?".to_string(),
    }
}

fn field_original<'a>(d: &'a IntentDraft, key: FieldKey) -> Option<&'a str> {
    d.fields
        .iter()
        .find(|f| f.key == key)
        .map(|f| f.value.original_span.as_str())
}

fn retry_message_for_failure(rc: ReasonCodeId, fail_detail: Option<&str>) -> String {
    if let Some(detail) = fail_detail {
        if detail.contains("stage2_provider_control=1")
            || detail.contains("WEB_ADMIN_DISABLED")
            || detail.contains("PROVIDER_DISABLED")
        {
            return selene_engines::ph1providerctl::PROVIDER_DISABLED_RESPONSE_TEXT.to_string();
        }
        if detail.contains("weather_realtime_provider_error") {
            return "I couldn't get the weather for that place right now.".to_string();
        }
        if detail.contains("weather_provider_not_wired") {
            return "I couldn't get the weather for that place right now.".to_string();
        }
        if detail.contains("weather_query_missing_place") {
            return "Which place do you mean?".to_string();
        }
        if detail.contains("ambiguous_time_location") {
            if let Some(alternatives) = detail
                .split_once("alternatives=")
                .map(|(_, alternatives)| alternatives)
                .map(time_clarification_prompt_for_alternatives)
                .filter(|options| !options.is_empty())
            {
                return alternatives;
            }
            return "That location has more than one timezone. Please ask with a specific city or local place.".to_string();
        }
        if detail.contains("missing_time_location") {
            return "Which place do you mean?".to_string();
        }
        if detail.contains("unsupported_time_location") {
            return "I can't resolve that location yet. Please ask with a supported city, country, or local place.".to_string();
        }
    }
    if rc == selene_engines::ph1e::reason_codes::E_FAIL_PROVIDER_MISSING_CONFIG {
        return format!(
            "Brave API key not configured. Run: selene vault set {}",
            ProviderSecretId::BraveSearchApiKey.as_str()
        );
    }
    if rc == selene_engines::ph1e::reason_codes::E_FAIL_PROVIDER_UPSTREAM {
        if let Some(detail) = safe_tool_fail_detail(fail_detail) {
            return format!("Upstream provider error ({detail}). Please try again.");
        }
        return "Upstream provider failed. Please try again.".to_string();
    }
    "Sorry — I couldn’t complete that just now. Could you try again?".to_string()
}

fn safe_tool_fail_detail(raw: Option<&str>) -> Option<String> {
    let raw = raw?;
    let mut cleaned = String::with_capacity(raw.len());
    for ch in raw.chars() {
        if !ch.is_control() {
            cleaned.push(ch);
        }
    }
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let collapsed = collapsed.trim();
    if collapsed.is_empty() {
        return None;
    }
    let bounded: String = collapsed.chars().take(180).collect();
    Some(bounded)
}

fn time_clarification_prompt_for_alternatives(raw: &str) -> String {
    let options = time_clarification_options(raw);
    if options.is_empty() {
        return String::new();
    }
    if time_clarification_alternatives_are_technical(raw) {
        return "That place has more than one timezone. Which city or local place should I use?"
            .to_string();
    }
    format!("That place has more than one timezone. Do you mean {options}?")
}

fn time_clarification_options(raw: &str) -> String {
    let mut options: Vec<String> = raw
        .split('|')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .take(3)
        .map(|value| value.trim_matches('.').to_string())
        .collect();
    options.dedup();
    match options.as_slice() {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} or {second}"),
        [first, second, third, ..] => format!("{first}, {second}, or {third}"),
    }
}

fn time_clarification_alternatives_are_technical(raw: &str) -> bool {
    raw.split('|')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .any(time_clarification_alternative_is_technical)
}

fn time_clarification_alternative_is_technical(value: &str) -> bool {
    let candidate = value
        .split_once(" (")
        .map(|(zone, _)| zone)
        .unwrap_or(value)
        .trim();
    let Some((area, place)) = candidate.split_once('/') else {
        return false;
    };
    !area.contains(char::is_whitespace)
        && !place.trim().is_empty()
        && area
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
        && candidate
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '-' | '+'))
}

fn weather_clarification_options(options: &[String]) -> String {
    match options {
        [] => String::new(),
        [one] => one.clone(),
        [first, second] => format!("{first} or {second}"),
        [first, second, third, ..] => format!("{first}, {second}, or {third}"),
    }
}

fn clarify_for_invite_link_recipient_resolution(
    d: &IntentDraft,
) -> Result<Option<ClarifyDirective>, ContractViolation> {
    if d.intent_type != IntentType::CreateInviteLink {
        return Ok(None);
    }

    if d.ambiguity_flags
        .contains(&AmbiguityFlag::RecipientAmbiguous)
    {
        let recipient_name = field_original(d, FieldKey::Recipient)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or("recipient");
        return Ok(Some(ClarifyDirective::v1(
            format!("Which {recipient_name}?"),
            vec![
                format!("{recipient_name} from work"),
                format!("{recipient_name} from family"),
            ],
            vec![FieldKey::Recipient],
        )?));
    }

    if has_missing_field(d, FieldKey::DeliveryMethod) {
        return Ok(Some(ClarifyDirective::v1(
            "How should I send it (Selene App / SMS / WhatsApp / WeChat / email)?".to_string(),
            vec![
                "Selene App".to_string(),
                "SMS".to_string(),
                "Email".to_string(),
            ],
            vec![FieldKey::DeliveryMethod],
        )?));
    }

    if has_missing_field(d, FieldKey::RecipientContact) {
        if let Some(recipient_name) = field_original(d, FieldKey::Recipient)
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            let channel = invite_delivery_channel_label(d);
            return Ok(Some(ClarifyDirective::v1(
                format!("What is {recipient_name}'s contact for {channel}?"),
                contact_answer_formats_for_channel(channel),
                vec![FieldKey::RecipientContact],
            )?));
        }
    }

    Ok(None)
}

fn hydrate_invite_link_contact_from_memory(
    draft: &IntentDraft,
    memory_candidates: &[MemoryCandidate],
    now: MonotonicTimeNs,
) -> Result<IntentDraft, ContractViolation> {
    if draft.intent_type != IntentType::CreateInviteLink {
        return Ok(draft.clone());
    }
    if !has_missing_field(draft, FieldKey::RecipientContact) {
        return Ok(draft.clone());
    }
    let recipient = field_original(draft, FieldKey::Recipient)
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(|name| name.to_ascii_lowercase());
    let Some(recipient) = recipient else {
        return Ok(draft.clone());
    };

    let safe_memory = filter_fresh_low_risk_candidates(memory_candidates, now);
    let delivery_channel = invite_delivery_channel_label(draft);
    let Some(contact) =
        resolve_invite_link_contact_from_memory(&safe_memory, &recipient, delivery_channel)
    else {
        return Ok(draft.clone());
    };

    let mut hydrated = draft.clone();
    hydrated
        .required_fields_missing
        .retain(|field| *field != FieldKey::RecipientContact);
    let contact_value = FieldValue::verbatim(contact)?;
    if let Some(existing) = hydrated
        .fields
        .iter_mut()
        .find(|field| field.key == FieldKey::RecipientContact)
    {
        existing.value = contact_value;
        existing.confidence = OverallConfidence::Med;
    } else {
        hydrated.fields.push(IntentField {
            key: FieldKey::RecipientContact,
            value: contact_value,
            confidence: OverallConfidence::Med,
        });
    }
    Ok(hydrated)
}

fn resolve_invite_link_contact_from_memory(
    candidates: &[&MemoryCandidate],
    recipient: &str,
    delivery_channel: &str,
) -> Option<String> {
    let mut selected: Option<String> = None;
    for candidate in candidates {
        let key = candidate.memory_key.as_str().to_ascii_lowercase();
        let value = candidate.memory_value.verbatim.trim();
        if value.is_empty() {
            continue;
        }
        let value_lower = value.to_ascii_lowercase();
        let recipient_match = key.contains(recipient) || value_lower.contains(recipient);
        if !recipient_match {
            continue;
        }
        let key_hints_contact = key.contains("contact")
            || key.contains("phone")
            || key.contains("sms")
            || key.contains("email")
            || key.contains("handle");
        if !key_hints_contact && !invite_contact_matches_channel(value, delivery_channel) {
            continue;
        }
        if !invite_contact_matches_channel(value, delivery_channel) {
            continue;
        }
        match &selected {
            None => selected = Some(value.to_string()),
            Some(existing) if !existing.eq_ignore_ascii_case(value) => return None,
            _ => {}
        }
    }
    selected
}

fn invite_contact_matches_channel(contact: &str, delivery_channel: &str) -> bool {
    match delivery_channel {
        "SMS" | "WhatsApp" | "WeChat" => looks_like_phone_number(contact),
        "email" => looks_like_email_address(contact),
        "Selene App" => {
            looks_like_selene_app_handle(contact)
                || contact.to_ascii_lowercase().contains("selene app")
        }
        _ => false,
    }
}

fn looks_like_phone_number(value: &str) -> bool {
    let digit_count = value.chars().filter(|c| c.is_ascii_digit()).count();
    digit_count >= 7 && digit_count <= 16
}

fn looks_like_email_address(value: &str) -> bool {
    let trimmed = value.trim();
    let Some((local, domain)) = trimmed.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn looks_like_selene_app_handle(value: &str) -> bool {
    let trimmed = value.trim();
    let len = trimmed.len();
    if !(3..=64).contains(&len) || trimmed.contains(char::is_whitespace) {
        return false;
    }
    trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn has_missing_field(d: &IntentDraft, key: FieldKey) -> bool {
    d.required_fields_missing.contains(&key)
}

fn invite_delivery_channel_label(d: &IntentDraft) -> &'static str {
    let token = d
        .fields
        .iter()
        .find(|f| f.key == FieldKey::DeliveryMethod)
        .and_then(|f| f.value.normalized_value.as_deref())
        .or_else(|| field_original(d, FieldKey::DeliveryMethod))
        .unwrap_or("selene_app")
        .trim()
        .to_ascii_lowercase();

    match token.as_str() {
        "selene_app" | "app" => "Selene App",
        "sms" | "text" => "SMS",
        "whatsapp" | "wa" => "WhatsApp",
        "wechat" | "we_chat" => "WeChat",
        "email" | "mail" => "email",
        _ => "Selene App",
    }
}

fn contact_answer_formats_for_channel(channel: &str) -> Vec<String> {
    match channel {
        "Selene App" => vec![
            "Use Tom in Selene App".to_string(),
            "Tom's Selene App user ID is tom_lee".to_string(),
        ],
        "SMS" => vec!["+14155551212".to_string(), "+442071234567".to_string()],
        "email" => vec![
            "tom@example.invalid".to_string(),
            "tom.lee@company.com".to_string(),
        ],
        "WhatsApp" => vec![
            "WhatsApp: +14155551212".to_string(),
            "WhatsApp: tom_lee".to_string(),
        ],
        "WeChat" => vec![
            "WeChat: tomlee".to_string(),
            "WeChat: tom_family".to_string(),
        ],
        _ => vec![
            "+14155551212".to_string(),
            "tom@example.invalid".to_string(),
            "WeChat: tomlee".to_string(),
        ],
    }
}

fn clarify_for_missing(
    intent_type: IntentType,
    missing: &[FieldKey],
) -> Result<ClarifyDirective, ContractViolation> {
    let primary = select_primary_missing(missing);
    let (question, formats) = match (intent_type, primary) {
        (_, FieldKey::IntentChoice) => (
            "Which one should I do first?".to_string(),
            vec!["The first one".to_string(), "The second one".to_string()],
        ),
        (_, FieldKey::ReferenceTarget) => (
            "What does that refer to?".to_string(),
            vec!["The meeting".to_string(), "The reminder".to_string()],
        ),
        (_, FieldKey::When) => (
            "What day and time?".to_string(),
            vec![
                "Tomorrow at 3pm".to_string(),
                "Friday 10am".to_string(),
                "2026-02-10 15:00".to_string(),
            ],
        ),
        (_, FieldKey::ReminderId) => (
            "Which reminder ID should I use?".to_string(),
            vec![
                "rem_0000000000000001".to_string(),
                "rem_0000000000000002".to_string(),
            ],
        ),
        (IntentType::UpdateBcastWaitPolicy, FieldKey::Amount) => (
            "What non-urgent wait time should I set before follow-up?".to_string(),
            vec![
                "2 minutes".to_string(),
                "300 seconds".to_string(),
                "10 min".to_string(),
            ],
        ),
        (IntentType::UpdateBcastUrgentFollowupPolicy, FieldKey::Task) => (
            "For urgent broadcasts, should follow-up be immediate or wait?".to_string(),
            vec![
                "Immediate".to_string(),
                "Wait".to_string(),
                "Delay urgent follow-up".to_string(),
            ],
        ),
        (_, FieldKey::Amount) => (
            "How much?".to_string(),
            vec![
                "$20".to_string(),
                "100 dollars".to_string(),
                "15".to_string(),
            ],
        ),
        (_, FieldKey::Task) => (
            "What exactly should I do?".to_string(),
            vec![
                "Remind me to call mom".to_string(),
                "Schedule a meeting".to_string(),
            ],
        ),
        (_, FieldKey::Recipient) => (
            "Who is this for?".to_string(),
            vec!["To Alex".to_string(), "To John".to_string()],
        ),
        (_, FieldKey::Place) => (
            "Where?".to_string(),
            vec!["At Marina Bay".to_string(), "At Sushi Den".to_string()],
        ),
        (_, FieldKey::PartySize) => (
            "For how many people?".to_string(),
            vec!["For 2".to_string(), "For four".to_string()],
        ),
        (_, FieldKey::Person) => (
            "Who is it with?".to_string(),
            vec!["With John".to_string(), "With Alex".to_string()],
        ),
        (_, FieldKey::InviteeType) => (
            "What kind of invite is this?".to_string(),
            vec![
                "Employee".to_string(),
                "Associate".to_string(),
                "Family member".to_string(),
            ],
        ),
        (_, FieldKey::DeliveryMethod) => (
            "How should I send it?".to_string(),
            vec![
                "SMS".to_string(),
                "Email".to_string(),
                "WhatsApp".to_string(),
            ],
        ),
        (_, FieldKey::RecipientContact) => (
            "Where should I send the link?".to_string(),
            vec![
                "+14155551212".to_string(),
                "name@example.invalid".to_string(),
                "WeChat: alice".to_string(),
            ],
        ),
        (_, FieldKey::TenantId) => (
            "Which company is this for?".to_string(),
            vec!["Selene".to_string(), "My company".to_string()],
        ),
        (_, FieldKey::RequestedCapabilityId) => (
            "Which capability should this request include?".to_string(),
            vec![
                "position.activate".to_string(),
                "access.override.create".to_string(),
                "payroll.approve".to_string(),
            ],
        ),
        (_, FieldKey::CapreqAction) => (
            "Which capability-request action should I run?".to_string(),
            vec![
                "create_draft".to_string(),
                "submit_for_approval".to_string(),
                "approve".to_string(),
            ],
        ),
        (_, FieldKey::CapreqId) => (
            "Which capability request ID is this for?".to_string(),
            vec![
                "capreq_abc123".to_string(),
                "capreq_tenant_1_payroll".to_string(),
                "capreq_store_17_mgr".to_string(),
            ],
        ),
        (_, FieldKey::AccessProfileId) => (
            "Which access profile is this for?".to_string(),
            vec![
                "AP_CLERK".to_string(),
                "AP_DRIVER".to_string(),
                "AP_CEO".to_string(),
            ],
        ),
        (_, FieldKey::SchemaVersionId) => (
            "Which schema version should I use?".to_string(),
            vec!["v1".to_string(), "v2".to_string(), "v3".to_string()],
        ),
        (_, FieldKey::ApScope) => (
            "Is this global or tenant scope?".to_string(),
            vec!["GLOBAL".to_string(), "TENANT".to_string()],
        ),
        (_, FieldKey::ApAction) => (
            "What access-profile action should I run?".to_string(),
            vec![
                "CREATE_DRAFT".to_string(),
                "UPDATE".to_string(),
                "ACTIVATE".to_string(),
                "RETIRE".to_string(),
            ],
        ),
        (_, FieldKey::AccessReviewChannel) => (
            "Should I send this to your phone or desktop for review, or read it out loud?"
                .to_string(),
            vec!["PHONE_DESKTOP".to_string(), "READ_OUT_LOUD".to_string()],
        ),
        (_, FieldKey::AccessRuleAction) => (
            "Which rule action should I record?".to_string(),
            vec![
                "AGREE".to_string(),
                "DISAGREE".to_string(),
                "EDIT".to_string(),
                "DELETE".to_string(),
                "DISABLE".to_string(),
                "ADD_CUSTOM_RULE".to_string(),
            ],
        ),
        (_, FieldKey::ProfilePayloadJson) => (
            "Please provide the profile rule payload.".to_string(),
            vec![
                "{\"allow\":[\"LINK_INVITE\"]}".to_string(),
                "{\"allow\":[\"CAPREQ_MANAGE\"],\"limits\":{\"amount\":5000}}".to_string(),
            ],
        ),
        (_, FieldKey::EscalationCaseId) => (
            "Which escalation case is this for?".to_string(),
            vec!["esc_case_001".to_string(), "esc_case_store_17".to_string()],
        ),
        (_, FieldKey::BoardPolicyId) => (
            "Which board policy should apply?".to_string(),
            vec![
                "board_policy_2_of_3".to_string(),
                "board_policy_70pct".to_string(),
            ],
        ),
        (_, FieldKey::TargetUserId) => (
            "Which user is the target?".to_string(),
            vec!["user_123".to_string(), "employee_warehouse_mgr".to_string()],
        ),
        (_, FieldKey::AccessInstanceId) => (
            "Which access instance should this use?".to_string(),
            vec!["acc_inst_001".to_string(), "acc_inst_driver_27".to_string()],
        ),
        (_, FieldKey::VoteAction) => (
            "What vote action should I run?".to_string(),
            vec!["CAST_VOTE".to_string(), "RESOLVE".to_string()],
        ),
        (_, FieldKey::VoteValue) => (
            "What vote value should I record?".to_string(),
            vec!["APPROVE".to_string(), "REJECT".to_string()],
        ),
        (_, FieldKey::OverrideResult) => (
            "What override result should apply?".to_string(),
            vec![
                "ONE_SHOT".to_string(),
                "TEMPORARY".to_string(),
                "TIME_WINDOW".to_string(),
                "PERMANENT".to_string(),
                "DENY".to_string(),
            ],
        ),
        (_, FieldKey::PositionId) => (
            "Which position should this use?".to_string(),
            vec![
                "position_driver".to_string(),
                "position_warehouse_manager".to_string(),
            ],
        ),
        (_, FieldKey::OverlayIdList) => (
            "Which overlay IDs should I apply?".to_string(),
            vec![
                "overlay_driver_safety".to_string(),
                "overlay_retail_limits".to_string(),
            ],
        ),
        (_, FieldKey::CompileReason) => (
            "Why are we compiling this access instance?".to_string(),
            vec![
                "POSITION_CHANGED".to_string(),
                "AP_VERSION_ACTIVATED".to_string(),
                "OVERRIDE_UPDATED".to_string(),
            ],
        ),
        (_, FieldKey::TargetScopeRef) => (
            "What target scope should this apply to?".to_string(),
            vec![
                "store_17".to_string(),
                "team.finance".to_string(),
                "tenant_default".to_string(),
            ],
        ),
        (_, FieldKey::Justification) => (
            "What is the justification?".to_string(),
            vec![
                "Monthly payroll processing".to_string(),
                "Need temporary manager coverage".to_string(),
                "Required for onboarding completion".to_string(),
            ],
        ),
    };

    ClarifyDirective::v1(question, formats, vec![primary])
}

fn select_primary_missing(missing: &[FieldKey]) -> FieldKey {
    for k in [
        FieldKey::IntentChoice,
        FieldKey::ReferenceTarget,
        FieldKey::AccessReviewChannel,
        FieldKey::AccessRuleAction,
        FieldKey::ApAction,
        FieldKey::AccessProfileId,
        FieldKey::SchemaVersionId,
        FieldKey::ApScope,
        FieldKey::ProfilePayloadJson,
        FieldKey::EscalationCaseId,
        FieldKey::BoardPolicyId,
        FieldKey::TargetUserId,
        FieldKey::AccessInstanceId,
        FieldKey::VoteAction,
        FieldKey::VoteValue,
        FieldKey::OverrideResult,
        FieldKey::PositionId,
        FieldKey::OverlayIdList,
        FieldKey::CompileReason,
        FieldKey::CapreqAction,
        FieldKey::CapreqId,
        FieldKey::RequestedCapabilityId,
        FieldKey::TargetScopeRef,
        FieldKey::Justification,
        FieldKey::TenantId,
        FieldKey::Amount,
        FieldKey::Recipient,
        FieldKey::ReminderId,
        FieldKey::Task,
        FieldKey::When,
    ] {
        if missing.contains(&k) {
            return k;
        }
    }
    missing.first().copied().unwrap_or(FieldKey::Task)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportDisplayTarget {
    Desktop,
    Phone,
}

impl ReportDisplayTarget {
    pub fn as_str(self) -> &'static str {
        match self {
            ReportDisplayTarget::Desktop => "desktop",
            ReportDisplayTarget::Phone => "phone",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "desktop" => Some(Self::Desktop),
            "phone" => Some(Self::Phone),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReportDisplayResolution {
    Resolved(ReportDisplayTarget),
    Clarify(String),
}

pub fn resolve_report_display_target(
    explicit_target: Option<&str>,
    remembered_default: Option<&str>,
) -> ReportDisplayResolution {
    if let Some(explicit) = explicit_target {
        if let Some(target) = ReportDisplayTarget::parse(explicit) {
            return ReportDisplayResolution::Resolved(target);
        }
    }
    if let Some(remembered) = remembered_default {
        if let Some(target) = ReportDisplayTarget::parse(remembered) {
            return ReportDisplayResolution::Resolved(target);
        }
    }
    ReportDisplayResolution::Clarify(
        "Where do you want this report displayed: desktop or phone?".to_string(),
    )
}

const STAGE8_5_PLANNING_SUBJECT: &str = "ph1x:frame:planning";
const STAGE8_5_WRITING_SUBJECT: &str = "ph1x:frame:writing";
const STAGE8_5_TOOL_CHOICE_PREFIX: &str = "ph1x:frame:clarify:";
const STAGE8_5_PROTECTED_PREFIX: &str = "ph1x:frame:protected_boundary:";

const STAGE8_5_FRAME_PLANNING_REF: &str = "ph1x:frame_ref:planning";
const STAGE8_5_FRAME_WRITING_REF: &str = "ph1x:frame_ref:writing";
const STAGE8_5_EXPECT_RECOMMENDATION_REF: &str = "ph1x:expected:recommendation";
const STAGE8_5_EXPECT_REWRITE_REF: &str = "ph1x:expected:rewrite";
const STAGE8_5_EXPECT_TOOL_CHOICE_REF: &str = "ph1x:expected:tool_choice";
const STAGE8_5_REFERENCE_PREVIOUS_REF: &str = "ph1x:reference:previous_output";
const STAGE8_5_OPEN_SELECTION_REF: &str = "ph1x:open:selection";
const STAGE8_5_COMPARISON_SET_REF: &str = "ph1x:comparison:options";
const STAGE8_5_PROTECTED_RISK_REF: &str = "ph1x:protected_risk:business_workflow";
const STAGE8_5_FAIL_CLOSED_REF: &str = "ph1x:directive:fail_closed";

const STAGE8_5_PLANNING_MARKERS: &[&str] = &[
    "plan",
    "planning",
    "trip",
    "travel",
    "visit",
    "visiting",
    "holiday",
    "vacation",
    "itinerary",
    "destination",
    "destinations",
];
const STAGE8_5_ACTIVITY_MARKERS: &[&str] = &[
    "ski",
    "skiing",
    "snow",
    "restaurant",
    "restaurants",
    "food",
    "dining",
    "hiking",
    "beach",
    "museum",
    "museums",
    "gallery",
    "galleries",
    "art",
    "culture",
    "history",
    "shopping",
    "nightlife",
    "nature",
    "wine",
    "temple",
    "temples",
];
const STAGE8_5_SELECTION_TARGETS: &[&str] = &[
    "city",
    "cities",
    "area",
    "areas",
    "region",
    "regions",
    "place",
    "places",
    "destination",
    "destinations",
    "option",
    "options",
    "one",
    "where",
];
const STAGE8_5_TIMING_TARGETS: &[&str] = &[
    "when", "season", "seasons", "month", "months", "year", "years", "timing",
];
const STAGE8_5_TOOL_TIME_TERMS: &[&str] = &["time", "clock", "hour", "hours"];
const STAGE8_5_TOOL_WEATHER_TERMS: &[&str] =
    &["weather", "forecast", "temperature", "rain", "raining"];
const STAGE8_5_ARTIFACT_MARKERS: &[&str] = &[
    "write", "draft", "compose", "rewrite", "story", "message", "email", "note", "report",
    "summary", "letter",
];
const STAGE8_5_ARTIFACT_MODIFIERS: &[&str] = &[
    "short",
    "shorter",
    "shorten",
    "brief",
    "concise",
    "condensed",
    "compact",
    "tighten",
    "trim",
    "reduce",
    "summarize",
    "summarise",
    "summarized",
    "summarised",
    "version",
    "longer",
    "warmer",
    "colder",
    "friendlier",
    "professional",
    "formal",
    "casual",
    "darker",
    "lighter",
    "clearer",
    "simpler",
    "add",
    "remove",
    "include",
    "mention",
    "tone",
];
const STAGE8_5_REFERENCE_TERMS: &[&str] = &[
    "it", "that", "this", "same", "those", "these", "they", "them", "their", "he", "him", "his",
    "she", "her", "hers", "either", "again", "previous", "prior", "first", "other",
];
const STAGE8_5_DEFINITE_REFERENCE_MARKERS: &[&str] = &[
    "the", "that", "this", "those", "these", "either", "both", "same", "other", "first", "second",
    "previous", "prior",
];
const STAGE8_5_DEFINITE_REFERENCE_TARGETS: &[&str] = &[
    "answer",
    "answers",
    "area",
    "areas",
    "choice",
    "choices",
    "city",
    "cities",
    "country",
    "countries",
    "destination",
    "destinations",
    "draft",
    "email",
    "hotel",
    "hotels",
    "item",
    "items",
    "message",
    "option",
    "options",
    "person",
    "people",
    "place",
    "places",
    "plan",
    "restaurant",
    "restaurants",
    "role",
    "roles",
    "room",
    "rooms",
    "source",
    "sources",
    "story",
    "text",
    "trip",
    "one",
    "ones",
];
const STAGE8_5_CONFIRMATION_TERMS: &[&str] = &[
    "yes",
    "yep",
    "yeah",
    "confirm",
    "confirmed",
    "approve",
    "proceed",
    "continue",
    "go",
    "ahead",
    "please",
];
const STAGE8_5_STOP_TOKENS: &[&str] = &[
    "and", "or", "but", "with", "for", "because", "that", "which", "what", "where", "when", "who",
    "how", "do", "does", "did", "should", "would", "could", "can", "i", "we", "you", "me", "my",
    "our", "the", "a", "an", "some", "great", "nice", "good", "best", "doing", "saying", "tell",
    "joke",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stage8_5ToolFamily {
    Time,
    Weather,
}

impl Stage8_5ToolFamily {
    fn from_token(token: &str) -> Option<Self> {
        if STAGE8_5_TOOL_TIME_TERMS.contains(&token) {
            Some(Self::Time)
        } else if STAGE8_5_TOOL_WEATHER_TERMS.contains(&token) {
            Some(Self::Weather)
        } else {
            None
        }
    }

    fn prompt_noun(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Weather => "weather",
        }
    }

    fn ref_id(self) -> &'static str {
        match self {
            Self::Time => "time",
            Self::Weather => "weather",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stage8_5Features {
    normalized: String,
    tokens: Vec<String>,
    explicit_tool_family: Option<Stage8_5ToolFamily>,
    selected_tool_family: Option<Stage8_5ToolFamily>,
    entity_fragment: Option<String>,
    destination: Option<String>,
    constraints: Vec<String>,
    reference_followup: bool,
    definite_reference_followup: bool,
    selection_followup: bool,
    timing_followup: bool,
    prior_options_followup: bool,
    writing_seed: bool,
    writing_modifier: Option<String>,
    correction_target: Option<Stage8_5ToolFamily>,
    confirmation_followup: bool,
    new_topic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage8_5CandidateDecision {
    pub rewritten_query: Option<String>,
    pub selected_candidate: Option<Ph1xContextCandidate>,
    pub candidates: Vec<Ph1xContextCandidate>,
    pub rejection_ledger: Ph1xCandidateRejectionLedger,
    pub owner_output_contract: Ph1xOwnerOutputContract,
    pub active_context_packet: ActiveContextPacket,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slice3aProviderProposalOperation {
    OneLineRewrite,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slice3aProviderProposalTarget {
    PreviousAssistantAnswer,
    StaleOrWrongTarget,
    ProtectedAction,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice3aOneLineProviderProposal {
    pub provider_id: String,
    pub provider_enabled: bool,
    pub operation: Slice3aProviderProposalOperation,
    pub target: Slice3aProviderProposalTarget,
    pub likely_owner: SuggestedNextEngine,
    pub protected_risk: ProtectedRisk,
    pub provider_call_attempt_count: u32,
    pub provider_network_dispatch_count: u32,
    pub raw_provider_output_exposed: bool,
    pub protected_execution_authorized: bool,
    pub simulation_authorized: bool,
    pub authority_authorized: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slice3aOneLineValidationError {
    ProviderOff,
    ProviderNetworkDispatchAttempted,
    MalformedProposal,
    WrongTarget,
    WrongOwner,
    CurrentTurnHijack,
    MissingPreviousAssistantAnswer,
    ProtectedRiskRejected,
    RawProviderOutputRejected,
    ProviderAuthorityRejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slice3aOneLineValidatedDirective {
    pub directive: HumanConversationDirective,
    pub owner_engine: SuggestedNextEngine,
    pub target_ref: String,
    pub evidence_refs: Vec<String>,
    pub provider_id: String,
    pub provider_call_attempt_count: u32,
    pub provider_network_dispatch_count: u32,
    pub raw_provider_output_exposed: bool,
    pub protected_execution_authorized: bool,
    pub simulation_authorized: bool,
    pub authority_authorized: bool,
}

pub fn slice3a_validate_one_line_provider_contract_prep(
    thread_state: &ThreadState,
    current_user_text: &str,
    proposal: &Slice3aOneLineProviderProposal,
) -> Result<Slice3aOneLineValidatedDirective, Slice3aOneLineValidationError> {
    if !proposal.provider_enabled {
        return Err(Slice3aOneLineValidationError::ProviderOff);
    }
    if proposal.provider_id.trim().is_empty()
        || proposal.operation != Slice3aProviderProposalOperation::OneLineRewrite
    {
        return Err(Slice3aOneLineValidationError::MalformedProposal);
    }
    if proposal.provider_network_dispatch_count != 0 {
        return Err(Slice3aOneLineValidationError::ProviderNetworkDispatchAttempted);
    }
    if proposal.raw_provider_output_exposed {
        return Err(Slice3aOneLineValidationError::RawProviderOutputRejected);
    }
    if proposal.protected_execution_authorized
        || proposal.simulation_authorized
        || proposal.authority_authorized
    {
        return Err(Slice3aOneLineValidationError::ProviderAuthorityRejected);
    }
    if proposal.protected_risk == ProtectedRisk::Protected
        || proposal.target == Slice3aProviderProposalTarget::ProtectedAction
        || ph1x_stage8_5_protected_request_from_subject(thread_state).is_some()
    {
        return Err(Slice3aOneLineValidationError::ProtectedRiskRejected);
    }
    if proposal.target != Slice3aProviderProposalTarget::PreviousAssistantAnswer {
        return Err(Slice3aOneLineValidationError::WrongTarget);
    }
    if proposal.likely_owner != SuggestedNextEngine::Ph1Write {
        return Err(Slice3aOneLineValidationError::WrongOwner);
    }

    let current_features = ph1x_stage8_5_features(current_user_text);
    if current_features.new_topic
        && !ph1x_stage8_5c_has_reference_signal(&current_features)
        && current_features.writing_modifier.is_none()
    {
        return Err(Slice3aOneLineValidationError::CurrentTurnHijack);
    }

    let previous = thread_state
        .last_turn_context
        .as_ref()
        .filter(|context| context.route_class != LastTurnRouteClass::Clarify)
        .filter(|context| !context.answer_text.trim().is_empty())
        .ok_or(Slice3aOneLineValidationError::MissingPreviousAssistantAnswer)?;

    let answer_ref = format!(
        "ph1x:slice3a:previous_answer:{}",
        ph1x_stage8_5c_decision_suffix(&previous.answer_text)
    );
    Ok(Slice3aOneLineValidatedDirective {
        directive: HumanConversationDirective::ModifyPreviousOutput,
        owner_engine: SuggestedNextEngine::Ph1Write,
        target_ref: answer_ref,
        evidence_refs: vec![
            "ph1x:slice3a:provider_contract_prep".to_string(),
            "ph1x:target:previous_assistant_answer".to_string(),
            "ph1x:owner:PH1.WRITE".to_string(),
            format!(
                "provider:attempt_count:{}",
                proposal.provider_call_attempt_count
            ),
            format!(
                "provider:network_dispatch_count:{}",
                proposal.provider_network_dispatch_count
            ),
        ],
        provider_id: proposal.provider_id.clone(),
        provider_call_attempt_count: proposal.provider_call_attempt_count,
        provider_network_dispatch_count: proposal.provider_network_dispatch_count,
        raw_provider_output_exposed: false,
        protected_execution_authorized: false,
        simulation_authorized: false,
        authority_authorized: false,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stage8_5CandidateWork {
    candidate: Ph1xContextCandidate,
    rewritten_query: Option<String>,
}

pub fn ph1x_universal_active_context_followup_query(
    thread_state: &ThreadState,
    transcript_text: Option<&str>,
) -> Option<String> {
    ph1x_stage8_5c_candidate_decision(thread_state, transcript_text)?.rewritten_query
}

pub fn ph1x_stage8_5c_candidate_decision(
    thread_state: &ThreadState,
    transcript_text: Option<&str>,
) -> Option<Stage8_5CandidateDecision> {
    let text = transcript_text?.trim();
    if text.is_empty() || text.len() > 2048 {
        return None;
    }
    let features = ph1x_stage8_5_features(text);
    if features.normalized.is_empty() {
        return None;
    }

    let mut candidates = ph1x_stage8_5c_generate_candidates(thread_state, text, &features);
    if candidates.is_empty() {
        return None;
    }
    candidates.sort_by(|left, right| {
        right
            .candidate
            .score
            .cmp(&left.candidate.score)
            .then_with(|| {
                left.candidate
                    .candidate_ref
                    .cmp(&right.candidate.candidate_ref)
            })
    });
    let selected_index = ph1x_stage8_5c_select_candidate_index(&candidates);
    let selected = selected_index.and_then(|idx| candidates.get(idx).cloned());
    let selected_candidate_ref = selected
        .as_ref()
        .map(|work| work.candidate.candidate_ref.clone());
    let selected_score = selected
        .as_ref()
        .map(|work| work.candidate.score)
        .unwrap_or(0);
    let selected_owner = selected
        .as_ref()
        .map(|work| work.candidate.owner_engine)
        .unwrap_or(SuggestedNextEngine::None);
    let selected_directive = selected
        .as_ref()
        .map(|work| work.candidate.directive)
        .unwrap_or(HumanConversationDirective::AnswerNewQuestion);
    let selected_protected_risk = selected
        .as_ref()
        .map(|work| work.candidate.protected_risk)
        .unwrap_or(ProtectedRisk::None);
    let ambiguity = ph1x_stage8_5c_ambiguity_for_score(selected_score, selected_protected_risk);
    let decision_suffix = ph1x_stage8_5c_decision_suffix(&features.normalized);
    let rejected_candidates_ref = format!("ph1x_rejected_candidates:{decision_suffix}");
    let ledger_ref = format!("ph1x_candidate_ledger:{decision_suffix}");
    let owner_contract_ref = format!("ph1x_owner_output:{decision_suffix}");
    let threshold_ref = "ph1x_threshold:stage8_5c:high_8500_medium_6500".to_string();
    let evidence_refs = ph1x_stage8_5c_decision_evidence_refs(
        thread_state,
        &features,
        &ledger_ref,
        &threshold_ref,
        &owner_contract_ref,
    );
    let allowed_next_action = ph1x_stage8_5c_allowed_action(selected_directive);
    let blocked_actions = ph1x_stage8_5c_blocked_actions(selected_protected_risk);
    let reason_code = ph1x_stage8_5c_reason_code(selected_directive, selected_owner);
    let rejections = ph1x_stage8_5c_rejections(
        &candidates,
        selected_candidate_ref.as_deref(),
        ambiguity,
        selected_score,
        &features,
    )?;

    let ledger = Ph1xCandidateRejectionLedger::v1(
        selected_candidate_ref.clone(),
        rejections,
        selected_directive,
        selected_owner,
        allowed_next_action.clone(),
        blocked_actions.clone(),
        reason_code,
        evidence_refs.clone(),
        selected_score,
        ambiguity,
        selected_protected_risk,
    )
    .ok()?;
    let owner_output_contract = Ph1xOwnerOutputContract::v1(
        selected_directive,
        selected_owner,
        allowed_next_action.clone(),
        blocked_actions.clone(),
        reason_code,
        evidence_refs.clone(),
        selected_candidate_ref.clone(),
        Some(rejected_candidates_ref.clone()),
        selected_score,
        ambiguity,
        selected_protected_risk,
    )
    .ok()?;
    let active_context_packet = ph1x_stage8_5c_active_context_packet(
        thread_state,
        &features,
        selected.as_ref().map(|work| &work.candidate),
        &rejected_candidates_ref,
        &ledger_ref,
        &threshold_ref,
        &owner_contract_ref,
        reason_code,
        evidence_refs,
        allowed_next_action,
        blocked_actions,
        ambiguity,
    )?;
    Some(Stage8_5CandidateDecision {
        rewritten_query: selected
            .as_ref()
            .and_then(|work| work.rewritten_query.clone()),
        selected_candidate: selected.map(|work| work.candidate),
        candidates: candidates
            .iter()
            .map(|work| work.candidate.clone())
            .collect(),
        rejection_ledger: ledger,
        owner_output_contract,
        active_context_packet,
    })
}

fn ph1x_stage8_5c_generate_candidates(
    thread_state: &ThreadState,
    text: &str,
    features: &Stage8_5Features,
) -> Vec<Stage8_5CandidateWork> {
    let mut out = Vec::new();
    let topic_switch_disqualifier = features
        .new_topic
        .then_some(Ph1xCandidateRejectionReasonCode::HardDisqualifierExplicitTopicSwitch);

    if let Some(prior_request) = ph1x_stage8_5_protected_request_from_subject(thread_state) {
        if features.confirmation_followup {
            out.push(ph1x_stage8_5c_candidate(
                "ph1x_candidate:protected_execution",
                Ph1xContextCandidateKind::ProtectedFailClosed,
                Some("protected_execution_without_authority".to_string()),
                HumanConversationDirective::FailClosedProtected,
                SuggestedNextEngine::ProtectedBoundary,
                ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                    semantic_fit: 9_800,
                    task_fit: 9_800,
                    recency_score: 9_500,
                    discourse_fit: 9_700,
                    risk_penalty: 10_000,
                    ..Default::default()
                }),
                Ph1xCandidateScoreFactors {
                    semantic_fit: 9_800,
                    task_fit: 9_800,
                    recency_score: 9_500,
                    discourse_fit: 9_700,
                    risk_penalty: 10_000,
                    ..Default::default()
                },
                ProtectedRisk::Protected,
                vec![
                    STAGE8_5_PROTECTED_RISK_REF.to_string(),
                    STAGE8_5_FAIL_CLOSED_REF.to_string(),
                ],
                Some(Ph1xCandidateRejectionReasonCode::HardDisqualifierProtectedRisk),
                None,
            ));
            out.push(ph1x_stage8_5c_candidate(
                "ph1x_candidate:protected_fail_closed",
                Ph1xContextCandidateKind::ProtectedFailClosed,
                Some("protected_fail_closed_boundary".to_string()),
                HumanConversationDirective::FailClosedProtected,
                SuggestedNextEngine::ProtectedBoundary,
                ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                    semantic_fit: 10_000,
                    task_fit: 10_000,
                    recency_score: 9_500,
                    discourse_fit: 10_000,
                    privacy_scope_fit: 10_000,
                    ..Default::default()
                }),
                Ph1xCandidateScoreFactors {
                    semantic_fit: 10_000,
                    task_fit: 10_000,
                    recency_score: 9_500,
                    discourse_fit: 10_000,
                    privacy_scope_fit: 10_000,
                    ..Default::default()
                },
                ProtectedRisk::Protected,
                vec![
                    STAGE8_5_PROTECTED_RISK_REF.to_string(),
                    STAGE8_5_FAIL_CLOSED_REF.to_string(),
                ],
                None,
                Some(prior_request),
            ));
        }
    }

    if let Some(target) = ph1x_stage8_5_tool_choice_target(thread_state) {
        if let Some(tool_family) = features.selected_tool_family {
            out.push(ph1x_stage8_5c_tool_candidate(
                "ph1x_candidate:open_clarification",
                Ph1xContextCandidateKind::OpenClarification,
                tool_family,
                &target,
                10_000,
                10_000,
                None,
            ));
        }
    }

    if let Some(target) = thread_state
        .last_turn_context
        .as_ref()
        .and_then(|context| ph1x_stage8_5_tool_choice_from_response(&context.answer_text))
        .map(|choice| choice.target)
    {
        if let Some(tool_family) = features.selected_tool_family {
            out.push(ph1x_stage8_5c_tool_candidate(
                "ph1x_candidate:last_answer_clarification",
                Ph1xContextCandidateKind::OpenClarification,
                tool_family,
                &target,
                9_700,
                10_000,
                None,
            ));
        }
    }

    if let Some(tool_family) = features.correction_target {
        if let Some(place) = ph1x_stage8_5_last_tool_entity(thread_state) {
            out.push(ph1x_stage8_5c_tool_candidate(
                "ph1x_candidate:correction_target",
                Ph1xContextCandidateKind::CorrectionTarget,
                tool_family,
                &place,
                9_600,
                9_200,
                topic_switch_disqualifier,
            ));
        }
    }

    if ph1x_stage8_5_has_planning_frame(thread_state)
        && (features.selection_followup
            || features.prior_options_followup
            || features.timing_followup
            || features.reference_followup
            || features.definite_reference_followup
            || !features.constraints.is_empty())
    {
        let target_ref = ph1x_stage8_5_join_or_default(
            &ph1x_stage8_5_frame_values(thread_state, "ph1x:topic:"),
            "active_planning_frame",
        );
        out.push(ph1x_stage8_5c_candidate(
            "ph1x_candidate:active_plan",
            Ph1xContextCandidateKind::ActivePlan,
            Some(target_ref),
            HumanConversationDirective::RouteToWrite,
            SuggestedNextEngine::Ph1Write,
            ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                semantic_fit: 9_200,
                task_fit: 9_400,
                entity_fit: 8_200,
                open_slot_fit: 9_300,
                recency_score: 8_800,
                topic_stack_score: 9_000,
                discourse_fit: 9_400,
                ambiguity_penalty: if features.prior_options_followup {
                    0
                } else {
                    600
                },
                ..Default::default()
            }),
            Ph1xCandidateScoreFactors {
                semantic_fit: 9_200,
                task_fit: 9_400,
                entity_fit: 8_200,
                open_slot_fit: 9_300,
                recency_score: 8_800,
                topic_stack_score: 9_000,
                discourse_fit: 9_400,
                ambiguity_penalty: if features.prior_options_followup {
                    0
                } else {
                    600
                },
                ..Default::default()
            },
            ProtectedRisk::None,
            ph1x_stage8_5c_frame_evidence_refs(thread_state),
            topic_switch_disqualifier,
            Some(ph1x_stage8_5_planning_prompt(
                thread_state,
                text,
                features.timing_followup,
                features.prior_options_followup,
                features.reference_followup || features.definite_reference_followup,
            )),
        ));
    }

    if let Some(entity) = &features.entity_fragment {
        if let Some(route_tool) = thread_state
            .last_turn_context
            .as_ref()
            .and_then(|context| ph1x_stage8_5_tool_family_from_route(context.route_class))
        {
            out.push(ph1x_stage8_5c_tool_candidate(
                "ph1x_candidate:active_tool_result",
                Ph1xContextCandidateKind::ActiveToolResult,
                route_tool,
                entity,
                9_700,
                9_500,
                topic_switch_disqualifier,
            ));
        }
    }

    if ph1x_stage8_5_has_writing_frame(thread_state) {
        if let Some(modifier) = &features.writing_modifier {
            out.push(ph1x_stage8_5c_candidate(
                "ph1x_candidate:active_writing_artifact",
                Ph1xContextCandidateKind::ActiveWritingArtifact,
                Some(STAGE8_5_WRITING_SUBJECT.to_string()),
                HumanConversationDirective::ModifyPreviousOutput,
                SuggestedNextEngine::Ph1Write,
                ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                    semantic_fit: 9_200,
                    task_fit: 9_500,
                    artifact_fit: 10_000,
                    recency_score: 9_000,
                    discourse_fit: 9_200,
                    ..Default::default()
                }),
                Ph1xCandidateScoreFactors {
                    semantic_fit: 9_200,
                    task_fit: 9_500,
                    artifact_fit: 10_000,
                    recency_score: 9_000,
                    discourse_fit: 9_200,
                    ..Default::default()
                },
                ProtectedRisk::None,
                ph1x_stage8_5c_frame_evidence_refs(thread_state),
                topic_switch_disqualifier,
                Some(ph1x_stage8_5_rewrite_previous_artifact_prompt(
                    thread_state,
                    text,
                    modifier,
                )),
            ));
        }
    }

    if ph1x_stage8_5_has_planning_frame(thread_state) {
        out.push(ph1x_stage8_5c_candidate(
            "ph1x_candidate:active_frame_observed",
            Ph1xContextCandidateKind::TopicStack,
            Some(STAGE8_5_PLANNING_SUBJECT.to_string()),
            HumanConversationDirective::ContinueCurrentTopic,
            SuggestedNextEngine::Ph1Write,
            ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                semantic_fit: 4_800,
                task_fit: 4_200,
                recency_score: 8_000,
                topic_stack_score: 7_000,
                ambiguity_penalty: 3_000,
                ..Default::default()
            }),
            Ph1xCandidateScoreFactors {
                semantic_fit: 4_800,
                task_fit: 4_200,
                recency_score: 8_000,
                topic_stack_score: 7_000,
                ambiguity_penalty: 3_000,
                ..Default::default()
            },
            ProtectedRisk::None,
            ph1x_stage8_5c_frame_evidence_refs(thread_state),
            topic_switch_disqualifier,
            None,
        ));
    }

    if thread_state
        .last_turn_context
        .as_ref()
        .and_then(|context| ph1x_stage8_5_tool_family_from_route(context.route_class))
        .is_some()
    {
        out.push(ph1x_stage8_5c_candidate(
            "ph1x_candidate:latest_tool_context_observed",
            Ph1xContextCandidateKind::LatestSeleneAnswer,
            Some("latest_tool_answer".to_string()),
            HumanConversationDirective::RouteToTool,
            SuggestedNextEngine::Ph1E,
            ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                semantic_fit: 4_600,
                task_fit: 4_200,
                tool_family_fit: 7_000,
                recency_score: 8_000,
                ambiguity_penalty: 3_200,
                ..Default::default()
            }),
            Ph1xCandidateScoreFactors {
                semantic_fit: 4_600,
                task_fit: 4_200,
                tool_family_fit: 7_000,
                recency_score: 8_000,
                ambiguity_penalty: 3_200,
                ..Default::default()
            },
            ProtectedRisk::None,
            ph1x_stage8_5c_frame_evidence_refs(thread_state),
            topic_switch_disqualifier,
            None,
        ));
    }

    let reference_signal = features.reference_followup || features.definite_reference_followup;
    if reference_signal
        && !features.new_topic
        && !ph1x_stage8_5_has_planning_frame(thread_state)
        && !ph1x_stage8_5_has_writing_frame(thread_state)
    {
        if let Some(context) = thread_state
            .last_turn_context
            .as_ref()
            .filter(|context| context.route_class != LastTurnRouteClass::Clarify)
            .filter(|context| !context.answer_text.trim().is_empty())
        {
            let factors = Ph1xCandidateScoreFactors {
                semantic_fit: 8_800,
                task_fit: 8_700,
                recency_score: 9_000,
                discourse_fit: 9_200,
                entity_fit: 7_800,
                ..Default::default()
            };
            out.push(ph1x_stage8_5c_candidate(
                "ph1x_candidate:latest_answer_context",
                Ph1xContextCandidateKind::LatestSeleneAnswer,
                Some("latest_selene_answer".to_string()),
                HumanConversationDirective::RouteToWrite,
                SuggestedNextEngine::Ph1Write,
                ph1x_stage8_5c_score_factors(&factors),
                factors,
                ProtectedRisk::None,
                vec![
                    "ph1x:latest_answer_context".to_string(),
                    format!("ph1x:last_answer_type:{:?}", context.route_class),
                ],
                topic_switch_disqualifier,
                Some(ph1x_stage8_5_latest_answer_prompt(
                    &context.answer_text,
                    text,
                )),
            ));
        }
    }

    if !ph1x_stage8_5c_has_hot_context(thread_state)
        && ph1x_stage8_5c_has_reference_signal(features)
    {
        out.push(ph1x_stage8_5c_candidate(
            "ph1x_candidate:fresh_memory_handoff",
            Ph1xContextCandidateKind::FreshMemoryHandoff,
            Some("fresh_memory_needed".to_string()),
            HumanConversationDirective::HandOffToMemory,
            SuggestedNextEngine::Ph1M,
            ph1x_stage8_5c_score_factors(&Ph1xCandidateScoreFactors {
                semantic_fit: 7_600,
                task_fit: 7_400,
                discourse_fit: 7_600,
                recency_score: 7_000,
                ..Default::default()
            }),
            Ph1xCandidateScoreFactors {
                semantic_fit: 7_600,
                task_fit: 7_400,
                discourse_fit: 7_600,
                recency_score: 7_000,
                ..Default::default()
            },
            ProtectedRisk::None,
            vec!["ph1m:fresh_memory_handoff_candidate".to_string()],
            None,
            None,
        ));
    }

    let fallback_score = if features.new_topic {
        9_200
    } else if out.is_empty() {
        8_400
    } else {
        4_500
    };
    out.push(ph1x_stage8_5c_candidate(
        "ph1x_candidate:new_topic_fallback",
        Ph1xContextCandidateKind::NewTopicFallback,
        Some("current_turn_as_new_topic".to_string()),
        HumanConversationDirective::AnswerNewQuestion,
        SuggestedNextEngine::Ph1Write,
        fallback_score,
        Ph1xCandidateScoreFactors {
            semantic_fit: fallback_score,
            task_fit: fallback_score,
            discourse_fit: fallback_score,
            ..Default::default()
        },
        ProtectedRisk::None,
        vec!["ph1x:no_context_or_topic_switch_fallback".to_string()],
        None,
        None,
    ));

    out
}

fn ph1x_stage8_5c_candidate(
    candidate_ref: &str,
    candidate_kind: Ph1xContextCandidateKind,
    target_ref: Option<String>,
    directive: HumanConversationDirective,
    owner_engine: SuggestedNextEngine,
    score: u16,
    score_factors: Ph1xCandidateScoreFactors,
    protected_risk: ProtectedRisk,
    evidence_refs: Vec<String>,
    disqualifier_applied: Option<Ph1xCandidateRejectionReasonCode>,
    rewritten_query: Option<String>,
) -> Stage8_5CandidateWork {
    Stage8_5CandidateWork {
        candidate: Ph1xContextCandidate::v1(
            candidate_ref.to_string(),
            candidate_kind,
            target_ref,
            directive,
            owner_engine,
            score,
            score_factors,
            protected_risk,
            evidence_refs,
            disqualifier_applied,
        )
        .expect("PH1.X Stage 8.5C candidate proof must validate"),
        rewritten_query,
    }
}

fn ph1x_stage8_5c_tool_candidate(
    candidate_ref: &str,
    candidate_kind: Ph1xContextCandidateKind,
    tool_family: Stage8_5ToolFamily,
    target: &str,
    semantic_fit: u16,
    task_fit: u16,
    disqualifier_applied: Option<Ph1xCandidateRejectionReasonCode>,
) -> Stage8_5CandidateWork {
    let factors = Ph1xCandidateScoreFactors {
        semantic_fit,
        task_fit,
        entity_fit: 9_300,
        tool_family_fit: 10_000,
        recency_score: 9_000,
        discourse_fit: 9_000,
        correction_fit: if candidate_kind == Ph1xContextCandidateKind::CorrectionTarget {
            9_500
        } else {
            0
        },
        clarification_fit: if candidate_kind == Ph1xContextCandidateKind::OpenClarification {
            10_000
        } else {
            0
        },
        ..Default::default()
    };
    ph1x_stage8_5c_candidate(
        candidate_ref,
        candidate_kind,
        Some(ph1x_stage8_5_make_ref("ph1x:entity", target)),
        if candidate_kind == Ph1xContextCandidateKind::CorrectionTarget {
            HumanConversationDirective::CorrectPreviousOutput
        } else {
            HumanConversationDirective::RouteToTool
        },
        SuggestedNextEngine::Ph1E,
        ph1x_stage8_5c_score_factors(&factors),
        factors,
        ProtectedRisk::None,
        vec![
            format!("ph1x:tool_family:{}", tool_family.ref_id()),
            ph1x_stage8_5_make_ref("ph1x:entity", target),
        ],
        disqualifier_applied,
        Some(ph1x_stage8_5_tool_prompt(tool_family, target)),
    )
}

fn ph1x_stage8_5c_score_factors(factors: &Ph1xCandidateScoreFactors) -> u16 {
    let positives = [
        factors.semantic_fit,
        factors.task_fit,
        factors.entity_fit,
        factors.artifact_fit,
        factors.tool_family_fit,
        factors.open_slot_fit,
        factors.recency_score,
        factors.speaker_continuity_score,
        factors.topic_stack_score,
        factors.discourse_fit,
        factors.clarification_fit,
        factors.correction_fit,
        factors.privacy_scope_fit,
    ];
    let mut positive_total = 0u32;
    let mut positive_count = 0u32;
    for value in positives {
        if value > 0 {
            positive_total = positive_total.saturating_add(value as u32);
            positive_count = positive_count.saturating_add(1);
        }
    }
    let positive = if positive_count == 0 {
        0
    } else {
        positive_total / positive_count
    };
    let penalty = [
        factors.risk_penalty,
        factors.ambiguity_penalty,
        factors.stale_context_penalty,
    ]
    .into_iter()
    .max()
    .unwrap_or(0) as u32;
    positive.saturating_sub(penalty).min(10_000) as u16
}

fn ph1x_stage8_5c_select_candidate_index(candidates: &[Stage8_5CandidateWork]) -> Option<usize> {
    let best = candidates
        .iter()
        .enumerate()
        .filter(|(_, work)| work.candidate.disqualifier_applied.is_none())
        .max_by_key(|(_, work)| work.candidate.score)?;
    if best.1.candidate.protected_risk == ProtectedRisk::Protected
        && best.1.candidate.owner_engine == SuggestedNextEngine::ProtectedBoundary
    {
        return Some(best.0);
    }
    if best.1.candidate.score >= 6_500 {
        Some(best.0)
    } else {
        candidates
            .iter()
            .enumerate()
            .find(|(_, work)| {
                work.candidate.candidate_kind == Ph1xContextCandidateKind::NewTopicFallback
                    && work.candidate.disqualifier_applied.is_none()
            })
            .map(|(idx, _)| idx)
    }
}

fn ph1x_stage8_5c_ambiguity_for_score(score: u16, protected_risk: ProtectedRisk) -> AmbiguityLevel {
    if protected_risk == ProtectedRisk::Protected {
        AmbiguityLevel::Medium
    } else if score >= 8_500 {
        AmbiguityLevel::Low
    } else if score >= 6_500 {
        AmbiguityLevel::Medium
    } else {
        AmbiguityLevel::High
    }
}

fn ph1x_stage8_5c_decision_evidence_refs(
    thread_state: &ThreadState,
    features: &Stage8_5Features,
    ledger_ref: &str,
    threshold_ref: &str,
    owner_contract_ref: &str,
) -> Vec<String> {
    let mut refs = vec![
        ledger_ref.to_string(),
        threshold_ref.to_string(),
        owner_contract_ref.to_string(),
        format!(
            "ph1x:turn_hash:{}",
            ph1x_stage8_5c_decision_suffix(&features.normalized)
        ),
    ];
    refs.extend(ph1x_stage8_5c_frame_evidence_refs(thread_state));
    refs.sort();
    refs.dedup();
    refs.into_iter().take(12).collect()
}

fn ph1x_stage8_5c_frame_evidence_refs(thread_state: &ThreadState) -> Vec<String> {
    let mut refs = Vec::new();
    if let Some(active_subject_ref) = &thread_state.active_subject_ref {
        refs.push(active_subject_ref.clone());
    }
    if let Some(interrupted_subject_ref) = &thread_state.interrupted_subject_ref {
        refs.push(interrupted_subject_ref.clone());
    }
    refs.extend(thread_state.pinned_context_refs.iter().take(8).cloned());
    refs.sort();
    refs.dedup();
    refs.into_iter().take(12).collect()
}

fn ph1x_stage8_5c_allowed_action(directive: HumanConversationDirective) -> Option<String> {
    Some(
        match directive {
            HumanConversationDirective::ContinueCurrentTopic => "continue_current_topic",
            HumanConversationDirective::ModifyPreviousOutput => "route_to_ph1write_modify",
            HumanConversationDirective::CorrectPreviousOutput => "route_to_owner_with_correction",
            HumanConversationDirective::AnswerNewQuestion => "answer_new_question",
            HumanConversationDirective::AskClarification => "ask_clarification",
            HumanConversationDirective::HandOffToMemory => "handoff_to_ph1m",
            HumanConversationDirective::RouteToTool => "route_to_ph1e",
            HumanConversationDirective::RouteToWrite => "route_to_ph1write",
            HumanConversationDirective::FailClosedProtected => "fail_closed_protected",
            HumanConversationDirective::WaitOrNoAction => "wait_or_no_action",
        }
        .to_string(),
    )
}

fn ph1x_stage8_5c_blocked_actions(protected_risk: ProtectedRisk) -> Vec<String> {
    if protected_risk == ProtectedRisk::Protected {
        vec![
            "protected_execution_without_simulation".to_string(),
            "protected_execution_without_authority".to_string(),
            "memory_or_voice_id_authority_grant".to_string(),
        ]
    } else {
        vec!["protected_execute".to_string()]
    }
}

fn ph1x_stage8_5c_reason_code(
    directive: HumanConversationDirective,
    owner_engine: SuggestedNextEngine,
) -> ReasonCodeId {
    match (directive, owner_engine) {
        (HumanConversationDirective::RouteToTool, SuggestedNextEngine::Ph1E) => {
            reason_codes::X_STAGE8_5C_CONTINUE_TOOL
        }
        (HumanConversationDirective::RouteToWrite, SuggestedNextEngine::Ph1Write) => {
            reason_codes::X_STAGE8_5C_CONTINUE_PLAN
        }
        (HumanConversationDirective::ModifyPreviousOutput, SuggestedNextEngine::Ph1Write) => {
            reason_codes::X_STAGE8_5C_MODIFY_ARTIFACT
        }
        (HumanConversationDirective::CorrectPreviousOutput, _) => {
            reason_codes::X_STAGE8_5C_CORRECT_TOOL
        }
        (HumanConversationDirective::AskClarification, _) => {
            reason_codes::X_STAGE8_5C_ASK_CLARIFICATION
        }
        (HumanConversationDirective::HandOffToMemory, SuggestedNextEngine::Ph1M) => {
            reason_codes::X_STAGE8_5C_HANDOFF_MEMORY
        }
        (
            HumanConversationDirective::FailClosedProtected,
            SuggestedNextEngine::ProtectedBoundary,
        ) => reason_codes::X_STAGE8_5C_FAIL_CLOSED_PROTECTED,
        _ => reason_codes::X_STAGE8_5C_ANSWER_NEW_TOPIC,
    }
}

fn ph1x_stage8_5c_rejections(
    candidates: &[Stage8_5CandidateWork],
    selected_candidate_ref: Option<&str>,
    ambiguity_level: AmbiguityLevel,
    selected_score: u16,
    features: &Stage8_5Features,
) -> Option<Vec<Ph1xCandidateRejection>> {
    let mut out = Vec::new();
    for work in candidates {
        if selected_candidate_ref.is_some_and(|selected| selected == work.candidate.candidate_ref) {
            continue;
        }
        let reason_code = work.candidate.disqualifier_applied.unwrap_or_else(|| {
            if work.candidate.score < 6_500 {
                Ph1xCandidateRejectionReasonCode::BelowMinimumEvidenceThreshold
            } else {
                Ph1xCandidateRejectionReasonCode::LowerScore
            }
        });
        let reason_text = match reason_code {
            Ph1xCandidateRejectionReasonCode::LowerScore => {
                "higher scoring candidate had stronger active-frame fit"
            }
            Ph1xCandidateRejectionReasonCode::BelowMinimumEvidenceThreshold => {
                "candidate did not meet the minimum evidence threshold"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierProtectedRisk => {
                "protected work cannot execute without simulation and authority"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierSpeakerPrivacyMismatch => {
                "speaker or privacy scope did not authorize private continuation"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierRejectedEvidence => {
                "rejected input evidence cannot become context"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierStaleContext => {
                "context was too stale for automatic continuation"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierExplicitTopicSwitch => {
                "current turn is a clear topic switch"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierWrongArtifactType => {
                "candidate target was the wrong artifact type"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierClosedTopic => {
                "candidate belongs to a closed topic"
            }
            Ph1xCandidateRejectionReasonCode::HardDisqualifierUnsupportedEvidence => {
                "candidate depends on unsupported evidence"
            }
        };
        out.push(
            Ph1xCandidateRejection::v1(
                work.candidate.candidate_ref.clone(),
                reason_code,
                Some(reason_text.to_string()),
                work.candidate.disqualifier_applied,
                work.candidate.owner_engine,
                work.candidate.protected_risk,
                ambiguity_level,
                selected_score,
                Some(ph1x_stage8_5c_confidence_reason(features, selected_score)),
                None,
                Some(reason_text.to_string()),
                work.candidate.evidence_refs.clone(),
            )
            .ok()?,
        );
    }
    Some(out)
}

#[allow(clippy::too_many_arguments)]
fn ph1x_stage8_5c_active_context_packet(
    thread_state: &ThreadState,
    features: &Stage8_5Features,
    selected: Option<&Ph1xContextCandidate>,
    rejected_candidates_ref: &str,
    ledger_ref: &str,
    threshold_ref: &str,
    owner_contract_ref: &str,
    reason_code: ReasonCodeId,
    evidence_refs: Vec<String>,
    allowed_next_action: Option<String>,
    blocked_actions: Vec<String>,
    ambiguity_level: AmbiguityLevel,
) -> Option<ActiveContextPacket> {
    let selected = selected?;
    let tool_family = selected
        .evidence_refs
        .iter()
        .find_map(|reference| reference.strip_prefix("ph1x:tool_family:"))
        .map(ToOwned::to_owned);
    let entity_focus = selected
        .target_ref
        .iter()
        .map(|target| target.trim_start_matches("ph1x:entity:").replace('_', " "))
        .collect::<Vec<_>>();
    let packet = ActiveContextPacket::v1(
        ph1x_stage8_5c_active_topic(thread_state, selected),
        Some(ph1x_stage8_5c_active_intent(selected)),
        ph1x_stage8_5c_interaction_posture(features, selected),
        ph1x_stage8_5c_conversation_rhythm(selected.directive),
        ph1x_stage8_5c_continuation_type(selected.directive),
        selected.target_ref.clone(),
        entity_focus,
        tool_family,
        (selected.candidate_kind == Ph1xContextCandidateKind::ActiveWritingArtifact)
            .then_some(STAGE8_5_WRITING_SUBJECT.to_string()),
        ph1x_stage8_5c_pending_slots(selected),
        (selected.candidate_kind == Ph1xContextCandidateKind::CorrectionTarget).then(|| {
            selected
                .target_ref
                .clone()
                .unwrap_or_else(|| "prior_answer".to_string())
        }),
        ph1x_stage8_5c_topic_stack(thread_state),
        ph1x_stage8_5c_response_shape(selected.directive),
        selected.score,
        ambiguity_level,
        selected.protected_risk,
        selected.candidate_kind == Ph1xContextCandidateKind::FreshMemoryHandoff,
        selected.owner_engine,
        evidence_refs,
    )
    .ok()?
    .with_universal_frame(UniversalActiveFrameFields {
        raw_user_turn_ref: Some(format!(
            "ph1x:raw_turn_hash:{}",
            ph1x_stage8_5c_decision_suffix(&features.normalized)
        )),
        normalized_user_turn_ref: Some(format!(
            "ph1x:normalized_turn_hash:{}",
            ph1x_stage8_5c_decision_suffix(&features.normalized)
        )),
        modality: Some("typed_or_voice_committed_turn".to_string()),
        user_goal: ph1x_stage8_5c_user_goal(thread_state, features),
        current_plan: ph1x_stage8_5c_current_plan(thread_state),
        open_question: ph1x_stage8_5c_open_question(selected),
        unresolved_decision: ph1x_stage8_5c_unresolved_decision(selected),
        prior_options_presented: ph1x_stage8_5_frame_values(thread_state, "ph1x:option:"),
        comparison_set: ph1x_stage8_5_frame_values(thread_state, "ph1x:comparison:"),
        constraints: ph1x_stage8_5_frame_values(thread_state, "ph1x:constraint:"),
        user_preference_in_turn: (!features.constraints.is_empty())
            .then(|| features.constraints.join(",")),
        expected_answer_type: ph1x_stage8_5c_expected_answer_type(selected),
        last_answer_type: thread_state
            .last_turn_context
            .as_ref()
            .map(|context| format!("{:?}", context.route_class)),
        last_clarification_question: (selected.candidate_kind
            == Ph1xContextCandidateKind::OpenClarification)
            .then(|| "open tool-family clarification".to_string()),
        clarification_answer_target: (selected.candidate_kind
            == Ph1xContextCandidateKind::OpenClarification)
            .then(|| {
                selected
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "pending_target".to_string())
            }),
        discourse_state: Some(ph1x_stage8_5c_discourse_state(selected).to_string()),
        topic_depth: ph1x_stage8_5c_topic_stack(thread_state).len() as u16,
        returnable_topic: thread_state.interrupted_subject_ref.clone(),
        speaker_continuity: Some("same_or_nullable".to_string()),
        confidence_reason: Some(ph1x_stage8_5c_confidence_reason(features, selected.score)),
        why_continue_reason: (selected.directive != HumanConversationDirective::AnswerNewQuestion)
            .then(|| "selected candidate met active-frame evidence threshold".to_string()),
        why_not_continue_reason: (selected.directive
            == HumanConversationDirective::AnswerNewQuestion)
            .then(|| "new-topic fallback selected or old context rejected".to_string()),
        selected_candidate: Some(selected.candidate_ref.clone()),
        rejected_candidates_ref: Some(rejected_candidates_ref.to_string()),
        candidate_rejection_ledger_ref: Some(ledger_ref.to_string()),
        minimum_evidence_threshold_ref: Some(threshold_ref.to_string()),
        owner_engine: Some(ph1x_stage8_5c_engine_label(selected.owner_engine).to_string()),
        allowed_next_action,
        blocked_actions,
        reason_code: Some(reason_code),
        ..Default::default()
    })
    .ok()?;
    let mut packet = packet;
    packet.evidence_refs.push(owner_contract_ref.to_string());
    packet.validate().ok()?;
    Some(packet)
}

fn ph1x_stage8_5c_continuation_type(directive: HumanConversationDirective) -> ContinuationType {
    match directive {
        HumanConversationDirective::ContinueCurrentTopic
        | HumanConversationDirective::RouteToTool
        | HumanConversationDirective::RouteToWrite => ContinuationType::ContinueCurrentTopic,
        HumanConversationDirective::ModifyPreviousOutput => ContinuationType::ModifyPreviousOutput,
        HumanConversationDirective::CorrectPreviousOutput => {
            ContinuationType::CorrectPreviousOutput
        }
        HumanConversationDirective::AskClarification => ContinuationType::AskClarification,
        HumanConversationDirective::HandOffToMemory => ContinuationType::HandOffToMemory,
        HumanConversationDirective::WaitOrNoAction => ContinuationType::NoActionRequired,
        HumanConversationDirective::AnswerNewQuestion
        | HumanConversationDirective::FailClosedProtected => ContinuationType::AnswerNewTopic,
    }
}

fn ph1x_stage8_5c_response_shape(directive: HumanConversationDirective) -> ResponseShape {
    match directive {
        HumanConversationDirective::AskClarification => ResponseShape::OneQuestionClarification,
        HumanConversationDirective::ModifyPreviousOutput => ResponseShape::RewriteOrModification,
        HumanConversationDirective::FailClosedProtected => ResponseShape::SafeRefusal,
        HumanConversationDirective::WaitOrNoAction => ResponseShape::WaitOrNoAction,
        HumanConversationDirective::RouteToWrite
        | HumanConversationDirective::ContinueCurrentTopic => ResponseShape::StructuredAnswer,
        _ => ResponseShape::DirectAnswer,
    }
}

fn ph1x_stage8_5c_conversation_rhythm(directive: HumanConversationDirective) -> ConversationRhythm {
    match directive {
        HumanConversationDirective::AskClarification => {
            ConversationRhythm::OneQuestionClarification
        }
        HumanConversationDirective::ModifyPreviousOutput => {
            ConversationRhythm::RewriteOrModification
        }
        HumanConversationDirective::FailClosedProtected => ConversationRhythm::ProtectedFailClosed,
        HumanConversationDirective::HandOffToMemory => ConversationRhythm::MemoryRecall,
        HumanConversationDirective::WaitOrNoAction => ConversationRhythm::WaitOrNoAction,
        _ => ConversationRhythm::DirectAnswer,
    }
}

fn ph1x_stage8_5c_interaction_posture(
    features: &Stage8_5Features,
    selected: &Ph1xContextCandidate,
) -> InteractionPosture {
    if selected.protected_risk == ProtectedRisk::Protected {
        InteractionPosture::ActionRequest
    } else if selected.candidate_kind == Ph1xContextCandidateKind::CorrectionTarget {
        InteractionPosture::Correction
    } else if selected.candidate_kind == Ph1xContextCandidateKind::ActiveWritingArtifact {
        InteractionPosture::Instruction
    } else if features.new_topic {
        InteractionPosture::TopicSwitch
    } else if selected.directive == HumanConversationDirective::HandOffToMemory {
        InteractionPosture::MemoryRequest
    } else {
        InteractionPosture::Continuation
    }
}

fn ph1x_stage8_5c_active_topic(
    thread_state: &ThreadState,
    selected: &Ph1xContextCandidate,
) -> Option<String> {
    match selected.candidate_kind {
        Ph1xContextCandidateKind::ActivePlan => Some("active planning frame".to_string()),
        Ph1xContextCandidateKind::ActiveWritingArtifact => {
            Some("active writing artifact".to_string())
        }
        Ph1xContextCandidateKind::ActiveToolResult
        | Ph1xContextCandidateKind::OpenClarification
        | Ph1xContextCandidateKind::CorrectionTarget => Some("active tool frame".to_string()),
        Ph1xContextCandidateKind::FreshMemoryHandoff => {
            Some("fresh memory handoff candidate".to_string())
        }
        Ph1xContextCandidateKind::ProtectedFailClosed => Some("protected boundary".to_string()),
        Ph1xContextCandidateKind::NewTopicFallback => Some("current turn".to_string()),
        _ => thread_state.active_subject_ref.clone(),
    }
}

fn ph1x_stage8_5c_active_intent(selected: &Ph1xContextCandidate) -> String {
    match selected.directive {
        HumanConversationDirective::RouteToTool => "continue_tool_with_resolved_context",
        HumanConversationDirective::RouteToWrite => "continue_planning_or_public_answer",
        HumanConversationDirective::ModifyPreviousOutput => "modify_active_artifact",
        HumanConversationDirective::CorrectPreviousOutput => "correct_previous_answer",
        HumanConversationDirective::HandOffToMemory => "ask_fresh_memory_for_context",
        HumanConversationDirective::FailClosedProtected => "fail_closed_protected",
        HumanConversationDirective::AnswerNewQuestion => "answer_new_question",
        HumanConversationDirective::AskClarification => "ask_clarification",
        HumanConversationDirective::ContinueCurrentTopic => "continue_current_topic",
        HumanConversationDirective::WaitOrNoAction => "wait_or_no_action",
    }
    .to_string()
}

fn ph1x_stage8_5c_pending_slots(selected: &Ph1xContextCandidate) -> Vec<String> {
    if selected.candidate_kind == Ph1xContextCandidateKind::OpenClarification {
        vec!["tool_family".to_string()]
    } else {
        Vec::new()
    }
}

fn ph1x_stage8_5c_topic_stack(thread_state: &ThreadState) -> Vec<String> {
    let mut topics = Vec::new();
    if let Some(active_subject_ref) = &thread_state.active_subject_ref {
        topics.push(active_subject_ref.clone());
    }
    if let Some(interrupted_subject_ref) = &thread_state.interrupted_subject_ref {
        topics.push(interrupted_subject_ref.clone());
    }
    topics.extend(
        thread_state
            .pinned_context_refs
            .iter()
            .filter(|reference| reference.starts_with("ph1x:frame"))
            .take(6)
            .cloned(),
    );
    topics.sort();
    topics.dedup();
    topics
}

fn ph1x_stage8_5c_user_goal(
    thread_state: &ThreadState,
    features: &Stage8_5Features,
) -> Option<String> {
    if ph1x_stage8_5_has_planning_frame(thread_state) || ph1x_stage8_5_planning_seed(features) {
        Some("continue active planning goal".to_string())
    } else if ph1x_stage8_5_has_writing_frame(thread_state) {
        Some("revise active writing artifact".to_string())
    } else {
        None
    }
}

fn ph1x_stage8_5c_current_plan(thread_state: &ThreadState) -> Option<String> {
    let topics = ph1x_stage8_5_frame_values(thread_state, "ph1x:topic:");
    let constraints = ph1x_stage8_5_frame_values(thread_state, "ph1x:constraint:");
    if topics.is_empty() && constraints.is_empty() {
        None
    } else {
        Some(format!(
            "topics: {}; constraints: {}",
            ph1x_stage8_5_join_or_default(&topics, "none"),
            ph1x_stage8_5_join_or_default(&constraints, "none")
        ))
    }
}

fn ph1x_stage8_5c_open_question(selected: &Ph1xContextCandidate) -> Option<String> {
    match selected.candidate_kind {
        Ph1xContextCandidateKind::ActivePlan => Some("resolve planning recommendation".to_string()),
        Ph1xContextCandidateKind::OpenClarification => {
            Some("answer pending clarification".to_string())
        }
        _ => None,
    }
}

fn ph1x_stage8_5c_unresolved_decision(selected: &Ph1xContextCandidate) -> Option<String> {
    match selected.candidate_kind {
        Ph1xContextCandidateKind::ActivePlan => {
            Some("select best option from comparison set".to_string())
        }
        Ph1xContextCandidateKind::OpenClarification => {
            Some("select tool family for target".to_string())
        }
        _ => None,
    }
}

fn ph1x_stage8_5c_expected_answer_type(selected: &Ph1xContextCandidate) -> Option<String> {
    Some(
        match selected.directive {
            HumanConversationDirective::RouteToTool => "tool_answer",
            HumanConversationDirective::RouteToWrite => "recommendation",
            HumanConversationDirective::ModifyPreviousOutput => "rewrite",
            HumanConversationDirective::CorrectPreviousOutput => "correction",
            HumanConversationDirective::FailClosedProtected => "safe_refusal",
            HumanConversationDirective::HandOffToMemory => "fresh_memory_decision",
            HumanConversationDirective::AnswerNewQuestion => "new_answer",
            HumanConversationDirective::AskClarification => "clarification",
            HumanConversationDirective::ContinueCurrentTopic => "continuation",
            HumanConversationDirective::WaitOrNoAction => "no_action",
        }
        .to_string(),
    )
}

fn ph1x_stage8_5c_discourse_state(selected: &Ph1xContextCandidate) -> &'static str {
    match selected.directive {
        HumanConversationDirective::AnswerNewQuestion => "topic_switch_or_new_topic",
        HumanConversationDirective::AskClarification => "clarification_needed",
        HumanConversationDirective::FailClosedProtected => "protected_boundary",
        HumanConversationDirective::HandOffToMemory => "memory_handoff",
        _ => "continuation_selected",
    }
}

fn ph1x_stage8_5c_confidence_reason(features: &Stage8_5Features, score: u16) -> String {
    if score >= 8_500 {
        "selected candidate met high confidence active-frame threshold".to_string()
    } else if score >= 6_500 {
        "selected candidate met medium confidence threshold and may require clarification"
            .to_string()
    } else if features.new_topic {
        "current turn has new-topic evidence stronger than active context".to_string()
    } else {
        "candidate confidence is weak and active context should not be forced".to_string()
    }
}

fn ph1x_stage8_5c_has_hot_context(thread_state: &ThreadState) -> bool {
    thread_state.active_subject_ref.is_some()
        || thread_state.last_turn_context.is_some()
        || !thread_state.pinned_context_refs.is_empty()
}

fn ph1x_stage8_5c_has_reference_signal(features: &Stage8_5Features) -> bool {
    features.entity_fragment.is_some()
        || features.selection_followup
        || features.prior_options_followup
        || features.timing_followup
        || features.writing_modifier.is_some()
        || features.definite_reference_followup
        || ph1x_stage8_5_has_any(&features.tokens, STAGE8_5_REFERENCE_TERMS)
}

fn ph1x_stage8_5c_engine_label(engine: SuggestedNextEngine) -> &'static str {
    match engine {
        SuggestedNextEngine::None => "none",
        SuggestedNextEngine::Ph1M => "PH1.M",
        SuggestedNextEngine::Ph1E => "PH1.E",
        SuggestedNextEngine::Ph1Write => "PH1.WRITE",
        SuggestedNextEngine::ProtectedBoundary => "PROTECTED_BOUNDARY",
    }
}

fn ph1x_stage8_5c_decision_suffix(normalized: &str) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    normalized.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

pub fn ph1x_update_universal_active_context_after_turn(
    thread_state: ThreadState,
    user_text: Option<&str>,
    response_text: Option<&str>,
    route_label: Option<&str>,
) -> ThreadState {
    let user = user_text.unwrap_or_default().trim();
    let response = response_text.unwrap_or_default().trim();
    let route = route_label.unwrap_or_default();
    let features = ph1x_stage8_5_features(user);

    if let Some(choice) = ph1x_stage8_5_tool_choice_from_response(response) {
        return ph1x_stage8_5_set_subject_refs(
            thread_state,
            &format!(
                "{STAGE8_5_TOOL_CHOICE_PREFIX}{}",
                ph1x_stage8_5_slug(&choice.target)
            ),
            ph1x_stage8_5_tool_choice_refs(&choice),
        );
    }

    if response.contains("NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION") {
        return ph1x_stage8_5_set_subject_refs(
            thread_state,
            &format!("{STAGE8_5_PROTECTED_PREFIX}{}", ph1x_stage8_5_slug(user)),
            vec![
                STAGE8_5_PROTECTED_RISK_REF.to_string(),
                STAGE8_5_FAIL_CLOSED_REF.to_string(),
                "ph1x:why_not_continue:no_simulation_no_authority".to_string(),
                "ph1x:discourse:protected_boundary".to_string(),
            ],
        );
    }

    if features.writing_seed {
        return ph1x_stage8_5_set_subject_refs(
            thread_state,
            STAGE8_5_WRITING_SUBJECT,
            ph1x_stage8_5_writing_refs(&features),
        );
    }

    if ph1x_stage8_5_planning_seed(&features) {
        return ph1x_stage8_5_set_subject_refs(
            thread_state,
            STAGE8_5_PLANNING_SUBJECT,
            ph1x_stage8_5_planning_refs(&features, response),
        );
    }

    if ph1x_stage8_5_direct_tool_turn(&features) {
        return ph1x_stage8_5_suspend_stage8_5_context(thread_state);
    }

    if ph1x_stage8_5_has_planning_frame(&thread_state)
        && (!features.constraints.is_empty()
            || features.reference_followup
            || features.definite_reference_followup
            || features.selection_followup
            || features.timing_followup
            || features.prior_options_followup)
    {
        return ph1x_stage8_5_set_subject_refs(
            thread_state,
            STAGE8_5_PLANNING_SUBJECT,
            ph1x_stage8_5_planning_refs(&features, response),
        );
    }

    if features.new_topic {
        return ph1x_stage8_5_suspend_stage8_5_context(thread_state);
    }

    if route == "H381_H380_LIVE_RESPONSE" {
        return thread_state;
    }

    thread_state
}

fn ph1x_stage8_5_set_subject_refs(
    mut thread_state: ThreadState,
    subject: &str,
    refs: Vec<String>,
) -> ThreadState {
    let original = thread_state.clone();
    thread_state.active_subject_ref = Some(ph1x_stage8_5_clean_context_fragment(subject, 256));
    for context_ref in refs {
        if thread_state.pinned_context_refs.len() >= 16 {
            break;
        }
        let context_ref = ph1x_stage8_5_clean_ref(&context_ref);
        if context_ref.is_empty() {
            continue;
        }
        if !thread_state
            .pinned_context_refs
            .iter()
            .any(|existing| existing == &context_ref)
        {
            thread_state.pinned_context_refs.push(context_ref);
        }
    }
    if thread_state.validate().is_ok() {
        thread_state
    } else {
        original
    }
}

fn ph1x_stage8_5_suspend_stage8_5_context(mut thread_state: ThreadState) -> ThreadState {
    if thread_state
        .active_subject_ref
        .as_deref()
        .is_some_and(|subject| subject.starts_with("ph1x:"))
        && thread_state.interrupted_subject_ref.is_none()
    {
        thread_state.interrupted_subject_ref = thread_state.active_subject_ref.clone();
    }
    if thread_state
        .active_subject_ref
        .as_deref()
        .is_some_and(|subject| subject.starts_with("ph1x:"))
    {
        thread_state.active_subject_ref = None;
    }
    thread_state
}

fn ph1x_stage8_5_normalized(text: &str) -> String {
    text.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                ' '
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn ph1x_stage8_5_features(text: &str) -> Stage8_5Features {
    let normalized = ph1x_stage8_5_normalized(text);
    let tokens = normalized
        .split_whitespace()
        .map(|token| token.to_string())
        .collect::<Vec<_>>();
    let explicit_tool_family = tokens
        .iter()
        .find_map(|token| Stage8_5ToolFamily::from_token(token));
    let selected_tool_family = ph1x_stage8_5_selected_tool_family(&tokens);
    let destination = ph1x_stage8_5_extract_destination(&tokens);
    let constraints = ph1x_stage8_5_extract_constraints(&tokens);
    let reference_followup = ph1x_stage8_5_reference_followup(&tokens);
    let definite_reference_followup = ph1x_stage8_5_definite_reference_followup(&tokens);
    let selection_followup = ph1x_stage8_5_selection_followup(&tokens);
    let timing_followup = ph1x_stage8_5_timing_followup(&tokens);
    let prior_options_followup = ph1x_stage8_5_prior_options_followup(&tokens);
    let writing_seed = ph1x_stage8_5_writing_seed(&tokens);
    let writing_modifier = ph1x_stage8_5_writing_modifier(&tokens);
    let correction_target = ph1x_stage8_5_correction_target(&tokens);
    let confirmation_followup = ph1x_stage8_5_confirmation_followup(&tokens);
    let entity_fragment = ph1x_stage8_5_entity_fragment(text, &tokens);
    let new_topic = ph1x_stage8_5_new_topic(
        &tokens,
        explicit_tool_family,
        writing_seed,
        correction_target,
        definite_reference_followup,
        selection_followup,
        timing_followup,
        prior_options_followup,
    );
    Stage8_5Features {
        normalized,
        tokens,
        explicit_tool_family,
        selected_tool_family,
        entity_fragment,
        destination,
        constraints,
        reference_followup,
        definite_reference_followup,
        selection_followup,
        timing_followup,
        prior_options_followup,
        writing_seed,
        writing_modifier,
        correction_target,
        confirmation_followup,
        new_topic,
    }
}

fn ph1x_stage8_5_has_any(tokens: &[String], values: &[&str]) -> bool {
    tokens
        .iter()
        .any(|token| values.iter().any(|value| token == value))
}

fn ph1x_stage8_5_planning_seed(features: &Stage8_5Features) -> bool {
    features.destination.is_some()
        && (ph1x_stage8_5_has_any(&features.tokens, STAGE8_5_PLANNING_MARKERS)
            || !features.constraints.is_empty())
}

fn ph1x_stage8_5_selection_followup(tokens: &[String]) -> bool {
    let has_selector = ph1x_stage8_5_has_any(tokens, &["which", "what", "where"]);
    has_selector
        && (ph1x_stage8_5_has_any(tokens, STAGE8_5_SELECTION_TARGETS)
            || ph1x_stage8_5_has_any(tokens, &["suggest", "recommend", "choose", "pick"]))
}

fn ph1x_stage8_5_reference_followup(tokens: &[String]) -> bool {
    ph1x_stage8_5_has_any(tokens, STAGE8_5_REFERENCE_TERMS)
        && ph1x_stage8_5_has_any(
            tokens,
            &[
                "what", "which", "where", "who", "how", "why", "do", "does", "did", "should",
                "would", "could", "can", "give", "provide", "tell", "show",
            ],
        )
}

fn ph1x_stage8_5_definite_reference_followup(tokens: &[String]) -> bool {
    ph1x_stage8_5_has_any(tokens, STAGE8_5_DEFINITE_REFERENCE_MARKERS)
        && ph1x_stage8_5_has_any(tokens, STAGE8_5_DEFINITE_REFERENCE_TARGETS)
        && ph1x_stage8_5_has_any(
            tokens,
            &[
                "what", "which", "where", "who", "how", "why", "do", "does", "did", "should",
                "would", "could", "can", "give", "provide", "tell", "show", "budget", "cost",
                "price", "prices", "rate", "rates",
            ],
        )
}

fn ph1x_stage8_5_timing_followup(tokens: &[String]) -> bool {
    ph1x_stage8_5_has_any(tokens, STAGE8_5_TIMING_TARGETS)
        && ph1x_stage8_5_has_any(tokens, &["which", "what", "best", "go", "visit", "travel"])
}

fn ph1x_stage8_5_prior_options_followup(tokens: &[String]) -> bool {
    ph1x_stage8_5_has_any(
        tokens,
        &["suggest", "suggested", "recommend", "recommended"],
    ) && ph1x_stage8_5_has_any(tokens, &["where", "what", "which", "did", "were"])
}

fn ph1x_stage8_5_writing_seed(tokens: &[String]) -> bool {
    let has_artifact = ph1x_stage8_5_has_any(tokens, STAGE8_5_ARTIFACT_MARKERS);
    let has_modifier = ph1x_stage8_5_has_any(tokens, STAGE8_5_ARTIFACT_MODIFIERS);
    let has_reference = ph1x_stage8_5_has_any(tokens, STAGE8_5_REFERENCE_TERMS);
    let has_generation_posture = ph1x_stage8_5_generation_posture(tokens);
    has_artifact && !(has_reference && has_modifier) && (has_generation_posture || !has_modifier)
}

fn ph1x_stage8_5_generation_posture(tokens: &[String]) -> bool {
    tokens.iter().enumerate().any(|(idx, token)| {
        if matches!(
            token.as_str(),
            "write" | "compose" | "create" | "tell" | "give"
        ) {
            return true;
        }
        if token != "draft" {
            return false;
        }
        let previous = idx.checked_sub(1).and_then(|prev| tokens.get(prev));
        !previous.is_some_and(|previous| {
            STAGE8_5_DEFINITE_REFERENCE_MARKERS.contains(&previous.as_str())
                || STAGE8_5_ARTIFACT_MODIFIERS.contains(&previous.as_str())
        })
    })
}

fn ph1x_stage8_5_direct_tool_turn(features: &Stage8_5Features) -> bool {
    features.explicit_tool_family.is_some()
        && features.correction_target.is_none()
        && !features.selection_followup
        && !features.timing_followup
        && !features.prior_options_followup
        && features.writing_modifier.is_none()
}

fn ph1x_stage8_5_writing_modifier(tokens: &[String]) -> Option<String> {
    let modifier = tokens
        .iter()
        .find(|token| STAGE8_5_ARTIFACT_MODIFIERS.contains(&token.as_str()))
        .cloned();
    if modifier.is_some()
        && (ph1x_stage8_5_has_any(tokens, STAGE8_5_REFERENCE_TERMS)
            || !ph1x_stage8_5_has_any(tokens, STAGE8_5_ARTIFACT_MARKERS)
            || tokens.len() <= 5)
    {
        modifier
    } else {
        None
    }
}

fn ph1x_stage8_5_confirmation_followup(tokens: &[String]) -> bool {
    tokens.len() <= 5 && ph1x_stage8_5_has_any(tokens, STAGE8_5_CONFIRMATION_TERMS)
}

fn ph1x_stage8_5_correction_target(tokens: &[String]) -> Option<Stage8_5ToolFamily> {
    let correction_marker = ph1x_stage8_5_has_any(tokens, &["not", "instead", "meant", "mean"]);
    if !correction_marker {
        return None;
    }
    tokens.iter().enumerate().rev().find_map(|(idx, token)| {
        let tool_family = Stage8_5ToolFamily::from_token(token)?;
        let prev = idx.checked_sub(1).and_then(|i| tokens.get(i));
        let prev2 = idx.checked_sub(2).and_then(|i| tokens.get(i));
        let negated = prev.is_some_and(|candidate| candidate == "not")
            || (prev.is_some_and(|candidate| candidate == "the")
                && prev2.is_some_and(|candidate| candidate == "not"));
        if negated {
            None
        } else {
            Some(tool_family)
        }
    })
}

fn ph1x_stage8_5_selected_tool_family(tokens: &[String]) -> Option<Stage8_5ToolFamily> {
    let compact = tokens
        .iter()
        .filter(|token| !STAGE8_5_REFERENCE_TERMS.contains(&token.as_str()))
        .collect::<Vec<_>>();
    if compact.len() <= 4 {
        compact
            .iter()
            .find_map(|token| Stage8_5ToolFamily::from_token(token))
    } else {
        None
    }
}

fn ph1x_stage8_5_new_topic(
    tokens: &[String],
    explicit_tool_family: Option<Stage8_5ToolFamily>,
    writing_seed: bool,
    correction_target: Option<Stage8_5ToolFamily>,
    definite_reference_followup: bool,
    selection_followup: bool,
    timing_followup: bool,
    prior_options_followup: bool,
) -> bool {
    if correction_target.is_some()
        || definite_reference_followup
        || selection_followup
        || timing_followup
        || prior_options_followup
    {
        return false;
    }
    if explicit_tool_family.is_some() || writing_seed {
        return true;
    }
    let has_question_lead = ph1x_stage8_5_has_any(tokens, &["what", "who", "how", "why", "tell"]);
    let has_reference = ph1x_stage8_5_has_any(tokens, STAGE8_5_REFERENCE_TERMS);
    let identity_like = ph1x_stage8_5_has_any(tokens, &["name", "identity", "called"])
        && ph1x_stage8_5_has_any(tokens, &["what", "who", "your"]);
    let origin_like = ph1x_stage8_5_has_any(tokens, &["from", "origin"])
        && ph1x_stage8_5_has_any(tokens, &["you", "your"]);
    let social_request = ph1x_stage8_5_has_any(tokens, &["joke"]);
    let broad_explain = (ph1x_stage8_5_has_any(tokens, &["explain", "define"])
        || (ph1x_stage8_5_has_any(tokens, &["tell"]) && ph1x_stage8_5_has_any(tokens, &["about"])))
        && !ph1x_stage8_5_has_any(tokens, &["our", "previous", "prior"]);
    identity_like
        || origin_like
        || social_request
        || (has_question_lead && !has_reference && broad_explain)
}

fn ph1x_stage8_5_extract_constraints(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for token in tokens {
        if STAGE8_5_ACTIVITY_MARKERS.contains(&token.as_str())
            && !out.iter().any(|existing| existing == token)
        {
            out.push(token.clone());
        }
    }
    out
}

fn ph1x_stage8_5_extract_destination(tokens: &[String]) -> Option<String> {
    for (idx, token) in tokens.iter().enumerate() {
        if matches!(
            token.as_str(),
            "to" | "in" | "for" | "around" | "through" | "across"
        ) {
            let mut collected = Vec::new();
            for next in tokens.iter().skip(idx + 1).take(4) {
                if STAGE8_5_STOP_TOKENS.contains(&next.as_str())
                    || STAGE8_5_ACTIVITY_MARKERS.contains(&next.as_str())
                {
                    break;
                }
                collected.push(next.clone());
            }
            if !collected.is_empty() {
                return Some(collected.join(" "));
            }
        }
    }
    tokens
        .iter()
        .find(|token| {
            !STAGE8_5_STOP_TOKENS.contains(&token.as_str())
                && !STAGE8_5_PLANNING_MARKERS.contains(&token.as_str())
                && !STAGE8_5_ACTIVITY_MARKERS.contains(&token.as_str())
                && token.len() > 3
        })
        .cloned()
}

fn ph1x_stage8_5_entity_fragment(text: &str, tokens: &[String]) -> Option<String> {
    if tokens.is_empty() || tokens.len() > 5 {
        return None;
    }
    if ph1x_stage8_5_has_any(tokens, STAGE8_5_TOOL_TIME_TERMS)
        || ph1x_stage8_5_has_any(tokens, STAGE8_5_TOOL_WEATHER_TERMS)
        || ph1x_stage8_5_has_any(tokens, STAGE8_5_CONFIRMATION_TERMS)
    {
        return None;
    }
    if ph1x_stage8_5_has_any(
        tokens,
        &[
            "you", "your", "yours", "me", "my", "mine", "meaning", "proof", "evidence", "source",
            "sources",
        ],
    ) {
        return None;
    }
    if ph1x_stage8_5_has_any(tokens, &["like"])
        && ph1x_stage8_5_has_any(tokens, &["in", "at", "for"])
    {
        return None;
    }
    let trimmed = text.trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace());
    let lower = trimmed.to_ascii_lowercase();
    for marker in [
        "what about ",
        "same for ",
        "same question ",
        "and ",
        "also ",
    ] {
        if lower.starts_with(marker) {
            let tail = trimmed.get(marker.len()..).unwrap_or_default();
            let candidate = ph1x_stage8_5_clean_entity_fragment(tail)?;
            return Some(candidate);
        }
    }
    if ph1x_stage8_5_has_any(
        tokens,
        &[
            "what", "where", "who", "why", "how", "with", "for", "from", "there", "here",
        ],
    ) {
        return None;
    }
    ph1x_stage8_5_clean_entity_fragment(trimmed)
}

fn ph1x_stage8_5_clean_entity_fragment(fragment: &str) -> Option<String> {
    let words = fragment
        .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace())
        .split_whitespace()
        .take(4)
        .collect::<Vec<_>>();
    if words.is_empty() {
        return None;
    }
    let normalized_words = words
        .iter()
        .map(|word| ph1x_stage8_5_normalized(word))
        .collect::<Vec<_>>();
    if normalized_words.iter().any(|word| {
        word.is_empty()
            || STAGE8_5_STOP_TOKENS.contains(&word.as_str())
            || STAGE8_5_REFERENCE_TERMS.contains(&word.as_str())
    }) {
        return None;
    }
    Some(words.join(" "))
}

fn ph1x_stage8_5_tool_family_from_route(route: LastTurnRouteClass) -> Option<Stage8_5ToolFamily> {
    match route {
        LastTurnRouteClass::ToolTime => Some(Stage8_5ToolFamily::Time),
        LastTurnRouteClass::ToolWeather => Some(Stage8_5ToolFamily::Weather),
        _ => None,
    }
}

fn ph1x_stage8_5_tool_prompt(tool_family: Stage8_5ToolFamily, target: &str) -> String {
    format!("what is the {} in {}", tool_family.prompt_noun(), target)
}

fn ph1x_stage8_5_rewrite_previous_artifact_prompt(
    thread_state: &ThreadState,
    user_instruction: &str,
    modifier: &str,
) -> String {
    let previous = thread_state
        .last_turn_context
        .as_ref()
        .filter(|context| context.route_class != LastTurnRouteClass::Clarify)
        .map(|context| ph1x_stage8_5_truncate_chars(context.answer_text.trim(), 1200))
        .filter(|text| !text.trim().is_empty());
    let instruction = format!(
        "Rewrite the previous text to satisfy the user's latest request. Return only the revised user-facing text. Do not mention prompts, drafts, active artifacts, context packets, or internal machinery. Preserve the existing subject, recipient, commitments, and purpose unless the user explicitly changes them. User request: {user_instruction}. Modification signal: {modifier}."
    );
    match previous {
        Some(previous) => format!("{instruction}\n\nPrevious text:\n{previous}"),
        None => instruction.to_string(),
    }
}

fn ph1x_stage8_5_latest_answer_prompt(latest_answer: &str, user_text: &str) -> String {
    let latest_answer = ph1x_stage8_5_truncate_chars(latest_answer.trim(), 900);
    format!(
        "Answer the current user follow-up by resolving its references against the latest Selene answer. Use the latest answer as context, preserve any named people, entities, options, and facts from it, and answer naturally without exposing context machinery.\n\nLatest Selene answer:\n{latest_answer}\n\nUser follow-up:\n{user_text}"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Stage8_5ToolChoice {
    target: String,
    alternatives: Vec<Stage8_5ToolFamily>,
}

fn ph1x_stage8_5_tool_choice_target(thread_state: &ThreadState) -> Option<String> {
    let subject = thread_state.active_subject_ref.as_deref()?;
    let target = subject.strip_prefix(STAGE8_5_TOOL_CHOICE_PREFIX)?;
    let clean = target.trim();
    if clean.is_empty() {
        None
    } else {
        Some(clean.replace('_', " "))
    }
}

fn ph1x_stage8_5_tool_choice_from_response(response: &str) -> Option<Stage8_5ToolChoice> {
    let trimmed = response.trim();
    if !trimmed.ends_with('?') {
        return None;
    }
    let lower = ph1x_stage8_5_normalized(trimmed);
    if !lower.contains(" or ") || !lower.contains(" for ") {
        return None;
    }
    let alternatives = lower
        .split(" for ")
        .next()
        .unwrap_or_default()
        .split(" or ")
        .filter_map(|part| {
            part.split_whitespace()
                .rev()
                .find_map(Stage8_5ToolFamily::from_token)
        })
        .collect::<Vec<_>>();
    if alternatives.len() < 2 {
        return None;
    }
    let target = trimmed
        .rsplit_once(" for ")
        .map(|(_, target)| target)
        .unwrap_or_default()
        .trim()
        .trim_end_matches('?')
        .trim();
    if target.is_empty() {
        None
    } else {
        Some(Stage8_5ToolChoice {
            target: target.to_string(),
            alternatives,
        })
    }
}

fn ph1x_stage8_5_tool_choice_refs(choice: &Stage8_5ToolChoice) -> Vec<String> {
    let mut refs = vec![
        STAGE8_5_EXPECT_TOOL_CHOICE_REF.to_string(),
        "ph1x:discourse:clarification".to_string(),
        ph1x_stage8_5_make_ref("ph1x:clarification_target", &choice.target),
    ];
    for alternative in &choice.alternatives {
        refs.push(format!(
            "ph1x:clarification_option:{}",
            alternative.ref_id()
        ));
    }
    refs
}

fn ph1x_stage8_5_last_tool_entity(thread_state: &ThreadState) -> Option<String> {
    thread_state
        .last_turn_context
        .as_ref()
        .and_then(|context| ph1x_stage8_5_entity_after_marker(&context.answer_text))
}

fn ph1x_stage8_5_entity_after_marker(text: &str) -> Option<String> {
    let trimmed = text.trim();
    for marker in [" in ", " for ", " at "].iter() {
        if let Some((_, tail)) = trimmed.rsplit_once(marker) {
            let candidate = tail
                .trim()
                .trim_matches(|ch: char| ch.is_ascii_punctuation() || ch.is_whitespace());
            let words = candidate
                .split_whitespace()
                .take(4)
                .collect::<Vec<_>>()
                .join(" ");
            if !words.is_empty() {
                return Some(words);
            }
        }
    }
    ph1x_stage8_5_extract_prior_options(trimmed)
        .into_iter()
        .next()
}

fn ph1x_stage8_5_planning_refs(features: &Stage8_5Features, response: &str) -> Vec<String> {
    let mut refs = vec![
        STAGE8_5_FRAME_PLANNING_REF.to_string(),
        STAGE8_5_EXPECT_RECOMMENDATION_REF.to_string(),
        STAGE8_5_OPEN_SELECTION_REF.to_string(),
        STAGE8_5_COMPARISON_SET_REF.to_string(),
    ];
    if let Some(destination) = &features.destination {
        refs.push(ph1x_stage8_5_make_ref("ph1x:topic", destination));
    }
    for constraint in &features.constraints {
        refs.push(ph1x_stage8_5_make_ref("ph1x:constraint", constraint));
    }
    for option in ph1x_stage8_5_extract_prior_options(response) {
        refs.push(ph1x_stage8_5_make_ref("ph1x:option", &option));
    }
    refs
}

fn ph1x_stage8_5_writing_refs(features: &Stage8_5Features) -> Vec<String> {
    let mut refs = vec![
        STAGE8_5_FRAME_WRITING_REF.to_string(),
        STAGE8_5_EXPECT_REWRITE_REF.to_string(),
        STAGE8_5_REFERENCE_PREVIOUS_REF.to_string(),
    ];
    for token in &features.tokens {
        if STAGE8_5_ARTIFACT_MARKERS.contains(&token.as_str()) {
            refs.push(ph1x_stage8_5_make_ref("ph1x:artifact", token));
        }
    }
    if let Some(destination) = &features.destination {
        refs.push(ph1x_stage8_5_make_ref("ph1x:entity", destination));
    }
    refs
}

fn ph1x_stage8_5_has_planning_frame(thread_state: &ThreadState) -> bool {
    thread_state.active_subject_ref.as_deref() == Some(STAGE8_5_PLANNING_SUBJECT)
        || thread_state.interrupted_subject_ref.as_deref() == Some(STAGE8_5_PLANNING_SUBJECT)
        || thread_state
            .pinned_context_refs
            .iter()
            .any(|context_ref| context_ref == STAGE8_5_FRAME_PLANNING_REF)
}

fn ph1x_stage8_5_has_writing_frame(thread_state: &ThreadState) -> bool {
    thread_state.active_subject_ref.as_deref() == Some(STAGE8_5_WRITING_SUBJECT)
        || thread_state
            .pinned_context_refs
            .iter()
            .any(|context_ref| context_ref == STAGE8_5_FRAME_WRITING_REF)
}

fn ph1x_stage8_5_frame_values(thread_state: &ThreadState, prefix: &str) -> Vec<String> {
    thread_state
        .pinned_context_refs
        .iter()
        .filter_map(|context_ref| context_ref.strip_prefix(prefix))
        .map(|value| value.replace('_', " "))
        .collect()
}

fn ph1x_stage8_5_planning_prompt(
    thread_state: &ThreadState,
    user_text: &str,
    timing_followup: bool,
    prior_options_followup: bool,
    reference_followup: bool,
) -> String {
    let topics = ph1x_stage8_5_frame_values(thread_state, "ph1x:topic:");
    let constraints = ph1x_stage8_5_frame_values(thread_state, "ph1x:constraint:");
    let options = ph1x_stage8_5_frame_values(thread_state, "ph1x:option:");
    let previous_answer = thread_state
        .last_turn_context
        .as_ref()
        .map(|context| ph1x_stage8_5_truncate_chars(context.answer_text.trim(), 900))
        .filter(|answer| !answer.trim().is_empty());
    let topic_text = ph1x_stage8_5_join_or_default(&topics, "the active trip or planning topic");
    let constraint_text =
        ph1x_stage8_5_join_or_default(&constraints, "the user's stated constraints");
    let option_text =
        ph1x_stage8_5_join_or_default(&options, "the prior options already presented");
    let focus = if timing_followup {
        "The user is asking for timing guidance inside the current planning frame."
    } else if prior_options_followup {
        "The user is asking about prior recommendations inside the current planning frame."
    } else if reference_followup {
        "The user is referring to the active planning frame or the latest answer inside it."
    } else {
        "The user is asking for a recommendation inside the current planning frame."
    };
    let mut prompt = format!(
        "Continue the active planning frame. Topic: {topic_text}. Constraints and interests: {constraint_text}. Prior options: {option_text}. {focus} Answer this follow-up inside that frame and do not switch to unrelated destinations unless the user asks to change topic: {user_text}"
    );
    if let Some(previous_answer) = previous_answer {
        prompt.push_str("\n\nLatest Selene answer in this frame:\n");
        prompt.push_str(&previous_answer);
    }
    prompt
}

fn ph1x_stage8_5_join_or_default(values: &[String], default: &str) -> String {
    if values.is_empty() {
        default.to_string()
    } else {
        values.join(", ")
    }
}

fn ph1x_stage8_5_extract_prior_options(response: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = Vec::new();
    for raw in response.split_whitespace() {
        let clean = raw.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
        let starts_upper = clean
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase());
        if starts_upper && clean.len() > 2 && !ph1x_stage8_5_common_sentence_start(clean) {
            current.push(clean.to_string());
            continue;
        }
        if !current.is_empty() {
            out.push(current.join(" "));
            current.clear();
        }
    }
    if !current.is_empty() {
        out.push(current.join(" "));
    }
    out.sort();
    out.dedup();
    out.into_iter().take(6).collect()
}

fn ph1x_stage8_5_common_sentence_start(value: &str) -> bool {
    matches!(
        value,
        "I" | "It" | "The" | "A" | "An" | "For" | "Both" | "Another" | "Consider" | "Sure"
    )
}

fn ph1x_stage8_5_protected_request_from_subject(thread_state: &ThreadState) -> Option<String> {
    let subject = thread_state.active_subject_ref.as_deref()?;
    let encoded = subject.strip_prefix(STAGE8_5_PROTECTED_PREFIX)?;
    let decoded = encoded.replace('_', " ");
    (!decoded.trim().is_empty()).then_some(decoded)
}

fn ph1x_stage8_5_make_ref(prefix: &str, value: &str) -> String {
    format!("{prefix}:{}", ph1x_stage8_5_slug(value))
}

fn ph1x_stage8_5_clean_ref(value: &str) -> String {
    ph1x_stage8_5_clean_context_fragment(&value.replace(' ', "_"), 128)
}

fn ph1x_stage8_5_clean_context_fragment(value: &str, max_chars: usize) -> String {
    ph1x_stage8_5_truncate_chars(
        &value
            .trim()
            .chars()
            .filter(|ch| !ch.is_control() && !ch.is_ascii_whitespace())
            .collect::<String>(),
        max_chars,
    )
}

fn ph1x_stage8_5_slug(value: &str) -> String {
    ph1x_stage8_5_normalized(value)
        .split_whitespace()
        .take(8)
        .collect::<Vec<_>>()
        .join("_")
}

fn ph1x_stage8_5_truncate_chars(text: &str, max_chars: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(max_chars) {
        out.push(ch);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionOk,
        SpeakerAssertionUnknown, SpeakerId, SpeakerLabel, DEFAULT_CONF_MID_BP,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{
        AcceptedSourcePacket, CacheStatus, ClaimEvidenceLink, ClaimRequestPacket,
        ClaimVerificationPacket, PresentationPacket, RejectedSourcePacket, RequestedEntityPacket,
        SearchImagePacket, SourceCardPacket, SourceChipPacket, SourceEvaluationPacket,
        SourceMetadata, SourceRef, ToolQueryHash, ToolRequestId, ToolStructuredField,
        ToolTextSnippet, WebAnswerVerificationPacket,
    };
    use selene_kernel_contracts::ph1k::{
        Confidence, DegradationClassBundle, InterruptCandidate, InterruptCandidateConfidenceBand,
        InterruptDegradationContext, InterruptGateConfidences, InterruptGates, InterruptLocaleTag,
        InterruptPhraseId, InterruptPhraseSetVersion, InterruptRiskContextClass,
        InterruptSpeechWindowMetrics, InterruptSubjectRelationConfidenceBundle,
        InterruptTimingMarkers, SpeechLikeness, PH1K_INTERRUPT_LOCALE_TAG_DEFAULT,
    };
    use selene_kernel_contracts::ph1lang::{LanguagePacket, LanguageSwitchScope};
    use selene_kernel_contracts::ph1m::{
        MemoryCandidate, MemoryConfidence, MemoryKey, MemoryProvenance, MemorySensitivityFlag,
        MemoryUsePolicy, MemoryValue,
    };
    use selene_kernel_contracts::ph1n::{
        AmbiguityFlag, Chat, Clarify, EvidenceSpan, FieldValue, IntentField, OverallConfidence,
        Ph1nResponse, SensitivityLevel, TranscriptHash,
    };
    use selene_kernel_contracts::ph1tts::AnswerId;
    use selene_kernel_contracts::ph1x::{
        ConfirmAnswer, DispatchRequest, IdentityContext, IdentityPromptState,
        InterruptContinuityOutcome, InterruptResumePolicy, InterruptSubjectRelation,
        LastTurnContext, LastTurnRouteClass, ResumeBuffer, ThreadState, TtsResumeSnapshot,
    };
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId, SchemaVersion};

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    fn policy_privacy() -> PolicyContextRef {
        PolicyContextRef::v1(true, false, SafetyTier::Standard)
    }

    fn base_thread() -> ThreadState {
        ThreadState::empty_v1()
    }

    fn last_turn_context(route_class: LastTurnRouteClass, answer_text: &str) -> LastTurnContext {
        let tool_used = matches!(
            route_class,
            LastTurnRouteClass::ToolTime
                | LastTurnRouteClass::ToolWeather
                | LastTurnRouteClass::ToolOther
        );
        LastTurnContext::v1(
            route_class,
            tool_used,
            tool_used,
            None,
            answer_text.to_string(),
            "0".repeat(64),
        )
        .unwrap()
    }

    fn slice3a_fake_one_line_proposal() -> Slice3aOneLineProviderProposal {
        Slice3aOneLineProviderProposal {
            provider_id: "TEST_FAKE_PROVIDER".to_string(),
            provider_enabled: true,
            operation: Slice3aProviderProposalOperation::OneLineRewrite,
            target: Slice3aProviderProposalTarget::PreviousAssistantAnswer,
            likely_owner: SuggestedNextEngine::Ph1Write,
            protected_risk: ProtectedRisk::None,
            provider_call_attempt_count: 1,
            provider_network_dispatch_count: 0,
            raw_provider_output_exposed: false,
            protected_execution_authorized: false,
            simulation_authorized: false,
            authority_authorized: false,
        }
    }

    fn base_thread_with_continuity(subject_ref: &str, active_speaker_user_id: &str) -> ThreadState {
        ThreadState::empty_v1()
            .with_continuity(
                Some(subject_ref.to_string()),
                Some(active_speaker_user_id.to_string()),
            )
            .unwrap()
    }

    fn now(n: u64) -> MonotonicTimeNs {
        MonotonicTimeNs(n)
    }

    #[test]
    fn stage8_5_japan_planning_city_followup_uses_active_context() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I'm interested in Japan and doing some skiing and visiting some great Japanese restaurants."),
	            Some("Japan has strong ski and restaurant options."),
	            Some("PUBLIC_CHAT"),
	        );

        let rewritten = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Which city do you suggest?"),
        )
        .expect("Japan planning follow-up should carry context");
        let areas = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Which areas do you suggest?"),
        )
        .expect("area recommendation should use the same active planning frame");

        assert!(rewritten.contains("active planning frame"));
        assert!(areas.contains("active planning frame"));
        assert!(rewritten.contains("japan"));
        assert!(areas.contains("japan"));
        assert!(rewritten.contains("skiing"));
        assert!(rewritten.contains("restaurants"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == "ph1x:topic:japan"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_COMPARISON_SET_REF));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_EXPECT_RECOMMENDATION_REF));
    }

    #[test]
    fn stage8_5_time_followup_preserves_tool_family_for_new_entity() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 9:40 AM in New York.",
        ));

        let rewritten =
            ph1x_universal_active_context_followup_query(&thread, Some("What about Sydney?"))
                .expect("new entity follow-up should preserve the time tool family");

        assert_eq!(rewritten, "what is the time in Sydney");
    }

    #[test]
    fn stage8_5_time_followup_handles_unseen_place_fragment() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 8:15 PM in Paris.",
        ));

        let rewritten = ph1x_universal_active_context_followup_query(&thread, Some("And Berlin"))
            .expect("short entity fragment should preserve the prior tool family");

        assert_eq!(rewritten, "what is the time in Berlin");
    }

    #[test]
    fn stage8_5_planning_region_followup_uses_unseen_destination() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I want to plan a trip to Canada with snow and dining."),
            Some("Canada has several mountain regions with strong dining scenes."),
            Some("PUBLIC_CHAT"),
        );

        let rewritten = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Which regions would you recommend?"),
        )
        .expect("planning area follow-up should carry the active frame");

        assert!(rewritten.contains("active planning frame"));
        assert!(rewritten.contains("canada"));
        assert!(rewritten.contains("snow"));
        assert!(rewritten.contains("dining"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_OPEN_SELECTION_REF));
    }

    #[test]
    fn stage8_5_planning_place_and_base_questions_share_same_frame() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I'm interested in Japan and doing some skiing and visiting some great Japanese restaurants."),
            Some("Japan has strong ski and restaurant options."),
            Some("PUBLIC_CHAT"),
        );

        let place = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Which place would you pick?"),
        )
        .expect("place recommendation should stay in the planning frame");
        let base = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Where would you base the trip?"),
        )
        .expect("base recommendation should stay in the planning frame");

        assert!(place.contains("active planning frame"));
        assert!(place.contains("japan"));
        assert!(base.contains("active planning frame"));
        assert!(base.contains("japan"));
    }

    #[test]
    fn stage8_5_japan_season_followup_uses_planning_constraints() {
        let seeded = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I'll be planning a trip to Japan."),
            Some("That sounds exciting. What activities do you want to include?"),
            Some("PUBLIC_CHAT"),
        );
        let thread = ph1x_update_universal_active_context_after_turn(
            seeded,
            Some("I want to do some skiing and enjoy Japanese food."),
            Some("Consider visiting Niseko in Hokkaido or Hakuba for skiing and Japanese food."),
            Some("PUBLIC_CHAT"),
        );

        let rewritten = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Which time of the year do you suggest?"),
        )
        .expect("Japan season follow-up should carry planning context");

        assert!(rewritten.contains("active planning frame"));
        assert!(rewritten.contains("japan"));
        assert!(rewritten.contains("skiing"));
        assert!(rewritten.contains("timing guidance"));
    }

    #[test]
    fn stage8_5_japan_prior_options_survive_time_interruption() {
        let seeded = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I'll be planning a trip to Japan."),
            Some("That sounds exciting. What activities do you want to include?"),
            Some("PUBLIC_CHAT"),
        );
        let japan_thread = ph1x_update_universal_active_context_after_turn(
            seeded,
            Some("I want to do some skiing and enjoy Japanese food."),
            Some("Consider visiting Niseko in Hokkaido or Hakuba for skiing and Japanese food."),
            Some("PUBLIC_CHAT"),
        );
        let interrupted_thread = ph1x_update_universal_active_context_after_turn(
            japan_thread,
            Some("What time is it in Brisbane?"),
            Some("It's 12:48 AM in Brisbane."),
            Some("TOOL_TIME"),
        );

        let rewritten = ph1x_universal_active_context_followup_query(
            &interrupted_thread,
            Some("And where did you suggest we should go?"),
        )
        .expect("returnable Japan topic should preserve prior suggested options");

        assert!(rewritten.contains("niseko"));
        assert!(rewritten.contains("hakuba"));
        assert!(interrupted_thread
            .interrupted_subject_ref
            .as_deref()
            .is_some_and(|subject| subject == STAGE8_5_PLANNING_SUBJECT));
    }

    #[test]
    fn stage8_5_tool_choice_the_time_fills_pending_location() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("And Melbourne"),
            Some("Should I check the weather or the time for Melbourne?"),
            Some("H381_H380_LIVE_RESPONSE"),
        );

        let rewritten = ph1x_universal_active_context_followup_query(&thread, Some("The time"))
            .expect("tool-choice follow-up should resolve the pending location");

        assert_eq!(rewritten, "what is the time in melbourne");
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == "ph1x:clarification_option:weather"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == "ph1x:clarification_target:melbourne"));
    }

    #[test]
    fn stage8_5_tool_choice_the_time_uses_last_turn_clarification_fallback() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::Clarify,
            "Should I check the weather or the time for Melbourne?",
        ));

        let rewritten = ph1x_universal_active_context_followup_query(&thread, Some("The time"))
            .expect("tool-choice answer should resolve from prior clarification text");

        assert_eq!(rewritten, "what is the time in Melbourne");
    }

    #[test]
    fn stage8_5_writing_artifact_shorter_resolves_previous_story() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Write me a short story about a locked factory."),
            Some("At midnight, the locked factory hummed back to life."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "At midnight, the locked factory hummed back to life.",
        ));

        let rewritten =
            ph1x_universal_active_context_followup_query(&thread, Some("Make it shorter."))
                .expect("writing follow-up should resolve the previous story");

        assert!(rewritten.contains("Return only the revised user-facing text"));
        assert!(!rewritten.contains("active writing artifact"));
        assert!(rewritten.contains("locked factory"));
        assert!(rewritten.contains("Previous text"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_EXPECT_REWRITE_REF));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_REFERENCE_PREVIOUS_REF));
    }

    #[test]
    fn stage8_5_writing_artifact_handles_unseen_style_modifiers() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Write me a short story about a locked factory."),
            Some("At midnight, the locked factory hummed back to life."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "At midnight, the locked factory hummed back to life.",
        ));

        let tighten = ph1x_universal_active_context_followup_query(&thread, Some("Tighten it."))
            .expect("unseen concise-edit verb should modify the writing artifact");
        let darker = ph1x_universal_active_context_followup_query(&thread, Some("Make it darker."))
            .expect("style-edit verb should modify the writing artifact");

        assert!(tighten.contains("Return only the revised user-facing text"));
        assert!(!tighten.contains("active writing artifact"));
        assert!(tighten.contains("locked factory"));
        assert!(darker.contains("darker"));
        assert!(darker.contains("Previous text"));
    }

    #[test]
    fn stage8_5_writing_artifact_warmer_keeps_mark_and_next_week() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Draft a message to Mark saying I'll come back next week."),
            Some("Mark, I will come back next week."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "Mark, I will come back next week.",
        ));

        let rewritten =
            ph1x_universal_active_context_followup_query(&thread, Some("Make it warmer."))
                .expect("style follow-up should resolve the draft artifact");

        assert!(rewritten.contains("Mark"));
        assert!(rewritten.contains("come back next week"));
        assert!(rewritten.contains("warmer"));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_FRAME_WRITING_REF));
    }

    #[test]
    fn stage8_5_message_artifact_handles_shorten_and_add_instructions() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Draft a message to Mark saying I'll come back next week."),
            Some("Mark, I will come back next week."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "Mark, I will come back next week.",
        ));

        let shorten =
            ph1x_universal_active_context_followup_query(&thread, Some("Shorten the draft."))
                .expect("draft shortening should modify the active artifact");
        let add_detail = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Add that I'll confirm timing soon."),
        )
        .expect("add-detail instruction should modify the active artifact");

        assert!(shorten.contains("Return only the revised user-facing text"));
        assert!(!shorten.contains("active writing artifact"));
        assert!(shorten.contains("Mark"));
        assert!(add_detail.contains("confirm timing soon"));
        assert!(add_detail.contains("Previous text"));
    }

    #[test]
    fn stage8_5_topic_switch_does_not_steal_name_question() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I'm interested in Japan and doing some skiing and visiting some great Japanese restaurants."),
            Some("Japan has strong ski and restaurant options."),
	            Some("PUBLIC_CHAT"),
	        );

        let rewritten =
            ph1x_universal_active_context_followup_query(&thread, Some("What is your name?"));

        assert!(rewritten.is_none());

        let joke = ph1x_universal_active_context_followup_query(&thread, Some("Tell me a joke."));
        assert!(joke.is_none());
    }

    #[test]
    fn stage8_5_origin_and_metadiscourse_do_not_become_place_followups() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 9:40 AM in New York.",
        ));

        let origin =
            ph1x_universal_active_context_followup_query(&thread, Some("Where are you from?"));
        let meaning =
            ph1x_universal_active_context_followup_query(&thread, Some("With meaning behind it"));

        assert!(origin.is_none());
        assert!(meaning.is_none());
    }

    #[test]
    fn stage8_5_condition_like_query_defers_to_current_turn_weather_route() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 9:40 AM in New York.",
        ));

        let rewritten = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Like in Sydney right now."),
        );

        assert!(rewritten.is_none());
    }

    #[test]
    fn stage8_5_weather_time_correction_uses_prior_location() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolWeather,
            "Sydney is mild and partly cloudy.",
        ));

        let rewritten =
            ph1x_universal_active_context_followup_query(&thread, Some("Not weather, time."))
                .expect("weather-to-time correction should keep the prior location");

        assert_eq!(rewritten, "what is the time in Sydney");
    }

    #[test]
    fn stage8_5_negated_tool_without_replacement_does_not_reroute() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolWeather,
            "Barcelona is mild and cloudy.",
        ));

        let rewritten = ph1x_universal_active_context_followup_query(
            &thread,
            Some("Not the weather, the proof"),
        );

        assert!(rewritten.is_none());
    }

    #[test]
    fn stage8_5_protected_confirmation_preserves_fail_closed_boundary() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Organize payroll for Tim."),
            Some("I can't perform or prepare that protected business action from this turn. NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION."),
            Some("H411_PROTECTED_EXECUTION_FAIL_CLOSED_PRESERVED"),
        );

        let rewritten = ph1x_universal_active_context_followup_query(&thread, Some("Yes, do it."))
            .expect("protected confirmation should stay in protected payroll lane");

        assert_eq!(rewritten, "organize payroll for tim");
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_PROTECTED_RISK_REF));
        assert!(thread
            .pinned_context_refs
            .iter()
            .any(|item| item == STAGE8_5_FAIL_CLOSED_REF));
    }

    #[test]
    fn slice3a_fake_provider_one_line_proposal_reaches_ph1x_validation() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "Provider governance matters because it keeps external model output advisory, bounded, auditable, and unable to bypass Selene's canonical owners.",
        ));
        let proposal = slice3a_fake_one_line_proposal();

        let validated = slice3a_validate_one_line_provider_contract_prep(
            &thread,
            "Can you give me one line?",
            &proposal,
        )
        .expect("fake provider proposal should validate as owner-local prep");

        assert_eq!(
            validated.directive,
            HumanConversationDirective::ModifyPreviousOutput
        );
        assert_eq!(validated.owner_engine, SuggestedNextEngine::Ph1Write);
        assert!(validated
            .target_ref
            .starts_with("ph1x:slice3a:previous_answer:"));
        assert_eq!(validated.provider_call_attempt_count, 1);
        assert_eq!(validated.provider_network_dispatch_count, 0);
        assert!(!validated.raw_provider_output_exposed);
        assert!(!validated.protected_execution_authorized);
        assert!(!validated.simulation_authorized);
        assert!(!validated.authority_authorized);
        assert!(validated
            .evidence_refs
            .iter()
            .any(|evidence| evidence == "ph1x:target:previous_assistant_answer"));
    }

    #[test]
    fn slice3a_provider_off_and_malformed_proposals_fail_before_authority() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "A short prior answer exists for target validation.",
        ));
        let mut provider_off = slice3a_fake_one_line_proposal();
        provider_off.provider_enabled = false;
        provider_off.provider_call_attempt_count = 0;

        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &provider_off,
            ),
            Err(Slice3aOneLineValidationError::ProviderOff)
        );

        let mut malformed = slice3a_fake_one_line_proposal();
        malformed.operation = Slice3aProviderProposalOperation::Unknown;
        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &malformed,
            ),
            Err(Slice3aOneLineValidationError::MalformedProposal)
        );

        let mut dispatched = slice3a_fake_one_line_proposal();
        dispatched.provider_network_dispatch_count = 1;
        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &dispatched,
            ),
            Err(Slice3aOneLineValidationError::ProviderNetworkDispatchAttempted)
        );
    }

    #[test]
    fn slice3a_rejects_wrong_target_unrelated_question_and_protected_authority() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "The previous answer should remain the only allowed rewrite target.",
        ));

        let mut wrong_target = slice3a_fake_one_line_proposal();
        wrong_target.target = Slice3aProviderProposalTarget::StaleOrWrongTarget;
        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &wrong_target,
            ),
            Err(Slice3aOneLineValidationError::WrongTarget)
        );

        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "What is your name?",
                &slice3a_fake_one_line_proposal(),
            ),
            Err(Slice3aOneLineValidationError::CurrentTurnHijack)
        );

        let mut protected = slice3a_fake_one_line_proposal();
        protected.protected_risk = ProtectedRisk::Protected;
        protected.protected_execution_authorized = true;
        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &protected,
            ),
            Err(Slice3aOneLineValidationError::ProviderAuthorityRejected)
        );

        let mut protected_target = slice3a_fake_one_line_proposal();
        protected_target.target = Slice3aProviderProposalTarget::ProtectedAction;
        assert_eq!(
            slice3a_validate_one_line_provider_contract_prep(
                &thread,
                "Can you give me one line?",
                &protected_target,
            ),
            Err(Slice3aOneLineValidationError::ProtectedRiskRejected)
        );
    }

    #[test]
    fn stage8_5c_time_continuation_records_selected_and_rejected_candidates() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 9:40 AM in New York.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(&thread, Some("What about Sydney?"))
            .expect("candidate decision should be produced");

        assert_eq!(
            decision.rewritten_query.as_deref(),
            Some("what is the time in Sydney")
        );
        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActiveToolResult)
        );
        assert!(decision.candidates.len() >= 2);
        assert!(!decision.rejection_ledger.rejected_candidates.is_empty());
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.rejection_reason_text.is_some()));
        assert_eq!(
            decision.owner_output_contract.owner_engine,
            SuggestedNextEngine::Ph1E
        );
        assert_eq!(
            decision.active_context_packet.selected_candidate.as_deref(),
            Some("ph1x_candidate:active_tool_result")
        );
        assert!(decision
            .active_context_packet
            .candidate_rejection_ledger_ref
            .is_some());
    }

    #[test]
    fn stage8_5c_topic_switch_rejects_old_tool_context() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::ToolTime,
            "It's 9:40 AM in New York.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(&thread, Some("What is your name?"))
            .expect("new question still produces a candidate ledger");

        assert!(decision.rewritten_query.is_none());
        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::NewTopicFallback)
        );
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| {
                rejection.rejection_reason_code
                    == Ph1xCandidateRejectionReasonCode::HardDisqualifierExplicitTopicSwitch
            }));
        assert_eq!(
            decision.owner_output_contract.selected_directive,
            HumanConversationDirective::AnswerNewQuestion
        );
    }

    #[test]
    fn stage8_5c_planning_candidate_records_owner_output_contract() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I want to plan a trip to Canada with snow and dining."),
            Some("Canada has several mountain regions with strong dining scenes."),
            Some("PUBLIC_CHAT"),
        );

        let decision =
            ph1x_stage8_5c_candidate_decision(&thread, Some("Where would you base the trip?"))
                .expect("planning follow-up should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActivePlan)
        );
        assert_eq!(
            decision.owner_output_contract.owner_engine,
            SuggestedNextEngine::Ph1Write
        );
        assert_eq!(
            decision
                .owner_output_contract
                .allowed_next_action
                .as_deref(),
            Some("route_to_ph1write")
        );
        assert!(decision
            .active_context_packet
            .user_goal
            .as_deref()
            .is_some_and(|goal| goal.contains("planning")));
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.candidate_ref == "ph1x_candidate:new_topic_fallback"));
    }

    #[test]
    fn stage8_5c_planning_reference_followup_uses_latest_answer_context() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I want to plan a trip to Canada with snow and dining."),
            Some("For the active trip, I would base it in Banff. The Fairmont Banff Springs is a strong lodging option."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "For the active trip, I would base it in Banff. The Fairmont Banff Springs is a strong lodging option.",
        ));

        let decision =
            ph1x_stage8_5c_candidate_decision(&thread, Some("How much should we budget for that?"))
                .expect("reference follow-up should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActivePlan)
        );
        assert_eq!(
            decision.owner_output_contract.owner_engine,
            SuggestedNextEngine::Ph1Write
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Latest Selene answer")));
        assert_eq!(
            decision.active_context_packet.reference_target.as_deref(),
            Some("plan")
        );
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.candidate_ref == "ph1x_candidate:new_topic_fallback"));
    }

    #[test]
    fn stage8_5c_planning_plural_option_reference_selects_active_plan() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I want to plan a two week alpine trip with skiing and food."),
            Some("For your trip, consider basing yourself in Verbier. I would compare W Verbier and Hotel Cordee des Alpes for lodging."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "For your trip, consider basing yourself in Verbier. I would compare W Verbier and Hotel Cordee des Alpes for lodging.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(
            &thread,
            Some("How much should we budget for either of those options?"),
        )
        .expect("plural option reference should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActivePlan)
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Latest Selene answer")));
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.candidate_ref == "ph1x_candidate:new_topic_fallback"));
    }

    #[test]
    fn stage8_5c_definite_lodging_reference_selects_active_plan() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("I want to plan an alpine trip with skiing, dining, and lodging."),
            Some(
                "For lodging, I would compare Alpine House and Ridge Lodge because both fit the active trip.",
            ),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "For lodging, I would compare Alpine House and Ridge Lodge because both fit the active trip.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(
            &thread,
            Some("What should we budget for the hotel?"),
        )
        .expect("definite lodging reference should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActivePlan)
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Latest Selene answer")));
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.candidate_ref == "ph1x_candidate:new_topic_fallback"));
    }

    #[test]
    fn stage8_5c_writing_candidate_keeps_rejection_ledger() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Write me a short story about a locked factory."),
            Some("At midnight, the locked factory hummed back to life."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "At midnight, the locked factory hummed back to life.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(&thread, Some("Tighten it."))
            .expect("writing modification should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActiveWritingArtifact)
        );
        assert_eq!(
            decision.owner_output_contract.selected_directive,
            HumanConversationDirective::ModifyPreviousOutput
        );
        assert!(decision
            .active_context_packet
            .writing_artifact
            .as_deref()
            .is_some_and(|artifact| artifact == STAGE8_5_WRITING_SUBJECT));
        assert!(!decision.rejection_ledger.rejected_candidates.is_empty());
    }

    #[test]
    fn stage8_5c_writing_same_artifact_summary_selects_active_artifact_after_clarify() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Write me a compact scene about an empty observatory."),
            Some("At dusk, the empty observatory opened its roof and caught the first star."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::Clarify,
            "Could you please provide more details on what you would like?",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(
            &thread,
            Some("I'd like a very short version of the same scene."),
        )
        .expect("same-artifact summary should produce candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActiveWritingArtifact)
        );
        assert_eq!(
            decision.owner_output_contract.selected_directive,
            HumanConversationDirective::ModifyPreviousOutput
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Return only the revised user-facing text")));
        assert!(!decision
            .rewritten_query
            .as_deref()
            .unwrap_or_default()
            .contains("active writing artifact"));
        assert!(!decision
            .rewritten_query
            .as_deref()
            .unwrap_or_default()
            .contains("Could you please provide more details"));
    }

    #[test]
    fn stage8_5c_tell_story_seed_supports_shorter_version_followup() {
        let mut thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Tell me a brief story about an empty observatory."),
            Some("At dusk, the empty observatory opened its roof and caught the first star."),
            Some("PUBLIC_CHAT"),
        );
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "At dusk, the empty observatory opened its roof and caught the first star.",
        ));

        let decision =
            ph1x_stage8_5c_candidate_decision(&thread, Some("Give me a shorter version."))
                .expect("story-generation posture should create a modifiable writing frame");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::ActiveWritingArtifact)
        );
        assert_eq!(
            decision.owner_output_contract.selected_directive,
            HumanConversationDirective::ModifyPreviousOutput
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Previous text")));
    }

    #[test]
    fn stage8_5c_latest_answer_pronoun_reference_selects_latest_answer_candidate() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "The current chair is Morgan Lee. Morgan Lee started the role in 2022.",
        ));

        let decision =
            ph1x_stage8_5c_candidate_decision(&thread, Some("How long has she been in that role?"))
                .expect("pronoun follow-up should produce latest-answer candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::LatestSeleneAnswer)
        );
        assert_eq!(
            decision.owner_output_contract.owner_engine,
            SuggestedNextEngine::Ph1Write
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Latest Selene answer")));
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| rejection.candidate_ref == "ph1x_candidate:new_topic_fallback"));
    }

    #[test]
    fn stage8_5c_definite_role_reference_selects_latest_answer_candidate() {
        let mut thread = base_thread();
        thread.last_turn_context = Some(last_turn_context(
            LastTurnRouteClass::PublicChat,
            "The current chair is Morgan Lee. Morgan Lee started the role in 2022.",
        ));

        let decision = ph1x_stage8_5c_candidate_decision(
            &thread,
            Some("How long has the chair had the role?"),
        )
        .expect("definite role follow-up should produce latest-answer candidate proof");

        assert_eq!(
            decision
                .selected_candidate
                .as_ref()
                .map(|candidate| candidate.candidate_kind),
            Some(Ph1xContextCandidateKind::LatestSeleneAnswer)
        );
        assert!(decision
            .rewritten_query
            .as_deref()
            .is_some_and(|query| query.contains("Latest Selene answer")));
    }

    #[test]
    fn stage8_5c_protected_candidate_records_hard_disqualifier() {
        let thread = ph1x_update_universal_active_context_after_turn(
            base_thread(),
            Some("Organize payroll for Tim."),
            Some("I can't perform or prepare that protected business action from this turn. NO_SIMULATION_NO_AUTHORITY_NO_PROTECTED_EXECUTION."),
            Some("H411_PROTECTED_EXECUTION_FAIL_CLOSED_PRESERVED"),
        );

        let decision = ph1x_stage8_5c_candidate_decision(&thread, Some("Yes, do it."))
            .expect("protected continuation should produce candidate proof");

        assert_eq!(
            decision.owner_output_contract.selected_directive,
            HumanConversationDirective::FailClosedProtected
        );
        assert_eq!(
            decision.owner_output_contract.owner_engine,
            SuggestedNextEngine::ProtectedBoundary
        );
        assert!(decision
            .owner_output_contract
            .blocked_actions
            .iter()
            .any(|action| action == "protected_execution_without_authority"));
        assert!(decision
            .rejection_ledger
            .rejected_candidates
            .iter()
            .any(|rejection| {
                rejection.rejection_reason_code
                    == Ph1xCandidateRejectionReasonCode::HardDisqualifierProtectedRisk
            }));
    }

    fn id_text() -> IdentityContext {
        IdentityContext::TextUserId("user-1".to_string())
    }

    fn id_voice_ok() -> IdentityContext {
        let ok = SpeakerAssertionOk::v1(
            SpeakerId::new("spk").unwrap(),
            None,
            vec![DiarizationSegment::v1(now(0), now(1), Some(SpeakerLabel::speaker_a())).unwrap()],
            SpeakerLabel::speaker_a(),
        )
        .unwrap();
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionOk(ok))
    }

    fn id_voice_unknown() -> IdentityContext {
        let u = SpeakerAssertionUnknown::v1(
            IdentityConfidence::Medium,
            ReasonCodeId(1),
            vec![DiarizationSegment::v1(now(0), now(1), None).unwrap()],
        )
        .unwrap();
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionUnknown(u))
    }

    fn id_voice_probable() -> IdentityContext {
        let ok = SpeakerAssertionOk::v1_with_metrics(
            SpeakerId::new("spk-probable").unwrap(),
            Some(selene_kernel_contracts::ph1_voice_id::UserId::new("jd").unwrap()),
            vec![DiarizationSegment::v1(now(0), now(1), Some(SpeakerLabel::speaker_a())).unwrap()],
            SpeakerLabel::speaker_a(),
            DEFAULT_CONF_MID_BP,
            Some(150),
            Some(ReasonCodeId(1)),
            selene_kernel_contracts::ph1_voice_id::SpoofLivenessStatus::Unknown,
            vec![],
        )
        .unwrap();
        IdentityContext::Voice(Ph1VoiceIdResponse::SpeakerAssertionOk(ok))
    }

    fn mem_preferred_name(name: &str) -> MemoryCandidate {
        MemoryCandidate::v1(
            MemoryKey::new("preferred_name").unwrap(),
            MemoryValue::v1(name.to_string(), None).unwrap(),
            MemoryConfidence::High,
            now(1),
            format!("Evidence: {name}"),
            MemoryProvenance::v1(None, None).unwrap(),
            MemorySensitivityFlag::Low,
            MemoryUsePolicy::AlwaysUsable,
            None,
        )
        .unwrap()
    }

    fn mem_sensitive_value(key: &str, value: &str) -> MemoryCandidate {
        MemoryCandidate::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(value.to_string(), None).unwrap(),
            MemoryConfidence::High,
            now(1),
            "Sensitive evidence".to_string(),
            MemoryProvenance::v1(None, None).unwrap(),
            MemorySensitivityFlag::Sensitive,
            MemoryUsePolicy::UserRequestedOnly,
            None,
        )
        .unwrap()
    }

    fn mem_invite_contact(key: &str, contact: &str) -> MemoryCandidate {
        MemoryCandidate::v1(
            MemoryKey::new(key).unwrap(),
            MemoryValue::v1(contact.to_string(), None).unwrap(),
            MemoryConfidence::High,
            now(1),
            format!("Contact evidence: {contact}"),
            MemoryProvenance::v1(None, None).unwrap(),
            MemorySensitivityFlag::Low,
            MemoryUsePolicy::AlwaysUsable,
            None,
        )
        .unwrap()
    }

    fn intent_draft(intent_type: IntentType) -> IntentDraft {
        let excerpt = "what time is it".to_string();
        IntentDraft::v1(
            intent_type,
            SchemaVersion(1),
            vec![],
            vec![],
            OverallConfidence::High,
            vec![EvidenceSpan {
                field: FieldKey::Task,
                transcript_hash: TranscriptHash(1),
                start_byte: 0,
                end_byte: excerpt.len() as u32,
                verbatim_excerpt: excerpt,
            }],
            ReasonCodeId(1),
            SensitivityLevel::Public,
            false,
            vec![],
            vec![],
        )
        .unwrap()
    }

    fn h416_chinese_language_packet() -> LanguagePacket {
        LanguagePacket::v1(
            "zh".to_string(),
            vec!["zh".to_string()],
            9_000,
            "zh".to_string(),
            "same_language_continuity".to_string(),
            "han-simplified".to_string(),
            false,
            false,
            LanguageSwitchScope::ThisTurn,
            "typed_input".to_string(),
            "not_measured".to_string(),
            false,
            vec!["h416_canonical_packet".to_string()],
        )
        .expect("h416 packet should validate")
    }

    fn interrupt_wait() -> InterruptCandidate {
        InterruptCandidate::v1(
            InterruptPhraseSetVersion(1),
            InterruptPhraseId(1),
            InterruptPhraseId(1),
            InterruptLocaleTag::new(PH1K_INTERRUPT_LOCALE_TAG_DEFAULT).unwrap(),
            "wait".to_string(),
            Confidence::new(0.9).unwrap(),
            InterruptCandidateConfidenceBand::High,
            InterruptRiskContextClass::Low,
            InterruptDegradationContext {
                capture_degraded: false,
                aec_unstable: false,
                device_changed: false,
                stream_gap_detected: false,
                class_bundle: DegradationClassBundle::from_flags(false, false, false, false),
            },
            InterruptTimingMarkers {
                window_start: MonotonicTimeNs(0),
                window_end: MonotonicTimeNs(1),
            },
            InterruptSpeechWindowMetrics {
                voiced_window_ms: 1,
            },
            InterruptSubjectRelationConfidenceBundle {
                lexical_confidence: Confidence::new(0.9).unwrap(),
                vad_confidence: Confidence::new(0.9).unwrap(),
                speech_likeness: SpeechLikeness::new(0.9).unwrap(),
                echo_safe_confidence: Confidence::new(0.95).unwrap(),
                nearfield_confidence: Some(Confidence::new(0.8).unwrap()),
                combined_confidence: Confidence::new(0.9).unwrap(),
            },
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

    fn tts_snapshot(text: &str, spoken_cursor_byte: u32) -> TtsResumeSnapshot {
        TtsResumeSnapshot::v1(
            AnswerId(77),
            Some("task_status".to_string()),
            text.to_string(),
            spoken_cursor_byte,
        )
        .unwrap()
    }

    fn dummy_source_metadata() -> SourceMetadata {
        SourceMetadata {
            schema_version: SchemaVersion(1),
            provider_hint: None,
            retrieved_at_unix_ms: 1,
            sources: vec![SourceRef {
                title: "source".to_string(),
                url: "https://example.invalid".to_string(),
            }],
            web_answer_verification: None,
        }
    }

    fn h415b_verified_source_metadata() -> SourceMetadata {
        let response_text = "Mira Solen is the CEO of Aurora Vale Cellars.".to_string();
        let hash = "0".repeat(64);
        let source_chip = SourceChipPacket {
            source_id: "source_001".to_string(),
            label: "Aurora Vale Cellars official leadership".to_string(),
            domain: "aurora-vale-cellars.test".to_string(),
            safe_click_url: "https://aurora-vale-cellars.test/leadership".to_string(),
            source_type: "WEB_RESULT".to_string(),
            accepted: true,
            claim_refs: vec!["claim_001".to_string()],
            icon_key: Some("source_link".to_string()),
            verified_for_claim: true,
            display_rank: 1,
            tooltip_or_accessibility_label:
                "Open Aurora Vale Cellars official leadership from aurora-vale-cellars.test"
                    .to_string(),
        };
        let presentation = PresentationPacket {
            display_text: response_text.clone(),
            response_text: response_text.clone(),
            tts_text: response_text.clone(),
            answer_class: "VERIFIED_DIRECT_ANSWER".to_string(),
            language: "en".to_string(),
            source_chips: vec![source_chip.clone()],
            source_cards: vec![SourceCardPacket {
                source_id: "source_001".to_string(),
                title: "Aurora Vale Cellars official leadership".to_string(),
                domain: "aurora-vale-cellars.test".to_string(),
                safe_click_url: "https://aurora-vale-cellars.test/leadership".to_string(),
                source_type: "WEB_RESULT".to_string(),
                short_excerpt_or_summary: "Accepted source for claim_001".to_string(),
                accepted: true,
                claim_refs: vec!["claim_001".to_string()],
                display_rank: 1,
                retrieved_at_human: None,
                metadata_safe_for_user: true,
            }],
            image_cards: Vec::new(),
            trace_id: "stage5_test_trace".to_string(),
            metadata_safe_for_user: true,
            response_style: "concise_default".to_string(),
            expandable_available: true,
            presentation_boundary_used: "PH1X_TEST_PRESENTATION".to_string(),
        };
        SourceMetadata {
            schema_version: SchemaVersion(1),
            provider_hint: Some("stage1_fixture".to_string()),
            retrieved_at_unix_ms: 1,
            sources: vec![SourceRef {
                title: "Aurora Vale Cellars official leadership".to_string(),
                url: "https://aurora-vale-cellars.test/leadership".to_string(),
            }],
            web_answer_verification: Some(WebAnswerVerificationPacket {
                requested_entity: RequestedEntityPacket {
                    requested_entity_id: "entity_aurora_vale_cellars".to_string(),
                    captured_text: "Aurora Vale Cellars".to_string(),
                    normalized_name: "aurora vale cellars".to_string(),
                    entity_type: "ORGANIZATION".to_string(),
                    known_entity_status: "SYNTHETIC_OR_TEST".to_string(),
                    synthetic_allowed: true,
                    source_turn_id: "turn_h415b".to_string(),
                    language_hint: Some("en".to_string()),
                    confidence: 9_000,
                },
                normalized_entity: "aurora vale cellars".to_string(),
                query: "Who is the CEO of Aurora Vale Cellars?".to_string(),
                expanded_query: "Who is the CEO of Aurora Vale Cellars?".to_string(),
                source_candidate_ids: vec!["source_001".to_string(), "source_002".to_string()],
                accepted_source_ids: vec!["source_001".to_string()],
                rejected_source_ids: vec!["source_002".to_string()],
                source_evaluations: vec![
                    SourceEvaluationPacket {
                        source_id: "source_001".to_string(),
                        requested_entity_id: "entity_aurora_vale_cellars".to_string(),
                        title: "Aurora Vale Cellars official leadership".to_string(),
                        domain: "aurora-vale-cellars.test".to_string(),
                        url: "https://aurora-vale-cellars.test/leadership".to_string(),
                        entity_match_result: "ENTITY_MATCH_STRONG".to_string(),
                        claim_support_result: "CLAIM_SUPPORT_DIRECT".to_string(),
                        source_strength: "ACCEPTED_DIRECT".to_string(),
                        accepted: true,
                        rejection_reasons: vec![],
                        claim_refs: vec!["claim_001".to_string()],
                        safe_for_user_display: true,
                        safe_for_tts: true,
                    },
                    SourceEvaluationPacket {
                        source_id: "source_002".to_string(),
                        requested_entity_id: "entity_aurora_vale_cellars".to_string(),
                        title: "Our CEO and executive management team | Southern Wine Board"
                            .to_string(),
                        domain: "regional-wine-board.test".to_string(),
                        url: "https://regional-wine-board.test/leadership".to_string(),
                        entity_match_result: "ENTITY_MATCH_REJECT".to_string(),
                        claim_support_result: "CLAIM_SUPPORT_NONE".to_string(),
                        source_strength: "REJECTED".to_string(),
                        accepted: false,
                        rejection_reasons: vec!["ENTITY_MISMATCH".to_string()],
                        claim_refs: vec![],
                        safe_for_user_display: false,
                        safe_for_tts: false,
                    },
                ],
                accepted_sources: vec![AcceptedSourcePacket {
                    source_id: "source_001".to_string(),
                    label: "Aurora Vale Cellars official leadership".to_string(),
                    domain: "aurora-vale-cellars.test".to_string(),
                    safe_click_url: "https://aurora-vale-cellars.test/leadership".to_string(),
                    source_type: "WEB_RESULT".to_string(),
                    supported_claim_refs: vec!["claim_001".to_string()],
                    entity_match_result: "ENTITY_MATCH_STRONG".to_string(),
                    claim_support_result: "CLAIM_SUPPORT_DIRECT".to_string(),
                    accepted: true,
                }],
                rejected_sources: vec![RejectedSourcePacket {
                    source_id: "source_002".to_string(),
                    domain: "regional-wine-board.test".to_string(),
                    source_type: "WEB_RESULT".to_string(),
                    accepted: false,
                    rejection_reasons: vec!["ENTITY_MISMATCH".to_string()],
                    entity_match_result: "ENTITY_MATCH_REJECT".to_string(),
                    claim_support_result: "CLAIM_SUPPORT_NONE".to_string(),
                    trace_only: true,
                }],
                source_chips: vec![source_chip],
                answer_claims: vec![response_text.clone()],
                claim_to_source_map: vec![("claim_001".to_string(), "source_001".to_string())],
                claim_requests: vec![ClaimRequestPacket {
                    request_id: "stage4_req_h415b".to_string(),
                    turn_id: "turn_h415b".to_string(),
                    claim_id: "claim_001".to_string(),
                    claim_type: "leadership_role".to_string(),
                    requested_entity: "Aurora Vale Cellars".to_string(),
                    normalized_entity: "aurora vale cellars".to_string(),
                    claim_text: "Verify the CEO of Aurora Vale Cellars.".to_string(),
                    expected_answer_shape: "person_role_entity".to_string(),
                    freshness_required: true,
                    source_requirements: vec![
                        "accepted_source".to_string(),
                        "person_role_entity_direct_support".to_string(),
                    ],
                    generated_from_user_prompt: true,
                    protected_lane: false,
                }],
                claim_verifications: vec![ClaimVerificationPacket {
                    claim_id: "claim_001".to_string(),
                    claim_type: "leadership_role".to_string(),
                    claim_text: "Verify the CEO of Aurora Vale Cellars.".to_string(),
                    requested_entity: "Aurora Vale Cellars".to_string(),
                    verification_status: "SUPPORTED".to_string(),
                    confidence: 9_000,
                    confidence_class: "HIGH".to_string(),
                    supporting_sources: vec!["source_001".to_string()],
                    contradicting_sources: vec![],
                    insufficient_sources: vec![],
                    rejected_sources: vec!["source_002".to_string()],
                    evidence_links: vec![ClaimEvidenceLink {
                        claim_id: "claim_001".to_string(),
                        source_id: "source_001".to_string(),
                        evidence_chunk_id: "source_001_evidence".to_string(),
                        evidence_excerpt_hash: hash.clone(),
                        entity_match: "ENTITY_MATCH_STRONG".to_string(),
                        claim_term_match: "CLAIM_SUPPORT_DIRECT".to_string(),
                        role_or_value_match: "CEO".to_string(),
                        freshness_match: "FRESH_OR_EVERGREEN".to_string(),
                        support_level: "DIRECT_SUPPORT".to_string(),
                        contradiction_level: "no_conflict".to_string(),
                        confidence: 9_000,
                        confidence_class: "HIGH".to_string(),
                        reason: "direct accepted evidence supports the material claim".to_string(),
                    }],
                    uncertainty_reason: None,
                    selected_answer_value: Some("Mira Solen".to_string()),
                    source_hierarchy_reason: None,
                    freshness_reason: Some(
                        "fresh or evergreen accepted evidence supported the current claim"
                            .to_string(),
                    ),
                    safe_for_direct_answer: true,
                    user_visible_summary: response_text.clone(),
                }],
                unsupported_claims_removed: vec![],
                contradiction_result: "no_conflict".to_string(),
                final_answer_class: "VERIFIED_DIRECT_ANSWER".to_string(),
                presentation,
                response_text: response_text.clone(),
                source_dump_present: false,
                rejected_sources_present_in_response_text: false,
                debug_trace_present_in_response_text: false,
                tts_input_text: response_text,
                displayed_response_text_sha256: hash.clone(),
                tts_input_text_sha256: hash,
                provider_call_count_when_disabled: 0,
            }),
        }
    }

    fn stage5_test_verified_source_metadata() -> SourceMetadata {
        let mut metadata = h415b_verified_source_metadata();
        metadata.sources = vec![SourceRef {
            title: "Test Company A official leadership".to_string(),
            url: "https://test-company-a.test/leadership".to_string(),
        }];
        let Some(verification) = metadata.web_answer_verification.as_mut() else {
            return metadata;
        };
        let response_text = "Test Person A is the CEO of Test Company A.".to_string();
        verification.requested_entity.requested_entity_id = "entity_test_company_a".to_string();
        verification.requested_entity.captured_text = "Test Company A".to_string();
        verification.requested_entity.normalized_name = "test company a".to_string();
        verification.normalized_entity = "test company a".to_string();
        verification.query = "Who is the CEO of Test Company A?".to_string();
        verification.expanded_query = verification.query.clone();
        if let Some(evaluation) = verification.source_evaluations.get_mut(0) {
            evaluation.requested_entity_id = "entity_test_company_a".to_string();
            evaluation.title = "Test Company A official leadership".to_string();
            evaluation.domain = "test-company-a.test".to_string();
            evaluation.url = "https://test-company-a.test/leadership".to_string();
        }
        if let Some(source) = verification.accepted_sources.get_mut(0) {
            source.label = "Test Company A official leadership".to_string();
            source.domain = "test-company-a.test".to_string();
            source.safe_click_url = "https://test-company-a.test/leadership".to_string();
        }
        if let Some(chip) = verification.source_chips.get_mut(0) {
            chip.label = "Test Company A official leadership".to_string();
            chip.domain = "test-company-a.test".to_string();
            chip.safe_click_url = "https://test-company-a.test/leadership".to_string();
            chip.tooltip_or_accessibility_label =
                "Open Test Company A official leadership from test-company-a.test".to_string();
        }
        verification.answer_claims = vec![response_text.clone()];
        if let Some((_, source_id)) = verification.claim_to_source_map.get_mut(0) {
            *source_id = "source_001".to_string();
        }
        if let Some(request) = verification.claim_requests.get_mut(0) {
            request.requested_entity = "Test Company A".to_string();
            request.normalized_entity = "test company a".to_string();
            request.claim_text = "Verify the CEO of Test Company A.".to_string();
        }
        if let Some(claim) = verification.claim_verifications.get_mut(0) {
            claim.claim_text = "Verify the CEO of Test Company A.".to_string();
            claim.requested_entity = "Test Company A".to_string();
            claim.selected_answer_value = Some("Test Person A".to_string());
            claim.user_visible_summary = response_text.clone();
        }
        verification.response_text = response_text.clone();
        verification.tts_input_text = response_text.clone();
        verification.presentation.display_text = response_text.clone();
        verification.presentation.response_text = response_text.clone();
        verification.presentation.tts_text = response_text.clone();
        verification.presentation.language = "en".to_string();
        verification.presentation.source_chips = verification.source_chips.clone();
        if let Some(card) = verification.presentation.source_cards.get_mut(0) {
            card.title = "Test Company A official leadership".to_string();
            card.domain = "test-company-a.test".to_string();
            card.safe_click_url = "https://test-company-a.test/leadership".to_string();
        }
        metadata
    }

    #[test]
    fn final_e2e_chinese_source_discovery_presentation_uses_chinese() {
        let mut metadata = stage5_test_verified_source_metadata();
        let verification = metadata
            .web_answer_verification
            .as_mut()
            .expect("verification packet");
        verification.final_answer_class = "SOURCE_DISCOVERY_ONLY".to_string();
        verification.presentation.answer_class = "SOURCE_DISCOVERY_ONLY".to_string();
        verification.requested_entity.captured_text = "人工智能研究".to_string();

        let text = stage5_chinese_web_presentation_text(verification)
            .expect("source discovery class should have Chinese presentation");
        assert_eq!(text, "我找到了关于 人工智能研究 的可引用网页结果。");
    }

    fn stage6_test_image_source_metadata() -> SourceMetadata {
        let mut metadata = stage5_test_verified_source_metadata();
        let Some(verification) = metadata.web_answer_verification.as_mut() else {
            return metadata;
        };
        verification.presentation.presentation_boundary_used =
            "PH1E_STAGE6_IMAGE_PRESENTATION".to_string();
        verification.presentation.trace_id = "stage6_test_trace".to_string();
        verification.presentation.image_cards = vec![SearchImagePacket {
            image_id: "stage6_image_test_a".to_string(),
            image_kind: "logo".to_string(),
            approved_asset_ref: "fixture-image-a.png".to_string(),
            safe_image_url: None,
            thumbnail_url: None,
            source_page_url: "https://test-company-a.test/leadership".to_string(),
            source_page_domain: "test-company-a.test".to_string(),
            source_label: "Test Company A official leadership".to_string(),
            caption: "Test Company A approved fixture image".to_string(),
            alt_text: "Test Company A approved fixture image".to_string(),
            query_relevance_score: 9_500,
            entity_match_score: 9_500,
            source_id: "source_001".to_string(),
            claim_refs: vec!["claim_001".to_string()],
            display_allowed: true,
            display_denied_reason: None,
            provider: "stage6_fixture".to_string(),
            provider_tier: "fixture".to_string(),
            metadata_only: false,
            rights_or_policy_status: "fixture_approved".to_string(),
            retrieved_at: None,
            metadata_safe_for_user: true,
            remote_image_load_allowed: false,
            fixture_or_local_asset: true,
            display_rank: 1,
            result_classes: vec![
                "STAGE6_IMAGE_PACKET_PASS".to_string(),
                "STAGE6_IMAGE_DISPLAY_GATE_PASS".to_string(),
                "STAGE6_IMAGE_URL_SAFETY_PASS".to_string(),
                "STAGE6_SOURCE_PAGE_LINK_PASS".to_string(),
                "STAGE6_QUERY_RELEVANCE_PASS".to_string(),
                "STAGE6_IMAGE_FETCH_OFF_ZERO_ATTEMPT_PASS".to_string(),
                "STAGE6_NO_REMOTE_IMAGE_LOAD_WHEN_FETCH_DISABLED_PASS".to_string(),
            ],
        }];
        metadata
    }

    #[test]
    fn at_x_dispatches_read_only_time_query_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::Time),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_web_search_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            2,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::WebSearchQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::WebSearch),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_news_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            3,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::NewsQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::News),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_url_fetch_and_cite_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            4,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::UrlFetchAndCiteQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::UrlFetchAndCite),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_document_understand_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            5,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DocumentUnderstandQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => {
                        assert_eq!(t.tool_name, ToolName::DocumentUnderstand)
                    }
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_photo_understand_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            6,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::PhotoUnderstandQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::PhotoUnderstand),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_data_analysis_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            13,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DataAnalysisQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::DataAnalysis),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_deep_research_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            15,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DeepResearchQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::DeepResearch),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_record_mode_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            17,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::RecordModeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::RecordMode),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_connector_query_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            20,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::ConnectorQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::ConnectorQuery),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_dispatches_read_only_list_reminders_to_tool_router_and_sets_pending_tool() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            21,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::ListReminders,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => {
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Tool { .. })
                ));
                match d.dispatch_request {
                    DispatchRequest::Tool(t) => assert_eq!(t.tool_name, ToolName::ConnectorQuery),
                    DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                    DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
    }

    #[test]
    fn at_x_tool_query_includes_project_and_pinned_context_refs_when_present() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let thread_state = base_thread()
            .with_project_context(
                Some("proj_q3_planning".to_string()),
                vec![
                    "ctx_budget_sheet".to_string(),
                    "ctx_roadmap_notes".to_string(),
                ],
            )
            .unwrap();

        let req = Ph1xRequest::v1(
            19,
            1,
            now(1),
            thread_state,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::WebSearchQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::Tool(t) => {
                    assert!(t.query.contains("project_id=proj_q3_planning"));
                    assert!(t
                        .query
                        .contains("pinned_context_refs=ctx_budget_sheet,ctx_roadmap_notes"));
                }
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
    }

    #[test]
    fn at_x_continuity_speaker_mismatch_fails_closed_into_one_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            91,
            1,
            now(1),
            base_thread_with_continuity("trip_japan", "user-1"),
            SessionState::Active,
            IdentityContext::TextUserId("user-2".to_string()),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Sure, let me help.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_continuity_context("trip_japan".to_string(), "user-2".to_string())
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert_eq!(out.reason_code, reason_codes::X_CONTINUITY_SPEAKER_MISMATCH);
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::ReferenceTarget]);
                assert!((2..=3).contains(&c.accepted_answer_formats.len()));
            }
            _ => panic!("expected Clarify directive"),
        }
    }

    #[test]
    fn at_x_continuity_subject_mismatch_with_pending_fails_closed_into_one_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let thread = ThreadState::v1(
            Some(PendingState::Clarify {
                missing_field: FieldKey::Task,
                attempts: 1,
            }),
            None,
        )
        .with_continuity(Some("trip_japan".to_string()), Some("user-1".to_string()))
        .unwrap();

        let req = Ph1xRequest::v1(
            92,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Okay.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_continuity_context("payroll".to_string(), "user-1".to_string())
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert_eq!(out.reason_code, reason_codes::X_CONTINUITY_SUBJECT_MISMATCH);
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::ReferenceTarget]);
                assert_eq!(c.accepted_answer_formats.len(), 2);
            }
            _ => panic!("expected Clarify directive"),
        }
    }

    #[test]
    fn at_x_tool_ok_completes_pending_dispatch_into_respond() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            7,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::Time {
                local_time_iso: "2026-02-09T12:00:00-05:00".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            7,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("2026-02-09T12:00:00-05:00"));
            }
            _ => panic!("expected Respond"),
        }
        assert!(out2.thread_state.pending.is_none());
        assert!(out2.idempotency_key.is_some());
    }

    #[test]
    fn at_x_tool_ok_time_new_york_renders_clean_final_answer_without_sources() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            7,
            11,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::Time {
                local_time_iso: "2026-04-24T22:42:00-04:00[America/New_York]".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            7,
            12,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "It's 10:42 PM in New York.");
                assert!(!r.response_text.contains("Sources:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_time_japan_renders_clean_final_answer_without_raw_iso() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            7,
            21,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::Time {
                local_time_iso: "2026-04-25T09:42:00+09:00[Asia/Tokyo]".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            7,
            22,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "It's 9:42 AM in Japan.");
                assert!(!r.response_text.contains("[Asia/Tokyo]"));
                assert!(!r.response_text.contains("Sources:"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_time_sydney_renders_clean_final_answer_without_raw_iso() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            7,
            31,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::Time {
                local_time_iso: "2026-04-25T10:42:00+10:00[Australia/Sydney]".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            7,
            32,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "It's 10:42 AM in Sydney.");
                assert!(!r.response_text.contains("[Australia/Sydney]"));
                assert!(!r.response_text.contains("Sources:"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn h362_tool_ok_time_uses_explicit_place_label_without_raw_iso() {
        let tool_ok = ToolResponse::ok_v1(
            selene_kernel_contracts::ph1e::ToolRequestId(1),
            selene_kernel_contracts::ph1e::ToolQueryHash(1),
            ToolResult::Time {
                local_time_iso: "2026-04-25T14:42:00+02:00[Europe/Berlin|Germany]".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(text, "It's 2:42 PM in Germany.");
        assert!(!text.contains("Europe/Berlin"));
        assert!(!text.contains("2026-04-25"));
        assert!(!text.contains("Sources:"));
    }

    #[test]
    fn h416_answer_language_ph1x_time_tool_uses_canonical_language_packet_before_formatter() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let request_id = selene_kernel_contracts::ph1e::ToolRequestId(4161);
        let tool_ok = ToolResponse::ok_v1(
            request_id,
            selene_kernel_contracts::ph1e::ToolQueryHash(4161),
            ToolResult::Time {
                local_time_iso: "2026-04-25T09:42:00+09:00[Asia/Tokyo]".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();
        let req = Ph1xRequest::v1(
            416,
            1,
            now(1),
            ThreadState::v1(
                Some(PendingState::Tool {
                    request_id,
                    attempts: 1,
                }),
                None,
            ),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            Some("zh".to_string()),
            None,
        )
        .unwrap()
        .with_language_packet(Some(h416_chinese_language_packet()))
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "东京现在是9:42 AM。");
                assert!(!r.response_text.starts_with("It's "));
                assert!(!r.response_text.contains("[Asia/Tokyo]"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn h416_answer_language_ph1x_weather_tool_uses_canonical_language_packet_before_formatter() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let request_id = selene_kernel_contracts::ph1e::ToolRequestId(4162);
        let tool_ok = ToolResponse::ok_v1(
            request_id,
            selene_kernel_contracts::ph1e::ToolQueryHash(4162),
            ToolResult::Weather {
                summary: "Tokyo is 21°C and partly cloudy.".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();
        let req = Ph1xRequest::v1(
            416,
            2,
            now(2),
            ThreadState::v1(
                Some(PendingState::Tool {
                    request_id,
                    attempts: 1,
                }),
                None,
            ),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            Some("zh".to_string()),
            None,
        )
        .unwrap()
        .with_language_packet(Some(h416_chinese_language_packet()))
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "东京现在是21°C，天气局部多云。");
                assert!(!r.response_text.starts_with("Tokyo is "));
                assert!(!r.response_text.contains("provider_payload"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn h362_time_ambiguity_renders_clean_clarification_options() {
        let text = retry_message_for_failure(
            selene_engines::ph1e::reason_codes::E_FAIL_QUERY_PARSE,
            Some(
                "ambiguous_time_location alternatives=mainland Portugal/Lisbon|Madeira|the Azores",
            ),
        );
        assert_eq!(
            text,
            "That place has more than one timezone. Do you mean mainland Portugal/Lisbon, Madeira, or the Azores?"
        );
        assert!(!text.contains("raw"));
        assert!(!text.contains("Retrieved at"));
    }

    #[test]
    fn h362_time_ambiguity_hides_technical_timezone_ids_for_user_clarification() {
        let text = retry_message_for_failure(
            selene_engines::ph1e::reason_codes::E_FAIL_QUERY_PARSE,
            Some("ambiguous_time_location alternatives=Australia/Lord_Howe (Lord Howe Island)|Antarctica/Macquarie (Macquarie Island)|Australia/Hobart (Tasmania)"),
        );
        assert_eq!(
            text,
            "That place has more than one timezone. Which city or local place should I use?"
        );
        assert!(!text.contains("Australia/"));
        assert!(!text.contains("Antarctica/"));
        assert!(!text.contains("IANA"));
    }

    #[test]
    fn stage2_disabled_provider_failure_renders_clean_no_search_response_for_tts() {
        let text = retry_message_for_failure(
            selene_engines::ph1e::reason_codes::E_FAIL_POLICY_BLOCK,
            Some("stage2_provider_control=1 route=WebSearch provider=brave_web_search allowed=false deny_reason=WEB_ADMIN_DISABLED provider_call_attempt_count=0 provider_network_dispatch_count=0"),
        );
        assert_eq!(
            text,
            selene_engines::ph1providerctl::PROVIDER_DISABLED_RESPONSE_TEXT
        );
        assert!(!text.contains("stage2_provider_control"));
        assert!(!text.contains("brave_web_search"));
        assert!(!text.contains("provider_call_attempt_count"));
    }

    #[test]
    fn stage7_provider_off_detail_stays_out_of_public_and_tts_text() {
        let text = retry_message_for_failure(
            selene_engines::ph1e::reason_codes::E_FAIL_POLICY_BLOCK,
            Some("stage2_provider_control=1 route=WebSearch provider=brave_web_search allowed=false deny_reason=WEB_ADMIN_DISABLED provider_call_attempt_count=0 provider_network_dispatch_count=0 billing_scope=NON_BILLABLE billable_class=BLOCKED_NOT_BILLABLE"),
        );
        assert_eq!(
            text,
            selene_engines::ph1providerctl::PROVIDER_DISABLED_RESPONSE_TEXT
        );
        for forbidden in [
            "stage2_provider_control",
            "provider_call_attempt_count",
            "provider_network_dispatch_count",
            "brave_web_search",
            "billing_scope",
            "BILLABLE",
            "Sources:",
            "raw provider",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
    }

    #[test]
    fn h363_time_missing_place_renders_clean_clarification() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let request_id = ToolRequestId(363);
        let thread_state = ThreadState::v1(
            Some(PendingState::Tool {
                request_id,
                attempts: 1,
            }),
            None,
        );
        let tool_fail = ToolResponse::fail_with_detail_v1(
            request_id,
            ToolQueryHash(363),
            selene_engines::ph1e::reason_codes::E_FAIL_QUERY_PARSE,
            Some("missing_time_location".to_string()),
            CacheStatus::Bypassed,
        )
        .unwrap();
        let req = Ph1xRequest::v1(
            363,
            1,
            now(1),
            thread_state,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_fail),
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.question, "Which place do you mean?");
                assert!(c.what_is_missing.contains(&FieldKey::Place));
                assert_eq!(
                    out.thread_state
                        .resume_buffer
                        .as_ref()
                        .and_then(|buffer| buffer.topic_hint.as_deref()),
                    Some(DETERMINISTIC_TIME_CLARIFICATION_TOPIC)
                );
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Clarify {
                        missing_field: FieldKey::Place,
                        ..
                    })
                ));
            }
            _ => panic!("expected deterministic time clarification"),
        }
    }

    #[test]
    fn at_x_tool_ok_weather_renders_clean_final_answer_without_sources() {
        let tool_ok = ToolResponse::ok_v1(
            selene_kernel_contracts::ph1e::ToolRequestId(1),
            selene_kernel_contracts::ph1e::ToolQueryHash(1),
            ToolResult::Weather {
                summary: "It's 22°C in Tokyo, Japan, with partly cloudy skies.".to_string(),
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(text, "It's 22°C in Tokyo, Japan, with partly cloudy skies.");
        assert!(!text.contains("Sources:"));
        assert!(!text.contains("Retrieved at (unix_ms):"));
    }

    #[test]
    fn h364_weather_tool_ambiguity_sets_place_resume_topic() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let request_id = ToolRequestId(364);
        let pending = ThreadState::v1(
            Some(PendingState::Tool {
                request_id,
                attempts: 1,
            }),
            None,
        );
        let amb = StructuredAmbiguity {
            summary:
                "Weather varies by city and place inside Portugal, so I need a specific location."
                    .to_string(),
            alternatives: vec![
                "Lisbon, Portugal".to_string(),
                "Porto, Portugal".to_string(),
                "Funchal, Madeira".to_string(),
            ],
        };
        let tool = ToolResponse {
            schema_version: SchemaVersion(1),
            request_id,
            query_hash: ToolQueryHash(364),
            tool_status: ToolStatus::Ok,
            tool_result: Some(ToolResult::Weather {
                summary: amb.summary.clone(),
            }),
            source_metadata: Some(dummy_source_metadata()),
            reason_code: ReasonCodeId(1),
            fail_reason_code: None,
            fail_detail: None,
            ambiguity: Some(amb),
            cache_status: CacheStatus::Bypassed,
        };
        let req = Ph1xRequest::v1(
            364,
            1,
            now(1),
            pending,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::WeatherQuery,
            ))),
            Some(tool),
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert!(c.question.contains("Which place should I use?"));
                assert_eq!(c.what_is_missing, vec![FieldKey::Place]);
                assert!(c
                    .accepted_answer_formats
                    .contains(&"Lisbon, Portugal".to_string()));
                assert_eq!(
                    out.thread_state
                        .resume_buffer
                        .as_ref()
                        .and_then(|buffer| buffer.topic_hint.as_deref()),
                    Some(DETERMINISTIC_WEATHER_CLARIFICATION_TOPIC)
                );
                assert!(matches!(
                    out.thread_state.pending,
                    Some(PendingState::Clarify {
                        missing_field: FieldKey::Place,
                        ..
                    })
                ));
            }
            _ => panic!("expected weather clarification"),
        }
    }

    #[test]
    fn best_available_web_search_without_verification_returns_closest_result_without_dump() {
        let text = web_search_without_verification_safe_degrade(&[ToolTextSnippet {
            title: "Fixture Entity Alpha official update".to_string(),
            snippet: "This snippet must stay out of response_text.".to_string(),
            url: "https://alpha-search-fixture.test/update".to_string(),
        }]);

        assert_eq!(
            text,
            "The closest search result I found is Fixture Entity Alpha official update."
        );
        assert!(!text.contains("This snippet"));
        assert!(!text.contains("https://"));
        assert!(!text.contains("Confidence:"));
        assert!(!text.contains("Sources:"));
    }

    #[test]
    fn at_x_tool_ok_web_search_renders_clean_answer_with_sources_without_raw_timestamp() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            8,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::WebSearchQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Result".to_string(),
                    snippet: "Snippet".to_string(),
                    url: "https://example.invalid/search-result".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            8,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r
                    .response_text
                    .contains("The closest search result I found is Result."));
                assert!(!r.response_text.contains("I found a web result"));
                assert!(!r.response_text.contains("Snippet"));
                assert!(!r.response_text.contains("https://example.invalid"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_news_renders_clean_answer_with_sources_without_raw_timestamp() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            9,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::NewsQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::News {
                items: vec![ToolTextSnippet {
                    title: "Headline".to_string(),
                    snippet: "Snippet".to_string(),
                    url: "https://example.invalid/news-result".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            9,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r
                    .response_text
                    .contains("The closest search result I found is Headline."));
                assert!(!r.response_text.contains("I found a web result"));
                assert!(!r.response_text.contains("Snippet"));
                assert!(!r.response_text.contains("https://example.invalid"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn h415b_ph1x_uses_verification_packet_and_keeps_rejected_sources_out_of_answer() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(415),
            ToolQueryHash(415),
            ToolResult::WebSearch {
                items: vec![
                    ToolTextSnippet {
                        title: "Aurora Vale Cellars official leadership".to_string(),
                        snippet: "Mira Solen is the CEO of Aurora Vale Cellars.".to_string(),
                        url: "https://aurora-vale-cellars.test/leadership".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Our CEO and executive management team | Southern Wine Board"
                            .to_string(),
                        snippet: "Dr Rowan Vale is the CEO of Southern Wine Board.".to_string(),
                        url: "https://regional-wine-board.test/leadership".to_string(),
                    },
                ],
            },
            h415b_verified_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(text, "Mira Solen is the CEO of Aurora Vale Cellars.");
        for forbidden in [
            "Southern Wine Board",
            "Dr Rowan Vale",
            "regional-wine-board.test",
            "Sources:",
            "https://",
            "source:",
            "trust:",
            "AUDIT_METADATA_ONLY",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
        let verification = tool_ok
            .source_metadata
            .as_ref()
            .and_then(|metadata| metadata.web_answer_verification.as_ref())
            .expect("verification packet must be present");
        assert_eq!(verification.accepted_source_ids, vec!["source_001"]);
        assert_eq!(verification.rejected_source_ids, vec!["source_002"]);
        assert_eq!(verification.source_chips.len(), 1);
        assert_eq!(verification.source_chips[0].source_id, "source_001");
        assert_eq!(verification.tts_input_text, text);
        assert_eq!(
            verification.displayed_response_text_sha256,
            verification.tts_input_text_sha256
        );
    }

    #[test]
    fn stage4_ph1x_preserves_claim_verification_and_clean_tts_text() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(4_004),
            ToolQueryHash(4_004),
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Raw source title should not drive answer".to_string(),
                    snippet: "Raw source snippet should not be spoken.".to_string(),
                    url: "https://example.invalid/raw-source".to_string(),
                }],
            },
            h415b_verified_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        let verification = tool_ok
            .source_metadata
            .as_ref()
            .and_then(|metadata| metadata.web_answer_verification.as_ref())
            .expect("Stage 4 verification metadata must be present");
        assert_eq!(text, verification.response_text);
        assert_eq!(verification.tts_input_text, verification.response_text);
        assert_eq!(verification.claim_requests.len(), 1);
        assert_eq!(verification.claim_verifications.len(), 1);
        assert_eq!(
            verification.claim_verifications[0].verification_status,
            "SUPPORTED"
        );
        assert_eq!(verification.claim_verifications[0].confidence_class, "HIGH");
        assert!(verification.claim_verifications[0].safe_for_direct_answer);
        assert!(!text.contains("Raw source"));
        assert!(!text.contains("Sources:"));
        assert!(!text.contains("source:"));
        assert!(!text.contains("provider_call_attempt_count"));
    }

    #[test]
    fn stage5_ph1x_preserves_same_language_websearch_presentation() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(5_001),
            ToolQueryHash(5_001),
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Test Company A official leadership".to_string(),
                    snippet: "Test Person A is the CEO of Test Company A.".to_string(),
                    url: "https://test-company-a.test/leadership".to_string(),
                }],
            },
            stage5_test_verified_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();
        let req = Ph1xRequest::v1(
            5_001,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            Some("zh".to_string()),
            None,
        )
        .unwrap()
        .with_language_packet(Some(h416_chinese_language_packet()))
        .unwrap();

        let text = tool_ok_text_for_request(&req, req.tool_response.as_ref().unwrap());
        assert_eq!(text, "Test Person A 是 Test Company A 的 CEO。");
        assert!(!text.contains("Sources:"));
        assert!(!text.contains("https://"));
        assert!(!text.contains("I found a web result"));
    }

    #[test]
    fn stage6_ph1x_preserves_clean_text_while_image_cards_remain_metadata_only() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(6_001),
            ToolQueryHash(6_001),
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Test Company A official leadership".to_string(),
                    snippet: "Test Person A is the CEO of Test Company A.".to_string(),
                    url: "https://test-company-a.test/leadership".to_string(),
                }],
            },
            stage6_test_image_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        let verification = tool_ok
            .source_metadata
            .as_ref()
            .and_then(|metadata| metadata.web_answer_verification.as_ref())
            .expect("Stage 6 image metadata should ride with verified web answer");
        assert_eq!(text, "Test Person A is the CEO of Test Company A.");
        assert_eq!(verification.presentation.image_cards.len(), 1);
        let image = &verification.presentation.image_cards[0];
        assert!(image.display_allowed);
        assert!(image.fixture_or_local_asset);
        assert!(!image.remote_image_load_allowed);
        assert_eq!(image.approved_asset_ref, "fixture-image-a.png");
        assert_eq!(verification.tts_input_text, text);
        assert!(!text.contains("fixture-image-a.png"));
        assert!(!text.contains("stage6_image"));
        assert!(!verification.tts_input_text.contains("fixture-image-a.png"));
    }

    #[test]
    fn h409_wrong_entity_ceo_source_safe_degrades_without_metadata_leak() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(409),
            ToolQueryHash(409),
            ToolResult::WebSearch {
                items: vec![
                    ToolTextSnippet {
                        title: "Our CEO and executive management team | Southern Wine Board".to_string(),
                        snippet: "Along with his wine sector experience, <strong>Dr Rowan Vale</strong> brings research management experience.".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Aurora Vale Cellars official site".to_string(),
                        snippet: "Organic and biodynamic wine producer in Australia.".to_string(),
                        url: "https://aurora-vale-cellars.test/".to_string(),
                    },
                ],
            },
            SourceMetadata {
                schema_version: SchemaVersion(1),
                provider_hint: None,
                retrieved_at_unix_ms: 1,
                sources: vec![
                    SourceRef {
                        title: "Our CEO and executive management team | Southern Wine Board — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                    SourceRef {
                        title: "Aurora Vale Cellars official site — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://aurora-vale-cellars.test/".to_string(),
                    },
                ],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(
            text,
            "I did not find a reliable public source naming a CEO."
        );
        for forbidden in [
            "Dr Rowan Vale",
            "<strong>",
            "</strong>",
            "source: UNKNOWN",
            "trust: UNVERIFIED",
            "freshness:",
            "retention:",
            "AUDIT_METADATA_ONLY",
            "citation_verified",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
    }

    #[test]
    fn h413_voice_like_synthetic_alias_ceo_source_safe_degrades_without_metadata_leak() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(413),
            ToolQueryHash(413),
            ToolResult::WebSearch {
                items: vec![
                    ToolTextSnippet {
                        title: "Our CEO and executive management team | Southern Wine Board".to_string(),
                        snippet: "Along with his wine sector experience, <strong>Dr Rowan Vale</strong> brings research management experience.".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Auroa Vale Cellars Australia".to_string(),
                        snippet: "Likely noisy public entity capture for organic wines in Australia.".to_string(),
                        url: "https://noisy-public-entity.test/auroa-vale-cellars".to_string(),
                    },
                ],
            },
            SourceMetadata {
                schema_version: SchemaVersion(1),
                provider_hint: None,
                retrieved_at_unix_ms: 1777468969264,
                sources: vec![
                    SourceRef {
                        title: "Our CEO and executive management team | Southern Wine Board — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                    SourceRef {
                        title: "Auroa Vale Cellars Australia".to_string(),
                        url: "https://noisy-public-entity.test/auroa-vale-cellars".to_string(),
                    },
                ],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(
            text,
            "I did not find a reliable public source naming a CEO."
        );
        for forbidden in [
            "Dr Rowan Vale",
            "Southern Wine Board",
            "<strong>",
            "</strong>",
            "source: UNKNOWN",
            "trust: UNVERIFIED",
            "freshness:",
            "retention:",
            "AUDIT_METADATA_ONLY",
            "citation_verified",
            "Retrieved at (unix_ms):",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
    }

    #[test]
    fn h414_synthetic_ceo_question_returns_direct_role_distinction_without_source_dump() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(414),
            ToolQueryHash(414),
            ToolResult::WebSearch {
                items: vec![
                    ToolTextSnippet {
                        title: "Aurora Vale Cellars official leadership".to_string(),
                        snippet: "Mira Solen is listed as Managing Director, Head of Grape and Wine Production.".to_string(),
                        url: "https://aurora-vale-cellars.test/".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Mira Solen - CEO at Aurora Vale Cellars".to_string(),
                        snippet: "A generic CEO ranking profile.".to_string(),
                        url: "https://leadership-rankings.test/mira-solen".to_string(),
                    },
                    ToolTextSnippet {
                        title: "Our CEO and executive management team | Southern Wine Board".to_string(),
                        snippet: "Dr Rowan Vale is the CEO of Southern Wine Board.".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                ],
            },
            SourceMetadata {
                schema_version: SchemaVersion(1),
                provider_hint: None,
                retrieved_at_unix_ms: 1777468969264,
                sources: vec![
                    SourceRef {
                        title: "Aurora Vale Cellars official leadership — source: PRIMARY_OFFICIAL; trust: HIGH_CONFIDENCE; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://aurora-vale-cellars.test/".to_string(),
                    },
                    SourceRef {
                        title: "Mira Solen - CEO at Aurora Vale Cellars — source: LOW_TRUST_SEO; trust: LOW_CONFIDENCE; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://leadership-rankings.test/mira-solen".to_string(),
                    },
                    SourceRef {
                        title: "Our CEO and executive management team | Southern Wine Board — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                        url: "https://regional-wine-board.test/about-us/our-ceo-and-executive-management-team".to_string(),
                    },
                ],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert!(text.starts_with("I did not find a reliable source naming a CEO"));
        assert!(text.contains(
            "closest source I found lists Mira Solen as Managing Director / Head of Grape and Wine Production"
        ));
        assert!(!text.contains("Confidence:"));
        assert!(!text.contains("Source:"));
        assert!(!text.contains("https://aurora-vale-cellars.test/"));
        assert!(!text.contains("I found a web result"));
        assert!(!text.contains("Sources:\n1."));
        assert!(!text.contains("Dr Rowan Vale"));
        assert!(!text.contains("leadership-rankings.test"));
        for forbidden in [
            "<strong>",
            "</strong>",
            "source: PRIMARY_OFFICIAL",
            "trust:",
            "freshness:",
            "retention:",
            "AUDIT_METADATA_ONLY",
            "citation_verified",
            "Retrieved at (unix_ms):",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
    }

    #[test]
    fn h414_weak_ceo_source_alone_safe_degrades_without_fake_ceo() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(415),
            ToolQueryHash(415),
            ToolResult::WebSearch {
                items: vec![ToolTextSnippet {
                    title: "Mira Solen - CEO at Aurora Vale Cellars".to_string(),
                    snippet: "A generic CEO ranking profile.".to_string(),
                    url: "https://leadership-rankings.test/mira-solen".to_string(),
                }],
            },
            SourceMetadata {
                schema_version: SchemaVersion(1),
                provider_hint: None,
                retrieved_at_unix_ms: 1777468969264,
                sources: vec![SourceRef {
                    title: "Mira Solen - CEO at Aurora Vale Cellars — source: LOW_TRUST_SEO; trust: LOW_CONFIDENCE; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                    url: "https://leadership-rankings.test/mira-solen".to_string(),
                }],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert_eq!(
            text,
            "I did not find a reliable public source naming a CEO."
        );
        assert!(!text.contains("Mira Solen is the CEO"));
        assert!(!text.contains("leadership-rankings"));
    }

    #[test]
    fn h409_public_tool_answer_sanitizes_html_and_internal_metadata() {
        let tool_ok = ToolResponse::ok_v1(
            ToolRequestId(410),
            ToolQueryHash(410),
            ToolResult::DeepResearch {
                summary: "Summary with <strong>clean evidence</strong>.".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "note".to_string(),
                    value: "AUDIT_METADATA_ONLY".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Public source — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                    snippet: "Snippet".to_string(),
                    url: "https://example.invalid/source".to_string(),
                }],
            },
            SourceMetadata {
                schema_version: SchemaVersion(1),
                provider_hint: None,
                retrieved_at_unix_ms: 1,
                sources: vec![SourceRef {
                    title: "Public source — source: UNKNOWN; trust: UNVERIFIED; freshness: STABLE_REFERENCE_ACCEPTABLE; citation_verified; retention:AUDIT_METADATA_ONLY".to_string(),
                    url: "https://example.invalid/source".to_string(),
                }],
                web_answer_verification: None,
            },
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let text = tool_ok_text(&tool_ok);
        assert!(text.contains("Summary: Summary with clean evidence."));
        assert!(text.contains("Public source"));
        for forbidden in [
            "<strong>",
            "</strong>",
            "source: UNKNOWN",
            "trust: UNVERIFIED",
            "freshness:",
            "retention:",
            "AUDIT_METADATA_ONLY",
            "citation_verified",
        ] {
            assert!(!text.contains(forbidden), "{forbidden} leaked in {text}");
        }
    }

    #[test]
    fn at_x_tool_ok_url_fetch_and_cite_includes_provenance_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            10,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::UrlFetchAndCiteQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::UrlFetchAndCite {
                citations: vec![ToolTextSnippet {
                    title: "Citation".to_string(),
                    snippet: "Quoted fact".to_string(),
                    url: "https://example.invalid/url-cite".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            10,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Citations:"));
                assert!(r.response_text.contains("https://example.invalid"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_document_understand_includes_structured_extraction_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            11,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DocumentUnderstandQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::DocumentUnderstand {
                summary: "Document summary".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "policy".to_string(),
                    value: "approved".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Doc citation".to_string(),
                    snippet: "Quoted text".to_string(),
                    url: "https://example.invalid/doc-cite".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            11,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Extracted fields:"));
                assert!(r.response_text.contains("Citations:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_photo_understand_includes_structured_extraction_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            12,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::PhotoUnderstandQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::PhotoUnderstand {
                summary: "Photo summary".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "visible_text".to_string(),
                    value: "Q4 revenue up 20%".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Image citation".to_string(),
                    snippet: "Visible chart label".to_string(),
                    url: "https://example.invalid/photo-cite".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            12,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Extracted fields:"));
                assert!(r.response_text.contains("Citations:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_data_analysis_includes_structured_extraction_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            14,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DataAnalysisQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::DataAnalysis {
                summary: "Data analysis summary".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "rows_analyzed".to_string(),
                    value: "128".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Data citation".to_string(),
                    snippet: "Rows 1-128".to_string(),
                    url: "https://example.invalid/data-cite".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            14,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Extracted fields:"));
                assert!(r.response_text.contains("Citations:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_deep_research_includes_structured_extraction_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            16,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::DeepResearchQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::DeepResearch {
                summary: "Deep research summary".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "scope".to_string(),
                    value: "multi-source".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Research citation".to_string(),
                    snippet: "Cross-source support".to_string(),
                    url: "https://example.invalid/research-cite".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            16,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Extracted fields:"));
                assert!(r.response_text.contains("Citations:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_record_mode_includes_action_items_and_evidence_refs() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            18,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::RecordModeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::RecordMode {
                summary: "Meeting summary".to_string(),
                action_items: vec![ToolStructuredField {
                    key: "action_item_1".to_string(),
                    value: "Send recap to team".to_string(),
                }],
                evidence_refs: vec![ToolStructuredField {
                    key: "chunk_002".to_string(),
                    value: "speaker=JD timecode=00:04:11-00:04:40".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            18,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Action items:"));
                assert!(r.response_text.contains("Recording evidence refs:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_connector_query_includes_structured_extraction_and_citations() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            21,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::ConnectorQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        let (request_id, query_hash) = match &out1.directive {
            Ph1xDirective::Dispatch(d) => match &d.dispatch_request {
                DispatchRequest::Tool(t) => (t.request_id, t.query_hash),
                DispatchRequest::SimulationCandidate(_) => panic!("expected Tool dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected Tool dispatch"),
            },
            _ => panic!("expected Dispatch"),
        };

        let tool_ok = ToolResponse::ok_v1(
            request_id,
            query_hash,
            ToolResult::ConnectorQuery {
                summary: "Connector result summary".to_string(),
                extracted_fields: vec![ToolStructuredField {
                    key: "matched_items".to_string(),
                    value: "3".to_string(),
                }],
                citations: vec![ToolTextSnippet {
                    title: "Gmail thread".to_string(),
                    snippet: "Q3 roadmap decision thread".to_string(),
                    url: "https://workspace.selene.local/gmail/thread_001".to_string(),
                }],
            },
            dummy_source_metadata(),
            None,
            ReasonCodeId(1),
            CacheStatus::Bypassed,
        )
        .unwrap();

        let second = Ph1xRequest::v1(
            21,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool_ok),
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("Summary:"));
                assert!(r.response_text.contains("Extracted fields:"));
                assert!(r.response_text.contains("Citations:"));
                assert!(!r.response_text.contains("Retrieved at (unix_ms):"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_no_silent_execution_confirm_before_impactful_intent() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::SendMoney);
        d.fields = vec![
            IntentField {
                key: FieldKey::Amount,
                value: FieldValue::verbatim("$20".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim("Alex".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];

        let req = Ph1xRequest::v1(
            1,
            2,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Confirm(_)));
        match out.thread_state.pending {
            Some(PendingState::Confirm { intent_draft, .. }) => {
                assert!(intent_draft.evidence_spans.is_empty());
            }
            _ => panic!("expected PendingState::Confirm"),
        }
    }

    #[test]
    fn at_x_memory_remember_dispatches_without_confirm() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let d = intent_draft(IntentType::MemoryRememberRequest);

        let req = Ph1xRequest::v1(
            1,
            12,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(
                        c.intent_draft.intent_type,
                        IntentType::MemoryRememberRequest
                    );
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_memory_query_dispatches_without_confirm() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let d = intent_draft(IntentType::MemoryQuery);

        let req = Ph1xRequest::v1(
            1,
            13,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::MemoryQuery);
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_memory_forget_still_requires_confirm() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let d = intent_draft(IntentType::MemoryForgetRequest);

        let req = Ph1xRequest::v1(
            1,
            14,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Confirm(_)));
        assert!(matches!(
            out.thread_state.pending,
            Some(PendingState::Confirm { .. })
        ));
    }

    #[test]
    fn at_x_confirm_yes_dispatches_step_up_for_high_stakes_intent() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::SendMoney);
        d.fields = vec![
            IntentField {
                key: FieldKey::Amount,
                value: FieldValue::verbatim("$20".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim("Alex".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];

        let first = Ph1xRequest::v1(
            1,
            10,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));
        assert!(matches!(
            out1.thread_state.pending,
            Some(PendingState::Confirm { .. })
        ));

        let second = Ph1xRequest::v1(
            1,
            11,
            now(2),
            out1.thread_state.clone(),
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

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::AccessStepUp(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::SendMoney);
                    assert!(c.intent_draft.required_fields_missing.is_empty());
                    assert_eq!(c.action_class, StepUpActionClass::Payments);
                    assert_eq!(c.challenge_method, StepUpChallengeMethod::DevicePasscode);
                }
                DispatchRequest::Tool(_) => panic!("expected AccessStepUp dispatch"),
                DispatchRequest::SimulationCandidate(_) => {
                    panic!("expected AccessStepUp dispatch")
                }
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(matches!(
            out2.thread_state.pending,
            Some(PendingState::StepUp { .. })
        ));
        assert!(out2.idempotency_key.is_some());
    }

    #[test]
    fn at_x_confirm_yes_dispatches_simulation_candidate_for_cancel_reminder() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::CancelReminder);
        d.fields = vec![IntentField {
            key: FieldKey::ReminderId,
            value: FieldValue::verbatim("rem_0000000000000001".to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }];

        let first = Ph1xRequest::v1(
            72,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));

        let second = Ph1xRequest::v1(
            72,
            2,
            now(2),
            out1.thread_state.clone(),
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
        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::CancelReminder);
                    assert!(c.intent_draft.required_fields_missing.is_empty());
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_confirm_no_aborts_and_clears_pending() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let d = intent_draft(IntentType::SendMoney);
        let first = Ph1xRequest::v1(
            2,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));

        let second = Ph1xRequest::v1(
            2,
            2,
            now(2),
            out1.thread_state.clone(),
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

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => assert!(r.response_text.contains("won’t")),
            _ => panic!("expected Respond directive"),
        }
        assert!(out2.thread_state.pending.is_none());
        assert!(out2.idempotency_key.is_some());
    }

    #[test]
    fn at_x_confirm_yes_dispatches_simulation_candidate_for_calendar_event() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let d = intent_draft(IntentType::CreateCalendarEvent);
        let first = Ph1xRequest::v1(
            9,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));
        assert!(matches!(
            out1.thread_state.pending,
            Some(PendingState::Confirm { .. })
        ));

        let second = Ph1xRequest::v1(
            9,
            2,
            now(2),
            out1.thread_state.clone(),
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

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::CreateCalendarEvent);
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => {
                    panic!("expected SimulationCandidate dispatch")
                }
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
        assert!(out2.idempotency_key.is_some());
    }

    #[test]
    fn at_x_confirm_yes_dispatches_simulation_candidate_for_bcast_wait_policy_update() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::UpdateBcastWaitPolicy);
        d.fields = vec![IntentField {
            key: FieldKey::Amount,
            value: FieldValue::normalized("2 minutes".to_string(), "120".to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }];

        let first = Ph1xRequest::v1(
            19,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));

        let second = Ph1xRequest::v1(
            19,
            2,
            now(2),
            out1.thread_state.clone(),
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
        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(
                        c.intent_draft.intent_type,
                        IntentType::UpdateBcastWaitPolicy
                    );
                    assert!(c.intent_draft.required_fields_missing.is_empty());
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_confirm_yes_dispatches_simulation_candidate_for_bcast_urgent_followup_policy_update() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::UpdateBcastUrgentFollowupPolicy);
        d.fields = vec![IntentField {
            key: FieldKey::Task,
            value: FieldValue::normalized("wait".to_string(), "wait".to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }];

        let first = Ph1xRequest::v1(
            20,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));

        let second = Ph1xRequest::v1(
            20,
            2,
            now(2),
            out1.thread_state.clone(),
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
        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(
                        c.intent_draft.intent_type,
                        IntentType::UpdateBcastUrgentFollowupPolicy
                    );
                    assert!(c.intent_draft.required_fields_missing.is_empty());
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_step_up_continue_dispatches_simulation_candidate() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let pending = ThreadState::v1(
            Some(PendingState::StepUp {
                intent_draft: confirm_snapshot_intent_draft(&intent_draft(IntentType::SendMoney)),
                requested_action: "PAYMENT_EXECUTE".to_string(),
                challenge_method: StepUpChallengeMethod::DevicePasscode,
                attempts: 1,
            }),
            None,
        );

        let req = Ph1xRequest::v1(
            55,
            2,
            now(5),
            pending,
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
                ReasonCodeId(91),
            )
            .unwrap(),
        ))
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::SendMoney);
                }
                _ => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_step_up_prefers_biometric_when_supported() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::AccessSchemaManage);
        d.fields = vec![
            IntentField {
                key: FieldKey::ApAction,
                value: FieldValue::verbatim("ACTIVATE".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessProfileId,
                value: FieldValue::verbatim("AP_TEST".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::SchemaVersionId,
                value: FieldValue::verbatim("v1".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::TenantId,
                value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessReviewChannel,
                value: FieldValue::verbatim("PHONE_DESKTOP".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessRuleAction,
                value: FieldValue::verbatim("AGREE".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];

        let first = Ph1xRequest::v1(
            56,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(out1.directive, Ph1xDirective::Confirm(_)));

        let second = Ph1xRequest::v1(
            56,
            2,
            now(2),
            out1.thread_state,
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
        .unwrap()
        .with_step_up_capabilities(Some(StepUpCapabilities::v1(true, true)))
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Dispatch(d) => match d.dispatch_request {
                DispatchRequest::AccessStepUp(c) => {
                    assert_eq!(c.challenge_method, StepUpChallengeMethod::DeviceBiometric);
                }
                _ => panic!("expected AccessStepUp dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
    }

    #[test]
    fn at_x_clarify_discipline_one_blocking_question_only() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::SendMoney);
        d.required_fields_missing = vec![FieldKey::Amount, FieldKey::When]; // Amount should win.

        let req = Ph1xRequest::v1(
            1,
            3,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::Amount]);
                assert!((2..=3).contains(&c.accepted_answer_formats.len()));
            }
            _ => panic!("expected Clarify directive"),
        }
        assert!(matches!(
            out.thread_state.pending,
            Some(PendingState::Clarify { .. })
        ));
    }

    #[test]
    fn run4_invite_link_ambiguous_tom_asks_exactly_one_disambiguation_question() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::CreateInviteLink);
        d.fields = vec![
            IntentField {
                key: FieldKey::InviteeType,
                value: FieldValue::normalized("associate".to_string(), "associate".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::DeliveryMethod,
                value: FieldValue::normalized("send".to_string(), "selene_app".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim("Tom".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];
        d.required_fields_missing = vec![FieldKey::RecipientContact];
        d.ambiguity_flags = vec![AmbiguityFlag::RecipientAmbiguous];

        let req = Ph1xRequest::v1(
            901,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.question, "Which Tom?");
                assert_eq!(c.what_is_missing, vec![FieldKey::Recipient]);
                assert!((2..=3).contains(&c.accepted_answer_formats.len()));
            }
            _ => panic!("expected clarify"),
        }
        assert_eq!(
            out.thread_state.pending,
            Some(PendingState::Clarify {
                missing_field: FieldKey::Recipient,
                attempts: 1
            })
        );
    }

    #[test]
    fn run4_invite_link_missing_contact_asks_exactly_one_delivery_contact_question() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::CreateInviteLink);
        d.fields = vec![
            IntentField {
                key: FieldKey::InviteeType,
                value: FieldValue::normalized("associate".to_string(), "associate".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::DeliveryMethod,
                value: FieldValue::normalized("send".to_string(), "selene_app".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim("Tom".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];
        d.required_fields_missing = vec![FieldKey::RecipientContact];

        let req = Ph1xRequest::v1(
            902,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.question, "What is Tom's contact for Selene App?");
                assert_eq!(c.what_is_missing, vec![FieldKey::RecipientContact]);
                assert!((2..=3).contains(&c.accepted_answer_formats.len()));
            }
            _ => panic!("expected clarify"),
        }
        assert_eq!(
            out.thread_state.pending,
            Some(PendingState::Clarify {
                missing_field: FieldKey::RecipientContact,
                attempts: 1
            })
        );
    }

    #[test]
    fn run5_invite_link_memory_contact_resolves_without_extra_clarify_when_identity_confirmed() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::CreateInviteLink);
        d.fields = vec![
            IntentField {
                key: FieldKey::InviteeType,
                value: FieldValue::normalized("associate".to_string(), "associate".to_string())
                    .unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::DeliveryMethod,
                value: FieldValue::normalized("send".to_string(), "sms".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::Recipient,
                value: FieldValue::verbatim("Tom".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];
        d.required_fields_missing = vec![FieldKey::RecipientContact];

        let req = Ph1xRequest::v1(
            903,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_ok(),
            policy_ok(),
            vec![mem_invite_contact("invite_contact_tom_sms", "+14155550100")],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert!(!matches!(out.directive, Ph1xDirective::Clarify(_)));
        let Some(PendingState::Confirm { intent_draft, .. }) = out.thread_state.pending else {
            panic!("expected confirm pending state after contact resolved from memory");
        };
        let contact = intent_draft
            .fields
            .iter()
            .find(|field| field.key == FieldKey::RecipientContact)
            .expect("recipient contact should be hydrated from memory");
        assert_eq!(contact.value.original_span, "+14155550100");
        assert!(intent_draft
            .required_fields_missing
            .iter()
            .all(|field| *field != FieldKey::RecipientContact));
    }

    #[test]
    fn at_x_access_schema_manage_missing_review_channel_asks_explicit_channel_question() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::AccessSchemaManage);
        d.required_fields_missing = vec![FieldKey::AccessReviewChannel];

        let req = Ph1xRequest::v1(
            8,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                let lower = c.question.to_ascii_lowercase();
                assert!(lower.contains("phone"));
                assert!(lower.contains("desktop"));
                assert!(lower.contains("read it out loud"));
                assert_eq!(c.what_is_missing, vec![FieldKey::AccessReviewChannel]);
            }
            _ => panic!("expected Clarify directive"),
        }
    }

    #[test]
    fn at_x_access_schema_manage_confirm_uses_professional_screen_writing() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let mut d = intent_draft(IntentType::AccessSchemaManage);
        d.fields = vec![
            IntentField {
                key: FieldKey::ApAction,
                value: FieldValue::verbatim("ACTIVATE".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessProfileId,
                value: FieldValue::verbatim("AP_CLERK".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::SchemaVersionId,
                value: FieldValue::verbatim("v3".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::ApScope,
                value: FieldValue::verbatim("TENANT".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::TenantId,
                value: FieldValue::verbatim("tenant_1".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessReviewChannel,
                value: FieldValue::verbatim("PHONE_DESKTOP".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
            IntentField {
                key: FieldKey::AccessRuleAction,
                value: FieldValue::verbatim("DISABLE".to_string()).unwrap(),
                confidence: OverallConfidence::High,
            },
        ];

        let req = Ph1xRequest::v1(
            8,
            2,
            now(2),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(d)),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Confirm(c) => {
                assert!(c.text.contains("AP_CLERK"));
                assert!(c.text.contains("PHONE_DESKTOP"));
                assert!(c.text.contains("DISABLE"));
                assert!(c.text.contains("Please confirm."));
            }
            _ => panic!("expected Confirm directive"),
        }
    }

    #[test]
    fn at_x_report_display_target_uses_explicit_then_memory_then_clarify() {
        assert_eq!(
            resolve_report_display_target(Some("desktop"), Some("phone")),
            ReportDisplayResolution::Resolved(ReportDisplayTarget::Desktop)
        );
        assert_eq!(
            resolve_report_display_target(None, Some("phone")),
            ReportDisplayResolution::Resolved(ReportDisplayTarget::Phone)
        );
        match resolve_report_display_target(None, None) {
            ReportDisplayResolution::Clarify(question) => {
                let q = question.to_ascii_lowercase();
                assert!(q.contains("desktop"));
                assert!(q.contains("phone"));
            }
            _ => panic!("expected clarify when display target is missing"),
        }
    }

    #[test]
    fn passes_through_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            9,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Clarify(
                Clarify::v1(
                    "When?".to_string(),
                    vec![FieldKey::When],
                    vec!["Tomorrow 3pm".to_string(), "Friday 10am".to_string()],
                    ReasonCodeId(1),
                    SensitivityLevel::Public,
                    false,
                    vec![],
                    vec![],
                )
                .unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Clarify(_)));
        assert!(matches!(
            out.thread_state.pending,
            Some(PendingState::Clarify { .. })
        ));
    }

    #[test]
    fn at_x_interruption_cancels_tts_and_waits() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            3,
            1,
            now(1),
            base_thread(),
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
            r.with_tts_resume_snapshot(Some(tts_snapshot(
                "First part. Remaining detail.",
                "First part.".len() as u32,
            )))
        })
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Same), Some(0.95))
        })
        .unwrap();
        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Wait(_)));
        assert_eq!(out.tts_control, Some(TtsControl::Cancel));
        assert_eq!(out.delivery_hint, DeliveryHint::Silent);
        assert!(out.thread_state.resume_buffer.is_some());
        let rb = out.thread_state.resume_buffer.unwrap();
        assert_eq!(rb.spoken_prefix, "First part.");
        assert_eq!(rb.unsaid_remainder, "Remaining detail.");
        assert!(out.thread_state.interrupted_subject_ref.is_some());
        assert!(!out.thread_state.return_check_pending);
        assert!(out.thread_state.return_check_expires_at.is_none());
    }

    #[test]
    fn at_x_interruption_without_relation_fails_closed_into_one_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            3,
            1,
            now(1),
            base_thread(),
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
            r.with_tts_resume_snapshot(Some(tts_snapshot(
                "First part. Remaining detail.",
                "First part.".len() as u32,
            )))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        let clarify = match out.directive {
            Ph1xDirective::Clarify(d) => d,
            _ => panic!("expected clarify"),
        };
        assert_eq!(out.tts_control, Some(TtsControl::Cancel));
        assert_eq!(
            out.reason_code,
            reason_codes::X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY
        );
        assert!(matches!(
            out.thread_state.pending,
            Some(PendingState::Clarify { .. })
        ));
        assert!(out.thread_state.resume_buffer.is_some());
        assert_eq!(clarify.accepted_answer_formats.len(), 3);
    }

    #[test]
    fn at_x_contract_invalid_locale_fails_closed() {
        let _rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            Some("   ".to_string()),
            None,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_x_session_suspended_fails_closed_to_wait() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Suspended,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Wait(_)));
        assert_eq!(out.delivery_hint, DeliveryHint::Silent);
    }

    #[test]
    fn at_x_privacy_mode_forces_text_only_on_respond() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            55,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_text(),
            policy_privacy(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hi".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();
        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Respond(_)));
        assert_eq!(out.delivery_hint, DeliveryHint::TextOnly);
    }

    #[test]
    fn tool_ambiguity_turns_into_one_clarify_question() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        // Create a pending tool state directly.
        let pending = ThreadState::v1(
            Some(PendingState::Tool {
                request_id: ToolRequestId(123),
                attempts: 1,
            }),
            None,
        );

        let amb = StructuredAmbiguity {
            summary: "I found multiple matches.".to_string(),
            alternatives: vec!["Option 1".to_string(), "Option 2".to_string()],
        };

        let tool = ToolResponse {
            schema_version: SchemaVersion(1),
            request_id: ToolRequestId(123),
            query_hash: ToolQueryHash(1),
            tool_status: ToolStatus::Ok,
            tool_result: Some(ToolResult::Time {
                local_time_iso: "x".to_string(),
            }),
            source_metadata: Some(dummy_source_metadata()),
            reason_code: ReasonCodeId(1),
            fail_reason_code: None,
            fail_detail: None,
            ambiguity: Some(amb),
            cache_status: CacheStatus::Bypassed,
        };

        let req = Ph1xRequest::v1(
            99,
            1,
            now(1),
            pending,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            None,
            Some(tool),
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Clarify(c) => {
                assert_eq!(c.what_is_missing, vec![FieldKey::IntentChoice]);
                assert!((2..=3).contains(&c.accepted_answer_formats.len()));
            }
            _ => panic!("expected Clarify"),
        }
    }

    #[test]
    fn at_x_resume_continue_consumes_resume_buffer() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("topic".to_string()),
            "already spoken".to_string(),
            "unsaid remainder".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::Continue,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => assert_eq!(r.response_text, "unsaid remainder"),
            _ => panic!("expected Respond"),
        }
        assert_eq!(out.reason_code, reason_codes::X_INTERRUPT_RESUME_NOW);
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeNow)
        );
        assert!(out.thread_state.resume_buffer.is_none());
    }

    #[test]
    fn at_x_resume_more_detail_consumes_resume_buffer() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            None,
            "already spoken".to_string(),
            "unsaid remainder".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::MoreDetail,
            ))),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => assert!(r.response_text.ends_with("unsaid remainder")),
            _ => panic!("expected Respond"),
        }
        assert_eq!(out.reason_code, reason_codes::X_INTERRUPT_RESUME_NOW);
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeNow)
        );
        assert!(out.thread_state.resume_buffer.is_none());
    }

    #[test]
    fn at_x_same_subject_merge_combines_resume_and_chat_into_one_response() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            77,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("I also need budget impact.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Same), Some(0.91))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        let response_text = match out.directive {
            Ph1xDirective::Respond(r) => r.response_text,
            _ => panic!("expected respond"),
        };
        assert_eq!(
            out.reason_code,
            reason_codes::X_INTERRUPT_SAME_SUBJECT_APPEND
        );
        assert_eq!(
            out.interrupt_continuity_outcome,
            Some(InterruptContinuityOutcome::SameSubjectAppend)
        );
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeNow)
        );
        assert!(response_text.contains("Remaining project milestones are due Friday."));
        assert!(response_text.contains("I also need budget impact."));
        assert!(out.thread_state.resume_buffer.is_none());
        assert!(out.thread_state.interrupted_subject_ref.is_none());
        assert!(!out.thread_state.return_check_pending);
        assert!(out.thread_state.return_check_expires_at.is_none());
    }

    #[test]
    fn at_x_switch_subject_answers_new_topic_and_keeps_return_check_buffer() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            78,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1(
                    "Shipping update: the package arrives tomorrow.".to_string(),
                    ReasonCodeId(1),
                )
                .unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Switch), Some(0.92))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        let response_text = match out.directive {
            Ph1xDirective::Respond(r) => r.response_text,
            _ => panic!("expected respond"),
        };
        assert_eq!(
            out.reason_code,
            reason_codes::X_INTERRUPT_RETURN_CHECK_ASKED
        );
        assert_eq!(
            out.interrupt_continuity_outcome,
            Some(InterruptContinuityOutcome::SwitchTopicThenReturnCheck)
        );
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeLater)
        );
        assert!(response_text.contains("Shipping update: the package arrives tomorrow."));
        assert!(response_text.contains("Do you still want to continue the previous topic?"));
        assert!(out.thread_state.resume_buffer.is_some());
        assert_eq!(
            out.thread_state.interrupted_subject_ref.as_deref(),
            Some("project_status")
        );
        assert!(out.thread_state.return_check_pending);
        assert!(out.thread_state.return_check_expires_at.is_some());
    }

    #[test]
    fn at_x_return_check_yes_applies_resume_now() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb))
            .with_interrupt_continuity_state(
                Some("project_status".to_string()),
                true,
                Some(now(11)),
            )
            .unwrap();
        let req = Ph1xRequest::v1(
            81,
            1,
            now(1),
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

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(
                    r.response_text,
                    "Remaining project milestones are due Friday."
                )
            }
            _ => panic!("expected respond"),
        }
        assert_eq!(out.reason_code, reason_codes::X_INTERRUPT_RESUME_NOW);
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeNow)
        );
        assert!(out.thread_state.resume_buffer.is_none());
        assert!(!out.thread_state.return_check_pending);
    }

    #[test]
    fn at_x_return_check_no_applies_discard_explicitly() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb))
            .with_interrupt_continuity_state(
                Some("project_status".to_string()),
                true,
                Some(now(11)),
            )
            .unwrap();
        let req = Ph1xRequest::v1(
            82,
            1,
            now(1),
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

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert!(r.response_text.contains("new topic only"))
            }
            _ => panic!("expected respond"),
        }
        assert_eq!(out.reason_code, reason_codes::X_INTERRUPT_DISCARD);
        assert_eq!(
            out.interrupt_resume_policy,
            Some(InterruptResumePolicy::Discard)
        );
        assert!(out.thread_state.resume_buffer.is_none());
        assert!(!out.thread_state.return_check_pending);
    }

    #[test]
    fn at_x_expired_resume_clears_interrupt_continuity_state_deterministically() {
        let expired_rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("previous_topic".to_string()),
            "Already spoken".to_string(),
            "Unsaid tail".to_string(),
            now(5),
        )
        .unwrap();
        let state = ThreadState::v1(None, Some(expired_rb))
            .with_interrupt_continuity_state(Some("previous_topic".to_string()), true, Some(now(6)))
            .unwrap();
        let cleared = clear_expired_resume_buffer(state, now(6));
        assert!(cleared.resume_buffer.is_none());
        assert!(cleared.interrupted_subject_ref.is_none());
        assert!(!cleared.return_check_pending);
        assert!(cleared.return_check_expires_at.is_none());
    }

    #[test]
    fn at_x_uncertain_subject_chat_fails_closed_into_one_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            79,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1(
                    "Tell me about shipping delays.".to_string(),
                    ReasonCodeId(1),
                )
                .unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Uncertain), Some(0.77))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        let clarify = match out.directive {
            Ph1xDirective::Clarify(c) => c,
            _ => panic!("expected clarify"),
        };
        assert_eq!(
            out.reason_code,
            reason_codes::X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY
        );
        assert!(matches!(
            out.thread_state.pending,
            Some(PendingState::Clarify { .. })
        ));
        assert!(out.thread_state.resume_buffer.is_some());
        assert_eq!(clarify.accepted_answer_formats.len(), 3);
    }

    #[test]
    fn at_x_uncertain_subject_intent_blocks_dispatch_with_one_clarify() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            Some("project_status".to_string()),
            "Already covered intro.".to_string(),
            "Remaining project milestones are due Friday.".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb));
        let req = Ph1xRequest::v1(
            80,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Uncertain), Some(0.81))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Clarify(_)));
        assert_eq!(
            out.reason_code,
            reason_codes::X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY
        );
        assert!(out.thread_state.resume_buffer.is_some());
    }

    #[test]
    fn at_x_replace_preserves_resume_buffer_while_handling_new_intent() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let rb = ResumeBuffer::v1(
            AnswerId(1),
            None,
            "already spoken".to_string(),
            "unsaid remainder".to_string(),
            now(10),
        )
        .unwrap();
        let thread = ThreadState::v1(None, Some(rb.clone()));
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            thread,
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::IntentDraft(intent_draft(
                IntentType::TimeQuery,
            ))),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Same), Some(0.90))
        })
        .unwrap();

        let out = rt.decide(&req).unwrap();
        assert!(
            matches!(out.directive, Ph1xDirective::Dispatch(_)),
            "expected dispatch, got {:?}",
            out.directive
        );
        assert!(out.thread_state.resume_buffer.is_some());
        assert_eq!(
            out.thread_state.resume_buffer.unwrap().unsaid_remainder,
            rb.unsaid_remainder
        );
    }

    #[test]
    fn at_x_interrupt_continuity_replay_keeps_resume_buffer_and_resume_text_lossless() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let interrupt_req = Ph1xRequest::v1(
            90,
            1,
            now(1),
            base_thread(),
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
            r.with_tts_resume_snapshot(Some(tts_snapshot(
                "Intro done. Remaining milestone details are due Friday.",
                "Intro done.".len() as u32,
            )))
        })
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Switch), Some(0.94))
        })
        .unwrap();

        let interrupt_out = rt.decide(&interrupt_req).unwrap();
        let interrupt_replay_out = rt.decide(&interrupt_req).unwrap();
        assert_eq!(interrupt_out, interrupt_replay_out);
        assert!(matches!(interrupt_out.directive, Ph1xDirective::Wait(_)));
        let captured_resume = interrupt_out
            .thread_state
            .resume_buffer
            .clone()
            .expect("interrupt turn must preserve unsaid remainder");

        let switch_req = Ph1xRequest::v1(
            91,
            2,
            now(2),
            interrupt_out.thread_state.clone(),
            SessionState::Active,
            id_text(),
            policy_ok(),
            vec![],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("New topic: shipping ETA?".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .and_then(|r| {
            r.with_interrupt_subject_relation(Some(InterruptSubjectRelation::Switch), Some(0.93))
        })
        .unwrap();

        let switch_out = rt.decide(&switch_req).unwrap();
        let switch_replay_out = rt.decide(&switch_req).unwrap();
        assert_eq!(switch_out, switch_replay_out);
        assert_eq!(
            switch_out.reason_code,
            reason_codes::X_INTERRUPT_RETURN_CHECK_ASKED
        );
        assert_eq!(
            switch_out
                .thread_state
                .resume_buffer
                .clone()
                .expect("switch turn must retain resume buffer")
                .unsaid_remainder,
            captured_resume.unsaid_remainder
        );
        assert!(switch_out.thread_state.return_check_pending);

        let resume_from_first = Ph1xRequest::v1(
            92,
            3,
            now(3),
            switch_out.thread_state.clone(),
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
        let resume_from_replay = Ph1xRequest::v1(
            92,
            3,
            now(3),
            switch_replay_out.thread_state.clone(),
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

        let resumed_out = rt.decide(&resume_from_first).unwrap();
        let resumed_replay_out = rt.decide(&resume_from_replay).unwrap();
        assert_eq!(resumed_out, resumed_replay_out);
        match resumed_out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, captured_resume.unsaid_remainder)
            }
            _ => panic!("expected resume response"),
        }
        assert_eq!(
            resumed_out.interrupt_resume_policy,
            Some(InterruptResumePolicy::ResumeNow)
        );
        assert!(resumed_out.thread_state.resume_buffer.is_none());
        assert!(!resumed_out.thread_state.return_check_pending);
    }

    #[test]
    fn at_x_08_memory_used_silently_by_default_preferred_name_applied() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_ok(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "Hello, John.");
                assert!(!r.response_text.to_ascii_lowercase().contains("remember"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_09_multi_speaker_or_unknown_blocks_personal_memory_out_loud() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_unknown(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => assert_eq!(
                r.response_text,
                "Hello. Quick check: can you confirm this is you?"
            ),
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_11_sensitive_memory_requires_permission_before_use_or_cite() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_ok(),
            policy_ok(),
            vec![mem_sensitive_value("ssn", "123-45-6789")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Okay.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                let lower = r.response_text.to_ascii_lowercase();
                assert!(lower.contains("sensitive"));
                assert!(r.response_text.contains("?"));
                assert!(!r.response_text.contains("123-45-6789"));
            }
            _ => panic!("expected Respond"),
        }
        match out.thread_state.pending {
            Some(PendingState::MemoryPermission {
                deferred_response_text,
                ..
            }) => assert_eq!(deferred_response_text, "Okay."),
            _ => panic!("expected PendingState::MemoryPermission"),
        }
    }

    #[test]
    fn at_x_sensitive_memory_permission_yes_releases_deferred_response() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            10,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_ok(),
            policy_ok(),
            vec![mem_sensitive_value("ssn", "123-45-6789")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Okay.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(
            out1.thread_state.pending,
            Some(PendingState::MemoryPermission { .. })
        ));

        let second = Ph1xRequest::v1(
            10,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_voice_ok(),
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

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => assert_eq!(r.response_text, "Okay."),
            _ => panic!("expected Respond"),
        }
        assert!(out2.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_sensitive_memory_permission_no_declines_and_clears_pending() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let first = Ph1xRequest::v1(
            11,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_ok(),
            policy_ok(),
            vec![mem_sensitive_value("ssn", "123-45-6789")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Okay.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        assert!(matches!(
            out1.thread_state.pending,
            Some(PendingState::MemoryPermission { .. })
        ));

        let second = Ph1xRequest::v1(
            11,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_voice_ok(),
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

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                let lower = r.response_text.to_ascii_lowercase();
                assert!(lower.contains("won’t") || lower.contains("won't"));
                assert!(r.response_text.contains("Okay."));
            }
            _ => panic!("expected Respond"),
        }
        assert!(out2.thread_state.pending.is_none());
    }

    #[test]
    fn at_x_12_unknown_speaker_no_personalization_even_if_memory_present() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let req = Ph1xRequest::v1(
            1,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_unknown(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => assert_eq!(
                r.response_text,
                "Hello. Quick check: can you confirm this is you?"
            ),
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_13_probable_identity_asks_once_and_blocks_personalization() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let first = Ph1xRequest::v1(
            12,
            1,
            now(1),
            base_thread(),
            SessionState::Active,
            id_voice_probable(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out1 = rt.decide(&first).unwrap();
        match out1.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "Hello. Quick check: is this jd?");
            }
            _ => panic!("expected Respond"),
        }
        assert_eq!(
            out1.thread_state
                .identity_prompt_state
                .as_ref()
                .map(|s| s.prompted_in_session),
            Some(true)
        );

        let second = Ph1xRequest::v1(
            12,
            2,
            now(2),
            out1.thread_state.clone(),
            SessionState::Active,
            id_voice_probable(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap();

        let out2 = rt.decide(&second).unwrap();
        match out2.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "Hello.");
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_14_same_scope_cooldown_blocks_identity_prompt() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let scope = "tenant_1:user_jd:device_a:voice_explicit";
        let mut thread = base_thread();
        thread.identity_prompt_state = Some(
            IdentityPromptState::v1_with_scope(false, Some(now(10)), Some(scope.to_string()), 1)
                .unwrap(),
        );
        let req = Ph1xRequest::v1(
            20,
            1,
            now(11),
            thread,
            SessionState::Active,
            id_voice_probable(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_identity_prompt_scope_key(Some(scope.to_string()))
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => assert_eq!(r.response_text, "Hello."),
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_15_different_scope_allows_identity_prompt() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let prior_scope = "tenant_1:user_jd:device_a:voice_explicit";
        let new_scope = "tenant_1:user_jd:device_b:voice_explicit";
        let mut thread = base_thread();
        thread.identity_prompt_state = Some(
            IdentityPromptState::v1_with_scope(
                false,
                Some(now(10)),
                Some(prior_scope.to_string()),
                1,
            )
            .unwrap(),
        );
        let req = Ph1xRequest::v1(
            21,
            1,
            now(11),
            thread,
            SessionState::Active,
            id_voice_probable(),
            policy_ok(),
            vec![mem_preferred_name("John")],
            None,
            Some(Ph1nResponse::Chat(
                Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
            )),
            None,
            None,
            None,
            None,
        )
        .unwrap()
        .with_identity_prompt_scope_key(Some(new_scope.to_string()))
        .unwrap();

        let out = rt.decide(&req).unwrap();
        match out.directive {
            Ph1xDirective::Respond(r) => {
                assert_eq!(r.response_text, "Hello. Quick check: is this jd?");
            }
            _ => panic!("expected Respond"),
        }
        assert_eq!(
            out.thread_state
                .identity_prompt_state
                .as_ref()
                .and_then(|s| s.prompt_scope_key.as_deref()),
            Some(new_scope)
        );
    }
}
