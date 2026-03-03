#![forbid(unsafe_code)]

use crate::web_search_plan::synthesis::claim_parser::{CitationRef, CitationRefKind};
use crate::web_search_plan::synthesis::conflict_detector::ConflictGroup;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateChunkInput {
    pub chunk_id: String,
    pub source_url: String,
    pub claim_text: String,
}

#[derive(Debug, Clone)]
pub struct DraftSynthesis {
    pub answer_text: String,
    pub bullet_evidence: Vec<String>,
    pub citations: Vec<CitationRef>,
    pub evidence_refs: Vec<String>,
    pub uncertainty_flags: Vec<String>,
}

pub fn render_grounded_draft(
    user_question: &str,
    chunks: &[TemplateChunkInput],
    conflicts: &[ConflictGroup],
) -> DraftSynthesis {
    let mut lines = Vec::new();
    lines.push("Direct Answer".to_string());
    lines.push(format!(
        "Grounded response for question: {}",
        user_question.trim()
    ));
    lines.push(String::new());

    lines.push("Evidence".to_string());
    let mut bullet_evidence = Vec::new();
    let mut citations = Vec::new();
    let mut evidence_refs = Vec::new();

    for chunk in chunks {
        let claim_text = chunk.claim_text.trim();
        let bullet = format!(
            "- {} [chunk:{}] [url:{}]",
            claim_text, chunk.chunk_id, chunk.source_url
        );
        lines.push(bullet);
        bullet_evidence.push(claim_text.to_string());

        if !chunk.chunk_id.trim().is_empty() {
            citations.push(CitationRef {
                kind: CitationRefKind::ChunkId,
                value: chunk.chunk_id.clone(),
            });
            evidence_refs.push(chunk.chunk_id.clone());
        }
        if !chunk.source_url.trim().is_empty() {
            citations.push(CitationRef {
                kind: CitationRefKind::SourceUrl,
                value: chunk.source_url.clone(),
            });
        }
    }

    lines.push(String::new());
    lines.push("Citations".to_string());

    citations.sort();
    citations.dedup();
    evidence_refs.sort();
    evidence_refs.dedup();

    for citation in &citations {
        match citation.kind {
            CitationRefKind::ChunkId => lines.push(format!("- chunk:{}", citation.value)),
            CitationRefKind::SourceUrl => lines.push(format!("- url:{}", citation.value)),
        }
    }

    let mut uncertainty_flags = Vec::new();
    if !conflicts.is_empty() {
        uncertainty_flags.push("conflicting_evidence_detected".to_string());
        lines.push(String::new());
        lines.push("Optional Uncertainty".to_string());
        for group in conflicts {
            let mut segments = Vec::new();
            for claim in &group.claims {
                segments.push(format!(
                    "{} [chunk:{}] [url:{}]",
                    claim.claim_text, claim.chunk_id, claim.source_url
                ));
            }
            lines.push(format!(
                "- group {} conflicts: {}",
                group.group_id,
                segments.join(" || ")
            ));
        }
    }

    DraftSynthesis {
        answer_text: lines.join("\n"),
        bullet_evidence,
        citations,
        evidence_refs,
        uncertainty_flags,
    }
}

pub fn render_insufficient_evidence_answer(
    user_question: &str,
    distinct_sources: usize,
    chunk_support: usize,
    corroboration_count: usize,
) -> String {
    format!(
        "Direct Answer\nInsufficient evidence to answer question: {}\n\nEvidence\n- distinct_sources={}\n- chunk_support={}\n- corroboration_count={}\n\nCitations\n- none",
        user_question.trim(),
        distinct_sources,
        chunk_support,
        corroboration_count
    )
}
