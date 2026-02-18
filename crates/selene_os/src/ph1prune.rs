#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1prune::{
    Ph1PruneRequest, Ph1PruneResponse, PruneCapabilityId, PruneClarifyOrderOk,
    PruneClarifyOrderRequest, PruneMissingFieldsOk, PruneMissingFieldsRequest, PruneRefuse,
    PruneRequestEnvelope, PruneValidationStatus,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PRUNE OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_PRUNE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5055_0101);
    pub const PH1_PRUNE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5055_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PruneWiringConfig {
    pub prune_enabled: bool,
    pub max_missing_fields: u8,
}

impl Ph1PruneWiringConfig {
    pub fn mvp_v1(prune_enabled: bool) -> Self {
        Self {
            prune_enabled,
            max_missing_fields: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub required_fields_missing: Vec<String>,
    pub ambiguity_flags: Vec<String>,
    pub uncertain_field_hints: Vec<String>,
    pub prefilled_fields: Vec<String>,
    pub confirmed_fields: Vec<String>,
    pub previous_clarify_field: Option<String>,
}

impl PruneTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        required_fields_missing: Vec<String>,
        ambiguity_flags: Vec<String>,
        uncertain_field_hints: Vec<String>,
        prefilled_fields: Vec<String>,
        confirmed_fields: Vec<String>,
        previous_clarify_field: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            required_fields_missing,
            ambiguity_flags,
            uncertain_field_hints,
            prefilled_fields,
            confirmed_fields,
            previous_clarify_field,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for PruneTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_field_list(
            "prune_turn_input.required_fields_missing",
            &self.required_fields_missing,
            32,
        )?;
        validate_field_list(
            "prune_turn_input.uncertain_field_hints",
            &self.uncertain_field_hints,
            16,
        )?;
        validate_field_list(
            "prune_turn_input.prefilled_fields",
            &self.prefilled_fields,
            32,
        )?;
        validate_field_list(
            "prune_turn_input.confirmed_fields",
            &self.confirmed_fields,
            32,
        )?;
        validate_field_list(
            "prune_turn_input.ambiguity_flags",
            &self.ambiguity_flags,
            16,
        )?;
        if let Some(field) = &self.previous_clarify_field {
            validate_field_key("prune_turn_input.previous_clarify_field", field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub prune_missing_fields: PruneMissingFieldsOk,
    pub prune_clarify_order: PruneClarifyOrderOk,
}

impl PruneForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        prune_missing_fields: PruneMissingFieldsOk,
        prune_clarify_order: PruneClarifyOrderOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            prune_missing_fields,
            prune_clarify_order,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for PruneForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.prune_missing_fields.validate()?;
        self.prune_clarify_order.validate()?;
        if self.prune_clarify_order.validation_status != PruneValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "prune_forward_bundle.prune_clarify_order.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PruneWiringOutcome {
    NotInvokedDisabled,
    NotInvokedInsufficientMissingFields,
    Refused(PruneRefuse),
    Forwarded(PruneForwardBundle),
}

pub trait Ph1PruneEngine {
    fn run(&self, req: &Ph1PruneRequest) -> Ph1PruneResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PruneWiring<E>
where
    E: Ph1PruneEngine,
{
    config: Ph1PruneWiringConfig,
    engine: E,
}

impl<E> Ph1PruneWiring<E>
where
    E: Ph1PruneEngine,
{
    pub fn new(config: Ph1PruneWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_missing_fields == 0 || config.max_missing_fields > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1prune_wiring_config.max_missing_fields",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &PruneTurnInput,
    ) -> Result<PruneWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.prune_enabled {
            return Ok(PruneWiringOutcome::NotInvokedDisabled);
        }

        if input.required_fields_missing.len() < 2 {
            return Ok(PruneWiringOutcome::NotInvokedInsufficientMissingFields);
        }

        let envelope = PruneRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_missing_fields, 32),
        )?;

        let missing_req = Ph1PruneRequest::PruneMissingFields(PruneMissingFieldsRequest::v1(
            envelope.clone(),
            input.required_fields_missing.clone(),
            input.ambiguity_flags.clone(),
            input.uncertain_field_hints.clone(),
            input.prefilled_fields.clone(),
            input.confirmed_fields.clone(),
            input.previous_clarify_field.clone(),
        )?);
        let missing_resp = self.engine.run(&missing_req);
        missing_resp.validate()?;

        let missing_ok = match missing_resp {
            Ph1PruneResponse::Refuse(r) => return Ok(PruneWiringOutcome::Refused(r)),
            Ph1PruneResponse::PruneMissingFieldsOk(ok) => ok,
            Ph1PruneResponse::PruneClarifyOrderOk(_) => {
                return Ok(PruneWiringOutcome::Refused(PruneRefuse::v1(
                    PruneCapabilityId::PruneMissingFields,
                    reason_codes::PH1_PRUNE_INTERNAL_PIPELINE_ERROR,
                    "unexpected clarify-order response for missing-fields request".to_string(),
                )?))
            }
        };

        let validate_req = Ph1PruneRequest::PruneClarifyOrder(PruneClarifyOrderRequest::v1(
            envelope,
            input.required_fields_missing.clone(),
            missing_ok.selected_missing_field.clone(),
            missing_ok.ordered_missing_fields.clone(),
            input.previous_clarify_field.clone(),
        )?);
        let validate_resp = self.engine.run(&validate_req);
        validate_resp.validate()?;

        let validate_ok = match validate_resp {
            Ph1PruneResponse::Refuse(r) => return Ok(PruneWiringOutcome::Refused(r)),
            Ph1PruneResponse::PruneClarifyOrderOk(ok) => ok,
            Ph1PruneResponse::PruneMissingFieldsOk(_) => {
                return Ok(PruneWiringOutcome::Refused(PruneRefuse::v1(
                    PruneCapabilityId::PruneClarifyOrder,
                    reason_codes::PH1_PRUNE_INTERNAL_PIPELINE_ERROR,
                    "unexpected missing-fields response for clarify-order request".to_string(),
                )?))
            }
        };

        if validate_ok.validation_status != PruneValidationStatus::Ok {
            return Ok(PruneWiringOutcome::Refused(PruneRefuse::v1(
                PruneCapabilityId::PruneClarifyOrder,
                reason_codes::PH1_PRUNE_VALIDATION_FAILED,
                "prune clarify-order validation failed".to_string(),
            )?));
        }

        let bundle =
            PruneForwardBundle::v1(input.correlation_id, input.turn_id, missing_ok, validate_ok)?;
        Ok(PruneWiringOutcome::Forwarded(bundle))
    }
}

fn validate_field_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII snake_case",
        });
    }
    Ok(())
}

fn validate_field_list(
    field: &'static str,
    items: &[String],
    max_len: usize,
) -> Result<(), ContractViolation> {
    if items.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max list size",
        });
    }
    for item in items {
        validate_field_key(field, item)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1prune::{PruneValidationStatus, PruneValidationStatus::Ok};
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicPruneEngine;

    impl Ph1PruneEngine for DeterministicPruneEngine {
        fn run(&self, req: &Ph1PruneRequest) -> Ph1PruneResponse {
            match req {
                Ph1PruneRequest::PruneMissingFields(r) => {
                    let selected = if r.required_fields_missing.contains(&"amount".to_string()) {
                        "amount".to_string()
                    } else {
                        r.required_fields_missing[0].clone()
                    };
                    let mut ordered = r.required_fields_missing.clone();
                    ordered.sort();
                    if let Some(pos) = ordered.iter().position(|f| f == &selected) {
                        ordered.swap(0, pos);
                    }
                    Ph1PruneResponse::PruneMissingFieldsOk(
                        PruneMissingFieldsOk::v1(ReasonCodeId(1), selected, ordered, true).unwrap(),
                    )
                }
                Ph1PruneRequest::PruneClarifyOrder(_r) => Ph1PruneResponse::PruneClarifyOrderOk(
                    PruneClarifyOrderOk::v1(ReasonCodeId(2), Ok, vec![], true).unwrap(),
                ),
            }
        }
    }

    struct DriftPruneEngine;

    impl Ph1PruneEngine for DriftPruneEngine {
        fn run(&self, req: &Ph1PruneRequest) -> Ph1PruneResponse {
            match req {
                Ph1PruneRequest::PruneMissingFields(r) => {
                    let ordered = r.required_fields_missing.clone();
                    Ph1PruneResponse::PruneMissingFieldsOk(
                        PruneMissingFieldsOk::v1(
                            ReasonCodeId(10),
                            ordered[0].clone(),
                            ordered,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PruneRequest::PruneClarifyOrder(_r) => Ph1PruneResponse::PruneClarifyOrderOk(
                    PruneClarifyOrderOk::v1(
                        ReasonCodeId(11),
                        PruneValidationStatus::Fail,
                        vec!["selected_repeats_previous_clarify".to_string()],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    #[test]
    fn at_prune_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1PruneWiring::new(Ph1PruneWiringConfig::mvp_v1(true), DeterministicPruneEngine)
                .unwrap();
        let input = PruneTurnInput::v1(
            CorrelationId(1501),
            TurnId(111),
            vec!["amount".to_string(), "recipient".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PruneWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
                assert_eq!(
                    bundle.prune_missing_fields.selected_missing_field,
                    "amount".to_string()
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_prune_02_os_skips_when_fewer_than_two_fields_are_missing() {
        let wiring =
            Ph1PruneWiring::new(Ph1PruneWiringConfig::mvp_v1(true), DeterministicPruneEngine)
                .unwrap();
        let input = PruneTurnInput::v1(
            CorrelationId(1502),
            TurnId(112),
            vec!["amount".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, PruneWiringOutcome::NotInvokedInsufficientMissingFields);
    }

    #[test]
    fn at_prune_03_os_fails_closed_when_clarify_order_validation_fails() {
        let wiring =
            Ph1PruneWiring::new(Ph1PruneWiringConfig::mvp_v1(true), DriftPruneEngine).unwrap();
        let input = PruneTurnInput::v1(
            CorrelationId(1503),
            TurnId(113),
            vec!["when".to_string(), "amount".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
            Some("when".to_string()),
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PruneWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.capability_id, PruneCapabilityId::PruneClarifyOrder);
                assert_eq!(
                    refuse.reason_code,
                    reason_codes::PH1_PRUNE_VALIDATION_FAILED
                );
            }
            _ => panic!("expected Refused"),
        }
    }

    #[test]
    fn at_prune_04_os_forwards_single_selected_field_for_ph1x_clarify() {
        let wiring =
            Ph1PruneWiring::new(Ph1PruneWiringConfig::mvp_v1(true), DeterministicPruneEngine)
                .unwrap();
        let input = PruneTurnInput::v1(
            CorrelationId(1504),
            TurnId(114),
            vec![
                "recipient".to_string(),
                "amount".to_string(),
                "when".to_string(),
            ],
            vec![],
            vec![],
            vec![],
            vec![],
            None,
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            PruneWiringOutcome::Forwarded(bundle) => {
                assert_eq!(bundle.prune_missing_fields.selected_missing_field, "amount");
                assert_eq!(
                    bundle.prune_missing_fields.ordered_missing_fields[0],
                    "amount"
                );
            }
            _ => panic!("expected Forwarded"),
        }
    }
}
