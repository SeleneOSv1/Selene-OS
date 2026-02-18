#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1context::{
    ContextBundleBuildOk, ContextBundleBuildRequest, ContextBundleTrimOk, ContextBundleTrimRequest,
    ContextCapabilityId, ContextRefuse, ContextRequestEnvelope, ContextSourceItem,
    ContextValidationStatus, Ph1ContextRequest, Ph1ContextResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.CONTEXT OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_CONTEXT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4358_0101);
    pub const PH1_CONTEXT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4358_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ContextWiringConfig {
    pub context_enabled: bool,
    pub max_items: u8,
    pub max_diagnostics: u8,
}

impl Ph1ContextWiringConfig {
    pub fn mvp_v1(context_enabled: bool) -> Self {
        Self {
            context_enabled,
            max_items: 12,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub intent_type: String,
    pub privacy_mode: bool,
    pub source_items: Vec<ContextSourceItem>,
    pub multi_signal_align_ok: bool,
    pub cache_hint_refresh_ok: bool,
}

impl ContextTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        intent_type: String,
        privacy_mode: bool,
        source_items: Vec<ContextSourceItem>,
        multi_signal_align_ok: bool,
        cache_hint_refresh_ok: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            intent_type,
            privacy_mode,
            source_items,
            multi_signal_align_ok,
            cache_hint_refresh_ok,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for ContextTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.intent_type.len() > 96 || self.intent_type.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "context_turn_input.intent_type",
                reason: "must be <= 96 chars and contain no control chars",
            });
        }
        if self.source_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "context_turn_input.source_items",
                reason: "must be <= 128",
            });
        }
        for source_item in &self.source_items {
            source_item.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub bundle_build: ContextBundleBuildOk,
    pub bundle_trim: ContextBundleTrimOk,
}

impl ContextForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        bundle_build: ContextBundleBuildOk,
        bundle_trim: ContextBundleTrimOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            bundle_build,
            bundle_trim,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for ContextForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.bundle_build.validate()?;
        self.bundle_trim.validate()?;
        if self.bundle_trim.validation_status != ContextValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "context_forward_bundle.bundle_trim.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoContextInput,
    Refused(ContextRefuse),
    Forwarded(ContextForwardBundle),
}

pub trait Ph1ContextEngine {
    fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1ContextWiring<E>
where
    E: Ph1ContextEngine,
{
    config: Ph1ContextWiringConfig,
    engine: E,
}

impl<E> Ph1ContextWiring<E>
where
    E: Ph1ContextEngine,
{
    pub fn new(config: Ph1ContextWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_items == 0 || config.max_items > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1context_wiring_config.max_items",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1context_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &ContextTurnInput,
    ) -> Result<ContextWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.context_enabled {
            return Ok(ContextWiringOutcome::NotInvokedDisabled);
        }
        if input.intent_type.trim().is_empty() || input.source_items.is_empty() {
            return Ok(ContextWiringOutcome::NotInvokedNoContextInput);
        }

        let envelope = ContextRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_items, 32),
            min(self.config.max_diagnostics, 16),
            true,
        )?;

        let build_req = Ph1ContextRequest::ContextBundleBuild(ContextBundleBuildRequest::v1(
            envelope.clone(),
            input.intent_type.clone(),
            input.privacy_mode,
            input.source_items.clone(),
            input.multi_signal_align_ok,
            input.cache_hint_refresh_ok,
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1ContextResponse::Refuse(refuse) => return Ok(ContextWiringOutcome::Refused(refuse)),
            Ph1ContextResponse::ContextBundleBuildOk(ok) => ok,
            Ph1ContextResponse::ContextBundleTrimOk(_) => {
                return Ok(ContextWiringOutcome::Refused(ContextRefuse::v1(
                    ContextCapabilityId::ContextBundleBuild,
                    reason_codes::PH1_CONTEXT_INTERNAL_PIPELINE_ERROR,
                    "unexpected bundle-trim response for bundle-build request".to_string(),
                )?));
            }
        };

        let trim_req = Ph1ContextRequest::ContextBundleTrim(ContextBundleTrimRequest::v1(
            envelope,
            input.intent_type.clone(),
            input.privacy_mode,
            build_ok.selected_item_ids.clone(),
            build_ok.ordered_bundle_items.clone(),
            input.multi_signal_align_ok,
            input.cache_hint_refresh_ok,
        )?);
        let trim_resp = self.engine.run(&trim_req);
        trim_resp.validate()?;

        let trim_ok = match trim_resp {
            Ph1ContextResponse::Refuse(refuse) => return Ok(ContextWiringOutcome::Refused(refuse)),
            Ph1ContextResponse::ContextBundleTrimOk(ok) => ok,
            Ph1ContextResponse::ContextBundleBuildOk(_) => {
                return Ok(ContextWiringOutcome::Refused(ContextRefuse::v1(
                    ContextCapabilityId::ContextBundleTrim,
                    reason_codes::PH1_CONTEXT_INTERNAL_PIPELINE_ERROR,
                    "unexpected bundle-build response for bundle-trim request".to_string(),
                )?));
            }
        };

        if trim_ok.validation_status != ContextValidationStatus::Ok {
            return Ok(ContextWiringOutcome::Refused(ContextRefuse::v1(
                ContextCapabilityId::ContextBundleTrim,
                reason_codes::PH1_CONTEXT_VALIDATION_FAILED,
                "context bundle trim validation failed".to_string(),
            )?));
        }

        let bundle =
            ContextForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, trim_ok)?;
        Ok(ContextWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1context::{
        ContextBundleBuildOk, ContextBundleItem, ContextBundleTrimOk, ContextSourceKind,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicContextEngine;

    impl Ph1ContextEngine for DeterministicContextEngine {
        fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse {
            match req {
                Ph1ContextRequest::ContextBundleBuild(r) => {
                    let mut ordered = r
                        .source_items
                        .iter()
                        .enumerate()
                        .map(|(idx, source_item)| {
                            ContextBundleItem::v1(
                                source_item.item_id.clone(),
                                source_item.source_engine.clone(),
                                source_item.source_kind,
                                (idx + 1) as u8,
                                source_item.content_ref.clone(),
                                source_item.evidence_ref.clone(),
                                source_item.sensitivity_private,
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    ordered.sort_by(|a, b| a.item_id.cmp(&b.item_id));
                    for (idx, item) in ordered.iter_mut().enumerate() {
                        item.bundle_rank = (idx + 1) as u8;
                    }

                    Ph1ContextResponse::ContextBundleBuildOk(
                        ContextBundleBuildOk::v1(
                            ReasonCodeId(1),
                            vec![ordered[0].item_id.clone()],
                            ordered,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ContextRequest::ContextBundleTrim(_r) => {
                    Ph1ContextResponse::ContextBundleTrimOk(
                        ContextBundleTrimOk::v1(
                            ReasonCodeId(2),
                            ContextValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftContextEngine;

    impl Ph1ContextEngine for DriftContextEngine {
        fn run(&self, req: &Ph1ContextRequest) -> Ph1ContextResponse {
            match req {
                Ph1ContextRequest::ContextBundleBuild(r) => {
                    let item = ContextBundleItem::v1(
                        r.source_items[0].item_id.clone(),
                        r.source_items[0].source_engine.clone(),
                        r.source_items[0].source_kind,
                        1,
                        r.source_items[0].content_ref.clone(),
                        r.source_items[0].evidence_ref.clone(),
                        r.source_items[0].sensitivity_private,
                    )
                    .unwrap();
                    Ph1ContextResponse::ContextBundleBuildOk(
                        ContextBundleBuildOk::v1(
                            ReasonCodeId(11),
                            vec![item.item_id.clone()],
                            vec![item],
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ContextRequest::ContextBundleTrim(_r) => {
                    Ph1ContextResponse::ContextBundleTrimOk(
                        ContextBundleTrimOk::v1(
                            ReasonCodeId(12),
                            ContextValidationStatus::Fail,
                            vec!["bundle_rank_sequence_gap_detected".to_string()],
                            false,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn source_item(
        id: &str,
        source_kind: ContextSourceKind,
        rank_score_bp: i16,
    ) -> ContextSourceItem {
        ContextSourceItem::v1(
            id.to_string(),
            "PH1.SUMMARY".to_string(),
            source_kind,
            rank_score_bp,
            format!("context:content:{}", id),
            format!("context:evidence:{}", id),
            false,
        )
        .unwrap()
    }

    fn input() -> ContextTurnInput {
        ContextTurnInput::v1(
            CorrelationId(9301),
            TurnId(521),
            "QUERY_WEATHER".to_string(),
            false,
            vec![source_item(
                "ctx_1",
                ContextSourceKind::SummaryEvidence,
                1200,
            )],
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_context_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1ContextWiring::new(
            Ph1ContextWiringConfig::mvp_v1(true),
            DeterministicContextEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ContextWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_context_02_os_output_is_deterministic() {
        let wiring = Ph1ContextWiring::new(
            Ph1ContextWiringConfig::mvp_v1(true),
            DeterministicContextEngine,
        )
        .unwrap();

        let out1 = wiring.run_turn(&input()).unwrap();
        let out2 = wiring.run_turn(&input()).unwrap();

        match (out1, out2) {
            (ContextWiringOutcome::Forwarded(a), ContextWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.bundle_build, b.bundle_build);
                assert_eq!(a.bundle_trim, b.bundle_trim);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_context_03_os_does_not_invoke_when_context_is_disabled() {
        let wiring = Ph1ContextWiring::new(
            Ph1ContextWiringConfig::mvp_v1(false),
            DeterministicContextEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, ContextWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_context_04_os_fails_closed_on_trim_validation_drift() {
        let wiring =
            Ph1ContextWiring::new(Ph1ContextWiringConfig::mvp_v1(true), DriftContextEngine)
                .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ContextWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.capability_id, ContextCapabilityId::ContextBundleTrim);
            }
            _ => panic!("expected Refused"),
        }
    }
}
