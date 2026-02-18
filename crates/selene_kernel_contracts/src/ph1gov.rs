#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1GOV_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovCapabilityId {
    GovPolicyEvaluate,
    GovDecisionCompute,
}

impl GovCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            GovCapabilityId::GovPolicyEvaluate => "GOV_POLICY_EVALUATE",
            GovCapabilityId::GovDecisionCompute => "GOV_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovArtifactKind {
    Blueprint,
    Simulation,
    CapabilityMap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovRequestedAction {
    Activate,
    Deprecate,
    Rollback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GovDecisionStatus {
    Allowed,
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GovArtifactVersion(pub u32);

impl Validate for GovArtifactVersion {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_artifact_version",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_reference_ids: u8,
    pub max_diagnostics: u8,
    pub enterprise_mode_signature_required: bool,
}

impl GovRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_reference_ids: u8,
        max_diagnostics: u8,
        enterprise_mode_signature_required: bool,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_reference_ids,
            max_diagnostics,
            enterprise_mode_signature_required,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for GovRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_request_envelope.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_reference_ids == 0 || self.max_reference_ids > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_request_envelope.max_reference_ids",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "gov_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: GovRequestEnvelope,
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
    pub enforce_single_active_blueprint: bool,
}

impl GovPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: GovRequestEnvelope,
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
        enforce_single_active_blueprint: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            envelope,
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
            enforce_single_active_blueprint,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for GovPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_request.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.tenant_id.validate()?;
        validate_token(
            "gov_policy_evaluate_request.artifact_id",
            &self.artifact_id,
            128,
        )?;
        self.artifact_version.validate()?;
        validate_sha256(
            "gov_policy_evaluate_request.artifact_hash_sha256",
            &self.artifact_hash_sha256,
        )?;
        validate_token(
            "gov_policy_evaluate_request.requester_user_id",
            &self.requester_user_id,
            96,
        )?;

        if self.envelope.enterprise_mode_signature_required && self.signature_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_request.signature_ref",
                reason: "must be present in enterprise signature-required mode",
            });
        }
        if let Some(signature_ref) = &self.signature_ref {
            validate_token(
                "gov_policy_evaluate_request.signature_ref",
                signature_ref,
                128,
            )?;
        }

        validate_ref_list(
            "gov_policy_evaluate_request.required_reference_ids",
            &self.required_reference_ids,
            self.envelope.max_reference_ids as usize,
        )?;
        validate_ref_list(
            "gov_policy_evaluate_request.active_reference_ids",
            &self.active_reference_ids,
            self.envelope.max_reference_ids as usize,
        )?;
        if self.existing_active_versions.len() > self.envelope.max_reference_ids as usize {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_request.existing_active_versions",
                reason: "exceeds envelope max_reference_ids",
            });
        }
        let mut seen_versions = BTreeSet::new();
        for version in &self.existing_active_versions {
            version.validate()?;
            if !seen_versions.insert(version.0) {
                return Err(ContractViolation::InvalidValue {
                    field: "gov_policy_evaluate_request.existing_active_versions",
                    reason: "must be unique",
                });
            }
        }

        match self.requested_action {
            GovRequestedAction::Rollback => {
                if self.rollback_target_version.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_policy_evaluate_request.rollback_target_version",
                        reason: "must be present when requested_action=ROLLBACK",
                    });
                }
            }
            GovRequestedAction::Activate | GovRequestedAction::Deprecate => {
                if self.rollback_target_version.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_policy_evaluate_request.rollback_target_version",
                        reason: "must be absent unless requested_action=ROLLBACK",
                    });
                }
            }
        }
        if let Some(rollback_target_version) = self.rollback_target_version {
            rollback_target_version.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: GovCapabilityId,
    pub reason_code: ReasonCodeId,
    pub artifact_kind: GovArtifactKind,
    pub artifact_id: String,
    pub requested_action: GovRequestedAction,
    pub requester_authorized: bool,
    pub signature_valid: bool,
    pub references_active: bool,
    pub single_active_blueprint_ok: bool,
    pub rollback_target_present: bool,
    pub deterministic: bool,
    pub audit_required: bool,
}

impl GovPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        requested_action: GovRequestedAction,
        requester_authorized: bool,
        signature_valid: bool,
        references_active: bool,
        single_active_blueprint_ok: bool,
        rollback_target_present: bool,
        deterministic: bool,
        audit_required: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            capability_id: GovCapabilityId::GovPolicyEvaluate,
            reason_code,
            artifact_kind,
            artifact_id,
            requested_action,
            requester_authorized,
            signature_valid,
            references_active,
            single_active_blueprint_ok,
            rollback_target_present,
            deterministic,
            audit_required,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for GovPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_ok.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        if self.capability_id != GovCapabilityId::GovPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_ok.capability_id",
                reason: "must be GOV_POLICY_EVALUATE",
            });
        }
        validate_token("gov_policy_evaluate_ok.artifact_id", &self.artifact_id, 128)?;
        if self.requested_action == GovRequestedAction::Rollback && !self.rollback_target_present {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_ok.rollback_target_present",
                reason: "must be true when requested_action=ROLLBACK",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.audit_required {
            return Err(ContractViolation::InvalidValue {
                field: "gov_policy_evaluate_ok.audit_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: GovRequestEnvelope,
    pub artifact_kind: GovArtifactKind,
    pub artifact_id: String,
    pub artifact_version: GovArtifactVersion,
    pub requested_action: GovRequestedAction,
    pub current_active_version: Option<GovArtifactVersion>,
    pub rollback_target_version: Option<GovArtifactVersion>,
    pub requester_authorized: bool,
    pub signature_valid: bool,
    pub references_active: bool,
    pub single_active_blueprint_ok: bool,
    pub deterministic: bool,
    pub audit_required: bool,
}

impl GovDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: GovRequestEnvelope,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        artifact_version: GovArtifactVersion,
        requested_action: GovRequestedAction,
        current_active_version: Option<GovArtifactVersion>,
        rollback_target_version: Option<GovArtifactVersion>,
        requester_authorized: bool,
        signature_valid: bool,
        references_active: bool,
        single_active_blueprint_ok: bool,
        deterministic: bool,
        audit_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            envelope,
            artifact_kind,
            artifact_id,
            artifact_version,
            requested_action,
            current_active_version,
            rollback_target_version,
            requester_authorized,
            signature_valid,
            references_active,
            single_active_blueprint_ok,
            deterministic,
            audit_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for GovDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_request.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "gov_decision_compute_request.artifact_id",
            &self.artifact_id,
            128,
        )?;
        self.artifact_version.validate()?;
        if let Some(current_active_version) = self.current_active_version {
            current_active_version.validate()?;
        }
        if let Some(rollback_target_version) = self.rollback_target_version {
            rollback_target_version.validate()?;
        }
        match self.requested_action {
            GovRequestedAction::Rollback => {
                if self.rollback_target_version.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_decision_compute_request.rollback_target_version",
                        reason: "must be present when requested_action=ROLLBACK",
                    });
                }
            }
            GovRequestedAction::Activate | GovRequestedAction::Deprecate => {
                if self.rollback_target_version.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "gov_decision_compute_request.rollback_target_version",
                        reason: "must be absent unless requested_action=ROLLBACK",
                    });
                }
            }
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_request.deterministic",
                reason: "must be true",
            });
        }
        if !self.audit_required {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_request.audit_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: GovCapabilityId,
    pub reason_code: ReasonCodeId,
    pub decision: GovDecisionStatus,
    pub artifact_kind: GovArtifactKind,
    pub artifact_id: String,
    pub requested_action: GovRequestedAction,
    pub active_version: Option<GovArtifactVersion>,
    pub deterministic: bool,
    pub audit_event_required: bool,
    pub no_execution_authority: bool,
}

impl GovDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        decision: GovDecisionStatus,
        artifact_kind: GovArtifactKind,
        artifact_id: String,
        requested_action: GovRequestedAction,
        active_version: Option<GovArtifactVersion>,
        deterministic: bool,
        audit_event_required: bool,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            capability_id: GovCapabilityId::GovDecisionCompute,
            reason_code,
            decision,
            artifact_kind,
            artifact_id,
            requested_action,
            active_version,
            deterministic,
            audit_event_required,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for GovDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_ok.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        if self.capability_id != GovCapabilityId::GovDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_ok.capability_id",
                reason: "must be GOV_DECISION_COMPUTE",
            });
        }
        validate_token(
            "gov_decision_compute_ok.artifact_id",
            &self.artifact_id,
            128,
        )?;
        if let Some(active_version) = self.active_version {
            active_version.validate()?;
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.audit_event_required {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_ok.audit_event_required",
                reason: "must be true",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "gov_decision_compute_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: GovCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl GovRefuse {
    pub fn v1(
        capability_id: GovCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1GOV_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for GovRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1GOV_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "gov_refuse.schema_version",
                reason: "must match PH1GOV_CONTRACT_VERSION",
            });
        }
        validate_text("gov_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1GovRequest {
    GovPolicyEvaluate(GovPolicyEvaluateRequest),
    GovDecisionCompute(GovDecisionComputeRequest),
}

impl Validate for Ph1GovRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1GovRequest::GovPolicyEvaluate(req) => req.validate(),
            Ph1GovRequest::GovDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1GovResponse {
    GovPolicyEvaluateOk(GovPolicyEvaluateOk),
    GovDecisionComputeOk(GovDecisionComputeOk),
    Refuse(GovRefuse),
}

impl Validate for Ph1GovResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1GovResponse::GovPolicyEvaluateOk(out) => out.validate(),
            Ph1GovResponse::GovDecisionComputeOk(out) => out.validate(),
            Ph1GovResponse::Refuse(out) => out.validate(),
        }
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

fn validate_text(
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
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

fn validate_ref_list(
    field: &'static str,
    refs: &[String],
    max_items: usize,
) -> Result<(), ContractViolation> {
    if refs.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max reference IDs",
        });
    }
    let mut seen = BTreeSet::new();
    for r in refs {
        validate_token(field, r, 128)?;
        if !seen.insert(r.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must be unique",
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> GovRequestEnvelope {
        GovRequestEnvelope::v1(CorrelationId(7001), TurnId(8101), 8, 8, true).unwrap()
    }

    #[test]
    fn at_gov_01_signature_required_in_enterprise_mode() {
        let req = GovPolicyEvaluateRequest::v1(
            envelope(),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovArtifactVersion(3),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            None,
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(2)],
            vec!["SIM_1".to_string()],
            vec!["SIM_1".to_string()],
            None,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_gov_02_hash_must_be_sha256() {
        let req = GovPolicyEvaluateRequest::v1(
            envelope(),
            TenantId::new("tenant_demo").unwrap(),
            GovArtifactKind::Simulation,
            "sim_payroll".to_string(),
            GovArtifactVersion(2),
            "not-a-sha".to_string(),
            Some("sig_valid".to_string()),
            GovRequestedAction::Activate,
            "admin_user".to_string(),
            true,
            vec![GovArtifactVersion(1)],
            vec!["CAP_A".to_string()],
            vec!["CAP_A".to_string()],
            None,
            false,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_gov_03_rollback_requires_target_version() {
        let req = GovDecisionComputeRequest::v1(
            envelope(),
            GovArtifactKind::CapabilityMap,
            "ecm_cap".to_string(),
            GovArtifactVersion(4),
            GovRequestedAction::Rollback,
            Some(GovArtifactVersion(4)),
            None,
            true,
            true,
            true,
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_gov_04_decision_output_requires_no_execution_authority() {
        let out = GovDecisionComputeOk::v1(
            ReasonCodeId(1),
            GovDecisionStatus::Allowed,
            GovArtifactKind::Blueprint,
            "bp_payroll".to_string(),
            GovRequestedAction::Activate,
            Some(GovArtifactVersion(3)),
            true,
            true,
            false,
        );
        assert!(out.is_err());
    }
}
