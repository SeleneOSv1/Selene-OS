#![forbid(unsafe_code)]

use std::cmp::min;
use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1doc::{
    DocCapabilityId, DocCitationMapBuildOk, DocCitationMapBuildRequest, DocEvidenceExtractOk,
    DocEvidenceExtractRequest, DocEvidenceId, DocEvidenceItem, DocRefuse, DocValidationStatus,
    Ph1DocRequest, Ph1DocResponse,
};
use selene_kernel_contracts::{ReasonCodeId, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DOC reason-code namespace. Values are placeholders until global registry lock.
    pub const PH1_DOC_OK_EVIDENCE_EXTRACT: ReasonCodeId = ReasonCodeId(0x444F_0001);
    pub const PH1_DOC_OK_CITATION_MAP_BUILD: ReasonCodeId = ReasonCodeId(0x444F_0002);

    pub const PH1_DOC_INPUT_SCHEMA_INVALID: ReasonCodeId = ReasonCodeId(0x444F_00F1);
    pub const PH1_DOC_UPSTREAM_INPUT_MISSING: ReasonCodeId = ReasonCodeId(0x444F_00F2);
    pub const PH1_DOC_BUDGET_EXCEEDED: ReasonCodeId = ReasonCodeId(0x444F_00F3);
    pub const PH1_DOC_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x444F_00F4);
    pub const PH1_DOC_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x444F_00F5);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1DocConfig {
    pub max_input_segments: usize,
    pub max_output_evidence_items: u8,
    pub max_citation_diagnostics: u8,
}

impl Ph1DocConfig {
    pub fn mvp_v1() -> Self {
        Self {
            max_input_segments: 128,
            max_output_evidence_items: 32,
            max_citation_diagnostics: 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Ph1DocRuntime {
    config: Ph1DocConfig,
}

impl Ph1DocRuntime {
    pub fn new(config: Ph1DocConfig) -> Self {
        Self { config }
    }

    pub fn run(&self, req: &Ph1DocRequest) -> Ph1DocResponse {
        if req.validate().is_err() {
            return self.refuse(
                capability_from_request(req),
                reason_codes::PH1_DOC_INPUT_SCHEMA_INVALID,
                "doc request failed contract validation",
            );
        }

        match req {
            Ph1DocRequest::DocEvidenceExtract(r) => self.run_extract(r),
            Ph1DocRequest::DocCitationMapBuild(r) => self.run_citation_map(r),
        }
    }

    fn run_extract(&self, req: &DocEvidenceExtractRequest) -> Ph1DocResponse {
        if req.segments.is_empty() {
            return self.refuse(
                DocCapabilityId::DocEvidenceExtract,
                reason_codes::PH1_DOC_UPSTREAM_INPUT_MISSING,
                "no document segments were provided",
            );
        }

        if req.segments.len() > self.config.max_input_segments {
            return self.refuse(
                DocCapabilityId::DocEvidenceExtract,
                reason_codes::PH1_DOC_BUDGET_EXCEEDED,
                "document segment budget exceeded",
            );
        }

        let output_budget = min(
            req.envelope.max_evidence_items,
            self.config.max_output_evidence_items,
        ) as usize;

        let mut dedupe: BTreeSet<(String, Option<u16>)> = BTreeSet::new();
        let mut out: Vec<DocEvidenceItem> = Vec::new();

        for segment in &req.segments {
            if out.len() >= output_budget {
                break;
            }
            let canonical = canonical_text(&segment.text);
            if canonical.is_empty() {
                continue;
            }
            if !dedupe.insert((canonical, segment.page_index)) {
                continue;
            }

            let evidence_id = match DocEvidenceId::new(format!(
                "doc_ev_{}_{}",
                segment.segment_id.as_str(),
                segment.page_index.unwrap_or(0)
            )) {
                Ok(id) => id,
                Err(_) => {
                    return self.refuse(
                        DocCapabilityId::DocEvidenceExtract,
                        reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                        "failed to build deterministic evidence id",
                    )
                }
            };

            let item = match DocEvidenceItem::v1(
                evidence_id,
                segment.segment_id.clone(),
                segment.page_index,
                truncate_to_char_boundary(segment.text.trim(), 512),
            ) {
                Ok(it) => it,
                Err(_) => {
                    return self.refuse(
                        DocCapabilityId::DocEvidenceExtract,
                        reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                        "failed to build doc evidence item",
                    )
                }
            };
            out.push(item);
        }

        if out.is_empty() {
            return self.refuse(
                DocCapabilityId::DocEvidenceExtract,
                reason_codes::PH1_DOC_UPSTREAM_INPUT_MISSING,
                "no document evidence could be extracted",
            );
        }

        match DocEvidenceExtractOk::v1(
            reason_codes::PH1_DOC_OK_EVIDENCE_EXTRACT,
            req.source_ref.clone(),
            out,
            true,
        ) {
            Ok(ok) => Ph1DocResponse::DocEvidenceExtractOk(ok),
            Err(_) => self.refuse(
                DocCapabilityId::DocEvidenceExtract,
                reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                "failed to construct doc extract output",
            ),
        }
    }

    fn run_citation_map(&self, req: &DocCitationMapBuildRequest) -> Ph1DocResponse {
        if req.evidence_items.is_empty() || req.citations.is_empty() {
            return self.refuse(
                DocCapabilityId::DocCitationMapBuild,
                reason_codes::PH1_DOC_UPSTREAM_INPUT_MISSING,
                "missing evidence items or citations",
            );
        }

        if req.citations.len() > req.envelope.max_evidence_items as usize {
            return self.refuse(
                DocCapabilityId::DocCitationMapBuild,
                reason_codes::PH1_DOC_BUDGET_EXCEEDED,
                "citation budget exceeded",
            );
        }

        let evidence_by_id: BTreeMap<DocEvidenceId, String> = req
            .evidence_items
            .iter()
            .map(|item| (item.evidence_id.clone(), canonical_text(&item.text)))
            .collect();

        let mut diagnostics: Vec<String> = Vec::new();

        for (idx, citation) in req.citations.iter().enumerate() {
            let mut has_match = false;
            let citation_text = canonical_text(&citation.snippet_text);

            for evidence_id in &citation.cited_evidence_ids {
                let Some(evidence_text) = evidence_by_id.get(evidence_id) else {
                    diagnostics.push(format!("citation_{idx}_missing_evidence_id"));
                    continue;
                };
                if evidence_text.contains(&citation_text) {
                    has_match = true;
                }
            }

            if !has_match {
                diagnostics.push(format!("citation_{idx}_not_evidence_backed"));
            }

            if diagnostics.len() >= self.config.max_citation_diagnostics as usize {
                break;
            }
        }

        let (status, reason_code) = if diagnostics.is_empty() {
            (
                DocValidationStatus::Ok,
                reason_codes::PH1_DOC_OK_CITATION_MAP_BUILD,
            )
        } else {
            (
                DocValidationStatus::Fail,
                reason_codes::PH1_DOC_VALIDATION_FAILED,
            )
        };

        match DocCitationMapBuildOk::v1(
            reason_code,
            req.source_ref.clone(),
            status,
            diagnostics,
            true,
        ) {
            Ok(ok) => Ph1DocResponse::DocCitationMapBuildOk(ok),
            Err(_) => self.refuse(
                DocCapabilityId::DocCitationMapBuild,
                reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                "failed to construct citation map output",
            ),
        }
    }

    fn refuse(
        &self,
        capability_id: DocCapabilityId,
        reason_code: ReasonCodeId,
        message: &'static str,
    ) -> Ph1DocResponse {
        let r = DocRefuse::v1(capability_id, reason_code, message.to_string())
            .expect("DocRefuse::v1 must construct for static message");
        Ph1DocResponse::Refuse(r)
    }
}

fn capability_from_request(req: &Ph1DocRequest) -> DocCapabilityId {
    match req {
        Ph1DocRequest::DocEvidenceExtract(_) => DocCapabilityId::DocEvidenceExtract,
        Ph1DocRequest::DocCitationMapBuild(_) => DocCapabilityId::DocCitationMapBuild,
    }
}

fn canonical_text(input: &str) -> String {
    input
        .trim()
        .to_ascii_lowercase()
        .split_whitespace()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_to_char_boundary(input: &str, max_chars: usize) -> String {
    if input.chars().count() <= max_chars {
        return input.to_string();
    }
    input.chars().take(max_chars).collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1doc::{
        DocCitation, DocCitationId, DocCitationMapBuildRequest, DocEvidenceExtractRequest,
        DocRequestEnvelope, DocumentSegment, DocumentSegmentId, DocumentSourceId,
        DocumentSourceKind, DocumentSourceRef,
    };
    use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};

    fn runtime() -> Ph1DocRuntime {
        Ph1DocRuntime::new(Ph1DocConfig::mvp_v1())
    }

    fn envelope(max_items: u8) -> DocRequestEnvelope {
        DocRequestEnvelope::v1(CorrelationId(301), TurnId(3), max_items).unwrap()
    }

    fn source() -> DocumentSourceRef {
        DocumentSourceRef::v1(
            DocumentSourceId::new("doc_src").unwrap(),
            DocumentSourceKind::Pdf,
        )
        .unwrap()
    }

    fn segment(id: &str, page: u16, text: &str) -> DocumentSegment {
        DocumentSegment::v1(
            DocumentSegmentId::new(id).unwrap(),
            Some(page),
            text.to_string(),
        )
        .unwrap()
    }

    #[test]
    fn at_doc_01_extract_output_is_schema_valid() {
        let req = Ph1DocRequest::DocEvidenceExtract(
            DocEvidenceExtractRequest::v1(
                envelope(4),
                source(),
                vec![
                    segment("s1", 1, "Section one"),
                    segment("s2", 1, "Section two"),
                ],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        assert!(out.validate().is_ok());
        match out {
            Ph1DocResponse::DocEvidenceExtractOk(ok) => {
                assert_eq!(ok.capability_id, DocCapabilityId::DocEvidenceExtract);
                assert!(ok.evidence_backed_only);
                assert_eq!(ok.evidence_items.len(), 2);
            }
            _ => panic!("expected DocEvidenceExtractOk"),
        }
    }

    #[test]
    fn at_doc_02_extract_order_is_deterministic_and_preserved() {
        let req = Ph1DocRequest::DocEvidenceExtract(
            DocEvidenceExtractRequest::v1(
                envelope(8),
                source(),
                vec![
                    segment("s1", 1, "B"),
                    segment("s2", 1, "A"),
                    segment("s3", 1, "B"),
                    segment("s4", 2, "C"),
                ],
            )
            .unwrap(),
        );

        let runtime = runtime();
        let out1 = runtime.run(&req);
        let out2 = runtime.run(&req);

        let list1 = match out1 {
            Ph1DocResponse::DocEvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|e| e.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected DocEvidenceExtractOk"),
        };
        let list2 = match out2 {
            Ph1DocResponse::DocEvidenceExtractOk(ok) => ok
                .evidence_items
                .into_iter()
                .map(|e| e.text)
                .collect::<Vec<_>>(),
            _ => panic!("expected DocEvidenceExtractOk"),
        };

        assert_eq!(list1, vec!["B", "A", "C"]);
        assert_eq!(list1, list2);
    }

    #[test]
    fn at_doc_03_budget_bound_is_enforced() {
        let req = Ph1DocRequest::DocEvidenceExtract(
            DocEvidenceExtractRequest::v1(
                envelope(2),
                source(),
                vec![
                    segment("s1", 1, "One"),
                    segment("s2", 1, "Two"),
                    segment("s3", 1, "Three"),
                ],
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1DocResponse::DocEvidenceExtractOk(ok) => {
                assert_eq!(ok.evidence_items.len(), 2);
            }
            _ => panic!("expected DocEvidenceExtractOk"),
        }
    }

    #[test]
    fn at_doc_04_citation_validation_fails_for_non_evidence_backed_snippet() {
        let extract = runtime().run(&Ph1DocRequest::DocEvidenceExtract(
            DocEvidenceExtractRequest::v1(
                envelope(4),
                source(),
                vec![
                    segment("s1", 1, "Visible line one"),
                    segment("s2", 1, "Visible line two"),
                ],
            )
            .unwrap(),
        ));

        let items = match extract {
            Ph1DocResponse::DocEvidenceExtractOk(ok) => ok.evidence_items,
            _ => panic!("expected DocEvidenceExtractOk"),
        };

        let mut citations = vec![
            DocCitation::v1(
                DocCitationId::new("cit_0").unwrap(),
                "Visible line one".to_string(),
                vec![items[0].evidence_id.clone()],
            )
            .unwrap(),
            DocCitation::v1(
                DocCitationId::new("cit_1").unwrap(),
                "Inferred hidden clause".to_string(),
                vec![items[1].evidence_id.clone()],
            )
            .unwrap(),
        ];

        let req = Ph1DocRequest::DocCitationMapBuild(
            DocCitationMapBuildRequest::v1(
                envelope(8),
                source(),
                items,
                std::mem::take(&mut citations),
            )
            .unwrap(),
        );

        let out = runtime().run(&req);
        match out {
            Ph1DocResponse::DocCitationMapBuildOk(ok) => {
                assert_eq!(ok.validation_status, DocValidationStatus::Fail);
                assert_eq!(ok.reason_code, reason_codes::PH1_DOC_VALIDATION_FAILED);
                assert!(ok
                    .diagnostics
                    .iter()
                    .any(|d| d == "citation_1_not_evidence_backed"));
            }
            _ => panic!("expected DocCitationMapBuildOk"),
        }
    }
}
