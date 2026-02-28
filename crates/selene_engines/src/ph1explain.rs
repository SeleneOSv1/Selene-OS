#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1d::PolicyContextRef;
use selene_kernel_contracts::ph1explain::{
    ExplainRequestType, ExplanationOk, ExplanationRefuse, ExplanationType, Ph1ExplainInput,
    Ph1ExplainResponse,
};
use selene_kernel_contracts::ph1n::FieldKey;
use selene_kernel_contracts::ph1x::Ph1xDirective;
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EXPLAIN reason-code namespace. Values are placeholders until the global registry is formalized.
    pub const EX_FORBIDDEN_BY_PRIVACY: ReasonCodeId = ReasonCodeId(0x5800_0001);
    pub const EX_INTERNAL: ReasonCodeId = ReasonCodeId(0x5800_0002);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ExplainConfig {
    pub allow_memory_evidence_quote: bool,
}

impl Ph1ExplainConfig {
    pub fn mvp_v1() -> Self {
        Self {
            allow_memory_evidence_quote: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1ExplainRuntime {
    config: Ph1ExplainConfig,
}

impl Ph1ExplainRuntime {
    pub fn new(config: Ph1ExplainConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, input: &Ph1ExplainInput) -> Result<Ph1ExplainResponse, ContractViolation> {
        input.validate()?;
        let request_type = input.explain_request.request_type;

        // Memory evidence explanations (HOW_KNOW) are privacy-gated.
        if request_type == ExplainRequestType::HowKnow {
            if let Some(m) = &input.memory_candidate_ref {
                return self.explain_from_memory(input.policy_context_ref, m);
            }
        }

        // Clarify / confirm explanations can be derived deterministically from the directive.
        if let Some(d) = &input.event_context_ref.conversation_directive {
            match d {
                Ph1xDirective::Clarify(c) => {
                    let field = c.what_is_missing.first().copied();
                    let txt = match request_type {
                        ExplainRequestType::WhatNext => {
                            format!("Answer this so I can continue: {}", c.question.as_str())
                        }
                        ExplainRequestType::WhyNot => match field {
                            Some(f) => format!(
                                "I couldn’t continue because I was missing the {}.",
                                field_name(f)
                            ),
                            None => "I couldn’t continue because I was missing some information."
                                .to_string(),
                        },
                        _ => match field {
                            Some(f) => {
                                format!("I asked because I was missing the {}.", field_name(f))
                            }
                            None => "I asked because I was missing some information.".to_string(),
                        },
                    };
                    return Ok(Ph1ExplainResponse::Explanation(ExplanationOk::v1(
                        txt,
                        map_request_type(request_type),
                        None,
                    )?));
                }
                Ph1xDirective::Confirm(_) => {
                    let txt = match request_type {
                        ExplainRequestType::WhatNext => {
                            "Please confirm, and then I can proceed.".to_string()
                        }
                        ExplainRequestType::Why => {
                            "I asked for confirmation before proceeding.".to_string()
                        }
                        ExplainRequestType::WhatHappened => {
                            "I paused to get your confirmation before proceeding.".to_string()
                        }
                        _ => {
                            "I didn’t proceed because I needed your confirmation first.".to_string()
                        }
                    };
                    return Ok(Ph1ExplainResponse::Explanation(ExplanationOk::v1(
                        txt,
                        map_request_type(request_type),
                        None,
                    )?));
                }
                _ => {}
            }
        }

        // "Why did you stop?" can cite the user phrase if it is provided (barge-in explanation).
        if matches!(
            request_type,
            ExplainRequestType::Why | ExplainRequestType::WhatHappened
        ) {
            if let Some(trigger) = input.event_context_ref.verbatim_trigger.as_deref() {
                let high = (input.event_context_ref.primary_reason_code.0 >> 24) as u8;
                if matches!(high, 0x4B | 0x54) {
                    let txt = format!("I stopped because you said \"{}\".", trigger);
                    return Ok(Ph1ExplainResponse::Explanation(ExplanationOk::v1(
                        txt,
                        map_request_type(request_type),
                        None,
                    )?));
                }
            }
        }

        // Fall back to reason-code class mapping.
        let (typ, txt) = explain_from_reason_code(
            request_type,
            input.event_context_ref.primary_reason_code,
            input.policy_context_ref,
        );
        Ok(Ph1ExplainResponse::Explanation(ExplanationOk::v1(
            txt, typ, None,
        )?))
    }

    fn explain_from_memory(
        &self,
        policy: PolicyContextRef,
        m: &selene_kernel_contracts::ph1explain::MemoryCandidateRef,
    ) -> Result<Ph1ExplainResponse, ContractViolation> {
        if policy.privacy_mode || m.is_sensitive {
            return Ok(Ph1ExplainResponse::ExplanationRefuse(
                ExplanationRefuse::v1(
                    reason_codes::EX_FORBIDDEN_BY_PRIVACY,
                    "I can’t explain that out loud right now.".to_string(),
                )?,
            ));
        }

        let evidence_quote = if self.config.allow_memory_evidence_quote {
            Some(m.evidence_quote.clone())
        } else {
            None
        };

        Ok(Ph1ExplainResponse::Explanation(ExplanationOk::v1(
            "I remember that because you told me earlier.".to_string(),
            ExplanationType::HowKnow,
            evidence_quote,
        )?))
    }
}

fn field_name(k: FieldKey) -> &'static str {
    match k {
        FieldKey::When => "time",
        FieldKey::Task => "task",
        FieldKey::ReminderId => "reminder ID",
        FieldKey::Person => "person",
        FieldKey::Place => "place",
        FieldKey::PartySize => "party size",
        FieldKey::Amount => "amount",
        FieldKey::Recipient => "recipient",
        FieldKey::InviteeType => "invitee type",
        FieldKey::DeliveryMethod => "delivery method",
        FieldKey::RecipientContact => "recipient contact",
        FieldKey::TenantId => "company",
        FieldKey::RequestedCapabilityId => "requested capability",
        FieldKey::TargetScopeRef => "target scope",
        FieldKey::Justification => "justification",
        FieldKey::CapreqAction => "capability-request action",
        FieldKey::CapreqId => "capability request ID",
        FieldKey::AccessProfileId => "access profile",
        FieldKey::SchemaVersionId => "schema version",
        FieldKey::ApScope => "access profile scope",
        FieldKey::ApAction => "access profile action",
        FieldKey::AccessReviewChannel => "review channel",
        FieldKey::AccessRuleAction => "rule review action",
        FieldKey::ProfilePayloadJson => "access profile rules payload",
        FieldKey::EscalationCaseId => "escalation case",
        FieldKey::BoardPolicyId => "board policy",
        FieldKey::TargetUserId => "target user",
        FieldKey::AccessInstanceId => "access instance",
        FieldKey::VoteAction => "vote action",
        FieldKey::VoteValue => "vote value",
        FieldKey::OverrideResult => "override result",
        FieldKey::PositionId => "position",
        FieldKey::OverlayIdList => "overlay list",
        FieldKey::CompileReason => "compile reason",
        FieldKey::IntentChoice => "which request",
        FieldKey::ReferenceTarget => "what you meant",
    }
}

fn explain_from_reason_code(
    request_type: ExplainRequestType,
    rc: ReasonCodeId,
    policy: PolicyContextRef,
) -> (ExplanationType, String) {
    let typ = map_request_type(request_type);

    let high = (rc.0 >> 24) as u8;
    let txt = match request_type {
        ExplainRequestType::WhatNext => {
            if policy.privacy_mode || policy.do_not_disturb {
                "You can continue silently as text, or turn off privacy/DND if you want me to speak."
                    .to_string()
            } else {
                match high {
                    0x43 => "Please say that again.".to_string(),
                    0x45 => "Want me to try that lookup again?".to_string(),
                    0x44 => "Try rephrasing, or give one missing detail.".to_string(),
                    0x57 => "Say \"Selene\" again when you're ready.".to_string(),
                    0x4C => "Say \"Selene\" if you want to keep going.".to_string(),
                    0x4B => "Tell me what you want to do next.".to_string(),
                    0x54 => "If you want me to continue, say \"continue\".".to_string(),
                    0x56 => "Please speak again so I can tell who's speaking.".to_string(),
                    0x58 => "Ask what you want me to explain, and I’ll keep it short.".to_string(),
                    _ => "Try again, or tell me what you want next.".to_string(),
                }
            }
        }
        ExplainRequestType::WhyNot => match high {
            0x43 => "I didn’t proceed because I didn’t catch that clearly enough.".to_string(),
            0x45 => "I didn’t proceed because that lookup didn’t complete.".to_string(),
            0x44 => "I didn’t proceed because I couldn’t produce a safe result.".to_string(),
            0x57 => "I didn’t proceed because I didn’t detect a clear wake.".to_string(),
            0x4C => "I didn’t proceed because the session had already closed.".to_string(),
            0x54 => "I didn’t proceed because I couldn’t speak out loud right now.".to_string(),
            0x56 => "I didn't proceed because I couldn't tell who was speaking.".to_string(),
            0x58 => "I didn’t proceed because I can’t explain that out loud right now.".to_string(),
            _ => "I didn’t proceed because something prevented me from continuing.".to_string(),
        },
        _ => match high {
            // 0x43: STT failures (PH1.C)
            0x43 => "I didn’t catch that clearly enough to trust it; could you say it again?"
                .to_string(),
            // 0x45: tool failures (PH1.E)
            0x45 => "That lookup didn’t complete successfully; want me to try again?".to_string(),
            // 0x44: probabilistic boundary failures (PH1.D)
            0x44 => "I couldn’t produce a safe answer for that right now.".to_string(),
            // 0x57: wake gate failures (PH1.W)
            0x57 => "I didn’t detect a clear wake just then.".to_string(),
            // 0x4C: lifecycle/presence (PH1.L)
            0x4C => "I stayed quiet because I thought the conversation had finished.".to_string(),
            // 0x4B: voice substrate / interruption (PH1.K)
            0x4B => "I paused because I thought you interrupted me.".to_string(),
            // 0x54: TTS playback issues (PH1.TTS)
            0x54 => "I couldn’t finish speaking just now.".to_string(),
            // 0x56: voice identity (PH1.VOICE.ID)
            0x56 => "I couldn’t confidently tell who was speaking.".to_string(),
            // 0x58: explanation refusal/internal.
            0x58 => "I can’t explain that out loud right now.".to_string(),
            _ => "Something prevented me from continuing; want to try again?".to_string(),
        },
    };

    (typ, txt)
}

fn map_request_type(t: ExplainRequestType) -> ExplanationType {
    match t {
        ExplainRequestType::Why => ExplanationType::Why,
        ExplainRequestType::WhyNot => ExplanationType::WhyNot,
        ExplainRequestType::HowKnow => ExplanationType::HowKnow,
        ExplainRequestType::WhatNext => ExplanationType::WhatNext,
        ExplainRequestType::WhatHappened => ExplanationType::WhatHappened,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1d::{PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1explain::{
        EventContextRef, ExplainRequest, MemoryCandidateRef,
    };
    use selene_kernel_contracts::ph1n::FieldKey;
    use selene_kernel_contracts::ph1x::ClarifyDirective;

    fn policy_ok() -> PolicyContextRef {
        PolicyContextRef::v1(false, false, SafetyTier::Standard)
    }

    #[test]
    fn at_ex_01_why_did_you_ask_cites_missing_field() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());
        let clarify = Ph1xDirective::Clarify(
            ClarifyDirective::v1(
                "When?".to_string(),
                vec!["Tomorrow 3pm".to_string(), "Friday 10am".to_string()],
                vec![FieldKey::When],
            )
            .unwrap(),
        );

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(ExplainRequestType::Why, Some("why?".to_string())).unwrap(),
            EventContextRef::v1(ReasonCodeId(1), vec![], Some(clarify), None).unwrap(),
            None,
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                assert_eq!(ok.explanation_type, ExplanationType::Why);
                assert!(ok.explanation_text.to_lowercase().contains("time"));
            }
            _ => panic!("expected explanation"),
        }
    }

    #[test]
    fn at_ex_02_why_not_cites_confirmation_gate() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());
        let confirm = Ph1xDirective::Confirm(
            selene_kernel_contracts::ph1x::ConfirmDirective::v1("Confirm?".to_string()).unwrap(),
        );

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(ExplainRequestType::WhyNot, Some("why not?".to_string())).unwrap(),
            EventContextRef::v1(ReasonCodeId(1), vec![], Some(confirm), None).unwrap(),
            None,
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                assert_eq!(ok.explanation_type, ExplanationType::WhyNot);
                assert!(ok.explanation_text.to_lowercase().contains("confirmation"));
            }
            _ => panic!("expected explanation"),
        }
    }

    #[test]
    fn at_ex_03_how_do_you_know_is_evidence_backed_or_refuses() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());
        let mem = MemoryCandidateRef::v1(
            "You said: \"My birthday is May 5\"".to_string(),
            None,
            false,
        )
        .unwrap();

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(
                ExplainRequestType::HowKnow,
                Some("how do you know?".to_string()),
            )
            .unwrap(),
            EventContextRef::v1(ReasonCodeId(1), vec![], None, None).unwrap(),
            Some(mem),
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                assert_eq!(ok.explanation_type, ExplanationType::HowKnow);
                assert!(ok.evidence_quote.is_some());
            }
            _ => panic!("expected explanation"),
        }
    }

    #[test]
    fn privacy_mode_refuses_memory_explanation() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());
        let mem = MemoryCandidateRef::v1("Sensitive".to_string(), None, true).unwrap();
        let policy_priv = PolicyContextRef::v1(true, false, SafetyTier::Standard);

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(
                ExplainRequestType::HowKnow,
                Some("how do you know?".to_string()),
            )
            .unwrap(),
            EventContextRef::v1(ReasonCodeId(1), vec![], None, None).unwrap(),
            Some(mem),
            policy_priv,
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        assert!(matches!(
            out,
            Ph1ExplainResponse::ExplanationRefuse(r) if r.reason_code == reason_codes::EX_FORBIDDEN_BY_PRIVACY
        ));
    }

    #[test]
    fn at_ex_05_no_internal_leakage_on_tool_failure() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());

        // 0x45xx_xxxx is PH1.E namespace in current skeletons.
        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(
                ExplainRequestType::WhatHappened,
                Some("what happened?".to_string()),
            )
            .unwrap(),
            EventContextRef::v1(ReasonCodeId(0x4500_0003), vec![], None, None).unwrap(),
            None,
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                let lower = ok.explanation_text.to_lowercase();
                assert!(!lower.contains("provider"));
                assert!(!lower.contains("threshold"));
                assert!(!lower.contains("0x"));
            }
            _ => panic!("expected explanation"),
        }
    }

    #[test]
    fn at_ex_04_why_did_you_stop_cites_interrupt_phrase_when_available() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(
                ExplainRequestType::WhatHappened,
                Some("why did you stop?".to_string()),
            )
            .unwrap(),
            EventContextRef::v1(
                // 0x4Bxx_xxxx is PH1.K namespace in current skeletons.
                ReasonCodeId(0x4B00_0001),
                vec![],
                None,
                Some("wait".to_string()),
            )
            .unwrap(),
            None,
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                assert_eq!(ok.explanation_type, ExplanationType::WhatHappened);
                assert!(ok.explanation_text.to_ascii_lowercase().contains("wait"));
            }
            _ => panic!("expected explanation"),
        }
    }

    #[test]
    fn at_ex_06_one_sentence_discipline() {
        let rt = Ph1ExplainRuntime::new(Ph1ExplainConfig::mvp_v1());

        let input = Ph1ExplainInput::v1(
            ExplainRequest::v1(ExplainRequestType::WhatNext, Some("what next?".to_string()))
                .unwrap(),
            EventContextRef::v1(
                // 0x45xx_xxxx is PH1.E namespace in current skeletons.
                ReasonCodeId(0x4500_0003),
                vec![],
                None,
                None,
            )
            .unwrap(),
            None,
            policy_ok(),
        )
        .unwrap();

        let out = rt.run(&input).unwrap();
        match out {
            Ph1ExplainResponse::Explanation(ok) => {
                let n = ok
                    .explanation_text
                    .chars()
                    .filter(|c| matches!(c, '.' | '!' | '?'))
                    .count();
                assert!(n <= 2);
                assert!(ok.explanation_text.to_ascii_lowercase().contains("try"));
            }
            _ => panic!("expected explanation"),
        }
    }
}
