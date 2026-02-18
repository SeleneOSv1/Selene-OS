#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1kms::{
    KmsAccessEvaluateOk, KmsAccessEvaluateRequest, KmsCapabilityId, KmsMaterialIssueOk,
    KmsMaterialIssueRequest, KmsOperation, KmsRefuse, KmsValidationStatus, Ph1KmsRequest,
    Ph1KmsResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KMS reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_KMS_OK_ACCESS_EVALUATE: ReasonCodeId = ReasonCodeId(0x4B4D_0001);
    pub const PH1_KMS_OK_MATERIAL_ISSUE: ReasonCodeId = ReasonCodeId(0x4B4D_0002);

    pub const PH1_KMS_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x4B4D_00F1);
    pub const PH1_KMS_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x4B4D_00F2);
    pub const PH1_KMS_NOT_AUTHORIZED: ReasonCodeId = ReasonCodeId(0x4B4D_00F3);
    pub const PH1_KMS_SECRET_NOT_FOUND: ReasonCodeId = ReasonCodeId(0x4B4D_00F4);
    pub const PH1_KMS_TTL_OUT_OF_BOUNDS: ReasonCodeId = ReasonCodeId(0x4B4D_00F5);
    pub const PH1_KMS_ROTATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B4D_00F6);
    pub const PH1_KMS_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B4D_00F7);
    pub const PH1_KMS_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B4D_00F8);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KmsConfig {
    pub min_ephemeral_ttl_ms: u32,
    pub max_ephemeral_ttl_ms: u32,
    pub default_ephemeral_ttl_ms: u32,
    pub max_diagnostics: u8,
}

impl Ph1KmsConfig {
    pub fn mvp_v1() -> Self {
        Self {
            min_ephemeral_ttl_ms: 30_000,
            max_ephemeral_ttl_ms: 3_600_000,
            default_ephemeral_ttl_ms: 300_000,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1KmsRuntime {
    config: Ph1KmsConfig,
}

impl Ph1KmsRuntime {
    pub fn new(config: Ph1KmsConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1KmsRequest) -> Ph1KmsResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_KMS_INPUT_SCHEMA_INVALID,
                "kms request failed contract validation",
            );
        }

        match req {
            Ph1KmsRequest::KmsAccessEvaluate(r) => self.run_access_evaluate(r),
            Ph1KmsRequest::KmsMaterialIssue(r) => self.run_material_issue(r),
        }
    }

    fn run_access_evaluate(&self, req: &KmsAccessEvaluateRequest) -> Ph1KmsResponse {
        if req.tenant_id.is_empty()
            || req.secret_name.is_empty()
            || req.requester_engine_id.is_empty()
        {
            return self.refuse(
                KmsCapabilityId::KmsAccessEvaluate,
                reason_codes::PH1_KMS_UPSTREAM_INPUT_MISSING,
                "required kms input is missing",
            );
        }

        if req.secret_name.contains("missing") {
            return self.refuse(
                KmsCapabilityId::KmsAccessEvaluate,
                reason_codes::PH1_KMS_SECRET_NOT_FOUND,
                "kms secret not found",
            );
        }

        if !requester_authorized(req) {
            return self.refuse(
                KmsCapabilityId::KmsAccessEvaluate,
                reason_codes::PH1_KMS_NOT_AUTHORIZED,
                "requester not authorized for kms operation",
            );
        }

        let resolved_ttl_ms = match req.operation {
            KmsOperation::IssueEphemeral => {
                let ttl = req
                    .requested_ttl_ms
                    .unwrap_or(self.config.default_ephemeral_ttl_ms);
                if ttl < self.config.min_ephemeral_ttl_ms || ttl > self.config.max_ephemeral_ttl_ms
                {
                    return self.refuse(
                        KmsCapabilityId::KmsAccessEvaluate,
                        reason_codes::PH1_KMS_TTL_OUT_OF_BOUNDS,
                        "requested ephemeral ttl is out of bounds",
                    );
                }
                Some(ttl)
            }
            _ => None,
        };

        let secret_ref = opaque_ref(
            "kms_ref",
            &[
                req.tenant_id.as_str(),
                req.secret_name.as_str(),
                operation_token(req.operation),
                req.requester_engine_id.as_str(),
            ],
        );

        match KmsAccessEvaluateOk::v1(
            reason_codes::PH1_KMS_OK_ACCESS_EVALUATE,
            req.operation,
            secret_ref,
            resolved_ttl_ms,
            true,
            true,
            true,
        ) {
            Ok(ok) => Ph1KmsResponse::KmsAccessEvaluateOk(ok),
            Err(_) => self.refuse(
                KmsCapabilityId::KmsAccessEvaluate,
                reason_codes::PH1_KMS_INTERNAL_PIPELINE_ERROR,
                "failed to construct kms access evaluate output",
            ),
        }
    }

    fn run_material_issue(&self, req: &KmsMaterialIssueRequest) -> Ph1KmsResponse {
        if !req.secret_ref.starts_with("kms_ref:") {
            return self.refuse(
                KmsCapabilityId::KmsMaterialIssue,
                reason_codes::PH1_KMS_VALIDATION_FAILED,
                "secret_ref is not recognized",
            );
        }

        let mut diagnostics = Vec::new();
        if req.require_no_secret_value_emission
            && (req.secret_ref.contains("secret") || req.secret_ref.contains("value"))
        {
            diagnostics.push("potential_secret_value_leak_detected".to_string());
        }

        let (secret_handle, ephemeral_credential_ref, rotated_version, revoked, reason_code) =
            match req.operation {
                KmsOperation::GetHandle => (
                    Some(opaque_ref(
                        "kms_handle",
                        &[req.tenant_id.as_str(), req.secret_ref.as_str(), "get"],
                    )),
                    None,
                    None,
                    false,
                    reason_codes::PH1_KMS_OK_MATERIAL_ISSUE,
                ),
                KmsOperation::IssueEphemeral => (
                    None,
                    Some(opaque_ref(
                        "kms_ephem",
                        &[
                            req.tenant_id.as_str(),
                            req.secret_ref.as_str(),
                            &req.resolved_ttl_ms.unwrap_or(0).to_string(),
                        ],
                    )),
                    None,
                    false,
                    reason_codes::PH1_KMS_OK_MATERIAL_ISSUE,
                ),
                KmsOperation::Rotate => match req.previous_version {
                    Some(previous_version) => (
                        Some(opaque_ref(
                            "kms_handle",
                            &[
                                req.tenant_id.as_str(),
                                req.secret_ref.as_str(),
                                "rotate",
                                &(previous_version + 1).to_string(),
                            ],
                        )),
                        None,
                        Some(previous_version + 1),
                        false,
                        reason_codes::PH1_KMS_OK_MATERIAL_ISSUE,
                    ),
                    None => {
                        diagnostics.push("rotation_previous_version_missing".to_string());
                        (
                            None,
                            None,
                            None,
                            false,
                            reason_codes::PH1_KMS_ROTATION_FAILED,
                        )
                    }
                },
                KmsOperation::Revoke => (
                    Some(opaque_ref(
                        "kms_revoked",
                        &[req.tenant_id.as_str(), req.secret_ref.as_str(), "revoke"],
                    )),
                    None,
                    None,
                    true,
                    reason_codes::PH1_KMS_OK_MATERIAL_ISSUE,
                ),
            };

        diagnostics.truncate(min(
            req.envelope.max_diagnostics as usize,
            self.config.max_diagnostics as usize,
        ));

        let validation_status = if diagnostics.is_empty() {
            KmsValidationStatus::Ok
        } else {
            KmsValidationStatus::Fail
        };

        match KmsMaterialIssueOk::v1(
            if validation_status == KmsValidationStatus::Ok {
                reason_code
            } else if req.operation == KmsOperation::Rotate {
                reason_codes::PH1_KMS_ROTATION_FAILED
            } else {
                reason_codes::PH1_KMS_VALIDATION_FAILED
            },
            validation_status,
            diagnostics,
            req.operation,
            secret_handle,
            ephemeral_credential_ref,
            rotated_version,
            revoked,
            true,
            true,
        ) {
            Ok(ok) => Ph1KmsResponse::KmsMaterialIssueOk(ok),
            Err(_) => self.refuse(
                KmsCapabilityId::KmsMaterialIssue,
                reason_codes::PH1_KMS_INTERNAL_PIPELINE_ERROR,
                "failed to construct kms material issue output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: KmsCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1KmsResponse {
        let refuse = KmsRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("KmsRefuse::v1 must construct for static messages");
        Ph1KmsResponse::Refuse(refuse)
    }
}

fn capability_from_request(req: &Ph1KmsRequest) -> KmsCapabilityId {
    match req {
        Ph1KmsRequest::KmsAccessEvaluate(_) => KmsCapabilityId::KmsAccessEvaluate,
        Ph1KmsRequest::KmsMaterialIssue(_) => KmsCapabilityId::KmsMaterialIssue,
    }
}

fn requester_authorized(req: &KmsAccessEvaluateRequest) -> bool {
    if !req.requester_allowlist.is_empty()
        && !req
            .requester_allowlist
            .iter()
            .any(|entry| entry == &req.requester_engine_id)
    {
        return false;
    }

    match req.operation {
        KmsOperation::Rotate | KmsOperation::Revoke => {
            if req.require_admin_for_rotation {
                req.requester_user_id.is_some()
            } else {
                true
            }
        }
        KmsOperation::GetHandle | KmsOperation::IssueEphemeral => true,
    }
}

fn operation_token(operation: KmsOperation) -> &'static str {
    match operation {
        KmsOperation::GetHandle => "get",
        KmsOperation::IssueEphemeral => "ephem",
        KmsOperation::Rotate => "rotate",
        KmsOperation::Revoke => "revoke",
    }
}

fn stable_hash(parts: &[&str]) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
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

fn opaque_ref(prefix: &str, parts: &[&str]) -> String {
    format!("{}:{:016x}", prefix, stable_hash(parts))
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
    use selene_kernel_contracts::ph1kms::{
        KmsAccessEvaluateRequest, KmsMaterialIssueRequest, KmsRequestEnvelope,
    };

    fn envelope() -> KmsRequestEnvelope {
        KmsRequestEnvelope::v1(CorrelationId(7101), TurnId(3301), 8, 6).unwrap()
    }

    #[test]
    fn at_kms_01_no_secret_value_appears_in_outputs() {
        let runtime = Ph1KmsRuntime::new(Ph1KmsConfig::mvp_v1());

        let access_req = Ph1KmsRequest::KmsAccessEvaluate(
            KmsAccessEvaluateRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                "payments_api_key".to_string(),
                KmsOperation::GetHandle,
                "PH1.TTS".to_string(),
                Some("admin_user".to_string()),
                vec!["PH1.TTS".to_string()],
                None,
                10,
                true,
            )
            .unwrap(),
        );

        let access_resp = runtime.run(&access_req);
        let Ph1KmsResponse::KmsAccessEvaluateOk(access_ok) = access_resp else {
            panic!("expected access evaluate ok");
        };

        let issue_req = Ph1KmsRequest::KmsMaterialIssue(
            KmsMaterialIssueRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                KmsOperation::GetHandle,
                access_ok.secret_ref,
                "PH1.TTS".to_string(),
                Some("admin_user".to_string()),
                None,
                None,
                true,
            )
            .unwrap(),
        );

        let issue_resp = runtime.run(&issue_req);
        let Ph1KmsResponse::KmsMaterialIssueOk(issue_ok) = issue_resp else {
            panic!("expected material issue ok");
        };

        let handle = issue_ok.secret_handle.unwrap();
        assert!(!handle.contains("payments_api_key"));
        assert!(!handle.contains("secret"));
    }

    #[test]
    fn at_kms_02_rotation_produces_new_version() {
        let runtime = Ph1KmsRuntime::new(Ph1KmsConfig::mvp_v1());

        let access_req = Ph1KmsRequest::KmsAccessEvaluate(
            KmsAccessEvaluateRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                "db_password_ref".to_string(),
                KmsOperation::Rotate,
                "PH1.OS".to_string(),
                Some("admin_user".to_string()),
                vec!["PH1.OS".to_string()],
                None,
                20,
                true,
            )
            .unwrap(),
        );

        let access_resp = runtime.run(&access_req);
        let Ph1KmsResponse::KmsAccessEvaluateOk(access_ok) = access_resp else {
            panic!("expected access evaluate ok");
        };

        let issue_req = Ph1KmsRequest::KmsMaterialIssue(
            KmsMaterialIssueRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                KmsOperation::Rotate,
                access_ok.secret_ref,
                "PH1.OS".to_string(),
                Some("admin_user".to_string()),
                None,
                Some(3),
                true,
            )
            .unwrap(),
        );

        let issue_resp = runtime.run(&issue_req);
        let Ph1KmsResponse::KmsMaterialIssueOk(issue_ok) = issue_resp else {
            panic!("expected material issue ok");
        };

        assert_eq!(issue_ok.validation_status, KmsValidationStatus::Ok);
        assert_eq!(issue_ok.rotated_version, Some(4));
    }

    #[test]
    fn at_kms_03_not_authorized_fails_closed() {
        let runtime = Ph1KmsRuntime::new(Ph1KmsConfig::mvp_v1());

        let req = Ph1KmsRequest::KmsAccessEvaluate(
            KmsAccessEvaluateRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                "api_key_store".to_string(),
                KmsOperation::GetHandle,
                "PH1.C".to_string(),
                None,
                vec!["PH1.TTS".to_string()],
                None,
                10,
                false,
            )
            .unwrap(),
        );

        let resp = runtime.run(&req);
        let Ph1KmsResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_KMS_NOT_AUTHORIZED);
    }

    #[test]
    fn at_kms_04_ephemeral_ttl_bounds_enforced() {
        let runtime = Ph1KmsRuntime::new(Ph1KmsConfig::mvp_v1());

        let req = Ph1KmsRequest::KmsAccessEvaluate(
            KmsAccessEvaluateRequest::v1(
                envelope(),
                "tenant_demo".to_string(),
                "api_key_store".to_string(),
                KmsOperation::IssueEphemeral,
                "PH1.C".to_string(),
                Some("admin_user".to_string()),
                vec!["PH1.C".to_string()],
                Some(10),
                10,
                false,
            )
            .unwrap(),
        );

        let resp = runtime.run(&req);
        let Ph1KmsResponse::Refuse(refuse) = resp else {
            panic!("expected refuse");
        };
        assert_eq!(refuse.reason_code, reason_codes::PH1_KMS_TTL_OUT_OF_BOUNDS);
    }
}
