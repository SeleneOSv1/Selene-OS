#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1endpoint::{
    EndpointBoundaryScoreOk, EndpointBoundaryScoreRequest, EndpointCapabilityId,
    EndpointHintsBuildOk, EndpointHintsBuildRequest, EndpointRefuse, EndpointRequestEnvelope,
    EndpointVadWindow, EndpointValidationStatus, Ph1EndpointRequest, Ph1EndpointResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.ENDPOINT OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_ENDPOINT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x454E_0101);
    pub const PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x454E_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1EndpointWiringConfig {
    pub endpoint_enabled: bool,
    pub max_vad_windows: u8,
}

impl Ph1EndpointWiringConfig {
    pub fn mvp_v1(endpoint_enabled: bool) -> Self {
        Self {
            endpoint_enabled,
            max_vad_windows: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub vad_windows: Vec<EndpointVadWindow>,
    pub transcript_token_estimate: u16,
    pub tts_playback_active: bool,
    pub previous_selected_segment_id: Option<String>,
}

impl EndpointTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        vad_windows: Vec<EndpointVadWindow>,
        transcript_token_estimate: u16,
        tts_playback_active: bool,
        previous_selected_segment_id: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            vad_windows,
            transcript_token_estimate,
            tts_playback_active,
            previous_selected_segment_id,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for EndpointTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.vad_windows.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_turn_input.vad_windows",
                reason: "must be <= 32 windows",
            });
        }
        for window in &self.vad_windows {
            window.validate()?;
        }
        if let Some(previous) = &self.previous_selected_segment_id {
            if previous.trim().is_empty()
                || previous.len() > 64
                || previous.chars().any(|c| c.is_control())
            {
                return Err(ContractViolation::InvalidValue {
                    field: "endpoint_turn_input.previous_selected_segment_id",
                    reason: "must be non-empty bounded token",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EndpointForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub hints_build: EndpointHintsBuildOk,
    pub boundary_score: EndpointBoundaryScoreOk,
}

impl EndpointForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        hints_build: EndpointHintsBuildOk,
        boundary_score: EndpointBoundaryScoreOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            hints_build,
            boundary_score,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for EndpointForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.hints_build.validate()?;
        self.boundary_score.validate()?;
        if self.boundary_score.validation_status != EndpointValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "endpoint_forward_bundle.boundary_score.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EndpointWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoVadWindows,
    Refused(EndpointRefuse),
    Forwarded(EndpointForwardBundle),
}

pub trait Ph1EndpointEngine {
    fn run(&self, req: &Ph1EndpointRequest) -> Ph1EndpointResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1EndpointWiring<E>
where
    E: Ph1EndpointEngine,
{
    config: Ph1EndpointWiringConfig,
    engine: E,
}

impl<E> Ph1EndpointWiring<E>
where
    E: Ph1EndpointEngine,
{
    pub fn new(config: Ph1EndpointWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_vad_windows == 0 || config.max_vad_windows > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1endpoint_wiring_config.max_vad_windows",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &EndpointTurnInput,
    ) -> Result<EndpointWiringOutcome, ContractViolation> {
        input.validate()?;
        if !self.config.endpoint_enabled {
            return Ok(EndpointWiringOutcome::NotInvokedDisabled);
        }
        if input.vad_windows.is_empty() {
            return Ok(EndpointWiringOutcome::NotInvokedNoVadWindows);
        }

        let envelope = EndpointRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_vad_windows, 32),
        )?;

        let build_req = Ph1EndpointRequest::EndpointHintsBuild(EndpointHintsBuildRequest::v1(
            envelope.clone(),
            input.vad_windows.clone(),
            input.transcript_token_estimate,
            input.tts_playback_active,
        )?);
        let build_resp = self.engine.run(&build_req);
        build_resp.validate()?;

        let build_ok = match build_resp {
            Ph1EndpointResponse::Refuse(refuse) => {
                return Ok(EndpointWiringOutcome::Refused(refuse))
            }
            Ph1EndpointResponse::EndpointHintsBuildOk(ok) => ok,
            Ph1EndpointResponse::EndpointBoundaryScoreOk(_) => {
                return Ok(EndpointWiringOutcome::Refused(EndpointRefuse::v1(
                    EndpointCapabilityId::EndpointHintsBuild,
                    reason_codes::PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR,
                    "unexpected boundary-score response for hints-build request".to_string(),
                )?))
            }
        };

        let validate_req =
            Ph1EndpointRequest::EndpointBoundaryScore(EndpointBoundaryScoreRequest::v1(
                envelope,
                build_ok.selected_segment_id.clone(),
                build_ok.ordered_segment_hints.clone(),
                input.previous_selected_segment_id.clone(),
            )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1EndpointResponse::Refuse(refuse) => {
                return Ok(EndpointWiringOutcome::Refused(refuse))
            }
            Ph1EndpointResponse::EndpointBoundaryScoreOk(ok) => ok,
            Ph1EndpointResponse::EndpointHintsBuildOk(_) => {
                return Ok(EndpointWiringOutcome::Refused(EndpointRefuse::v1(
                    EndpointCapabilityId::EndpointBoundaryScore,
                    reason_codes::PH1_ENDPOINT_INTERNAL_PIPELINE_ERROR,
                    "unexpected hints-build response for boundary-score request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != EndpointValidationStatus::Ok {
            return Ok(EndpointWiringOutcome::Refused(EndpointRefuse::v1(
                EndpointCapabilityId::EndpointBoundaryScore,
                reason_codes::PH1_ENDPOINT_VALIDATION_FAILED,
                "endpoint boundary-score validation failed".to_string(),
            )?));
        }

        let bundle =
            EndpointForwardBundle::v1(input.correlation_id, input.turn_id, build_ok, validate_ok)?;
        Ok(EndpointWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1endpoint::{
        EndpointBoundaryScoreOk, EndpointConfidenceBucket, EndpointHintsBuildOk,
        EndpointSegmentHint,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicEndpointEngine;

    impl Ph1EndpointEngine for DeterministicEndpointEngine {
        fn run(&self, req: &Ph1EndpointRequest) -> Ph1EndpointResponse {
            match req {
                Ph1EndpointRequest::EndpointHintsBuild(_r) => {
                    let hint = EndpointSegmentHint::v1(
                        "endpoint_segment_01".to_string(),
                        "window_1".to_string(),
                        0,
                        420,
                        "silence_window".to_string(),
                        90,
                        EndpointConfidenceBucket::High,
                        true,
                    )
                    .unwrap();
                    Ph1EndpointResponse::EndpointHintsBuildOk(
                        EndpointHintsBuildOk::v1(
                            ReasonCodeId(1),
                            "endpoint_segment_01".to_string(),
                            vec![hint],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1EndpointRequest::EndpointBoundaryScore(_r) => {
                    Ph1EndpointResponse::EndpointBoundaryScoreOk(
                        EndpointBoundaryScoreOk::v1(
                            ReasonCodeId(2),
                            EndpointValidationStatus::Ok,
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

    struct DriftEndpointEngine;

    impl Ph1EndpointEngine for DriftEndpointEngine {
        fn run(&self, req: &Ph1EndpointRequest) -> Ph1EndpointResponse {
            match req {
                Ph1EndpointRequest::EndpointHintsBuild(_r) => {
                    let hint = EndpointSegmentHint::v1(
                        "endpoint_segment_01".to_string(),
                        "window_1".to_string(),
                        0,
                        420,
                        "voiced_window".to_string(),
                        74,
                        EndpointConfidenceBucket::Medium,
                        false,
                    )
                    .unwrap();
                    Ph1EndpointResponse::EndpointHintsBuildOk(
                        EndpointHintsBuildOk::v1(
                            ReasonCodeId(10),
                            "endpoint_segment_01".to_string(),
                            vec![hint],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1EndpointRequest::EndpointBoundaryScore(_r) => {
                    Ph1EndpointResponse::EndpointBoundaryScoreOk(
                        EndpointBoundaryScoreOk::v1(
                            ReasonCodeId(11),
                            EndpointValidationStatus::Fail,
                            vec!["selected_not_first_in_ordered_hints".to_string()],
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn window() -> EndpointVadWindow {
        EndpointVadWindow::v1("window_1".to_string(), 0, 420, 90, 88, 300).unwrap()
    }

    #[test]
    fn at_endpoint_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1EndpointWiring::new(
            Ph1EndpointWiringConfig::mvp_v1(true),
            DeterministicEndpointEngine,
        )
        .unwrap();
        let input = EndpointTurnInput::v1(
            CorrelationId(1701),
            TurnId(131),
            vec![window()],
            20,
            false,
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            EndpointWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.hints_build.selected_segment_id,
                    "endpoint_segment_01".to_string()
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_endpoint_02_os_preserves_selected_hint_for_ph1c_handoff() {
        let wiring = Ph1EndpointWiring::new(
            Ph1EndpointWiringConfig::mvp_v1(true),
            DeterministicEndpointEngine,
        )
        .unwrap();
        let input = EndpointTurnInput::v1(
            CorrelationId(1702),
            TurnId(132),
            vec![window()],
            20,
            false,
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            EndpointWiringOutcome::Forwarded(bundle) => {
                assert_eq!(
                    bundle.hints_build.selected_segment_id,
                    bundle.hints_build.ordered_segment_hints[0].segment_id
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_endpoint_03_os_does_not_invoke_when_endpoint_is_disabled() {
        let wiring = Ph1EndpointWiring::new(
            Ph1EndpointWiringConfig::mvp_v1(false),
            DeterministicEndpointEngine,
        )
        .unwrap();
        let input = EndpointTurnInput::v1(
            CorrelationId(1703),
            TurnId(133),
            vec![window()],
            20,
            false,
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, EndpointWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_endpoint_04_os_fails_closed_on_boundary_score_validation_drift() {
        let wiring =
            Ph1EndpointWiring::new(Ph1EndpointWiringConfig::mvp_v1(true), DriftEndpointEngine)
                .unwrap();
        let input = EndpointTurnInput::v1(
            CorrelationId(1704),
            TurnId(134),
            vec![window()],
            20,
            false,
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            EndpointWiringOutcome::Refused(refuse) => {
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_ENDPOINT_VALIDATION_FAILED
                );
                assert_eq!(
                    refuse.capability_id,
                    EndpointCapabilityId::EndpointBoundaryScore
                );
            }
            _ => panic!("expected Refused"),
        }
    }
}
