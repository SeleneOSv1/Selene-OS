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
    PendingState, Ph1xDirective, Ph1xRequest, Ph1xResponse, ThreadState, WaitDirective,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1xConfig {
    pub tool_timeout_ms: u32,
    pub tool_max_results: u8,
}

impl Ph1xConfig {
    pub fn mvp_v1() -> Self {
        Self {
            tool_timeout_ms: 2_000,
            tool_max_results: 5,
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
            return self.out_wait(
                req,
                base_thread_state,
                reason_codes::X_INTERRUPT_CANCEL,
                Some("interrupted".to_string()),
                Some(TtsControl::Cancel),
            );
        }

        // If we are completing a prior tool dispatch, handle the ToolResponse deterministically.
        if let Some(tr) = &req.tool_response {
            return self.decide_from_tool_response(req, tr, base_thread_state, delivery_base);
        }

        // Confirmation answers are handled before NLP, using pending confirm snapshot.
        if let Some(ans) = req.confirm_answer {
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
                        base_thread_state.resume_buffer.clone(),
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

                self.out_respond(
                    req,
                    clear_pending(base_thread_state),
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

                match d.intent_type {
                    IntentType::Continue => {
                        return self.out_respond(
                            req,
                            base_thread_state,
                            reason_codes::X_RESUME_CONTINUE,
                            rb.unsaid_remainder,
                            delivery_base,
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
                        return self.out_respond(
                            req,
                            base_thread_state,
                            reason_codes::X_RESUME_MORE_DETAIL,
                            text,
                            delivery_base,
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
            IntentType::TimeQuery | IntentType::WeatherQuery
        ) {
            let (tool_name, query) = match d.intent_type {
                IntentType::TimeQuery => (ToolName::Time, intent_query_text(d)),
                IntentType::WeatherQuery => (ToolName::Weather, intent_query_text(d)),
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
        self.out(
            req,
            directive,
            thread_state,
            None,
            delivery_base,
            reason_code,
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
        let directive = Ph1xDirective::Clarify(ClarifyDirective::v1(
            question,
            accepted_answer_formats,
            what_is_missing,
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
        let delivery = match delivery_hint {
            DeliveryHint::Silent => DeliveryHint::Silent,
            _ => delivery_hint_from_base(directive_kind(&directive), delivery_hint),
        };

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
        )?;
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
        }
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
        }
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
        IntentType::TimeQuery | IntentType::WeatherQuery => "Is that right?".to_string(),
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
        FieldKey::CapreqAction,
        FieldKey::CapreqId,
        FieldKey::RequestedCapabilityId,
        FieldKey::TargetScopeRef,
        FieldKey::Justification,
        FieldKey::TenantId,
        FieldKey::Amount,
        FieldKey::Recipient,
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
        CacheStatus, SourceMetadata, SourceRef, ToolQueryHash, ToolRequestId,
    };
    use selene_kernel_contracts::ph1k::{
        Confidence, InterruptCandidate, InterruptGates, InterruptPhraseId,
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
    use selene_kernel_contracts::ph1x::{ConfirmAnswer, DispatchRequest};
    use selene_kernel_contracts::ph1x::{IdentityContext, ResumeBuffer, ThreadState};
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
            InterruptPhraseId(1),
            "wait".to_string(),
            Confidence::new(0.9).unwrap(),
            InterruptGates {
                vad_ok: true,
                echo_safe_ok: true,
                phrase_ok: true,
                nearfield_ok: true,
            },
            MonotonicTimeNs(1),
            ReasonCodeId(1),
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
                }
            }
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out.idempotency_key.is_some());
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
            },
            _ => panic!("expected Dispatch directive"),
        }
        assert!(out2.thread_state.pending.is_none());
        assert!(out2.idempotency_key.is_some());
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
        .unwrap();
        let out = rt.decide(&req).unwrap();
        assert!(matches!(out.directive, Ph1xDirective::Wait(_)));
        assert_eq!(out.tts_control, Some(TtsControl::Cancel));
        assert_eq!(out.delivery_hint, DeliveryHint::Silent);
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
