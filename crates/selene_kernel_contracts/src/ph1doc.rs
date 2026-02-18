#![forbid(unsafe_code)]

use crate::ph1j::{CorrelationId, TurnId};
use crate::{ContractViolation, ReasonCodeId, SchemaVersion, Validate};

pub const PH1DOC_CONTRACT_VERSION: SchemaVersion = SchemaVersion(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocCapabilityId {
    DocEvidenceExtract,
    DocCitationMapBuild,
}

impl DocCapabilityId {
    pub fn as_str(self) -> &'static str {
        match self {
            DocCapabilityId::DocEvidenceExtract => "DOC_EVIDENCE_EXTRACT",
            DocCapabilityId::DocCitationMapBuild => "DOC_CITATION_MAP_BUILD",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRequestEnvelope {
    pub schema_version: SchemaVersion,
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub max_evidence_items: u8,
}

impl DocRequestEnvelope {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        max_evidence_items: u8,
    ) -> Result<Self, ContractViolation> {
        let env = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            correlation_id,
            turn_id,
            max_evidence_items,
        };
        env.validate()?;
        Ok(env)
    }
}

impl Validate for DocRequestEnvelope {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_request_envelope.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        if self.max_evidence_items == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_request_envelope.max_evidence_items",
                reason: "must be > 0",
            });
        }
        if self.max_evidence_items > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_request_envelope.max_evidence_items",
                reason: "must be <= 64",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocumentSourceKind {
    Pdf,
    Word,
    Html,
    Scan,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentSourceId(String);

impl DocumentSourceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DocumentSourceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSourceRef {
    pub schema_version: SchemaVersion,
    pub source_id: DocumentSourceId,
    pub source_kind: DocumentSourceKind,
}

impl DocumentSourceRef {
    pub fn v1(
        source_id: DocumentSourceId,
        source_kind: DocumentSourceKind,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            source_id,
            source_kind,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DocumentSourceRef {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "document_source_ref.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.source_id.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocumentSegmentId(String);

impl DocumentSegmentId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DocumentSegmentId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentSegment {
    pub schema_version: SchemaVersion,
    pub segment_id: DocumentSegmentId,
    pub page_index: Option<u16>,
    pub text: String,
}

impl DocumentSegment {
    pub fn v1(
        segment_id: DocumentSegmentId,
        page_index: Option<u16>,
        text: String,
    ) -> Result<Self, ContractViolation> {
        let s = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            segment_id,
            page_index,
            text,
        };
        s.validate()?;
        Ok(s)
    }
}

impl Validate for DocumentSegment {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.segment_id.validate()?;
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 1024 {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment.text",
                reason: "must be <= 1024 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "document_segment.text",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocEvidenceId(String);

impl DocEvidenceId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DocEvidenceId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEvidenceItem {
    pub schema_version: SchemaVersion,
    pub evidence_id: DocEvidenceId,
    pub segment_id: DocumentSegmentId,
    pub page_index: Option<u16>,
    pub text: String,
}

impl DocEvidenceItem {
    pub fn v1(
        evidence_id: DocEvidenceId,
        segment_id: DocumentSegmentId,
        page_index: Option<u16>,
        text: String,
    ) -> Result<Self, ContractViolation> {
        let e = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            evidence_id,
            segment_id,
            page_index,
            text,
        };
        e.validate()?;
        Ok(e)
    }
}

impl Validate for DocEvidenceItem {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_item.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.evidence_id.validate()?;
        self.segment_id.validate()?;
        if self.text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_item.text",
                reason: "must not be empty",
            });
        }
        if self.text.len() > 512 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_item.text",
                reason: "must be <= 512 chars",
            });
        }
        if self.text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_item.text",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DocCitationId(String);

impl DocCitationId {
    pub fn new(id: impl Into<String>) -> Result<Self, ContractViolation> {
        let id = id.into();
        if id.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must not be empty",
            });
        }
        if id.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must be <= 128 chars",
            });
        }
        if id.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must not contain control characters",
            });
        }
        Ok(Self(id))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Validate for DocCitationId {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.0.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must not be empty",
            });
        }
        if self.0.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must be <= 128 chars",
            });
        }
        if self.0.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_id",
                reason: "must not contain control characters",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCitation {
    pub schema_version: SchemaVersion,
    pub citation_id: DocCitationId,
    pub snippet_text: String,
    pub cited_evidence_ids: Vec<DocEvidenceId>,
}

impl DocCitation {
    pub fn v1(
        citation_id: DocCitationId,
        snippet_text: String,
        cited_evidence_ids: Vec<DocEvidenceId>,
    ) -> Result<Self, ContractViolation> {
        let c = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            citation_id,
            snippet_text,
            cited_evidence_ids,
        };
        c.validate()?;
        Ok(c)
    }
}

impl Validate for DocCitation {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.citation_id.validate()?;
        if self.snippet_text.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.snippet_text",
                reason: "must not be empty",
            });
        }
        if self.snippet_text.len() > 240 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.snippet_text",
                reason: "must be <= 240 chars",
            });
        }
        if self.snippet_text.chars().any(|c| c.is_control()) {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.snippet_text",
                reason: "must not contain control characters",
            });
        }
        if self.cited_evidence_ids.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.cited_evidence_ids",
                reason: "must not be empty",
            });
        }
        if self.cited_evidence_ids.len() > 6 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation.cited_evidence_ids",
                reason: "must be <= 6",
            });
        }
        for id in &self.cited_evidence_ids {
            id.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEvidenceExtractRequest {
    pub schema_version: SchemaVersion,
    pub envelope: DocRequestEnvelope,
    pub source_ref: DocumentSourceRef,
    pub segments: Vec<DocumentSegment>,
}

impl DocEvidenceExtractRequest {
    pub fn v1(
        envelope: DocRequestEnvelope,
        source_ref: DocumentSourceRef,
        segments: Vec<DocumentSegment>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            envelope,
            source_ref,
            segments,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DocEvidenceExtractRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_request.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;
        if self.segments.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_request.segments",
                reason: "must not be empty",
            });
        }
        if self.segments.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_request.segments",
                reason: "must be <= 256 items",
            });
        }
        for s in &self.segments {
            s.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCitationMapBuildRequest {
    pub schema_version: SchemaVersion,
    pub envelope: DocRequestEnvelope,
    pub source_ref: DocumentSourceRef,
    pub evidence_items: Vec<DocEvidenceItem>,
    pub citations: Vec<DocCitation>,
}

impl DocCitationMapBuildRequest {
    pub fn v1(
        envelope: DocRequestEnvelope,
        source_ref: DocumentSourceRef,
        evidence_items: Vec<DocEvidenceItem>,
        citations: Vec<DocCitation>,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            envelope,
            source_ref,
            evidence_items,
            citations,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DocCitationMapBuildRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_request.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        self.envelope.validate()?;
        self.source_ref.validate()?;

        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_request.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 128 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_request.evidence_items",
                reason: "must be <= 128 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }

        if self.citations.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_request.citations",
                reason: "must not be empty",
            });
        }
        if self.citations.len() > self.envelope.max_evidence_items as usize {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_request.citations",
                reason: "must be <= envelope.max_evidence_items",
            });
        }
        for citation in &self.citations {
            citation.validate()?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DocRequest {
    DocEvidenceExtract(DocEvidenceExtractRequest),
    DocCitationMapBuild(DocCitationMapBuildRequest),
}

impl Validate for Ph1DocRequest {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1DocRequest::DocEvidenceExtract(r) => r.validate(),
            Ph1DocRequest::DocCitationMapBuild(r) => r.validate(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DocValidationStatus {
    Ok,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocEvidenceExtractOk {
    pub schema_version: SchemaVersion,
    pub capability_id: DocCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: DocumentSourceRef,
    pub evidence_items: Vec<DocEvidenceItem>,
    pub evidence_backed_only: bool,
}

impl DocEvidenceExtractOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: DocumentSourceRef,
        evidence_items: Vec<DocEvidenceItem>,
        evidence_backed_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            capability_id: DocCapabilityId::DocEvidenceExtract,
            reason_code,
            source_ref,
            evidence_items,
            evidence_backed_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for DocEvidenceExtractOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_ok.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        if self.capability_id != DocCapabilityId::DocEvidenceExtract {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_ok.capability_id",
                reason: "must be DOC_EVIDENCE_EXTRACT",
            });
        }
        self.source_ref.validate()?;
        if self.evidence_items.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_ok.evidence_items",
                reason: "must not be empty",
            });
        }
        if self.evidence_items.len() > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_ok.evidence_items",
                reason: "must be <= 64 items",
            });
        }
        for item in &self.evidence_items {
            item.validate()?;
        }
        if !self.evidence_backed_only {
            return Err(ContractViolation::InvalidValue {
                field: "doc_evidence_extract_ok.evidence_backed_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocCitationMapBuildOk {
    pub schema_version: SchemaVersion,
    pub capability_id: DocCapabilityId,
    pub reason_code: ReasonCodeId,
    pub source_ref: DocumentSourceRef,
    pub validation_status: DocValidationStatus,
    pub diagnostics: Vec<String>,
    pub evidence_backed_only: bool,
}

impl DocCitationMapBuildOk {
    pub fn v1(
        reason_code: ReasonCodeId,
        source_ref: DocumentSourceRef,
        validation_status: DocValidationStatus,
        diagnostics: Vec<String>,
        evidence_backed_only: bool,
    ) -> Result<Self, ContractViolation> {
        let o = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            capability_id: DocCapabilityId::DocCitationMapBuild,
            reason_code,
            source_ref,
            validation_status,
            diagnostics,
            evidence_backed_only,
        };
        o.validate()?;
        Ok(o)
    }
}

impl Validate for DocCitationMapBuildOk {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_ok.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        if self.capability_id != DocCapabilityId::DocCitationMapBuild {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_ok.capability_id",
                reason: "must be DOC_CITATION_MAP_BUILD",
            });
        }
        self.source_ref.validate()?;
        if self.diagnostics.len() > 16 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_ok.diagnostics",
                reason: "must be <= 16",
            });
        }
        for d in &self.diagnostics {
            if d.trim().is_empty() {
                return Err(ContractViolation::InvalidValue {
                    field: "doc_citation_map_build_ok.diagnostics",
                    reason: "entries must not be empty",
                });
            }
            if d.len() > 128 {
                return Err(ContractViolation::InvalidValue {
                    field: "doc_citation_map_build_ok.diagnostics",
                    reason: "entry must be <= 128 chars",
                });
            }
        }
        if self.validation_status == DocValidationStatus::Fail && self.diagnostics.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_ok.diagnostics",
                reason: "must include diagnostics when validation_status=FAIL",
            });
        }
        if !self.evidence_backed_only {
            return Err(ContractViolation::InvalidValue {
                field: "doc_citation_map_build_ok.evidence_backed_only",
                reason: "must be true",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocRefuse {
    pub schema_version: SchemaVersion,
    pub capability_id: DocCapabilityId,
    pub reason_code: ReasonCodeId,
    pub message: String,
}

impl DocRefuse {
    pub fn v1(
        capability_id: DocCapabilityId,
        reason_code: ReasonCodeId,
        message: String,
    ) -> Result<Self, ContractViolation> {
        let r = Self {
            schema_version: PH1DOC_CONTRACT_VERSION,
            capability_id,
            reason_code,
            message,
        };
        r.validate()?;
        Ok(r)
    }
}

impl Validate for DocRefuse {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.schema_version != PH1DOC_CONTRACT_VERSION {
            return Err(ContractViolation::InvalidValue {
                field: "doc_refuse.schema_version",
                reason: "must match PH1DOC_CONTRACT_VERSION",
            });
        }
        if self.message.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "doc_refuse.message",
                reason: "must not be empty",
            });
        }
        if self.message.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_refuse.message",
                reason: "must be <= 256 chars",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ph1DocResponse {
    DocEvidenceExtractOk(DocEvidenceExtractOk),
    DocCitationMapBuildOk(DocCitationMapBuildOk),
    Refuse(DocRefuse),
}

impl Validate for Ph1DocResponse {
    fn validate(&self) -> Result<(), ContractViolation> {
        match self {
            Ph1DocResponse::DocEvidenceExtractOk(o) => o.validate(),
            Ph1DocResponse::DocCitationMapBuildOk(o) => o.validate(),
            Ph1DocResponse::Refuse(r) => r.validate(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(max_items: u8) -> DocRequestEnvelope {
        DocRequestEnvelope::v1(CorrelationId(1), TurnId(1), max_items).unwrap()
    }

    fn source() -> DocumentSourceRef {
        DocumentSourceRef::v1(
            DocumentSourceId::new("doc_001").unwrap(),
            DocumentSourceKind::Pdf,
        )
        .unwrap()
    }

    fn segment(id: &str, text: &str) -> DocumentSegment {
        DocumentSegment::v1(
            DocumentSegmentId::new(id).unwrap(),
            Some(1),
            text.to_string(),
        )
        .unwrap()
    }

    #[test]
    fn evidence_extract_request_rejects_empty_segments() {
        let req = DocEvidenceExtractRequest::v1(env(4), source(), vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn citation_map_build_request_rejects_empty_citations() {
        let item = DocEvidenceItem::v1(
            DocEvidenceId::new("ev1").unwrap(),
            DocumentSegmentId::new("seg1").unwrap(),
            Some(1),
            "line".to_string(),
        )
        .unwrap();
        let req = DocCitationMapBuildRequest::v1(env(4), source(), vec![item], vec![]);
        assert!(req.is_err());
    }

    #[test]
    fn evidence_extract_ok_requires_evidence_backed_only_true() {
        let out = DocEvidenceExtractOk::v1(
            ReasonCodeId(1),
            source(),
            vec![DocEvidenceItem::v1(
                DocEvidenceId::new("ev1").unwrap(),
                DocumentSegmentId::new("seg1").unwrap(),
                Some(1),
                "line".to_string(),
            )
            .unwrap()],
            false,
        );
        assert!(out.is_err());
    }

    #[test]
    fn citation_map_ok_requires_diagnostic_when_status_fail() {
        let out = DocCitationMapBuildOk::v1(
            ReasonCodeId(1),
            source(),
            DocValidationStatus::Fail,
            vec![],
            true,
        );
        assert!(out.is_err());
    }

    #[test]
    fn segment_text_is_bounded() {
        let text = "x".repeat(1025);
        let s = DocumentSegment::v1(DocumentSegmentId::new("seg").unwrap(), Some(1), text);
        assert!(s.is_err());
    }

    #[test]
    fn citation_has_required_evidence_refs() {
        let c = DocCitation::v1(
            DocCitationId::new("cit").unwrap(),
            "snippet".to_string(),
            vec![],
        );
        assert!(c.is_err());
    }

    #[test]
    fn helper_segment_builder_compiles() {
        let s = segment("seg_a", "content");
        assert_eq!(s.segment_id.as_str(), "seg_a");
    }
}
