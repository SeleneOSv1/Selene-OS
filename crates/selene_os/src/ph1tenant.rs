#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1tenant::{
    Ph1TenantRequest, Ph1TenantResponse, TenantBinding, TenantCapabilityId,
    TenantDecisionComputeOk, TenantDecisionComputeRequest, TenantIdentityContext,
    TenantPolicyEvaluateOk, TenantPolicyEvaluateRequest, TenantRefuse, TenantRequestEnvelope,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.TENANT OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_TENANT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5445_0101);
    pub const PH1_TENANT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5445_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1TenantWiringConfig {
    pub tenant_enabled: bool,
    pub max_candidates: u8,
    pub max_missing_fields: u8,
    pub max_diagnostics: u8,
}

impl Ph1TenantWiringConfig {
    pub fn mvp_v1(tenant_enabled: bool) -> Self {
        Self {
            tenant_enabled,
            max_candidates: 8,
            max_missing_fields: 2,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub identity_context: TenantIdentityContext,
    pub device_id: Option<String>,
    pub session_id: Option<String>,
    pub now_ns: MonotonicTimeNs,
    pub explicit_tenant_selection_token: Option<String>,
    pub explicit_tenant_id: Option<TenantId>,
    pub candidate_bindings: Vec<TenantBinding>,
}

impl TenantTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        identity_context: TenantIdentityContext,
        device_id: Option<String>,
        session_id: Option<String>,
        now_ns: MonotonicTimeNs,
        explicit_tenant_selection_token: Option<String>,
        explicit_tenant_id: Option<TenantId>,
        candidate_bindings: Vec<TenantBinding>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            identity_context,
            device_id,
            session_id,
            now_ns,
            explicit_tenant_selection_token,
            explicit_tenant_id,
            candidate_bindings,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for TenantTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.identity_context.validate()?;
        if let Some(device_id) = &self.device_id {
            validate_token("tenant_turn_input.device_id", device_id, 96)?;
        }
        if let Some(session_id) = &self.session_id {
            validate_token("tenant_turn_input.session_id", session_id, 96)?;
        }
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_turn_input.now_ns",
                reason: "must be > 0",
            });
        }
        if let Some(explicit_tenant_selection_token) = &self.explicit_tenant_selection_token {
            validate_token(
                "tenant_turn_input.explicit_tenant_selection_token",
                explicit_tenant_selection_token,
                128,
            )?;
        }
        if let Some(explicit_tenant_id) = &self.explicit_tenant_id {
            explicit_tenant_id.validate()?;
        }
        if self.candidate_bindings.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_turn_input.candidate_bindings",
                reason: "must contain <= 16 bindings",
            });
        }
        for binding in &self.candidate_bindings {
            binding.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: TenantPolicyEvaluateOk,
    pub decision_compute: TenantDecisionComputeOk,
}

impl TenantForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: TenantPolicyEvaluateOk,
        decision_compute: TenantDecisionComputeOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            policy_evaluate,
            decision_compute,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for TenantForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantWiringOutcome {
    NotInvokedDisabled,
    Refused(TenantRefuse),
    Forwarded(TenantForwardBundle),
}

pub trait Ph1TenantEngine {
    fn run(&self, req: &Ph1TenantRequest) -> Ph1TenantResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1TenantWiring<E>
where
    E: Ph1TenantEngine,
{
    config: Ph1TenantWiringConfig,
    engine: E,
}

impl<E> Ph1TenantWiring<E>
where
    E: Ph1TenantEngine,
{
    pub fn new(config: Ph1TenantWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_candidates == 0 || config.max_candidates > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tenant_wiring_config.max_candidates",
                reason: "must be within 1..=16",
            });
        }
        if config.max_missing_fields == 0 || config.max_missing_fields > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tenant_wiring_config.max_missing_fields",
                reason: "must be within 1..=4",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1tenant_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &TenantTurnInput,
    ) -> Result<TenantWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.tenant_enabled {
            return Ok(TenantWiringOutcome::NotInvokedDisabled);
        }

        let envelope = TenantRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_candidates, 16),
            min(self.config.max_missing_fields, 4),
            min(self.config.max_diagnostics, 16),
        )?;

        let policy_req = Ph1TenantRequest::TenantPolicyEvaluate(TenantPolicyEvaluateRequest::v1(
            envelope.clone(),
            input.identity_context.clone(),
            input.device_id.clone(),
            input.session_id.clone(),
            input.now_ns,
            input.explicit_tenant_selection_token.clone(),
            input.explicit_tenant_id.clone(),
            input.candidate_bindings.clone(),
            true,
            true,
            true,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        policy_resp.validate()?;

        let policy_ok = match policy_resp {
            Ph1TenantResponse::Refuse(refuse) => return Ok(TenantWiringOutcome::Refused(refuse)),
            Ph1TenantResponse::TenantPolicyEvaluateOk(ok) => ok,
            Ph1TenantResponse::TenantDecisionComputeOk(_) => {
                return Ok(TenantWiringOutcome::Refused(TenantRefuse::v1(
                    TenantCapabilityId::TenantPolicyEvaluate,
                    reason_codes::PH1_TENANT_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy-evaluate request".to_string(),
                )?));
            }
        };

        let selected_binding = input
            .candidate_bindings
            .iter()
            .find(|binding| Some(&binding.tenant_id) == policy_ok.selected_tenant_id.as_ref());

        let decision_req =
            Ph1TenantRequest::TenantDecisionCompute(TenantDecisionComputeRequest::v1(
                envelope,
                policy_ok.identity_known,
                policy_ok.candidate_count,
                policy_ok.selected_tenant_id.clone(),
                selected_binding.map(|binding| binding.policy_context_ref.clone()),
                selected_binding.and_then(|binding| binding.default_locale.clone()),
                selected_binding.is_some_and(|binding| binding.tenant_disabled),
                selected_binding.is_some_and(|binding| binding.tenant_policy_blocked),
                policy_ok.multiple_match,
                true,
                true,
                true,
            )?);
        let decision_resp = self.engine.run(&decision_req);
        decision_resp.validate()?;

        let decision_ok = match decision_resp {
            Ph1TenantResponse::Refuse(refuse) => return Ok(TenantWiringOutcome::Refused(refuse)),
            Ph1TenantResponse::TenantDecisionComputeOk(ok) => ok,
            Ph1TenantResponse::TenantPolicyEvaluateOk(_) => {
                return Ok(TenantWiringOutcome::Refused(TenantRefuse::v1(
                    TenantCapabilityId::TenantDecisionCompute,
                    reason_codes::PH1_TENANT_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision-compute request".to_string(),
                )?));
            }
        };

        let bundle =
            TenantForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(TenantWiringOutcome::Forwarded(bundle))
    }
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be non-empty",
        });
    }
    if value.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if value.chars().any(|c| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == ':' || c == '.' || c == '/')
    }) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must contain token-safe ASCII only",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1tenant::{TenantResolveStatus, TenantSelectionSource};
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct MockTenantEngine {
        policy_response: Ph1TenantResponse,
        decision_response: Ph1TenantResponse,
    }

    impl Ph1TenantEngine for MockTenantEngine {
        fn run(&self, req: &Ph1TenantRequest) -> Ph1TenantResponse {
            match req {
                Ph1TenantRequest::TenantPolicyEvaluate(_) => self.policy_response.clone(),
                Ph1TenantRequest::TenantDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    fn binding(tenant_id: &str) -> TenantBinding {
        TenantBinding::v1(
            TenantId::new(tenant_id).unwrap(),
            "policy/default".to_string(),
            Some("en-US".to_string()),
            false,
            false,
        )
        .unwrap()
    }

    fn base_input() -> TenantTurnInput {
        TenantTurnInput::v1(
            CorrelationId(7601),
            TurnId(8601),
            TenantIdentityContext::SignedInUser {
                user_id: "user_1".to_string(),
            },
            Some("device_1".to_string()),
            Some("session_1".to_string()),
            MonotonicTimeNs(1_000_000),
            None,
            None,
            vec![binding("tenant_a")],
        )
        .unwrap()
    }

    fn policy_ok() -> TenantPolicyEvaluateOk {
        TenantPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            true,
            1,
            Some(TenantId::new("tenant_a").unwrap()),
            TenantSelectionSource::DeterministicSingleCandidate,
            false,
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok() -> TenantDecisionComputeOk {
        TenantDecisionComputeOk::v1(
            ReasonCodeId(2),
            TenantResolveStatus::Ok,
            Some(TenantId::new("tenant_a").unwrap()),
            Some("policy/default".to_string()),
            Some("en-US".to_string()),
            vec![],
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_tenant_01_disabled_returns_not_invoked() {
        let engine = MockTenantEngine {
            policy_response: Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1TenantResponse::TenantDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1TenantWiring::new(Ph1TenantWiringConfig::mvp_v1(false), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        assert!(matches!(out, TenantWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_tenant_02_policy_refuse_propagates() {
        let refuse = TenantRefuse::v1(
            TenantCapabilityId::TenantPolicyEvaluate,
            ReasonCodeId(100),
            "tenant policy reject".to_string(),
        )
        .unwrap();
        let engine = MockTenantEngine {
            policy_response: Ph1TenantResponse::Refuse(refuse.clone()),
            decision_response: Ph1TenantResponse::TenantDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1TenantWiring::new(Ph1TenantWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let TenantWiringOutcome::Refused(refused) = out else {
            panic!("expected refuse");
        };
        assert_eq!(refused.reason_code, refuse.reason_code);
    }

    #[test]
    fn at_tenant_03_forwarded_bundle_is_valid() {
        let engine = MockTenantEngine {
            policy_response: Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1TenantResponse::TenantDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1TenantWiring::new(Ph1TenantWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let TenantWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded");
        };
        assert_eq!(bundle.decision_compute.status, TenantResolveStatus::Ok);
    }

    #[test]
    fn at_tenant_04_fail_closed_on_unexpected_response_variant() {
        let engine = MockTenantEngine {
            policy_response: Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok()),
        };
        let wiring = Ph1TenantWiring::new(Ph1TenantWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let TenantWiringOutcome::Refused(refused) = out else {
            panic!("expected refused");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_TENANT_INTERNAL_PIPELINE_ERROR
        );
    }
}
