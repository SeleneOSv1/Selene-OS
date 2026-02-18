#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::ph1position::TenantId;
use crate::{ContractViolation, MonotonicTimeNs, ReasonCodeId, SchemaVersion, Validate};

pub const PH1TENANT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TenantCapabilityId {
    TenantPolicyEvaluate,
    TenantDecisionCompute,
}

impl TenantCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            TenantCapabilityId::TenantPolicyEvaluate => "TENANT_POLICY_EVALUATE",
            TenantCapabilityId::TenantDecisionCompute => "TENANT_DECISION_COMPUTE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TenantResolveStatus {
    Ok,
    NeedsClarify,
    Refused,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TenantSelectionSource {
    None,
    ExplicitSelection,
    DeterministicSingleCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TenantIdentityContext {
    VoiceAssertionOk { user_id: String },
    VoiceAssertionUnknown,
    SignedInUser { user_id: String },
}

impl Validate for TenantIdentityContext {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            TenantIdentityContext::VoiceAssertionOk { user_id }
            | TenantIdentityContext::SignedInUser { user_id } => {
                validate_token("tenant_identity_context.user_id", user_id, 96)?
            }
            TenantIdentityContext::VoiceAssertionUnknown => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantBinding {
    pub tenant_id: TenantId,
    pub policy_context_ref: String,
    pub default_locale: Option<String>,
    pub tenant_disabled: bool,
    pub tenant_policy_blocked: bool,
}

impl TenantBinding {
    pub fn v1(
        tenant_id: TenantId,
        policy_context_ref: String,
        default_locale: Option<String>,
        tenant_disabled: bool,
        tenant_policy_blocked: bool,
    ) -> Result<Self, ContractViolation> {
        let row = Self {
            tenant_id,
            policy_context_ref,
            default_locale,
            tenant_disabled,
            tenant_policy_blocked,
        };
        row.validate()?;
        Ok(row)
    }
}

impl Validate for TenantBinding {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.tenant_id.validate()?;
        validate_token(
            "tenant_binding.policy_context_ref",
            &self.policy_context_ref,
            128,
        )?;
        if let Some(default_locale) = &self.default_locale {
            validate_token("tenant_binding.default_locale", default_locale, 32)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_candidates: u8,
    pub max_missing_fields: u8,
    pub max_diagnostics: u8,
}

impl TenantRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_candidates: u8,
        max_missing_fields: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_candidates,
            max_missing_fields,
            max_diagnostics,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for TenantRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_request_envelope.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_candidates == 0 || self.max_candidates > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_request_envelope.max_candidates",
                reason: "must be within 1..=16",
            });
        }
        if self.max_missing_fields == 0 || self.max_missing_fields > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_request_envelope.max_missing_fields",
                reason: "must be within 1..=4",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantPolicyEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: TenantRequestEnvelope,
    pub identity_context: TenantIdentityContext,
    pub device_id: Option<String>,
    pub session_id: Option<String>,
    pub now_ns: MonotonicTimeNs,
    pub explicit_tenant_selection_token: Option<String>,
    pub explicit_tenant_id: Option<TenantId>,
    pub candidate_bindings: Vec<TenantBinding>,
    pub deterministic: bool,
    pub no_permission_decision: bool,
    pub no_cross_tenant_access: bool,
}

impl TenantPolicyEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: TenantRequestEnvelope,
        identity_context: TenantIdentityContext,
        device_id: Option<String>,
        session_id: Option<String>,
        now_ns: MonotonicTimeNs,
        explicit_tenant_selection_token: Option<String>,
        explicit_tenant_id: Option<TenantId>,
        candidate_bindings: Vec<TenantBinding>,
        deterministic: bool,
        no_permission_decision: bool,
        no_cross_tenant_access: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            envelope,
            identity_context,
            device_id,
            session_id,
            now_ns,
            explicit_tenant_selection_token,
            explicit_tenant_id,
            candidate_bindings,
            deterministic,
            no_permission_decision,
            no_cross_tenant_access,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for TenantPolicyEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.identity_context.validate()?;

        if let Some(device_id) = &self.device_id {
            validate_token("tenant_policy_evaluate_request.device_id", device_id, 96)?;
        }
        if let Some(session_id) = &self.session_id {
            validate_token("tenant_policy_evaluate_request.session_id", session_id, 96)?;
        }
        if self.now_ns.0 == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.now_ns",
                reason: "must be > 0",
            });
        }
        if let Some(explicit_tenant_selection_token) = &self.explicit_tenant_selection_token {
            validate_token(
                "tenant_policy_evaluate_request.explicit_tenant_selection_token",
                explicit_tenant_selection_token,
                128,
            )?;
        }
        if let Some(explicit_tenant_id) = &self.explicit_tenant_id {
            explicit_tenant_id.validate()?;
        }
        if self.candidate_bindings.len() > self.envelope.max_candidates as usize {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.candidate_bindings",
                reason: "exceeds envelope max_candidates",
            });
        }
        let mut seen_tenant_ids = BTreeSet::new();
        for binding in &self.candidate_bindings {
            binding.validate()?;
            if !seen_tenant_ids.insert(binding.tenant_id.as_str().to_string()) {
                return Err(ContractViolation::InvalidValue {
                    field: "tenant_policy_evaluate_request.candidate_bindings",
                    reason: "tenant_id must be unique",
                });
            }
        }
        if self.explicit_tenant_selection_token.is_some() && self.explicit_tenant_id.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.explicit_tenant_id",
                reason: "must be present when explicit_tenant_selection_token is present",
            });
        }
        if let Some(explicit_tenant_id) = &self.explicit_tenant_id {
            let found = self
                .candidate_bindings
                .iter()
                .any(|binding| binding.tenant_id == *explicit_tenant_id);
            if !found {
                return Err(ContractViolation::InvalidValue {
                    field: "tenant_policy_evaluate_request.explicit_tenant_id",
                    reason: "must exist in candidate_bindings when present",
                });
            }
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_permission_decision {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.no_permission_decision",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_access {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_request.no_cross_tenant_access",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantPolicyEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: TenantCapabilityId,
    pub reason_code: ReasonCodeId,
    pub identity_known: bool,
    pub candidate_count: u8,
    pub selected_tenant_id: Option<TenantId>,
    pub selection_source: TenantSelectionSource,
    pub multiple_match: bool,
    pub deterministic: bool,
    pub no_permission_decision: bool,
    pub no_cross_tenant_access: bool,
}

impl TenantPolicyEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        identity_known: bool,
        candidate_count: u8,
        selected_tenant_id: Option<TenantId>,
        selection_source: TenantSelectionSource,
        multiple_match: bool,
        deterministic: bool,
        no_permission_decision: bool,
        no_cross_tenant_access: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            capability_id: TenantCapabilityId::TenantPolicyEvaluate,
            reason_code,
            identity_known,
            candidate_count,
            selected_tenant_id,
            selection_source,
            multiple_match,
            deterministic,
            no_permission_decision,
            no_cross_tenant_access,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for TenantPolicyEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != TenantCapabilityId::TenantPolicyEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.capability_id",
                reason: "must be TENANT_POLICY_EVALUATE",
            });
        }
        if let Some(selected_tenant_id) = &self.selected_tenant_id {
            selected_tenant_id.validate()?;
        }
        if self.candidate_count > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.candidate_count",
                reason: "must be <= 16",
            });
        }
        if self.selected_tenant_id.is_none() && self.selection_source != TenantSelectionSource::None
        {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.selection_source",
                reason: "must be NONE when selected_tenant_id is absent",
            });
        }
        if self.selected_tenant_id.is_some() && self.selection_source == TenantSelectionSource::None
        {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.selection_source",
                reason: "must not be NONE when selected_tenant_id is present",
            });
        }
        if self.multiple_match && self.candidate_count < 2 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.multiple_match",
                reason: "requires candidate_count >= 2",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_permission_decision {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.no_permission_decision",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_access {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_policy_evaluate_ok.no_cross_tenant_access",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantDecisionComputeRequest {
    pub schema_version: SchemaVersion,
    pub envelope: TenantRequestEnvelope,
    pub identity_known: bool,
    pub candidate_count: u8,
    pub selected_tenant_id: Option<TenantId>,
    pub selected_policy_context_ref: Option<String>,
    pub selected_default_locale: Option<String>,
    pub selected_tenant_disabled: bool,
    pub selected_tenant_policy_blocked: bool,
    pub multiple_match: bool,
    pub deterministic: bool,
    pub no_permission_decision: bool,
    pub no_cross_tenant_access: bool,
}

impl TenantDecisionComputeRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: TenantRequestEnvelope,
        identity_known: bool,
        candidate_count: u8,
        selected_tenant_id: Option<TenantId>,
        selected_policy_context_ref: Option<String>,
        selected_default_locale: Option<String>,
        selected_tenant_disabled: bool,
        selected_tenant_policy_blocked: bool,
        multiple_match: bool,
        deterministic: bool,
        no_permission_decision: bool,
        no_cross_tenant_access: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            envelope,
            identity_known,
            candidate_count,
            selected_tenant_id,
            selected_policy_context_ref,
            selected_default_locale,
            selected_tenant_disabled,
            selected_tenant_policy_blocked,
            multiple_match,
            deterministic,
            no_permission_decision,
            no_cross_tenant_access,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for TenantDecisionComputeRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.candidate_count > self.envelope.max_candidates {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.candidate_count",
                reason: "must be <= envelope.max_candidates",
            });
        }
        if let Some(selected_tenant_id) = &self.selected_tenant_id {
            selected_tenant_id.validate()?;
        }
        if let Some(selected_policy_context_ref) = &self.selected_policy_context_ref {
            validate_token(
                "tenant_decision_compute_request.selected_policy_context_ref",
                selected_policy_context_ref,
                128,
            )?;
        }
        if let Some(selected_default_locale) = &self.selected_default_locale {
            validate_token(
                "tenant_decision_compute_request.selected_default_locale",
                selected_default_locale,
                32,
            )?;
        }
        if self.selected_tenant_id.is_some() && self.selected_policy_context_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.selected_policy_context_ref",
                reason: "must be present when selected_tenant_id is present",
            });
        }
        if self.selected_tenant_id.is_none()
            && (self.selected_policy_context_ref.is_some()
                || self.selected_default_locale.is_some()
                || self.selected_tenant_disabled
                || self.selected_tenant_policy_blocked)
        {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.selected_tenant_id",
                reason: "selected tenant detail fields must be absent/false when selected_tenant_id is absent",
            });
        }
        if self.multiple_match && self.candidate_count < 2 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.multiple_match",
                reason: "requires candidate_count >= 2",
            });
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_permission_decision {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.no_permission_decision",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_access {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_request.no_cross_tenant_access",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantDecisionComputeOk {
    pub schema_version: SchemaVersion,
    pub capability_id: TenantCapabilityId,
    pub reason_code: ReasonCodeId,
    pub status: TenantResolveStatus,
    pub tenant_id: Option<TenantId>,
    pub policy_context_ref: Option<String>,
    pub default_locale: Option<String>,
    pub missing_fields: Vec<String>,
    pub deterministic: bool,
    pub no_permission_decision: bool,
    pub no_cross_tenant_access: bool,
}

impl TenantDecisionComputeOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        status: TenantResolveStatus,
        tenant_id: Option<TenantId>,
        policy_context_ref: Option<String>,
        default_locale: Option<String>,
        missing_fields: Vec<String>,
        deterministic: bool,
        no_permission_decision: bool,
        no_cross_tenant_access: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            capability_id: TenantCapabilityId::TenantDecisionCompute,
            reason_code,
            status,
            tenant_id,
            policy_context_ref,
            default_locale,
            missing_fields,
            deterministic,
            no_permission_decision,
            no_cross_tenant_access,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for TenantDecisionComputeOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != TenantCapabilityId::TenantDecisionCompute {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.capability_id",
                reason: "must be TENANT_DECISION_COMPUTE",
            });
        }
        if let Some(tenant_id) = &self.tenant_id {
            tenant_id.validate()?;
        }
        if let Some(policy_context_ref) = &self.policy_context_ref {
            validate_token(
                "tenant_decision_compute_ok.policy_context_ref",
                policy_context_ref,
                128,
            )?;
        }
        if let Some(default_locale) = &self.default_locale {
            validate_token(
                "tenant_decision_compute_ok.default_locale",
                default_locale,
                32,
            )?;
        }
        if self.missing_fields.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.missing_fields",
                reason: "must be <= 4",
            });
        }
        for missing_field in &self.missing_fields {
            validate_token(
                "tenant_decision_compute_ok.missing_fields",
                missing_field,
                64,
            )?;
        }
        match self.status {
            TenantResolveStatus::Ok => {
                if self.tenant_id.is_none() || self.policy_context_ref.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tenant_decision_compute_ok.tenant_id",
                        reason: "OK status requires tenant_id and policy_context_ref",
                    });
                }
                if !self.missing_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tenant_decision_compute_ok.missing_fields",
                        reason: "must be empty when status=OK",
                    });
                }
            }
            TenantResolveStatus::NeedsClarify => {
                if self.tenant_id.is_some() || self.policy_context_ref.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tenant_decision_compute_ok.tenant_id",
                        reason: "must be absent when status=NEEDS_CLARIFY",
                    });
                }
                if self.missing_fields.len() != 1 {
                    return Err(ContractViolation::InvalidValue {
                        field: "tenant_decision_compute_ok.missing_fields",
                        reason: "must contain exactly one field when status=NEEDS_CLARIFY",
                    });
                }
            }
            TenantResolveStatus::Refused | TenantResolveStatus::Fail => {
                if !self.missing_fields.is_empty() {
                    return Err(ContractViolation::InvalidValue {
                        field: "tenant_decision_compute_ok.missing_fields",
                        reason: "must be empty when status=REFUSED or FAIL",
                    });
                }
            }
        }
        if !self.deterministic {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.deterministic",
                reason: "must be true",
            });
        }
        if !self.no_permission_decision {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.no_permission_decision",
                reason: "must be true",
            });
        }
        if !self.no_cross_tenant_access {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_decision_compute_ok.no_cross_tenant_access",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TenantRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: TenantCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl TenantRefuse {
    pub fn v1(
        capability_id: TenantCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1TENANT_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for TenantRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1TENANT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "tenant_refuse.schema_version",
                reason: "must match PH1TENANT_CONTRACT_VERSION",
            });
        }
        validate_text("tenant_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1TenantRequest {
    TenantPolicyEvaluate(TenantPolicyEvaluateRequest),
    TenantDecisionCompute(TenantDecisionComputeRequest),
}

impl Validate for Ph1TenantRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1TenantRequest::TenantPolicyEvaluate(req) => req.validate(),
            Ph1TenantRequest::TenantDecisionCompute(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1TenantResponse {
    TenantPolicyEvaluateOk(TenantPolicyEvaluateOk),
    TenantDecisionComputeOk(TenantDecisionComputeOk),
    Refuse(TenantRefuse),
}

impl Validate for Ph1TenantResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1TenantResponse::TenantPolicyEvaluateOk(out) => out.validate(),
            Ph1TenantResponse::TenantDecisionComputeOk(out) => out.validate(),
            Ph1TenantResponse::Refuse(out) => out.validate(),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> TenantRequestEnvelope {
        TenantRequestEnvelope::v1(CorrelationId(7401), TurnId(8401), 8, 2, 8).unwrap()
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

    #[test]
    fn at_tenant_01_needs_clarify_requires_exactly_one_missing_field() {
        let out = TenantDecisionComputeOk::v1(
            ReasonCodeId(1),
            TenantResolveStatus::NeedsClarify,
            None,
            None,
            None,
            vec![],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_tenant_02_explicit_selection_must_exist_in_candidates() {
        let req = TenantPolicyEvaluateRequest::v1(
            envelope(),
            TenantIdentityContext::SignedInUser {
                user_id: "user_1".to_string(),
            },
            Some("device_1".to_string()),
            Some("session_1".to_string()),
            MonotonicTimeNs(1000),
            Some("tenant_token_1".to_string()),
            Some(TenantId::new("tenant_c").unwrap()),
            vec![binding("tenant_a"), binding("tenant_b")],
            true,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_tenant_03_ok_requires_tenant_and_policy_context() {
        let out = TenantDecisionComputeOk::v1(
            ReasonCodeId(1),
            TenantResolveStatus::Ok,
            Some(TenantId::new("tenant_a").unwrap()),
            None,
            Some("en-US".to_string()),
            vec![],
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_tenant_04_no_cross_tenant_access_invariant_is_required() {
        let req = TenantDecisionComputeRequest::v1(
            envelope(),
            true,
            1,
            Some(TenantId::new("tenant_a").unwrap()),
            Some("policy/default".to_string()),
            Some("en-US".to_string()),
            false,
            false,
            false,
            true,
            true,
            false,
        );
        assert!(req.is_err());
    }
}
