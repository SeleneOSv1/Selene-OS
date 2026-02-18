#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1PRUNE_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PruneCapabilityId {
    PruneMissingFields,
    PruneClarifyOrder,
}

impl PruneCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PruneCapabilityId::PruneMissingFields => "PRUNE_MISSING_FIELDS",
            PruneCapabilityId::PruneClarifyOrder => "PRUNE_CLARIFY_ORDER",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PruneValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_missing_fields: u8,
}

impl PruneRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_missing_fields: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_missing_fields,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PruneRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_request_envelope.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_missing_fields == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "prune_request_envelope.max_missing_fields",
                reason: "must be > 0",
            });
        }
        if self.max_missing_fields > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "prune_request_envelope.max_missing_fields",
                reason: "must be <= 32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneMissingFieldsRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PruneRequestEnvelope,
    pub required_fields_missing: Vec<String>,
    pub ambiguity_flags: Vec<String>,
    pub uncertain_field_hints: Vec<String>,
    pub prefilled_fields: Vec<String>,
    pub confirmed_fields: Vec<String>,
    pub previous_clarify_field: Option<String>,
}

impl PruneMissingFieldsRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PruneRequestEnvelope,
        required_fields_missing: Vec<String>,
        ambiguity_flags: Vec<String>,
        uncertain_field_hints: Vec<String>,
        prefilled_fields: Vec<String>,
        confirmed_fields: Vec<String>,
        previous_clarify_field: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            envelope,
            required_fields_missing,
            ambiguity_flags,
            uncertain_field_hints,
            prefilled_fields,
            confirmed_fields,
            previous_clarify_field,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PruneMissingFieldsRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_request.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.required_fields_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_request.required_fields_missing",
                reason: "must not be empty",
            });
        }
        validate_field_list(
            "prune_missing_fields_request.required_fields_missing",
            &self.required_fields_missing,
            32,
        )?;
        validate_token_list(
            "prune_missing_fields_request.ambiguity_flags",
            &self.ambiguity_flags,
            16,
        )?;
        validate_field_list(
            "prune_missing_fields_request.uncertain_field_hints",
            &self.uncertain_field_hints,
            16,
        )?;
        validate_field_list(
            "prune_missing_fields_request.prefilled_fields",
            &self.prefilled_fields,
            32,
        )?;
        validate_field_list(
            "prune_missing_fields_request.confirmed_fields",
            &self.confirmed_fields,
            32,
        )?;
        if let Some(field) = &self.previous_clarify_field {
            validate_field_key("prune_missing_fields_request.previous_clarify_field", field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneClarifyOrderRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PruneRequestEnvelope,
    pub required_fields_missing: Vec<String>,
    pub selected_missing_field: String,
    pub ordered_missing_fields: Vec<String>,
    pub previous_clarify_field: Option<String>,
}

impl PruneClarifyOrderRequest {
    pub fn v1(
        envelope: PruneRequestEnvelope,
        required_fields_missing: Vec<String>,
        selected_missing_field: String,
        ordered_missing_fields: Vec<String>,
        previous_clarify_field: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            envelope,
            required_fields_missing,
            selected_missing_field,
            ordered_missing_fields,
            previous_clarify_field,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PruneClarifyOrderRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_request.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.required_fields_missing.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_request.required_fields_missing",
                reason: "must not be empty",
            });
        }
        validate_field_list(
            "prune_clarify_order_request.required_fields_missing",
            &self.required_fields_missing,
            32,
        )?;
        validate_field_key(
            "prune_clarify_order_request.selected_missing_field",
            &self.selected_missing_field,
        )?;
        if self.ordered_missing_fields.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_request.ordered_missing_fields",
                reason: "must not be empty",
            });
        }
        validate_field_list(
            "prune_clarify_order_request.ordered_missing_fields",
            &self.ordered_missing_fields,
            32,
        )?;
        if let Some(field) = &self.previous_clarify_field {
            validate_field_key("prune_clarify_order_request.previous_clarify_field", field)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PruneRequest {
    PruneMissingFields(PruneMissingFieldsRequest),
    PruneClarifyOrder(PruneClarifyOrderRequest),
}

impl Validate for Ph1PruneRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PruneRequest::PruneMissingFields(req) => req.validate(),
            Ph1PruneRequest::PruneClarifyOrder(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneMissingFieldsOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PruneCapabilityId,
    pub reason_code: ReasonCodeId,
    pub selected_missing_field: String,
    pub ordered_missing_fields: Vec<String>,
    pub no_execution_authority: bool,
}

impl PruneMissingFieldsOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        selected_missing_field: String,
        ordered_missing_fields: Vec<String>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            capability_id: PruneCapabilityId::PruneMissingFields,
            reason_code,
            selected_missing_field,
            ordered_missing_fields,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PruneMissingFieldsOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_ok.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PruneCapabilityId::PruneMissingFields {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_ok.capability_id",
                reason: "must be PRUNE_MISSING_FIELDS",
            });
        }
        validate_field_key(
            "prune_missing_fields_ok.selected_missing_field",
            &self.selected_missing_field,
        )?;
        if self.ordered_missing_fields.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_ok.ordered_missing_fields",
                reason: "must not be empty",
            });
        }
        validate_field_list(
            "prune_missing_fields_ok.ordered_missing_fields",
            &self.ordered_missing_fields,
            32,
        )?;
        if !self
            .ordered_missing_fields
            .iter()
            .any(|f| f == &self.selected_missing_field)
        {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_ok.selected_missing_field",
                reason: "must exist in ordered_missing_fields",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "prune_missing_fields_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneClarifyOrderOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PruneCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: PruneValidationStatus,
    pub diagnostics: Vec<String>,
    pub no_execution_authority: bool,
}

impl PruneClarifyOrderOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: PruneValidationStatus,
        diagnostics: Vec<String>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            capability_id: PruneCapabilityId::PruneClarifyOrder,
            reason_code,
            validation_status,
            diagnostics,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PruneClarifyOrderOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_ok.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PruneCapabilityId::PruneClarifyOrder {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_ok.capability_id",
                reason: "must be PRUNE_CLARIFY_ORDER",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_ok.diagnostics",
                reason: "must be <= 16 entries",
            });
        }
        for diagnostic in &self.diagnostics {
            validate_token("prune_clarify_order_ok.diagnostics", diagnostic, 96)?;
        }
        if self.validation_status == PruneValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "prune_clarify_order_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PruneRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PruneCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PruneRefuse {
    pub fn v1(
        capability_id: PruneCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1PRUNE_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PruneRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1PRUNE_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "prune_refuse.schema_version",
                reason: "must match PH1PRUNE_CONTRACT_VERSION",
            });
        }
        validate_token("prune_refuse.message", &self.message, 256)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PruneResponse {
    PruneMissingFieldsOk(PruneMissingFieldsOk),
    PruneClarifyOrderOk(PruneClarifyOrderOk),
    Refuse(PruneRefuse),
}

impl Validate for Ph1PruneResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PruneResponse::PruneMissingFieldsOk(out) => out.validate(),
            Ph1PruneResponse::PruneClarifyOrderOk(out) => out.validate(),
            Ph1PruneResponse::Refuse(out) => out.validate(),
        }
    }
}

fn validate_field_list(
    field: &'static str,
    items: &[String],
    max_len: usize,
) -> Result<(), ContractViolation> {
    if items.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max list size",
        });
    }

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for item in items {
        validate_field_key(field, item)?;
        if !seen.insert(item.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain duplicate entries",
            });
        }
    }

    Ok(())
}

fn validate_token_list(
    field: &'static str,
    items: &[String],
    max_len: usize,
) -> Result<(), ContractViolation> {
    if items.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max list size",
        });
    }
    let mut seen: BTreeSet<&str> = BTreeSet::new();
    for item in items {
        validate_token(field, item, 64)?;
        if !seen.insert(item.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "must not contain duplicate entries",
            });
        }
    }
    Ok(())
}

fn validate_field_key(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if value.len() > 64 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 64 chars",
        });
    }
    if value.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must start with an ASCII lowercase letter or underscore",
        });
    }
    if !value
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII snake_case",
        });
    }
    if value.contains("__") {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain repeated underscores",
        });
    }
    Ok(())
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

    fn envelope(max_missing_fields: u8) -> PruneRequestEnvelope {
        PruneRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_missing_fields).unwrap()
    }

    #[test]
    fn prune_missing_fields_request_rejects_non_snake_case_field() {
        let req = PruneMissingFieldsRequest::v1(
            envelope(8),
            vec!["BadField".to_string()],
            vec![],
            vec![],
            vec!["existing_field".to_string()],
            vec!["confirmed_field".to_string()],
            None,
        );
        assert!(req.is_err());
    }

    #[test]
    fn prune_missing_fields_ok_requires_selected_field_to_exist_in_ordered_list() {
        let out = PruneMissingFieldsOk::v1(
            ReasonCodeId(1),
            "recipient".to_string(),
            vec!["amount".to_string(), "when".to_string()],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn prune_clarify_order_ok_requires_diagnostics_when_status_fail() {
        let out =
            PruneClarifyOrderOk::v1(ReasonCodeId(2), PruneValidationStatus::Fail, vec![], true);
        assert!(out.is_err());
    }

    #[test]
    fn prune_refuse_rejects_empty_message() {
        let out = PruneRefuse::v1(
            PruneCapabilityId::PruneMissingFields,
            ReasonCodeId(3),
            "".to_string(),
        );
        assert!(out.is_err());
    }
}
