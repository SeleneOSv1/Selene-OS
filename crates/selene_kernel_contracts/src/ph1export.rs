#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1EXPORT_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportCapabilityId {
    ExportAccessEvaluate,
    ExportArtifactBuild,
}

impl ExportCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            ExportCapabilityId::ExportAccessEvaluate => "EXPORT_ACCESS_EVALUATE",
            ExportCapabilityId::ExportArtifactBuild => "EXPORT_ARTIFACT_BUILD",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportScopeKind {
    WorkOrderId,
    TimeRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportIncludeKind {
    AuditEvents,
    WorkOrderLedger,
    ConversationTurns,
}

impl ExportIncludeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ExportIncludeKind::AuditEvents => "audit_events",
            ExportIncludeKind::WorkOrderLedger => "work_order_ledger",
            ExportIncludeKind::ConversationTurns => "conversation_turns",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportResultStatus {
    Ok,
    Refused,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportScope {
    pub schema_version: SchemaVersion,
    pub kind: ExportScopeKind,
    pub work_order_id: Option<String>,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
}

impl ExportScope {
    pub fn work_order_id_v1(work_order_id: String) -> Result<Self, ContractViolation> {
        let scope = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            kind: ExportScopeKind::WorkOrderId,
            work_order_id: Some(work_order_id),
            start_ms: None,
            end_ms: None,
        };
        scope.validate()?;
        Ok(scope)
    }

    pub fn time_range_v1(start_ms: u64, end_ms: u64) -> Result<Self, ContractViolation> {
        let scope = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            kind: ExportScopeKind::TimeRange,
            work_order_id: None,
            start_ms: Some(start_ms),
            end_ms: Some(end_ms),
        };
        scope.validate()?;
        Ok(scope)
    }
}

impl Validate for ExportScope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_scope.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }

        match self.kind {
            ExportScopeKind::WorkOrderId => {
                if self.start_ms.is_some() || self.end_ms.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "export_scope.start_ms",
                        reason: "time bounds must be absent when kind=WORK_ORDER_ID",
                    });
                }
                match &self.work_order_id {
                    Some(work_order_id) => {
                        validate_token("export_scope.work_order_id", work_order_id, 128)?;
                    }
                    None => {
                        return Err(ContractViolation::InvalidValue {
                            field: "export_scope.work_order_id",
                            reason: "must be present when kind=WORK_ORDER_ID",
                        });
                    }
                }
            }
            ExportScopeKind::TimeRange => {
                if self.work_order_id.is_some() {
                    return Err(ContractViolation::InvalidValue {
                        field: "export_scope.work_order_id",
                        reason: "must be absent when kind=TIME_RANGE",
                    });
                }
                let start_ms = self.start_ms.ok_or(ContractViolation::InvalidValue {
                    field: "export_scope.start_ms",
                    reason: "must be present when kind=TIME_RANGE",
                })?;
                let end_ms = self.end_ms.ok_or(ContractViolation::InvalidValue {
                    field: "export_scope.end_ms",
                    reason: "must be present when kind=TIME_RANGE",
                })?;
                if start_ms == 0 || end_ms == 0 {
                    return Err(ContractViolation::InvalidValue {
                        field: "export_scope.start_ms",
                        reason: "start_ms and end_ms must be > 0",
                    });
                }
                if start_ms >= end_ms {
                    return Err(ContractViolation::InvalidValue {
                        field: "export_scope.end_ms",
                        reason: "must be greater than start_ms",
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_include_items: u8,
    pub max_diagnostics: u8,
    pub max_time_range_ms: u64,
}

impl ExportRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_include_items: u8,
        max_diagnostics: u8,
        max_time_range_ms: u64,
    ) -> Result<Self, ContractViolation> {
        let envelope = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_include_items,
            max_diagnostics,
            max_time_range_ms,
        };
        envelope.validate()?;
        Ok(envelope)
    }
}

impl Validate for ExportRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_request_envelope.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_include_items == 0 || self.max_include_items > 3 {
            return Err(ContractViolation::InvalidValue {
                field: "export_request_envelope.max_include_items",
                reason: "must be within 1..=3",
            });
        }
        if self.max_diagnostics == 0 || self.max_diagnostics > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "export_request_envelope.max_diagnostics",
                reason: "must be within 1..=16",
            });
        }
        if self.max_time_range_ms == 0 || self.max_time_range_ms > 31_536_000_000 {
            return Err(ContractViolation::InvalidValue {
                field: "export_request_envelope.max_time_range_ms",
                reason: "must be within 1..=31_536_000_000",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportAccessEvaluateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ExportRequestEnvelope,
    pub tenant_id: String,
    pub export_scope: ExportScope,
    pub requester_user_id: String,
    pub include: Vec<ExportIncludeKind>,
    pub redaction_policy_ref: String,
    pub now_ms: u64,
    pub require_audit_event: bool,
    pub disallow_raw_audio: bool,
}

impl ExportAccessEvaluateRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: ExportRequestEnvelope,
        tenant_id: String,
        export_scope: ExportScope,
        requester_user_id: String,
        include: Vec<ExportIncludeKind>,
        redaction_policy_ref: String,
        now_ms: u64,
        require_audit_event: bool,
        disallow_raw_audio: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            envelope,
            tenant_id,
            export_scope,
            requester_user_id,
            include,
            redaction_policy_ref,
            now_ms,
            require_audit_event,
            disallow_raw_audio,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ExportAccessEvaluateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_request.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "export_access_evaluate_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        self.export_scope.validate()?;
        validate_token(
            "export_access_evaluate_request.requester_user_id",
            &self.requester_user_id,
            96,
        )?;
        validate_include(
            "export_access_evaluate_request.include",
            &self.include,
            self.envelope.max_include_items as usize,
        )?;
        validate_token(
            "export_access_evaluate_request.redaction_policy_ref",
            &self.redaction_policy_ref,
            128,
        )?;
        if self.now_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_request.now_ms",
                reason: "must be > 0",
            });
        }
        if !self.require_audit_event {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_request.require_audit_event",
                reason: "must be true",
            });
        }
        if !self.disallow_raw_audio {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_request.disallow_raw_audio",
                reason: "must be true",
            });
        }

        if self.export_scope.kind == ExportScopeKind::TimeRange {
            let start_ms = self
                .export_scope
                .start_ms
                .expect("validated export_scope must include start_ms");
            let end_ms = self
                .export_scope
                .end_ms
                .expect("validated export_scope must include end_ms");
            if end_ms - start_ms > self.envelope.max_time_range_ms {
                return Err(ContractViolation::InvalidValue {
                    field: "export_access_evaluate_request.export_scope",
                    reason: "time range exceeds envelope max_time_range_ms",
                });
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportAccessEvaluateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ExportCapabilityId,
    pub reason_code: ReasonCodeId,
    pub tenant_id: String,
    pub export_scope_ref: String,
    pub include: Vec<ExportIncludeKind>,
    pub redaction_policy_ref: String,
    pub requester_authorized: bool,
    pub deterministic_redaction_required: bool,
    pub raw_audio_excluded: bool,
    pub audit_event_required: bool,
}

impl ExportAccessEvaluateOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        tenant_id: String,
        export_scope_ref: String,
        include: Vec<ExportIncludeKind>,
        redaction_policy_ref: String,
        requester_authorized: bool,
        deterministic_redaction_required: bool,
        raw_audio_excluded: bool,
        audit_event_required: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            capability_id: ExportCapabilityId::ExportAccessEvaluate,
            reason_code,
            tenant_id,
            export_scope_ref,
            include,
            redaction_policy_ref,
            requester_authorized,
            deterministic_redaction_required,
            raw_audio_excluded,
            audit_event_required,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for ExportAccessEvaluateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ExportCapabilityId::ExportAccessEvaluate {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.capability_id",
                reason: "must be EXPORT_ACCESS_EVALUATE",
            });
        }
        validate_token("export_access_evaluate_ok.tenant_id", &self.tenant_id, 64)?;
        validate_token(
            "export_access_evaluate_ok.export_scope_ref",
            &self.export_scope_ref,
            128,
        )?;
        validate_include("export_access_evaluate_ok.include", &self.include, 3)?;
        validate_token(
            "export_access_evaluate_ok.redaction_policy_ref",
            &self.redaction_policy_ref,
            128,
        )?;
        if !self.requester_authorized {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.requester_authorized",
                reason: "must be true for success output",
            });
        }
        if !self.deterministic_redaction_required {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.deterministic_redaction_required",
                reason: "must be true",
            });
        }
        if !self.raw_audio_excluded {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.raw_audio_excluded",
                reason: "must be true",
            });
        }
        if !self.audit_event_required {
            return Err(ContractViolation::InvalidValue {
                field: "export_access_evaluate_ok.audit_event_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportArtifactBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: ExportRequestEnvelope,
    pub tenant_id: String,
    pub export_scope_ref: String,
    pub requester_user_id: String,
    pub include: Vec<ExportIncludeKind>,
    pub redaction_policy_ref: String,
    pub now_ms: u64,
    pub deterministic_redaction_required: bool,
    pub raw_audio_excluded: bool,
    pub audit_event_required: bool,
}

impl ExportArtifactBuildRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        envelope: ExportRequestEnvelope,
        tenant_id: String,
        export_scope_ref: String,
        requester_user_id: String,
        include: Vec<ExportIncludeKind>,
        redaction_policy_ref: String,
        now_ms: u64,
        deterministic_redaction_required: bool,
        raw_audio_excluded: bool,
        audit_event_required: bool,
    ) -> Result<Self, ContractViolation> {
        let req = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            envelope,
            tenant_id,
            export_scope_ref,
            requester_user_id,
            include,
            redaction_policy_ref,
            now_ms,
            deterministic_redaction_required,
            raw_audio_excluded,
            audit_event_required,
        };
        req.validate()?;
        Ok(req)
    }
}

impl Validate for ExportArtifactBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_request.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        validate_token(
            "export_artifact_build_request.tenant_id",
            &self.tenant_id,
            64,
        )?;
        validate_token(
            "export_artifact_build_request.export_scope_ref",
            &self.export_scope_ref,
            128,
        )?;
        validate_token(
            "export_artifact_build_request.requester_user_id",
            &self.requester_user_id,
            96,
        )?;
        validate_include(
            "export_artifact_build_request.include",
            &self.include,
            self.envelope.max_include_items as usize,
        )?;
        validate_token(
            "export_artifact_build_request.redaction_policy_ref",
            &self.redaction_policy_ref,
            128,
        )?;
        if self.now_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_request.now_ms",
                reason: "must be > 0",
            });
        }
        if !self.deterministic_redaction_required {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_request.deterministic_redaction_required",
                reason: "must be true",
            });
        }
        if !self.raw_audio_excluded {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_request.raw_audio_excluded",
                reason: "must be true",
            });
        }
        if !self.audit_event_required {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_request.audit_event_required",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportArtifactBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: ExportCapabilityId,
    pub reason_code: ReasonCodeId,
    pub status: ExportResultStatus,
    pub export_artifact_id: String,
    pub export_hash: String,
    pub export_payload_ref: String,
    pub redaction_applied: bool,
    pub raw_audio_excluded: bool,
    pub audit_event_emitted: bool,
    pub deterministic_hash: bool,
}

impl ExportArtifactBuildOk {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        reason_code: ReasonCodeId,
        status: ExportResultStatus,
        export_artifact_id: String,
        export_hash: String,
        export_payload_ref: String,
        redaction_applied: bool,
        raw_audio_excluded: bool,
        audit_event_emitted: bool,
        deterministic_hash: bool,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            capability_id: ExportCapabilityId::ExportArtifactBuild,
            reason_code,
            status,
            export_artifact_id,
            export_hash,
            export_payload_ref,
            redaction_applied,
            raw_audio_excluded,
            audit_event_emitted,
            deterministic_hash,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for ExportArtifactBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        if self.capability_id != ExportCapabilityId::ExportArtifactBuild {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.capability_id",
                reason: "must be EXPORT_ARTIFACT_BUILD",
            });
        }
        if self.status != ExportResultStatus::Ok {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.status",
                reason: "must be OK for success output",
            });
        }
        validate_token(
            "export_artifact_build_ok.export_artifact_id",
            &self.export_artifact_id,
            128,
        )?;
        validate_hex64("export_artifact_build_ok.export_hash", &self.export_hash)?;
        validate_token(
            "export_artifact_build_ok.export_payload_ref",
            &self.export_payload_ref,
            160,
        )?;
        if !self.raw_audio_excluded {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.raw_audio_excluded",
                reason: "must be true",
            });
        }
        if !self.audit_event_emitted {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.audit_event_emitted",
                reason: "must be true",
            });
        }
        if !self.deterministic_hash {
            return Err(ContractViolation::InvalidValue {
                field: "export_artifact_build_ok.deterministic_hash",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: ExportCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl ExportRefuse {
    pub fn v1(
        capability_id: ExportCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let refuse = Self {
            schema_version: PH1EXPORT_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        refuse.validate()?;
        Ok(refuse)
    }
}

impl Validate for ExportRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1EXPORT_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "export_refuse.schema_version",
                reason: "must match PH1EXPORT_CONTRACT_VERSION",
            });
        }
        validate_text("export_refuse.message", &self.message, 192)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ExportRequest {
    ExportAccessEvaluate(ExportAccessEvaluateRequest),
    ExportArtifactBuild(ExportArtifactBuildRequest),
}

impl Validate for Ph1ExportRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ExportRequest::ExportAccessEvaluate(req) => req.validate(),
            Ph1ExportRequest::ExportArtifactBuild(req) => req.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1ExportResponse {
    ExportAccessEvaluateOk(ExportAccessEvaluateOk),
    ExportArtifactBuildOk(ExportArtifactBuildOk),
    Refuse(ExportRefuse),
}

impl Validate for Ph1ExportResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1ExportResponse::ExportAccessEvaluateOk(out) => out.validate(),
            Ph1ExportResponse::ExportArtifactBuildOk(out) => out.validate(),
            Ph1ExportResponse::Refuse(refuse) => refuse.validate(),
        }
    }
}

fn validate_include(
    field: &'static str,
    include: &[ExportIncludeKind],
    max_items: usize,
) -> Result<(), ContractViolation> {
    if include.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if include.len() > max_items {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max include items",
        });
    }
    let mut seen = BTreeSet::new();
    for item in include {
        if !seen.insert(item.as_str()) {
            return Err(ContractViolation::InvalidValue {
                field,
                reason: "include entries must be unique",
            });
        }
    }
    Ok(())
}

fn validate_hex64(field: &'static str, value: &str) -> Result<(), ContractViolation> {
    if value.len() != 64 || !value.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be a 64-char hex value",
        });
    }
    Ok(())
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

fn validate_text(
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
    if !value.is_ascii() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be ASCII",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope() -> ExportRequestEnvelope {
        ExportRequestEnvelope::v1(CorrelationId(3301), TurnId(7401), 3, 8, 86_400_000).unwrap()
    }

    #[test]
    fn at_export_01_time_range_must_fit_envelope() {
        let req = ExportAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            ExportScope::time_range_v1(1, 86_500_000).unwrap(),
            "admin_user".to_string(),
            vec![ExportIncludeKind::AuditEvents],
            "policy_default".to_string(),
            10,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_export_02_include_entries_must_be_unique() {
        let req = ExportAccessEvaluateRequest::v1(
            envelope(),
            "tenant_demo".to_string(),
            ExportScope::work_order_id_v1("wo_123".to_string()).unwrap(),
            "admin_user".to_string(),
            vec![
                ExportIncludeKind::AuditEvents,
                ExportIncludeKind::AuditEvents,
            ],
            "policy_default".to_string(),
            10,
            true,
            true,
        );
        assert!(req.is_err());
    }

    #[test]
    fn at_export_03_hash_must_be_sha256_hex() {
        let out = ExportArtifactBuildOk::v1(
            ReasonCodeId(1),
            ExportResultStatus::Ok,
            "export_artifact_1".to_string(),
            "zz-not-hex".to_string(),
            "export_payload:1".to_string(),
            true,
            true,
            true,
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn at_export_04_success_requires_audit_and_raw_audio_excluded() {
        let out = ExportArtifactBuildOk::v1(
            ReasonCodeId(1),
            ExportResultStatus::Ok,
            "export_artifact_1".to_string(),
            "8f14e45fceea167a5a36dedd4bea2543fcbf13f8b8f6cbf7a22f6f7a4f6f6f61".to_string(),
            "export_payload:1".to_string(),
            true,
            true,
            false,
            true,
        );
        assert!(out.is_err());
    }
}
