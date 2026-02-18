#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1policy::{
    Ph1PolicyRequest, Ph1PolicyResponse, PolicyCapabilityId, PolicyPromptDecision,
    PolicyPromptDedupeDecideOk, PolicyPromptDedupeDecideRequest, PolicyRefuse,
    PolicyRequestEnvelope, PolicyRulesetGetActiveOk, PolicyRulesetGetActiveRequest,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.POLICY OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_POLICY_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x504F_0101);
    pub const PH1_POLICY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x504F_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PolicyWiringConfig {
    pub policy_enabled: bool,
    pub max_required_fields: u8,
}

impl Ph1PolicyWiringConfig {
    pub fn mvp_v1(policy_enabled: bool) -> Self {
        Self {
            policy_enabled,
            max_required_fields: 16,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPromptTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub work_order_id: String,
    pub user_id: Option<String>,
    pub now_ns: u64,
    pub required_fields: Vec<String>,
    pub known_fields: Vec<String>,
    pub asked_fields: Vec<String>,
    pub prompt_dedupe_keys: Vec<String>,
    pub authoritative_prefill_fields: Vec<String>,
}

impl PolicyPromptTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        work_order_id: String,
        user_id: Option<String>,
        now_ns: u64,
        required_fields: Vec<String>,
        known_fields: Vec<String>,
        asked_fields: Vec<String>,
        prompt_dedupe_keys: Vec<String>,
        authoritative_prefill_fields: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            work_order_id,
            user_id,
            now_ns,
            required_fields,
            known_fields,
            asked_fields,
            prompt_dedupe_keys,
            authoritative_prefill_fields,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for PolicyPromptTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("policy_prompt_turn_input.tenant_id", &self.tenant_id, 96)?;
        validate_token(
            "policy_prompt_turn_input.work_order_id",
            &self.work_order_id,
            96,
        )?;
        if let Some(user_id) = &self.user_id {
            validate_token("policy_prompt_turn_input.user_id", user_id, 96)?;
        }
        if self.now_ns == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_turn_input.now_ns",
                reason: "must be > 0",
            });
        }
        if self.required_fields.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_turn_input.required_fields",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPromptForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub ruleset: PolicyRulesetGetActiveOk,
    pub prompt_decision: PolicyPromptDedupeDecideOk,
}

impl PolicyPromptForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        ruleset: PolicyRulesetGetActiveOk,
        prompt_decision: PolicyPromptDedupeDecideOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            ruleset,
            prompt_decision,
        };
        bundle.validate()?;
        Ok(bundle)
    }

    pub fn prompt_gate_ok(&self) -> bool {
        matches!(self.prompt_decision.decision, PolicyPromptDecision::Ask)
    }
}

impl Validate for PolicyPromptForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.ruleset.validate()?;
        self.prompt_decision.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyPromptWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoPromptNeeded,
    Refused(PolicyRefuse),
    Forwarded(PolicyPromptForwardBundle),
}

pub trait Ph1PolicyEngine {
    fn run(&self, req: &Ph1PolicyRequest) -> Ph1PolicyResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1PolicyWiring<E>
where
    E: Ph1PolicyEngine,
{
    config: Ph1PolicyWiringConfig,
    engine: E,
}

impl<E> Ph1PolicyWiring<E>
where
    E: Ph1PolicyEngine,
{
    pub fn new(config: Ph1PolicyWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_required_fields == 0 || config.max_required_fields > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1policy_wiring_config.max_required_fields",
                reason: "must be within 1..=32",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_prompt_gate(
        &self,
        input: &PolicyPromptTurnInput,
    ) -> Result<PolicyPromptWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.policy_enabled {
            return Ok(PolicyPromptWiringOutcome::NotInvokedDisabled);
        }
        if input.required_fields.is_empty() {
            return Ok(PolicyPromptWiringOutcome::NotInvokedNoPromptNeeded);
        }

        let envelope = PolicyRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_required_fields, 32),
        )?;

        let ruleset_req =
            Ph1PolicyRequest::PolicyRulesetGetActive(PolicyRulesetGetActiveRequest::v1(
                envelope.clone(),
                input.tenant_id.clone(),
                input.user_id.clone(),
                input.now_ns,
            )?);
        let ruleset_resp = self.engine.run(&ruleset_req);
        if ruleset_resp.validate().is_err() {
            return Ok(PolicyPromptWiringOutcome::Refused(PolicyRefuse::v1(
                PolicyCapabilityId::PolicyRulesetGetActive,
                reason_codes::PH1_POLICY_VALIDATION_FAILED,
                "invalid policy ruleset response contract".to_string(),
            )?));
        }
        let ruleset_ok = match ruleset_resp {
            Ph1PolicyResponse::Refuse(r) => return Ok(PolicyPromptWiringOutcome::Refused(r)),
            Ph1PolicyResponse::PolicyRulesetGetActiveOk(ok) => ok,
            Ph1PolicyResponse::PolicyPromptDedupeDecideOk(_) => {
                return Ok(PolicyPromptWiringOutcome::Refused(PolicyRefuse::v1(
                    PolicyCapabilityId::PolicyRulesetGetActive,
                    reason_codes::PH1_POLICY_INTERNAL_PIPELINE_ERROR,
                    "unexpected prompt-dedupe response for ruleset request".to_string(),
                )?))
            }
        };

        let prompt_req =
            Ph1PolicyRequest::PolicyPromptDedupeDecide(PolicyPromptDedupeDecideRequest::v1(
                envelope,
                input.tenant_id.clone(),
                input.work_order_id.clone(),
                input.now_ns,
                input.required_fields.clone(),
                input.known_fields.clone(),
                input.asked_fields.clone(),
                input.prompt_dedupe_keys.clone(),
                input.authoritative_prefill_fields.clone(),
            )?);
        let prompt_resp = self.engine.run(&prompt_req);
        if prompt_resp.validate().is_err() {
            return Ok(PolicyPromptWiringOutcome::Refused(PolicyRefuse::v1(
                PolicyCapabilityId::PolicyPromptDedupeDecide,
                reason_codes::PH1_POLICY_VALIDATION_FAILED,
                "invalid prompt-dedupe response contract".to_string(),
            )?));
        }
        let prompt_ok = match prompt_resp {
            Ph1PolicyResponse::Refuse(r) => return Ok(PolicyPromptWiringOutcome::Refused(r)),
            Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok) => ok,
            Ph1PolicyResponse::PolicyRulesetGetActiveOk(_) => {
                return Ok(PolicyPromptWiringOutcome::Refused(PolicyRefuse::v1(
                    PolicyCapabilityId::PolicyPromptDedupeDecide,
                    reason_codes::PH1_POLICY_INTERNAL_PIPELINE_ERROR,
                    "unexpected ruleset response for prompt-dedupe request".to_string(),
                )?))
            }
        };

        let bundle = PolicyPromptForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            ruleset_ok,
            prompt_ok,
        )?;
        Ok(PolicyPromptWiringOutcome::Forwarded(bundle))
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1policy::{
        PolicyPromptDecision, PolicyPromptDedupeDecideOk, PolicyRulesetGetActiveOk,
    };
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct DeterministicPolicyEngine;

    impl Ph1PolicyEngine for DeterministicPolicyEngine {
        fn run(&self, req: &Ph1PolicyRequest) -> Ph1PolicyResponse {
            match req {
                Ph1PolicyRequest::PolicyRulesetGetActive(_r) => {
                    Ph1PolicyResponse::PolicyRulesetGetActiveOk(
                        PolicyRulesetGetActiveOk::v1(
                            ReasonCodeId(1),
                            "policy_v1".to_string(),
                            "h_1111".to_string(),
                            vec!["PROMPT_DEDUPE".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PolicyRequest::PolicyPromptDedupeDecide(_r) => {
                    Ph1PolicyResponse::PolicyPromptDedupeDecideOk(
                        PolicyPromptDedupeDecideOk::v1(
                            ReasonCodeId(2),
                            PolicyPromptDecision::Ask,
                            Some("amount".to_string()),
                            Some("tenant:work:amount".to_string()),
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    #[derive(Debug, Clone)]
    struct DriftPolicyEngine;

    impl Ph1PolicyEngine for DriftPolicyEngine {
        fn run(&self, req: &Ph1PolicyRequest) -> Ph1PolicyResponse {
            match req {
                Ph1PolicyRequest::PolicyRulesetGetActive(_r) => {
                    Ph1PolicyResponse::PolicyPromptDedupeDecideOk(
                        PolicyPromptDedupeDecideOk::v1(
                            ReasonCodeId(3),
                            PolicyPromptDecision::Ask,
                            Some("amount".to_string()),
                            Some("tenant:work:amount".to_string()),
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1PolicyRequest::PolicyPromptDedupeDecide(_r) => {
                    Ph1PolicyResponse::PolicyRulesetGetActiveOk(
                        PolicyRulesetGetActiveOk::v1(
                            ReasonCodeId(4),
                            "policy_v1".to_string(),
                            "h_2222".to_string(),
                            vec!["PROMPT_DEDUPE".to_string()],
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn input_with_required_fields() -> PolicyPromptTurnInput {
        PolicyPromptTurnInput::v1(
            CorrelationId(7901),
            TurnId(8901),
            "tenant_1".to_string(),
            "work_1".to_string(),
            Some("user_1".to_string()),
            10,
            vec!["amount".to_string()],
            vec![],
            vec![],
            vec![],
            vec![],
        )
        .unwrap()
    }

    #[test]
    fn at_policy_wiring_01_not_invoked_when_disabled() {
        let wiring = Ph1PolicyWiring::new(
            Ph1PolicyWiringConfig::mvp_v1(false),
            DeterministicPolicyEngine,
        )
        .unwrap();
        let out = wiring
            .run_prompt_gate(&input_with_required_fields())
            .unwrap();
        assert!(matches!(out, PolicyPromptWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_policy_wiring_02_forwards_ruleset_and_prompt_decision() {
        let wiring = Ph1PolicyWiring::new(
            Ph1PolicyWiringConfig::mvp_v1(true),
            DeterministicPolicyEngine,
        )
        .unwrap();
        let out = wiring
            .run_prompt_gate(&input_with_required_fields())
            .unwrap();
        let PolicyPromptWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded output");
        };
        assert_eq!(bundle.prompt_decision.decision, PolicyPromptDecision::Ask);
        assert!(bundle.prompt_gate_ok());
    }

    #[test]
    fn at_policy_wiring_03_fails_closed_on_variant_mismatch() {
        let wiring =
            Ph1PolicyWiring::new(Ph1PolicyWiringConfig::mvp_v1(true), DriftPolicyEngine).unwrap();
        let out = wiring
            .run_prompt_gate(&input_with_required_fields())
            .unwrap();
        assert!(matches!(out, PolicyPromptWiringOutcome::Refused(_)));
    }
}
