#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1prefetch::{
    Ph1PrefetchRequest, Ph1PrefetchResponse, PrefetchCapabilityId, PrefetchPlanBuildOk,
    PrefetchPlanBuildRequest, PrefetchPrioritizeOk, PrefetchPrioritizeRequest, PrefetchRefuse,
    PrefetchRequestEnvelope, PrefetchValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PREFETCH OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PREFETCH_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5052_4701);
    pub const PH1_PREFETCH_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5052_47F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PrefetchWiringConfig {
    pub prefetch_enabled: bool,
    pub max_candidates: u8,
}

impl Ph1PrefetchWiringConfig {
    pub fn mvp_v1(prefetch_enabled: bool) -> Self {
        Self {
            prefetch_enabled,
            max_candidates: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub intent_type: String,
    pub locale: Option<String>,
    pub search_query_hints: Vec<String>,
    pub privacy_mode: bool,
}

impl PrefetchTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        intent_type: String,
        locale: Option<String>,
        search_query_hints: Vec<String>,
        privacy_mode: bool,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            intent_type,
            locale,
            search_query_hints,
            privacy_mode,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for PrefetchTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.intent_type.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_turn_input.intent_type",
                reason: "must be <= 96 chars",
            });
        }
        if self.intent_type.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_turn_input.intent_type",
                reason: "must not contain control characters",
            });
        }
        if let Some(locale) = &self.locale {
            if locale.len() > 32 || locale.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "prefetch_turn_input.locale",
                    reason: "must be <= 32 chars and contain no control characters",
                });
            }
        }
        if self.search_query_hints.len() > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_turn_input.search_query_hints",
                reason: "must be <= 8",
            });
        }
        for search_query_hint in &self.search_query_hints {
            if search_query_hint.len() > 256 || search_query_hint.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "prefetch_turn_input.search_query_hints",
                    reason: "entries must be <= 256 chars and contain no control characters",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrefetchForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub plan_build: PrefetchPlanBuildOk,
    pub prioritize: PrefetchPrioritizeOk,
}

impl PrefetchForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        plan_build: PrefetchPlanBuildOk,
        prioritize: PrefetchPrioritizeOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            plan_build,
            prioritize,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for PrefetchForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.plan_build.validate()?;
        self.prioritize.validate()?;
        if self.prioritize.validation_status != PrefetchValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "prefetch_forward_bundle.prioritize.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefetchWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoPrefetchInput,
    Refused(PrefetchRefuse),
    Forwarded(PrefetchForwardBundle),
}

pub trait Ph1PrefetchEngine {
    fn run(&self, req: &Ph1PrefetchRequest) -> Ph1PrefetchResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PrefetchWiring<E>
where
    E: Ph1PrefetchEngine,
{
    config: Ph1PrefetchWiringConfig,
    engine: E,
}

impl<E> Ph1PrefetchWiring<E>
where
    E: Ph1PrefetchEngine,
{
    pub fn new(config: Ph1PrefetchWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_candidates == 0 || config.max_candidates > 8 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1prefetch_wiring_config.max_candidates",
                reason: "must be within 1..=8",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &PrefetchTurnInput,
    ) -> Result<PrefetchWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.prefetch_enabled {
            return Ok(PrefetchWiringOutcome::NotInvokedDisabled);
        }

        if input.intent_type.trim().is_empty() {
            return Ok(PrefetchWiringOutcome::NotInvokedNoPrefetchInput);
        }

        let envelope = PrefetchRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_candidates, 8),
        )?;

        let plan_req = Ph1PrefetchRequest::PrefetchPlanBuild(PrefetchPlanBuildRequest::v1(
            envelope.clone(),
            input.intent_type.clone(),
            input.locale.clone(),
            input.search_query_hints.clone(),
            true,
            input.privacy_mode,
        )?);
        let plan_resp = self.engine.run(&plan_req);
        plan_resp.validate()?;

        let plan_ok = match plan_resp {
            Ph1PrefetchResponse::Refuse(r) => return Ok(PrefetchWiringOutcome::Refused(r)),
            Ph1PrefetchResponse::PrefetchPlanBuildOk(ok) => ok,
            Ph1PrefetchResponse::PrefetchPrioritizeOk(_) => {
                return Ok(PrefetchWiringOutcome::Refused(PrefetchRefuse::v1(
                    PrefetchCapabilityId::PrefetchPlanBuild,
                    reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                    "unexpected prioritize response for plan-build request".to_string(),
                )?))
            }
        };

        let prioritize_req = Ph1PrefetchRequest::PrefetchPrioritize(PrefetchPrioritizeRequest::v1(
            envelope,
            input.intent_type.clone(),
            input.locale.clone(),
            input.search_query_hints.clone(),
            true,
            input.privacy_mode,
            plan_ok.candidates.clone(),
        )?);
        let prioritize_resp = self.engine.run(&prioritize_req);
        prioritize_resp.validate()?;

        let prioritize_ok = match prioritize_resp {
            Ph1PrefetchResponse::Refuse(r) => return Ok(PrefetchWiringOutcome::Refused(r)),
            Ph1PrefetchResponse::PrefetchPrioritizeOk(ok) => ok,
            Ph1PrefetchResponse::PrefetchPlanBuildOk(_) => {
                return Ok(PrefetchWiringOutcome::Refused(PrefetchRefuse::v1(
                    PrefetchCapabilityId::PrefetchPrioritize,
                    reason_codes::PH1_PREFETCH_INTERNAL_PIPELINE_ERROR,
                    "unexpected plan-build response for prioritize request".to_string(),
                )?))
            }
        };

        if prioritize_ok.validation_status != PrefetchValidationStatus::Ok {
            return Ok(PrefetchWiringOutcome::Refused(PrefetchRefuse::v1(
                PrefetchCapabilityId::PrefetchPrioritize,
                reason_codes::PH1_PREFETCH_VALIDATION_FAILED,
                "prefetch prioritize validation failed".to_string(),
            )?));
        }

        let bundle =
            PrefetchForwardBundle::v1(input.correlation_id, input.turn_id, plan_ok, prioritize_ok)?;
        Ok(PrefetchWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1prefetch::{PrefetchCandidate, PrefetchToolKind};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicPrefetchEngine;

    impl Ph1PrefetchEngine for DeterministicPrefetchEngine {
        fn run(&self, req: &Ph1PrefetchRequest) -> Ph1PrefetchResponse {
            match req {
                Ph1PrefetchRequest::PrefetchPlanBuild(_r) => {
                    let candidates = vec![
                        PrefetchCandidate::v1(
                            "pf_00_weather".to_string(),
                            PrefetchToolKind::Weather,
                            "weather in singapore".to_string(),
                            300,
                            9000,
                            "pf_t131_0_weather".to_string(),
                        )
                        .unwrap(),
                        PrefetchCandidate::v1(
                            "pf_01_time".to_string(),
                            PrefetchToolKind::Time,
                            "current local time".to_string(),
                            30,
                            7000,
                            "pf_t131_1_time".to_string(),
                        )
                        .unwrap(),
                    ];
                    Ph1PrefetchResponse::PrefetchPlanBuildOk(
                        PrefetchPlanBuildOk::v1(ReasonCodeId(1), candidates, true).unwrap(),
                    )
                }
                Ph1PrefetchRequest::PrefetchPrioritize(_r) => {
                    Ph1PrefetchResponse::PrefetchPrioritizeOk(
                        PrefetchPrioritizeOk::v1(
                            ReasonCodeId(2),
                            PrefetchValidationStatus::Ok,
                            vec!["pf_00_weather".to_string(), "pf_01_time".to_string()],
                            vec![],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    struct DriftPrefetchEngine;

    impl Ph1PrefetchEngine for DriftPrefetchEngine {
        fn run(&self, req: &Ph1PrefetchRequest) -> Ph1PrefetchResponse {
            match req {
                Ph1PrefetchRequest::PrefetchPlanBuild(_r) => {
                    let candidates = vec![PrefetchCandidate::v1(
                        "pf_00_weather".to_string(),
                        PrefetchToolKind::Weather,
                        "weather in singapore".to_string(),
                        300,
                        9000,
                        "pf_t131_0_weather".to_string(),
                    )
                    .unwrap()];
                    Ph1PrefetchResponse::PrefetchPlanBuildOk(
                        PrefetchPlanBuildOk::v1(ReasonCodeId(10), candidates, true).unwrap(),
                    )
                }
                Ph1PrefetchRequest::PrefetchPrioritize(_r) => {
                    Ph1PrefetchResponse::PrefetchPrioritizeOk(
                        PrefetchPrioritizeOk::v1(
                            ReasonCodeId(11),
                            PrefetchValidationStatus::Fail,
                            vec!["pf_00_weather".to_string()],
                            vec!["pf_00_weather_ttl_seconds_mismatch".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    #[test]
    fn at_prefetch_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring = Ph1PrefetchWiring::new(
            Ph1PrefetchWiringConfig::mvp_v1(true),
            DeterministicPrefetchEngine,
        )
        .unwrap();

        let input = PrefetchTurnInput::v1(
            CorrelationId(1701),
            TurnId(131),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec!["weather in singapore".to_string()],
            false,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PrefetchWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(bundle.prioritize.prioritized_candidate_ids.len(), 2);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_prefetch_02_os_preserves_prioritized_order_for_downstream_scheduler() {
        let wiring = Ph1PrefetchWiring::new(
            Ph1PrefetchWiringConfig::mvp_v1(true),
            DeterministicPrefetchEngine,
        )
        .unwrap();

        let input = PrefetchTurnInput::v1(
            CorrelationId(1702),
            TurnId(132),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec!["weather in singapore".to_string()],
            false,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PrefetchWiringOutcome::Forwarded(bundle) => {
                assert_eq!(
                    bundle.prioritize.prioritized_candidate_ids,
                    vec!["pf_00_weather".to_string(), "pf_01_time".to_string()]
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_prefetch_03_os_does_not_invoke_when_prefetch_is_disabled() {
        let wiring = Ph1PrefetchWiring::new(
            Ph1PrefetchWiringConfig::mvp_v1(false),
            DeterministicPrefetchEngine,
        )
        .unwrap();

        let input = PrefetchTurnInput::v1(
            CorrelationId(1703),
            TurnId(133),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec!["weather in singapore".to_string()],
            false,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, PrefetchWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_prefetch_04_os_fails_closed_on_prioritize_validation_drift() {
        let wiring =
            Ph1PrefetchWiring::new(Ph1PrefetchWiringConfig::mvp_v1(true), DriftPrefetchEngine)
                .unwrap();

        let input = PrefetchTurnInput::v1(
            CorrelationId(1704),
            TurnId(134),
            "QUERY_WEATHER".to_string(),
            Some("en-US".to_string()),
            vec!["weather in singapore".to_string()],
            false,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PrefetchWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_PREFETCH_VALIDATION_FAILED);
                assert_eq!(r.capability_id, PrefetchCapabilityId::PrefetchPrioritize);
            }
            _ => panic!("expected Refused"),
        }
    }
}
