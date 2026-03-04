#![forbid(unsafe_code)]

use crate::web_search_plan::eval::corpus_packs::EvalCase;
use crate::web_search_plan::replay::runner::load_fixture_case;
use crate::web_search_plan::replay::snapshot::{build_snapshot, hash_canonical_json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseMetrics {
    pub citation_coverage_ratio: f64,
    pub refusal_correctness: bool,
    pub freshness_compliance: bool,
    pub conflict_handling: bool,
    pub trust_filtering: bool,
    pub determinism_ok: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CaseEvaluation {
    pub case_id: String,
    pub pass: bool,
    pub reasons: Vec<String>,
    pub snapshot_hash: String,
    pub metrics: CaseMetrics,
}

pub fn evaluate_cases(cases: &[EvalCase]) -> Result<Vec<CaseEvaluation>, String> {
    let mut evaluations = Vec::with_capacity(cases.len());
    for case in cases {
        evaluations.push(evaluate_case(case)?);
    }
    Ok(evaluations)
}

pub fn evaluate_case(case: &EvalCase) -> Result<CaseEvaluation, String> {
    let fixture = load_fixture_case(case.fixture_case_id.as_str())?;
    if fixture.case_id != case.fixture_case_id {
        return Err(format!(
            "fixture_case_id mismatch expected={} actual={}",
            case.fixture_case_id, fixture.case_id
        ));
    }

    let snapshot_first = build_snapshot(
        case.case_id.as_str(),
        fixture.stop_reason.as_str(),
        &fixture.evidence_packet,
        fixture.synthesis_packet.as_ref(),
        fixture.write_packet.as_ref(),
        fixture.audit_packet.as_ref(),
    )?;
    let snapshot_second = build_snapshot(
        case.case_id.as_str(),
        fixture.stop_reason.as_str(),
        &fixture.evidence_packet,
        fixture.synthesis_packet.as_ref(),
        fixture.write_packet.as_ref(),
        fixture.audit_packet.as_ref(),
    )?;

    let snapshot_hash = hash_canonical_json(&json!(snapshot_first.clone()))?;
    let determinism_ok =
        snapshot_first == snapshot_second && snapshot_hash == case.expected_snapshot_hash;

    let citation_coverage_ratio = compute_citation_coverage_ratio(fixture.synthesis_packet.as_ref())?;
    let refusal_observed = snapshot_first.reason_codes.iter().any(|code| is_refusal_code(code));
    let refusal_correctness = if case.expected_outcome == "refusal" {
        refusal_observed
    } else {
        !refusal_observed
    };

    let freshness_compliance = if case.expect_stale_refusal {
        snapshot_first
            .reason_codes
            .iter()
            .any(|code| code == "stale_data" || code == "freshness_policy_unmet")
    } else {
        true
    };

    let conflict_handling = if case.expect_conflict_flag {
        synthesis_has_conflict_flag(fixture.synthesis_packet.as_ref())?
    } else {
        true
    };

    let trust_filtering = if case.expect_trust_filtering {
        evidence_sources_trust_filtered(&fixture.evidence_packet)?
    } else {
        true
    };

    let metrics = CaseMetrics {
        citation_coverage_ratio,
        refusal_correctness,
        freshness_compliance,
        conflict_handling,
        trust_filtering,
        determinism_ok,
    };

    let mut reasons = Vec::new();
    if case.expected_outcome == "answer" && (citation_coverage_ratio - 1.0).abs() > f64::EPSILON {
        reasons.push(format!(
            "citation coverage must be 1.0 for answer case; got {}",
            citation_coverage_ratio
        ));
    }
    if !metrics.refusal_correctness {
        reasons.push("refusal correctness gate failed".to_string());
    }
    if !metrics.freshness_compliance {
        reasons.push("freshness compliance gate failed".to_string());
    }
    if !metrics.conflict_handling {
        reasons.push("conflict handling gate failed".to_string());
    }
    if !metrics.trust_filtering {
        reasons.push("trust filtering gate failed".to_string());
    }
    if !metrics.determinism_ok {
        reasons.push(format!(
            "determinism mismatch expected_snapshot_hash={} actual_snapshot_hash={}",
            case.expected_snapshot_hash, snapshot_hash
        ));
    }

    Ok(CaseEvaluation {
        case_id: case.case_id.clone(),
        pass: reasons.is_empty(),
        reasons,
        snapshot_hash,
        metrics,
    })
}

fn compute_citation_coverage_ratio(synthesis_packet: Option<&Value>) -> Result<f64, String> {
    let Some(packet) = synthesis_packet else {
        return Ok(0.0);
    };

    let bullets = packet
        .get("bullet_evidence")
        .and_then(Value::as_array)
        .ok_or_else(|| "synthesis bullet_evidence must be array".to_string())?;
    let citations = packet
        .get("citations")
        .and_then(Value::as_array)
        .ok_or_else(|| "synthesis citations must be array".to_string())?;

    if bullets.is_empty() {
        return Ok(0.0);
    }
    let covered = citations.len().min(bullets.len()) as f64;
    Ok(covered / bullets.len() as f64)
}

fn synthesis_has_conflict_flag(synthesis_packet: Option<&Value>) -> Result<bool, String> {
    let Some(packet) = synthesis_packet else {
        return Ok(false);
    };

    let uncertainty_has_conflict = packet
        .get("uncertainty_flags")
        .and_then(Value::as_array)
        .map(|flags| {
            flags.iter().any(|entry| {
                entry
                    .as_str()
                    .map(|flag| flag == "conflict_detected" || flag == "conflict_present")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    let reason_code_conflict = packet
        .get("reason_codes")
        .and_then(Value::as_array)
        .map(|codes| {
            codes.iter().any(|entry| {
                entry
                    .as_str()
                    .map(|code| code == "conflicting_evidence_detected")
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);

    Ok(uncertainty_has_conflict || reason_code_conflict)
}

fn evidence_sources_trust_filtered(evidence_packet: &Value) -> Result<bool, String> {
    let sources = evidence_packet
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| "evidence sources must be array".to_string())?;

    if sources.is_empty() {
        return Ok(false);
    }

    for source in sources {
        let tier = source
            .get("trust_tier")
            .and_then(Value::as_str)
            .unwrap_or("UNKNOWN");
        if tier == "LOW" || tier == "UNKNOWN" {
            return Ok(false);
        }
    }

    Ok(true)
}

fn is_refusal_code(code: &str) -> bool {
    matches!(
        code,
        "insufficient_evidence" | "stale_data" | "freshness_policy_unmet"
    )
}
