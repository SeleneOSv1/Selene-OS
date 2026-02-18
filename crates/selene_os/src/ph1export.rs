#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1export::{
    ExportAccessEvaluateOk, ExportAccessEvaluateRequest, ExportArtifactBuildOk,
    ExportArtifactBuildRequest, ExportCapabilityId, ExportIncludeKind, ExportRefuse,
    ExportRequestEnvelope, ExportResultStatus, ExportScope, Ph1ExportRequest, Ph1ExportResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.EXPORT OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_EXPORT_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x4558_0101);
    pub const PH1_EXPORT_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x4558_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1ExportWiringConfig {
    pub export_enabled: bool,
    pub max_include_items: u8,
    pub max_diagnostics: u8,
    pub max_time_range_ms: u64,
}

impl Ph1ExportWiringConfig {
    pub fn mvp_v1(export_enabled: bool) -> Self {
        Self {
            export_enabled,
            max_include_items: 3,
            max_diagnostics: 8,
            max_time_range_ms: 31 * 24 * 60 * 60 * 1000,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub tenant_id: String,
    pub export_scope: ExportScope,
    pub requester_user_id: String,
    pub include: Vec<ExportIncludeKind>,
    pub redaction_policy_ref: String,
    pub now_ms: u64,
    pub require_audit_event: bool,
    pub disallow_raw_audio: bool,
}

impl ExportTurnInput {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        tenant_id: String,
        export_scope: ExportScope,
        requester_user_id: String,
        include: Vec<ExportIncludeKind>,
        redaction_policy_ref: String,
        now_ms: u64,
        require_audit_event: bool,
        disallow_raw_audio: bool,
    ) -> Result<Self, ContractViolation> {
        let input = Self {
            correlation_id,
            turn_id,
            tenant_id,
            export_scope,
            requester_user_id,
            include,
            redaction_policy_ref,
            now_ms,
            require_audit_event,
            disallow_raw_audio,
        };
        input.validate()?;
        Ok(input)
    }
}

impl Validate for ExportTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.export_scope.validate()?;
        validate_token("export_turn_input.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "export_turn_input.requester_user_id",
            &self.requester_user_id,
            96,
        )?;
        if self.include.is_empty() || self.include.len() > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "export_turn_input.include",
                reason: "must contain 1..=3 include entries",
            });
        }
        validate_token(
            "export_turn_input.redaction_policy_ref",
            &self.redaction_policy_ref,
            128,
        )?;
        if self.now_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "export_turn_input.now_ms",
                reason: "must be > 0",
            });
        }
        if !self.require_audit_event {
            return Err(ContractViolation::InvalidValue {
                field: "export_turn_input.require_audit_event",
                reason: "must be true",
            });
        }
        if !self.disallow_raw_audio {
            return Err(ContractViolation::InvalidValue {
                field: "export_turn_input.disallow_raw_audio",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub access_evaluate: ExportAccessEvaluateOk,
    pub artifact_build: ExportArtifactBuildOk,
}

impl ExportForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        access_evaluate: ExportAccessEvaluateOk,
        artifact_build: ExportArtifactBuildOk,
    ) -> Result<Self, ContractViolation> {
        let bundle = Self {
            correlation_id,
            turn_id,
            access_evaluate,
            artifact_build,
        };
        bundle.validate()?;
        Ok(bundle)
    }
}

impl Validate for ExportForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.access_evaluate.validate()?;
        self.artifact_build.validate()?;
        if self.artifact_build.status != ExportResultStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "export_forward_bundle.artifact_build.status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportWiringOutcome {
    NotInvokedDisabled,
    Refused(ExportRefuse),
    Forwarded(ExportForwardBundle),
}

pub trait Ph1ExportEngine {
    fn run(&self, req: &Ph1ExportRequest) -> Ph1ExportResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1ExportWiring<E>
where
    E: Ph1ExportEngine,
{
    config: Ph1ExportWiringConfig,
    engine: E,
}

impl<E> Ph1ExportWiring<E>
where
    E: Ph1ExportEngine,
{
    pub fn new(config: Ph1ExportWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_include_items == 0 || config.max_include_items > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1export_wiring_config.max_include_items",
                reason: "must be within 1..=3",
            });
        }
        if config.max_diagnostics == 0 || config.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1export_wiring_config.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if config.max_time_range_ms == 0 || config.max_time_range_ms > 31_536_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1export_wiring_config.max_time_range_ms",
                reason: "must be within 1..=31_536_000_000",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(
        &self,
        input: &ExportTurnInput,
    ) -> Result<ExportWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.export_enabled {
            return Ok(ExportWiringOutcome::NotInvokedDisabled);
        }

        let envelope = ExportRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_include_items, 3),
            min(self.config.max_diagnostics, 16),
            self.config.max_time_range_ms,
        )?;

        let access_req = Ph1ExportRequest::ExportAccessEvaluate(ExportAccessEvaluateRequest::v1(
            envelope.clone(),
            input.tenant_id.clone(),
            input.export_scope.clone(),
            input.requester_user_id.clone(),
            input.include.clone(),
            input.redaction_policy_ref.clone(),
            input.now_ms,
            input.require_audit_event,
            input.disallow_raw_audio,
        )?);
        let access_resp = self.engine.run(&access_req);
        access_resp.validate()?;

        let access_ok = match access_resp {
            Ph1ExportResponse::Refuse(refuse) => return Ok(ExportWiringOutcome::Refused(refuse)),
            Ph1ExportResponse::ExportAccessEvaluateOk(ok) => ok,
            Ph1ExportResponse::ExportArtifactBuildOk(_) => {
                return Ok(ExportWiringOutcome::Refused(ExportRefuse::v1(
                    ExportCapabilityId::ExportAccessEvaluate,
                    reason_codes::PH1_EXPORT_INTERNAL_PIPELINE_ERROR,
                    "unexpected artifact-build response for access-evaluate request".to_string(),
                )?));
            }
        };

        let artifact_req = Ph1ExportRequest::ExportArtifactBuild(ExportArtifactBuildRequest::v1(
            envelope,
            access_ok.tenant_id.clone(),
            access_ok.export_scope_ref.clone(),
            input.requester_user_id.clone(),
            access_ok.include.clone(),
            access_ok.redaction_policy_ref.clone(),
            input.now_ms,
            access_ok.deterministic_redaction_required,
            access_ok.raw_audio_excluded,
            access_ok.audit_event_required,
        )?);
        let artifact_resp = self.engine.run(&artifact_req);
        artifact_resp.validate()?;

        let artifact_ok = match artifact_resp {
            Ph1ExportResponse::Refuse(refuse) => return Ok(ExportWiringOutcome::Refused(refuse)),
            Ph1ExportResponse::ExportArtifactBuildOk(ok) => ok,
            Ph1ExportResponse::ExportAccessEvaluateOk(_) => {
                return Ok(ExportWiringOutcome::Refused(ExportRefuse::v1(
                    ExportCapabilityId::ExportArtifactBuild,
                    reason_codes::PH1_EXPORT_INTERNAL_PIPELINE_ERROR,
                    "unexpected access-evaluate response for artifact-build request".to_string(),
                )?));
            }
        };

        if artifact_ok.status != ExportResultStatus::Ok {
            return Ok(ExportWiringOutcome::Refused(ExportRefuse::v1(
                ExportCapabilityId::ExportArtifactBuild,
                reason_codes::PH1_EXPORT_VALIDATION_FAILED,
                "export artifact status was not OK".to_string(),
            )?));
        }

        let bundle =
            ExportForwardBundle::v1(input.correlation_id, input.turn_id, access_ok, artifact_ok)?;
        Ok(ExportWiringOutcome::Forwarded(bundle))
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
    use selene_kernel_contracts::ph1export::{
        ExportAccessEvaluateOk, ExportArtifactBuildOk, ExportResultStatus,
    };
    use selene_kernel_contracts::ReasonCodeId;

    #[derive(Clone)]
    struct DeterministicExportEngine;

    impl Ph1ExportEngine for DeterministicExportEngine {
        fn run(&self, req: &Ph1ExportRequest) -> Ph1ExportResponse {
            match req {
                Ph1ExportRequest::ExportAccessEvaluate(r) => {
                    Ph1ExportResponse::ExportAccessEvaluateOk(
                        ExportAccessEvaluateOk::v1(
                            ReasonCodeId(41),
                            r.tenant_id.clone(),
                            "export_scope:deterministic".to_string(),
                            r.include.clone(),
                            r.redaction_policy_ref.clone(),
                            true,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1ExportRequest::ExportArtifactBuild(r) => {
                    let hash_input = format!(
                        "{}|{}|{}|{}|{}",
                        r.tenant_id,
                        r.export_scope_ref,
                        r.requester_user_id,
                        r.redaction_policy_ref,
                        r.now_ms
                    );
                    let hash = {
                        let mut acc: u64 = 0xcbf2_9ce4_8422_2325;
                        for byte in hash_input.as_bytes() {
                            acc ^= u64::from(*byte);
                            acc = acc.wrapping_mul(0x0000_0100_0000_01b3);
                        }
                        format!("{acc:016x}{acc:016x}{acc:016x}{acc:016x}")
                    };
                    let redaction_applied = r
                        .include
                        .iter()
                        .any(|item| *item == ExportIncludeKind::ConversationTurns);
                    Ph1ExportResponse::ExportArtifactBuildOk(
                        ExportArtifactBuildOk::v1(
                            ReasonCodeId(42),
                            ExportResultStatus::Ok,
                            "export_artifact:deterministic".to_string(),
                            hash,
                            "export_payload:deterministic".to_string(),
                            redaction_applied,
                            true,
                            true,
                            true,
                        )
                        .unwrap(),
                    )
                }
            }
        }
    }

    fn sample_input() -> ExportTurnInput {
        ExportTurnInput::v1(
            CorrelationId(9901),
            TurnId(1901),
            "tenant_demo".to_string(),
            ExportScope::work_order_id_v1("wo_123".to_string()).unwrap(),
            "finance_admin".to_string(),
            vec![
                ExportIncludeKind::AuditEvents,
                ExportIncludeKind::WorkOrderLedger,
                ExportIncludeKind::ConversationTurns,
            ],
            "policy_default".to_string(),
            500,
            true,
            true,
        )
        .unwrap()
    }

    #[test]
    fn at_export_01_os_tamper_evident_hash_is_stable() {
        let wiring = Ph1ExportWiring::new(
            Ph1ExportWiringConfig::mvp_v1(true),
            DeterministicExportEngine,
        )
        .unwrap();
        let input = sample_input();

        let out1 = wiring.run_turn(&input).unwrap();
        let out2 = wiring.run_turn(&input).unwrap();

        let hash1 = match out1 {
            ExportWiringOutcome::Forwarded(bundle) => bundle.artifact_build.export_hash,
            _ => panic!("expected forwarded outcome"),
        };
        let hash2 = match out2 {
            ExportWiringOutcome::Forwarded(bundle) => bundle.artifact_build.export_hash,
            _ => panic!("expected forwarded outcome"),
        };
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn at_export_02_os_redaction_is_applied_deterministically() {
        let wiring = Ph1ExportWiring::new(
            Ph1ExportWiringConfig::mvp_v1(true),
            DeterministicExportEngine,
        )
        .unwrap();
        let out = wiring.run_turn(&sample_input()).unwrap();
        match out {
            ExportWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.artifact_build.redaction_applied);
            }
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_export_03_os_export_is_audited() {
        let wiring = Ph1ExportWiring::new(
            Ph1ExportWiringConfig::mvp_v1(true),
            DeterministicExportEngine,
        )
        .unwrap();
        let out = wiring.run_turn(&sample_input()).unwrap();
        match out {
            ExportWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.artifact_build.audit_event_emitted);
                assert!(bundle.artifact_build.raw_audio_excluded);
            }
            _ => panic!("expected forwarded outcome"),
        }
    }

    #[test]
    fn at_export_04_os_disabled_returns_not_invoked() {
        let wiring = Ph1ExportWiring::new(
            Ph1ExportWiringConfig::mvp_v1(false),
            DeterministicExportEngine,
        )
        .unwrap();
        let out = wiring.run_turn(&sample_input()).unwrap();
        assert_eq!(out, ExportWiringOutcome::NotInvokedDisabled);
    }
}
