#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1POLICY_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyCapabilityId {
    PolicyPromptDedupeDecide,
    PolicyRulesetGetActive,
}

impl PolicyCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            PolicyCapabilityId::PolicyPromptDedupeDecide => "POLICY_PROMPT_DEDUP_DECIDE",
            PolicyCapabilityId::PolicyRulesetGetActive => "POLICY_RULESET_GET_ACTIVE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PolicyPromptDecision {
    Ask,
    Skip,
    AskDifferentField,
    Stop,
}

impl PolicyPromptDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            PolicyPromptDecision::Ask => "ASK",
            PolicyPromptDecision::Skip => "SKIP",
            PolicyPromptDecision::AskDifferentField => "ASK_DIFFERENT_FIELD",
            PolicyPromptDecision::Stop => "STOP",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_required_fields: u8,
}

impl PolicyRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_required_fields: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_required_fields,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for PolicyRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_request_envelope.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_required_fields == 0 || self.max_required_fields > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_request_envelope.max_required_fields",
                reason: "must be within 1..=32",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPromptDedupeDecideRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PolicyRequestEnvelope,
    pub tenant_id: String,
    pub work_order_id: String,
    pub now_ns: u64,
    pub required_fields: Vec<String>,
    pub known_fields: Vec<String>,
    pub asked_fields: Vec<String>,
    pub prompt_dedupe_keys: Vec<String>,
    pub authoritative_prefill_fields: Vec<String>,
}

impl PolicyPromptDedupeDecideRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: PolicyRequestEnvelope,
        tenant_id: String,
        work_order_id: String,
        now_ns: u64,
        required_fields: Vec<String>,
        known_fields: Vec<String>,
        asked_fields: Vec<String>,
        prompt_dedupe_keys: Vec<String>,
        authoritative_prefill_fields: Vec<String>,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            envelope,
            tenant_id,
            work_order_id,
            now_ns,
            required_fields,
            known_fields,
            asked_fields,
            prompt_dedupe_keys,
            authoritative_prefill_fields,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PolicyPromptDedupeDecideRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_request.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token_ascii(
            "policy_prompt_dedupe_decide_request.tenant_id",
            &self.tenant_id,
            96,
        )?;
        validate_token_ascii(
            "policy_prompt_dedupe_decide_request.work_order_id",
            &self.work_order_id,
            96,
        )?;
        if self.now_ns == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_request.now_ns",
                reason: "must be > 0",
            });
        }
        if self.required_fields.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_request.required_fields",
                reason: "must not be empty",
            });
        }
        if self.required_fields.len() > self.envelope.max_required_fields as usize {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_request.required_fields",
                reason: "must not exceed envelope.max_required_fields",
            });
        }
        validate_field_list(
            "policy_prompt_dedupe_decide_request.required_fields",
            &self.required_fields,
            32,
        )?;
        validate_field_list(
            "policy_prompt_dedupe_decide_request.known_fields",
            &self.known_fields,
            256,
        )?;
        validate_field_list(
            "policy_prompt_dedupe_decide_request.asked_fields",
            &self.asked_fields,
            256,
        )?;
        validate_field_list(
            "policy_prompt_dedupe_decide_request.authoritative_prefill_fields",
            &self.authoritative_prefill_fields,
            256,
        )?;
        if self.prompt_dedupe_keys.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_request.prompt_dedupe_keys",
                reason: "must be <= 512",
            });
        }
        for key in &self.prompt_dedupe_keys {
            validate_token_ascii(
                "policy_prompt_dedupe_decide_request.prompt_dedupe_keys",
                key,
                128,
            )?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRulesetGetActiveRequest {
    pub schema_version: SchemaVersion,
    pub envelope: PolicyRequestEnvelope,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub now_ns: u64,
}

impl PolicyRulesetGetActiveRequest {
    pub fn v1(
        envelope: PolicyRequestEnvelope,
        tenant_id: String,
        user_id: Option<String>,
        now_ns: u64,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            envelope,
            tenant_id,
            user_id,
            now_ns,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for PolicyRulesetGetActiveRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_request.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token_ascii(
            "policy_ruleset_get_active_request.tenant_id",
            &self.tenant_id,
            96,
        )?;
        if let Some(user_id) = &self.user_id {
            validate_token_ascii("policy_ruleset_get_active_request.user_id", user_id, 96)?;
        }
        if self.now_ns == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_request.now_ns",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PolicyRequest {
    PolicyPromptDedupeDecide(PolicyPromptDedupeDecideRequest),
    PolicyRulesetGetActive(PolicyRulesetGetActiveRequest),
}

impl Validate for Ph1PolicyRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PolicyRequest::PolicyPromptDedupeDecide(r) => r.validate(),
            Ph1PolicyRequest::PolicyRulesetGetActive(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyPromptDedupeDecideOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PolicyCapabilityId,
    pub reason_code: ReasonCodeId,
    pub decision: PolicyPromptDecision,
    pub field_to_ask: Option<String>,
    pub prompt_dedupe_key: Option<String>,
    pub no_execution_authority: bool,
}

impl PolicyPromptDedupeDecideOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        decision: PolicyPromptDecision,
        field_to_ask: Option<String>,
        prompt_dedupe_key: Option<String>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            capability_id: PolicyCapabilityId::PolicyPromptDedupeDecide,
            reason_code,
            decision,
            field_to_ask,
            prompt_dedupe_key,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PolicyPromptDedupeDecideOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_ok.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PolicyCapabilityId::PolicyPromptDedupeDecide {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_ok.capability_id",
                reason: "must be POLICY_PROMPT_DEDUP_DECIDE",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "policy_prompt_dedupe_decide_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        match self.decision {
            PolicyPromptDecision::Ask => {
                validate_optional_field_to_ask(
                    "policy_prompt_dedupe_decide_ok.field_to_ask",
                    &self.field_to_ask,
                )?;
                validate_optional_dedupe_key(
                    "policy_prompt_dedupe_decide_ok.prompt_dedupe_key",
                    &self.prompt_dedupe_key,
                )?;
            }
            PolicyPromptDecision::AskDifferentField => {
                validate_optional_field_to_ask(
                    "policy_prompt_dedupe_decide_ok.field_to_ask",
                    &self.field_to_ask,
                )?;
            }
            PolicyPromptDecision::Skip | PolicyPromptDecision::Stop => {
                if self.field_to_ask.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "policy_prompt_dedupe_decide_ok.field_to_ask",
                        reason: "must be None when decision is SKIP or STOP",
                    });
                }
                if self.prompt_dedupe_key.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "policy_prompt_dedupe_decide_ok.prompt_dedupe_key",
                        reason: "must be None when decision is SKIP or STOP",
                    });
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRulesetGetActiveOk {
    pub schema_version: SchemaVersion,
    pub capability_id: PolicyCapabilityId,
    pub reason_code: ReasonCodeId,
    pub policy_ruleset_version: String,
    pub ruleset_hash: String,
    pub enabled_rules: Vec<String>,
    pub no_execution_authority: bool,
}

impl PolicyRulesetGetActiveOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        policy_ruleset_version: String,
        ruleset_hash: String,
        enabled_rules: Vec<String>,
        no_execution_authority: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            capability_id: PolicyCapabilityId::PolicyRulesetGetActive,
            reason_code,
            policy_ruleset_version,
            ruleset_hash,
            enabled_rules,
            no_execution_authority,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PolicyRulesetGetActiveOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_ok.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        if self.capability_id != PolicyCapabilityId::PolicyRulesetGetActive {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_ok.capability_id",
                reason: "must be POLICY_RULESET_GET_ACTIVE",
            });
        }
        if !self.no_execution_authority {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_ok.no_execution_authority",
                reason: "must be true",
            });
        }
        validate_token_ascii(
            "policy_ruleset_get_active_ok.policy_ruleset_version",
            &self.policy_ruleset_version,
            64,
        )?;
        validate_token_ascii(
            "policy_ruleset_get_active_ok.ruleset_hash",
            &self.ruleset_hash,
            128,
        )?;
        if self.enabled_rules.is_empty() || self.enabled_rules.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_ruleset_get_active_ok.enabled_rules",
                reason: "must contain 1..=64 entries",
            });
        }
        for rule in &self.enabled_rules {
            validate_token_ascii("policy_ruleset_get_active_ok.enabled_rules", rule, 64)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: PolicyCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl PolicyRefuse {
    pub fn v1(
        capability_id: PolicyCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1POLICY_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for PolicyRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1POLICY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "policy_refuse.schema_version",
                reason: "must match PH1POLICY_CONTRACT_VERSION",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "policy_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "policy_refuse.message",
                reason: "must be <= 512 chars",
            });
        }
        if self.message.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "policy_refuse.message",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1PolicyResponse {
    PolicyPromptDedupeDecideOk(PolicyPromptDedupeDecideOk),
    PolicyRulesetGetActiveOk(PolicyRulesetGetActiveOk),
    Refuse(PolicyRefuse),
}

impl Validate for Ph1PolicyResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1PolicyResponse::PolicyPromptDedupeDecideOk(r) => r.validate(),
            Ph1PolicyResponse::PolicyRulesetGetActiveOk(r) => r.validate(),
            Ph1PolicyResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_optional_field_to_ask(
    field: &'static str,
    value: &Option<String>,
) -> Result<(), ContractViolation> {
    let Some(value) = value else {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be present for this decision",
        });
    };
    validate_token_ascii(field, value, 64)
}

fn validate_optional_dedupe_key(
    field: &'static str,
    value: &Option<String>,
) -> Result<(), ContractViolation> {
    let Some(value) = value else {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be present for this decision",
        });
    };
    validate_token_ascii(field, value, 128)
}

fn validate_field_list(
    field: &'static str,
    items: &[String],
    max_len: usize,
) -> Result<(), ContractViolation> {
    if items.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "list exceeds allowed length",
        });
    }
    for item in items {
        validate_token_ascii(field, item, 64)?;
    }
    Ok(())
}

fn validate_token_ascii(
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    if value
        .chars()
        .any(|c| c.is_control() || c.is_ascii_whitespace())
    {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control or whitespace characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> PolicyRequestEnvelope {
        PolicyRequestEnvelope::v1(CorrelationId(7401), TurnId(8401), 8).unwrap()
    }

    #[test]
    fn policy_contract_01_prompt_request_requires_required_fields() {
        let req = PolicyPromptDedupeDecideRequest::v1(
            envelope(),
            "tenant_default".to_string(),
            "work_01".to_string(),
            10,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
        );
        assert!(req.is_err());
    }

    #[test]
    fn policy_contract_02_ask_requires_field_and_key() {
        let out = PolicyPromptDedupeDecideOk::v1(
            ReasonCodeId(11),
            PolicyPromptDecision::Ask,
            None,
            None,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn policy_contract_03_skip_forbids_field_and_key() {
        let out = PolicyPromptDedupeDecideOk::v1(
            ReasonCodeId(12),
            PolicyPromptDecision::Skip,
            Some("field_a".to_string()),
            Some("dedupe".to_string()),
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn policy_contract_04_ruleset_requires_no_execution_authority() {
        let out = PolicyRulesetGetActiveOk::v1(
            ReasonCodeId(13),
            "v1".to_string(),
            "hash_abc".to_string(),
            vec!["PROMPT_DEDUPE".to_string()],
            false,
        );
        assert!(out.is_err());
    }
}
