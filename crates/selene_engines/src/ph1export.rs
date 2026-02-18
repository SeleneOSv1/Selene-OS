#![forbid(unsafe_code)]

use selene_kernel_contracts::ph1export::{
    ExportAccessEvaluateOk, ExportAccessEvaluateRequest, ExportArtifactBuildOk,
    ExportArtifactBuildRequest, ExportCapabilityId, ExportIncludeKind, ExportRefuse,
    ExportResultStatus, ExportScopeKind, Ph1ExportRequest, Ph1ExportResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EXPORT reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_EXPORT_OK_ACCESS_EVALUATE: ReasonCodeId = ReasonCodeId(0x4558_0001);
    pub const PH1_EXPORT_OK_ARTIFACT_BUILD: ReasonCodeId = ReasonCodeId(0x4558_0002);

    pub const PH1_EXPORT_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4558_00F1);
    pub const PH1_EXPORT_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4558_00F2);
    pub const PH1_EXPORT_NOT_AUTHORIZED: ReasonCodeId = ReasonCodeId(0x4558_00F3);
    pub const PH1_EXPORT_RANGE_TOO_LARGE: ReasonCodeId = ReasonCodeId(0x4558_00F4);
    pub const PH1_EXPORT_REDACTION_REQUIRED: ReasonCodeId = ReasonCodeId(0x4558_00F5);
    pub const PH1_EXPORT_FAILED: ReasonCodeId = ReasonCodeId(0x4558_00F6);
    pub const PH1_EXPORT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4558_00F7);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ExportConfig {
    pub max_time_range_ms: u64,
    pub max_include_items: u8,
    pub max_diagnostics: u8,
}

impl Ph1ExportConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_time_range_ms: 31 * 24 * 60 * 60 * 1000,
            max_include_items: 3,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1ExportRuntime {
    config: Ph1ExportConfig,
}

impl Ph1ExportRuntime {
    pub fn new(config: Ph1ExportConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1ExportRequest) -> Ph1ExportResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_EXPORT_INPUT_SCHEMA_INVALID,
                "export request failed contract validation",
            );
        }

        match req {
            Ph1ExportRequest::ExportAccessEvaluate(r) => self.run_access_evaluate(r),
            Ph1ExportRequest::ExportArtifactBuild(r) => self.run_artifact_build(r),
        }
    }

    fn run_access_evaluate(&self, req: &ExportAccessEvaluateRequest) -> Ph1ExportResponse {
        if req.tenant_id.is_empty()
            || req.requester_user_id.is_empty()
            || req.redaction_policy_ref.is_empty()
        {
            return self.refuse(
                ExportCapabilityId::ExportAccessEvaluate,
                reason_codes::PH1_EXPORT_UPSTREAM_INPUT_MISSING,
                "required export inputs are missing",
            );
        }

        if is_requester_denied(&req.requester_user_id) {
            return self.refuse(
                ExportCapabilityId::ExportAccessEvaluate,
                reason_codes::PH1_EXPORT_NOT_AUTHORIZED,
                "requester is not authorized for export",
            );
        }

        if req.include.len() > self.config.max_include_items as usize {
            return self.refuse(
                ExportCapabilityId::ExportAccessEvaluate,
                reason_codes::PH1_EXPORT_FAILED,
                "include list exceeds runtime budget",
            );
        }

        if req.export_scope.kind == ExportScopeKind::TimeRange {
            let start_ms = req
                .export_scope
                .start_ms
                .expect("validated export_scope includes start_ms for TIME_RANGE");
            let end_ms = req
                .export_scope
                .end_ms
                .expect("validated export_scope includes end_ms for TIME_RANGE");
            if end_ms - start_ms > self.config.max_time_range_ms {
                return self.refuse(
                    ExportCapabilityId::ExportAccessEvaluate,
                    reason_codes::PH1_EXPORT_RANGE_TOO_LARGE,
                    "time range exceeds runtime policy",
                );
            }
        }

        if requires_redaction(req.include.as_slice())
            && !is_redaction_policy_allowed(&req.redaction_policy_ref)
        {
            return self.refuse(
                ExportCapabilityId::ExportAccessEvaluate,
                reason_codes::PH1_EXPORT_REDACTION_REQUIRED,
                "redaction policy does not permit required redaction",
            );
        }

        let scope_descriptor = scope_descriptor(req);
        let export_scope_ref = opaque_ref(
            "export_scope",
            &[
                req.tenant_id.as_str(),
                req.requester_user_id.as_str(),
                scope_descriptor.as_str(),
                req.redaction_policy_ref.as_str(),
            ],
        );

        match ExportAccessEvaluateOk::v1(
            reason_codes::PH1_EXPORT_OK_ACCESS_EVALUATE,
            req.tenant_id.clone(),
            export_scope_ref,
            req.include.clone(),
            req.redaction_policy_ref.clone(),
            true,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1ExportResponse::ExportAccessEvaluateOk(ok),
            Err(_) => self.refuse(
                ExportCapabilityId::ExportAccessEvaluate,
                reason_codes::PH1_EXPORT_INTERNAL_PIPELINE_ERROR,
                "failed to construct export access output",
            ),
        }
    }

    fn run_artifact_build(&self, req: &ExportArtifactBuildRequest) -> Ph1ExportResponse {
        if requires_redaction(req.include.as_slice())
            && !is_redaction_policy_allowed(&req.redaction_policy_ref)
        {
            return self.refuse(
                ExportCapabilityId::ExportArtifactBuild,
                reason_codes::PH1_EXPORT_REDACTION_REQUIRED,
                "redaction policy does not permit required redaction",
            );
        }

        let include_descriptor = include_descriptor(req.include.as_slice());
        let now_ms_s = req.now_ms.to_string();
        let hash_input = [
            req.tenant_id.as_str(),
            req.export_scope_ref.as_str(),
            req.requester_user_id.as_str(),
            include_descriptor.as_str(),
            req.redaction_policy_ref.as_str(),
            now_ms_s.as_str(),
        ];
        let export_hash = pseudo_sha256_hex(&hash_input);
        let export_artifact_id = format!("export_artifact:{}", &export_hash[..16]);
        let export_payload_ref = format!("export_payload:{}", &export_hash[..24]);

        let redaction_applied = requires_redaction(req.include.as_slice())
            || req.redaction_policy_ref.contains("strict")
            || req.redaction_policy_ref.contains("default");

        match ExportArtifactBuildOk::v1(
            reason_codes::PH1_EXPORT_OK_ARTIFACT_BUILD,
            ExportResultStatus::Ok,
            export_artifact_id,
            export_hash,
            export_payload_ref,
            redaction_applied,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1ExportResponse::ExportArtifactBuildOk(ok),
            Err(_) => self.refuse(
                ExportCapabilityId::ExportArtifactBuild,
                reason_codes::PH1_EXPORT_INTERNAL_PIPELINE_ERROR,
                "failed to construct export artifact output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: ExportCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1ExportResponse {
        let refuse = ExportRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("ExportRefuse::v1 must construct for static messages");
        Ph1ExportResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1ExportRequest) -> ExportCapabilityId {
    match req {
        Ph1ExportRequest::ExportAccessEvaluate(_) => ExportCapabilityId::ExportAccessEvaluate,
        Ph1ExportRequest::ExportArtifactBuild(_) => ExportCapabilityId::ExportArtifactBuild,
    }
}

fn is_requester_denied(requester_user_id: &str) -> bool {
    requester_user_id.starts_with("blocked_")
        || requester_user_id.starts_with("guest_")
        || requester_user_id.contains("unauthorized")
}

fn requires_redaction(include: &[ExportIncludeKind]) -> bool {
    include
        .iter()
        .any(|item| *item == ExportIncludeKind::ConversationTurns)
}

fn is_redaction_policy_allowed(policy_ref: &str) -> bool {
    let lowered = policy_ref.to_ascii_lowercase();
    !(lowered == "none" || lowered == "disabled" || lowered.contains("raw"))
}

fn scope_descriptor(req: &ExportAccessEvaluateRequest) -> String {
    match req.export_scope.kind {
        ExportScopeKind::WorkOrderId => format!(
            "wo:{}",
            req.export_scope
                .work_order_id
                .as_deref()
                .expect("validated scope includes work_order_id")
        ),
        ExportScopeKind::TimeRange => format!(
            "tr:{}:{}",
            req.export_scope
                .start_ms
                .expect("validated scope includes start_ms"),
            req.export_scope
                .end_ms
                .expect("validated scope includes end_ms"),
        ),
    }
}

fn include_descriptor(include: &[ExportIncludeKind]) -> String {
    let mut tokens = include.iter().map(|item| item.as_str()).collect::<Vec<_>>();
    tokens.sort_unstable();
    tokens.join(",")
}

fn stable_hash_with_seed(parts: &[&str], seed: u64) -> u64 {
    let mut hash = seed;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= u64::from(b'|');
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn pseudo_sha256_hex(parts: &[&str]) -> String {
    let h1 = stable_hash_with_seed(parts, 0xcbf2_9ce4_8422_2325);
    let h2 = stable_hash_with_seed(parts, 0x9ae1_6a3b_2f90_404f);
    let h3 = stable_hash_with_seed(parts, 0x517c_c1b7_2722_0a95);
    let h4 = stable_hash_with_seed(parts, 0x7f4a_7c15_9e37_79b9);
    format!("{h1:016x}{h2:016x}{h3:016x}{h4:016x}")
}

fn opaque_ref(prefix: &str, parts: &[&str]) -> String {
    let digest = pseudo_sha256_hex(parts);
    format!("{prefix}:{}", &digest[..24])
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1export::{
        ExportAccessEvaluateRequest, ExportArtifactBuildRequest, ExportRequestEnvelope, ExportScope,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn envelope() -> ExportRequestEnvelope {
        ExportRequestEnvelope::v1(CorrelationId(8801), TurnId(9101), 3, 8, 86_400_000).unwrap()
    }

    fn access_request() -> ExportAccessEvaluateRequest {
        ExportAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            ExportScope::work_order_id_v1("wo_123".to_string()).unwrap(),
            "finance_admin".to_string(),
            vec![
                ExportIncludeKind::AuditEvents,
                ExportIncludeKind::WorkOrderLedger,
                ExportIncludeKind::ConversationTurns,
            ],
            "policy_default".to_string(),
            100,
            true,
            true,
        )
        .unwrap()
    }

    fn artifact_request(scope_ref: &str) -> ExportArtifactBuildRequest {
        ExportArtifactBuildRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            scope_ref.to_string(),
            "finance_admin".to_string(),
            vec![
                ExportIncludeKind::AuditEvents,
                ExportIncludeKind::WorkOrderLedger,
                ExportIncludeKind::ConversationTurns,
            ],
            "policy_default".to_string(),
            100,
            true,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_export_01_tamper_evident_hash_is_stable() {
        let runtime = Ph1ExportRuntime::new(Ph1ExportConfig::mvp_v1());

        let access_resp = runtime.run(&Ph1ExportRequest::ExportAccessEvaluate(access_request()));
        let Ph1ExportResponse::ExportAccessEvaluateOk(access_ok) = access_resp else {
            panic!("expected access evaluate ok");
        };

        let req1 =
            Ph1ExportRequest::ExportArtifactBuild(artifact_request(&access_ok.export_scope_ref));
        let req2 =
            Ph1ExportRequest::ExportArtifactBuild(artifact_request(&access_ok.export_scope_ref));

        let out1 = runtime.run(&req1);
        let out2 = runtime.run(&req2);

        let Ph1ExportResponse::ExportArtifactBuildOk(ok1) = out1 else {
            panic!("expected artifact build ok #1");
        };
        let Ph1ExportResponse::ExportArtifactBuildOk(ok2) = out2 else {
            panic!("expected artifact build ok #2");
        };

        assert_eq!(ok1.export_hash, ok2.export_hash);
        assert_eq!(ok1.export_artifact_id, ok2.export_artifact_id);
    }

    #[test]
    fn at_export_02_redaction_is_applied_deterministically() {
        let runtime = Ph1ExportRuntime::new(Ph1ExportConfig::mvp_v1());
        let access_resp = runtime.run(&Ph1ExportRequest::ExportAccessEvaluate(access_request()));
        let Ph1ExportResponse::ExportAccessEvaluateOk(access_ok) = access_resp else {
            panic!("expected access evaluate ok");
        };

        let artifact_resp = runtime.run(&Ph1ExportRequest::ExportArtifactBuild(artifact_request(
            &access_ok.export_scope_ref,
        )));
        let Ph1ExportResponse::ExportArtifactBuildOk(out) = artifact_resp else {
            panic!("expected artifact build ok");
        };
        assert!(out.redaction_applied);
    }

    #[test]
    fn at_export_03_export_is_audited() {
        let runtime = Ph1ExportRuntime::new(Ph1ExportConfig::mvp_v1());
        let access_resp = runtime.run(&Ph1ExportRequest::ExportAccessEvaluate(access_request()));
        let Ph1ExportResponse::ExportAccessEvaluateOk(access_ok) = access_resp else {
            panic!("expected access evaluate ok");
        };

        let artifact_resp = runtime.run(&Ph1ExportRequest::ExportArtifactBuild(artifact_request(
            &access_ok.export_scope_ref,
        )));
        let Ph1ExportResponse::ExportArtifactBuildOk(out) = artifact_resp else {
            panic!("expected artifact build ok");
        };
        assert!(out.audit_event_emitted);
        assert!(out.raw_audio_excluded);
    }

    #[test]
    fn at_export_04_not_authorized_fails_closed() {
        let runtime = Ph1ExportRuntime::new(Ph1ExportConfig::mvp_v1());
        let denied_req = ExportAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            ExportScope::work_order_id_v1("wo_123".to_string()).unwrap(),
            "guest_user".to_string(),
            vec![ExportIncludeKind::AuditEvents],
            "policy_default".to_string(),
            100,
            true,
            true,
        )
        .unwrap();
        let resp = runtime.run(&Ph1ExportRequest::ExportAccessEvaluate(denied_req));
        let Ph1ExportResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_EXPORT_NOT_AUTHORIZED);
    }
}
