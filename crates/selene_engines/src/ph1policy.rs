#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use selene_kernel_contracts::ph1policy::{
    Ph1PolicyRequest, Ph1PolicyResponse, PolicyCapabilityId, PolicyPromptDecision,
    PolicyPromptDedupeDecideOk, PolicyPromptDedupeDecideRequest, PolicyRefuse,
    PolicyRulesetGetActiveOk, PolicyRulesetGetActiveRequest,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.POLICY reason-code namespace. Values are placeholders until global registry lock.
    pub const POLICY_FIELD_ALREADY_KNOWN: ReasonCodeId = ReasonCodeId(0x504F_0001);
    pub const POLICY_ALREADY_ASKED: ReasonCodeId = ReasonCodeId(0x504F_0002);
    pub const POLICY_CONFLICT_REQUIRES_ONE_ASK: ReasonCodeId = ReasonCodeId(0x504F_0003);
    pub const POLICY_NEXT_FIELD: ReasonCodeId = ReasonCodeId(0x504F_0004);
    pub const POLICY_RULESET_OK: ReasonCodeId = ReasonCodeId(0x504F_0005);

    pub const POLICY_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x504F_00F1);
    pub const POLICY_REQUIRED_FIELDS_EMPTY: ReasonCodeId = ReasonCodeId(0x504F_00F2);
    pub const POLICY_WORK_ORDER_SCOPE_INVALID: ReasonCodeId = ReasonCodeId(0x504F_00F3);
    pub const POLICY_RULESET_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x504F_00F4);
    pub const POLICY_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x504F_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PolicyConfig {
    pub max_required_fields: u8,
}

impl Ph1PolicyConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_required_fields: 32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PolicyRuntime {
    config: Ph1PolicyConfig,
}

impl Ph1PolicyRuntime {
    pub fn new(config: Ph1PolicyConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PolicyRequest) -> Ph1PolicyResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::POLICY_INPUT_SCHEMA_INVALID,
                "policy request failed contract validation",
            );
        }

        match req {
            Ph1PolicyRequest::PolicyPromptDedupeDecide(r) => self.run_prompt_dedupe_decide(r),
            Ph1PolicyRequest::PolicyRulesetGetActive(r) => self.run_ruleset_get_active(r),
        }
    }

    fn run_prompt_dedupe_decide(&self, req: &PolicyPromptDedupeDecideRequest) -> Ph1PolicyResponse {
        if req.required_fields.is_empty() {
            return self.refuse(
                PolicyCapabilityId::PolicyPromptDedupeDecide,
                reason_codes::POLICY_REQUIRED_FIELDS_EMPTY,
                "required_fields is empty",
            );
        }
        if req.required_fields.len() > self.config.max_required_fields as usize {
            return self.refuse(
                PolicyCapabilityId::PolicyPromptDedupeDecide,
                reason_codes::POLICY_INPUT_SCHEMA_INVALID,
                "required_fields exceeds runtime budget",
            );
        }
        if req.work_order_id.trim().is_empty() || req.tenant_id.trim().is_empty() {
            return self.refuse(
                PolicyCapabilityId::PolicyPromptDedupeDecide,
                reason_codes::POLICY_WORK_ORDER_SCOPE_INVALID,
                "tenant_id/work_order_id must be present",
            );
        }

        let known: BTreeSet<&str> = req
            .known_fields
            .iter()
            .chain(req.authoritative_prefill_fields.iter())
            .map(String::as_str)
            .collect();
        let asked: BTreeSet<&str> = req.asked_fields.iter().map(String::as_str).collect();
        let dedupe: BTreeSet<&str> = req.prompt_dedupe_keys.iter().map(String::as_str).collect();

        let unresolved: Vec<&str> = req
            .required_fields
            .iter()
            .map(String::as_str)
            .filter(|field| !known.contains(field))
            .collect();

        if unresolved.is_empty() {
            return self.prompt_ok(
                reason_codes::POLICY_FIELD_ALREADY_KNOWN,
                PolicyPromptDecision::Skip,
                None,
                None,
            );
        }

        for field in &unresolved {
            let key = prompt_dedupe_key(&req.tenant_id, &req.work_order_id, field);
            if !asked.contains(field) && !dedupe.contains(key.as_str()) {
                return self.prompt_ok(
                    reason_codes::POLICY_NEXT_FIELD,
                    PolicyPromptDecision::Ask,
                    Some((*field).to_string()),
                    Some(key),
                );
            }
        }

        if unresolved.len() == 1 {
            return self.prompt_ok(
                reason_codes::POLICY_ALREADY_ASKED,
                PolicyPromptDecision::Stop,
                None,
                None,
            );
        }

        self.prompt_ok(
            reason_codes::POLICY_CONFLICT_REQUIRES_ONE_ASK,
            PolicyPromptDecision::AskDifferentField,
            Some(unresolved[0].to_string()),
            None,
        )
    }

    fn run_ruleset_get_active(&self, req: &PolicyRulesetGetActiveRequest) -> Ph1PolicyResponse {
        if req.tenant_id.trim().is_empty() {
            return self.refuse(
                PolicyCapabilityId::PolicyRulesetGetActive,
                reason_codes::POLICY_RULESET_NOT_FOUND,
                "tenant_id is empty",
            );
        }

        let mut enabled_rules = vec!["PROMPT_DEDUPE".to_string(), "ONE_FIELD_ONLY".to_string()];
        if req.user_id.is_some() {
            enabled_rules.push("USER_SCOPE_HINT".to_string());
        }
        let policy_ruleset_version = "policy_v1".to_string();
        let ruleset_hash = ruleset_hash(
            &req.tenant_id,
            req.user_id.as_deref().unwrap_or("none"),
            req.now_ns,
        );

        match PolicyRulesetGetActiveOk::v1(
            reason_codes::POLICY_RULESET_OK,
            policy_ruleset_version,
            ruleset_hash,
            enabled_rules,
            true,
        ) {
            Ok(ok) => Ph1PolicyResponse::PolicyRulesetGetActiveOk(ok),
            Err(_) => self.refuse(
                PolicyCapabilityId::PolicyRulesetGetActive,
                reason_codes::POLICY_INTERNAL_PIPELINE_ERROR,
                "failed to construct ruleset output",
            ),
        }
    }

    fn prompt_ok(
        &self,
        reason_code: ReasonCodeId,
        decision: PolicyPromptDecision,
        field_to_ask: Option<String>,
        prompt_dedupe_key: Option<String>,
    ) -> Ph1PolicyResponse {
        match PolicyPromptDedupeDecideOk::v1(
            reason_code,
            decision,
            field_to_ask,
            prompt_dedupe_key,
            true,
        ) {
            Ok(ok) => Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok),
            Err(_) => self.refuse(
                PolicyCapabilityId::PolicyPromptDedupeDecide,
                reason_codes::POLICY_INTERNAL_PIPELINE_ERROR,
                "failed to construct prompt dedupe output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PolicyCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PolicyResponse {
        let out = PolicyRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PolicyRefuse::v1 must construct for static messages");
        Ph1PolicyResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1PolicyRequest) -> PolicyCapabilityId {
    match req {
        Ph1PolicyRequest::PolicyPromptDedupeDecide(_) => {
            PolicyCapabilityId::PolicyPromptDedupeDecide
        }
        Ph1PolicyRequest::PolicyRulesetGetActive(_) => PolicyCapabilityId::PolicyRulesetGetActive,
    }
}

fn prompt_dedupe_key(tenant_id: &str, work_order_id: &str, field: &str) -> String {
    format!("{tenant_id}:{work_order_id}:{field}")
}

fn ruleset_hash(tenant_id: &str, user_id: &str, now_ns: u64) -> String {
    let payload = format!("{tenant_id}|{user_id}|{}", now_ns / 1_000_000_000);
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in payload.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("h_{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1policy::{
        PolicyRequestEnvelope, PolicyRulesetGetActiveRequest,
    };

    fn runtime() -> Ph1PolicyRuntime {
        Ph1PolicyRuntime::new(Ph1PolicyConfig::mvp_v1())
    }

    fn env() -> PolicyRequestEnvelope {
        PolicyRequestEnvelope::v1(CorrelationId(7101), TurnId(8101), 8).unwrap()
    }

    #[test]
    fn at_policy_01_skip_when_required_fields_already_known() {
        let req = Ph1PolicyRequest::PolicyPromptDedupeDecide(
            PolicyPromptDedupeDecideRequest::v1(
                env(),
                "tenant_1".to_string(),
                "work_1".to_string(),
                1_000,
                vec!["amount".to_string()],
                vec!["amount".to_string()],
                vec![],
                vec![],
                vec![],
            )
            .unwrap(),
        );
        let out = runtime().run(&req);
        let Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok) = out else {
            panic!("expected prompt dedupe output");
        };
        assert_eq!(ok.decision, PolicyPromptDecision::Skip);
        assert_eq!(ok.reason_code, reason_codes::POLICY_FIELD_ALREADY_KNOWN);
    }

    #[test]
    fn at_policy_02_ask_next_required_field_with_dedupe_key() {
        let req = Ph1PolicyRequest::PolicyPromptDedupeDecide(
            PolicyPromptDedupeDecideRequest::v1(
                env(),
                "tenant_1".to_string(),
                "work_1".to_string(),
                1_000,
                vec!["amount".to_string(), "recipient".to_string()],
                vec![],
                vec![],
                vec![],
                vec![],
            )
            .unwrap(),
        );
        let out = runtime().run(&req);
        let Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok) = out else {
            panic!("expected prompt dedupe output");
        };
        assert_eq!(ok.decision, PolicyPromptDecision::Ask);
        assert_eq!(ok.field_to_ask.as_deref(), Some("amount"));
        assert_eq!(
            ok.prompt_dedupe_key.as_deref(),
            Some("tenant_1:work_1:amount")
        );
    }

    #[test]
    fn at_policy_03_stop_when_single_unresolved_field_was_already_asked() {
        let req = Ph1PolicyRequest::PolicyPromptDedupeDecide(
            PolicyPromptDedupeDecideRequest::v1(
                env(),
                "tenant_1".to_string(),
                "work_1".to_string(),
                1_000,
                vec!["amount".to_string()],
                vec![],
                vec!["amount".to_string()],
                vec!["tenant_1:work_1:amount".to_string()],
                vec![],
            )
            .unwrap(),
        );
        let out = runtime().run(&req);
        let Ph1PolicyResponse::PolicyPromptDedupeDecideOk(ok) = out else {
            panic!("expected prompt dedupe output");
        };
        assert_eq!(ok.decision, PolicyPromptDecision::Stop);
        assert_eq!(ok.reason_code, reason_codes::POLICY_ALREADY_ASKED);
    }

    #[test]
    fn at_policy_04_ruleset_get_active_is_deterministic() {
        let req = Ph1PolicyRequest::PolicyRulesetGetActive(
            PolicyRulesetGetActiveRequest::v1(
                env(),
                "tenant_1".to_string(),
                Some("user_1".to_string()),
                2_000_000_000,
            )
            .unwrap(),
        );
        let out = runtime().run(&req);
        let Ph1PolicyResponse::PolicyRulesetGetActiveOk(ok) = out else {
            panic!("expected ruleset output");
        };
        assert_eq!(ok.reason_code, reason_codes::POLICY_RULESET_OK);
        assert_eq!(ok.policy_ruleset_version, "policy_v1");
        assert!(ok.enabled_rules.contains(&"PROMPT_DEDUPE".to_string()));
    }
}
