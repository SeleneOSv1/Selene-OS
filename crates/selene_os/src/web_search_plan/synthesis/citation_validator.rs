#![forbid(unsafe_code)]

use crate::web_search_plan::synthesis::claim_parser::{AtomicClaim, CitationRef, CitationRefKind};
use serde_json::Value;
use std::collections::BTreeSet;

#[derive(Debug, Clone)]
pub struct EvidenceCitationIndex {
    pub source_urls: BTreeSet<String>,
    pub chunk_ids: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CitationValidationSummary {
    pub number_of_claims: usize,
    pub number_of_citations: usize,
    pub citation_coverage_ratio: f64,
    pub unique_citations: Vec<CitationRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitationValidationError {
    UnsupportedClaim { claim_id: String, message: String },
    CitationMismatch { claim_id: String, message: String },
}

pub fn build_evidence_citation_index(
    evidence_packet: &Value,
) -> Result<EvidenceCitationIndex, String> {
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| "evidence packet missing sources array".to_string())?;
    let content_chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .ok_or_else(|| "evidence packet missing content_chunks array".to_string())?;

    let mut source_urls = BTreeSet::new();
    let mut chunk_ids = BTreeSet::new();

    for source in sources {
        if let Some(url) = source.get("url").and_then(Value::as_str) {
            let url = url.trim();
            if !url.is_empty() {
                source_urls.insert(url.to_string());
            }
        }
    }

    for chunk in content_chunks {
        let chunk_id = chunk
            .get("chunk_id")
            .and_then(Value::as_str)
            .or_else(|| {
                chunk
                    .get("citation")
                    .and_then(Value::as_object)
                    .and_then(|citation| citation.get("chunk_id"))
                    .and_then(Value::as_str)
            })
            .unwrap_or("")
            .trim();
        if !chunk_id.is_empty() {
            chunk_ids.insert(chunk_id.to_string());
        }

        let source_url = chunk
            .get("source_url")
            .and_then(Value::as_str)
            .or_else(|| {
                chunk
                    .get("citation")
                    .and_then(Value::as_object)
                    .and_then(|citation| citation.get("source_url"))
                    .and_then(Value::as_str)
            })
            .unwrap_or("")
            .trim();
        if !source_url.is_empty() {
            source_urls.insert(source_url.to_string());
        }
    }

    Ok(EvidenceCitationIndex {
        source_urls,
        chunk_ids,
    })
}

pub fn validate_claim_citation_coverage(
    claims: &[AtomicClaim],
    evidence_index: &EvidenceCitationIndex,
) -> Result<CitationValidationSummary, CitationValidationError> {
    if claims.is_empty() {
        return Err(CitationValidationError::UnsupportedClaim {
            claim_id: "claim_000".to_string(),
            message: "no factual claims extracted from synthesis draft".to_string(),
        });
    }

    let mut covered_claims = 0usize;
    let mut total_citations = 0usize;
    let mut unique_citations: BTreeSet<CitationRef> = BTreeSet::new();

    for claim in claims {
        if claim.citations.is_empty() {
            return Err(CitationValidationError::UnsupportedClaim {
                claim_id: claim.claim_id.clone(),
                message: format!("claim '{}' has no citations", claim.text),
            });
        }

        for citation in &claim.citations {
            match citation.kind {
                CitationRefKind::ChunkId => {
                    if !evidence_index.chunk_ids.contains(citation.value.as_str()) {
                        return Err(CitationValidationError::CitationMismatch {
                            claim_id: claim.claim_id.clone(),
                            message: format!(
                                "claim references unknown chunk_id {}",
                                citation.value
                            ),
                        });
                    }
                }
                CitationRefKind::SourceUrl => {
                    if !evidence_index.source_urls.contains(citation.value.as_str()) {
                        return Err(CitationValidationError::CitationMismatch {
                            claim_id: claim.claim_id.clone(),
                            message: format!(
                                "claim references unknown source_url {}",
                                citation.value
                            ),
                        });
                    }
                }
            }
            total_citations = total_citations.saturating_add(1);
            unique_citations.insert(citation.clone());
        }

        covered_claims = covered_claims.saturating_add(1);
    }

    let citation_coverage_ratio = if claims.is_empty() {
        0.0
    } else {
        covered_claims as f64 / claims.len() as f64
    };

    Ok(CitationValidationSummary {
        number_of_claims: claims.len(),
        number_of_citations: total_citations,
        citation_coverage_ratio,
        unique_citations: unique_citations.into_iter().collect(),
    })
}
