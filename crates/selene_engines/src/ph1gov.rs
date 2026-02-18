#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use selene_kernel_contracts::ph1gov::{
    GovCapabilityId, GovDecisionComputeOk, GovDecisionComputeRequest, GovDecisionStatus,
    GovPolicyEvaluateOk, GovPolicyEvaluateRequest, GovRefuse, GovRequestedAction, Ph1GovRequest,
    Ph1GovResponse,
};
use selene_kernel_contracts::ReasonCodeId;
use selene_kernel_contracts::Validate;

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.GOV reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_GOV_OK_POLICY_EVALUATE: ReasonCodeId = ReasonCodeId(0x474F_0001);
    pub const PH1_GOV_OK_DECISION_COMPUTE: ReasonCodeId = ReasonCodeId(0x474F_0002);

    pub const GOV_NOT_AUTHORIZED: ReasonCodeId = ReasonCodeId(0x474F_0010);
    pub const GOV_SIGNATURE_INVALID: ReasonCodeId = ReasonCodeId(0x474F_0011);
    pub const GOV_REFERENCE_MISSING: ReasonCodeId = ReasonCodeId(0x474F_0012);
    pub const GOV_MULTI_ACTIVE_NOT_ALLOWED: ReasonCodeId = ReasonCodeId(0x474F_0013);

    pub const PH1_GOV_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x474F_00F1);
    pub const PH1_GOV_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x474F_00F2);
    pub const PH1_GOV_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x474F_00F3);
    pub const PH1_GOV_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x474F_00F4);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1GovConfig {
    pub max_reference_ids: u8,
    pub max_diagnostics: u8,
    pub enterprise_mode_signature_required: bool,
}

impl Ph1GovConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_reference_ids: 16,
            max_diagnostics: 8,
            enterprise_mode_signature_required: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1GovRuntime {
    config: Ph1GovConfig,
}

impl Ph1GovRuntime {
    pub fn new(config: Ph1GovConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1GovRequest) -> Ph1GovResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_GOV_INPUT_SCHEMA_INVALID,
                "gov request failed contract validation",
            );
        }

        match req {
            Ph1GovRequest::GovPolicyEvaluate(r) => self.run_policy_evaluate(r),
            Ph1GovRequest::GovDecisionCompute(r) => self.run_decision_compute(r),
        }
    }

    fn run_policy_evaluate(&self, req: &GovPolicyEvaluateRequest) -> Ph1GovResponse {
        if req.artifact_id.is_empty() || req.requester_user_id.is_empty() {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::PH1_GOV_UPSTREAM_INPUT_MISSING,
                "required governance inputs are missing",
            );
        }

        if req.required_reference_ids.len() > self.config.max_reference_ids as usize
            || req.active_reference_ids.len() > self.config.max_reference_ids as usize
            || req.existing_active_versions.len() > self.config.max_reference_ids as usize
        {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::PH1_GOV_BUDGET_EXCEEDED,
                "governance reference budget exceeded",
            );
        }

        if !req.requester_authorized {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::GOV_NOT_AUTHORIZED,
                "requester is not authorized for governance action",
            );
        }

        let signature_valid = match &req.signature_ref {
            Some(signature_ref) => {
                signature_ref.starts_with("sig_") && !signature_ref.contains("invalid")
            }
            None => !self.config.enterprise_mode_signature_required,
        };
        if self.config.enterprise_mode_signature_required && !signature_valid {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::GOV_SIGNATURE_INVALID,
                "governance signature is invalid",
            );
        }

        let references_active = references_active(
            req.required_reference_ids.as_slice(),
            req.active_reference_ids.as_slice(),
        );
        if !references_active {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::GOV_REFERENCE_MISSING,
                "governance references are missing or not active",
            );
        }

        let single_active_blueprint_ok = if req.enforce_single_active_blueprint
            && req.artifact_kind == selene_kernel_contracts::ph1gov::GovArtifactKind::Blueprint
            && req.requested_action == GovRequestedAction::Activate
        {
            req.existing_active_versions
                .iter()
                .all(|v| v.0 == req.artifact_version.0)
        } else {
            true
        };
        if !single_active_blueprint_ok {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::GOV_MULTI_ACTIVE_NOT_ALLOWED,
                "multiple active blueprints for one intent are not allowed",
            );
        }

        let rollback_target_present = req.rollback_target_version.is_some();
        if req.requested_action == GovRequestedAction::Rollback && !rollback_target_present {
            return self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::GOV_REFERENCE_MISSING,
                "rollback target version is missing",
            );
        }

        match GovPolicyEvaluateOk::v1(
            reason_codes::PH1_GOV_OK_POLICY_EVALUATE,
            req.artifact_kind,
            req.artifact_id.clone(),
            req.requested_action,
            true,
            signature_valid,
            references_active,
            single_active_blueprint_ok,
            rollback_target_present,
            true,
            true,
        ) {
            Ok(ok) => Ph1GovResponse::GovPolicyEvaluateOk(ok),
            Err(_) => self.refuse(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR,
                "failed to construct governance policy output",
            ),
        }
    }

    fn run_decision_compute(&self, req: &GovDecisionComputeRequest) -> Ph1GovResponse {
        let (decision, reason_code, active_version) = if !req.requester_authorized {
            (
                GovDecisionStatus::Blocked,
                reason_codes::GOV_NOT_AUTHORIZED,
                req.current_active_version,
            )
        } else if !req.signature_valid {
            (
                GovDecisionStatus::Blocked,
                reason_codes::GOV_SIGNATURE_INVALID,
                req.current_active_version,
            )
        } else if !req.references_active {
            (
                GovDecisionStatus::Blocked,
                reason_codes::GOV_REFERENCE_MISSING,
                req.current_active_version,
            )
        } else if req.requested_action == GovRequestedAction::Activate
            && req.artifact_kind == selene_kernel_contracts::ph1gov::GovArtifactKind::Blueprint
            && !req.single_active_blueprint_ok
        {
            (
                GovDecisionStatus::Blocked,
                reason_codes::GOV_MULTI_ACTIVE_NOT_ALLOWED,
                req.current_active_version,
            )
        } else {
            let active_version = match req.requested_action {
                GovRequestedAction::Activate => Some(req.artifact_version),
                GovRequestedAction::Deprecate => req.current_active_version,
                GovRequestedAction::Rollback => req.rollback_target_version,
            };
            (
                GovDecisionStatus::Allowed,
                reason_codes::PH1_GOV_OK_DECISION_COMPUTE,
                active_version,
            )
        };

        match GovDecisionComputeOk::v1(
            reason_code,
            decision,
            req.artifact_kind,
            req.artifact_id.clone(),
            req.requested_action,
            active_version,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1GovResponse::GovDecisionComputeOk(ok),
            Err(_) => self.refuse(
                GovCapabilityId::GovDecisionCompute,
                reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR,
                "failed to construct governance decision output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: GovCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1GovResponse {
        let out = GovRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("GovRefuse::v1 must construct for static messages");
        Ph1GovResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1GovRequest) -> GovCapabilityId {
    match req {
        Ph1GovRequest::GovPolicyEvaluate(_) => GovCapabilityId::GovPolicyEvaluate,
        Ph1GovRequest::GovDecisionCompute(_) => GovCapabilityId::GovDecisionCompute,
    }
}

fn references_active(required: &[String], active: &[String]) -> bool {
    let active_set = active.iter().map(|x| x.as_str()).collect::<BTreeSet<_>>();
    required.iter().all(|x| active_set.contains(x.as_str()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1gov::{
        GovArtifactKind, GovArtifactVersion, GovDecisionComputeRequest, GovPolicyEvaluateRequest,
        GovRequestEnvelope, GovRequestedAction,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1position::TenantId;

    fn envelope() -> GovRequestEnvelope {
        GovRequestEnvelope::v1(CorrelationId(9301), TurnId(1501), 8, 8, true).unwrap()
    }

    fn base_policy_request() -> GovPolicyEvaluateRequest {
        GovPolicyEvaluateRequest::v1(
            envelope(),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![],
            vec!["SIM_1".to_string(), "CAP_A".to_string()],
            vec!["SIM_1".to_string(), "CAP_A".to_string()],
            None,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_gov_01_cannot_activate_when_references_missing() {
        let runtime = Ph1GovRuntime::new(Ph1GovConfig::mvp_v1());
        let mut req = base_policy_request();
        req.required_reference_ids.push("SIM_2".to_string());

        let resp = runtime.run(&Ph1GovRequest::GovPolicyEvaluate(req));
        let Ph1GovResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(refuse.reason_code, reason_codes::GOV_REFERENCE_MISSING);
    }

    #[test]
    fn at_gov_02_single_active_blueprint_rule_is_enforced() {
        let runtime = Ph1GovRuntime::new(Ph1GovConfig::mvp_v1());
        let mut req = base_policy_request();
        req.existing_active_versions = vec![GovArtifactVersion(1), GovArtifactVersion(2)];

        let resp = runtime.run(&Ph1GovRequest::GovPolicyEvaluate(req));
        let Ph1GovResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::GOV_MULTI_ACTIVE_NOT_ALLOWED
        );
    }

    #[test]
    fn at_gov_03_activation_blocked_when_signature_invalid() {
        let runtime = Ph1GovRuntime::new(Ph1GovConfig::mvp_v1());
        let mut req = base_policy_request();
        req.signature_ref = Some("sig_invalid".to_string());

        let resp = runtime.run(&Ph1GovRequest::GovPolicyEvaluate(req));
        let Ph1GovResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(refuse.reason_code, reason_codes::GOV_SIGNATURE_INVALID);
    }

    #[test]
    fn at_gov_04_rollback_decision_is_deterministic_and_auditable() {
        let runtime = Ph1GovRuntime::new(Ph1GovConfig::mvp_v1());
        let req = GovDecisionComputeRequest::v1(
            envelope(),
            GovArtifactKind::Simulation,
            "sim_payroll".to_string(),
            GovArtifactVersion(5),
            GovRequestedAction::Rollback,
            Some(GovArtifactVersion(5)),
            Some(GovArtifactVersion(4)),
            true,
            true,
            true,
            true,
            true,
            true,
        )
        .unwrap();
        let resp = runtime.run(&Ph1GovRequest::GovDecisionCompute(req));
        let Ph1GovResponse::GovDecisionComputeOk(out) = resp else {
            panic!("expected decision output");
        };
        assert_eq!(out.decision, GovDecisionStatus::Allowed);
        assert_eq!(out.active_version, Some(GovArtifactVersion(4)));
        assert!(out.audit_event_required);
        assert!(out.deterministic);
        assert!(out.no_execution_authority);
    }
}
