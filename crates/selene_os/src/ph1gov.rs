#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1gov::{
    GovArtifactKind, GovArtifactVersion, GovCapabilityId, GovDecisionComputeOk,
    GovDecisionComputeRequest, GovDecisionStatus, GovPolicyEvaluateOk, GovPolicyEvaluateRequest,
    GovRefuse, GovRequestEnvelope, GovRequestedAction, Ph1GovRequest, Ph1GovResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1position::TenantId;
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.GOV OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_GOV_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x474F_0101);
    pub const PH1_GOV_POLICY_SCOPE_FAILED: ReasonCodeId = ReasonCodeId(0x474F_0102);
    pub const PH1_GOV_COHORT_GATES_FAILED: ReasonCodeId = ReasonCodeId(0x474F_0103);
    pub const PH1_GOV_STABILITY_WINDOW_FAILED: ReasonCodeId = ReasonCodeId(0x474F_0104);
    pub const PH1_GOV_REVOCATION_REQUIRES_ROLLBACK: ReasonCodeId = ReasonCodeId(0x474F_0105);
    pub const PH1_GOV_REVOCATION_SLA_BREACH: ReasonCodeId = ReasonCodeId(0x474F_0106);
    pub const PH1_GOV_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x474F_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1GovWiringConfig {
    pub gov_enabled: bool,
    pub max_reference_ids: u8,
    pub max_diagnostics: u8,
    pub enterprise_mode_signature_required: bool,
    pub enforce_single_active_blueprint: bool,
}

impl Ph1GovWiringConfig {
    pub fn mvp_v1(gov_enabled: bool) -> Self {
        Self {
            gov_enabled,
            max_reference_ids: 16,
            max_diagnostics: 8,
            enterprise_mode_signature_required: true,
            enforce_single_active_blueprint: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: TenantId,
    pub artifact_kind: GovArtifactKind,
    pub artifact_id: String,
    pub artifact_version: GovArtifactVersion,
    pub artifact_hash_sha256: String,
    pub signature_ref: Option<String>,
    pub requested_action: GovRequestedAction,
    pub requester_user_id: String,
    pub requester_authorized: bool,
    pub existing_active_versions: Vec<GovArtifactVersion>,
    pub required_reference_ids: Vec<String>,
    pub active_reference_ids: Vec<String>,
    pub rollback_target_version: Option<GovArtifactVersion>,
    pub current_active_version: Option<GovArtifactVersion>,
    pub privacy_policy_passed: bool,
    pub consent_scope_active: bool,
    pub tenant_scope_verified: bool,
    pub required_cohort_keys: Vec<String>,
    pub passing_cohort_keys: Vec<String>,
    pub stability_window_days_passed: u8,
    pub consent_revoked: bool,
    pub revocation_sla_met: bool,
}

impl GovTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        artifact_version: GovArtifactVersion,
        artifact_hash_sha256: String,
        signature_ref: Option<String>,
        requested_action: GovRequestedAction,
        requester_user_id: String,
        requester_authorized: bool,
        existing_active_versions: Vec<GovArtifactVersion>,
        required_reference_ids: Vec<String>,
        active_reference_ids: Vec<String>,
        rollback_target_version: Option<GovArtifactVersion>,
        current_active_version: Option<GovArtifactVersion>,
    ) -> Result<Self, ContractViolation> {
        Self::v2(
            correlation_id,
            turn_id,
            tenant_id,
            artifact_kind,
            artifact_id,
            artifact_version,
            artifact_hash_sha256,
            signature_ref,
            requested_action,
            requester_user_id,
            requester_authorized,
            existing_active_versions,
            required_reference_ids,
            active_reference_ids,
            rollback_target_version,
            current_active_version,
            true,
            true,
            true,
            Vec::new(),
            Vec::new(),
            7,
            false,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn v2(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: TenantId,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        artifact_version: GovArtifactVersion,
        artifact_hash_sha256: String,
        signature_ref: Option<String>,
        requested_action: GovRequestedAction,
        requester_user_id: String,
        requester_authorized: bool,
        existing_active_versions: Vec<GovArtifactVersion>,
        required_reference_ids: Vec<String>,
        active_reference_ids: Vec<String>,
        rollback_target_version: Option<GovArtifactVersion>,
        current_active_version: Option<GovArtifactVersion>,
        privacy_policy_passed: bool,
        consent_scope_active: bool,
        tenant_scope_verified: bool,
        required_cohort_keys: Vec<String>,
        passing_cohort_keys: Vec<String>,
        stability_window_days_passed: u8,
        consent_revoked: bool,
        revocation_sla_met: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            artifact_kind,
            artifact_id,
            artifact_version,
            artifact_hash_sha256,
            signature_ref,
            requested_action,
            requester_user_id,
            requester_authorized,
            existing_active_versions,
            required_reference_ids,
            active_reference_ids,
            rollback_target_version,
            current_active_version,
            privacy_policy_passed,
            consent_scope_active,
            tenant_scope_verified,
            required_cohort_keys,
            passing_cohort_keys,
            stability_window_days_passed,
            consent_revoked,
            revocation_sla_met,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for GovTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.tenant_id.validate()?;
        validate_token("gov_turn_input.artifact_id", &self.artifact_id, 128)?;
        self.artifact_version.validate()?;
        validate_sha256(
            "gov_turn_input.artifact_hash_sha256",
            &self.artifact_hash_sha256,
        )?;
        validate_token(
            "gov_turn_input.requester_user_id",
            &self.requester_user_id,
            96,
        )?;

        if self.required_reference_ids.is_empty() || self.required_reference_ids.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_turn_input.required_reference_ids",
                reason: "must contain 1..=32 reference ids",
            });
        }
        if self.active_reference_ids.is_empty() || self.active_reference_ids.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_turn_input.active_reference_ids",
                reason: "must contain 1..=32 reference ids",
            });
        }

        for version in &self.existing_active_versions {
            version.validate()?;
        }
        if let Some(rollback_target_version) = self.rollback_target_version {
            rollback_target_version.validate()?;
        }
        if let Some(current_active_version) = self.current_active_version {
            current_active_version.validate()?;
        }

        match self.requested_action {
            GovRequestedAction::Rollback => {
                if self.rollback_target_version.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_turn_input.rollback_target_version",
                        reason: "must be present when requested_action=ROLLBACK",
                    });
                }
            }
            GovRequestedAction::Activate | GovRequestedAction::Deprecate => {
                if self.rollback_target_version.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_turn_input.rollback_target_version",
                        reason: "must be absent unless requested_action=ROLLBACK",
                    });
                }
            }
        }

        if let Some(signature_ref) = &self.signature_ref {
            validate_token("gov_turn_input.signature_ref", signature_ref, 128)?;
        }
        if self.required_cohort_keys.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_turn_input.required_cohort_keys",
                reason: "must contain <= 16 entries",
            });
        }
        if self.passing_cohort_keys.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_turn_input.passing_cohort_keys",
                reason: "must contain <= 16 entries",
            });
        }
        let mut required_cohorts = std::collections::BTreeSet::new();
        for key in &self.required_cohort_keys {
            validate_token("gov_turn_input.required_cohort_keys", key, 64)?;
            if !required_cohorts.insert(key.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "gov_turn_input.required_cohort_keys",
                    reason: "must not contain duplicates",
                });
            }
        }
        let mut passing_cohorts = std::collections::BTreeSet::new();
        for key in &self.passing_cohort_keys {
            validate_token("gov_turn_input.passing_cohort_keys", key, 64)?;
            if !passing_cohorts.insert(key.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "gov_turn_input.passing_cohort_keys",
                    reason: "must not contain duplicates",
                });
            }
        }
        if self.stability_window_days_passed > 31 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_turn_input.stability_window_days_passed",
                reason: "must be <= 31",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub policy_evaluate: GovPolicyEvaluateOk,
    pub decision_compute: GovDecisionComputeOk,
}

impl GovForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        policy_evaluate: GovPolicyEvaluateOk,
        decision_compute: GovDecisionComputeOk,
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

    pub fn to_builder_dispatch_ticket(
        &self,
    ) -> Result<GovBuilderDispatchOutcome, ContractViolation> {
        self.validate()?;
        if self.decision_class().class != GovDecisionClass::Allow {
            return Ok(GovBuilderDispatchOutcome::NotDispatchedDecisionNotAllowed);
        }
        let active_version =
            self.decision_compute
                .active_version
                .ok_or(ContractViolation::InvalidValue {
                    field: "gov_forward_bundle.decision_compute.active_version",
                    reason: "must be present when decision=ALLOWED for builder dispatch",
                })?;
        let ticket = GovBuilderDispatchTicket::v1(
            self.correlation_id,
            self.turn_id,
            self.decision_compute.artifact_kind,
            self.decision_compute.artifact_id.clone(),
            self.decision_compute.requested_action,
            active_version,
        )?;
        Ok(GovBuilderDispatchOutcome::DispatchReady(ticket))
    }

    pub fn decision_class(&self) -> GovResolvedDecision {
        let class = match self.decision_compute.decision {
            GovDecisionStatus::Allowed => match self.decision_compute.requested_action {
                GovRequestedAction::Activate | GovRequestedAction::Deprecate => {
                    GovDecisionClass::Allow
                }
                GovRequestedAction::Rollback => GovDecisionClass::Rollback,
            },
            GovDecisionStatus::Blocked => GovDecisionClass::Block,
        };
        GovResolvedDecision {
            class,
            reason_code: self.decision_compute.reason_code,
        }
    }
}

impl Validate for GovForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.policy_evaluate.validate()?;
        self.decision_compute.validate()?;
        if !self.decision_compute.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "gov_forward_bundle.decision_compute.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovBuilderDispatchTicket {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub artifact_kind: GovArtifactKind,
    pub artifact_id: String,
    pub requested_action: GovRequestedAction,
    pub active_version: GovArtifactVersion,
}

impl GovBuilderDispatchTicket {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        requested_action: GovRequestedAction,
        active_version: GovArtifactVersion,
    ) -> Result<Self, ContractViolation> {
        let ticket = Self {
            correlation_id,
            turn_id,
            artifact_kind,
            artifact_id,
            requested_action,
            active_version,
        };
        ticket.validate()?;
        Ok(ticket)
    }
}

impl Validate for GovBuilderDispatchTicket {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token(
            "gov_builder_dispatch_ticket.artifact_id",
            &self.artifact_id,
            128,
        )?;
        self.active_version.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovBuilderDispatchOutcome {
    NotDispatchedDecisionNotAllowed,
    DispatchReady(GovBuilderDispatchTicket),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GovDecisionClass {
    Allow,
    Hold,
    Block,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GovResolvedDecision {
    pub class: GovDecisionClass,
    pub reason_code: selene_kernel_contracts::ReasonCodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GovWiringOutcome {
    NotInvokedDisabled,
    Refused(GovRefuse),
    Forwarded(GovForwardBundle),
}

impl GovWiringOutcome {
    pub fn resolved_decision(&self) -> Option<GovResolvedDecision> {
        match self {
            GovWiringOutcome::NotInvokedDisabled => None,
            GovWiringOutcome::Refused(refuse) => Some(GovResolvedDecision {
                class: GovDecisionClass::Hold,
                reason_code: refuse.reason_code,
            }),
            GovWiringOutcome::Forwarded(bundle) => Some(bundle.decision_class()),
        }
    }
}

pub trait Ph1GovEngine {
    fn run(&self, req: &Ph1GovRequest) -> Ph1GovResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1GovWiring<E>
where
    E: Ph1GovEngine,
{
    config: Ph1GovWiringConfig,
    engine: E,
}

impl<E> Ph1GovWiring<E>
where
    E: Ph1GovEngine,
{
    pub fn new(config: Ph1GovWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_reference_ids == 0 || config.max_reference_ids > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1gov_wiring_config.max_reference_ids",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1gov_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &GovTurnInput) -> Result<GovWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.gov_enabled {
            return Ok(GovWiringOutcome::NotInvokedDisabled);
        }
        if !input.privacy_policy_passed
            || !input.consent_scope_active
            || !input.tenant_scope_verified
        {
            return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::PH1_GOV_POLICY_SCOPE_FAILED,
                "policy/privacy/consent/tenant-scope preflight failed".to_string(),
            )?));
        }
        if input.requested_action == GovRequestedAction::Activate
            && !(7..=14).contains(&input.stability_window_days_passed)
        {
            return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                GovCapabilityId::GovPolicyEvaluate,
                reason_codes::PH1_GOV_STABILITY_WINDOW_FAILED,
                "stability window must be within 7..=14 days before activation".to_string(),
            )?));
        }
        if !input.required_cohort_keys.is_empty() {
            let passing = input
                .passing_cohort_keys
                .iter()
                .map(|k| k.as_str())
                .collect::<std::collections::BTreeSet<_>>();
            let missing = input
                .required_cohort_keys
                .iter()
                .filter(|required| !passing.contains(required.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                    GovCapabilityId::GovPolicyEvaluate,
                    reason_codes::PH1_GOV_COHORT_GATES_FAILED,
                    format!("cohort gates failed for {}", missing.join(",")),
                )?));
            }
        }
        if input.consent_revoked {
            if !input.revocation_sla_met {
                return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                    GovCapabilityId::GovPolicyEvaluate,
                    reason_codes::PH1_GOV_REVOCATION_SLA_BREACH,
                    "consent revocation rollback/freeze SLA not met".to_string(),
                )?));
            }
            if input.requested_action != GovRequestedAction::Rollback {
                return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                    GovCapabilityId::GovPolicyEvaluate,
                    reason_codes::PH1_GOV_REVOCATION_REQUIRES_ROLLBACK,
                    "consent revocation requires rollback action".to_string(),
                )?));
            }
        }

        let envelope = GovRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_reference_ids, 32),
            min(self.config.max_diagnostics, 16),
            self.config.enterprise_mode_signature_required,
        )?;

        let policy_req = Ph1GovRequest::GovPolicyEvaluate(GovPolicyEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.artifact_kind,
            input.artifact_id.clone(),
            input.artifact_version,
            input.artifact_hash_sha256.clone(),
            input.signature_ref.clone(),
            input.requested_action,
            input.requester_user_id.clone(),
            input.requester_authorized,
            input.existing_active_versions.clone(),
            input.required_reference_ids.clone(),
            input.active_reference_ids.clone(),
            input.rollback_target_version,
            self.config.enforce_single_active_blueprint,
        )?);
        let policy_resp = self.engine.run(&policy_req);
        policy_resp.validate()?;

        let policy_ok = match policy_resp {
            Ph1GovResponse::Refuse(refuse) => return Ok(GovWiringOutcome::Refused(refuse)),
            Ph1GovResponse::GovPolicyEvaluateOk(ok) => ok,
            Ph1GovResponse::GovDecisionComputeOk(_) => {
                return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                    GovCapabilityId::GovPolicyEvaluate,
                    reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR,
                    "unexpected decision-compute response for policy-evaluate request".to_string(),
                )?));
            }
        };

        let decision_req = Ph1GovRequest::GovDecisionCompute(GovDecisionComputeRequest::v1(
            envelope,
            input.artifact_kind,
            input.artifact_id.clone(),
            input.artifact_version,
            input.requested_action,
            input.current_active_version,
            input.rollback_target_version,
            policy_ok.requester_authorized,
            policy_ok.signature_valid,
            policy_ok.references_active,
            policy_ok.single_active_blueprint_ok,
            policy_ok.deterministic,
            policy_ok.audit_required,
        )?);
        let decision_resp = self.engine.run(&decision_req);
        decision_resp.validate()?;

        let decision_ok = match decision_resp {
            Ph1GovResponse::Refuse(refuse) => return Ok(GovWiringOutcome::Refused(refuse)),
            Ph1GovResponse::GovDecisionComputeOk(ok) => ok,
            Ph1GovResponse::GovPolicyEvaluateOk(_) => {
                return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                    GovCapabilityId::GovDecisionCompute,
                    reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR,
                    "unexpected policy-evaluate response for decision-compute request".to_string(),
                )?));
            }
        };

        if !decision_ok.no_execution_authority {
            return Ok(GovWiringOutcome::Refused(GovRefuse::v1(
                GovCapabilityId::GovDecisionCompute,
                reason_codes::PH1_GOV_VALIDATION_FAILED,
                "decision output violated no-execution-authority rule".to_string(),
            )?));
        }

        let bundle =
            GovForwardBundle::v1(input.correlation_id, input.turn_id, policy_ok, decision_ok)?;
        Ok(GovWiringOutcome::Forwarded(bundle))
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

fn validate_sha256(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be a 64-char hex value",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1gov::{GovDecisionStatus, GovRefuse};
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Debug, Clone)]
    struct MockGovEngine {
        policy_response: Ph1GovResponse,
        decision_response: Ph1GovResponse,
    }

    impl Ph1GovEngine for MockGovEngine {
        fn run(&self, req: &Ph1GovRequest) -> Ph1GovResponse {
            match req {
                Ph1GovRequest::GovPolicyEvaluate(_) => self.policy_response.clone(),
                Ph1GovRequest::GovDecisionCompute(_) => self.decision_response.clone(),
            }
        }
    }

    fn base_input() -> GovTurnInput {
        GovTurnInput::v1(
            CorrelationId(5201),
            TurnId(5301),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string(), "CAP_A".to_string()],
            vec!["SIM_1".to_string(), "CAP_A".to_string()],
            None,
            Some(GovArtifactVersion(2)),
        )
        .unwrap()
    }

    fn policy_ok() -> GovPolicyEvaluateOk {
        GovPolicyEvaluateOk::v1(
            ReasonCodeId(1),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovRequestedAction::Activate,
            true,
            true,
            true,
            true,
            false,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_ok() -> GovDecisionComputeOk {
        GovDecisionComputeOk::v1(
            ReasonCodeId(2),
            GovDecisionStatus::Allowed,
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovRequestedAction::Activate,
            Some(GovArtifactVersion(3)),
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_blocked() -> GovDecisionComputeOk {
        GovDecisionComputeOk::v1(
            ReasonCodeId(3),
            GovDecisionStatus::Blocked,
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovRequestedAction::Activate,
            Some(GovArtifactVersion(2)),
            true,
            true,
            true,
        )
        .unwrap()
    }

    fn decision_allow_rollback() -> GovDecisionComputeOk {
        GovDecisionComputeOk::v1(
            ReasonCodeId(4),
            GovDecisionStatus::Allowed,
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovRequestedAction::Rollback,
            Some(GovArtifactVersion(2)),
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_gov_01_disabled_returns_not_invoked() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(false), engine).unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        assert!(matches!(outcome, GovWiringOutcome::NotInvokedDisabled));
    }

    #[test]
    fn at_gov_02_policy_refuse_propagates() {
        let refuse = GovRefuse::v1(
            GovCapabilityId::GovPolicyEvaluate,
            ReasonCodeId(100),
            "not authorized".to_string(),
        )
        .unwrap();
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::Refuse(refuse.clone()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        let GovWiringOutcome::Refused(out) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(out.reason_code, refuse.reason_code);
    }

    #[test]
    fn at_gov_03_forwarded_bundle_is_deterministic_and_non_executing() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        let GovWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded bundle");
        };
        assert!(bundle.policy_evaluate.deterministic);
        assert!(bundle.decision_compute.no_execution_authority);
        assert!(bundle.decision_compute.audit_event_required);
    }

    #[test]
    fn at_gov_04_fail_closed_on_unexpected_decision_response_variant() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();

        let outcome = wiring.run_turn(&base_input()).unwrap();
        let GovWiringOutcome::Refused(out) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            out.reason_code,
            reason_codes::PH1_GOV_INTERNAL_PIPELINE_ERROR
        );
    }

    #[test]
    fn at_gov_05_builder_dispatch_ticket_emits_only_for_explicit_allow() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let outcome = wiring.run_turn(&base_input()).unwrap();
        let GovWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded");
        };
        let dispatch = bundle.to_builder_dispatch_ticket().unwrap();
        let GovBuilderDispatchOutcome::DispatchReady(ticket) = dispatch else {
            panic!("expected builder dispatch ticket");
        };
        assert_eq!(ticket.active_version, GovArtifactVersion(3));
        assert_eq!(ticket.requested_action, GovRequestedAction::Activate);
    }

    #[test]
    fn at_gov_06_builder_dispatch_blocks_non_allow_decisions() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_blocked()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let outcome = wiring.run_turn(&base_input()).unwrap();
        let GovWiringOutcome::Forwarded(bundle) = outcome else {
            panic!("expected forwarded");
        };
        let dispatch = bundle.to_builder_dispatch_ticket().unwrap();
        assert_eq!(
            dispatch,
            GovBuilderDispatchOutcome::NotDispatchedDecisionNotAllowed
        );
    }

    #[test]
    fn at_gov_07_preflight_policy_scope_gates_fail_closed_before_engine() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let input = GovTurnInput::v2(
            CorrelationId(5201),
            TurnId(5301),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string()],
            vec!["SIM_1".to_string()],
            None,
            Some(GovArtifactVersion(2)),
            false,
            true,
            true,
            Vec::new(),
            Vec::new(),
            7,
            false,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let GovWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_GOV_POLICY_SCOPE_FAILED
        );
    }

    #[test]
    fn at_gov_08_resolved_decision_contract_maps_allow_hold_block_rollback() {
        let hold = GovWiringOutcome::Refused(
            GovRefuse::v1(
                GovCapabilityId::GovPolicyEvaluate,
                ReasonCodeId(901),
                "hold".to_string(),
            )
            .unwrap(),
        );
        assert_eq!(
            hold.resolved_decision().unwrap().class,
            GovDecisionClass::Hold
        );

        let allow_bundle =
            GovForwardBundle::v1(CorrelationId(1), TurnId(1), policy_ok(), decision_ok()).unwrap();
        assert_eq!(allow_bundle.decision_class().class, GovDecisionClass::Allow);

        let block_bundle =
            GovForwardBundle::v1(CorrelationId(1), TurnId(1), policy_ok(), decision_blocked())
                .unwrap();
        assert_eq!(block_bundle.decision_class().class, GovDecisionClass::Block);

        let rollback_bundle = GovForwardBundle::v1(
            CorrelationId(1),
            TurnId(1),
            policy_ok(),
            decision_allow_rollback(),
        )
        .unwrap();
        assert_eq!(
            rollback_bundle.decision_class().class,
            GovDecisionClass::Rollback
        );
    }

    #[test]
    fn at_gov_09_cohort_safety_gates_require_all_required_cohorts_pass() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let input = GovTurnInput::v2(
            CorrelationId(5201),
            TurnId(5301),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string()],
            vec!["SIM_1".to_string()],
            None,
            Some(GovArtifactVersion(2)),
            true,
            true,
            true,
            vec!["lang:en".to_string(), "noise:high".to_string()],
            vec!["lang:en".to_string()],
            7,
            false,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let GovWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_GOV_COHORT_GATES_FAILED
        );
    }

    #[test]
    fn at_gov_10_stability_window_requires_7_to_14_days_before_activate() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let input = GovTurnInput::v2(
            CorrelationId(5201),
            TurnId(5301),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string()],
            vec!["SIM_1".to_string()],
            None,
            Some(GovArtifactVersion(2)),
            true,
            true,
            true,
            Vec::new(),
            Vec::new(),
            3,
            false,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let GovWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_GOV_STABILITY_WINDOW_FAILED
        );
    }

    #[test]
    fn at_gov_11_consent_revoke_forces_rollback_within_sla() {
        let engine = MockGovEngine {
            policy_response: Ph1GovResponse::GovPolicyEvaluateOk(policy_ok()),
            decision_response: Ph1GovResponse::GovDecisionComputeOk(decision_ok()),
        };
        let wiring = Ph1GovWiring::new(Ph1GovWiringConfig::mvp_v1(true), engine).unwrap();
        let input = GovTurnInput::v2(
            CorrelationId(5201),
            TurnId(5301),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string()],
            vec!["SIM_1".to_string()],
            None,
            Some(GovArtifactVersion(2)),
            true,
            true,
            true,
            Vec::new(),
            Vec::new(),
            7,
            true,
            true,
        )
        .unwrap();
        let outcome = wiring.run_turn(&input).unwrap();
        let GovWiringOutcome::Refused(refuse) = outcome else {
            panic!("expected refuse");
        };
        assert_eq!(
            refuse.reason_code,
            reason_codes::PH1_GOV_REVOCATION_REQUIRES_ROLLBACK
        );
    }

    #[test]
    fn at_gov_12_builder_dispatch_blocks_allowed_rollback_path() {
        let bundle = GovForwardBundle::v1(
            CorrelationId(1),
            TurnId(1),
            policy_ok(),
            decision_allow_rollback(),
        )
        .unwrap();
        let dispatch = bundle.to_builder_dispatch_ticket().unwrap();
        assert_eq!(
            dispatch,
            GovBuilderDispatchOutcome::NotDispatchedDecisionNotAllowed
        );
    }
}
