#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1lang::{
    LangCapabilityId, LangMultipleDetectOk, LangMultipleDetectRequest, LangRefuse,
    LangRequestEnvelope, LangResponseMode, LangSegmentResponseMapOk, LangSegmentResponseMapRequest,
    LangSourceModality, LangValidationStatus, Ph1LangRequest, Ph1LangResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LANG OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LANG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C41_0101);
    pub const PH1_LANG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C41_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1LangWiringConfig {
    pub lang_enabled: bool,
    pub max_segments: u8,
}

impl Ph1LangWiringConfig {
    pub fn mvp_v1(lang_enabled: bool) -> Self {
        Self {
            lang_enabled,
            max_segments: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub transcript_text: String,
    pub locale_hint: Option<String>,
    pub source_modality: LangSourceModality,
    pub user_language_preferences: Vec<String>,
    pub response_mode: LangResponseMode,
}

impl LangTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        transcript_text: String,
        locale_hint: Option<String>,
        source_modality: LangSourceModality,
        user_language_preferences: Vec<String>,
        response_mode: LangResponseMode,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            transcript_text,
            locale_hint,
            source_modality,
            user_language_preferences,
            response_mode,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for LangTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;

        if self.transcript_text.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_turn_input.transcript_text",
                reason: "must be <= 2048 chars",
            });
        }

        if self.transcript_text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "lang_turn_input.transcript_text",
                reason: "must not contain control characters",
            });
        }

        if let Some(locale_hint) = &self.locale_hint {
            if locale_hint.len() > 32
                || !locale_hint
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_turn_input.locale_hint",
                    reason: "must be <= 32 chars and contain only ASCII alphanumeric/hyphen",
                });
            }
        }

        if self.user_language_preferences.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "lang_turn_input.user_language_preferences",
                reason: "must be <= 16 items",
            });
        }

        for language_tag in &self.user_language_preferences {
            if language_tag.is_empty()
                || language_tag.len() > 16
                || !language_tag
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-')
            {
                return Err(ContractViolation::InvalidValue {
                    field: "lang_turn_input.user_language_preferences",
                    reason: "language tags must be <= 16 chars and ASCII alphanumeric/hyphen",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LangForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub detect: LangMultipleDetectOk,
    pub map: LangSegmentResponseMapOk,
}

impl LangForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        detect: LangMultipleDetectOk,
        map: LangSegmentResponseMapOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            detect,
            map,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for LangForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.detect.validate()?;
        self.map.validate()?;

        if self.map.validation_status != LangValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "lang_forward_bundle.map.validation_status",
                reason: "must be OK",
            });
        }

        if !self.detect.no_translation_performed || !self.map.no_translation_performed {
            return Err(ContractViolation::InvalidValue {
                field: "lang_forward_bundle.no_translation_performed",
                reason: "must remain true in both outputs",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoTranscript,
    Refused(LangRefuse),
    Forwarded(LangForwardBundle),
}

pub trait Ph1LangEngine {
    fn run(&self, req: &Ph1LangRequest) -> Ph1LangResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1LangWiring<E>
where
    E: Ph1LangEngine,
{
    config: Ph1LangWiringConfig,
    engine: E,
}

impl<E> Ph1LangWiring<E>
where
    E: Ph1LangEngine,
{
    pub fn new(config: Ph1LangWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_segments == 0 || config.max_segments > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1lang_wiring_config.max_segments",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &LangTurnInput) -> Result<LangWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.lang_enabled {
            return Ok(LangWiringOutcome::NotInvokedDisabled);
        }

        if input.transcript_text.trim().is_empty() {
            return Ok(LangWiringOutcome::NotInvokedNoTranscript);
        }

        let envelope = LangRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_segments, 16),
        )?;

        let detect_req = Ph1LangRequest::LangMultipleDetect(LangMultipleDetectRequest::v1(
            envelope.clone(),
            input.transcript_text.clone(),
            input.locale_hint.clone(),
            input.source_modality,
        )?);
        let detect_resp = self.engine.run(&detect_req);
        detect_resp.validate()?;

        let detect_ok = match detect_resp {
            Ph1LangResponse::Refuse(r) => return Ok(LangWiringOutcome::Refused(r)),
            Ph1LangResponse::LangMultipleDetectOk(ok) => ok,
            Ph1LangResponse::LangSegmentResponseMapOk(_) => {
                return Ok(LangWiringOutcome::Refused(LangRefuse::v1(
                    LangCapabilityId::LangMultipleDetect,
                    reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR,
                    "unexpected segment-response-map response for multiple-detect request"
                        .to_string(),
                )?))
            }
        };

        let map_req = Ph1LangRequest::LangSegmentResponseMap(LangSegmentResponseMapRequest::v1(
            envelope,
            input.transcript_text.clone(),
            input.locale_hint.clone(),
            input.source_modality,
            detect_ok.detected_languages.clone(),
            detect_ok.segment_spans.clone(),
            input.user_language_preferences.clone(),
            input.response_mode,
        )?);

        let map_resp = self.engine.run(&map_req);
        map_resp.validate()?;

        let map_ok = match map_resp {
            Ph1LangResponse::Refuse(r) => return Ok(LangWiringOutcome::Refused(r)),
            Ph1LangResponse::LangSegmentResponseMapOk(ok) => ok,
            Ph1LangResponse::LangMultipleDetectOk(_) => {
                return Ok(LangWiringOutcome::Refused(LangRefuse::v1(
                    LangCapabilityId::LangSegmentResponseMap,
                    reason_codes::PH1_LANG_INTERNAL_PIPELINE_ERROR,
                    "unexpected multiple-detect response for segment-response-map request"
                        .to_string(),
                )?))
            }
        };

        if map_ok.validation_status != LangValidationStatus::Ok {
            return Ok(LangWiringOutcome::Refused(LangRefuse::v1(
                LangCapabilityId::LangSegmentResponseMap,
                reason_codes::PH1_LANG_VALIDATION_FAILED,
                "lang response mapping validation failed".to_string(),
            )?));
        }

        let bundle = LangForwardBundle::v1(input.correlation_id, input.turn_id, detect_ok, map_ok)?;
        Ok(LangWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1lang::{LangPlanScope, LangResponsePlanItem};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicLangEngine;

    impl Ph1LangEngine for DeterministicLangEngine {
        fn run(&self, req: &Ph1LangRequest) -> Ph1LangResponse {
            match req {
                Ph1LangRequest::LangMultipleDetect(r) => {
                    let segment = selene_kernel_contracts::ph1lang::LangSegment::v1(
                        "seg_000".to_string(),
                        0,
                        r.transcript_text.len() as u32,
                        "en".to_string(),
                        r.transcript_text.clone(),
                    )
                    .unwrap();
                    Ph1LangResponse::LangMultipleDetectOk(
                        LangMultipleDetectOk::v1(
                            ReasonCodeId(1),
                            vec!["en".to_string()],
                            vec![segment],
                            "en".to_string(),
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1LangRequest::LangSegmentResponseMap(r) => {
                    let plan = match r.response_mode {
                        LangResponseMode::Voice => vec![LangResponsePlanItem::v1(
                            LangPlanScope::Turn,
                            None,
                            "en".to_string(),
                        )
                        .unwrap()],
                        LangResponseMode::Text => r
                            .segment_spans
                            .iter()
                            .map(|segment| {
                                LangResponsePlanItem::v1(
                                    LangPlanScope::Segment,
                                    Some(segment.segment_id.clone()),
                                    segment.language_tag.clone(),
                                )
                                .unwrap()
                            })
                            .collect(),
                    };

                    Ph1LangResponse::LangSegmentResponseMapOk(
                        LangSegmentResponseMapOk::v1(
                            ReasonCodeId(2),
                            LangValidationStatus::Ok,
                            plan,
                            "en".to_string(),
                            vec![],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftLangEngine;

    impl Ph1LangEngine for DriftLangEngine {
        fn run(&self, req: &Ph1LangRequest) -> Ph1LangResponse {
            match req {
                Ph1LangRequest::LangMultipleDetect(r) => {
                    let segment = selene_kernel_contracts::ph1lang::LangSegment::v1(
                        "seg_000".to_string(),
                        0,
                        r.transcript_text.len() as u32,
                        "en".to_string(),
                        r.transcript_text.clone(),
                    )
                    .unwrap();
                    Ph1LangResponse::LangMultipleDetectOk(
                        LangMultipleDetectOk::v1(
                            ReasonCodeId(10),
                            vec!["en".to_string()],
                            vec![segment],
                            "en".to_string(),
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1LangRequest::LangSegmentResponseMap(r) => {
                    let plan = vec![LangResponsePlanItem::v1(
                        LangPlanScope::Turn,
                        None,
                        r.detected_languages[0].clone(),
                    )
                    .unwrap()];
                    Ph1LangResponse::LangSegmentResponseMapOk(
                        LangSegmentResponseMapOk::v1(
                            ReasonCodeId(11),
                            LangValidationStatus::Fail,
                            plan,
                            r.detected_languages[0].clone(),
                            vec!["user_language_preferences_no_match".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    #[test]
    fn at_lang_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1LangWiring::new(Ph1LangWiringConfig::mvp_v1(true), DeterministicLangEngine).unwrap();

        let input = LangTurnInput::v1(
            CorrelationId(1201),
            TurnId(81),
            "hello world".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Text,
            vec!["en".to_string()],
            LangResponseMode::Voice,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            LangWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(bundle.map.validation_status, LangValidationStatus::Ok);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_lang_02_text_mode_preserves_segment_plan_for_downstream() {
        let wiring =
            Ph1LangWiring::new(Ph1LangWiringConfig::mvp_v1(true), DeterministicLangEngine).unwrap();

        let input = LangTurnInput::v1(
            CorrelationId(1202),
            TurnId(82),
            "hello world".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Text,
            vec!["en".to_string()],
            LangResponseMode::Text,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            LangWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert!(bundle
                    .map
                    .response_language_plan
                    .iter()
                    .all(|item| item.scope == LangPlanScope::Segment));
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_lang_03_validation_fail_fails_closed_before_handoff() {
        let wiring =
            Ph1LangWiring::new(Ph1LangWiringConfig::mvp_v1(true), DriftLangEngine).unwrap();

        let input = LangTurnInput::v1(
            CorrelationId(1203),
            TurnId(83),
            "hello world".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Voice,
            vec!["en".to_string()],
            LangResponseMode::Voice,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            LangWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_LANG_VALIDATION_FAILED);
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_lang_04_disabled_or_empty_turn_never_invokes_engine() {
        let wiring_disabled =
            Ph1LangWiring::new(Ph1LangWiringConfig::mvp_v1(false), DeterministicLangEngine)
                .unwrap();

        let input = LangTurnInput::v1(
            CorrelationId(1204),
            TurnId(84),
            "hello".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Text,
            vec![],
            LangResponseMode::Voice,
        )
        .unwrap();

        assert_eq!(
            wiring_disabled.run_turn(&input).unwrap(),
            LangWiringOutcome::NotInvokedDisabled
        );

        let wiring_enabled =
            Ph1LangWiring::new(Ph1LangWiringConfig::mvp_v1(true), DeterministicLangEngine).unwrap();

        let empty_input = LangTurnInput::v1(
            CorrelationId(1204),
            TurnId(85),
            "   ".to_string(),
            Some("en-US".to_string()),
            LangSourceModality::Text,
            vec![],
            LangResponseMode::Voice,
        )
        .unwrap();

        assert_eq!(
            wiring_enabled.run_turn(&empty_input).unwrap(),
            LangWiringOutcome::NotInvokedNoTranscript
        );
    }
}
