#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1tenant::{
    Ph1TenantRequest, Ph1TenantResponse, TenantBinding, TenantCapabilityId,
    TenantDecisionComputeOk, TenantDecisionComputeRequest, TenantIdentityContext,
    TenantPolicyEvaluateOk, TenantPolicyEvaluateRequest, TenantRefuse, TenantResolveStatus,
    TenantSelectionSource,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.TENANT reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_TENANT_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x5445_0001);
    pub const PH1_TENANT_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x5445_0002);

    pub const TENANT_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x5445_0010);
    pub const TENANT_MULTI_MATCH: ReasonCodeId = ReasonCodeId(0x5445_0011);
    pub const TENANT_DISABLED: ReasonCodeId = ReasonCodeId(0x5445_0012);
    pub const TENANT_POLICY_BLOCKED: ReasonCodeId = ReasonCodeId(0x5445_0013);

    pub const PH1_TENANT_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5445_00F1);
    pub const PH1_TENANT_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5445_00F2);
    pub const PH1_TENANT_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5445_00F3);
    pub const PH1_TENANT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5445_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1TenantConfig {
    pub max_candidates: u8,
    pub max_missing_fields: u8,
    pub max_diagnostics: u8,
}

impl Ph1TenantConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_candidates: 8,
            max_missing_fields: 2,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1TenantRuntime {
    config: Ph1TenantConfig,
}

impl Ph1TenantRuntime {
    pub fn new(config: Ph1TenantConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1TenantRequest) -> Ph1TenantResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_TENANT_INPUT_SCHEMA_INVALID,
                "tenant request failed contract validation",
            );
        }

        match req {
            Ph1TenantRequest::TenantPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1TenantRequest::TenantDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &TenantPolicyEvaluateRequest) -> Ph1TenantResponse {
        if req.candidate_bindings.len() > self.config.max_candidates as usize {
            return self.refuse(
                TenantCapabilityId::TenantPolicyEvaluate,
                reason_codes::PH1_TENANT_BUDGET_EXCEEDED,
                "candidate tenant budget exceeded",
            );
        }

        let identity_known = !matches!(
            req.identity_context,
            TenantIdentityContext::VoiceAssertionUnknown
        );
        let explicit_selection = req.explicit_tenant_id.clone();

        if req.explicit_tenant_selection_token.is_some() && explicit_selection.is_none() {
            return self.refuse(
                TenantCapabilityId::TenantPolicyEvaluate,
                reason_codes::PH1_TENANT_UPSTREAM_INPUT_MISSING,
                "explicit tenant token is present but tenant selection is missing",
            );
        }

        let selected_tenant_id = if let Some(explicit_tenant_id) = explicit_selection {
            Some(explicit_tenant_id)
        } else if identity_known && req.candidate_bindings.len() == 1 {
            Some(req.candidate_bindings[0].tenant_id.clone())
        } else {
            None
        };

        let selection_source = if req.explicit_tenant_id.is_some() {
            TenantSelectionSource::ExplicitSelection
        } else if selected_tenant_id.is_some() {
            TenantSelectionSource::DeterministicSingleCandidate
        } else {
            TenantSelectionSource::None
        };

        let multiple_match = req.candidate_bindings.len() >= 2 && req.explicit_tenant_id.is_none();

        match TenantPolicyEvaluateOk::v1(
            reason_codes::PH1_TENANT_OK_POLICY_EVALUATE,
            identity_known,
            req.candidate_bindings.len() as u8,
            selected_tenant_id,
            selection_source,
            multiple_match,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1TenantResponse::TenantPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                TenantCapabilityId::TenantPolicyEvaluate,
                reason_codes::PH1_TENANT_INTERNAL_PIPELINE_ERROR,
                "failed to construct tenant policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &TenantDecisionComputeRequest) -> Ph1TenantResponse {
        let (status, reason_code, tenant_id, policy_context_ref, default_locale, missing_fields) =
            if !req.identity_known {
                (
                    TenantResolveStatus::NeedsClarify,
                    reason_codes::TENANT_NOT_FOUND,
                    None,
                    None,
                    None,
                    vec!["tenant_choice".to_string()],
                )
            } else if req.multiple_match && req.selected_tenant_id.is_none() {
                (
                    TenantResolveStatus::NeedsClarify,
                    reason_codes::TENANT_MULTI_MATCH,
                    None,
                    None,
                    None,
                    vec!["tenant_choice".to_string()],
                )
            } else if req.selected_tenant_id.is_none() {
                (
                    TenantResolveStatus::NeedsClarify,
                    reason_codes::TENANT_NOT_FOUND,
                    None,
                    None,
                    None,
                    vec!["tenant_choice".to_string()],
                )
            } else if req.selected_tenant_disabled {
                (
                    TenantResolveStatus::Refused,
                    reason_codes::TENANT_DISABLED,
                    None,
                    None,
                    None,
                    vec![],
                )
            } else if req.selected_tenant_policy_blocked {
                (
                    TenantResolveStatus::Refused,
                    reason_codes::TENANT_POLICY_BLOCKED,
                    None,
                    None,
                    None,
                    vec![],
                )
            } else {
                (
                    TenantResolveStatus::Ok,
                    reason_codes::PH1_TENANT_OK_DECISION_COMPUTE,
                    req.selected_tenant_id.clone(),
                    req.selected_policy_context_ref.clone(),
                    req.selected_default_locale.clone(),
                    vec![],
                )
            };

        if missing_fields.len() > self.config.max_missing_fields as usize {
            return self.refuse(
                TenantCapabilityId::TenantDecisionCompute,
                reason_codes::PH1_TENANT_BUDGET_EXCEEDED,
                "tenant missing-field budget exceeded",
            );
        }

        match TenantDecisionComputeOk::v1(
            reason_code,
            status,
            tenant_id,
            policy_context_ref,
            default_locale,
            missing_fields,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1TenantResponse::TenantDecisionComputeOk(ok),
            Err(_) => self.refuse(
                TenantCapabilityId::TenantDecisionCompute,
                reason_codes::PH1_TENANT_INTERNAL_PIPELINE_ERROR,
                "failed to construct tenant decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: TenantCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1TenantResponse {
        let out = TenantRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("TenantRefuse::v1 must construct for static messages");
        Ph1TenantResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1TenantRequest) -> TenantCapabilityId {
    match req {
        Ph1TenantRequest::TenantPolicyEvaluate(_) => TenantCapabilityId::TenantPolicyEvaluate,
        Ph1TenantRequest::TenantDecisionCompute(_) => TenantCapabilityId::TenantDecisionCompute,
    }
}

pub fn selected_binding<'a>(
    bindings: &'a [TenantBinding],
    selected_tenant_id: &Option<selene_kernel_contracts::ph1position::TenantId>,
) -> Option<&'a TenantBinding> {
    let selected_tenant_id = selected_tenant_id.as_ref()?;
    bindings
        .iter()
        .find(|binding| binding.tenant_id == *selected_tenant_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;
    use selene_kernel_contracts::ph1tenant::{
        TenantBinding, TenantDecisionComputeRequest, TenantIdentityContext,
        TenantPolicyEvaluateRequest, TenantRequestEnvelope,
    };
    use selene_kernel_contracts::MonotonicTimeNs;

    fn envelope() -> TenantRequestEnvelope {
        TenantRequestEnvelope::v1(CorrelationId(7501), TurnId(8501), 8, 2, 8).unwrap()
    }

    fn binding(tenant_id: &str, disabled: bool, policy_blocked: bool) -> TenantBinding {
        TenantBinding::v1(
            TenantId::new(tenant_id).unwrap(),
            "policy/default".to_string(),
            Some("en-US".to_string()),
            disabled,
            policy_blocked,
        )
        .unwrap()
    }

    fn base_request(bindings: Vec<TenantBinding>) -> TenantPolicyEvaluateRequest {
        TenantPolicyEvaluateRequest::v1(
            envelope(),
            TenantIdentityContext::SignedInUser {
                user_id: "user_1".to_string(),
            },
            Some("device_1".to_string()),
            Some("session_1".to_string()),
            MonotonicTimeNs(1_000_000),
            None,
            None,
            bindings,
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_tenant_01_deterministic_tenant_mapping() {
        let runtime = Ph1TenantRuntime::new(Ph1TenantConfig::mvp_v1());
        let req = base_request(vec![binding("tenant_a", false, false)]);
        let out1 = runtime.run(&Ph1TenantRequest::TenantPolicyEvaluate(req.clone()));
        let out2 = runtime.run(&Ph1TenantRequest::TenantPolicyEvaluate(req));

        let ok1 = match out1 {
            Ph1TenantResponse::TenantPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected tenant policy output #1"),
        };
        let ok2 = match out2 {
            Ph1TenantResponse::TenantPolicyEvaluateOk(ok) => ok,
            _ => panic!("expected tenant policy output #2"),
        };
        assert_eq!(ok1.selected_tenant_id, ok2.selected_tenant_id);
        assert_eq!(
            ok1.selection_source,
            TenantSelectionSource::DeterministicSingleCandidate
        );
    }

    #[test]
    fn at_tenant_02_multi_tenant_requires_clarify() {
        let runtime = Ph1TenantRuntime::new(Ph1TenantConfig::mvp_v1());
        let policy_req = base_request(vec![
            binding("tenant_a", false, false),
            binding("tenant_b", false, false),
        ]);
        let policy_out = runtime.run(&Ph1TenantRequest::TenantPolicyEvaluate(policy_req));
        let Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };
        assert!(policy_ok.multiple_match);

        let decision_req = TenantDecisionComputeRequest::v1(
            envelope(),
            policy_ok.identity_known,
            policy_ok.candidate_count,
            None,
            None,
            None,
            false,
            false,
            policy_ok.multiple_match,
            true,
            true,
            true,
        )
        .unwrap();
        let decision_out = runtime.run(&Ph1TenantRequest::TenantDecisionCompute(decision_req));
        let Ph1TenantResponse::TenantDecisionComputeOk(ok) = decision_out else {
            panic!("expected decision output");
        };
        assert_eq!(ok.status, TenantResolveStatus::NeedsClarify);
        assert_eq!(ok.reason_code, reason_codes::TENANT_MULTI_MATCH);
    }

    #[test]
    fn at_tenant_03_disabled_tenant_fails_closed() {
        let runtime = Ph1TenantRuntime::new(Ph1TenantConfig::mvp_v1());
        let policy_req = base_request(vec![binding("tenant_a", true, false)]);
        let policy_out = runtime.run(&Ph1TenantRequest::TenantPolicyEvaluate(policy_req));
        let Ph1TenantResponse::TenantPolicyEvaluateOk(policy_ok) = policy_out else {
            panic!("expected policy output");
        };
        let decision_req = TenantDecisionComputeRequest::v1(
            envelope(),
            policy_ok.identity_known,
            policy_ok.candidate_count,
            policy_ok.selected_tenant_id.clone(),
            Some("policy/default".to_string()),
            Some("en-US".to_string()),
            true,
            false,
            false,
            true,
            true,
            true,
        )
        .unwrap();
        let decision_out = runtime.run(&Ph1TenantRequest::TenantDecisionCompute(decision_req));
        let Ph1TenantResponse::TenantDecisionComputeOk(ok) = decision_out else {
            panic!("expected decision output");
        };
        assert_eq!(ok.status, TenantResolveStatus::Refused);
        assert_eq!(ok.reason_code, reason_codes::TENANT_DISABLED);
    }

    #[test]
    fn at_tenant_04_no_cross_tenant_reads_writes_invariant() {
        let runtime = Ph1TenantRuntime::new(Ph1TenantConfig::mvp_v1());
        let bad_req = TenantPolicyEvaluateRequest::v1(
            envelope(),
            TenantIdentityContext::SignedInUser {
                user_id: "user_1".to_string(),
            },
            Some("device_1".to_string()),
            Some("session_1".to_string()),
            MonotonicTimeNs(1_000_000),
            None,
            None,
            vec![binding("tenant_a", false, false)],
            true,
            true,
            false,
        );
        assert!(bad_req.is_err());

        let req = base_request(vec![binding("tenant_a", false, false)]);
        let out = runtime.run(&Ph1TenantRequest::TenantPolicyEvaluate(req));
        let Ph1TenantResponse::TenantPolicyEvaluateOk(ok) = out else {
            panic!("expected policy output");
        };
        assert!(ok.no_cross_tenant_access);
    }
}
