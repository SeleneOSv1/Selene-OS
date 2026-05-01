#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1_voice_id::{IdentityTierV2, Ph1VoiceIdResponse, VoiceIdentityV2};
use selene_kernel_contracts::ph1e::{
    StrictBudget, StructuredAmbiguity, ToolName, ToolRequest, ToolRequestOrigin, ToolResponse,
    ToolResult, ToolStatus,
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
    ClarifyDirective, ConfirmDirective, DeliveryHint, DispatchDirective, IdentityContext,
    IdentityPromptState, InterruptContinuityOutcome, InterruptResumePolicy,
    InterruptSubjectRelation, PendingState, Ph1xDirective, Ph1xRequest, Ph1xResponse, ResumeBuffer,
    StepUpActionClass, StepUpCapabilities, StepUpChallengeMethod, StepUpOutcome, StepUpResult,
    ThreadState, WaitDirective,
};
use selene_kernel_contracts::provider_secrets::ProviderSecretId;
use selene_kernel_contracts::{
    ContractViolation, MonotonicTimeNs, ReasonCodeId, SessionState, Validate,
};

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
            "Awaiting a place, city, country, region, or IANA timezone for the pending deterministic time query.".to_string(),
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
                "An IANA timezone, e.g. America/New_York".to_string(),
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
        return verification.response_text.clone();
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
        "I could not verify this from accepted sources.".to_string()
    } else {
        "I found search candidates, but I could not verify the requested claim from accepted sources."
            .to_string()
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
            "I couldn't verify a publicly listed CEO from reliable public sources. I did find {person} listed as {role}, but that is not the same as a verified CEO listing."
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
        return Some(
            "I couldn't verify a publicly listed CEO from reliable public sources.".to_string(),
        );
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
                .map(time_clarification_options)
                .filter(|options| !options.is_empty())
            {
                return format!(
                    "That place has more than one timezone. Do you mean {}?",
                    alternatives
                );
            }
            return "That location has more than one timezone. Please ask with a specific city or IANA timezone.".to_string();
        }
        if detail.contains("missing_time_location") {
            return "Which place do you mean?".to_string();
        }
        if detail.contains("unsupported_time_location") {
            return "I can't resolve that location yet. Please ask with a supported city, country, or IANA timezone.".to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionOk,
        SpeakerAssertionUnknown, SpeakerId, SpeakerLabel, DEFAULT_CONF_MID_BP,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{
        AcceptedSourcePacket, CacheStatus, RejectedSourcePacket, RequestedEntityPacket,
        SourceChipPacket, SourceEvaluationPacket, SourceMetadata, SourceRef, ToolQueryHash,
        ToolRequestId, ToolStructuredField, ToolTextSnippet, WebAnswerVerificationPacket,
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
        InterruptContinuityOutcome, InterruptResumePolicy, InterruptSubjectRelation, ResumeBuffer,
        ThreadState, TtsResumeSnapshot,
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
                source_chips: vec![SourceChipPacket {
                    source_id: "source_001".to_string(),
                    label: "Aurora Vale Cellars official leadership".to_string(),
                    domain: "aurora-vale-cellars.test".to_string(),
                    safe_click_url: "https://aurora-vale-cellars.test/leadership".to_string(),
                    source_type: "WEB_RESULT".to_string(),
                    accepted: true,
                    claim_refs: vec!["claim_001".to_string()],
                }],
                answer_claims: vec![response_text.clone()],
                claim_to_source_map: vec![("claim_001".to_string(), "source_001".to_string())],
                final_answer_class: "VERIFIED_DIRECT_ANSWER".to_string(),
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
                    .contains("I found search candidates, but I could not verify"));
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
                    .contains("I found search candidates, but I could not verify"));
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
            "I couldn't verify a publicly listed CEO from reliable public sources."
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
            "I couldn't verify a publicly listed CEO from reliable public sources."
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
        assert!(text.starts_with("I couldn't verify a publicly listed CEO"));
        assert!(text.contains(
            "Mira Solen listed as Managing Director / Head of Grape and Wine Production"
        ));
        assert!(text.contains("not the same as a verified CEO listing"));
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
            "I couldn't verify a publicly listed CEO from reliable public sources."
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
