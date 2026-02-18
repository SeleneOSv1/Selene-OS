#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::ph1quota::{
    Ph1QuotaRequest, Ph1QuotaResponse, QuotaCapabilityId, QuotaDecisionAction,
    QuotaDecisionComputeOk, QuotaDecisionComputeRequest, QuotaOperationKind, QuotaPolicyEvaluateOk,
    QuotaPolicyEvaluateRequest, QuotaRefuse, QuotaRequestEnvelope,
};
use selene_kernel_contracts::{ContractViolation, MonotonicTimeNs, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.QUOTA OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_QUOTA_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5154_0101);
    pub const PH1_QUOTA_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5154_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1QuotaWiringConfig {
    pub quota_enabled: bool,
    pub max_wait_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1QuotaWiringConfig {
    pub fn mvp_v1(quota_enabled: bool) -> Self {
        Self {
            quota_enabled,
            max_wait_ms: 120_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: TenantId,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub operation_kind: QuotaOperationKind,
    pub capability_id: Option<String>,
    pub tool_name: Option<String>,
    pub now_ns: MonotonicTimeNs,
    pub cost_hint_microunits: Option<u64>,
    pub rate_limit_exceeded: bool,
    pub budget_exceeded: bool,
    pub policy_blocked: bool,
    pub wait_permitted: bool,
    pub suggested_wait_ms: Option<u32>,
}

impl QuotaTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        user_id: Option<String>,
        device_id: Option<String>,
        operation_kind: QuotaOperationKind,
        capability_id: Option<String>,
        tool_name: Option<String>,
        now_ns: MonotonicTimeNs,
        cost_hint_microunits: Option<u64>,
        rate_limit_exceeded: bool,
        budget_exceeded: bool,
        policy_blocked: bool,
        wait_permitted: bool,
        suggested_wait_ms: Option<u32>,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            user_id,
            device_id,
            operation_kind,
            capability_id,
            tool_name,
            now_ns,
            cost_hint_microunits,
            rate_limit_exceeded,
            budget_exceeded,
            policy_blocked,
            wait_permitted,
            suggested_wait_ms,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for QuotaTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.tenant_id.validate()?;

        if let Some(user_id) = &self.user_id {
            validate_token("quota_turn_input.user_id", user_id, 96)?;
        }
        if let Some(device_id) = &self.device_id {
            validate_token("quota_turn_input.device_id", device_id, 96)?;
        }
        if let Some(capability_id) = &self.capability_id {
            validate_token("quota_turn_input.capability_id", capability_id, 128)?;
        }
        if let Some(tool_name) = &self.tool_name {
            validate_token("quota_turn_input.tool_name", tool_name, 96)?;
        }

        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "quota_turn_input.now_ns",
                reason: "must be > 0",
            });
        }
        if let Some(cost_hint_microunits) = self.cost_hint_microunits {
            if cost_hint_microunits == 0 || cost_hint_microunits > 1_000_000_000_000 {
                return Err(ContractViolation::InvalidValue {
                    field: "quota_turn_input.cost_hint_microunits",
                    reason: "must be within 1..=1_000_000_000_000 when present",
                });
            }
        }

        match self.operation_kind {
            QuotaOperationKind::Tool => {
                if self.tool_name.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_turn_input.tool_name",
                        reason: "must be present when operation_kind=TOOL",
                    });
                }
                if self.capability_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_turn_input.capability_id",
                        reason: "must be absent when operation_kind=TOOL",
                    });
                }
            }
            QuotaOperationKind::Stt
            | QuotaOperationKind::Tts
            | QuotaOperationKind::Simulation
            | QuotaOperationKind::Export => {
                if self.capability_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_turn_input.capability_id",
                        reason: "must be present when operation_kind is not TOOL",
                    });
                }
                if self.tool_name.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "quota_turn_input.tool_name",
                        reason: "must be absent when operation_kind is not TOOL",
                    });
                }
            }
        }

        if self.policy_blocked && self.wait_permitted {
            return Err(ContractViolation::InvalidValue {
                field: "quota_turn_input.wait_permitted",
                reason: "must be false when policy_blocked=true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuotaForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: QuotaPolicyEvaluateOk,
    pub decision_compute: QuotaDecisionComputeOk,
}

impl QuotaForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: QuotaPolicyEvaluateOk,
        decision_compute: QuotaDecisionComputeOk,
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

impl Validate for QuotaForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;
        if !self.decision_compute.no_gate_order_change || !self.decision_compute.no_authority_grant
        {
            return Err(ContractViolation::InvalidValue {
                field: "quota_forward_bundle.decision_compute",
                reason: "must preserve no_authority_grant and no_gate_order_change invariants",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuotaWiringOutcome {
    NotInvokedDisabled,
    Refused(QuotaRefuse),
    Forwarded(QuotaForwardBundle),
}

pub trait Ph1QuotaEngine {
    fn run(&self, req: &Ph1QuotaRequest) -> Ph1QuotaResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1QuotaWiring<E>
where
    E: Ph1QuotaEngine,
{
    config: Ph1QuotaWiringConfig,
    engine: E,
}

impl<E> Ph1QuotaWiring<E>
where
    E: Ph1QuotaEngine,
{
    pub fn new(config: Ph1QuotaWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_wait_ms == 0 || config.max_wait_ms > 3_600_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1quota_wiring_config.max_wait_ms",
                reason: "must be within 1..=3_600_000",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1quota_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &QuotaTurnInput,
    ) -> Result<QuotaWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.quota_enabled {
            return Ok(QuotaWiringOutcome::NotInvokedDisabled);
        }

        let envelope = QuotaRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_diagnostics, 16),
            min(self.config.max_wait_ms, 3_600_000),
        )?;

        let policy_req = Ph1QuotaRequest::QuotaPolicyEvaluate(QuotaPolicyEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.user_id.clone(),
            input.device_id.clone(),
            input.operation_kind,
            input.capability_id.clone(),
            input.tool_name.clone(),
            input.now_ns,
            input.cost_hint_microunits,
            input.rate_limit_exceeded,
            input.budget_exceeded,
            input.policy_blocked,
            input.wait_permitted,
            input.suggested_wait_ms,
            true,
            true,
            true,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        policy_resp.validate()?;

        let policy_ok = match policy_resp {
            Ph1QuotaResponse::Refuse(refuse) => return Ok(QuotaWiringOutcome::Refused(refuse)),
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(ok) => ok,
            Ph1QuotaResponse::QuotaDecisionComputeOk(_) => {
                return Ok(QuotaWiringOutcome::Refused(QuotaRefuse::v1(
                    QuotaCapabilityId::QuotaPolicyEvaluate,
                    reason_codes::PH1_QUOTA_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy-evaluate request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1QuotaRequest::QuotaDecisionCompute(QuotaDecisionComputeRequest::v1(
            envelope,
            input.tenant_id.clone(),
            input.operation_kind,
            policy_ok.throttle_cause,
            policy_ok.allow_eligible,
            policy_ok.wait_permitted,
            policy_ok.wait_ms,
            policy_ok.refuse_required,
            policy_ok.deterministic,
            policy_ok.no_authority_grant,
            policy_ok.no_gate_order_change,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        decision_resp.validate()?;

        let decision_ok = match decision_resp {
            Ph1QuotaResponse::Refuse(refuse) => return Ok(QuotaWiringOutcome::Refused(refuse)),
            Ph1QuotaResponse::QuotaDecisionComputeOk(ok) => ok,
            Ph1QuotaResponse::QuotaPolicyEvaluateOk(_) => {
                return Ok(QuotaWiringOutcome::Refused(QuotaRefuse::v1(
                    QuotaCapabilityId::QuotaDecisionCompute,
                    reason_codes::PH1_QUOTA_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision-compute request".to_string(),
                )?));
            }
        };

        if decision_ok.action == QuotaDecisionAction::Wait && decision_ok.wait_ms.is_none() {
            return Ok(QuotaWiringOutcome::Refused(QuotaRefuse::v1(
                QuotaCapabilityId::QuotaDecisionCompute,
                reason_codes::PH1_QUOTA_VALIDATION_FAILED,
                "wait decision must include wait_ms".to_string(),
            )?));
        }

        let bundle =
            QuotaForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(QuotaWiringOutcome::Forwarded(bundle))
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
    use selene_kernel_contracts::ph1quota::{QuotaDecisionComputeOk, QuotaThrottleCause};
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct MockQuotaEngine {
        policy_response: Ph1QuotaResponse,
        decision_response: Ph1QuotaResponse,
    }

    impl Ph1QuotaEngine for MockQuotaEngine {
        fn run(&self, req: &Ph1QuotaRequest) -> Ph1QuotaResponse {
            match req {
                Ph1QuotaRequest::QuotaPolicyEvaluate(_) => self.policy_response.clone(),
                Ph1QuotaRequest::QuotaDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    fn base_input() -> QuotaTurnInput {
        QuotaTurnInput::v1(
            CorrelationId(7301),
            TurnId(8301),
            TenantId::new("tenant_demo").unwrap(),
            Some("user_1".to_string()),
            Some("device_1".to_string()),
            QuotaOperationKind::Stt,
            Some("PH1C_TRANSCRIPT_OK_COMMIT_ROW".to_string()),
            None,
            MonotonicTimeNs(10_000),
            Some(1000),
            true,
            false,
            false,
            true,
            Some(2000),
        )
        .unwrap()
    }

    fn policy_ok_wait() -> QuotaPolicyEvaluateOk {
        QuotaPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            TenantId::new("tenant_demo").unwrap(),
            QuotaOperationKind::Stt,
            Some("PH1C_TRANSCRIPT_OK_COMMIT_ROW".to_string()),
            QuotaThrottleCause::RateLimit,
            false,
            true,
            Some(2000),
            false,
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok_wait() -> QuotaDecisionComputeOk {
        QuotaDecisionComputeOk::v1(
            ReasonCodeId(2),
            QuotaDecisionAction::Wait,
            Some(2000),
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_quota_01_disabled_returns_not_invoked() {
        let engine = MockQuotaEngine {
            policy_response: Ph1QuotaResponse::QuotaPolicyEvaluateOk(policy_ok_wait()),
            decision_response: Ph1QuotaResponse::QuotaDecisionComputeOk(decision_ok_wait()),
        };
        let wiring = Ph1QuotaWiring::new(Ph1QuotaWiringConfig::mvp_v1(false), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        assert!(matches!(out, QuotaWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_quota_02_policy_refuse_propagates() {
        let refuse = QuotaRefuse::v1(
            QuotaCapabilityId::QuotaPolicyEvaluate,
            ReasonCodeId(100),
            "policy blocked".to_string(),
        )
        .unwrap();
        let engine = MockQuotaEngine {
            policy_response: Ph1QuotaResponse::Refuse(refuse.clone()),
            decision_response: Ph1QuotaResponse::QuotaDecisionComputeOk(decision_ok_wait()),
        };
        let wiring = Ph1QuotaWiring::new(Ph1QuotaWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let QuotaWiringOutcome::Refused(refused) = out else {
            panic!("expected refuse");
        };
        assert_eq!(refused.reason_code, refuse.reason_code);
    }

    #[test]
    fn at_quota_03_forwarded_wait_bundle_is_valid() {
        let engine = MockQuotaEngine {
            policy_response: Ph1QuotaResponse::QuotaPolicyEvaluateOk(policy_ok_wait()),
            decision_response: Ph1QuotaResponse::QuotaDecisionComputeOk(decision_ok_wait()),
        };
        let wiring = Ph1QuotaWiring::new(Ph1QuotaWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let QuotaWiringOutcome::Forwarded(bundle) = out else {
            panic!("expected forwarded");
        };
        assert_eq!(bundle.decision_compute.action, QuotaDecisionAction::Wait);
        assert_eq!(bundle.decision_compute.wait_ms, Some(2000));
    }

    #[test]
    fn at_quota_04_fail_closed_on_unexpected_response_variant() {
        let engine = MockQuotaEngine {
            policy_response: Ph1QuotaResponse::QuotaPolicyEvaluateOk(policy_ok_wait()),
            decision_response: Ph1QuotaResponse::QuotaPolicyEvaluateOk(policy_ok_wait()),
        };
        let wiring = Ph1QuotaWiring::new(Ph1QuotaWiringConfig::mvp_v1(true), engine).unwrap();
        let out = wiring.run_turn(&base_input()).unwrap();
        let QuotaWiringOutcome::Refused(refused) = out else {
            panic!("expected refused");
        };
        assert_eq!(
            refused.reason_code,
            reason_codes::PH1_QUOTA_INTERNAL_PIPELINE_ERROR
        );
    }
}
