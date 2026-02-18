#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1diag::{
    DiagCapabilityId, DiagConsistencyCheckOk, DiagConsistencyCheckRequest, DiagDeliveryMode,
    DiagReasonSetBuildOk, DiagReasonSetBuildRequest, DiagRefuse, DiagRequestEnvelope,
    DiagValidationStatus, Ph1DiagRequest, Ph1DiagResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DIAG OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_DIAG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4449_0101);
    pub const PH1_DIAG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4449_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1DiagWiringConfig {
    pub diag_enabled: bool,
    pub max_flags: u8,
}

impl Ph1DiagWiringConfig {
    pub fn mvp_v1(diag_enabled: bool) -> Self {
        Self {
            diag_enabled,
            max_flags: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub intent_type: String,
    pub required_fields_missing: Vec<String>,
    pub ambiguity_flags: Vec<String>,
    pub requires_confirmation: bool,
    pub confirmation_received: bool,
    pub privacy_mode: bool,
    pub delivery_mode_requested: DiagDeliveryMode,
    pub sensitive_memory_candidate_present: bool,
    pub memory_permission_granted: bool,
}

impl DiagTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        intent_type: String,
        required_fields_missing: Vec<String>,
        ambiguity_flags: Vec<String>,
        requires_confirmation: bool,
        confirmation_received: bool,
        privacy_mode: bool,
        delivery_mode_requested: DiagDeliveryMode,
        sensitive_memory_candidate_present: bool,
        memory_permission_granted: bool,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            intent_type,
            required_fields_missing,
            ambiguity_flags,
            requires_confirmation,
            confirmation_received,
            privacy_mode,
            delivery_mode_requested,
            sensitive_memory_candidate_present,
            memory_permission_granted,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for DiagTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.intent_type.len() > 96 {
            return Err(ContractViolation::InvalidValue {
                field: "diag_turn_input.intent_type",
                reason: "must be <= 96 chars",
            });
        }
        if self.intent_type.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "diag_turn_input.intent_type",
                reason: "must not contain control characters",
            });
        }
        if self.required_fields_missing.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "diag_turn_input.required_fields_missing",
                reason: "must be <= 16",
            });
        }
        for field_name in &self.required_fields_missing {
            if field_name.len() > 64 || field_name.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "diag_turn_input.required_fields_missing",
                    reason: "entries must be <= 64 chars and contain no control characters",
                });
            }
        }
        if self.ambiguity_flags.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "diag_turn_input.ambiguity_flags",
                reason: "must be <= 16",
            });
        }
        for ambiguity_flag in &self.ambiguity_flags {
            if ambiguity_flag.len() > 64 || ambiguity_flag.chars().any(|c| c.is_control()) {
                return Err(ContractViolation::InvalidValue {
                    field: "diag_turn_input.ambiguity_flags",
                    reason: "entries must be <= 64 chars and contain no control characters",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub consistency_check: DiagConsistencyCheckOk,
    pub reason_set_build: DiagReasonSetBuildOk,
}

impl DiagForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        consistency_check: DiagConsistencyCheckOk,
        reason_set_build: DiagReasonSetBuildOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            consistency_check,
            reason_set_build,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for DiagForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.consistency_check.validate()?;
        self.reason_set_build.validate()?;
        if self.reason_set_build.validation_status != DiagValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "diag_forward_bundle.reason_set_build.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiagWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoDiagInput,
    Refused(DiagRefuse),
    Forwarded(DiagForwardBundle),
}

pub trait Ph1DiagEngine {
    fn run(&self, req: &Ph1DiagRequest) -> Ph1DiagResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1DiagWiring<E>
where
    E: Ph1DiagEngine,
{
    config: Ph1DiagWiringConfig,
    engine: E,
}

impl<E> Ph1DiagWiring<E>
where
    E: Ph1DiagEngine,
{
    pub fn new(config: Ph1DiagWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_flags == 0 || config.max_flags > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1diag_wiring_config.max_flags",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &DiagTurnInput) -> Result<DiagWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.diag_enabled {
            return Ok(DiagWiringOutcome::NotInvokedDisabled);
        }
        if input.intent_type.trim().is_empty() {
            return Ok(DiagWiringOutcome::NotInvokedNoDiagInput);
        }

        let envelope = DiagRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_flags, 16),
        )?;

        let consistency_req =
            Ph1DiagRequest::DiagConsistencyCheck(DiagConsistencyCheckRequest::v1(
                envelope.clone(),
                input.intent_type.clone(),
                input.required_fields_missing.clone(),
                input.ambiguity_flags.clone(),
                input.requires_confirmation,
                input.confirmation_received,
                input.privacy_mode,
                input.delivery_mode_requested,
                input.sensitive_memory_candidate_present,
                input.memory_permission_granted,
            )?);
        let consistency_resp = self.engine.run(&consistency_req);
        consistency_resp.validate()?;

        let consistency_ok = match consistency_resp {
            Ph1DiagResponse::Refuse(r) => return Ok(DiagWiringOutcome::Refused(r)),
            Ph1DiagResponse::DiagConsistencyCheckOk(ok) => ok,
            Ph1DiagResponse::DiagReasonSetBuildOk(_) => {
                return Ok(DiagWiringOutcome::Refused(DiagRefuse::v1(
                    DiagCapabilityId::DiagConsistencyCheck,
                    reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                    "unexpected reason-set response for consistency-check request".to_string(),
                )?))
            }
        };

        let reason_set_req = Ph1DiagRequest::DiagReasonSetBuild(DiagReasonSetBuildRequest::v1(
            envelope,
            input.intent_type.clone(),
            input.required_fields_missing.clone(),
            input.ambiguity_flags.clone(),
            input.requires_confirmation,
            input.confirmation_received,
            input.privacy_mode,
            input.delivery_mode_requested,
            input.sensitive_memory_candidate_present,
            input.memory_permission_granted,
            consistency_ok.diagnostic_flags.clone(),
        )?);
        let reason_set_resp = self.engine.run(&reason_set_req);
        reason_set_resp.validate()?;

        let reason_set_ok = match reason_set_resp {
            Ph1DiagResponse::Refuse(r) => return Ok(DiagWiringOutcome::Refused(r)),
            Ph1DiagResponse::DiagReasonSetBuildOk(ok) => ok,
            Ph1DiagResponse::DiagConsistencyCheckOk(_) => {
                return Ok(DiagWiringOutcome::Refused(DiagRefuse::v1(
                    DiagCapabilityId::DiagReasonSetBuild,
                    reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                    "unexpected consistency-check response for reason-set request".to_string(),
                )?))
            }
        };

        if reason_set_ok.validation_status != DiagValidationStatus::Ok {
            return Ok(DiagWiringOutcome::Refused(DiagRefuse::v1(
                DiagCapabilityId::DiagReasonSetBuild,
                reason_codes::PH1_DIAG_VALIDATION_FAILED,
                "diag reason-set validation failed".to_string(),
            )?));
        }

        let bundle = DiagForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            consistency_ok,
            reason_set_ok,
        )?;
        Ok(DiagWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1diag::{DiagCheckArea, DiagDiagnosticFlag};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicDiagEngine;

    impl Ph1DiagEngine for DeterministicDiagEngine {
        fn run(&self, req: &Ph1DiagRequest) -> Ph1DiagResponse {
            match req {
                Ph1DiagRequest::DiagConsistencyCheck(_r) => {
                    let flags = vec![
                        DiagDiagnosticFlag::v1(
                            "clarify_missing_field".to_string(),
                            DiagCheckArea::RequiredField,
                            true,
                            "missing_required_field".to_string(),
                        )
                        .unwrap(),
                        DiagDiagnosticFlag::v1(
                            "confirmation_pending".to_string(),
                            DiagCheckArea::Confirmation,
                            true,
                            "confirmation_required".to_string(),
                        )
                        .unwrap(),
                    ];
                    Ph1DiagResponse::DiagConsistencyCheckOk(
                        DiagConsistencyCheckOk::v1(ReasonCodeId(1), flags, true).unwrap(),
                    )
                }
                Ph1DiagRequest::DiagReasonSetBuild(_r) => Ph1DiagResponse::DiagReasonSetBuildOk(
                    DiagReasonSetBuildOk::v1(
                        ReasonCodeId(2),
                        DiagValidationStatus::Ok,
                        vec![
                            "confirmation_required".to_string(),
                            "missing_required_field".to_string(),
                        ],
                        vec![],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftDiagEngine;

    impl Ph1DiagEngine for DriftDiagEngine {
        fn run(&self, req: &Ph1DiagRequest) -> Ph1DiagResponse {
            match req {
                Ph1DiagRequest::DiagConsistencyCheck(_r) => {
                    let flags = vec![DiagDiagnosticFlag::v1(
                        "clarify_missing_field".to_string(),
                        DiagCheckArea::RequiredField,
                        true,
                        "missing_required_field".to_string(),
                    )
                    .unwrap()];
                    Ph1DiagResponse::DiagConsistencyCheckOk(
                        DiagConsistencyCheckOk::v1(ReasonCodeId(10), flags, true).unwrap(),
                    )
                }
                Ph1DiagRequest::DiagReasonSetBuild(_r) => Ph1DiagResponse::DiagReasonSetBuildOk(
                    DiagReasonSetBuildOk::v1(
                        ReasonCodeId(11),
                        DiagValidationStatus::Fail,
                        vec!["missing_required_field".to_string()],
                        vec!["confirmation_pending_missing".to_string()],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn input() -> DiagTurnInput {
        DiagTurnInput::v1(
            CorrelationId(1901),
            TurnId(151),
            "MESSAGE_DRAFT".to_string(),
            vec!["recipient".to_string()],
            vec![],
            true,
            false,
            false,
            DiagDeliveryMode::VoiceAllowed,
            false,
            false,
        )
        .unwrap()
    }

    #[test]
    fn at_diag_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1DiagWiring::new(Ph1DiagWiringConfig::mvp_v1(true), DeterministicDiagEngine).unwrap();
        let out = wiring.run_turn(&input()).unwrap();
        match out {
            DiagWiringOutcome::Forwarded(bundle) => assert!(bundle.validate().is_ok()),
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_diag_02_os_forwards_blocking_flags_for_ph1x_gating() {
        let wiring =
            Ph1DiagWiring::new(Ph1DiagWiringConfig::mvp_v1(true), DeterministicDiagEngine).unwrap();
        let out = wiring.run_turn(&input()).unwrap();
        match out {
            DiagWiringOutcome::Forwarded(bundle) => {
                assert!(bundle
                    .consistency_check
                    .diagnostic_flags
                    .iter()
                    .any(|flag| flag.flag_id == "confirmation_pending"));
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_diag_03_os_does_not_invoke_when_diag_is_disabled() {
        let wiring =
            Ph1DiagWiring::new(Ph1DiagWiringConfig::mvp_v1(false), DeterministicDiagEngine)
                .unwrap();
        let out = wiring.run_turn(&input()).unwrap();
        assert_eq!(out, DiagWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_diag_04_os_fails_closed_on_reason_set_validation_drift() {
        let wiring =
            Ph1DiagWiring::new(Ph1DiagWiringConfig::mvp_v1(true), DriftDiagEngine).unwrap();
        let out = wiring.run_turn(&input()).unwrap();
        match out {
            DiagWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_DIAG_VALIDATION_FAILED);
                assert_eq!(r.capability_id, DiagCapabilityId::DiagReasonSetBuild);
            }
            _ => panic!("expected Refused"),
        }
    }
}
