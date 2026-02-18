#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::ph1kms::{
    KmsAccessEvaluateOk, KmsAccessEvaluateRequest, KmsCapabilityId, KmsMaterialIssueOk,
    KmsMaterialIssueRequest, KmsOperation, KmsRefuse, KmsRequestEnvelope, KmsValidationStatus,
    Ph1KmsRequest, Ph1KmsResponse,
};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.KMS OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_KMS_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4B4D_0101);
    pub const PH1_KMS_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4B4D_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1KmsWiringConfig {
    pub kms_enabled: bool,
    pub max_allowlist_entries: u8,
    pub max_diagnostics: u8,
}

impl Ph1KmsWiringConfig {
    pub fn mvp_v1(kms_enabled: bool) -> Self {
        Self {
            kms_enabled,
            max_allowlist_entries: 16,
            max_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub secret_name: String,
    pub operation: KmsOperation,
    pub requester_engine_id: String,
    pub requester_user_id: Option<String>,
    pub requester_allowlist: Vec<String>,
    pub requested_ttl_ms: Option<u32>,
    pub now_ms: u64,
    pub previous_version: Option<u32>,
    pub require_admin_for_rotation: bool,
    pub require_no_secret_value_emission: bool,
}

impl KmsTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        secret_name: String,
        operation: KmsOperation,
        requester_engine_id: String,
        requester_user_id: Option<String>,
        requester_allowlist: Vec<String>,
        requested_ttl_ms: Option<u32>,
        now_ms: u64,
        previous_version: Option<u32>,
        require_admin_for_rotation: bool,
        require_no_secret_value_emission: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            secret_name,
            operation,
            requester_engine_id,
            requester_user_id,
            requester_allowlist,
            requested_ttl_ms,
            now_ms,
            previous_version,
            require_admin_for_rotation,
            require_no_secret_value_emission,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for KmsTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        validate_token("kms_turn_input.tenant_id", &self.tenant_id, 64)?;
        validate_token("kms_turn_input.secret_name", &self.secret_name, 128)?;
        validate_token(
            "kms_turn_input.requester_engine_id",
            &self.requester_engine_id,
            96,
        )?;

        if let Some(requester_user_id) = &self.requester_user_id {
            validate_token("kms_turn_input.requester_user_id", requester_user_id, 96)?;
        }

        if self.requester_allowlist.len() > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_turn_input.requester_allowlist",
                reason: "must be <= 32",
            });
        }
        for entry in &self.requester_allowlist {
            validate_token("kms_turn_input.requester_allowlist", entry, 96)?;
        }

        if self.now_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "kms_turn_input.now_ms",
                reason: "must be > 0",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KmsForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub access_evaluate: KmsAccessEvaluateOk,
    pub material_issue: KmsMaterialIssueOk,
}

impl KmsForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        access_evaluate: KmsAccessEvaluateOk,
        material_issue: KmsMaterialIssueOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            access_evaluate,
            material_issue,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for KmsForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.access_evaluate.validate()?;
        self.material_issue.validate()?;

        if self.material_issue.validation_status != KmsValidationStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "kms_forward_bundle.material_issue.validation_status",
                reason: "must be OK",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KmsWiringOutcome {
    NotInvokedDisabled,
    Refused(KmsRefuse),
    Forwarded(KmsForwardBundle),
}

pub trait Ph1KmsEngine {
    fn run(&self, req: &Ph1KmsRequest) -> Ph1KmsResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1KmsWiring<E>
where
    E: Ph1KmsEngine,
{
    config: Ph1KmsWiringConfig,
    engine: E,
}

impl<E> Ph1KmsWiring<E>
where
    E: Ph1KmsEngine,
{
    pub fn new(config: Ph1KmsWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_allowlist_entries == 0 || config.max_allowlist_entries > 32 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1kms_wiring_config.max_allowlist_entries",
                reason: "must be within 1..=32",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1kms_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }

        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &KmsTurnInput) -> Result<KmsWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.kms_enabled {
            return Ok(KmsWiringOutcome::NotInvokedDisabled);
        }

        let envelope = KmsRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_allowlist_entries, 32),
            min(self.config.max_diagnostics, 16),
        )?;

        let access_req = Ph1KmsRequest::KmsAccessEvaluate(KmsAccessEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.secret_name.clone(),
            input.operation,
            input.requester_engine_id.clone(),
            input.requester_user_id.clone(),
            input.requester_allowlist.clone(),
            input.requested_ttl_ms,
            input.now_ms,
            input.require_admin_for_rotation,
        )?);
        let access_resp = self.engine.run(&access_req);
        access_resp.validate()?;

        let access_ok = match access_resp {
            Ph1KmsResponse::Refuse(refuse) => return Ok(KmsWiringOutcome::Refused(refuse)),
            Ph1KmsResponse::KmsAccessEvaluateOk(ok) => ok,
            Ph1KmsResponse::KmsMaterialIssueOk(_) => {
                return Ok(KmsWiringOutcome::Refused(KmsRefuse::v1(
                    KmsCapabilityId::KmsAccessEvaluate,
                    reason_codes::PH1_KMS_INTERNAL_PIPELINE_ERROR,
                    "unexpected material-issue response for access-evaluate request".to_string(),
                )?));
            }
        };

        let issue_req = Ph1KmsRequest::KmsMaterialIssue(KmsMaterialIssueRequest::v1(
            envelope,
            input.tenant_id.clone(),
            input.operation,
            access_ok.secret_ref.clone(),
            input.requester_engine_id.clone(),
            input.requester_user_id.clone(),
            access_ok.resolved_ttl_ms,
            input.previous_version,
            input.require_no_secret_value_emission,
        )?);
        let issue_resp = self.engine.run(&issue_req);
        issue_resp.validate()?;

        let issue_ok = match issue_resp {
            Ph1KmsResponse::Refuse(refuse) => return Ok(KmsWiringOutcome::Refused(refuse)),
            Ph1KmsResponse::KmsMaterialIssueOk(ok) => ok,
            Ph1KmsResponse::KmsAccessEvaluateOk(_) => {
                return Ok(KmsWiringOutcome::Refused(KmsRefuse::v1(
                    KmsCapabilityId::KmsMaterialIssue,
                    reason_codes::PH1_KMS_INTERNAL_PIPELINE_ERROR,
                    "unexpected access-evaluate response for material-issue request".to_string(),
                )?));
            }
        };

        if issue_ok.validation_status != KmsValidationStatus::Ok {
            return Ok(KmsWiringOutcome::Refused(KmsRefuse::v1(
                KmsCapabilityId::KmsMaterialIssue,
                reason_codes::PH1_KMS_VALIDATION_FAILED,
                "kms material issue validation failed".to_string(),
            )?));
        }

        let bundle =
            KmsForwardBundle::v1(input.correlation_id, input.turn_id, access_ok, issue_ok)?;
        Ok(KmsWiringOutcome::Forwarded(bundle))
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

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Clone)]
    struct DeterministicKmsEngine {
        force_issue_fail: bool,
    }

    impl Ph1KmsEngine for DeterministicKmsEngine {
        fn run(&self, req: &Ph1KmsRequest) -> Ph1KmsResponse {
            match req {
                Ph1KmsRequest::KmsAccessEvaluate(r) => Ph1KmsResponse::KmsAccessEvaluateOk(
                    KmsAccessEvaluateOk::v1(
                        ReasonCodeId(1),
                        r.operation,
                        "kms_ref:deadbeef".to_string(),
                        r.requested_ttl_ms,
                        true,
                        true,
                        true,
                    )
                    .unwrap(),
                ),
                Ph1KmsRequest::KmsMaterialIssue(r) => {
                    let (status, diagnostics) = if self.force_issue_fail {
                        (
                            KmsValidationStatus::Fail,
                            vec!["rotation_previous_version_missing".to_string()],
                        )
                    } else {
                        (KmsValidationStatus::Ok, vec![])
                    };

                    let out = match r.operation {
                        KmsOperation::GetHandle => KmsMaterialIssueOk::v1(
                            ReasonCodeId(1),
                            status,
                            diagnostics,
                            KmsOperation::GetHandle,
                            Some("kms_handle:abcd".to_string()),
                            None,
                            None,
                            false,
                            true,
                            true,
                        )
                        .unwrap(),
                        KmsOperation::IssueEphemeral => KmsMaterialIssueOk::v1(
                            ReasonCodeId(1),
                            status,
                            diagnostics,
                            KmsOperation::IssueEphemeral,
                            None,
                            Some("kms_ephem:abcd".to_string()),
                            None,
                            false,
                            true,
                            true,
                        )
                        .unwrap(),
                        KmsOperation::Rotate => KmsMaterialIssueOk::v1(
                            ReasonCodeId(1),
                            status,
                            diagnostics,
                            KmsOperation::Rotate,
                            Some("kms_handle:abcd".to_string()),
                            None,
                            Some(2),
                            false,
                            true,
                            true,
                        )
                        .unwrap(),
                        KmsOperation::Revoke => KmsMaterialIssueOk::v1(
                            ReasonCodeId(1),
                            status,
                            diagnostics,
                            KmsOperation::Revoke,
                            Some("kms_revoked:abcd".to_string()),
                            None,
                            None,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    };

                    Ph1KmsResponse::KmsMaterialIssueOk(out)
                }
            }
        }
    }

    fn sample_input(operation: KmsOperation) -> KmsTurnInput {
        KmsTurnInput::v1(
            CorrelationId(9401),
            TurnId(1601),
            "tenant_demo".to_string(),
            "api_key_store".to_string(),
            operation,
            "PH1.OS".to_string(),
            Some("admin_user".to_string()),
            vec!["PH1.OS".to_string()],
            if operation == KmsOperation::IssueEphemeral {
                Some(60_000)
            } else {
                None
            },
            5,
            if operation == KmsOperation::Rotate {
                Some(1)
            } else {
                None
            },
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_kms_01_os_invokes_and_returns_forward_bundle() {
        let wiring = Ph1KmsWiring::new(
            Ph1KmsWiringConfig::mvp_v1(true),
            DeterministicKmsEngine {
                force_issue_fail: false,
            },
        )
        .unwrap();

        let out = wiring
            .run_turn(&sample_input(KmsOperation::GetHandle))
            .unwrap();
        match out {
            KmsWiringOutcome::Forwarded(bundle) => {
                assert_eq!(
                    bundle.material_issue.validation_status,
                    KmsValidationStatus::Ok
                );
            }
            _ => panic!("expected forwarded bundle"),
        }
    }

    #[test]
    fn at_kms_02_disabled_returns_not_invoked() {
        let wiring = Ph1KmsWiring::new(
            Ph1KmsWiringConfig::mvp_v1(false),
            DeterministicKmsEngine {
                force_issue_fail: false,
            },
        )
        .unwrap();

        let out = wiring
            .run_turn(&sample_input(KmsOperation::GetHandle))
            .unwrap();
        assert_eq!(out, KmsWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_kms_03_issue_ephemeral_path_is_wired() {
        let wiring = Ph1KmsWiring::new(
            Ph1KmsWiringConfig::mvp_v1(true),
            DeterministicKmsEngine {
                force_issue_fail: false,
            },
        )
        .unwrap();

        let out = wiring
            .run_turn(&sample_input(KmsOperation::IssueEphemeral))
            .unwrap();
        match out {
            KmsWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.material_issue.ephemeral_credential_ref.is_some());
            }
            _ => panic!("expected forwarded bundle"),
        }
    }

    #[test]
    fn at_kms_04_validation_fail_is_refused() {
        let wiring = Ph1KmsWiring::new(
            Ph1KmsWiringConfig::mvp_v1(true),
            DeterministicKmsEngine {
                force_issue_fail: true,
            },
        )
        .unwrap();

        let out = wiring
            .run_turn(&sample_input(KmsOperation::Rotate))
            .unwrap();
        match out {
            KmsWiringOutcome::Refused(refuse) => {
                assert_eq!(refuse.reason_code, reason_codes::PH1_KMS_VALIDATION_FAILED);
                assert_eq!(refuse.capability_id, KmsCapabilityId::KmsMaterialIssue);
            }
            _ => panic!("expected refused outcome"),
        }
    }
}
