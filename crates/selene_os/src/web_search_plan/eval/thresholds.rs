#![forbid(unsafe_code)]

use crate::web_search_plan::eval::corpus_packs::EvalCase;
use crate::web_search_plan::eval::metrics::CaseEvaluation;
use crate::web_search_plan::registry_loader::read_text;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const THRESHOLDS_FILE: &str = "eval/thresholds.json";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalThresholds {
    pub thresholds_version: String,
    pub citation_coverage_min: f64,
    pub refusal_correctness_min: f64,
    pub freshness_compliance_min: f64,
    pub determinism_required: bool,
    pub max_allowed_regressions: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThresholdOutcome {
    pub pass: bool,
    pub failing_case_ids: Vec<String>,
    pub failures: Vec<String>,
}

pub fn load_thresholds() -> Result<EvalThresholds, String> {
    let text = read_text(THRESHOLDS_FILE)?;
    serde_json::from_str::<EvalThresholds>(&text)
        .map_err(|e| format!("failed to parse {}: {}", THRESHOLDS_FILE, e))
}

pub fn validate_thresholds(thresholds: &EvalThresholds) -> Result<(), String> {
    if thresholds.thresholds_version.trim().is_empty() {
        return Err("eval thresholds_version must not be empty".to_string());
    }
    ensure_ratio("citation_coverage_min", thresholds.citation_coverage_min)?;
    ensure_ratio(
        "refusal_correctness_min",
        thresholds.refusal_correctness_min,
    )?;
    ensure_ratio(
        "freshness_compliance_min",
        thresholds.freshness_compliance_min,
    )?;
    Ok(())
}

pub fn evaluate_thresholds(
    cases: &[EvalCase],
    evaluations: &[CaseEvaluation],
    thresholds: &EvalThresholds,
) -> Result<ThresholdOutcome, String> {
    if cases.len() != evaluations.len() {
        return Err(format!(
            "threshold evaluation requires matching case/evaluation lengths; cases={} evaluations={}",
            cases.len(),
            evaluations.len()
        ));
    }

    let mut failures = Vec::new();
    let mut failing_case_ids = BTreeSet::new();

    let mut answer_total = 0u32;
    let mut answer_citation_pass = 0u32;
    let mut refusal_total = 0u32;
    let mut refusal_pass = 0u32;
    let mut freshness_total = 0u32;
    let mut freshness_pass = 0u32;

    for (case, evaluation) in cases.iter().zip(evaluations.iter()) {
        if case.expected_outcome == "answer" {
            answer_total += 1;
            if evaluation.metrics.citation_coverage_ratio + f64::EPSILON
                >= thresholds.citation_coverage_min
            {
                answer_citation_pass += 1;
            } else {
                failing_case_ids.insert(case.case_id.clone());
            }
        }

        if case.expected_outcome == "refusal" {
            refusal_total += 1;
            if evaluation.metrics.refusal_correctness {
                refusal_pass += 1;
            } else {
                failing_case_ids.insert(case.case_id.clone());
            }
        }

        if case.expect_stale_refusal {
            freshness_total += 1;
            if evaluation.metrics.freshness_compliance {
                freshness_pass += 1;
            } else {
                failing_case_ids.insert(case.case_id.clone());
            }
        }

        if thresholds.determinism_required && !evaluation.metrics.determinism_ok {
            failing_case_ids.insert(case.case_id.clone());
        }
        if !evaluation.pass {
            failing_case_ids.insert(case.case_id.clone());
        }
    }

    let citation_ratio = ratio(answer_citation_pass, answer_total);
    if citation_ratio + f64::EPSILON < thresholds.citation_coverage_min {
        failures.push(format!(
            "citation_coverage_ratio threshold failed: actual={} required={}",
            citation_ratio, thresholds.citation_coverage_min
        ));
    }

    let refusal_ratio = ratio(refusal_pass, refusal_total);
    if refusal_ratio + f64::EPSILON < thresholds.refusal_correctness_min {
        failures.push(format!(
            "refusal_correctness threshold failed: actual={} required={}",
            refusal_ratio, thresholds.refusal_correctness_min
        ));
    }

    let freshness_ratio = ratio(freshness_pass, freshness_total);
    if freshness_ratio + f64::EPSILON < thresholds.freshness_compliance_min {
        failures.push(format!(
            "freshness_compliance threshold failed: actual={} required={}",
            freshness_ratio, thresholds.freshness_compliance_min
        ));
    }

    let regression_count = failing_case_ids.len() as u32;
    if regression_count > thresholds.max_allowed_regressions {
        failures.push(format!(
            "max_allowed_regressions threshold failed: actual={} allowed={}",
            regression_count, thresholds.max_allowed_regressions
        ));
    }

    Ok(ThresholdOutcome {
        pass: failures.is_empty(),
        failing_case_ids: failing_case_ids.into_iter().collect(),
        failures,
    })
}

fn ensure_ratio(name: &str, value: f64) -> Result<(), String> {
    if (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{} must be within [0.0, 1.0], got {}", name, value))
    }
}

fn ratio(passed: u32, total: u32) -> f64 {
    if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    }
}
