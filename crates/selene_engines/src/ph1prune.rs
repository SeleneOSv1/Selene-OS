#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1prune::{
    Ph1PruneRequest, Ph1PruneResponse, PruneCapabilityId, PruneClarifyOrderOk,
    PruneClarifyOrderRequest, PruneMissingFieldsOk, PruneMissingFieldsRequest, PruneRefuse,
    PruneValidationStatus,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.PRUNE reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_PRUNE_OK_MISSING_FIELDS: ReasonCodeId = ReasonCodeId(0x5055_0001);
    pub const PH1_PRUNE_OK_CLARIFY_ORDER: ReasonCodeId = ReasonCodeId(0x5055_0002);

    pub const PH1_PRUNE_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x5055_00F1);
    pub const PH1_PRUNE_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x5055_00F2);
    pub const PH1_PRUNE_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x5055_00F3);
    pub const PH1_PRUNE_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x5055_00F4);
    pub const PH1_PRUNE_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x5055_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1PruneConfig {
    pub max_missing_fields: u8,
    pub max_diagnostics: u8,
}

impl Ph1PruneConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_missing_fields: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1PruneRuntime {
    config: Ph1PruneConfig,
}

impl Ph1PruneRuntime {
    pub fn new(config: Ph1PruneConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1PruneRequest) -> Ph1PruneResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_PRUNE_INPUT_SCHEMA_INVALID,
                "prune request failed contract validation",
            );
        }

        match req {
            Ph1PruneRequest::PruneMissingFields(r) => self.run_missing_fields(r),
            Ph1PruneRequest::PruneClarifyOrder(r) => self.run_clarify_order(r),
        }
    }

    fn run_missing_fields(&self, req: &PruneMissingFieldsRequest) -> Ph1PruneResponse {
        if req.required_fields_missing.is_empty() {
            return self.refuse(
                PruneCapabilityId::PruneMissingFields,
                reason_codes::PH1_PRUNE_UPSTREAM_INPUT_MISSING,
                "required_fields_missing is empty",
            );
        }

        let budget = min(
            req.envelope.max_missing_fields,
            self.config.max_missing_fields,
        ) as usize;
        if req.required_fields_missing.len() > budget {
            return self.refuse(
                PruneCapabilityId::PruneMissingFields,
                reason_codes::PH1_PRUNE_BUDGET_EXCEEDED,
                "required_fields_missing exceeds configured budget",
            );
        }

        let prefilled: BTreeSet<String> = req.prefilled_fields.iter().cloned().collect();
        let confirmed: BTreeSet<String> = req.confirmed_fields.iter().cloned().collect();

        let mut candidates: Vec<String> = req
            .required_fields_missing
            .iter()
            .filter(|field| !prefilled.contains(*field) && !confirmed.contains(*field))
            .cloned()
            .collect();
        candidates.dedup();

        if candidates.is_empty() {
            return self.refuse(
                PruneCapabilityId::PruneMissingFields,
                reason_codes::PH1_PRUNE_UPSTREAM_INPUT_MISSING,
                "all missing fields are already prefilled/confirmed",
            );
        }

        let mut signal_fields: BTreeSet<String> =
            req.uncertain_field_hints.iter().cloned().collect();
        signal_fields.extend(derive_fields_from_ambiguity_flags(&req.ambiguity_flags));

        let previous = req.previous_clarify_field.as_deref();
        candidates.sort_by(|a, b| {
            let score_a = score_field(a, previous, &signal_fields);
            let score_b = score_field(b, previous, &signal_fields);
            score_b
                .cmp(&score_a)
                .then(priority_rank(a).cmp(&priority_rank(b)))
                .then(a.cmp(b))
        });

        let ordered_missing_fields = candidates.into_iter().take(budget).collect::<Vec<_>>();
        let selected_missing_field = ordered_missing_fields.first().cloned().unwrap_or_default();

        match PruneMissingFieldsOk::v1(
            reason_codes::PH1_PRUNE_OK_MISSING_FIELDS,
            selected_missing_field,
            ordered_missing_fields,
            true,
        ) {
            Ok(ok) => Ph1PruneResponse::PruneMissingFieldsOk(ok),
            Err(_) => self.refuse(
                PruneCapabilityId::PruneMissingFields,
                reason_codes::PH1_PRUNE_INTERNAL_PIPELINE_ERROR,
                "failed to build prune missing-fields output",
            ),
        }
    }

    fn run_clarify_order(&self, req: &PruneClarifyOrderRequest) -> Ph1PruneResponse {
        if req.required_fields_missing.is_empty() || req.ordered_missing_fields.is_empty() {
            return self.refuse(
                PruneCapabilityId::PruneClarifyOrder,
                reason_codes::PH1_PRUNE_UPSTREAM_INPUT_MISSING,
                "required_fields_missing/ordered_missing_fields must be non-empty",
            );
        }

        let budget = min(
            req.envelope.max_missing_fields,
            self.config.max_missing_fields,
        ) as usize;
        if req.required_fields_missing.len() > budget || req.ordered_missing_fields.len() > budget {
            return self.refuse(
                PruneCapabilityId::PruneClarifyOrder,
                reason_codes::PH1_PRUNE_BUDGET_EXCEEDED,
                "clarify-order inputs exceed configured budget",
            );
        }

        let mut diagnostics: Vec<String> = Vec::new();
        let required_set: BTreeSet<String> = req.required_fields_missing.iter().cloned().collect();

        if req.selected_missing_field != req.ordered_missing_fields[0] {
            diagnostics.push("selected_not_first_in_ordered_list".to_string());
        }
        if !required_set.contains(&req.selected_missing_field) {
            diagnostics.push("selected_missing_field_not_required".to_string());
        }
        if req
            .ordered_missing_fields
            .iter()
            .any(|field| !required_set.contains(field))
        {
            diagnostics.push("ordered_fields_not_subset_of_required".to_string());
        }

        let mut seen: BTreeSet<&String> = BTreeSet::new();
        if req
            .ordered_missing_fields
            .iter()
            .any(|field| !seen.insert(field))
        {
            diagnostics.push("ordered_fields_contain_duplicate".to_string());
        }

        if let Some(previous) = &req.previous_clarify_field {
            if previous == &req.selected_missing_field
                && req
                    .ordered_missing_fields
                    .iter()
                    .any(|field| field != &req.selected_missing_field)
            {
                diagnostics.push("selected_repeats_previous_clarify".to_string());
            }
        }

        diagnostics.truncate(self.config.max_diagnostics as usize);
        let (status, reason_code) = if diagnostics.is_empty() {
            (
                PruneValidationStatus::Ok,
                reason_codes::PH1_PRUNE_OK_CLARIFY_ORDER,
            )
        } else {
            (
                PruneValidationStatus::Fail,
                reason_codes::PH1_PRUNE_VALIDATION_FAILED,
            )
        };

        match PruneClarifyOrderOk::v1(reason_code, status, diagnostics, true) {
            Ok(ok) => Ph1PruneResponse::PruneClarifyOrderOk(ok),
            Err(_) => self.refuse(
                PruneCapabilityId::PruneClarifyOrder,
                reason_codes::PH1_PRUNE_INTERNAL_PIPELINE_ERROR,
                "failed to build prune clarify-order output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: PruneCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1PruneResponse {
        let out = PruneRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("PruneRefuse::v1 must construct for static messages");
        Ph1PruneResponse::Refuse(out)
    }
}

fn capability_from_request(req: &Ph1PruneRequest) -> PruneCapabilityId {
    match req {
        Ph1PruneRequest::PruneMissingFields(_) => PruneCapabilityId::PruneMissingFields,
        Ph1PruneRequest::PruneClarifyOrder(_) => PruneCapabilityId::PruneClarifyOrder,
    }
}

fn derive_fields_from_ambiguity_flags(flags: &[String]) -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    for flag in flags {
        if flag.contains("date") || flag.contains("time") {
            fields.insert("when".to_string());
        }
        if flag.contains("amount") {
            fields.insert("amount".to_string());
        }
        if flag.contains("recipient") {
            fields.insert("recipient".to_string());
        }
        if flag.contains("reference") {
            fields.insert("reference_target".to_string());
        }
        if flag.contains("intent") {
            fields.insert("intent_choice".to_string());
        }
        if flag.contains("location") || flag.contains("place") {
            fields.insert("location".to_string());
        }
    }
    fields
}

fn score_field(field: &str, previous: Option<&str>, signal_fields: &BTreeSet<String>) -> i32 {
    let mut score: i32 = 0;
    if signal_fields.contains(field) {
        score += 100;
    }
    if previous.is_some_and(|p| p == field) {
        score -= 200;
    }
    score += 50 - min(priority_rank(field), 50) as i32;
    score
}

fn priority_rank(field: &str) -> usize {
    const PRIORITY_ORDER: [&str; 9] = [
        "intent_choice",
        "reference_target",
        "recipient",
        "amount",
        "when",
        "location",
        "target",
        "action",
        "message",
    ];
    PRIORITY_ORDER
        .iter()
        .position(|entry| entry == &field)
        .unwrap_or(1000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1prune::{PruneRequestEnvelope, PruneValidationStatus};

    fn runtime() -> Ph1PruneRuntime {
        Ph1PruneRuntime::new(Ph1PruneConfig::mvp_v1())
    }

    fn envelope(max_missing_fields: u8) -> PruneRequestEnvelope {
        PruneRequestEnvelope::v1(CorrelationId(1401), TurnId(101), max_missing_fields).unwrap()
    }

    #[test]
    fn at_prune_01_missing_fields_output_is_schema_valid() {
        let req = Ph1PruneRequest::PruneMissingFields(
            PruneMissingFieldsRequest::v1(
                envelope(8),
                vec!["amount".to_string(), "recipient".to_string()],
                vec![],
                vec!["amount".to_string()],
                vec![],
                vec![],
                None,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1PruneResponse::PruneMissingFieldsOk(ok) => {
                assert_eq!(ok.selected_missing_field, "amount".to_string());
            }
            _ => panic!("expected PruneMissingFieldsOk"),
        }
    }

    #[test]
    fn at_prune_02_ordering_is_deterministic() {
        let req = Ph1PruneRequest::PruneMissingFields(
            PruneMissingFieldsRequest::v1(
                envelope(8),
                vec![
                    "when".to_string(),
                    "amount".to_string(),
                    "recipient".to_string(),
                ],
                vec!["amount_ambiguous".to_string()],
                vec![],
                vec![],
                vec![],
                None,
            )
            .unwrap(),
        );

        let out1 = runtime().run(&req);
        let out2 = runtime().run(&req);

        let ordered1 = match out1 {
            Ph1PruneResponse::PruneMissingFieldsOk(ok) => ok.ordered_missing_fields,
            _ => panic!("expected PruneMissingFieldsOk"),
        };
        let ordered2 = match out2 {
            Ph1PruneResponse::PruneMissingFieldsOk(ok) => ok.ordered_missing_fields,
            _ => panic!("expected PruneMissingFieldsOk"),
        };
        assert_eq!(ordered1, ordered2);
    }

    #[test]
    fn at_prune_03_previous_clarify_field_is_penalized_when_alternatives_exist() {
        let req = Ph1PruneRequest::PruneMissingFields(
            PruneMissingFieldsRequest::v1(
                envelope(8),
                vec!["when".to_string(), "amount".to_string()],
                vec![],
                vec![],
                vec![],
                vec![],
                Some("when".to_string()),
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PruneResponse::PruneMissingFieldsOk(ok) => {
                assert_eq!(ok.selected_missing_field, "amount".to_string());
                assert_eq!(ok.ordered_missing_fields[0], "amount".to_string());
            }
            _ => panic!("expected PruneMissingFieldsOk"),
        }
    }

    #[test]
    fn at_prune_04_clarify_order_validation_fails_on_drift() {
        let req = Ph1PruneRequest::PruneClarifyOrder(
            PruneClarifyOrderRequest::v1(
                envelope(8),
                vec!["amount".to_string(), "recipient".to_string()],
                "recipient".to_string(),
                vec!["amount".to_string(), "recipient".to_string()],
                None,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1PruneResponse::PruneClarifyOrderOk(ok) => {
                assert_eq!(ok.validation_status, PruneValidationStatus::Fail);
            }
            _ => panic!("expected PruneClarifyOrderOk"),
        }
    }
}
