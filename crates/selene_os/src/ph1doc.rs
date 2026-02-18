#![forbid(unsafe_code)]

use std::cmp::min;

use selene_kernel_contracts::ph1doc::{
    DocCapabilityId, DocCitation, DocCitationId, DocCitationMapBuildOk, DocCitationMapBuildRequest,
    DocEvidenceExtractOk, DocEvidenceExtractRequest, DocRefuse, DocRequestEnvelope,
    DocumentSegment, DocumentSourceRef, Ph1DocRequest, Ph1DocResponse,
};
use selene_kernel_contracts::ph1j::{CorrelationId, TurnId};
use selene_kernel_contracts::{ContractViolation, Validate};

pub mod reason_codes {
    use selene_kernel_contracts::ReasonCodeId;

    // PH1.DOC OS wiring reason-code namespace. Values are placeholders until registry lock.
    pub const PH1_DOC_VALIDATION_FAILED: ReasonCodeId = ReasonCodeId(0x444F_0101);
    pub const PH1_DOC_INTERNAL_PIPELINE_ERROR: ReasonCodeId = ReasonCodeId(0x444F_01F1);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ph1DocWiringConfig {
    pub doc_enabled: bool,
    pub max_evidence_items: u8,
}

impl Ph1DocWiringConfig {
    pub fn mvp_v1(doc_enabled: bool) -> Self {
        Self {
            doc_enabled,
            max_evidence_items: 32,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocTurnInput {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_ref: DocumentSourceRef,
    pub segments: Vec<DocumentSegment>,
}

impl DocTurnInput {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_ref: DocumentSourceRef,
        segments: Vec<DocumentSegment>,
    ) -> Result<Self, ContractViolation> {
        let i = Self {
            correlation_id,
            turn_id,
            source_ref,
            segments,
        };
        i.validate()?;
        Ok(i)
    }
}

impl Validate for DocTurnInput {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.source_ref.validate()?;
        if self.segments.len() > 256 {
            return Err(ContractViolation::InvalidValue {
                field: "doc_turn_input.segments",
                reason: "must be <= 256",
            });
        }
        for segment in &self.segments {
            segment.validate()?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocForwardBundle {
    pub correlation_id: CorrelationId,
    pub turn_id: TurnId,
    pub source_ref: DocumentSourceRef,
    pub extract: DocEvidenceExtractOk,
    pub citation_map: DocCitationMapBuildOk,
}

impl DocForwardBundle {
    pub fn v1(
        correlation_id: CorrelationId,
        turn_id: TurnId,
        source_ref: DocumentSourceRef,
        extract: DocEvidenceExtractOk,
        citation_map: DocCitationMapBuildOk,
    ) -> Result<Self, ContractViolation> {
        let b = Self {
            correlation_id,
            turn_id,
            source_ref,
            extract,
            citation_map,
        };
        b.validate()?;
        Ok(b)
    }
}

impl Validate for DocForwardBundle {
    fn validate(&self) -> Result<(), ContractViolation> {
        self.correlation_id.validate()?;
        self.turn_id.validate()?;
        self.source_ref.validate()?;
        self.extract.validate()?;
        self.citation_map.validate()?;
        if self.extract.source_ref != self.source_ref {
            return Err(ContractViolation::InvalidValue {
                field: "doc_forward_bundle.extract.source_ref",
                reason: "must match bundle source_ref",
            });
        }
        if self.citation_map.source_ref != self.source_ref {
            return Err(ContractViolation::InvalidValue {
                field: "doc_forward_bundle.citation_map.source_ref",
                reason: "must match bundle source_ref",
            });
        }
        if self.citation_map.validation_status
            != selene_kernel_contracts::ph1doc::DocValidationStatus::Ok
        {
            return Err(ContractViolation::InvalidValue {
                field: "doc_forward_bundle.citation_map.validation_status",
                reason: "must be OK",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocWiringOutcome {
    NotInvokedDisabled,
    NotInvokedNoDocumentInput,
    Refused(DocRefuse),
    Forwarded(DocForwardBundle),
}

pub trait Ph1DocEngine {
    fn run(&self, req: &Ph1DocRequest) -> Ph1DocResponse;
}

#[derive(Debug, Clone)]
pub struct Ph1DocWiring<E>
where
    E: Ph1DocEngine,
{
    config: Ph1DocWiringConfig,
    engine: E,
}

impl<E> Ph1DocWiring<E>
where
    E: Ph1DocEngine,
{
    pub fn new(config: Ph1DocWiringConfig, engine: E) -> Result<Self, ContractViolation> {
        if config.max_evidence_items == 0 || config.max_evidence_items > 64 {
            return Err(ContractViolation::InvalidValue {
                field: "ph1doc_wiring_config.max_evidence_items",
                reason: "must be within 1..=64",
            });
        }
        Ok(Self { config, engine })
    }

    pub fn run_turn(&self, input: &DocTurnInput) -> Result<DocWiringOutcome, ContractViolation> {
        input.validate()?;

        if !self.config.doc_enabled {
            return Ok(DocWiringOutcome::NotInvokedDisabled);
        }

        if input.segments.is_empty() {
            return Ok(DocWiringOutcome::NotInvokedNoDocumentInput);
        }

        let envelope = DocRequestEnvelope::v1(
            input.correlation_id,
            input.turn_id,
            min(self.config.max_evidence_items, 64),
        )?;

        let extract_req = Ph1DocRequest::DocEvidenceExtract(DocEvidenceExtractRequest::v1(
            envelope.clone(),
            input.source_ref.clone(),
            input.segments.clone(),
        )?);
        let extract_resp = self.engine.run(&extract_req);
        extract_resp.validate()?;

        let extract_ok = match extract_resp {
            Ph1DocResponse::Refuse(r) => return Ok(DocWiringOutcome::Refused(r)),
            Ph1DocResponse::DocEvidenceExtractOk(ok) => ok,
            Ph1DocResponse::DocCitationMapBuildOk(_) => {
                return Ok(DocWiringOutcome::Refused(DocRefuse::v1(
                    DocCapabilityId::DocEvidenceExtract,
                    reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                    "unexpected citation-map response for extract request".to_string(),
                )?))
            }
        };

        let citations = extract_ok
            .evidence_items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                DocCitation::v1(
                    DocCitationId::new(format!("doc_cit_{idx:03}"))?,
                    item.text.clone(),
                    vec![item.evidence_id.clone()],
                )
            })
            .collect::<Result<Vec<_>, ContractViolation>>()?;

        let citation_req = Ph1DocRequest::DocCitationMapBuild(DocCitationMapBuildRequest::v1(
            envelope,
            input.source_ref.clone(),
            extract_ok.evidence_items.clone(),
            citations,
        )?);
        let citation_resp = self.engine.run(&citation_req);
        citation_resp.validate()?;

        let citation_ok = match citation_resp {
            Ph1DocResponse::Refuse(r) => return Ok(DocWiringOutcome::Refused(r)),
            Ph1DocResponse::DocCitationMapBuildOk(ok) => ok,
            Ph1DocResponse::DocEvidenceExtractOk(_) => {
                return Ok(DocWiringOutcome::Refused(DocRefuse::v1(
                    DocCapabilityId::DocCitationMapBuild,
                    reason_codes::PH1_DOC_INTERNAL_PIPELINE_ERROR,
                    "unexpected extract response for citation-map request".to_string(),
                )?))
            }
        };

        if citation_ok.validation_status != selene_kernel_contracts::ph1doc::DocValidationStatus::Ok
        {
            return Ok(DocWiringOutcome::Refused(DocRefuse::v1(
                DocCapabilityId::DocCitationMapBuild,
                reason_codes::PH1_DOC_VALIDATION_FAILED,
                "doc citation map validation failed".to_string(),
            )?));
        }

        let bundle = DocForwardBundle::v1(
            input.correlation_id,
            input.turn_id,
            input.source_ref.clone(),
            extract_ok,
            citation_ok,
        )?;

        Ok(DocWiringOutcome::Forwarded(bundle))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1doc::{
        DocCitationMapBuildOk, DocEvidenceExtractOk, DocEvidenceId, DocEvidenceItem,
        DocValidationStatus, DocumentSegmentId, DocumentSourceId, DocumentSourceKind,
    };
    use selene_kernel_contracts::ReasonCodeId;

    struct DeterministicDocEngine;

    impl Ph1DocEngine for DeterministicDocEngine {
        fn run(&self, req: &Ph1DocRequest) -> Ph1DocResponse {
            match req {
                Ph1DocRequest::DocEvidenceExtract(r) => {
                    let items = r
                        .segments
                        .iter()
                        .take(r.envelope.max_evidence_items as usize)
                        .enumerate()
                        .map(|(idx, s)| {
                            DocEvidenceItem::v1(
                                DocEvidenceId::new(format!("doc_ev_{idx:03}")).unwrap(),
                                s.segment_id.clone(),
                                s.page_index,
                                s.text.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    Ph1DocResponse::DocEvidenceExtractOk(
                        DocEvidenceExtractOk::v1(
                            ReasonCodeId(1),
                            r.source_ref.clone(),
                            items,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1DocRequest::DocCitationMapBuild(r) => Ph1DocResponse::DocCitationMapBuildOk(
                    DocCitationMapBuildOk::v1(
                        ReasonCodeId(2),
                        r.source_ref.clone(),
                        DocValidationStatus::Ok,
                        vec![],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    struct DriftDocEngine;

    impl Ph1DocEngine for DriftDocEngine {
        fn run(&self, req: &Ph1DocRequest) -> Ph1DocResponse {
            match req {
                Ph1DocRequest::DocEvidenceExtract(r) => {
                    let items = r
                        .segments
                        .iter()
                        .enumerate()
                        .map(|(idx, s)| {
                            DocEvidenceItem::v1(
                                DocEvidenceId::new(format!("doc_ev_{idx:03}")).unwrap(),
                                s.segment_id.clone(),
                                s.page_index,
                                s.text.clone(),
                            )
                            .unwrap()
                        })
                        .collect::<Vec<_>>();
                    Ph1DocResponse::DocEvidenceExtractOk(
                        DocEvidenceExtractOk::v1(
                            ReasonCodeId(10),
                            r.source_ref.clone(),
                            items,
                            true,
                        )
                        .unwrap(),
                    )
                }
                Ph1DocRequest::DocCitationMapBuild(r) => Ph1DocResponse::DocCitationMapBuildOk(
                    DocCitationMapBuildOk::v1(
                        ReasonCodeId(11),
                        r.source_ref.clone(),
                        DocValidationStatus::Fail,
                        vec!["citation_1_not_evidence_backed".to_string()],
                        true,
                    )
                    .unwrap(),
                ),
            }
        }
    }

    fn source() -> DocumentSourceRef {
        DocumentSourceRef::v1(
            DocumentSourceId::new("doc_os_src").unwrap(),
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
    fn at_doc_01_os_invokes_and_returns_schema_valid_forward_bundle() {
        let wiring =
            Ph1DocWiring::new(Ph1DocWiringConfig::mvp_v1(true), DeterministicDocEngine).unwrap();

        let input = DocTurnInput::v1(
            CorrelationId(901),
            TurnId(41),
            source(),
            vec![segment("s1", 1, "Line one"), segment("s2", 1, "Line two")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            DocWiringOutcome::Forwarded(bundle) => {
                assert!(bundle.validate().is_ok());
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_doc_02_os_preserves_evidence_order_for_downstream_summary_context() {
        let wiring =
            Ph1DocWiring::new(Ph1DocWiringConfig::mvp_v1(true), DeterministicDocEngine).unwrap();

        let input = DocTurnInput::v1(
            CorrelationId(902),
            TurnId(42),
            source(),
            vec![
                segment("s1", 1, "First"),
                segment("s2", 1, "Second"),
                segment("s3", 1, "Third"),
            ],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            DocWiringOutcome::Forwarded(bundle) => {
                let list = bundle
                    .extract
                    .evidence_items
                    .into_iter()
                    .map(|e| e.text)
                    .collect::<Vec<_>>();
                assert_eq!(list, vec!["First", "Second", "Third"]);
            }
            _ => panic!("expected Forwarded"),
        }
    }

    #[test]
    fn at_doc_03_os_does_not_invoke_when_doc_is_disabled() {
        let wiring =
            Ph1DocWiring::new(Ph1DocWiringConfig::mvp_v1(false), DeterministicDocEngine).unwrap();

        let input = DocTurnInput::v1(
            CorrelationId(903),
            TurnId(43),
            source(),
            vec![segment("s1", 1, "Any line")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        assert_eq!(out, DocWiringOutcome::NotInvokedDisabled);
    }

    #[test]
    fn at_doc_04_os_fails_closed_on_citation_validation_drift() {
        let wiring = Ph1DocWiring::new(Ph1DocWiringConfig::mvp_v1(true), DriftDocEngine).unwrap();

        let input = DocTurnInput::v1(
            CorrelationId(904),
            TurnId(44),
            source(),
            vec![segment("s1", 1, "Visible line")],
        )
        .unwrap();

        let out = wiring.run_turn(&input).unwrap();
        match out {
            DocWiringOutcome::Refused(r) => {
                assert_eq!(r.reason_code, reason_codes::PH1_DOC_VALIDATION_FAILED);
                assert_eq!(r.capability_id, DocCapabilityId::DocCitationMapBuild);
            }
            _ => panic!("expected Refused"),
        }
    }
}
