#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1listen::{
    ListenCapabilityId, ListenCorrectionSnapshot, ListenRefuse, ListenRequestEnvelope,
    ListenSessionContext, ListenSignalCollectOk, ListenSignalCollectRequest, ListenSignalFilterOk,
    ListenSignalFilterRequest, ListenSignalWindow, ListenValidationStatus, Ph1ListenRequest,
    Ph1ListenResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.LISTEN OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_LISTEN_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4C49_0101);
    pub const PH1_LISTEN_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4C49_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ListenWiringConfig {
    pub listen_enabled: bool,
    pub max_signal_windows: u8,
    pub max_adjustments: u8,
    pub max_diagnostics: u8,
}

impl Ph1ListenWiringConfig {
    pub fn mvp_v1(listen_enabled: bool) -> Self {
        Self {
            listen_enabled,
            max_signal_windows: 24,
            max_adjustments: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_windows: Vec<ListenSignalWindow>,
    pub correction_snapshot: ListenCorrectionSnapshot,
    pub session_context: ListenSessionContext,
}

impl ListenTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_windows: Vec<ListenSignalWindow>,
        correction_snapshot: ListenCorrectionSnapshot,
        session_context: ListenSessionContext,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            signal_windows,
            correction_snapshot,
            session_context,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for ListenTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.correction_snapshot.validate()?;
        self.session_context.validate()?;
        if self.signal_windows.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "listen_turn_input.signal_windows",
                reason: "must be <= 64",
            });
        }
        for signal_window in &self.signal_windows {
            signal_window.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub signal_collect: ListenSignalCollectOk,
    pub signal_filter: ListenSignalFilterOk,
}

impl ListenForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        signal_collect: ListenSignalCollectOk,
        signal_filter: ListenSignalFilterOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            signal_collect,
            signal_filter,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for ListenForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.signal_collect.validate()?;
        self.signal_filter.validate()?;
        if self.signal_filter.validation_status != ListenValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "listen_forward_bundle.signal_filter.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ListenWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoSignals,
    Refused(ListenRefuse),
    Forwarded(ListenForwardBundle),
}

pub trait Ph1ListenEngine {
    fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1ListenWiring<E>
where
    E: Ph1ListenEngine,
{
    config: Ph1ListenWiringConfig,
    engine: E,
}

impl<E> Ph1ListenWiring<E>
where
    E: Ph1ListenEngine,
{
    pub fn new(config: Ph1ListenWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_signal_windows == 0 || config.max_signal_windows > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_signal_windows",
                reason: "must be within 1..=64",
            });
        }
        if config.max_adjustments == 0 || config.max_adjustments > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_adjustments",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1listen_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &ListenTurnInput,
    ) -> Result<ListenWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.listen_enabled {
            return Ok(ListenWiringOutcome::NotInvokedDisabled);
        }
        if input.signal_windows.is_empty() {
            return Ok(ListenWiringOutcome::NotInvokedNoSignals);
        }

        let envelope = ListenRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_signal_windows, 64),
            min(self.config.max_adjustments, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let collect_req = Ph1ListenRequest::ListenSignalCollect(ListenSignalCollectRequest::v1(
            envelope.clone(),
            input.signal_windows.clone(),
            input.correction_snapshot.clone(),
            input.session_context.clone(),
        )?);
        let collect_resp = self.engine.run(&collect_req);
        collect_resp.validate()?;

        let collect_ok = match collect_resp {
            Ph1ListenResponse::Refuse(refuse) => return Ok(ListenWiringOutcome::Refused(refuse)),
            Ph1ListenResponse::ListenSignalCollectOk(ok) => ok,
            Ph1ListenResponse::ListenSignalFilterOk(_) => {
                return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                    ListenCapabilityId::ListenSignalCollect,
                    reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-filter response for signal-collect request".to_string(),
                )?));
            }
        };

        let filter_req = Ph1ListenRequest::ListenSignalFilter(ListenSignalFilterRequest::v1(
            envelope,
            collect_ok.environment_profile_ref.clone(),
            collect_ok.selected_adjustment_id.clone(),
            collect_ok.ordered_adjustments.clone(),
            true,
        )?);
        let filter_resp = self.engine.run(&filter_req);
        filter_resp.validate()?;

        let filter_ok = match filter_resp {
            Ph1ListenResponse::Refuse(refuse) => return Ok(ListenWiringOutcome::Refused(refuse)),
            Ph1ListenResponse::ListenSignalFilterOk(ok) => ok,
            Ph1ListenResponse::ListenSignalCollectOk(_) => {
                return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                    ListenCapabilityId::ListenSignalFilter,
                    reason_codes::PH1_LISTEN_INTERNAL_PIPELINE_ERROR,
                    "unexpected signal-collect response for signal-filter request".to_string(),
                )?));
            }
        };

        if filter_ok.validation_status != ListenValidationStatus::Ok {
            return Ok(ListenWiringOutcome::Refused(ListenRefuse::v1(
                ListenCapabilityId::ListenSignalFilter,
                reason_codes::PH1_LISTEN_VALIDATION_FAILED,
                "listen signal-filter validation failed".to_string(),
            )?));
        }

        let bundle =
            ListenForwardBundle::v1(input.correlation_id, input.turn_id, collect_ok, filter_ok)?;
        Ok(ListenWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1listen::{
        ListenAdjustmentHint, ListenCaptureProfile, ListenDeliveryPolicyHint,
        ListenEndpointProfile, ListenEnvironmentMode,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicListenEngine;

    impl Ph1ListenEngine for DeterministicListenEngine {
        fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse {
            match req {
                Ph1ListenRequest::ListenSignalCollect(r) => {
                    let mut adjustments = r
                        .signal_windows
                        .iter()
                        .enumerate()
                        .map(|(idx, window)| {
                            ListenAdjustmentHint::v1(
                                format!("adj_{}", window.window_id),
                                if r.session_context.session_mode_meeting {
                                    ListenEnvironmentMode::Meeting
                                } else {
                                    ListenEnvironmentMode::Office
                                },
                                ListenCaptureProfile::Standard,
                                ListenEndpointProfile::Balanced,
                                ListenDeliveryPolicyHint::VoicePreferred,
                                1000 - (idx as i16 * 100),
                                window.window_id.clone(),
                                window.evidence_ref.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    adjustments.sort_by(|a, b| {
                        b.priority_bp
                            .cmp(&a.priority_bp)
                            .then(a.adjustment_id.cmp(&b.adjustment_id))
                    });

                    Ph1ListenResponse::ListenSignalCollectOk(
                        ListenSignalCollectOk::v1(
                            ReasonCodeId(201),
                            "env:office:standard:voice_preferred".to_string(),
                            adjustments[0].adjustment_id.clone(),
                            adjustments,
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ListenRequest::ListenSignalFilter(_r) => {
                    Ph1ListenResponse::ListenSignalFilterOk(
                        ListenSignalFilterOk::v1(
                            ReasonCodeId(202),
                            ListenValidationStatus::Ok,
                            vec![],
                            true,
                            true,
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

    struct DriftListenEngine;

    impl Ph1ListenEngine for DriftListenEngine {
        fn run(&self, req: &Ph1ListenRequest) -> Ph1ListenResponse {
            match req {
                Ph1ListenRequest::ListenSignalCollect(r) => {
                    let adjustment = ListenAdjustmentHint::v1(
                        format!("adj_{}", r.signal_windows[0].window_id),
                        ListenEnvironmentMode::Noisy,
                        ListenCaptureProfile::NoiseSuppressed,
                        ListenEndpointProfile::Balanced,
                        ListenDeliveryPolicyHint::TextPreferred,
                        900,
                        r.signal_windows[0].window_id.clone(),
                        r.signal_windows[0].evidence_ref.clone(),
                    )
                    .unwrap();
                    Ph1ListenResponse::ListenSignalCollectOk(
                        ListenSignalCollectOk::v1(
                            ReasonCodeId(211),
                            "env:noisy:noise_suppressed:text_preferred".to_string(),
                            adjustment.adjustment_id.clone(),
                            vec![adjustment],
                            true,
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ListenRequest::ListenSignalFilter(_r) => {
                    Ph1ListenResponse::ListenSignalFilterOk(
                        ListenSignalFilterOk::v1(
                            ReasonCodeId(212),
                            ListenValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_adjustments".to_string()],
                            true,
                            true,
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

    fn window(id: &str, noise_level_dbfs: i16) -> ListenSignalWindow {
        ListenSignalWindow::v1(
            id.to_string(),
            "PH1.K".to_string(),
            8200,
            7700,
            noise_level_dbfs,
            0,
            420,
            format!("listen:evidence:{}", id),
        )
        .unwrap()
    }

    fn correction() -> ListenCorrectionSnapshot {
        ListenCorrectionSnapshot::v1(2, 1, 1, 1400).unwrap()
    }

    fn context() -> ListenSessionContext {
        ListenSessionContext::v1(false, false, false, false).unwrap()
    }

    fn input() -> ListenTurnInput {
        ListenTurnInput::v1(
            CorrelationId(3501),
            TurnId(321),
            vec![window("w_1", -30), window("w_2", -45)],
            correction(),
            context(),
        )
        .unwrap()
    }

    #[test]
    fn at_listen_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(true),
            DeterministicListenEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ListenWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_listen_02_os_output_is_deterministic() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(true),
            DeterministicListenEngine,
        )
        .unwrap();

        let out1 = wiring.run_turn(&input()).unwrap();
        let out2 = wiring.run_turn(&input()).unwrap();

        match (out1, out2) {
            (ListenWiringOutcome::Forwarded(a), ListenWiringOutcome::Forwarded(b)) => {
                assert_eq!(a.signal_collect, b.signal_collect);
                assert_eq!(a.signal_filter, b.signal_filter);
            }
            _ => panic!("expected Forwarded outcomes"),
        }
    }

    #[test]
    fn at_listen_03_os_does_not_invoke_when_listen_is_disabled() {
        let wiring = Ph1ListenWiring::new(
            Ph1ListenWiringConfig::mvp_v1(false),
            DeterministicListenEngine,
        )
        .unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, ListenWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_listen_04_os_fails_closed_on_signal_filter_validation_drift() {
        let wiring =
            Ph1ListenWiring::new(Ph1ListenWiringConfig::mvp_v1(true), DriftListenEngine).unwrap();

        let out = wiring.run_turn(&input()).unwrap();
        match out {
            ListenWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_LISTEN_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
