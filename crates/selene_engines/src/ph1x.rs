#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1e::{
    StrictBudget, StructuredAmbiguity, ToolName, ToolRequest, ToolRequestOrigin, ToolResponse,
    ToolResult, ToolStatus,
};
use selene_kernel_contracts::ph1m::{
    MemoryCandidate, MemoryConfidence, MemorySensitivityFlag, MemoryUsePolicy,
};
use selene_kernel_contracts::ph1n::{
    FieldKey, IntentDraft, IntentType, OverallConfidence, Ph1nResponse,
};
use selene_kernel_contracts::ph1tts::TtsControl;
use selene_kernel_contracts::ph1x::{
    ClarifyDirective, ConfirmDirective, DeliveryHint, DispatchDirective, IdentityContext,
    InterruptContinuityOutcome, InterruptResumePolicy, InterruptSubjectRelation, PendingState,
    Ph1xDirective, Ph1xRequest, Ph1xResponse, ResumeBuffer, ThreadState, WaitDirective,
};
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
    pub const X_INTERRUPT_RELATION_UNCERTAIN_CLARIFY: ReasonCodeId = ReasonCodeId(0x5800_0017);
    pub const X_INTERRUPT_SAME_SUBJECT_APPEND: ReasonCodeId = ReasonCodeId(0x5800_0018);
    pub const X_INTERRUPT_SWITCH_TOPIC: ReasonCodeId = ReasonCodeId(0x5800_0019);
    pub const X_INTERRUPT_RETURN_CHECK_ASKED: ReasonCodeId = ReasonCodeId(0x5800_001A);
    pub const X_INTERRUPT_RESUME_NOW: ReasonCodeId = ReasonCodeId(0x5800_001B);
    pub const X_INTERRUPT_DISCARD: ReasonCodeId = ReasonCodeId(0x5800_001C);
}

const INTERRUPT_RELATION_CONFIDENCE_MIN: f32 = 0.70;

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
                let next_state = ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field: FieldKey::ReferenceTarget,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
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
                let next_state = ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field: FieldKey::ReferenceTarget,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
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

        if let Some(rc) = req.last_failure_reason_code {
            return self.out_respond(
                req,
                clear_pending(base_thread_state),
                reason_codes::X_LAST_FAILURE,
                retry_message_for_failure(rc),
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
                let next_state = ThreadState::v1(
                    Some(PendingState::Clarify {
                        missing_field,
                        attempts,
                    }),
                    base_thread_state.resume_buffer.clone(),
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
                let allow_personalization = identity_allows_personalization(&req.identity_context);
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
                if !same_subject_merge_applied
                    && switch_subject_confident
                    && next_thread_state.resume_buffer.is_some()
                {
                    text = append_switch_topic_return_check(text);
                    switch_topic_return_check_applied = true;
                    next_thread_state = mark_return_check_pending(
                        next_thread_state,
                        req.now,
                        self.config.resume_buffer_ttl_ms,
                    );
                }

                // Sensitive memory requires permission before it is used or cited.
                // When triggered, defer the already-generated response text and ask one permission question.
                if allow_personalization
                    && contains_sensitive_candidate(&req.memory_candidates, req.now)
                {
                    let attempts =
                        bump_attempts(&base_thread_state.pending, PendingKind::MemoryPermission);
                    let next_state = ThreadState::v1(
                        Some(PendingState::MemoryPermission {
                            deferred_response_text: truncate_to_char_boundary(text, 32_768),
                            attempts,
                        }),
                        next_thread_state.resume_buffer.clone(),
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
            let next_state = ThreadState::v1(
                Some(PendingState::Clarify {
                    missing_field,
                    attempts,
                }),
                None,
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

        if d.overall_confidence != OverallConfidence::High || !d.required_fields_missing.is_empty()
        {
            let clarify = clarify_for_missing(d.intent_type, &d.required_fields_missing)?;
            let missing_field = clarify.what_is_missing[0];
            let attempts = bump_attempts(
                &base_thread_state.pending,
                PendingKind::Clarify(missing_field),
            );
            let next_state = ThreadState::v1(
                Some(PendingState::Clarify {
                    missing_field,
                    attempts,
                }),
                base_thread_state.resume_buffer.clone(),
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
            let next_state = ThreadState::v1(
                Some(PendingState::Tool {
                    request_id,
                    attempts,
                }),
                base_thread_state.resume_buffer.clone(),
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
        let next_state = ThreadState::v1(
            Some(PendingState::Confirm {
                intent_draft: confirm_snapshot_intent_draft(d),
                attempts,
            }),
            base_thread_state.resume_buffer.clone(),
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
            return self.out_tool_ambiguity(req, base_thread_state, a.clone(), delivery_base);
        }

        match tr.tool_status {
            ToolStatus::Ok => {
                let text = tool_ok_text(tr);
                self.out_respond(
                    req,
                    clear_pending(base_thread_state),
                    reason_codes::X_TOOL_OK,
                    text,
                    delivery_base,
                )
            }
            ToolStatus::Fail => self.out_respond(
                req,
                clear_pending(base_thread_state),
                reason_codes::X_TOOL_FAIL,
                "Sorry — I couldn’t complete that just now. Could you try again?".to_string(),
                delivery_base,
            ),
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
        let next_state = ThreadState::v1(
            Some(PendingState::Clarify {
                missing_field: FieldKey::IntentChoice,
                attempts,
            }),
            base_thread_state.resume_buffer.clone(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingKind {
    Clarify(FieldKey),
    Confirm(IntentType),
    MemoryPermission,
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

fn identity_allows_personalization(id: &IdentityContext) -> bool {
    match id {
        IdentityContext::TextUserId(_) => true,
        IdentityContext::Voice(v) => matches!(
            v,
            selene_kernel_contracts::ph1_voice_id::Ph1VoiceIdResponse::SpeakerAssertionOk(_)
        ),
    }
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

fn tool_ok_text(tr: &ToolResponse) -> String {
    // Deterministic shaping. Never mention providers here.
    let mut out = String::new();
    if let Some(r) = &tr.tool_result {
        match r {
            ToolResult::Time { local_time_iso } => {
                out.push_str("Local time: ");
                out.push_str(local_time_iso);
                out.push('.');
            }
            ToolResult::Weather { summary } => {
                out.push_str(summary);
            }
            ToolResult::WebSearch { items } | ToolResult::News { items } => {
                out.push_str("Here are the results:\n");
                for (i, it) in items.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::UrlFetchAndCite { citations } => {
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::DocumentUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", field.key, field.value));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::PhotoUnderstand {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", field.key, field.value));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::DataAnalysis {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", field.key, field.value));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::DeepResearch {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", field.key, field.value));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
            ToolResult::RecordMode {
                summary,
                action_items,
                evidence_refs,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Action items:\n");
                for item in action_items.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", item.key, item.value));
                }
                out.push_str("Recording evidence refs:\n");
                for evidence in evidence_refs.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", evidence.key, evidence.value));
                }
            }
            ToolResult::ConnectorQuery {
                summary,
                extracted_fields,
                citations,
            } => {
                out.push_str("Summary: ");
                out.push_str(summary);
                out.push('\n');
                out.push_str("Extracted fields:\n");
                for field in extracted_fields.iter().take(10) {
                    out.push_str(&format!("- {}: {}\n", field.key, field.value));
                }
                out.push_str("Citations:\n");
                for (i, it) in citations.iter().enumerate().take(5) {
                    out.push_str(&format!("{}. {} ({})\n", i + 1, it.title, it.url));
                }
            }
        }
    }
    if let Some(meta) = &tr.source_metadata {
        if !out.ends_with('\n') && !out.is_empty() {
            out.push('\n');
        }
        out.push_str("Sources:\n");
        for (i, s) in meta.sources.iter().enumerate().take(5) {
            out.push_str(&format!("{}. {} ({})\n", i + 1, s.title, s.url));
        }
        out.push_str(&format!(
            "Retrieved at (unix_ms): {}",
            meta.retrieved_at_unix_ms
        ));
    }
    if out.trim().is_empty() {
        // Defensive fallback; shouldn't happen due to ToolResponse validation.
        "Done.".to_string()
    } else {
        out
    }
}

fn intent_query_text(d: &IntentDraft) -> String {
    // Deterministic: prefer a task evidence excerpt if present, else use a stable fallback token.
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

fn retry_message_for_failure(_rc: ReasonCodeId) -> String {
    // Keep it honest and human; avoid leaking internals at this layer.
    "Sorry — I couldn’t complete that just now. Could you try again?".to_string()
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
                "name@example.com".to_string(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1_voice_id::{
        DiarizationSegment, IdentityConfidence, Ph1VoiceIdResponse, SpeakerAssertionOk,
        SpeakerAssertionUnknown, SpeakerId, SpeakerLabel,
    };
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{
        CacheStatus, SourceMetadata, SourceRef, ToolQueryHash, ToolRequestId, ToolStructuredField,
        ToolTextSnippet,
    };
    use selene_kernel_contracts::ph1k::{
        Confidence, DegradationClassBundle, InterruptCandidate, InterruptCandidateConfidenceBand,
        InterruptDegradationContext, InterruptGateConfidences, InterruptGates, InterruptLocaleTag,
        InterruptPhraseId, InterruptPhraseSetVersion, InterruptRiskContextClass,
        InterruptSpeechWindowMetrics, InterruptSubjectRelationConfidenceBundle,
        InterruptTimingMarkers, SpeechLikeness, PH1K_INTERRUPT_LOCALE_TAG_DEFAULT,
    };
    use selene_kernel_contracts::ph1m::{
        MemoryCandidate, MemoryConfidence, MemoryKey, MemoryProvenance, MemorySensitivityFlag,
        MemoryUsePolicy, MemoryValue,
    };
    use selene_kernel_contracts::ph1n::{
        Chat, Clarify, EvidenceSpan, FieldValue, IntentField, OverallConfidence, Ph1nResponse,
        SensitivityLevel, TranscriptHash,
    };
    use selene_kernel_contracts::ph1tts::AnswerId;
    use selene_kernel_contracts::ph1x::{
        ConfirmAnswer, DispatchRequest, IdentityContext, InterruptContinuityOutcome,
        InterruptResumePolicy, InterruptSubjectRelation, ResumeBuffer, ThreadState,
        TtsResumeSnapshot,
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
                url: "https://example.com".to_string(),
            }],
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
    fn at_x_dispatches_list_reminders_to_read_only_tool_lane() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());

        let req = Ph1xRequest::v1(
            61,
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
    fn at_x_tool_ok_web_search_includes_provenance_source_and_timestamp() {
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
                    url: "https://example.com/search-result".to_string(),
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
                assert!(r.response_text.contains("https://example.com"));
                assert!(r.response_text.contains("Retrieved at (unix_ms): 1"));
            }
            _ => panic!("expected Respond"),
        }
    }

    #[test]
    fn at_x_tool_ok_news_includes_provenance_source_and_timestamp() {
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
                    url: "https://example.com/news-result".to_string(),
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
                assert!(r.response_text.contains("https://example.com"));
                assert!(r.response_text.contains("Retrieved at (unix_ms): 1"));
            }
            _ => panic!("expected Respond"),
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
                    url: "https://example.com/url-cite".to_string(),
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
                assert!(r.response_text.contains("https://example.com"));
                assert!(r.response_text.contains("Retrieved at (unix_ms): 1"));
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
                    url: "https://example.com/doc-cite".to_string(),
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
                assert!(r.response_text.contains("Retrieved at (unix_ms): 1"));
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
                    url: "https://example.com/photo-cite".to_string(),
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
                assert!(r.response_text.contains("Retrieved at (unix_ms): 1"));
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
    fn at_x_confirm_yes_dispatches_simulation_candidate_and_clears_pending() {
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
                DispatchRequest::SimulationCandidate(c) => {
                    assert_eq!(c.intent_draft.intent_type, IntentType::SendMoney);
                    assert!(c.intent_draft.required_fields_missing.is_empty());
                }
                DispatchRequest::Tool(_) => panic!("expected SimulationCandidate dispatch"),
                DispatchRequest::AccessStepUp(_) => panic!("expected SimulationCandidate dispatch"),
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
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
            71,
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
            71,
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
    fn at_x_confirm_yes_dispatches_simulation_candidate_for_bcast_wait_policy_update() {
        let rt = Ph1xRuntime::new(Ph1xConfig::mvp_v1());
        let mut d = intent_draft(IntentType::UpdateBcastWaitPolicy);
        d.fields = vec![IntentField {
            key: FieldKey::Amount,
            value: FieldValue::normalized("2 minutes".to_string(), "120".to_string()).unwrap(),
            confidence: OverallConfidence::High,
        }];

        let first = Ph1xRequest::v1(
            73,
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
            73,
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
        assert!(matches!(out.directive, Ph1xDirective::Dispatch(_)));
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
            Ph1xDirective::Respond(r) => assert_eq!(r.response_text, "Hello."),
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
            Ph1xDirective::Respond(r) => assert_eq!(r.response_text, "Hello."),
            _ => panic!("expected Respond"),
        }
    }
}
