#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1KMS_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KmsCapabilityId {
    KmsAccessEvaluate,
    KmsMaterialIssue,
}

impl KmsCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            KmsCapabilityId::KmsAccessEvaluate => "KMS_ACCESS_EVALUATE",
            KmsCapabilityId::KmsMaterialIssue => "KMS_MATERIAL_ISSUE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KmsOperation {
    GetHandle,
    IssueEphemeral,
    Rotate,
    Revoke,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KmsValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_allowlist_entries: u8,
    pub max_diagnostics: u8,
}

impl KmsRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_allowlist_entries: u8,
        max_diagnostics: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_allowlist_entries,
            max_diagnostics,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for KmsRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_request_envelope.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_allowlist_entries == 0 || self.max_allowlist_entries > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_request_envelope.max_allowlist_entries",
                reason: "must be within 1..=32",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsAccessEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KmsRequestEnvelope,
    pub tenant_id: String,
    pub secret_name: String,
    pub operation: KmsOperation,
    pub requester_engine_id: String,
    pub requester_user_id: Option<String>,
    pub requester_allowlist: Vec<String>,
    pub requested_ttl_ms: Option<u32>,
    pub now_ms: u64,
    pub require_admin_for_rotation: bool,
}

impl KmsAccessEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: KmsRequestEnvelope,
        tenant_id: String,
        secret_name: String,
        operation: KmsOperation,
        requester_engine_id: String,
        requester_user_id: Option<String>,
        requester_allowlist: Vec<String>,
        requested_ttl_ms: Option<u32>,
        now_ms: u64,
        require_admin_for_rotation: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            envelope,
            tenant_id,
            secret_name,
            operation,
            requester_engine_id,
            requester_user_id,
            requester_allowlist,
            requested_ttl_ms,
            now_ms,
            require_admin_for_rotation,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KmsAccessEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_request.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token("kms_access_evaluate_request.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "kms_access_evaluate_request.secret_name",
            &self.secret_name,
            128,
        )?;
        validate_token(
            "kms_access_evaluate_request.requester_engine_id",
            &self.requester_engine_id,
            96,
        )?;
        if let Some(requester_user_id) = &self.requester_user_id {
            validate_token(
                "kms_access_evaluate_request.requester_user_id",
                requester_user_id,
                96,
            )?;
        }

        if self.requester_allowlist.len() > self.envelope.max_allowlist_entries as usize {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_request.requester_allowlist",
                reason: "exceeds envelope max_allowlist_entries",
            });
        }
        let mut allowlist_set = BTreeSet::new();
        for entry in &self.requester_allowlist {
            validate_token("kms_access_evaluate_request.requester_allowlist", entry, 96)?;
            if !allowlist_set.insert(entry.as_str()) {
                return Err(ContractViolation::InvalidValue {
                    field: "kms_access_evaluate_request.requester_allowlist",
                    reason: "must be unique",
                });
            }
        }

        if self.now_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_request.now_ms",
                reason: "must be > 0",
            });
        }

        match self.operation {
            KmsOperation::IssueEphemeral => {
                if self.requested_ttl_ms.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_access_evaluate_request.requested_ttl_ms",
                        reason: "must be present when operation=ISSUE_EPHEMERAL",
                    });
                }
            }
            KmsOperation::Rotate | KmsOperation::Revoke => {
                if self.require_admin_for_rotation && self.requester_user_id.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_access_evaluate_request.requester_user_id",
                        reason: "must be present for rotate/revoke when admin is required",
                    });
                }
            }
            KmsOperation::GetHandle => {
                if self.requested_ttl_ms.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_access_evaluate_request.requested_ttl_ms",
                        reason: "must be absent when operation=GET_HANDLE",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsAccessEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KmsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub operation: KmsOperation,
    pub secret_ref: String,
    pub resolved_ttl_ms: Option<u32>,
    pub requester_authorized: bool,
    pub no_secret_value_emitted: bool,
    pub audit_safe: bool,
}

impl KmsAccessEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        operation: KmsOperation,
        secret_ref: String,
        resolved_ttl_ms: Option<u32>,
        requester_authorized: bool,
        no_secret_value_emitted: bool,
        audit_safe: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            capability_id: KmsCapabilityId::KmsAccessEvaluate,
            reason_code,
            operation,
            secret_ref,
            resolved_ttl_ms,
            requester_authorized,
            no_secret_value_emitted,
            audit_safe,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for KmsAccessEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KmsCapabilityId::KmsAccessEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.capability_id",
                reason: "must be KMS_ACCESS_EVALUATE",
            });
        }
        validate_token("kms_access_evaluate_ok.secret_ref", &self.secret_ref, 128)?;
        if self.operation == KmsOperation::IssueEphemeral && self.resolved_ttl_ms.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.resolved_ttl_ms",
                reason: "must be present when operation=ISSUE_EPHEMERAL",
            });
        }
        if !self.requester_authorized {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.requester_authorized",
                reason: "must be true for success output",
            });
        }
        if !self.no_secret_value_emitted {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.no_secret_value_emitted",
                reason: "must be true",
            });
        }
        if !self.audit_safe {
            return Err(ContractViolation::InvalidValue {
                field: "kms_access_evaluate_ok.audit_safe",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsMaterialIssueRequest {
    pub schema_version: SchemaVersion,
    pub envelope: KmsRequestEnvelope,
    pub tenant_id: String,
    pub operation: KmsOperation,
    pub secret_ref: String,
    pub requester_engine_id: String,
    pub requester_user_id: Option<String>,
    pub resolved_ttl_ms: Option<u32>,
    pub previous_version: Option<u32>,
    pub require_no_secret_value_emission: bool,
}

impl KmsMaterialIssueRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: KmsRequestEnvelope,
        tenant_id: String,
        operation: KmsOperation,
        secret_ref: String,
        requester_engine_id: String,
        requester_user_id: Option<String>,
        resolved_ttl_ms: Option<u32>,
        previous_version: Option<u32>,
        require_no_secret_value_emission: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            envelope,
            tenant_id,
            operation,
            secret_ref,
            requester_engine_id,
            requester_user_id,
            resolved_ttl_ms,
            previous_version,
            require_no_secret_value_emission,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for KmsMaterialIssueRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_request.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token("kms_material_issue_request.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "kms_material_issue_request.secret_ref",
            &self.secret_ref,
            128,
        )?;
        validate_token(
            "kms_material_issue_request.requester_engine_id",
            &self.requester_engine_id,
            96,
        )?;
        if let Some(requester_user_id) = &self.requester_user_id {
            validate_token(
                "kms_material_issue_request.requester_user_id",
                requester_user_id,
                96,
            )?;
        }

        match self.operation {
            KmsOperation::IssueEphemeral => {
                if self.resolved_ttl_ms.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_material_issue_request.resolved_ttl_ms",
                        reason: "must be present when operation=ISSUE_EPHEMERAL",
                    });
                }
            }
            KmsOperation::Rotate => {
                if self.previous_version.is_none() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_material_issue_request.previous_version",
                        reason: "must be present when operation=ROTATE",
                    });
                }
            }
            KmsOperation::GetHandle | KmsOperation::Revoke => {
                if self.resolved_ttl_ms.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "kms_material_issue_request.resolved_ttl_ms",
                        reason: "must be absent for get/revoke",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsMaterialIssueOk {
    pub schema_version: SchemaVersion,
    pub capability_id: KmsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: KmsValidationStatus,
    pub diagnostics: Vec<String>,
    pub operation: KmsOperation,
    pub secret_handle: Option<String>,
    pub ephemeral_credential_ref: Option<String>,
    pub rotated_version: Option<u32>,
    pub revoked: bool,
    pub no_secret_value_emitted: bool,
    pub audit_safe: bool,
}

impl KmsMaterialIssueOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: KmsValidationStatus,
        diagnostics: Vec<String>,
        operation: KmsOperation,
        secret_handle: Option<String>,
        ephemeral_credential_ref: Option<String>,
        rotated_version: Option<u32>,
        revoked: bool,
        no_secret_value_emitted: bool,
        audit_safe: bool,
    ) -> Result<Self, ContractViolation> {
        let ok = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            capability_id: KmsCapabilityId::KmsMaterialIssue,
            reason_code,
            validation_status,
            diagnostics,
            operation,
            secret_handle,
            ephemeral_credential_ref,
            rotated_version,
            revoked,
            no_secret_value_emitted,
            audit_safe,
        };
        ok.validate()?;
        Ok(ok)
    }
}

impl Validate for KmsMaterialIssueOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_ok.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        if self.capability_id != KmsCapabilityId::KmsMaterialIssue {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_ok.capability_id",
                reason: "must be KMS_MATERIAL_ISSUE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("kms_material_issue_ok.diagnostics", diagnostic, 96)?;
        }

        if self.validation_status == KmsValidationStatus::Ok {
            if !self.diagnostics.is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "kms_material_issue_ok.diagnostics",
                    reason: "must be empty when validation_status=OK",
                });
            }
            match self.operation {
                KmsOperation::GetHandle => {
                    if self.secret_handle.is_none() || self.ephemeral_credential_ref.is_some() {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.secret_handle",
                            reason: "GET_HANDLE requires secret_handle only",
                        });
                    }
                    if self.rotated_version.is_some() || self.revoked {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.rotated_version",
                            reason: "GET_HANDLE must not set rotated/revoked",
                        });
                    }
                }
                KmsOperation::IssueEphemeral => {
                    if self.ephemeral_credential_ref.is_none() || self.secret_handle.is_some() {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.ephemeral_credential_ref",
                            reason: "ISSUE_EPHEMERAL requires ephemeral ref only",
                        });
                    }
                    if self.rotated_version.is_some() || self.revoked {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.rotated_version",
                            reason: "ISSUE_EPHEMERAL must not set rotated/revoked",
                        });
                    }
                }
                KmsOperation::Rotate => {
                    if self.secret_handle.is_none() || self.rotated_version.is_none() {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.rotated_version",
                            reason: "ROTATE requires secret_handle + rotated_version",
                        });
                    }
                    if self.ephemeral_credential_ref.is_some() || self.revoked {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.ephemeral_credential_ref",
                            reason: "ROTATE must not set ephemeral/revoked",
                        });
                    }
                }
                KmsOperation::Revoke => {
                    if self.secret_handle.is_none() || !self.revoked {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.revoked",
                            reason: "REVOKE requires revoked=true and secret_handle",
                        });
                    }
                    if self.ephemeral_credential_ref.is_some() || self.rotated_version.is_some() {
                        return Err(ContractViolation::InvalidValue {
                            field: "kms_material_issue_ok.rotated_version",
                            reason: "REVOKE must not set rotated/ephemeral",
                        });
                    }
                }
            }
        }

        if let Some(secret_handle) = &self.secret_handle {
            validate_token("kms_material_issue_ok.secret_handle", secret_handle, 160)?;
        }
        if let Some(ephemeral_credential_ref) = &self.ephemeral_credential_ref {
            validate_token(
                "kms_material_issue_ok.ephemeral_credential_ref",
                ephemeral_credential_ref,
                160,
            )?;
        }

        if !self.no_secret_value_emitted {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_ok.no_secret_value_emitted",
                reason: "must be true",
            });
        }
        if !self.audit_safe {
            return Err(ContractViolation::InvalidValue {
                field: "kms_material_issue_ok.audit_safe",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: KmsCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl KmsRefuse {
    pub fn v1(
        capability_id: KmsCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1KMS_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for KmsRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1KMS_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "kms_refuse.schema_version",
                reason: "must match PH1KMS_CONTRACT_VERSION",
            });
        }
        validate_text("kms_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KmsRequest {
    KmsAccessEvaluate(KmsAccessEvaluateRequest),
    KmsMaterialIssue(KmsMaterialIssueRequest),
}

impl Validate for Ph1KmsRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KmsRequest::KmsAccessEvaluate(req) => req.validate(),
            Ph1KmsRequest::KmsMaterialIssue(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1KmsResponse {
    KmsAccessEvaluateOk(KmsAccessEvaluateOk),
    KmsMaterialIssueOk(KmsMaterialIssueOk),
    Refuse(KmsRefuse),
}

impl Validate for Ph1KmsResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1KmsResponse::KmsAccessEvaluateOk(ok) => ok.validate(),
            Ph1KmsResponse::KmsMaterialIssueOk(ok) => ok.validate(),
            Ph1KmsResponse::Refuse(r) => r.validate(),
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

    fn envelope() -> KmsRequestEnvelope {
        KmsRequestEnvelope::v1(CorrelationId(9301), TurnId(1201), 8, 6).unwrap()
    }

    #[test]
    fn at_kms_contract_01_issue_ephemeral_requires_ttl() {
        let req = KmsAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "api_key_store".to_string(),
            KmsOperation::IssueEphemeral,
            "PH1.TTS".to_string(),
            Some("user_admin".to_string()),
            vec!["PH1.TTS".to_string()],
            None,
            1,
            true,
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_kms_contract_02_rotation_admin_requirement_enforced() {
        let req = KmsAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            "api_key_store".to_string(),
            KmsOperation::Rotate,
            "PH1.OS".to_string(),
            None,
            vec!["PH1.OS".to_string()],
            None,
            1,
            true,
        );

        assert!(req.is_err());
    }

    #[test]
    fn at_kms_contract_03_material_ok_must_keep_secret_values_out() {
        let ok = KmsMaterialIssueOk::v1(
            ReasonCodeId(1),
            KmsValidationStatus::Ok,
            vec![],
            KmsOperation::GetHandle,
            Some("kms_handle:abcd".to_string()),
            None,
            None,
            false,
            true,
            true,
        )
        .unwrap();

        assert!(ok.validate().is_ok());
    }

    #[test]
    fn at_kms_contract_04_rotate_shape_requires_rotated_version() {
        let out = KmsMaterialIssueOk::v1(
            ReasonCodeId(1),
            KmsValidationStatus::Ok,
            vec![],
            KmsOperation::Rotate,
            Some("kms_handle:abcd".to_string()),
            None,
            None,
            false,
            true,
            true,
        );

        assert!(out.is_err());
    }
}
