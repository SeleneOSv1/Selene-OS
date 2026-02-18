#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1srl::{
    Ph1SrlRequest, Ph1SrlResponse, SrlArgumentNormalizeOk, SrlArgumentNormalizeRequest,
    SrlCapabilityId, SrlFrameBuildOk, SrlFrameBuildRequest, SrlRefuse, SrlRequestEnvelope,
    SrlUncertainSpan, SrlValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.SRL OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_SRL_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5352_0101);
    pub const PH1_SRL_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5352_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1SrlWiringConfig {
    pub srl_enabled: bool,
    pub max_spans: u8,
    pub max_notes: u8,
    pub max_ambiguities: u8,
    pub max_diagnostics: u8,
}

impl Ph1SrlWiringConfig {
    pub fn mvp_v1(srl_enabled: bool) -> Self {
        Self {
            srl_enabled,
            max_spans: 32,
            max_notes: 16,
            max_ambiguities: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub transcript_hash: String,
    pub transcript_text: String,
    pub language_tag: String,
    pub uncertain_spans: Vec<SrlUncertainSpan>,
    pub know_dictionary_hints: Vec<String>,
}

impl SrlTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        transcript_hash: String,
        transcript_text: String,
        language_tag: String,
        uncertain_spans: Vec<SrlUncertainSpan>,
        know_dictionary_hints: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            transcript_hash,
            transcript_text,
            language_tag,
            uncertain_spans,
            know_dictionary_hints,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for SrlTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_hash("srl_turn_input.transcript_hash", &self.transcript_hash)?;
        validate_text(
            "srl_turn_input.transcript_text",
            &self.transcript_text,
            4096,
        )?;
        validate_language_tag("srl_turn_input.language_tag", &self.language_tag)?;

        if self.uncertain_spans.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_turn_input.uncertain_spans",
                reason: "must be <= 32",
            });
        }
        for span in &self.uncertain_spans {
            span.validate()?;
        }

        if self.know_dictionary_hints.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "srl_turn_input.know_dictionary_hints",
                reason: "must be <= 64",
            });
        }
        for hint in &self.know_dictionary_hints {
            validate_token("srl_turn_input.know_dictionary_hints", hint, 96)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SrlForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub frame_build: SrlFrameBuildOk,
    pub argument_normalize: SrlArgumentNormalizeOk,
}

impl SrlForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        frame_build: SrlFrameBuildOk,
        argument_normalize: SrlArgumentNormalizeOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            frame_build,
            argument_normalize,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for SrlForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.frame_build.validate()?;
        self.argument_normalize.validate()?;

        if self.argument_normalize.validation_status != SrlValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "srl_forward_bundle.argument_normalize.validation_status",
                reason: "must be OK",
            });
        }
        if !self.frame_build.ambiguity_flags.is_empty() && !self.argument_normalize.clarify_required
        {
            return Err(ContractViolation::InvalidValue {
                field: "srl_forward_bundle.argument_normalize.clarify_required",
                reason: "must remain true when frame_build has ambiguity_flags",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SrlWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoTranscript,
    Refused(SrlRefuse),
    Forwarded(SrlForwardBundle),
}

pub trait Ph1SrlEngine {
    fn run(&self, req: &Ph1SrlRequest) -> Ph1SrlResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1SrlWiring<E>
where
    E: Ph1SrlEngine,
{
    config: Ph1SrlWiringConfig,
    engine: E,
}

impl<E> Ph1SrlWiring<E>
where
    E: Ph1SrlEngine,
{
    pub fn new(config: Ph1SrlWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_spans == 0 || config.max_spans > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1srl_wiring_config.max_spans",
                reason: "must be within 1..=64",
            });
        }
        if config.max_notes == 0 || config.max_notes > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1srl_wiring_config.max_notes",
                reason: "must be within 1..=64",
            });
        }
        if config.max_ambiguities == 0 || config.max_ambiguities > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1srl_wiring_config.max_ambiguities",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1srl_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &SrlTurnInput) -> Result<SrlWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.srl_enabled {
            return Ok(SrlWiringOutcome::NotInvokedDisabled);
        }
        if input.transcript_text.trim().is_empty() {
            return Ok(SrlWiringOutcome::NotInvokedNoTranscript);
        }

        let envelope = SrlRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_spans, 64),
            min(self.config.max_notes, 64),
            min(self.config.max_ambiguities, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let frame_req = Ph1SrlRequest::SrlFrameBuild(SrlFrameBuildRequest::v1(
            envelope.clone(),
            input.transcript_hash.clone(),
            input.transcript_text.clone(),
            input.language_tag.clone(),
            input.uncertain_spans.clone(),
            input.know_dictionary_hints.clone(),
            true,
            true,
            true,
        )?);
        let frame_resp = self.engine.run(&frame_req);
        frame_resp.validate()?;

        let frame_ok = match frame_resp {
            Ph1SrlResponse::Refuse(refuse) => return Ok(SrlWiringOutcome::Refused(refuse)),
            Ph1SrlResponse::SrlFrameBuildOk(ok) => ok,
            Ph1SrlResponse::SrlArgumentNormalizeOk(_) => {
                return Ok(SrlWiringOutcome::Refused(SrlRefuse::v1(
                    SrlCapabilityId::SrlFrameBuild,
                    reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                    "unexpected argument-normalize response for frame-build request".to_string(),
                )?));
            }
        };

        let normalize_req = Ph1SrlRequest::SrlArgumentNormalize(SrlArgumentNormalizeRequest::v1(
            envelope,
            input.transcript_hash.clone(),
            frame_ok.repaired_transcript_text.clone(),
            frame_ok.frame_spans.clone(),
            frame_ok.repair_notes.clone(),
            frame_ok.ambiguity_flags.clone(),
            true,
            true,
            true,
            true,
        )?);
        let normalize_resp = self.engine.run(&normalize_req);
        normalize_resp.validate()?;

        let normalize_ok = match normalize_resp {
            Ph1SrlResponse::Refuse(refuse) => return Ok(SrlWiringOutcome::Refused(refuse)),
            Ph1SrlResponse::SrlArgumentNormalizeOk(ok) => ok,
            Ph1SrlResponse::SrlFrameBuildOk(_) => {
                return Ok(SrlWiringOutcome::Refused(SrlRefuse::v1(
                    SrlCapabilityId::SrlArgumentNormalize,
                    reason_codes::PH1_SRL_INTERNAL_PIPELINE_ERROR,
                    "unexpected frame-build response for argument-normalize request".to_string(),
                )?));
            }
        };

        if normalize_ok.validation_status != SrlValidationStatus::Ok {
            return Ok(SrlWiringOutcome::Refused(SrlRefuse::v1(
                SrlCapabilityId::SrlArgumentNormalize,
                reason_codes::PH1_SRL_VALIDATION_FAILED,
                "srl argument normalization validation failed".to_string(),
            )?));
        }

        let bundle =
            SrlForwardBundle::v1(input.correlation_id, input.turn_id, frame_ok, normalize_ok)?;
        Ok(SrlWiringOutcome::Forwarded(bundle))
    }
}

fn validate_hash(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() < 16 || value.len() > 128 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 16..=128 chars",
        });
    }
    if !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be hex",
        });
    }
    Ok(())
}

fn validate_text(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(char::is_control) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control chars",
        });
    }
    Ok(())
}

fn validate_language_tag(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() || value.len() > 16 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be 1..=16 chars",
        });
    }
    if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII alphanumeric/hyphen",
        });
    }
    Ok(())
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain only ASCII token characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1srl::{SrlFrameSpan, SrlRepairNote, SrlRoleLabel};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicSrlEngine;

    impl Ph1SrlEngine for DeterministicSrlEngine {
        fn run(&self, req: &Ph1SrlRequest) -> Ph1SrlResponse {
            match req {
                Ph1SrlRequest::SrlFrameBuild(r) => {
                    let span = SrlFrameSpan::v1(
                        "span_001".to_string(),
                        0,
                        r.transcript_text.len() as u32,
                        r.transcript_text.clone(),
                        r.transcript_text.replace("tmr", "tomorrow"),
                        "en".to_string(),
                        SrlRoleLabel::Action,
                    )
                    .unwrap();
                    let note = SrlRepairNote::v1(
                        "note_001".to_string(),
                        "SHORTHAND_NORMALIZED".to_string(),
                        "normalized tmr -> tomorrow".to_string(),
                        "srl:span:1".to_string(),
                    )
                    .unwrap();
                    Ph1SrlResponse::SrlFrameBuildOk(
                        SrlFrameBuildOk::v1(
                            ReasonCodeId(901),
                            r.transcript_text.replace("tmr", "tomorrow"),
                            vec![span],
                            vec![note],
                            vec![],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1SrlRequest::SrlArgumentNormalize(r) => Ph1SrlResponse::SrlArgumentNormalizeOk(
                    SrlArgumentNormalizeOk::v1(
                        ReasonCodeId(902),
                        SrlValidationStatus::Ok,
                        vec![],
                        r.frame_spans.clone(),
                        r.ambiguity_flags.clone(),
                        !r.ambiguity_flags.is_empty(),
                        true,
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftSrlEngine;

    impl Ph1SrlEngine for DriftSrlEngine {
        fn run(&self, req: &Ph1SrlRequest) -> Ph1SrlResponse {
            match req {
                Ph1SrlRequest::SrlFrameBuild(r) => {
                    let span = SrlFrameSpan::v1(
                        "span_001".to_string(),
                        0,
                        r.transcript_text.len() as u32,
                        r.transcript_text.clone(),
                        r.transcript_text.clone(),
                        "en".to_string(),
                        SrlRoleLabel::Unknown,
                    )
                    .unwrap();
                    Ph1SrlResponse::SrlFrameBuildOk(
                        SrlFrameBuildOk::v1(
                            ReasonCodeId(911),
                            r.transcript_text.clone(),
                            vec![span],
                            vec![],
                            vec!["date_ambiguous".to_string()],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1SrlRequest::SrlArgumentNormalize(r) => Ph1SrlResponse::SrlArgumentNormalizeOk(
                    SrlArgumentNormalizeOk::v1(
                        ReasonCodeId(912),
                        SrlValidationStatus::Fail,
                        vec!["span_order_not_canonical".to_string()],
                        r.frame_spans.clone(),
                        r.ambiguity_flags.clone(),
                        true,
                        true,
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn input() -> SrlTurnInput {
        SrlTurnInput::v1(
            CorrelationId(6401),
            TurnId(611),
            "0123456789abcdef0123456789abcdef".to_string(),
            "Selene tmr remind me".to_string(),
            "en".to_string(),
            vec![SrlUncertainSpan::v1("u1".to_string(), 7, 10, Some("when".to_string())).unwrap()],
            vec!["selene".to_string()],
        )
        .unwrap()
    }

    #[test]
    fn at_srl_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1SrlWiring::new(Ph1SrlWiringConfig::mvp_v1(true), DeterministicSrlEngine).unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        match outcome {
            SrlWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_srl_02_validation_fail_is_refused() {
        let wiring = Ph1SrlWiring::new(Ph1SrlWiringConfig::mvp_v1(true), DriftSrlEngine).unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        match outcome {
            SrlWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_SRL_VALIDATION_FAILED);
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_srl_03_disabled_returns_not_invoked() {
        let wiring =
            Ph1SrlWiring::new(Ph1SrlWiringConfig::mvp_v1(false), DeterministicSrlEngine).unwrap();

        let outcome = wiring.run_turn(&input()).unwrap();
        assert_eq!(outcome, SrlWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_srl_04_empty_transcript_returns_not_invoked() {
        let wiring =
            Ph1SrlWiring::new(Ph1SrlWiringConfig::mvp_v1(true), DeterministicSrlEngine).unwrap();

        let empty_input = SrlTurnInput::v1(
            CorrelationId(6401),
            TurnId(611),
            "0123456789abcdef0123456789abcdef".to_string(),
            "   ".to_string(),
            "en".to_string(),
            vec![],
            vec![],
        )
        .unwrap();

        let outcome = wiring.run_turn(&empty_input).unwrap();
        assert_eq!(outcome, SrlWiringOutcome::NotInvokedNoTranscript);
    }
}
