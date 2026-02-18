#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1pattern::{
    PatternCapabilityId, PatternMineOfflineOk, PatternMineOfflineRequest, PatternProposalEmitOk,
    PatternProposalEmitRequest, PatternRefuse, PatternRequestEnvelope, PatternSignal,
    PatternValidationStatus, Ph1PatternRequest, Ph1PatternResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PATTERN OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PATTERN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5041_0101);
    pub const PH1_PATTERN_OFFLINE_ONLY_REQUIRED: ReasonCodeId = ReasonCodeId(0x5041_0102);
    pub const PH1_PATTERN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5041_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PatternWiringConfig {
    pub pattern_enabled: bool,
    pub max_signals: u8,
    pub max_proposals: u8,
    pub offline_pipeline_only: bool,
}

impl Ph1PatternWiringConfig {
    pub fn mvp_v1(pattern_enabled: bool) -> Self {
        Self {
            pattern_enabled,
            max_signals: 32,
            max_proposals: 16,
            offline_pipeline_only: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternOfflineInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signals: Vec<PatternSignal>,
    pub analysis_window_days: u16,
    pub offline_pipeline_only: bool,
}

impl PatternOfflineInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signals: Vec<PatternSignal>,
        analysis_window_days: u16,
        offline_pipeline_only: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            signals,
            analysis_window_days,
            offline_pipeline_only,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for PatternOfflineInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.signals.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_offline_input.signals",
                reason: "must be <= 64",
            });
        }
        for signal in &self.signals {
            signal.validate()?;
        }
        if self.analysis_window_days == 0 || self.analysis_window_days > 365 {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_offline_input.analysis_window_days",
                reason: "must be within 1..=365",
            });
        }
        if !self.offline_pipeline_only {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_offline_input.offline_pipeline_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatternForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub mine_offline: PatternMineOfflineOk,
    pub proposal_emit: PatternProposalEmitOk,
}

impl PatternForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        mine_offline: PatternMineOfflineOk,
        proposal_emit: PatternProposalEmitOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            mine_offline,
            proposal_emit,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for PatternForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.mine_offline.validate()?;
        self.proposal_emit.validate()?;
        if self.proposal_emit.validation_status != PatternValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "pattern_forward_bundle.proposal_emit.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PatternWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(PatternRefuse),
    Forwarded(PatternForwardBundle),
}

pub trait Ph1PatternEngine {
    fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PatternWiring<E>
where
    E: Ph1PatternEngine,
{
    config: Ph1PatternWiringConfig,
    engine: E,
}

impl<E> Ph1PatternWiring<E>
where
    E: Ph1PatternEngine,
{
    pub fn new(config: Ph1PatternWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signals == 0 || config.max_signals > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pattern_wiring_config.max_signals",
                reason: "must be within 1..=64",
            });
        }
        if config.max_proposals == 0 || config.max_proposals > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1pattern_wiring_config.max_proposals",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_offline(
        &self,
        input: &PatternOfflineInput,
    ) -> Result<PatternWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.pattern_enabled {
            return Ok(PatternWiringOutcome::NotInvokedDisabled);
        }
        if input.signals.is_empty() {
            return Ok(PatternWiringOutcome::NotInvokedNoSignals);
        }

        if !self.config.offline_pipeline_only || !input.offline_pipeline_only {
            return Ok(PatternWiringOutcome::Refused(PatternRefuse::v1(
                PatternCapabilityId::PatternMineOffline,
                reason_codes::PH1_PATTERN_OFFLINE_ONLY_REQUIRED,
                "pattern wiring requires offline pipeline".to_string(),
            )?));
        }

        let envelope = PatternRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signals, 64),
            min(self.config.max_proposals, 32),
            true,
        )?;

        let mine_req = Ph1PatternRequest::PatternMineOffline(PatternMineOfflineRequest::v1(
            envelope.clone(),
            input.signals.clone(),
            input.analysis_window_days,
        )?);
        let mine_resp = self.engine.run(&mine_req);
        mine_resp.validate()?;

        let mine_ok = match mine_resp {
            Ph1PatternResponse::Refuse(refuse) => return Ok(PatternWiringOutcome::Refused(refuse)),
            Ph1PatternResponse::PatternMineOfflineOk(ok) => ok,
            Ph1PatternResponse::PatternProposalEmitOk(_) => {
                return Ok(PatternWiringOutcome::Refused(PatternRefuse::v1(
                    PatternCapabilityId::PatternMineOffline,
                    reason_codes::PH1_PATTERN_INTERNAL_PIPELINE_ERROR,
                    "unexpected proposal-emit response for mine request".to_string(),
                )?))
            }
        };

        let emit_req = Ph1PatternRequest::PatternProposalEmit(PatternProposalEmitRequest::v1(
            envelope,
            mine_ok.selected_proposal_id.clone(),
            mine_ok.ordered_proposals.clone(),
        )?);
        let emit_resp = self.engine.run(&emit_req);
        emit_resp.validate()?;

        let emit_ok = match emit_resp {
            Ph1PatternResponse::Refuse(refuse) => return Ok(PatternWiringOutcome::Refused(refuse)),
            Ph1PatternResponse::PatternProposalEmitOk(ok) => ok,
            Ph1PatternResponse::PatternMineOfflineOk(_) => {
                return Ok(PatternWiringOutcome::Refused(PatternRefuse::v1(
                    PatternCapabilityId::PatternProposalEmit,
                    reason_codes::PH1_PATTERN_INTERNAL_PIPELINE_ERROR,
                    "unexpected mine response for proposal-emit request".to_string(),
                )?))
            }
        };

        if emit_ok.validation_status != PatternValidationStatus::Ok {
            return Ok(PatternWiringOutcome::Refused(PatternRefuse::v1(
                PatternCapabilityId::PatternProposalEmit,
                reason_codes::PH1_PATTERN_VALIDATION_FAILED,
                "pattern proposal emission validation failed".to_string(),
            )?));
        }

        let bundle =
            PatternForwardBundle::v1(input.correlation_id, input.turn_id, mine_ok, emit_ok)?;
        Ok(PatternWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1pattern::{
        PatternMineOfflineOk, PatternProposalEmitOk, PatternProposalItem, PatternProposalTarget,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicPatternEngine;

    impl Ph1PatternEngine for DeterministicPatternEngine {
        fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
            match req {
                Ph1PatternRequest::PatternMineOffline(r) => {
                    let mut items = r
                        .signals
                        .iter()
                        .enumerate()
                        .map(|(idx, signal)| {
                            PatternProposalItem::v1(
                                format!("proposal_{}", signal.signal_id),
                                PatternProposalTarget::PaeProviderRoutingWeights,
                                (idx + 1) as u8,
                                85u8.saturating_sub((idx as u8) * 3),
                                3,
                                signal.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    items.sort_by(|a, b| {
                        b.confidence_pct
                            .cmp(&a.confidence_pct)
                            .then(a.proposal_id.cmp(&b.proposal_id))
                    });
                    for (idx, item) in items.iter_mut().enumerate() {
                        item.rank = (idx + 1) as u8;
                    }

                    Ph1PatternResponse::PatternMineOfflineOk(
                        PatternMineOfflineOk::v1(
                            ReasonCodeId(51),
                            items[0].proposal_id.clone(),
                            items,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PatternRequest::PatternProposalEmit(_r) => {
                    Ph1PatternResponse::PatternProposalEmitOk(
                        PatternProposalEmitOk::v1(
                            ReasonCodeId(52),
                            PatternValidationStatus::Ok,
                            vec![],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftPatternEngine;

    impl Ph1PatternEngine for DriftPatternEngine {
        fn run(&self, req: &Ph1PatternRequest) -> Ph1PatternResponse {
            match req {
                Ph1PatternRequest::PatternMineOffline(r) => {
                    let item = PatternProposalItem::v1(
                        format!("proposal_{}", r.signals[0].signal_id),
                        PatternProposalTarget::PaeProviderRoutingWeights,
                        1,
                        80,
                        3,
                        r.signals[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1PatternResponse::PatternMineOfflineOk(
                        PatternMineOfflineOk::v1(
                            ReasonCodeId(61),
                            item.proposal_id.clone(),
                            vec![item],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PatternRequest::PatternProposalEmit(_r) => {
                    Ph1PatternResponse::PatternProposalEmitOk(
                        PatternProposalEmitOk::v1(
                            ReasonCodeId(62),
                            PatternValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_proposals".to_string()],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn signal(
        signal_id: &str,
        metric_key: &str,
        metric_value_bp: i16,
        occurrence_count: u32,
    ) -> PatternSignal {
        PatternSignal::v1(
            signal_id.to_string(),
            "PH1.J".to_string(),
            metric_key.to_string(),
            metric_value_bp,
            occurrence_count,
            "evidence:pattern:3".to_string(),
        )
        .unwrap()
    }

    fn signals() -> Vec<PatternSignal> {
        vec![
            signal("sig_provider", "provider_fallback_rate", 240, 80),
            signal("sig_clarify", "clarify_loop_rate", 210, 70),
            signal("sig_context", "context_miss_rate", 180, 60),
        ]
    }

    #[test]
    fn at_pattern_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1PatternWiring::new(
            Ph1PatternWiringConfig::mvp_v1(true),
            DeterministicPatternEngine,
        )
        .unwrap();
        let input =
            PatternOfflineInput::v1(CorrelationId(2901), TurnId(261), signals(), 30, true).unwrap();

        let out = wiring.run_offline(&input).unwrap();
        match out {
            PatternWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.mine_offline.selected_proposal_id,
                    "proposal_sig_provider"
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_pattern_02_os_order_is_deterministic_for_same_input() {
        let wiring = Ph1PatternWiring::new(
            Ph1PatternWiringConfig::mvp_v1(true),
            DeterministicPatternEngine,
        )
        .unwrap();
        let input =
            PatternOfflineInput::v1(CorrelationId(2902), TurnId(262), signals(), 30, true).unwrap();

        let out1 = wiring.run_offline(&input).unwrap();
        let out2 = wiring.run_offline(&input).unwrap();

        let ordered1 = match out1 {
            PatternWiringOutcome::Forwarded(bundle) => bundle.mine_offline.ordered_proposals,
            _ => panic!("expected Forwarded"),
        };
        let ordered2 = match out2 {
            PatternWiringOutcome::Forwarded(bundle) => bundle.mine_offline.ordered_proposals,
            _ => panic!("expected Forwarded"),
        };

        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_pattern_03_os_does_not_invoke_when_disabled() {
        let wiring = Ph1PatternWiring::new(
            Ph1PatternWiringConfig::mvp_v1(false),
            DeterministicPatternEngine,
        )
        .unwrap();
        let input =
            PatternOfflineInput::v1(CorrelationId(2903), TurnId(263), signals(), 30, true).unwrap();

        let out = wiring.run_offline(&input).unwrap();
        assert_eq!(out, PatternWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_pattern_04_os_fails_closed_on_proposal_emit_validation_drift() {
        let wiring =
            Ph1PatternWiring::new(Ph1PatternWiringConfig::mvp_v1(true), DriftPatternEngine)
                .unwrap();
        let input =
            PatternOfflineInput::v1(CorrelationId(2904), TurnId(264), signals(), 30, true).unwrap();

        let out = wiring.run_offline(&input).unwrap();
        match out {
            PatternWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.capability_id,
                    PatternCapabilityId::PatternProposalEmit
                );
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_PATTERN_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
