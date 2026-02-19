#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1c::{Ph1cRequest, Ph1cResponse, RetryAdvice, TranscriptReject};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.C OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_C_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4343_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1cWiringConfig {
    pub ph1c_enabled: bool,
}

impl Ph1cWiringConfig {
    pub fn mvp_v1(ph1c_enabled: bool) -> Self {
        Self { ph1c_enabled }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Ph1cWiringOutcome {
    NotInvokedDisabled,
    Refused(TranscriptReject),
    Forwarded(Ph1cResponse),
}

pub trait Ph1cEngine {
    fn run(&self, req: &Ph1cRequest) -> Ph1cResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1cWiring<E>
where
    E: Ph1cEngine,
{
    config: Ph1cWiringConfig,
    engine: E,
}

impl<E> Ph1cWiring<E>
where
    E: Ph1cEngine,
{
    pub fn new(config: Ph1cWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, req: &Ph1cRequest) -> Result<Ph1cWiringOutcome, ContractViolation> {
        req.validate()?;

        if !self.config.ph1c_enabled {
            return Ok(Ph1cWiringOutcome::NotInvokedDisabled);
        }

        let out = self.engine.run(req);
        if validate_response(&out).is_err() {
            return Ok(Ph1cWiringOutcome::Refused(fail_closed_reject()?));
        }

        Ok(Ph1cWiringOutcome::Forwarded(out))
    }
}

fn validate_response(resp: &Ph1cResponse) -> Result<(), ContractViolation> {
    match resp {
        Ph1cResponse::TranscriptOk(ok) => ok.validate(),
        Ph1cResponse::TranscriptReject(r) => r.validate(),
    }
}

fn fail_closed_reject() -> Result<TranscriptReject, ContractViolation> {
    let reject = TranscriptReject::v1(
        reason_codes::PH1_C_INTERNAL_PIPELINE_ERROR,
        RetryAdvice::Repeat,
    );
    reject.validate()?;
    Ok(reject)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1c::{
        ConfidenceBucket, LanguageTag, SessionStateRef, TranscriptOk, PH1C_CONTRACT_VERSION,
    };
    use selene_kernel_contracts::ph1k::{
        AudioDeviceId, AudioStreamId, DeviceHealth, DeviceState, PreRollBufferId,
    };
    use selene_kernel_contracts::ph1w::{BoundedAudioSegmentRef, SessionState};
    use selene_kernel_contracts::{MonotonicTimeNs, ReasonCodeId};

    #[derive(Debug, Clone)]
    struct StubEngine {
        out: Ph1cResponse,
    }

    impl Ph1cEngine for StubEngine {
        fn run(&self, _req: &Ph1cRequest) -> Ph1cResponse {
            self.out.clone()
        }
    }

    fn dev(id: &str) -> AudioDeviceId {
        AudioDeviceId::new(id).unwrap()
    }

    fn req() -> Ph1cRequest {
        let seg = BoundedAudioSegmentRef::v1(
            AudioStreamId(1),
            PreRollBufferId(1),
            MonotonicTimeNs(10),
            MonotonicTimeNs(30),
            MonotonicTimeNs(12),
            MonotonicTimeNs(15),
        )
        .unwrap();
        Ph1cRequest::v1(
            seg,
            SessionStateRef::v1(SessionState::Active, false),
            DeviceState::v1(dev("mic"), dev("spk"), DeviceHealth::Healthy, vec![]),
            None,
            None,
            None,
        )
        .unwrap()
    }

    fn ok_response() -> Ph1cResponse {
        Ph1cResponse::TranscriptOk(
            TranscriptOk::v1(
                "set a reminder".to_string(),
                LanguageTag::new("en").unwrap(),
                ConfidenceBucket::High,
            )
            .unwrap(),
        )
    }

    #[test]
    fn at_c_wiring_01_disabled_returns_not_invoked() {
        let w = Ph1cWiring::new(
            Ph1cWiringConfig::mvp_v1(false),
            StubEngine { out: ok_response() },
        )
        .unwrap();
        assert_eq!(
            w.run_turn(&req()).unwrap(),
            Ph1cWiringOutcome::NotInvokedDisabled
        );
    }

    #[test]
    fn at_c_wiring_02_forwards_valid_response() {
        let w = Ph1cWiring::new(
            Ph1cWiringConfig::mvp_v1(true),
            StubEngine { out: ok_response() },
        )
        .unwrap();
        match w.run_turn(&req()).unwrap() {
            Ph1cWiringOutcome::Forwarded(Ph1cResponse::TranscriptOk(ok)) => {
                assert_eq!(ok.transcript_text, "set a reminder")
            }
            other => panic!("expected forwarded transcript_ok, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_wiring_03_invalid_engine_payload_fails_closed() {
        let invalid = Ph1cResponse::TranscriptOk(TranscriptOk {
            schema_version: PH1C_CONTRACT_VERSION,
            transcript_text: "".to_string(),
            language_tag: LanguageTag::new("en").unwrap(),
            confidence_bucket: ConfidenceBucket::High,
            uncertain_spans: vec![],
            audit_meta: None,
        });
        let w =
            Ph1cWiring::new(Ph1cWiringConfig::mvp_v1(true), StubEngine { out: invalid }).unwrap();
        match w.run_turn(&req()).unwrap() {
            Ph1cWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_C_INTERNAL_PIPELINE_ERROR);
                assert_eq!(r.retry_advice, RetryAdvice::Repeat);
            }
            other => panic!("expected refused output, got: {other:?}"),
        }
    }

    #[test]
    fn at_c_wiring_04_invalid_request_contract_is_rejected() {
        let mut r = req();
        r.bounded_audio_segment_ref.t_end = MonotonicTimeNs(5);
        let w = Ph1cWiring::new(
            Ph1cWiringConfig::mvp_v1(true),
            StubEngine { out: ok_response() },
        )
        .unwrap();
        assert!(w.run_turn(&r).is_err());
    }

    #[test]
    fn at_c_wiring_05_valid_reject_response_is_forwarded() {
        let w = Ph1cWiring::new(
            Ph1cWiringConfig::mvp_v1(true),
            StubEngine {
                out: Ph1cResponse::TranscriptReject(TranscriptReject::v1(
                    ReasonCodeId(1),
                    RetryAdvice::SpeakSlower,
                )),
            },
        )
        .unwrap();
        match w.run_turn(&req()).unwrap() {
            Ph1cWiringOutcome::Forwarded(Ph1cResponse::TranscriptReject(r)) => {
                assert_eq!(r.retry_advice, RetryAdvice::SpeakSlower)
            }
            other => panic!("expected forwarded transcript_reject, got: {other:?}"),
        }
    }
}
