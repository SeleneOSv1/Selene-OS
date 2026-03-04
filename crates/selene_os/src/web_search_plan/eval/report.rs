#![forbid(unsafe_code)]

use crate::web_search_plan::eval::corpus_packs::{
    load_default_corpus_packs, merge_cases, EvalCase, DEFAULT_CORPUS_PACK_FILES,
};
use crate::web_search_plan::eval::metrics::{evaluate_cases, CaseEvaluation};
use crate::web_search_plan::eval::thresholds::{
    evaluate_thresholds, load_thresholds, validate_thresholds,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousEvalConfig {
    pub head_commit: String,
    pub run_timestamp_utc: String,
    pub date_tag: String,
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinuousEvalOutcome {
    pub report_path: PathBuf,
    pub overall_pass: bool,
    pub failing_case_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSummary {
    pub passed: u32,
    pub total: u32,
    pub ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricSummaries {
    pub citation_coverage: MetricSummary,
    pub refusal_correctness: MetricSummary,
    pub freshness_compliance: MetricSummary,
    pub conflict_handling: MetricSummary,
    pub trust_filtering: MetricSummary,
    pub determinism_ok: MetricSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalTotals {
    pub cases_passed: u32,
    pub cases_failed: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvalReport {
    pub head_commit: String,
    pub run_timestamp_utc: String,
    pub corpus_pack_id: String,
    pub thresholds_version: String,
    pub totals: EvalTotals,
    pub per_metric: MetricSummaries,
    pub per_case_results: Vec<CaseEvaluation>,
    pub failing_case_ids: Vec<String>,
    pub threshold_failures: Vec<String>,
    pub overall_pass: bool,
}

pub fn generate_eval_report(config: &ContinuousEvalConfig) -> Result<ContinuousEvalOutcome, String> {
    if config.head_commit.trim().is_empty() {
        return Err("head_commit is required".to_string());
    }
    if config.run_timestamp_utc.trim().is_empty() {
        return Err("run_timestamp_utc is required".to_string());
    }
    if config.date_tag.trim().is_empty() {
        return Err("date_tag is required".to_string());
    }

    let packs = load_default_corpus_packs()?;
    let cases = merge_cases(&packs)?;
    let evaluations = evaluate_cases(&cases)?;

    let thresholds = load_thresholds()?;
    validate_thresholds(&thresholds)?;
    let threshold_outcome = evaluate_thresholds(&cases, &evaluations, &thresholds)?;

    let totals = EvalTotals {
        cases_passed: evaluations.iter().filter(|entry| entry.pass).count() as u32,
        cases_failed: evaluations.iter().filter(|entry| !entry.pass).count() as u32,
    };

    let report = EvalReport {
        head_commit: config.head_commit.clone(),
        run_timestamp_utc: config.run_timestamp_utc.clone(),
        corpus_pack_id: "run32_continuous_eval_v1".to_string(),
        thresholds_version: thresholds.thresholds_version,
        totals,
        per_metric: build_metric_summaries(&cases, &evaluations),
        per_case_results: evaluations,
        failing_case_ids: threshold_outcome.failing_case_ids.clone(),
        threshold_failures: threshold_outcome.failures.clone(),
        overall_pass: threshold_outcome.pass,
    };

    let output_path = write_report(
        &config.output_dir,
        config.date_tag.as_str(),
        config.head_commit.as_str(),
        &report,
    )?;

    Ok(ContinuousEvalOutcome {
        report_path: output_path,
        overall_pass: report.overall_pass,
        failing_case_ids: threshold_outcome.failing_case_ids,
    })
}

fn write_report(
    output_dir: &Path,
    date_tag: &str,
    head_commit: &str,
    report: &EvalReport,
) -> Result<PathBuf, String> {
    fs::create_dir_all(output_dir)
        .map_err(|e| format!("failed creating report directory {}: {}", output_dir.display(), e))?;

    let path = output_dir.join(format!(
        "EvalReport_{}_{}.json",
        date_tag,
        short_head(head_commit)
    ));
    let encoded = serde_json::to_string_pretty(report)
        .map_err(|e| format!("failed encoding eval report JSON: {}", e))?;
    fs::write(&path, encoded).map_err(|e| format!("failed writing {}: {}", path.display(), e))?;
    Ok(path)
}

fn build_metric_summaries(cases: &[EvalCase], evaluations: &[CaseEvaluation]) -> MetricSummaries {
    let answer_total = cases
        .iter()
        .filter(|case| case.expected_outcome == "answer")
        .count() as u32;
    let answer_passed = cases
        .iter()
        .zip(evaluations.iter())
        .filter(|(case, evaluation)| {
            case.expected_outcome == "answer"
                && (evaluation.metrics.citation_coverage_ratio - 1.0).abs() < f64::EPSILON
        })
        .count() as u32;

    let refusal_total = cases
        .iter()
        .filter(|case| case.expected_outcome == "refusal")
        .count() as u32;
    let refusal_passed = cases
        .iter()
        .zip(evaluations.iter())
        .filter(|(case, evaluation)| {
            case.expected_outcome == "refusal" && evaluation.metrics.refusal_correctness
        })
        .count() as u32;

    let freshness_total = cases
        .iter()
        .filter(|case| case.expect_stale_refusal)
        .count() as u32;
    let freshness_passed = cases
        .iter()
        .zip(evaluations.iter())
        .filter(|(case, evaluation)| case.expect_stale_refusal && evaluation.metrics.freshness_compliance)
        .count() as u32;

    let conflict_total = cases
        .iter()
        .filter(|case| case.expect_conflict_flag)
        .count() as u32;
    let conflict_passed = cases
        .iter()
        .zip(evaluations.iter())
        .filter(|(case, evaluation)| case.expect_conflict_flag && evaluation.metrics.conflict_handling)
        .count() as u32;

    let trust_total = cases
        .iter()
        .filter(|case| case.expect_trust_filtering)
        .count() as u32;
    let trust_passed = cases
        .iter()
        .zip(evaluations.iter())
        .filter(|(case, evaluation)| case.expect_trust_filtering && evaluation.metrics.trust_filtering)
        .count() as u32;

    let determinism_total = evaluations.len() as u32;
    let determinism_passed = evaluations
        .iter()
        .filter(|evaluation| evaluation.metrics.determinism_ok)
        .count() as u32;

    MetricSummaries {
        citation_coverage: MetricSummary {
            passed: answer_passed,
            total: answer_total,
            ratio: ratio(answer_passed, answer_total),
        },
        refusal_correctness: MetricSummary {
            passed: refusal_passed,
            total: refusal_total,
            ratio: ratio(refusal_passed, refusal_total),
        },
        freshness_compliance: MetricSummary {
            passed: freshness_passed,
            total: freshness_total,
            ratio: ratio(freshness_passed, freshness_total),
        },
        conflict_handling: MetricSummary {
            passed: conflict_passed,
            total: conflict_total,
            ratio: ratio(conflict_passed, conflict_total),
        },
        trust_filtering: MetricSummary {
            passed: trust_passed,
            total: trust_total,
            ratio: ratio(trust_passed, trust_total),
        },
        determinism_ok: MetricSummary {
            passed: determinism_passed,
            total: determinism_total,
            ratio: ratio(determinism_passed, determinism_total),
        },
    }
}

fn ratio(passed: u32, total: u32) -> f64 {
    if total == 0 {
        1.0
    } else {
        passed as f64 / total as f64
    }
}

fn short_head(head_commit: &str) -> &str {
    if head_commit.len() > 12 {
        &head_commit[..12]
    } else {
        head_commit
    }
}

pub fn expected_pack_files() -> &'static [&'static str] {
    DEFAULT_CORPUS_PACK_FILES
}
