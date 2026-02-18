#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1SUMMARY_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SummaryCapabilityId {
    SummaryBuild,
    SummaryCitationValidate,
}

impl SummaryCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            SummaryCapabilityId::SummaryBuild => "SUMMARY_BUILD",
            SummaryCapabilityId::SummaryCitationValidate => "SUMMARY_CITATION_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_summary_bullets: u8,
}

impl SummaryRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_summary_bullets: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_summary_bullets,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for SummaryRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_request_envelope.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_summary_bullets == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_request_envelope.max_summary_bullets",
                reason: "must be > 0",
            });
        }
        if self.max_summary_bullets > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_request_envelope.max_summary_bullets",
                reason: "must be <= 16",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SummaryEvidenceId(String);

impl SummaryEvidenceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for SummaryEvidenceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryEvidenceItem {
    pub schema_version: SchemaVersion,
    pub evidence_id: SummaryEvidenceId,
    pub text: String,
}

impl SummaryEvidenceItem {
    pub fn v1(evidence_id: SummaryEvidenceId, text: String) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            evidence_id,
            text,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for SummaryEvidenceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_item.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        self.evidence_id.validate()?;
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_item.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_item.text",
                reason: "must be <= 512 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "summary_evidence_item.text",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryBullet {
    pub schema_version: SchemaVersion,
    pub text: String,
    pub cited_evidence_ids: Vec<SummaryEvidenceId>,
}

impl SummaryBullet {
    pub fn v1(
        text: String,
        cited_evidence_ids: Vec<SummaryEvidenceId>,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            text,
            cited_evidence_ids,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for SummaryBullet {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 240 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.text",
                reason: "must be <= 240 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.text",
                reason: "must not contain control characters",
            });
        }
        if self.cited_evidence_ids.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.cited_evidence_ids",
                reason: "must not be empty",
            });
        }
        if self.cited_evidence_ids.len() > 4 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_bullet.cited_evidence_ids",
                reason: "must be <= 4",
            });
        }
        for id in &self.cited_evidence_ids {
            id.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SummaryRequestEnvelope,
    pub evidence_items: Vec<SummaryEvidenceItem>,
}

impl SummaryBuildRequest {
    pub fn v1(
        envelope: SummaryRequestEnvelope,
        evidence_items: Vec<SummaryEvidenceItem>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            envelope,
            evidence_items,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SummaryBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_request.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_request.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_request.evidence_items",
                reason: "must be <= 128 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryCitationValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: SummaryRequestEnvelope,
    pub evidence_items: Vec<SummaryEvidenceItem>,
    pub summary_bullets: Vec<SummaryBullet>,
}

impl SummaryCitationValidateRequest {
    pub fn v1(
        envelope: SummaryRequestEnvelope,
        evidence_items: Vec<SummaryEvidenceItem>,
        summary_bullets: Vec<SummaryBullet>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            envelope,
            evidence_items,
            summary_bullets,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SummaryCitationValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_request.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_request.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_request.evidence_items",
                reason: "must be <= 128 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        if self.summary_bullets.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_request.summary_bullets",
                reason: "must not be empty",
            });
        }
        if self.summary_bullets.len() > self.envelope.max_summary_bullets as usize {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_request.summary_bullets",
                reason: "must be <= envelope.max_summary_bullets",
            });
        }
        for bullet in &self.summary_bullets {
            bullet.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SummaryRequest {
    SummaryBuild(SummaryBuildRequest),
    SummaryCitationValidate(SummaryCitationValidateRequest),
}

impl Validate for Ph1SummaryRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SummaryRequest::SummaryBuild(r) => r.validate(),
            Ph1SummaryRequest::SummaryCitationValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SummaryValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SummaryCapabilityId,
    pub reason_code: ReasonCodeId,
    pub summary_bullets: Vec<SummaryBullet>,
    pub evidence_backed_only: bool,
}

impl SummaryBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        summary_bullets: Vec<SummaryBullet>,
        evidence_backed_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            capability_id: SummaryCapabilityId::SummaryBuild,
            reason_code,
            summary_bullets,
            evidence_backed_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for SummaryBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_ok.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SummaryCapabilityId::SummaryBuild {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_ok.capability_id",
                reason: "must be SUMMARY_BUILD",
            });
        }
        if self.summary_bullets.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_ok.summary_bullets",
                reason: "must not be empty",
            });
        }
        if self.summary_bullets.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_ok.summary_bullets",
                reason: "must be <= 16",
            });
        }
        for bullet in &self.summary_bullets {
            bullet.validate()?;
        }
        if !self.evidence_backed_only {
            return Err(ContractViolation::InvalidValue {
                field: "summary_build_ok.evidence_backed_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryCitationValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: SummaryCapabilityId,
    pub reason_code: ReasonCodeId,
    pub validation_status: SummaryValidationStatus,
    pub diagnostics: Vec<String>,
    pub evidence_backed_only: bool,
}

impl SummaryCitationValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        validation_status: SummaryValidationStatus,
        diagnostics: Vec<String>,
        evidence_backed_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            capability_id: SummaryCapabilityId::SummaryCitationValidate,
            reason_code,
            validation_status,
            diagnostics,
            evidence_backed_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for SummaryCitationValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_ok.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        if self.capability_id != SummaryCapabilityId::SummaryCitationValidate {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_ok.capability_id",
                reason: "must be SUMMARY_CITATION_VALIDATE",
            });
        }
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for d in &self.diagnostics {
            if d.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "summary_citation_validate_ok.diagnostics",
                    reason: "entries must not be empty",
                });
            }
            if d.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "summary_citation_validate_ok.diagnostics",
                    reason: "entry must be <= 128 chars",
                });
            }
        }
        if self.validation_status == SummaryValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.evidence_backed_only {
            return Err(ContractViolation::InvalidValue {
                field: "summary_citation_validate_ok.evidence_backed_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: SummaryCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl SummaryRefuse {
    pub fn v1(
        capability_id: SummaryCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1SUMMARY_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for SummaryRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1SUMMARY_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "summary_refuse.schema_version",
                reason: "must match PH1SUMMARY_CONTRACT_VERSION",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "summary_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "summary_refuse.message",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1SummaryResponse {
    SummaryBuildOk(SummaryBuildOk),
    SummaryCitationValidateOk(SummaryCitationValidateOk),
    Refuse(SummaryRefuse),
}

impl Validate for Ph1SummaryResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1SummaryResponse::SummaryBuildOk(o) => o.validate(),
            Ph1SummaryResponse::SummaryCitationValidateOk(o) => o.validate(),
            Ph1SummaryResponse::Refuse(r) => r.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max_bullets: u8) -> SummaryRequestEnvelope {
        SummaryRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_bullets).unwrap()
    }

    fn evidence(id: &str, text: &str) -> SummaryEvidenceItem {
        SummaryEvidenceItem::v1(SummaryEvidenceId::new(id).unwrap(), text.to_string()).unwrap()
    }

    #[test]
    fn summary_build_request_rejects_empty_evidence() {
        let req = SummaryBuildRequest::v1(envelope(4), vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn summary_citation_validate_request_rejects_bullet_over_budget() {
        let req = SummaryCitationValidateRequest::v1(
            envelope(1),
            vec![evidence("e1", "line 1")],
            vec![
                SummaryBullet::v1(
                    "line 1".to_string(),
                    vec![SummaryEvidenceId::new("e1").unwrap()],
                )
                .unwrap(),
                SummaryBullet::v1(
                    "line 2".to_string(),
                    vec![SummaryEvidenceId::new("e1").unwrap()],
                )
                .unwrap(),
            ],
        );
        assert!(req.is_err());
    }

    #[test]
    fn summary_build_ok_requires_evidence_backed_only_true() {
        let ok = SummaryBuildOk::v1(
            ReasonCodeId(1),
            vec![SummaryBullet::v1(
                "line 1".to_string(),
                vec![SummaryEvidenceId::new("e1").unwrap()],
            )
            .unwrap()],
            false,
        );
        assert!(ok.is_err());
    }

    #[test]
    fn citation_validate_ok_requires_diagnostic_on_fail() {
        let ok = SummaryCitationValidateOk::v1(
            ReasonCodeId(1),
            SummaryValidationStatus::Fail,
            vec![],
            true,
        );
        assert!(ok.is_err());
    }
}
