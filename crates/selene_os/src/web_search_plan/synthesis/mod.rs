#![forbid(unsafe_code)]

pub mod boundary_guard;
pub mod citation_validator;
pub mod claim_extractor;
pub mod conflict_handler;
pub mod insufficiency_gate;
pub mod template;

use crate::web_search_plan::synthesis::boundary_guard::{
    assert_evidence_boundary, EvidenceBoundaryContext,
};
use crate::web_search_plan::synthesis::citation_validator::{
    build_evidence_citation_index, validate_claim_citation_coverage, CitationValidationError,
};
use crate::web_search_plan::synthesis::claim_extractor::{extract_atomic_claims, CitationRefKind};
use crate::web_search_plan::synthesis::conflict_handler::detect_conflicts;
use crate::web_search_plan::synthesis::insufficiency_gate::{
    assess_evidence_sufficiency, EvidenceSufficiencyPolicy,
};
use crate::web_search_plan::synthesis::template::{
    render_grounded_draft, render_insufficient_evidence_answer, TemplateChunkInput,
};
use crate::web_search_plan::diag::{
    default_failed_transitions, try_build_debug_packet, DebugPacketContext, DebugStatus,
};
use serde_json::{json, Map, Value};

pub const SYNTHESIS_TEMPLATE_VERSION: &str = "1.0.0";
pub const SYNTHESIS_ENGINE_ID: &str = "PH1.D";

pub trait ExternalLookup {
    fn lookup(&self, query: &str) -> Option<String>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SynthesisPolicy {
    pub sufficiency: EvidenceSufficiencyPolicy,
    pub max_claims: usize,
}

impl Default for SynthesisPolicy {
    fn default() -> Self {
        Self {
            sufficiency: EvidenceSufficiencyPolicy::default(),
            max_claims: 8,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesisAuditMetrics {
    pub number_of_claims: usize,
    pub number_of_citations: usize,
    pub citation_coverage_ratio: f64,
    pub conflict_detected: bool,
    pub insufficient_evidence_flag: bool,
    pub synthesis_template_version: String,
}

#[derive(Debug, Clone)]
pub struct SynthesisResult {
    pub synthesis_packet: Value,
    pub audit_metrics: SynthesisAuditMetrics,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynthesisError {
    EvidenceBoundaryViolation(String),
    InvalidEvidence(String),
    CitationMismatch(String),
    UnsupportedClaim(String),
}

pub fn synthesize_evidence_bound(
    user_question: &str,
    evidence_packet: &Value,
    created_at_ms: i64,
    trace_id: &str,
    policy: SynthesisPolicy,
    external_lookup: Option<&dyn ExternalLookup>,
) -> Result<SynthesisResult, SynthesisError> {
    assert_evidence_boundary(EvidenceBoundaryContext::from_external_lookup_requested(
        external_lookup.is_some(),
    ))
    .map_err(|violation| {
        let message = format!(
            "PH1.D evidence boundary violation: {}",
            violation.as_str()
        );
        emit_synthesis_debug_packet(
            trace_id,
            created_at_ms,
            "policy_violation",
            "policy_violation",
            &message,
        );
        SynthesisError::EvidenceBoundaryViolation(message)
    })?;

    let citation_index = build_evidence_citation_index(evidence_packet).map_err(|message| {
        emit_synthesis_debug_packet(
            trace_id,
            created_at_ms,
            "input_unparseable",
            "input_unparseable",
            &message,
        );
        SynthesisError::InvalidEvidence(message)
    })?;

    let sufficiency = assess_evidence_sufficiency(evidence_packet, policy.sufficiency);
    let conflicts = detect_conflicts(evidence_packet);

    if !sufficiency.is_sufficient {
        let answer_text = render_insufficient_evidence_answer(
            user_question,
            sufficiency.distinct_sources,
            sufficiency.chunk_support,
        );
        let uncertainty_flags = if conflicts.is_empty() {
            Vec::new()
        } else {
            vec!["conflicting_evidence_detected".to_string()]
        };
        let reason_codes = vec!["insufficient_evidence".to_string()];

        let packet = build_synthesis_packet(
            trace_id,
            created_at_ms,
            answer_text,
            vec!["Insufficient evidence for grounded synthesis.".to_string()],
            vec![],
            uncertainty_flags,
            reason_codes,
            Vec::new(),
        );

        let metrics = SynthesisAuditMetrics {
            number_of_claims: 0,
            number_of_citations: 0,
            citation_coverage_ratio: 0.0,
            conflict_detected: !conflicts.is_empty(),
            insufficient_evidence_flag: true,
            synthesis_template_version: SYNTHESIS_TEMPLATE_VERSION.to_string(),
        };

        return Ok(SynthesisResult {
            synthesis_packet: packet,
            audit_metrics: metrics,
        });
    }

    let ranked_chunks = rank_chunks_for_template(evidence_packet, policy.max_claims);
    if ranked_chunks.is_empty() {
        let message = "no evidence chunks available for synthesis".to_string();
        emit_synthesis_debug_packet(
            trace_id,
            created_at_ms,
            "insufficient_evidence",
            "insufficient_evidence",
            &message,
        );
        return Err(SynthesisError::InvalidEvidence(message));
    }

    let draft = render_grounded_draft(user_question, &ranked_chunks, &conflicts);
    let claims = extract_atomic_claims(&draft.answer_text);
    let validation = validate_claim_citation_coverage(&claims, &citation_index).map_err(|err| {
        match err {
            CitationValidationError::CitationMismatch { message, .. } => {
                emit_synthesis_debug_packet(
                    trace_id,
                    created_at_ms,
                    "citation_mismatch",
                    "citation_mismatch",
                    message.as_str(),
                );
                SynthesisError::CitationMismatch(message)
            }
            CitationValidationError::UnsupportedClaim { message, .. } => {
                emit_synthesis_debug_packet(
                    trace_id,
                    created_at_ms,
                    "unsupported_claim",
                    "unsupported_claim",
                    message.as_str(),
                );
                SynthesisError::UnsupportedClaim(message)
            }
        }
    })?;

    let mut reason_codes = Vec::new();
    if !conflicts.is_empty() {
        reason_codes.push("conflicting_evidence_detected".to_string());
    }

    let citations_json: Vec<Value> = validation
        .unique_citations
        .iter()
        .map(|citation| match citation.kind {
            CitationRefKind::ChunkId => json!({"type": "chunk_id", "value": citation.value}),
            CitationRefKind::SourceUrl => json!({"type": "source_url", "value": citation.value}),
        })
        .collect();

    let packet = build_synthesis_packet(
        trace_id,
        created_at_ms,
        draft.answer_text,
        draft.bullet_evidence,
        citations_json,
        draft.uncertainty_flags,
        reason_codes,
        draft.evidence_refs,
    );

    let metrics = SynthesisAuditMetrics {
        number_of_claims: validation.number_of_claims,
        number_of_citations: validation.number_of_citations,
        citation_coverage_ratio: validation.citation_coverage_ratio,
        conflict_detected: !conflicts.is_empty(),
        insufficient_evidence_flag: false,
        synthesis_template_version: SYNTHESIS_TEMPLATE_VERSION.to_string(),
    };

    Ok(SynthesisResult {
        synthesis_packet: packet,
        audit_metrics: metrics,
    })
}

pub fn append_synthesis_audit_fields(
    audit_packet: &mut Value,
    metrics: &SynthesisAuditMetrics,
) -> Result<(), String> {
    let obj = audit_packet
        .as_object_mut()
        .ok_or_else(|| "audit packet must be object".to_string())?;

    let transition_value = obj
        .entry("turn_state_transition".to_string())
        .or_insert_with(|| Value::Object(Map::new()));

    let transition_obj = if transition_value.is_object() {
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition must be object".to_string())?
    } else if let Some(state) = transition_value.as_str() {
        *transition_value = json!({"state": state});
        transition_value
            .as_object_mut()
            .ok_or_else(|| "turn_state_transition conversion failed".to_string())?
    } else {
        return Err("turn_state_transition must be string or object".to_string());
    };

    transition_obj.insert(
        "synthesis_audit".to_string(),
        json!({
            "number_of_claims": metrics.number_of_claims,
            "number_of_citations": metrics.number_of_citations,
            "citation_coverage_ratio": metrics.citation_coverage_ratio,
            "conflict_detected": metrics.conflict_detected,
            "insufficient_evidence_flag": metrics.insufficient_evidence_flag,
            "synthesis_template_version": metrics.synthesis_template_version,
        }),
    );

    Ok(())
}

fn rank_chunks_for_template(evidence_packet: &Value, max_claims: usize) -> Vec<TemplateChunkInput> {
    let chunks = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut ranked: Vec<(usize, usize, String, TemplateChunkInput)> = Vec::new();
    for chunk in chunks {
        let chunk_id = chunk
            .get("chunk_id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .trim()
            .to_string();
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
            .trim()
            .to_string();
        let claim_text = chunk
            .get("text_excerpt")
            .and_then(Value::as_str)
            .or_else(|| chunk.get("excerpt").and_then(Value::as_str))
            .unwrap_or("")
            .trim()
            .to_string();
        if claim_text.is_empty() {
            continue;
        }

        let text_len = chunk
            .get("text_len_chars")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or_else(|| claim_text.chars().count());
        let chunk_index = chunk
            .get("chunk_index")
            .and_then(Value::as_u64)
            .map(|value| value as usize)
            .unwrap_or(usize::MAX);

        ranked.push((
            usize::MAX.saturating_sub(text_len),
            chunk_index,
            chunk_id.clone(),
            TemplateChunkInput {
                chunk_id,
                source_url,
                claim_text,
            },
        ));
    }

    ranked.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));

    ranked
        .into_iter()
        .take(max_claims)
        .map(|(_, _, _, chunk)| chunk)
        .collect()
}

fn build_synthesis_packet(
    trace_id: &str,
    created_at_ms: i64,
    answer_text: String,
    bullet_evidence: Vec<String>,
    citations: Vec<Value>,
    uncertainty_flags: Vec<String>,
    reason_codes: Vec<String>,
    evidence_refs: Vec<String>,
) -> Value {
    json!({
        "schema_version": "1.0.0",
        "produced_by": SYNTHESIS_ENGINE_ID,
        "intended_consumers": ["PH1.WRITE"],
        "created_at_ms": created_at_ms,
        "trace_id": trace_id,
        "answer_text": answer_text,
        "bullet_evidence": bullet_evidence,
        "citations": citations,
        "uncertainty_flags": uncertainty_flags,
        "reason_codes": reason_codes,
        "evidence_refs": evidence_refs,
        "synthesis_template_version": SYNTHESIS_TEMPLATE_VERSION,
    })
}

fn emit_synthesis_debug_packet(
    trace_id: &str,
    created_at_ms: i64,
    error_kind: &str,
    reason_code: &str,
    message: &str,
) {
    let transitions = default_failed_transitions(created_at_ms);
    let _ = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status: DebugStatus::Failed,
        provider: "Synthesis",
        error_kind,
        reason_code,
        proxy_mode: None,
        source_url: None,
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(message),
        fallback_used: None,
        health_status_before_fallback: None,
    });
}

#[cfg(test)]
pub mod synthesis_tests;
