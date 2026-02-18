#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::BTreeSet;

use selene_kernel_contracts::ph1diag::{
    DiagCapabilityId, DiagCheckArea, DiagConsistencyCheckOk, DiagConsistencyCheckRequest,
    DiagDiagnosticFlag, DiagReasonSetBuildOk, DiagReasonSetBuildRequest, DiagRefuse,
    DiagValidationStatus, Ph1DiagRequest, Ph1DiagResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DIAG reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_DIAG_OK_CONSISTENCY_CHECK: ReasonCodeId = ReasonCodeId(0x4449_0001);
    pub const PH1_DIAG_OK_REASON_SET_BUILD: ReasonCodeId = ReasonCodeId(0x4449_0002);

    pub const PH1_DIAG_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4449_00F1);
    pub const PH1_DIAG_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4449_00F2);
    pub const PH1_DIAG_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x4449_00F3);
    pub const PH1_DIAG_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4449_00F4);
    pub const PH1_DIAG_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4449_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1DiagConfig {
    pub max_flags: u8,
    pub max_reason_set: u8,
    pub max_diagnostics: u8,
}

impl Ph1DiagConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_flags: 8,
            max_reason_set: 8,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1DiagRuntime {
    config: Ph1DiagConfig,
}

impl Ph1DiagRuntime {
    pub fn new(config: Ph1DiagConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1DiagRequest) -> Ph1DiagResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_DIAG_INPUT_SCHEMA_INVALID,
                "diag request failed contract validation",
            );
        }

        match req {
            Ph1DiagRequest::DiagConsistencyCheck(r) => self.run_consistency_check(r),
            Ph1DiagRequest::DiagReasonSetBuild(r) => self.run_reason_set_build(r),
        }
    }

    fn run_consistency_check(&self, req: &DiagConsistencyCheckRequest) -> Ph1DiagResponse {
        if req.intent_type.trim().is_empty() {
            return self.refuse(
                DiagCapabilityId::DiagConsistencyCheck,
                reason_codes::PH1_DIAG_UPSTREAM_INPUT_MISSING,
                "intent_type is empty",
            );
        }

        let max_flags = min(req.envelope.max_flags, self.config.max_flags) as usize;
        let diagnostic_flags = match build_expected_flags(req, max_flags) {
            Ok(flags) => flags,
            Err(_) => {
                return self.refuse(
                    DiagCapabilityId::DiagConsistencyCheck,
                    reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                    "failed to build diagnostic flags",
                );
            }
        };

        if diagnostic_flags.len() > max_flags {
            return self.refuse(
                DiagCapabilityId::DiagConsistencyCheck,
                reason_codes::PH1_DIAG_BUDGET_EXCEEDED,
                "diagnostic_flags exceeds budget",
            );
        }

        match DiagConsistencyCheckOk::v1(
            reason_codes::PH1_DIAG_OK_CONSISTENCY_CHECK,
            diagnostic_flags,
            true,
        ) {
            Ok(ok) => Ph1DiagResponse::DiagConsistencyCheckOk(ok),
            Err(_) => self.refuse(
                DiagCapabilityId::DiagConsistencyCheck,
                reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                "failed to construct diag consistency check output",
            ),
        }
    }

    fn run_reason_set_build(&self, req: &DiagReasonSetBuildRequest) -> Ph1DiagResponse {
        if req.intent_type.trim().is_empty() {
            return self.refuse(
                DiagCapabilityId::DiagReasonSetBuild,
                reason_codes::PH1_DIAG_UPSTREAM_INPUT_MISSING,
                "intent_type is empty",
            );
        }

        let max_flags = min(req.envelope.max_flags, self.config.max_flags) as usize;
        let expected_flags = match build_expected_flags_from_reason_request(req, max_flags) {
            Ok(flags) => flags,
            Err(_) => {
                return self.refuse(
                    DiagCapabilityId::DiagReasonSetBuild,
                    reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                    "failed to rebuild expected diagnostic flags",
                );
            }
        };

        let mut diagnostics: Vec<String> = Vec::new();
        compare_flags(
            req.diagnostic_flags.as_slice(),
            expected_flags.as_slice(),
            self.config.max_diagnostics as usize,
            &mut diagnostics,
        );
        diagnostics.truncate(self.config.max_diagnostics as usize);

        let reason_set = build_reason_set(
            req.diagnostic_flags.as_slice(),
            self.config.max_reason_set as usize,
        );
        if reason_set.len() > self.config.max_reason_set as usize {
            return self.refuse(
                DiagCapabilityId::DiagReasonSetBuild,
                reason_codes::PH1_DIAG_BUDGET_EXCEEDED,
                "reason_set exceeds budget",
            );
        }

        let (validation_status, reason_code) = if diagnostics.is_empty() {
            (
                DiagValidationStatus::Ok,
                reason_codes::PH1_DIAG_OK_REASON_SET_BUILD,
            )
        } else {
            (
                DiagValidationStatus::Fail,
                reason_codes::PH1_DIAG_VALIDATION_FAILED,
            )
        };

        match DiagReasonSetBuildOk::v1(
            reason_code,
            validation_status,
            reason_set,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1DiagResponse::DiagReasonSetBuildOk(ok),
            Err(_) => self.refuse(
                DiagCapabilityId::DiagReasonSetBuild,
                reason_codes::PH1_DIAG_INTERNAL_PIPELINE_ERROR,
                "failed to construct diag reason-set output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: DiagCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1DiagResponse {
        let r = DiagRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("DiagRefuse::v1 must construct for static message");
        Ph1DiagResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1DiagRequest) -> DiagCapabilityId {
    match req {
        Ph1DiagRequest::DiagConsistencyCheck(_) => DiagCapabilityId::DiagConsistencyCheck,
        Ph1DiagRequest::DiagReasonSetBuild(_) => DiagCapabilityId::DiagReasonSetBuild,
    }
}

fn build_expected_flags(
    req: &DiagConsistencyCheckRequest,
    max_flags: usize,
) -> Result<Vec<DiagDiagnosticFlag>, selene_kernel_contracts::ContractViolation> {
    let context = FlagContext {
        required_fields_missing: req.required_fields_missing.as_slice(),
        ambiguity_flags: req.ambiguity_flags.as_slice(),
        requires_confirmation: req.requires_confirmation,
        confirmation_received: req.confirmation_received,
        privacy_mode: req.privacy_mode,
        delivery_mode_requested: req.delivery_mode_requested,
        sensitive_memory_candidate_present: req.sensitive_memory_candidate_present,
        memory_permission_granted: req.memory_permission_granted,
    };
    build_flags(context, max_flags)
}

fn build_expected_flags_from_reason_request(
    req: &DiagReasonSetBuildRequest,
    max_flags: usize,
) -> Result<Vec<DiagDiagnosticFlag>, selene_kernel_contracts::ContractViolation> {
    let context = FlagContext {
        required_fields_missing: req.required_fields_missing.as_slice(),
        ambiguity_flags: req.ambiguity_flags.as_slice(),
        requires_confirmation: req.requires_confirmation,
        confirmation_received: req.confirmation_received,
        privacy_mode: req.privacy_mode,
        delivery_mode_requested: req.delivery_mode_requested,
        sensitive_memory_candidate_present: req.sensitive_memory_candidate_present,
        memory_permission_granted: req.memory_permission_granted,
    };
    build_flags(context, max_flags)
}

struct FlagContext<'a> {
    required_fields_missing: &'a [String],
    ambiguity_flags: &'a [String],
    requires_confirmation: bool,
    confirmation_received: bool,
    privacy_mode: bool,
    delivery_mode_requested: selene_kernel_contracts::ph1diag::DiagDeliveryMode,
    sensitive_memory_candidate_present: bool,
    memory_permission_granted: bool,
}

fn build_flags(
    context: FlagContext<'_>,
    max_flags: usize,
) -> Result<Vec<DiagDiagnosticFlag>, selene_kernel_contracts::ContractViolation> {
    let mut flags: Vec<DiagDiagnosticFlag> = Vec::new();

    if !context.required_fields_missing.is_empty() {
        flags.push(DiagDiagnosticFlag::v1(
            "clarify_missing_field".to_string(),
            DiagCheckArea::RequiredField,
            true,
            "missing_required_field".to_string(),
        )?);
    }

    if !context.ambiguity_flags.is_empty() {
        flags.push(DiagDiagnosticFlag::v1(
            "clarify_ambiguous_reference".to_string(),
            DiagCheckArea::Intent,
            true,
            "ambiguous_reference".to_string(),
        )?);
    }

    if context.requires_confirmation && !context.confirmation_received {
        flags.push(DiagDiagnosticFlag::v1(
            "confirmation_pending".to_string(),
            DiagCheckArea::Confirmation,
            true,
            "confirmation_required".to_string(),
        )?);
    }

    if context.privacy_mode
        && context.delivery_mode_requested
            == selene_kernel_contracts::ph1diag::DiagDeliveryMode::VoiceAllowed
    {
        flags.push(DiagDiagnosticFlag::v1(
            "privacy_delivery_conflict".to_string(),
            DiagCheckArea::PrivacyDelivery,
            true,
            "privacy_requires_text".to_string(),
        )?);
    }

    if context.sensitive_memory_candidate_present && !context.memory_permission_granted {
        flags.push(DiagDiagnosticFlag::v1(
            "memory_permission_required".to_string(),
            DiagCheckArea::MemorySafety,
            true,
            "memory_permission_required".to_string(),
        )?);
    }

    flags.truncate(max_flags);
    Ok(flags)
}

fn compare_flags(
    actual: &[DiagDiagnosticFlag],
    expected: &[DiagDiagnosticFlag],
    max_diagnostics: usize,
    diagnostics: &mut Vec<String>,
) {
    let actual_ids = actual
        .iter()
        .map(|flag| flag.flag_id.as_str())
        .collect::<BTreeSet<_>>();
    let expected_ids = expected
        .iter()
        .map(|flag| flag.flag_id.as_str())
        .collect::<BTreeSet<_>>();

    for expected_flag in expected {
        if !actual_ids.contains(expected_flag.flag_id.as_str()) {
            diagnostics.push(format!("{}_missing", expected_flag.flag_id));
            if diagnostics.len() >= max_diagnostics {
                return;
            }
        }
    }

    for actual_flag in actual {
        match expected
            .iter()
            .find(|flag| flag.flag_id == actual_flag.flag_id)
        {
            Some(expected_flag) => {
                if actual_flag.check_area != expected_flag.check_area {
                    diagnostics.push(format!("{}_check_area_mismatch", actual_flag.flag_id));
                }
                if actual_flag.is_blocking != expected_flag.is_blocking {
                    diagnostics.push(format!("{}_is_blocking_mismatch", actual_flag.flag_id));
                }
                if actual_flag.reason_hint != expected_flag.reason_hint {
                    diagnostics.push(format!("{}_reason_hint_mismatch", actual_flag.flag_id));
                }
            }
            None => diagnostics.push(format!("{}_unexpected", actual_flag.flag_id)),
        }

        if diagnostics.len() >= max_diagnostics {
            return;
        }
    }

    if actual_ids != expected_ids && diagnostics.len() < max_diagnostics {
        diagnostics.push("flag_id_set_mismatch".to_string());
    }
}

fn build_reason_set(flags: &[DiagDiagnosticFlag], max_reason_set: usize) -> Vec<String> {
    let mut reason_set = flags
        .iter()
        .map(|flag| flag.reason_hint.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    if reason_set.is_empty() {
        reason_set.push("diag_consistent".to_string());
    }
    reason_set.truncate(max_reason_set);
    reason_set
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1diag::{DiagDeliveryMode, DiagRequestEnvelope};
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1DiagRuntime {
        Ph1DiagRuntime::new(Ph1DiagConfig::mvp_v1())
    }

    fn envelope(max_flags: u8) -> DiagRequestEnvelope {
        DiagRequestEnvelope::v1(CorrelationId(1801), TurnId(141), max_flags).unwrap()
    }

    #[test]
    fn at_diag_01_consistency_check_output_is_schema_valid() {
        let req = Ph1DiagRequest::DiagConsistencyCheck(
            DiagConsistencyCheckRequest::v1(
                envelope(8),
                "QUERY_WEATHER".to_string(),
                vec!["location".to_string()],
                vec![],
                false,
                false,
                false,
                DiagDeliveryMode::VoiceAllowed,
                false,
                false,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1DiagResponse::DiagConsistencyCheckOk(ok) => {
                assert!(!ok.diagnostic_flags.is_empty());
                assert!(ok
                    .diagnostic_flags
                    .iter()
                    .any(|flag| flag.flag_id == "clarify_missing_field"));
            }
            _ => panic!("expected DiagConsistencyCheckOk"),
        }
    }

    #[test]
    fn at_diag_02_consistency_check_is_deterministic() {
        let req = Ph1DiagRequest::DiagConsistencyCheck(
            DiagConsistencyCheckRequest::v1(
                envelope(8),
                "MESSAGE_DRAFT".to_string(),
                vec!["recipient".to_string()],
                vec!["recipient_ambiguous".to_string()],
                true,
                false,
                true,
                DiagDeliveryMode::VoiceAllowed,
                true,
                false,
            )
            .unwrap(),
        );

        let runtime = runtime();
        let out_1 = runtime.run(&req);
        let out_2 = runtime.run(&req);

        let flags_1 = match out_1 {
            Ph1DiagResponse::DiagConsistencyCheckOk(ok) => ok
                .diagnostic_flags
                .iter()
                .map(|flag| flag.flag_id.clone())
                .collect::<Vec<_>>(),
            _ => panic!("expected DiagConsistencyCheckOk"),
        };
        let flags_2 = match out_2 {
            Ph1DiagResponse::DiagConsistencyCheckOk(ok) => ok
                .diagnostic_flags
                .iter()
                .map(|flag| flag.flag_id.clone())
                .collect::<Vec<_>>(),
            _ => panic!("expected DiagConsistencyCheckOk"),
        };
        assert_eq!(flags_1, flags_2);
    }

    #[test]
    fn at_diag_03_flag_budget_is_enforced() {
        let req = Ph1DiagRequest::DiagConsistencyCheck(
            DiagConsistencyCheckRequest::v1(
                envelope(2),
                "MESSAGE_DRAFT".to_string(),
                vec!["recipient".to_string()],
                vec!["recipient_ambiguous".to_string()],
                true,
                false,
                true,
                DiagDeliveryMode::VoiceAllowed,
                true,
                false,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1DiagResponse::DiagConsistencyCheckOk(ok) => {
                assert!(ok.diagnostic_flags.len() <= 2);
            }
            _ => panic!("expected DiagConsistencyCheckOk"),
        }
    }

    #[test]
    fn at_diag_04_reason_set_validation_fails_for_drifted_flags() {
        let drifted_flags = vec![DiagDiagnosticFlag::v1(
            "clarify_missing_field".to_string(),
            DiagCheckArea::RequiredField,
            true,
            "wrong_reason_hint".to_string(),
        )
        .unwrap()];

        let req = Ph1DiagRequest::DiagReasonSetBuild(
            DiagReasonSetBuildRequest::v1(
                envelope(8),
                "QUERY_WEATHER".to_string(),
                vec!["location".to_string()],
                vec![],
                false,
                false,
                false,
                DiagDeliveryMode::VoiceAllowed,
                false,
                false,
                drifted_flags,
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1DiagResponse::DiagReasonSetBuildOk(ok) => {
                assert_eq!(ok.validation_status, DiagValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_DIAG_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic == "clarify_missing_field_reason_hint_mismatch"));
            }
            _ => panic!("expected DiagReasonSetBuildOk"),
        }
    }
}
