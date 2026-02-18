#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1DIAG_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagCapabilityId {
    DiagConsistencyCheck,
    DiagReasonSetBuild,
}

impl DiagCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagCapabilityId::DiagConsistencyCheck => "DIAG_CONSISTENCY_CHECK",
            DiagCapabilityId::DiagReasonSetBuild => "DIAG_REASON_SET_BUILD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagDeliveryMode {
    VoiceAllowed,
    TextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagCheckArea {
    Intent,
    RequiredField,
    Confirmation,
    PrivacyDelivery,
    MemorySafety,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_flags: u8,
}

impl DiagRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_flags: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_flags,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for DiagRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_request_envelope.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_flags == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "diag_request_envelope.max_flags",
                reason: "must be > 0",
            });
        }
        if self.max_flags > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "diag_request_envelope.max_flags",
                reason: "must be <= 16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagDiagnosticFlag {
    pub schema_version: SchemaVersion,
    pub flag_id: String,
    pub check_area: DiagCheckArea,
    pub is_blocking: bool,
    pub reason_hint: String,
}

impl DiagDiagnosticFlag {
    pub fn v1(
        flag_id: String,
        check_area: DiagCheckArea,
        is_blocking: bool,
        reason_hint: String,
    ) -> Result<Self, ContractViolation> {
        let f = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            flag_id,
            check_area,
            is_blocking,
            reason_hint,
        };
        f.validate()?;
        Ok(f)
    }
}

impl Validate for DiagDiagnosticFlag {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_diagnostic_flag.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        validate_text("diag_diagnostic_flag.flag_id", &self.flag_id, 64)?;
        validate_text("diag_diagnostic_flag.reason_hint", &self.reason_hint, 96)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagConsistencyCheckRequest {
    pub schema_version: SchemaVersion,
    pub envelope: DiagRequestEnvelope,
    pub intent_type: String,
    pub required_fields_missing: Vec<String>,
    pub ambiguity_flags: Vec<String>,
    pub requires_confirmation: bool,
    pub confirmation_received: bool,
    pub privacy_mode: bool,
    pub delivery_mode_requested: DiagDeliveryMode,
    pub sensitive_memory_candidate_present: bool,
    pub memory_permission_granted: bool,
}

impl DiagConsistencyCheckRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: DiagRequestEnvelope,
        intent_type: String,
        required_fields_missing: Vec<String>,
        ambiguity_flags: Vec<String>,
        requires_confirmation: bool,
        confirmation_received: bool,
        privacy_mode: bool,
        delivery_mode_requested: DiagDeliveryMode,
        sensitive_memory_candidate_present: bool,
        memory_permission_granted: bool,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            envelope,
            intent_type,
            required_fields_missing,
            ambiguity_flags,
            requires_confirmation,
            confirmation_received,
            privacy_mode,
            delivery_mode_requested,
            sensitive_memory_candidate_present,
            memory_permission_granted,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DiagConsistencyCheckRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_consistency_check_request.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "diag_consistency_check_request.intent_type",
            &self.intent_type,
            96,
        )?;
        validate_string_list(
            "diag_consistency_check_request.required_fields_missing",
            &self.required_fields_missing,
            self.envelope.max_flags as usize,
            64,
        )?;
        validate_string_list(
            "diag_consistency_check_request.ambiguity_flags",
            &self.ambiguity_flags,
            self.envelope.max_flags as usize,
            64,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagReasonSetBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: DiagRequestEnvelope,
    pub intent_type: String,
    pub required_fields_missing: Vec<String>,
    pub ambiguity_flags: Vec<String>,
    pub requires_confirmation: bool,
    pub confirmation_received: bool,
    pub privacy_mode: bool,
    pub delivery_mode_requested: DiagDeliveryMode,
    pub sensitive_memory_candidate_present: bool,
    pub memory_permission_granted: bool,
    pub diagnostic_flags: Vec<DiagDiagnosticFlag>,
}

impl DiagReasonSetBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: DiagRequestEnvelope,
        intent_type: String,
        required_fields_missing: Vec<String>,
        ambiguity_flags: Vec<String>,
        requires_confirmation: bool,
        confirmation_received: bool,
        privacy_mode: bool,
        delivery_mode_requested: DiagDeliveryMode,
        sensitive_memory_candidate_present: bool,
        memory_permission_granted: bool,
        diagnostic_flags: Vec<DiagDiagnosticFlag>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            envelope,
            intent_type,
            required_fields_missing,
            ambiguity_flags,
            requires_confirmation,
            confirmation_received,
            privacy_mode,
            delivery_mode_requested,
            sensitive_memory_candidate_present,
            memory_permission_granted,
            diagnostic_flags,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DiagReasonSetBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_reason_set_build_request.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_text(
            "diag_reason_set_build_request.intent_type",
            &self.intent_type,
            96,
        )?;
        validate_string_list(
            "diag_reason_set_build_request.required_fields_missing",
            &self.required_fields_missing,
            self.envelope.max_flags as usize,
            64,
        )?;
        validate_string_list(
            "diag_reason_set_build_request.ambiguity_flags",
            &self.ambiguity_flags,
            self.envelope.max_flags as usize,
            64,
        )?;
        validate_diag_flags(
            "diag_reason_set_build_request.diagnostic_flags",
            &self.diagnostic_flags,
            self.envelope.max_flags as usize,
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DiagRequest {
    DiagConsistencyCheck(DiagConsistencyCheckRequest),
    DiagReasonSetBuild(DiagReasonSetBuildRequest),
}

impl Validate for Ph1DiagRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1DiagRequest::DiagConsistencyCheck(r) => r.validate(),
            Ph1DiagRequest::DiagReasonSetBuild(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagConsistencyCheckOk {
    pub schema_version: SchemaVersion,
    pub capability_id: DiagCapabilityId,
    pub reason_code: ReasonCodeId,
    pub diagnostic_flags: Vec<DiagDiagnosticFlag>,
    pub no_execution_authority: bool,
}

impl DiagConsistencyCheckOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        diagnostic_flags: Vec<DiagDiagnosticFlag>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            capability_id: DiagCapabilityId::DiagConsistencyCheck,
            reason_code,
            diagnostic_flags,
            no_execution_authority,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for DiagConsistencyCheckOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_consistency_check_ok.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != DiagCapabilityId::DiagConsistencyCheck {
            return Err(ContractViolation::InvalidValue {
                field: "diag_consistency_check_ok.capability_id",
                reason: "must be DIAG_CONSISTENCY_CHECK",
            });
        }
        validate_diag_flags(
            "diag_consistency_check_ok.diagnostic_flags",
            &self.diagnostic_flags,
            16,
        )?;
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "diag_consistency_check_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagReasonSetBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: DiagCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: DiagValidationStatus,
    pub reason_set: Vec<String>,
    pub diagnostics: Vec<String>,
    pub no_execution_authority: bool,
}

impl DiagReasonSetBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: DiagValidationStatus,
        reason_set: Vec<String>,
        diagnostics: Vec<String>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            capability_id: DiagCapabilityId::DiagReasonSetBuild,
            reason_code,
            validation_status,
            reason_set,
            diagnostics,
            no_execution_authority,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for DiagReasonSetBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_reason_set_build_ok.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        if self.capability_id != DiagCapabilityId::DiagReasonSetBuild {
            return Err(ContractViolation::InvalidValue {
                field: "diag_reason_set_build_ok.capability_id",
                reason: "must be DIAG_REASON_SET_BUILD",
            });
        }
        validate_string_list(
            "diag_reason_set_build_ok.reason_set",
            &self.reason_set,
            16,
            96,
        )?;
        validate_string_list(
            "diag_reason_set_build_ok.diagnostics",
            &self.diagnostics,
            16,
            96,
        )?;
        if self.validation_status == DiagValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "diag_reason_set_build_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "diag_reason_set_build_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: DiagCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl DiagRefuse {
    pub fn v1(
        capability_id: DiagCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DIAG_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DiagRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DIAG_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "diag_refuse.schema_version",
                reason: "must match PH1DIAG_CONTRACT_VERSION",
            });
        }
        validate_text("diag_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DiagResponse {
    DiagConsistencyCheckOk(DiagConsistencyCheckOk),
    DiagReasonSetBuildOk(DiagReasonSetBuildOk),
    Refuse(DiagRefuse),
}

impl Validate for Ph1DiagResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1DiagResponse::DiagConsistencyCheckOk(o) => o.validate(),
            Ph1DiagResponse::DiagReasonSetBuildOk(o) => o.validate(),
            Ph1DiagResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_diag_flags(
    field: &'static str,
    diagnostic_flags: &[DiagDiagnosticFlag],
    max_entries: usize,
) -> Result<(), ContractViolation> {
    if diagnostic_flags.len() > max_entries {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max entries",
        });
    }
    let mut flag_ids: BTreeSet<&str> = BTreeSet::new();
    for diagnostic_flag in diagnostic_flags {
        diagnostic_flag.validate()?;
        if !flag_ids.insert(diagnostic_flag.flag_id.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "flag_id entries must be unique",
            });
        }
    }
    Ok(())
}

fn validate_string_list(
    field: &'static str,
    values: &[String],
    max_entries: usize,
    max_len: usize,
) -> Result<(), ContractViolation> {
    if values.len() > max_entries {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max entries",
        });
    }
    let mut dedupe: BTreeSet<&str> = BTreeSet::new();
    for value in values {
        validate_text(field, value, max_len)?;
        if !dedupe.insert(value.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "entries must be unique",
            });
        }
    }
    Ok(())
}

fn validate_text(field: &'static str, text: &str, max_len: usize) -> Result<(), ContractViolation> {
    if text.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if text.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if text.chars().any(|c| c.is_control()) {
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

    fn envelope(max_flags: u8) -> DiagRequestEnvelope {
        DiagRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_flags).unwrap()
    }

    #[test]
    fn diag_consistency_check_request_rejects_empty_intent_type() {
        let req = DiagConsistencyCheckRequest::v1(
            envelope(4),
            "".to_string(),
            vec![],
            vec![],
            false,
            false,
            false,
            DiagDeliveryMode::VoiceAllowed,
            false,
            false,
        );
        assert!(req.is_err());
    }

    #[test]
    fn diag_reason_set_build_request_rejects_duplicate_flags() {
        let flag = DiagDiagnosticFlag::v1(
            "clarify_missing_field".to_string(),
            DiagCheckArea::RequiredField,
            true,
            "missing_required_field".to_string(),
        )
        .unwrap();

        let req = DiagReasonSetBuildRequest::v1(
            envelope(4),
            "QUERY_WEATHER".to_string(),
            vec!["location".to_string()],
            vec![],
            false,
            false,
            false,
            DiagDeliveryMode::VoiceAllowed,
            false,
            false,
            vec![flag.clone(), flag],
        );
        assert!(req.is_err());
    }

    #[test]
    fn diag_reason_set_build_ok_requires_diagnostics_on_fail() {
        let out = DiagReasonSetBuildOk::v1(
            ReasonCodeId(1),
            DiagValidationStatus::Fail,
            vec!["missing_required_field".to_string()],
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn diag_consistency_check_ok_requires_no_execution_authority_true() {
        let out = DiagConsistencyCheckOk::v1(ReasonCodeId(1), vec![], false);
        assert!(out.is_err());
    }
}
