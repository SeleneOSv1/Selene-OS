#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1d::{
    Ph1dFail, Ph1dFailureKind, Ph1dOk, Ph1dRequest, Ph1dResponse, PH1D_CONTRACT_VERSION,
};
use selene_kernel_contracts::{ContractViolation, ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.D OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_D_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4444_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1dWiringConfig {
    pub ph1d_enabled: bool,
}

impl Ph1dWiringConfig {
    pub fn mvp_v1(ph1d_enabled: bool) -> Self {
        Self { ph1d_enabled }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1dWiringOutcome {
    NotInvokedDisabled,
    Refused(Ph1dFail),
    Forwarded(Ph1dResponse),
}

pub trait Ph1dEngine {
    fn run(&self, req: &Ph1dRequest) -> Ph1dResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1dWiring<E>
where
    E: Ph1dEngine,
{
    config: Ph1dWiringConfig,
    engine: E,
}

impl<E> Ph1dWiring<E>
where
    E: Ph1dEngine,
{
    pub fn new(config: Ph1dWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, req: &Ph1dRequest) -> Result<Ph1dWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.ph1d_enabled {
            return Ok(Ph1dWiringOutcome::NotInvokedDisabled);
        }

        let out = self.engine.run(req);
        if validate_response(&out).is_err() {
            return Ok(Ph1dWiringOutcome::Refused(Ph1dFail::v1(
                reason_codes::PH1_D_INTERNAL_PIPELINE_ERROR,
                Ph1dFailureKind::InvalidSchema,
            )));
        }
        Ok(Ph1dWiringOutcome::Forwarded(out))
    }
}

fn validate_response(resp: &Ph1dResponse) -> Result<(), ContractViolation> {
    match resp {
        Ph1dResponse::Ok(ok) => match ok {
            Ph1dOk::Chat(c) => c.validate(),
            Ph1dOk::Intent(i) => i.validate(),
            Ph1dOk::Clarify(c) => c.validate(),
            Ph1dOk::Analysis(a) => a.validate(),
        },
        Ph1dResponse::Fail(f) => validate_fail(f),
    }
}

fn validate_fail(f: &Ph1dFail) -> Result<(), ContractViolation> {
    if f.schema_version != PH1D_CONTRACT_VERSION {
        return Err(ContractViolation::InvalidValue {
            field: "ph1d_fail.schema_version",
            reason: "must match PH1D_CONTRACT_VERSION",
        });
    }
    if f.reason_code == ReasonCodeId(0) {
        return Err(ContractViolation::InvalidValue {
            field: "ph1d_fail.reason_code",
            reason: "must be non-zero",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{ConfidenceBucket, LanguageTag, SessionStateRef};
    use selene_kernel_contracts::ph1d::{Ph1dChat, Ph1dClarify, PolicyContextRef, SafetyTier};
    use selene_kernel_contracts::ph1e::{ToolCatalogRef, ToolName};
    use selene_kernel_contracts::ph1n::{Chat, FieldKey, Ph1nResponse, TranscriptHash};
    use selene_kernel_contracts::ph1w::SessionState;

    #[derive(Debug, Clone)]
    struct StubEngine {
        out: Ph1dResponse,
    }

    impl Ph1dEngine for StubEngine {
        fn run(&self, _req: &Ph1dRequest) -> Ph1dResponse {
            self.out.clone()
        }
    }

    fn req(transcript: &str) -> Ph1dRequest {
        let ok = selene_kernel_contracts::ph1c::TranscriptOk::v1(
            transcript.to_string(),
            LanguageTag::new("en").unwrap(),
            ConfidenceBucket::High,
        )
        .unwrap();
        Ph1dRequest::v1(
            ok,
            Ph1nResponse::Chat(Chat::v1("hi".to_string(), ReasonCodeId(1)).unwrap()),
            SessionStateRef::v1(SessionState::Active, false),
            PolicyContextRef::v1(false, false, SafetyTier::Standard),
            ToolCatalogRef::v1(vec![ToolName::Time]).unwrap(),
        )
        .unwrap()
    }

    fn ok_chat_response() -> Ph1dResponse {
        Ph1dResponse::Ok(Ph1dOk::Chat(
            Ph1dChat::v1("hello".to_string(), ReasonCodeId(1)).unwrap(),
        ))
    }

    #[test]
    fn at_d_wiring_01_disabled_returns_not_invoked() {
        let w = Ph1dWiring::new(
            Ph1dWiringConfig::mvp_v1(false),
            StubEngine {
                out: ok_chat_response(),
            },
        )
        .unwrap();
        assert_eq!(
            w.run_turn(&req("hello")).unwrap(),
            Ph1dWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_d_wiring_02_forwards_valid_response() {
        let w = Ph1dWiring::new(
            Ph1dWiringConfig::mvp_v1(true),
            StubEngine {
                out: ok_chat_response(),
            },
        )
        .unwrap();
        match w.run_turn(&req("hello")).unwrap() {
            Ph1dWiringOutcome::Forwarded(Ph1dResponse::Ok(Ph1dOk::Chat(c))) => {
                assert_eq!(c.response_text, "hello")
            }
            other => panic!("expected forwarded chat response, got: {other:?}"),
        }
    }

    #[test]
    fn at_d_wiring_03_invalid_engine_payload_fails_closed() {
        let invalid = Ph1dResponse::Ok(Ph1dOk::Clarify(Ph1dClarify {
            schema_version: PH1D_CONTRACT_VERSION,
            question: "When?".to_string(),
            what_is_missing: vec![FieldKey::When, FieldKey::Task],
            accepted_answer_formats: vec!["Tomorrow".to_string(), "Friday".to_string()],
            reason_code: ReasonCodeId(1),
        }));
        let w = Ph1dWiring::new(
            Ph1dWiringConfig::mvp_v1(true),
            StubEngine { out: invalid },
        )
        .unwrap();
        match w.run_turn(&req("hello")).unwrap() {
            Ph1dWiringOutcome::Refused(f) => {
                assert_eq!(
                    f.reason_code,
                    reason_codes::PH1_D_INTERNAL_PIPELINE_ERROR
                );
                assert_eq!(f.kind, Ph1dFailureKind::InvalidSchema);
            }
            other => panic!("expected refused output, got: {other:?}"),
        }
    }

    #[test]
    fn at_d_wiring_04_invalid_request_envelope_is_rejected() {
        let mut r = req("hello");
        r.transcript_hash = TranscriptHash(r.transcript_hash.0.wrapping_add(1));
        let w = Ph1dWiring::new(
            Ph1dWiringConfig::mvp_v1(true),
            StubEngine {
                out: ok_chat_response(),
            },
        )
        .unwrap();
        assert!(w.run_turn(&r).is_err());
    }

    #[test]
    fn at_d_wiring_05_valid_fail_response_is_forwarded() {
        let w = Ph1dWiring::new(
            Ph1dWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ph1dResponse::Fail(Ph1dFail::v1(
                    ReasonCodeId(1),
                    Ph1dFailureKind::Timeout,
                )),
            },
        )
        .unwrap();
        match w.run_turn(&req("hello")).unwrap() {
            Ph1dWiringOutcome::Forwarded(Ph1dResponse::Fail(f)) => {
                assert_eq!(f.kind, Ph1dFailureKind::Timeout)
            }
            other => panic!("expected forwarded fail response, got: {other:?}"),
        }
    }
}
