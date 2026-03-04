#![forbid(unsafe_code)]

use crate::web_search_plan::eval::corpus_packs::{load_default_corpus_packs, merge_cases};
use crate::web_search_plan::eval::metrics::{evaluate_case, evaluate_cases};
use crate::web_search_plan::eval::report::{generate_eval_report, ContinuousEvalConfig};
use crate::web_search_plan::eval::thresholds::{evaluate_thresholds, load_thresholds, validate_thresholds};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn temp_report_dir() -> PathBuf {
    std::env::temp_dir().join("selene_web_search_eval_tests")
}

#[test]
fn test_t1_corpus_packs_load_deterministically() {
    let first = load_default_corpus_packs().expect("default corpus packs should load");
    let second = load_default_corpus_packs().expect("default corpus packs should load on replay");
    assert_eq!(first, second, "corpus pack loading must be deterministic");
}

#[test]
fn test_t2_metric_outputs_deterministic() {
    let packs = load_default_corpus_packs().expect("default corpus packs should load");
    let cases = merge_cases(&packs).expect("cases should merge");
    let first = evaluate_cases(&cases).expect("first metric run should pass");
    let second = evaluate_cases(&cases).expect("second metric run should pass");
    assert_eq!(first, second, "metric evaluation must be deterministic");
}

#[test]
fn test_t3_threshold_breach_fails() {
    let packs = load_default_corpus_packs().expect("default corpus packs should load");
    let cases = merge_cases(&packs).expect("cases should merge");
    let mut evaluations = evaluate_cases(&cases).expect("metrics should evaluate");
    let mut thresholds = load_thresholds().expect("thresholds should load");
    validate_thresholds(&thresholds).expect("thresholds should validate");

    let first_answer_index = cases
        .iter()
        .position(|case| case.expected_outcome == "answer")
        .expect("at least one answer case is required");
    evaluations[first_answer_index].metrics.citation_coverage_ratio = 0.0;
    evaluations[first_answer_index].pass = false;
    evaluations[first_answer_index]
        .reasons
        .push("forced threshold breach".to_string());

    thresholds.max_allowed_regressions = 0;
    let outcome =
        evaluate_thresholds(&cases, &evaluations, &thresholds).expect("threshold evaluation should run");
    assert!(!outcome.pass, "threshold outcome must fail on breach");
    assert!(
        !outcome.failing_case_ids.is_empty(),
        "failed threshold should return failing case ids"
    );
}

#[test]
fn test_t4_report_json_stable_except_timestamp() {
    let output_dir = temp_report_dir();
    fs::create_dir_all(&output_dir).expect("output dir should exist");

    let first = generate_eval_report(&ContinuousEvalConfig {
        head_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        run_timestamp_utc: "2026-03-04T00:00:00Z".to_string(),
        date_tag: "20260304T000000Z".to_string(),
        output_dir: output_dir.clone(),
    })
    .expect("first report should generate");
    let second = generate_eval_report(&ContinuousEvalConfig {
        head_commit: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
        run_timestamp_utc: "2026-03-04T00:10:00Z".to_string(),
        date_tag: "20260304T001000Z".to_string(),
        output_dir: output_dir.clone(),
    })
    .expect("second report should generate");

    let first_text =
        fs::read_to_string(first.report_path).expect("first report file should be readable");
    let second_text =
        fs::read_to_string(second.report_path).expect("second report file should be readable");
    let mut first_json: Value = serde_json::from_str(&first_text).expect("first report json");
    let mut second_json: Value = serde_json::from_str(&second_text).expect("second report json");
    first_json
        .as_object_mut()
        .expect("first report must be object")
        .remove("run_timestamp_utc");
    second_json
        .as_object_mut()
        .expect("second report must be object")
        .remove("run_timestamp_utc");

    assert_eq!(
        first_json, second_json,
        "report payload should be stable except timestamp"
    );
}

#[test]
fn test_t5_snapshot_hash_match_enforced() {
    let packs = load_default_corpus_packs().expect("default corpus packs should load");
    let mut cases = merge_cases(&packs).expect("cases should merge");
    let first = cases.first_mut().expect("at least one case required");
    first.expected_snapshot_hash =
        "0000000000000000000000000000000000000000000000000000000000000000".to_string();

    let evaluation = evaluate_case(first).expect("case evaluation should run");
    assert!(!evaluation.pass, "snapshot mismatch must fail case");
    assert!(
        evaluation
            .reasons
            .iter()
            .any(|reason| reason.contains("determinism mismatch")),
        "snapshot mismatch should be reported in reasons"
    );
}
