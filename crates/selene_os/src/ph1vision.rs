#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1vision::{
    Ph1VisionRequest, Ph1VisionResponse, VisionCapabilityId, VisionEvidenceExtractRequest,
    VisionEvidenceItem, VisionRawSourceRef, VisionRefuse, VisionRequestEnvelope,
    VisionValidationStatus,
    VisionVisibleContentValidateOk, VisionVisibleContentValidateRequest, VisualSourceRef,
    VisualToken,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.VISION OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_VISION_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5649_0201);
    pub const PH1_VISION_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5649_02F1);
    pub const PH1_VISION_WIRING_CONTRACT_INVALID: ReasonCodeId = ReasonCodeId(0x5649_02F2);
    pub const PH1_VISION_OPT_IN_REQUIRED: ReasonCodeId = ReasonCodeId(0x5649_02F3);
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
    pub turn_opt_in_enabled: bool,
    pub source_ref: VisualSourceRef,
    pub raw_source_ref: Option<VisionRawSourceRef>,
    pub visible_tokens: Vec<VisualToken>,
}

impl VisionTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        turn_opt_in_enabled: bool,
        source_ref: VisualSourceRef,
        raw_source_ref: Option<VisionRawSourceRef>,
        visible_tokens: Vec<VisualToken>,
    ) -> Result<Self, ContractViolation> {
        let v = Self {
            correlation_id,
            turn_id,
            turn_opt_in_enabled,
            source_ref,
            raw_source_ref,
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
        if let Some(raw_source_ref) = &self.raw_source_ref {
            raw_source_ref.validate()?;
        }
        if self.visible_tokens.is_empty() && self.raw_source_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_turn_input",
                reason: "must include visible_tokens or raw_source_ref",
            });
        }
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
        if input.validate().is_err() {
            return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                VisionCapabilityId::EvidenceExtract,
                "vision turn input failed contract validation",
            )?));
        }

        if !self.config.vision_opt_in_enabled || !input.turn_opt_in_enabled {
            return Ok(VisionWiringOutcome::Refused(VisionRefuse::v1(
                VisionCapabilityId::EvidenceExtract,
                reason_codes::PH1_VISION_OPT_IN_REQUIRED,
                "vision opt-in is required for this turn".to_string(),
            )?));
        }

        let visible_tokens = if input.visible_tokens.is_empty() {
            derive_visible_tokens_from_raw_source(input.raw_source_ref.as_ref())
        } else {
            input.visible_tokens.clone()
        };

        if visible_tokens.is_empty() {
            return Ok(VisionWiringOutcome::NotInvokedNoVisualInput);
        }

        let max_items = min(self.config.max_evidence_items, 64);
        let envelope = match VisionRequestEnvelope::v1(input.correlation_id, input.turn_id, true, max_items) {
            Ok(v) => v,
            Err(_) => {
                return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                    VisionCapabilityId::EvidenceExtract,
                    "failed to construct vision request envelope",
                )?))
            }
        };

        let extract_req = match VisionEvidenceExtractRequest::v1(
            envelope.clone(),
            input.source_ref.clone(),
            input.raw_source_ref.clone(),
            visible_tokens.clone(),
        ) {
            Ok(req) => Ph1VisionRequest::EvidenceExtract(req),
            Err(_) => {
                return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                    VisionCapabilityId::EvidenceExtract,
                    "failed to construct vision extract request",
                )?))
            }
        };

        let extract_resp = self.engine.run(&extract_req);
        if extract_resp.validate().is_err() {
            return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                VisionCapabilityId::EvidenceExtract,
                "vision extract response failed contract validation",
            )?));
        }

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

        let validate_req = match VisionVisibleContentValidateRequest::v1(
            envelope,
            input.source_ref.clone(),
            visible_tokens,
            extract_ok.evidence_items.clone(),
        ) {
            Ok(req) => Ph1VisionRequest::VisibleContentValidate(req),
            Err(_) => {
                return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                    VisionCapabilityId::VisibleContentValidate,
                    "failed to construct vision validate request",
                )?))
            }
        };

        let validate_resp = self.engine.run(&validate_req);
        if validate_resp.validate().is_err() {
            return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                VisionCapabilityId::VisibleContentValidate,
                "vision validate response failed contract validation",
            )?));
        }

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

        let bundle = match VisionForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            extract_ok.source_ref,
            extract_ok.evidence_items,
            extract_ok.visible_content_only,
        ) {
            Ok(bundle) => bundle,
            Err(_) => {
                return Ok(VisionWiringOutcome::Refused(refuse_contract_invalid(
                    VisionCapabilityId::EvidenceExtract,
                    "failed to construct PH1.VISION forward bundle",
                )?))
            }
        };

        Ok(VisionWiringOutcome::Forwarded {
            bundle,
            validation: validate_ok,
        })
    }
}

fn refuse_contract_invalid(
    capability_id: VisionCapabilityId,
    message: &'static str,
) -> Result<VisionRefuse, ContractViolation> {
    VisionRefuse::v1(
        capability_id,
        reason_codes::PH1_VISION_WIRING_CONTRACT_INVALID,
        message.to_string(),
    )
}

fn derive_visible_tokens_from_raw_source(raw_source_ref: Option<&VisionRawSourceRef>) -> Vec<VisualToken> {
    let Some(raw_source_ref) = raw_source_ref else {
        return Vec::new();
    };
    let mut out = Vec::new();
    if let Some(image_ref) = raw_source_ref.image_ref.as_deref() {
        out.extend(derive_tokens_from_ref(image_ref));
    }
    if let Some(blob_ref) = raw_source_ref.blob_ref.as_deref() {
        out.extend(derive_tokens_from_ref(blob_ref));
    }
    out
}

fn derive_tokens_from_ref(value: &str) -> Vec<VisualToken> {
    value
        .split(|c: char| !c.is_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .take(64)
        .filter_map(|token| VisualToken::v1(token.to_string(), None).ok())
        .collect()
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
                        .map(|t| t.token.to_lowercase())
                        .collect::<Vec<_>>();
                    let mut bad = vec![];
                    for (idx, e) in r.evidence_items.iter().enumerate() {
                        if !visible.iter().any(|v| v == &e.text.to_lowercase()) {
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
                        .map(|t| t.token.to_lowercase())
                        .collect::<Vec<_>>();
                    let mut bad = vec![];
                    for (idx, e) in r.evidence_items.iter().enumerate() {
                        if !visible.iter().any(|v| v == &e.text.to_lowercase()) {
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
            true,
            source(),
            None,
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
            true,
            source(),
            None,
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
            true,
            source(),
            None,
            vec![token("anything")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        let VisionWiringOutcome::Refused(refuse) = out else {
            panic!("expected refused outcome when global opt-in disabled");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_VISION_OPT_IN_REQUIRED);
    }

    #[test]
    fn at_vision_04_os_fails_closed_when_validation_detects_inference_drift() {
        let wiring =
            Ph1VisionWiring::new(Ph1VisionWiringConfig::mvp_v1(true), InferenceDriftEngine)
                .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(104),
            TurnId(58),
            true,
            source(),
            None,
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

    #[test]
    fn at_vision_05_os_refuses_when_turn_opt_in_not_granted() {
        let wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(true),
            DeterministicVisionEngine,
        )
        .unwrap();

        let input = VisionTurnInput::v1(
            CorrelationId(105),
            TurnId(59),
            false,
            source(),
            None,
            vec![token("anything")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        let VisionWiringOutcome::Refused(refuse) = out else {
            panic!("expected refusal when turn opt-in is not granted");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_VISION_OPT_IN_REQUIRED);
    }

    #[test]
    fn at_vision_06_os_supports_raw_source_when_visible_tokens_absent() {
        let wiring = Ph1VisionWiring::new(
            Ph1VisionWiringConfig::mvp_v1(true),
            DeterministicVisionEngine,
        )
        .unwrap();
        let raw_source = VisionRawSourceRef::v1(
            Some("image://invoice_total".to_string()),
            Some("blob://capture/line_items".to_string()),
        )
        .unwrap();
        let input = VisionTurnInput::v1(
            CorrelationId(106),
            TurnId(60),
            true,
            source(),
            Some(raw_source),
            vec![],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        let VisionWiringOutcome::Forwarded { bundle, .. } = out else {
            panic!("expected forwarded outcome");
        };
        assert!(!bundle.evidence_items.is_empty());
    }
}
