#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1c::{ConfidenceBucket, SelectedSlot};
use selene_kernel_contracts::ph1n::{
    Clarify, FieldKey, OverallConfidence, Ph1nRequest, Ph1nResponse, SensitivityLevel,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.NLP OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_NLP_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4E4C_01F1);
    pub const PH1_NLP_HANDOFF_INVALID: ReasonCodeId = ReasonCodeId(0x4E4C_01F2);
    pub const PH1_NLP_CLARIFY_REQUIRED: ReasonCodeId = ReasonCodeId(0x4E4C_01F3);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1nWiringConfig {
    pub ph1n_enabled: bool,
    pub require_ph1c_handoff: bool,
}

impl Ph1nWiringConfig {
    pub fn mvp_v1(ph1n_enabled: bool) -> Self {
        Self {
            ph1n_enabled,
            require_ph1c_handoff: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1nWiringOutcome {
    NotInvokedDisabled,
    Refused(Ph1nResponse),
    Forwarded(Ph1nResponse),
}

pub trait Ph1nEngine {
    fn run(&self, req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation>;
}

#[derive(Debug, Clone)]
pub struct Ph1nWiring<E>
where
    E: Ph1nEngine,
{
    config: Ph1nWiringConfig,
    engine: E,
}

impl<E> Ph1nWiring<E>
where
    E: Ph1nEngine,
{
    pub fn new(config: Ph1nWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, req: &Ph1nRequest) -> Result<Ph1nWiringOutcome, ContractViolation> {
        if req.validate().is_err() {
            return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                reason_codes::PH1_NLP_HANDOFF_INVALID,
            )?));
        }

        if !self.config.ph1n_enabled {
            return Ok(Ph1nWiringOutcome::NotInvokedDisabled);
        }

        if self.config.require_ph1c_handoff && validate_ph1c_handoff(req).is_err() {
            return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                reason_codes::PH1_NLP_HANDOFF_INVALID,
            )?));
        }
        let clarify_policy_required = requires_clarify_policy(req);

        let out = match self.engine.run(req) {
            Ok(out) => out,
            Err(_) => {
                return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                    reason_codes::PH1_NLP_INTERNAL_PIPELINE_ERROR,
                )?))
            }
        };

        if validate_response(&out).is_err() {
            return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                reason_codes::PH1_NLP_INTERNAL_PIPELINE_ERROR,
            )?));
        }
        if clarify_policy_required && !matches!(out, Ph1nResponse::Clarify(_)) {
            return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                reason_codes::PH1_NLP_CLARIFY_REQUIRED,
            )?));
        }
        if matches!(
            out,
            Ph1nResponse::IntentDraft(ref draft) if draft.overall_confidence != OverallConfidence::High
        ) {
            return Ok(Ph1nWiringOutcome::Refused(fail_closed_clarify(
                reason_codes::PH1_NLP_CLARIFY_REQUIRED,
            )?));
        }

        Ok(Ph1nWiringOutcome::Forwarded(out))
    }
}

fn validate_response(resp: &Ph1nResponse) -> Result<(), ContractViolation> {
    match resp {
        Ph1nResponse::IntentDraft(d) => d.validate(),
        Ph1nResponse::Clarify(c) => c.validate(),
        Ph1nResponse::Chat(c) => c.validate(),
    }
}

fn validate_ph1c_handoff(req: &Ph1nRequest) -> Result<(), ContractViolation> {
    let Some(meta) = &req.transcript_ok.audit_meta else {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.transcript_ok.audit_meta",
            reason: "must be present for PH1.C->PH1.NLP handoff",
        });
    };
    if meta.attempt_count == 0 {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.transcript_ok.audit_meta.attempt_count",
            reason: "must be > 0 for PH1.C->PH1.NLP handoff",
        });
    }
    if meta.selected_slot == SelectedSlot::None {
        return Err(ContractViolation::InvalidValue {
            field: "ph1n_request.transcript_ok.audit_meta.selected_slot",
            reason: "must not be NONE for PH1.C->PH1.NLP handoff",
        });
    }
    Ok(())
}

fn requires_clarify_policy(req: &Ph1nRequest) -> bool {
    req.transcript_ok.confidence_bucket != ConfidenceBucket::High
        || !req.uncertain_spans.is_empty()
        || !req.transcript_ok.uncertain_spans.is_empty()
}

fn fail_closed_clarify(
    reason_code: selene_kernel_contracts::ReasonCodeId,
) -> Result<Ph1nResponse, ContractViolation> {
    Ok(Ph1nResponse::Clarify(Clarify::v1(
        "I need one detail before I continue. What exactly should I use?".to_string(),
        vec![FieldKey::Task],
        vec![
            "One short sentence".to_string(),
            "A few keywords".to_string(),
        ],
        reason_code,
        SensitivityLevel::Public,
        false,
        vec![],
        vec![],
    )?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{
        ConfidenceBucket, LanguageTag, Ph1cAuditMeta, QualityBucket, RouteClassUsed,
        RoutingModeUsed, SelectedSlot, SessionStateRef, TranscriptOk,
    };
    use selene_kernel_contracts::ph1n::{
        Chat, Clarify, IntentDraft, IntentType, OverallConfidence, UncertainSpan,
        UncertainSpanKind, PH1N_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1w::SessionState;
    use selene_kernel_contracts::{ReasonCodeId, SchemaVersion};

    #[derive(Debug, Clone)]
    struct StubEngine {
        out: Result<Ph1nResponse, ContractViolation>,
    }

    impl Ph1nEngine for StubEngine {
        fn run(&self, _req: &Ph1nRequest) -> Result<Ph1nResponse, ContractViolation> {
            self.out.clone()
        }
    }

    fn req(transcript: &str) -> Ph1nRequest {
        let ok = TranscriptOk::v1_with_metadata(
            transcript.to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
            vec![],
            Some(
                Ph1cAuditMeta::v1(
                    RouteClassUsed::OnDevice,
                    1,
                    1,
                    SelectedSlot::Primary,
                    RoutingModeUsed::Lead,
                    false,
                    100,
                    QualityBucket::High,
                    QualityBucket::High,
                    QualityBucket::High,
                    None,
                    None,
                    Some("ph1k_handoff_standard".to_string()),
                    Some("openai_google_clarify_v1".to_string()),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        Ph1nRequest::v1(ok, SessionStateRef::v1(SessionState::Active, false)).unwrap()
    }

    fn ok_intent_response() -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::TimeQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::High,
                vec![],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    fn low_intent_response() -> Ph1nResponse {
        Ph1nResponse::IntentDraft(
            IntentDraft::v1(
                IntentType::TimeQuery,
                SchemaVersion(1),
                vec![],
                vec![],
                OverallConfidence::Low,
                vec![],
                ReasonCodeId(1),
                SensitivityLevel::Public,
                false,
                vec![],
                vec![],
            )
            .unwrap(),
        )
    }

    #[test]
    fn at_n_wiring_01_disabled_returns_not_invoked() {
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(false),
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        assert_eq!(
            w.run_turn(&req("what time is it")).unwrap(),
            Ph1nWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_n_wiring_02_forwards_valid_response() {
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&req("what time is it")).unwrap() {
            Ph1nWiringOutcome::Forwarded(Ph1nResponse::IntentDraft(d)) => {
                assert_eq!(d.intent_type, IntentType::TimeQuery)
            }
            other => panic!("expected forwarded intent draft, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_03_invalid_engine_payload_fails_closed() {
        let invalid = Ph1nResponse::Clarify(Clarify {
            schema_version: PH1N_CONTRACT_VERSION,
            question: "When exactly?".to_string(),
            what_is_missing: vec![FieldKey::When, FieldKey::Task],
            accepted_answer_formats: vec!["Tomorrow 3pm".to_string(), "Friday 10am".to_string()],
            reason_code: ReasonCodeId(1),
            sensitivity_level: SensitivityLevel::Public,
            requires_confirmation: false,
            ambiguity_flags: vec![],
            routing_hints: vec![],
        });
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine { out: Ok(invalid) },
        )
        .unwrap();
        match w.run_turn(&req("remind me tomorrow")).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_INTERNAL_PIPELINE_ERROR);
                assert_eq!(c.what_is_missing, vec![FieldKey::Task]);
            }
            other => panic!("expected refused clarify fallback, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_04_invalid_request_contract_fails_closed() {
        let mut r = req("remind me tomorrow");
        r.uncertain_spans.push(UncertainSpan {
            schema_version: PH1N_CONTRACT_VERSION,
            kind: UncertainSpanKind::Unknown,
            field_hint: Some(FieldKey::When),
            start_byte: 10,
            end_byte: 10,
        });
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&r).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_HANDOFF_INVALID)
            }
            other => panic!("expected fail-closed clarify fallback, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_05_engine_error_fails_closed() {
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Err(ContractViolation::InvalidValue {
                    field: "ph1n_runtime",
                    reason: "forced failure",
                }),
            },
        )
        .unwrap();
        match w.run_turn(&req("remind me tomorrow")).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_INTERNAL_PIPELINE_ERROR)
            }
            other => panic!("expected refused clarify fallback, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_06_valid_chat_response_is_forwarded() {
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(Ph1nResponse::Chat(
                    Chat::v1("Hello.".to_string(), ReasonCodeId(1)).unwrap(),
                )),
            },
        )
        .unwrap();
        match w.run_turn(&req("hello")).unwrap() {
            Ph1nWiringOutcome::Forwarded(Ph1nResponse::Chat(c)) => {
                assert_eq!(c.response_text, "Hello.")
            }
            other => panic!("expected forwarded chat response, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_07_missing_ph1c_handoff_meta_fails_closed_when_required() {
        let transcript = TranscriptOk::v1(
            "remind me tomorrow".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let req =
            Ph1nRequest::v1(transcript, SessionStateRef::v1(SessionState::Active, false)).unwrap();
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&req).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_HANDOFF_INVALID)
            }
            other => panic!("expected fail-closed clarify fallback, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_08_missing_handoff_meta_can_be_allowed_for_non_ph1c_paths() {
        let transcript = TranscriptOk::v1(
            "remind me tomorrow".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        let req =
            Ph1nRequest::v1(transcript, SessionStateRef::v1(SessionState::Active, false)).unwrap();
        let mut cfg = Ph1nWiringConfig::mvp_v1(true);
        cfg.require_ph1c_handoff = false;
        let w = Ph1nWiring::new(
            cfg,
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&req).unwrap() {
            Ph1nWiringOutcome::Forwarded(Ph1nResponse::IntentDraft(_)) => {}
            other => panic!("expected forwarded intent draft, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_09_low_transcript_confidence_requires_clarify() {
        let transcript = TranscriptOk::v1_with_metadata(
            "remind me tomorrow".to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::Low,
            vec![],
            Some(
                Ph1cAuditMeta::v1(
                    RouteClassUsed::OnDevice,
                    1,
                    1,
                    SelectedSlot::Primary,
                    RoutingModeUsed::Lead,
                    false,
                    100,
                    QualityBucket::High,
                    QualityBucket::High,
                    QualityBucket::High,
                    None,
                    None,
                    Some("ph1k_handoff_standard".to_string()),
                    Some("openai_google_clarify_v1".to_string()),
                )
                .unwrap(),
            ),
        )
        .unwrap();
        let req =
            Ph1nRequest::v1(transcript, SessionStateRef::v1(SessionState::Active, false)).unwrap();
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(ok_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&req).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_CLARIFY_REQUIRED)
            }
            other => panic!("expected fail-closed clarify fallback, got: {other:?}"),
        }
    }

    #[test]
    fn at_n_wiring_10_low_hypothesis_confidence_requires_clarify() {
        let w = Ph1nWiring::new(
            Ph1nWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ok(low_intent_response()),
            },
        )
        .unwrap();
        match w.run_turn(&req("what time is it")).unwrap() {
            Ph1nWiringOutcome::Refused(Ph1nResponse::Clarify(c)) => {
                assert_eq!(c.reason_code, reason_codes::PH1_NLP_CLARIFY_REQUIRED)
            }
            other => panic!("expected fail-closed clarify fallback, got: {other:?}"),
        }
    }
}
