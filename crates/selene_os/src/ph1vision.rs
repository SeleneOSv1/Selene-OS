#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1vision::{
    Ph1VisionRequest, Ph1VisionResponse, VisionCapabilityId, VisionEvidenceExtractRequest,
    VisionEvidenceItem, VisionRefuse, VisionRequestEnvelope, VisionValidationStatus,
    VisionVisibleContentValidateOk, VisionVisibleContentValidateRequest, VisualSourceRef,
    VisualToken,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VISION OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_VISION_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5649_0201);
    pub const PH1_VISION_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5649_02F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1VisionWiringConfig {
    pub vision_opt_in_enabled: bool,
    pub max_evidence_items: u8,
}

impl Ph1VisionWiringConfig {
    pub fn mvp_v1(vision_opt_in_enabled: bool) -> Self {
        Self {
            vision_opt_in_enabled,
            max_evidence_items: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_ref: VisualSourceRef,
    pub visible_tokens: Vec<VisualToken>,
}

impl VisionTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_ref: VisualSourceRef,
        visible_tokens: Vec<VisualToken>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            correlation_id,
            turn_id,
            source_ref,
            visible_tokens,
        };
        v.validate()?;
        Ok(v)
    }
}

impl Validate for VisionTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.source_ref.validate()?;
        if self.visible_tokens.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_turn_input.visible_tokens",
                reason: "must be <= 256 items",
            });
        }
        for token in &self.visible_tokens {
            token.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_ref: VisualSourceRef,
    pub evidence_items: Vec<VisionEvidenceItem>,
    pub visible_content_only: bool,
}

impl VisionForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_ref: VisualSourceRef,
        evidence_items: Vec<VisionEvidenceItem>,
        visible_content_only: bool,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            source_ref,
            evidence_items,
            visible_content_only,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for VisionForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.source_ref.validate()?;
        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_forward_bundle.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_forward_bundle.evidence_items",
                reason: "must be <= 64 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }
        if !self.visible_content_only {
            return Err(ContractViolation::InvalidValue {
                field: "vision_forward_bundle.visible_content_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VisionWiringOutcome {
    NotInvokedOptOut,
    NotInvokedNoVisualInput,
    Refused(VisionRefuse),
    Forwarded {
        bundle: VisionForwardBundle,
        validation: VisionVisibleContentValidateOk,
    },
}

pub trait Ph1VisionEngine {
    fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1VisionWiring<E>
where
    E: Ph1VisionEngine,
{
    config: Ph1VisionWiringConfig,
    engine: E,
}

impl<E> Ph1VisionWiring<E>
where
    E: Ph1VisionEngine,
{
    pub fn new(config: Ph1VisionWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_evidence_items == 0 || config.max_evidence_items > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1vision_wiring_config.max_evidence_items",
                reason: "must be within 1..=64",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &VisionTurnInput,
    ) -> Result<VisionWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.vision_opt_in_enabled {
            return Ok(VisionWiringOutcome::NotInvokedOptOut);
        }

        if input.visible_tokens.is_empty() {
            return Ok(VisionWiringOutcome::NotInvokedNoVisualInput);
        }

        let max_items = min(self.config.max_evidence_items, 64);
        let envelope =
            VisionRequestEnvelope::v1(input.correlation_id, input.turn_id, true, max_items)?;

        let extract_req = Ph1VisionRequest::EvidenceExtract(VisionEvidenceExtractRequest::v1(
            envelope.clone(),
            input.source_ref.clone(),
            input.visible_tokens.clone(),
        )?);

        let extract_resp = self.engine.run(&extract_req);
        extract_resp.validate()?;

        let extract_ok = match extract_resp {
            Ph1VisionResponse::Refuse(r) => return Ok(VisionWiringOutcome::Refused(r)),
            Ph1VisionResponse::EvidenceExtractOk(ok) => ok,
            Ph1VisionResponse::VisibleContentValidateOk(_) => {
                let r = VisionRefuse::v1(
                    VisionCapabilityId::EvidenceExtract,
                    reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                    "unexpected validation response for extract request".to_string(),
                )?;
                return Ok(VisionWiringOutcome::Refused(r));
            }
        };

        let validate_req =
            Ph1VisionRequest::VisibleContentValidate(VisionVisibleContentValidateRequest::v1(
                envelope,
                input.source_ref.clone(),
                input.visible_tokens.clone(),
                extract_ok.evidence_items.clone(),
            )?);

        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1VisionResponse::Refuse(r) => return Ok(VisionWiringOutcome::Refused(r)),
            Ph1VisionResponse::VisibleContentValidateOk(ok) => ok,
            Ph1VisionResponse::EvidenceExtractOk(_) => {
                let r = VisionRefuse::v1(
                    VisionCapabilityId::VisibleContentValidate,
                    reason_codes::PH1_VISION_INTERNAL_PIPELINE_ERROR,
                    "unexpected extract response for validation request".to_string(),
                )?;
                return Ok(VisionWiringOutcome::Refused(r));
            }
        };

        if validate_ok.validation_status != VisionValidationStatus::Ok {
            let r = VisionRefuse::v1(
                VisionCapabilityId::VisibleContentValidate,
                reason_codes::PH1_VISION_VALIDATION_FAILED,
                "vision visible-content validation failed".to_string(),
            )?;
            return Ok(VisionWiringOutcome::Refused(r));
        }

        let bundle = VisionForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            extract_ok.source_ref,
            extract_ok.evidence_items,
            extract_ok.visible_content_only,
        )?;

        Ok(VisionWiringOutcome::Forwarded {
            bundle,
            validation: validate_ok,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1vision::{
        VisionEvidenceExtractOk, VisualSourceId, VisualSourceKind,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicVisionEngine;

    impl Ph1VisionEngine for DeterministicVisionEngine {
        fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse {
            match req {
                Ph1VisionRequest::EvidenceExtract(r) => {
                    if !r.envelope.opt_in_enabled {
                        return Ph1VisionResponse::Refuse(
                            VisionRefuse::v1(
                                VisionCapabilityId::EvidenceExtract,
                                ReasonCodeId(1),
                                "opt-in disabled".to_string(),
                            )
                            .unwrap(),
                        );
                    }
                    let items = r
                        .visible_tokens
                        .iter()
                        .take(r.envelope.max_evidence_items as usize)
                        .map(|t| VisionEvidenceItem::v1(t.token.clone(), t.bbox).unwrap())
                        .collect::<Vec<_>>();
                    Ph1VisionResponse::EvidenceExtractOk(
                        VisionEvidenceExtractOk::v1(
                            ReasonCodeId(11),
                            r.source_ref.clone(),
                            items,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1VisionRequest::VisibleContentValidate(r) => {
                    let visible = r
                        .visible_tokens
                        .iter()
                        .map(|t| t.token.to_ascii_lowercase())
                        .collect::<Vec<_>>();
                    let mut bad = vec![];
                    for (idx, e) in r.evidence_items.iter().enumerate() {
                        if !visible.iter().any(|v| v == &e.text.to_ascii_lowercase()) {
                            bad.push(format!("evidence_index_{idx}_not_visible_content"));
                        }
                    }

                    let (status, rc) = if bad.is_empty() {
                        (VisionValidationStatus::Ok, ReasonCodeId(12))
                    } else {
                        (VisionValidationStatus::Fail, ReasonCodeId(13))
                    };

                    Ph1VisionResponse::VisibleContentValidateOk(
                        VisionVisibleContentValidateOk::v1(
                            rc,
                            r.source_ref.clone(),
                            status,
                            bad,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct InferenceDriftEngine;

    impl Ph1VisionEngine for InferenceDriftEngine {
        fn run(&self, req: &Ph1VisionRequest) -> Ph1VisionResponse {
            match req {
                Ph1VisionRequest::EvidenceExtract(r) => {
                    let mut items =
                        vec![
                            VisionEvidenceItem::v1("inferred_hidden_value".to_string(), None)
                                .unwrap(),
                        ];
                    for token in &r.visible_tokens {
                        items
                            .push(VisionEvidenceItem::v1(token.token.clone(), token.bbox).unwrap());
                    }
                    Ph1VisionResponse::EvidenceExtractOk(
                        VisionEvidenceExtractOk::v1(
                            ReasonCodeId(20),
                            r.source_ref.clone(),
                            items,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1VisionRequest::VisibleContentValidate(r) => {
                    let visible = r
                        .visible_tokens
                        .iter()
                        .map(|t| t.token.to_ascii_lowercase())
                        .collect::<Vec<_>>();
                    let mut bad = vec![];
                    for (idx, e) in r.evidence_items.iter().enumerate() {
                        if !visible.iter().any(|v| v == &e.text.to_ascii_lowercase()) {
                            bad.push(format!("evidence_index_{idx}_not_visible_content"));
                        }
                    }
                    Ph1VisionResponse::VisibleContentValidateOk(
                        VisionVisibleContentValidateOk::v1(
                            ReasonCodeId(21),
                            r.source_ref.clone(),
                            VisionValidationStatus::Fail,
                            bad,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn source() -> VisualSourceRef {
        VisualSourceRef::v1(
            VisualSourceId::new("src_vision_01").unwrap(),
            VisualSourceKind::Image,
        )
        .unwrap()
    }

    fn token(s: &str) -> VisualToken {
        VisualToken::v1(s.to_string(), None).unwrap()
    }

    #[test]
    fn at_vision_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(true),
            DeterministicVisionEngine,
        )
        .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(101),
            TurnId(55),
            source(),
            vec![token("title"), token("subtotal")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            VisionWiringOutcome::Forwarded { bundle, validation } => {
                assert!(bundle.validate().is_ok());
                assert_eq!(validation.validation_status, VisionValidationStatus::Ok);
            }
            _ => panic!("expected Forwarded outcome"),
        }
    }

    #[test]
    fn at_vision_02_os_preserves_deterministic_order_for_related_engine_handoff() {
        let wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(true),
            DeterministicVisionEngine,
        )
        .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(102),
            TurnId(56),
            source(),
            vec![token("first"), token("second"), token("third")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            VisionWiringOutcome::Forwarded { bundle, .. } => {
                let list = bundle
                    .evidence_items
                    .into_iter()
                    .map(|it| it.text)
                    .collect::<Vec<_>>();
                assert_eq!(list, vec!["first", "second", "third"]);
            }
            _ => panic!("expected Forwarded outcome"),
        }
    }

    #[test]
    fn at_vision_03_os_does_not_invoke_when_opt_in_disabled() {
        let wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(false),
            DeterministicVisionEngine,
        )
        .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(103),
            TurnId(57),
            source(),
            vec![token("anything")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, VisionWiringOutcome::NotInvokedOptOut);
    }

    #[test]
    fn at_vision_04_os_fails_closed_when_validation_detects_inference_drift() {
        let wiring =
            Ph1VisionWiring::new(Ph1VisionWiringConfig::mvp_v1(true), InferenceDriftEngine)
                .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(104),
            TurnId(58),
            source(),
            vec![token("visible_only")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            VisionWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_VISION_VALIDATION_FAILED);
                assert_eq!(r.capability_id, VisionCapabilityId::VisibleContentValidate);
            }
            _ => panic!("expected Refused outcome"),
        }
    }
}
