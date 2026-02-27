#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1VISION_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisionCapabilityId {
    EvidenceExtract,
    VisibleContentValidate,
}

impl VisionCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            VisionCapabilityId::EvidenceExtract => "VISION_EVIDENCE_EXTRACT",
            VisionCapabilityId::VisibleContentValidate => "VISION_VISIBLE_CONTENT_VALIDATE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub opt_in_enabled: bool,
    pub max_evidence_items: u8,
}

impl VisionRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        opt_in_enabled: bool,
        max_evidence_items: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            opt_in_enabled,
            max_evidence_items,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for VisionRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_evidence_items == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.max_evidence_items",
                reason: "must be > 0",
            });
        }
        if self.max_evidence_items > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_request_envelope.max_evidence_items",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisualSourceKind {
    Image,
    Screenshot,
    Diagram,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualSourceId(String);

impl VisualSourceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for VisualSourceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualSourceRef {
    pub schema_version: SchemaVersion,
    pub source_id: VisualSourceId,
    pub source_kind: VisualSourceKind,
}

impl VisualSourceRef {
    pub fn v1(
        source_id: VisualSourceId,
        source_kind: VisualSourceKind,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            source_id,
            source_kind,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisualSourceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "visual_source_ref.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.source_id.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisionRawSourceRef {
    pub schema_version: SchemaVersion,
    pub image_ref: Option<String>,
    pub blob_ref: Option<String>,
}

impl VisionRawSourceRef {
    pub fn v1(
        image_ref: Option<String>,
        blob_ref: Option<String>,
    ) -> Result<Self, ContractViolation> {
        let raw = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            image_ref,
            blob_ref,
        };
        raw.validate()?;
        Ok(raw)
    }
}

impl Validate for VisionRawSourceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_raw_source_ref.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        validate_optional_ref("vision_raw_source_ref.image_ref", self.image_ref.as_deref())?;
        validate_optional_ref("vision_raw_source_ref.blob_ref", self.blob_ref.as_deref())?;
        if self.image_ref.is_none() && self.blob_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_raw_source_ref",
                reason: "must include image_ref or blob_ref",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundingBoxPx {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl BoundingBoxPx {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Result<Self, ContractViolation> {
        let b = Self { x, y, w, h };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for BoundingBoxPx {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.w == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bounding_box_px.w",
                reason: "must be > 0",
            });
        }
        if self.h == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "bounding_box_px.h",
                reason: "must be > 0",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisualToken {
    pub schema_version: SchemaVersion,
    pub token: String,
    pub bbox: Option<BoundingBoxPx>,
}

impl VisualToken {
    pub fn v1(token: String, bbox: Option<BoundingBoxPx>) -> Result<Self, ContractViolation> {
        let t = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            token,
            bbox,
        };
        t.validate()?;
        Ok(t)
    }
}

impl Validate for VisualToken {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.token.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must not be empty",
            });
        }
        if self.token.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must be <= 256 chars",
            });
        }
        if self.token.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "visual_token.token",
                reason: "must not contain control characters",
            });
        }
        if let Some(b) = &self.bbox {
            b.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionEvidenceExtractRequest {
    pub schema_version: SchemaVersion,
    pub envelope: VisionRequestEnvelope,
    pub source_ref: VisualSourceRef,
    pub raw_source_ref: Option<VisionRawSourceRef>,
    /// Input token list is the strict visible-content plane. No inference beyond these tokens.
    pub visible_tokens: Vec<VisualToken>,
}

impl VisionEvidenceExtractRequest {
    pub fn v1(
        envelope: VisionRequestEnvelope,
        source_ref: VisualSourceRef,
        raw_source_ref: Option<VisionRawSourceRef>,
        visible_tokens: Vec<VisualToken>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            envelope,
            source_ref,
            raw_source_ref,
            visible_tokens,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionEvidenceExtractRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;
        if self.visible_tokens.is_empty() && self.raw_source_ref.is_none() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request",
                reason: "must include visible_tokens or raw_source_ref",
            });
        }
        if self.visible_tokens.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_request.visible_tokens",
                reason: "must be <= 256 items",
            });
        }
        for token in &self.visible_tokens {
            token.validate()?;
        }
        if let Some(raw_source_ref) = &self.raw_source_ref {
            raw_source_ref.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionVisibleContentValidateRequest {
    pub schema_version: SchemaVersion,
    pub envelope: VisionRequestEnvelope,
    pub source_ref: VisualSourceRef,
    pub visible_tokens: Vec<VisualToken>,
    pub evidence_items: Vec<VisionEvidenceItem>,
}

impl VisionVisibleContentValidateRequest {
    pub fn v1(
        envelope: VisionRequestEnvelope,
        source_ref: VisualSourceRef,
        visible_tokens: Vec<VisualToken>,
        evidence_items: Vec<VisionEvidenceItem>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            envelope,
            source_ref,
            visible_tokens,
            evidence_items,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionVisibleContentValidateRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;

        if self.visible_tokens.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.visible_tokens",
                reason: "must not be empty",
            });
        }
        if self.visible_tokens.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.visible_tokens",
                reason: "must be <= 256 items",
            });
        }
        for token in &self.visible_tokens {
            token.validate()?;
        }

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > self.envelope.max_evidence_items as usize {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_request.evidence_items",
                reason: "must be <= envelope.max_evidence_items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1VisionRequest {
    EvidenceExtract(VisionEvidenceExtractRequest),
    VisibleContentValidate(VisionVisibleContentValidateRequest),
}

impl Validate for Ph1VisionRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VisionRequest::EvidenceExtract(r) => r.validate(),
            Ph1VisionRequest::VisibleContentValidate(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VisionEvidenceItem {
    pub schema_version: SchemaVersion,
    pub text: String,
    pub bbox: Option<BoundingBoxPx>,
}

impl VisionEvidenceItem {
    pub fn v1(text: String, bbox: Option<BoundingBoxPx>) -> Result<Self, ContractViolation> {
        let i = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            text,
            bbox,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for VisionEvidenceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must be <= 256 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_item.text",
                reason: "must not contain control characters",
            });
        }
        if let Some(b) = &self.bbox {
            b.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VisionValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionEvidenceExtractOk {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: VisualSourceRef,
    pub evidence_items: Vec<VisionEvidenceItem>,
    pub visible_content_only: bool,
}

impl VisionEvidenceExtractOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: VisualSourceRef,
        evidence_items: Vec<VisionEvidenceItem>,
        visible_content_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id: VisionCapabilityId::EvidenceExtract,
            reason_code,
            source_ref,
            evidence_items,
            visible_content_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for VisionEvidenceExtractOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.capability_id != VisionCapabilityId::EvidenceExtract {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.capability_id",
                reason: "must be VISION_EVIDENCE_EXTRACT",
            });
        }
        self.source_ref.validate()?;

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.evidence_items",
                reason: "must be <= 64 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        if !self.visible_content_only {
            return Err(ContractViolation::InvalidValue {
                field: "vision_evidence_extract_ok.visible_content_only",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionVisibleContentValidateOk {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: VisualSourceRef,
    pub validation_status: VisionValidationStatus,
    pub diagnostics: Vec<String>,
    pub visible_content_only: bool,
}

impl VisionVisibleContentValidateOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: VisualSourceRef,
        validation_status: VisionValidationStatus,
        diagnostics: Vec<String>,
        visible_content_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id: VisionCapabilityId::VisibleContentValidate,
            reason_code,
            source_ref,
            validation_status,
            diagnostics,
            visible_content_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for VisionVisibleContentValidateOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.capability_id != VisionCapabilityId::VisibleContentValidate {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.capability_id",
                reason: "must be VISION_VISIBLE_CONTENT_VALIDATE",
            });
        }
        self.source_ref.validate()?;

        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.diagnostics",
                reason: "must be <= 16 items",
            });
        }
        for d in &self.diagnostics {
            if d.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_visible_content_validate_ok.diagnostics",
                    reason: "entries must not be empty",
                });
            }
            if d.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "vision_visible_content_validate_ok.diagnostics",
                    reason: "entry must be <= 128 chars",
                });
            }
        }
        if self.validation_status == VisionValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.diagnostics",
                reason: "must include at least one diagnostic when validation_status=FAIL",
            });
        }

        if !self.visible_content_only {
            return Err(ContractViolation::InvalidValue {
                field: "vision_visible_content_validate_ok.visible_content_only",
                reason: "must be true",
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisionRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: VisionCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl VisionRefuse {
    pub fn v1(
        capability_id: VisionCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1VISION_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for VisionRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1VISION_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.schema_version",
                reason: "must match PH1VISION_CONTRACT_VERSION",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "vision_refuse.message",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1VisionResponse {
    EvidenceExtractOk(VisionEvidenceExtractOk),
    VisibleContentValidateOk(VisionVisibleContentValidateOk),
    Refuse(VisionRefuse),
}

impl Validate for Ph1VisionResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1VisionResponse::EvidenceExtractOk(o) => o.validate(),
            Ph1VisionResponse::VisibleContentValidateOk(o) => o.validate(),
            Ph1VisionResponse::Refuse(r) => r.validate(),
        }
    }
}

fn validate_optional_ref(field: &'static str, value: Option<&str>) -> Result<(), ContractViolation> {
    let Some(value) = value else {
        return Ok(());
    };
    if value.trim().is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty when provided",
        });
    }
    if value.len() > 512 {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must be <= 512 chars",
        });
    }
    if value.chars().any(|c| c.is_control()) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(max: u8, opt_in: bool) -> VisionRequestEnvelope {
        VisionRequestEnvelope::v1(CorrelationId(1), TurnId(1), opt_in, max).unwrap()
    }

    fn source() -> VisualSourceRef {
        VisualSourceRef::v1(
            VisualSourceId::new("img_001").unwrap(),
            VisualSourceKind::Image,
        )
        .unwrap()
    }

    fn token(s: &str) -> VisualToken {
        VisualToken::v1(s.to_string(), None).unwrap()
    }

    #[test]
    fn envelope_rejects_zero_max_evidence_items() {
        let env = VisionRequestEnvelope::v1(CorrelationId(1), TurnId(1), true, 0);
        assert!(env.is_err());
    }

    #[test]
    fn extract_request_requires_visible_tokens() {
        let req = VisionEvidenceExtractRequest::v1(envelope(4, true), source(), None, vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn validate_request_rejects_more_evidence_than_envelope_budget() {
        let req = VisionVisibleContentValidateRequest::v1(
            envelope(1, true),
            source(),
            vec![token("hello")],
            vec![
                VisionEvidenceItem::v1("hello".to_string(), None).unwrap(),
                VisionEvidenceItem::v1("world".to_string(), None).unwrap(),
            ],
        );
        assert!(req.is_err());
    }

    #[test]
    fn extract_request_accepts_raw_source_without_visible_tokens() {
        let req = VisionEvidenceExtractRequest::v1(
            envelope(4, true),
            source(),
            Some(
                VisionRawSourceRef::v1(
                    Some("image://capture_001".to_string()),
                    Some("blob://vision/0001".to_string()),
                )
                .unwrap(),
            ),
            vec![],
        );
        assert!(req.is_ok());
    }

    #[test]
    fn evidence_ok_requires_visible_content_only_true() {
        let out = VisionEvidenceExtractOk::v1(
            ReasonCodeId(1),
            source(),
            vec![VisionEvidenceItem::v1("invoice_total".to_string(), None).unwrap()],
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn validate_ok_requires_diagnostic_when_status_fail() {
        let out = VisionVisibleContentValidateOk::v1(
            ReasonCodeId(1),
            source(),
            VisionValidationStatus::Fail,
            vec![],
            true,
        );
        assert!(out.is_err());
    }
}
