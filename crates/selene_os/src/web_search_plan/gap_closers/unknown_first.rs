#![forbid(unsafe_code)]

use crate::web_search_plan::gap_closers::claim_confidence::ClaimConfidenceItem;
use rust_decimal::{Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const UNKNOWN_FIRST_VERSION: &str = "1.0.0";

fn unknown_first_min_confidence() -> Decimal {
    Decimal::new(55, 2)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnknownFirstDecision {
    pub version: String,
    pub unknown_required: bool,
    pub reason_code: Option<String>,
    pub causes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownFirstSignals {
    pub claim_confidences: Vec<ClaimConfidenceItem>,
    pub has_unresolved_conflict: bool,
    pub citation_coverage_ratio: Decimal,
    pub explicit_reason_codes: Vec<String>,
}

pub fn evaluate_unknown_first(signals: &UnknownFirstSignals) -> UnknownFirstDecision {
    let mut causes = Vec::new();

    if signals.claim_confidences.is_empty() {
        causes.push("no_claim_confidence".to_string());
    }

    if signals.claim_confidences.iter().any(|claim| {
        parse_confidence(claim.confidence_score.as_str())
            .map(|value| value < unknown_first_min_confidence())
            .unwrap_or(true)
    }) {
        causes.push("low_claim_confidence".to_string());
    }

    if signals
        .claim_confidences
        .iter()
        .any(|claim| claim.supporting_citation_count == 0)
    {
        causes.push("claim_without_citation".to_string());
    }

    if signals.citation_coverage_ratio < Decimal::ONE {
        causes.push("citation_coverage_below_1".to_string());
    }

    if signals.has_unresolved_conflict {
        causes.push("unresolved_conflict".to_string());
    }

    if signals.explicit_reason_codes.iter().any(|code| {
        matches!(
            code.as_str(),
            "insufficient_evidence" | "citation_mismatch" | "unsupported_claim"
        )
    }) {
        causes.push("explicit_evidence_failure_reason".to_string());
    }

    causes.sort();
    causes.dedup();

    UnknownFirstDecision {
        version: UNKNOWN_FIRST_VERSION.to_string(),
        unknown_required: !causes.is_empty(),
        reason_code: if causes.is_empty() {
            None
        } else {
            Some("insufficient_evidence".to_string())
        },
        causes,
    }
}

pub fn evaluate_unknown_first_pre_synthesis(evidence_packet: &Value) -> UnknownFirstDecision {
    if let Some(explicit_required) = evidence_packet
        .pointer("/trust_metadata/planning/gap_closers/unknown_first/unknown_required")
        .and_then(Value::as_bool)
    {
        let causes = evidence_packet
            .pointer("/trust_metadata/planning/gap_closers/unknown_first/causes")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| entry.as_str().map(ToString::to_string))
            .collect::<Vec<String>>();
        return UnknownFirstDecision {
            version: UNKNOWN_FIRST_VERSION.to_string(),
            unknown_required: explicit_required,
            reason_code: if explicit_required {
                Some("insufficient_evidence".to_string())
            } else {
                None
            },
            causes,
        };
    }

    let source_count = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .map(|sources| sources.len())
        .unwrap_or(0);
    let chunk_count = evidence_packet
        .get("content_chunks")
        .and_then(Value::as_array)
        .map(|chunks| chunks.len())
        .unwrap_or(0);
    let reason_codes = evidence_packet
        .pointer("/trust_metadata/planning/reason_codes")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| entry.as_str().map(ToString::to_string))
        .collect::<Vec<String>>();
    let conflict_present = evidence_packet
        .pointer("/trust_metadata/planning/parity/stitching_summary/has_conflict")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let degraded_mode = evidence_packet
        .pointer("/trust_metadata/planning/degraded_evidence_mode")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    let mut causes = Vec::new();
    if source_count == 0 {
        causes.push("no_sources".to_string());
    }
    if chunk_count == 0
        && reason_codes
            .iter()
            .any(|code| matches!(code.as_str(), "insufficient_evidence" | "budget_exhausted"))
    {
        causes.push("no_content_chunks_with_evidence_failure".to_string());
    }
    if degraded_mode
        && reason_codes
            .iter()
            .any(|code| code == "insufficient_evidence")
    {
        causes.push("degraded_and_insufficient".to_string());
    }
    if conflict_present
        && reason_codes
            .iter()
            .any(|code| code == "conflicting_evidence_detected")
    {
        causes.push("unresolved_conflict".to_string());
    }
    if reason_codes.iter().any(|code| {
        matches!(
            code.as_str(),
            "insufficient_evidence"
                | "citation_mismatch"
                | "budget_exhausted"
                | "provider_upstream_failed"
        )
    }) {
        causes.push("planning_reason_code_requires_unknown".to_string());
    }
    causes.sort();
    causes.dedup();

    UnknownFirstDecision {
        version: UNKNOWN_FIRST_VERSION.to_string(),
        unknown_required: !causes.is_empty(),
        reason_code: if causes.is_empty() {
            None
        } else {
            Some("insufficient_evidence".to_string())
        },
        causes,
    }
}

pub fn decision_to_json(decision: &UnknownFirstDecision) -> Value {
    serde_json::to_value(decision).unwrap_or(Value::Null)
}

fn parse_confidence(raw: &str) -> Option<Decimal> {
    Decimal::from_str_exact(raw)
        .ok()
        .map(|value| value.round_dp_with_strategy(4, RoundingStrategy::MidpointAwayFromZero))
}
