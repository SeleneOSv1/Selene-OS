#![forbid(unsafe_code)]

use crate::web_search_plan::reason_code_validator::validate_reason_codes_registered;
use crate::web_search_plan::registry_loader::{load_reason_code_registry, read_text};
use crate::web_search_plan::replay::corpus::REPLAY_EXPECTED_FILE;
use crate::web_search_plan::replay::runner::ReplayCaseResult;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExpectedSnapshot {
    pub evidence_hash: String,
    pub synthesis_hash: Option<String>,
    pub write_hash: Option<String>,
    pub audit_hash: Option<String>,
    pub stop_reason: String,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedQualityGates {
    pub citation_coverage_ratio_min: f64,
    pub expect_refusal: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExpectedReplayCase {
    pub case_id: String,
    pub snapshot: ExpectedSnapshot,
    pub quality_gates: ExpectedQualityGates,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayExpected {
    pub expected_version: String,
    pub cases: Vec<ExpectedReplayCase>,
}

pub fn load_replay_expected() -> Result<ReplayExpected, String> {
    let text = read_text(REPLAY_EXPECTED_FILE)?;
    serde_json::from_str::<ReplayExpected>(&text)
        .map_err(|e| format!("failed to parse {}: {}", REPLAY_EXPECTED_FILE, e))
}

pub fn evaluate_regressions(
    results: &[ReplayCaseResult],
    expected: &ReplayExpected,
) -> Result<(), String> {
    if expected.expected_version.trim().is_empty() {
        return Err("replay expected_version must not be empty".to_string());
    }

    let reason_registry = load_reason_code_registry()?;

    let actual_map: BTreeMap<&str, &ReplayCaseResult> =
        results.iter().map(|result| (result.case_id.as_str(), result)).collect();

    if expected.cases.len() != actual_map.len() {
        return Err(format!(
            "expected case count {} does not match actual {}",
            expected.cases.len(),
            actual_map.len()
        ));
    }

    for expected_case in &expected.cases {
        let actual = actual_map
            .get(expected_case.case_id.as_str())
            .ok_or_else(|| format!("missing replay result for case {}", expected_case.case_id))?;

        let snapshot = &actual.snapshot;
        if snapshot.evidence_hash != expected_case.snapshot.evidence_hash
            || snapshot.synthesis_hash != expected_case.snapshot.synthesis_hash
            || snapshot.write_hash != expected_case.snapshot.write_hash
            || snapshot.audit_hash != expected_case.snapshot.audit_hash
            || snapshot.stop_reason != expected_case.snapshot.stop_reason
            || snapshot.reason_codes != expected_case.snapshot.reason_codes
        {
            return Err(format!(
                "snapshot regression for case {} expected={:?} actual={:?}",
                expected_case.case_id, expected_case.snapshot, snapshot
            ));
        }

        validate_reason_codes_registered(&snapshot.reason_codes, &reason_registry)?;

        if actual.metrics.citation_coverage_ratio + f64::EPSILON
            < expected_case.quality_gates.citation_coverage_ratio_min
        {
            return Err(format!(
                "citation coverage gate failed for case {}: actual={} min={}",
                expected_case.case_id,
                actual.metrics.citation_coverage_ratio,
                expected_case.quality_gates.citation_coverage_ratio_min
            ));
        }

        if !expected_case.quality_gates.expect_refusal
            && (actual.metrics.citation_coverage_ratio - 1.0).abs() > f64::EPSILON
        {
            return Err(format!(
                "answer case {} citation coverage must remain exactly 1.0; got {}",
                expected_case.case_id, actual.metrics.citation_coverage_ratio
            ));
        }

        if !actual.metrics.refusal_correctness {
            return Err(format!(
                "refusal correctness failed for case {}",
                expected_case.case_id
            ));
        }

        if !actual.metrics.latency_budget_compliance {
            return Err(format!(
                "latency budget compliance failed for case {}",
                expected_case.case_id
            ));
        }

        if !actual.metrics.determinism_ok {
            return Err(format!(
                "determinism check failed for case {}",
                expected_case.case_id
            ));
        }
    }

    Ok(())
}
