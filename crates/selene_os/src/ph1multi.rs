#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1multi::{
    MultiBundleComposeOk, MultiBundleComposeRequest, MultiCapabilityId, MultiRefuse,
    MultiRequestEnvelope, MultiSignalAlignOk, MultiSignalAlignRequest, MultiSourceSignal,
    MultiValidationStatus, Ph1MultiRequest, Ph1MultiResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.MULTI OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_MULTI_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4D55_0101);
    pub const PH1_MULTI_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4D55_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1MultiWiringConfig {
    pub multi_enabled: bool,
    pub max_signals: u8,
    pub max_bundle_items: u8,
}

impl Ph1MultiWiringConfig {
    pub fn mvp_v1(multi_enabled: bool) -> Self {
        Self {
            multi_enabled,
            max_signals: 32,
            max_bundle_items: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signals: Vec<MultiSourceSignal>,
    pub include_vision: bool,
}

impl MultiTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signals: Vec<MultiSourceSignal>,
        include_vision: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            signals,
            include_vision,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for MultiTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.signals.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "multi_turn_input.signals",
                reason: "must be <= 64 signals",
            });
        }
        for signal in &self.signals {
            signal.validate()?;
        }
        if self.include_vision
            && !self.signals.iter().any(|signal| {
                matches!(
                    signal.modality,
                    selene_kernel_contracts::ph1multi::MultiModality::Vision
                )
            })
        {
            return Err(ContractViolation::InvalidValue {
                field: "multi_turn_input.include_vision",
                reason: "include_vision requires at least one vision signal",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MultiForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub bundle_compose: MultiBundleComposeOk,
    pub signal_align: MultiSignalAlignOk,
}

impl MultiForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        bundle_compose: MultiBundleComposeOk,
        signal_align: MultiSignalAlignOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            bundle_compose,
            signal_align,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for MultiForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.bundle_compose.validate()?;
        self.signal_align.validate()?;
        if self.signal_align.validation_status != MultiValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "multi_forward_bundle.signal_align.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MultiWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(MultiRefuse),
    Forwarded(MultiForwardBundle),
}

pub trait Ph1MultiEngine {
    fn run(&self, req: &Ph1MultiRequest) -> Ph1MultiResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1MultiWiring<E>
where
    E: Ph1MultiEngine,
{
    config: Ph1MultiWiringConfig,
    engine: E,
}

impl<E> Ph1MultiWiring<E>
where
    E: Ph1MultiEngine,
{
    pub fn new(config: Ph1MultiWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signals == 0 || config.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1multi_wiring_config.max_signals",
                reason: "must be within 1..=64",
            });
        }
        if config.max_bundle_items == 0 || config.max_bundle_items > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1multi_wiring_config.max_bundle_items",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &MultiTurnInput,
    ) -> Result<MultiWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.multi_enabled {
            return Ok(MultiWiringOutcome::NotInvokedDisabled);
        }
        if input.signals.is_empty() {
            return Ok(MultiWiringOutcome::NotInvokedNoSignals);
        }

        let envelope = MultiRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signals, 64),
            min(self.config.max_bundle_items, 32),
            true,
        )?;

        let compose_req = Ph1MultiRequest::MultiBundleCompose(MultiBundleComposeRequest::v1(
            envelope.clone(),
            input.signals.clone(),
            input.include_vision,
        )?);
        let compose_resp = self.engine.run(&compose_req);
        compose_resp.validate()?;

        let compose_ok = match compose_resp {
            Ph1MultiResponse::Refuse(refuse) => return Ok(MultiWiringOutcome::Refused(refuse)),
            Ph1MultiResponse::MultiBundleComposeOk(ok) => ok,
            Ph1MultiResponse::MultiSignalAlignOk(_) => {
                return Ok(MultiWiringOutcome::Refused(MultiRefuse::v1(
                    MultiCapabilityId::MultiBundleCompose,
                    reason_codes::PH1_MULTI_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-align response for bundle-compose request".to_string(),
                )?))
            }
        };

        let align_req = Ph1MultiRequest::MultiSignalAlign(MultiSignalAlignRequest::v1(
            envelope,
            compose_ok.selected_signal_id.clone(),
            compose_ok.ordered_bundle_items.clone(),
        )?);
        let align_resp = self.engine.run(&align_req);
        align_resp.validate()?;

        let align_ok = match align_resp {
            Ph1MultiResponse::Refuse(refuse) => return Ok(MultiWiringOutcome::Refused(refuse)),
            Ph1MultiResponse::MultiSignalAlignOk(ok) => ok,
            Ph1MultiResponse::MultiBundleComposeOk(_) => {
                return Ok(MultiWiringOutcome::Refused(MultiRefuse::v1(
                    MultiCapabilityId::MultiSignalAlign,
                    reason_codes::PH1_MULTI_INTERNAL_PIPELINE_ERROR,
                    "unexpected bundle-compose response for signal-align request".to_string(),
                )?))
            }
        };

        if align_ok.validation_status != MultiValidationStatus::Ok {
            return Ok(MultiWiringOutcome::Refused(MultiRefuse::v1(
                MultiCapabilityId::MultiSignalAlign,
                reason_codes::PH1_MULTI_VALIDATION_FAILED,
                "multi signal alignment failed validation".to_string(),
            )?));
        }

        let bundle =
            MultiForwardBundle::v1(input.correlation_id, input.turn_id, compose_ok, align_ok)?;
        Ok(MultiWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1multi::{
        MultiBundleComposeOk, MultiBundleItem, MultiModality, MultiSignalAlignOk,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicMultiEngine;

    impl Ph1MultiEngine for DeterministicMultiEngine {
        fn run(&self, req: &Ph1MultiRequest) -> Ph1MultiResponse {
            match req {
                Ph1MultiRequest::MultiBundleCompose(r) => {
                    let mut items = r
                        .signals
                        .iter()
                        .enumerate()
                        .map(|(idx, signal)| {
                            MultiBundleItem::v1(
                                signal.signal_id.clone(),
                                signal.source_engine.clone(),
                                signal.modality,
                                (idx + 1) as u8,
                                signal.confidence_pct,
                                signal.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    items.sort_by(|a, b| {
                        b.confidence_pct
                            .cmp(&a.confidence_pct)
                            .then(a.signal_id.cmp(&b.signal_id))
                    });
                    for (idx, item) in items.iter_mut().enumerate() {
                        item.fused_rank = (idx + 1) as u8;
                    }

                    Ph1MultiResponse::MultiBundleComposeOk(
                        MultiBundleComposeOk::v1(
                            ReasonCodeId(11),
                            items[0].signal_id.clone(),
                            items,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1MultiRequest::MultiSignalAlign(_r) => Ph1MultiResponse::MultiSignalAlignOk(
                    MultiSignalAlignOk::v1(
                        ReasonCodeId(12),
                        MultiValidationStatus::Ok,
                        vec![],
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftMultiEngine;

    impl Ph1MultiEngine for DriftMultiEngine {
        fn run(&self, req: &Ph1MultiRequest) -> Ph1MultiResponse {
            match req {
                Ph1MultiRequest::MultiBundleCompose(r) => {
                    let item = MultiBundleItem::v1(
                        r.signals[0].signal_id.clone(),
                        r.signals[0].source_engine.clone(),
                        r.signals[0].modality,
                        1,
                        r.signals[0].confidence_pct,
                        r.signals[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1MultiResponse::MultiBundleComposeOk(
                        MultiBundleComposeOk::v1(
                            ReasonCodeId(21),
                            item.signal_id.clone(),
                            vec![item],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1MultiRequest::MultiSignalAlign(_r) => Ph1MultiResponse::MultiSignalAlignOk(
                    MultiSignalAlignOk::v1(
                        ReasonCodeId(22),
                        MultiValidationStatus::Fail,
                        vec!["selected_not_first_in_ordered_bundle".to_string()],
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn signal(
        signal_id: &str,
        source_engine: &str,
        modality: MultiModality,
        confidence_pct: u8,
        evidence_ref: Option<&str>,
    ) -> MultiSourceSignal {
        MultiSourceSignal::v1(
            signal_id.to_string(),
            source_engine.to_string(),
            modality,
            "context_hint".to_string(),
            "value".to_string(),
            evidence_ref.map(|v| v.to_string()),
            confidence_pct,
            true,
        )
        .unwrap()
    }

    fn signals() -> Vec<MultiSourceSignal> {
        vec![
            signal("s_voice", "PH1.LISTEN", MultiModality::Voice, 82, None),
            signal("s_text", "PH1.CACHE", MultiModality::Text, 88, None),
            signal(
                "s_vision",
                "PH1.VISION",
                MultiModality::Vision,
                70,
                Some("vision:evidence:2"),
            ),
        ]
    }

    #[test]
    fn at_multi_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1MultiWiring::new(Ph1MultiWiringConfig::mvp_v1(true), DeterministicMultiEngine)
                .unwrap();
        let input = MultiTurnInput::v1(CorrelationId(2301), TurnId(201), signals(), true).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            MultiWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(bundle.bundle_compose.selected_signal_id, "s_text");
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_multi_02_os_order_is_deterministic_for_same_input() {
        let wiring =
            Ph1MultiWiring::new(Ph1MultiWiringConfig::mvp_v1(true), DeterministicMultiEngine)
                .unwrap();
        let input = MultiTurnInput::v1(CorrelationId(2302), TurnId(202), signals(), true).unwrap();

        let out1 = wiring.run_turn(&input).unwrap();
        let out2 = wiring.run_turn(&input).unwrap();

        let ordered1 = match out1 {
            MultiWiringOutcome::Forwarded(bundle) => bundle.bundle_compose.ordered_bundle_items,
            _ => panic!("expected Forwarded"),
        };
        let ordered2 = match out2 {
            MultiWiringOutcome::Forwarded(bundle) => bundle.bundle_compose.ordered_bundle_items,
            _ => panic!("expected Forwarded"),
        };

        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_multi_03_os_preserves_vision_evidence_bundle_for_context_handoff() {
        let wiring =
            Ph1MultiWiring::new(Ph1MultiWiringConfig::mvp_v1(true), DeterministicMultiEngine)
                .unwrap();
        let input = MultiTurnInput::v1(CorrelationId(2303), TurnId(203), signals(), true).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            MultiWiringOutcome::Forwarded(bundle) => {
                let vision = bundle
                    .bundle_compose
                    .ordered_bundle_items
                    .iter()
                    .find(|item| item.modality == MultiModality::Vision)
                    .expect("vision item must exist");
                assert_eq!(vision.evidence_ref.as_deref(), Some("vision:evidence:2"));
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_multi_04_os_fails_closed_on_signal_align_validation_drift() {
        let wiring =
            Ph1MultiWiring::new(Ph1MultiWiringConfig::mvp_v1(true), DriftMultiEngine).unwrap();
        let input = MultiTurnInput::v1(CorrelationId(2304), TurnId(204), signals(), true).unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            MultiWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.capability_id, MultiCapabilityId::MultiSignalAlign);
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_MULTI_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
