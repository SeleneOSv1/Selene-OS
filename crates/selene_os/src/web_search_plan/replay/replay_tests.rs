#![forbid(unsafe_code)]

use crate::web_search_plan::packet_validator::validate_packet;
use crate::web_search_plan::registry_loader::load_packet_schema_registry;
use crate::web_search_plan::replay::corpus::{load_replay_corpus, validate_replay_corpus};
use crate::web_search_plan::replay::regressions::{evaluate_regressions, load_replay_expected};
use crate::web_search_plan::replay::runner::{load_fixture_case, run_replay_corpus, run_replay_with_regression_gate};

#[test]
fn test_t1_corpus_loads_and_validates() {
    let corpus = load_replay_corpus().expect("replay corpus should load");
    validate_replay_corpus(&corpus).expect("replay corpus should validate");
    assert!(!corpus.cases.is_empty(), "corpus should include cases");
}

#[test]
fn test_t2_fixture_evidence_packet_validates_against_schema() {
    let packet_registry = load_packet_schema_registry().expect("packet registry should load");
    let corpus = load_replay_corpus().expect("replay corpus should load");

    for case in corpus.cases {
        let fixture = load_fixture_case(case.case_id.as_str())
            .unwrap_or_else(|e| panic!("fixture for case {} should load: {}", case.case_id, e));
        validate_packet("EvidencePacket", &fixture.evidence_packet, &packet_registry)
            .unwrap_or_else(|e| panic!("evidence packet for case {} invalid: {}", case.case_id, e));
    }
}

#[test]
fn test_t3_snapshot_hashes_stable_across_runs() {
    let run_a = run_replay_corpus().expect("first replay run should pass");
    let run_b = run_replay_corpus().expect("second replay run should pass");
    assert_eq!(run_a, run_b, "replay runs should be deterministic");
}

#[test]
fn test_t4_expected_snapshot_mismatch_fails() {
    let results = run_replay_corpus().expect("replay run should pass");
    let mut expected = load_replay_expected().expect("expected replay file should load");

    let first = expected
        .cases
        .first_mut()
        .expect("expected replay cases must not be empty");
    first.snapshot.evidence_hash = "0000000000000000000000000000000000000000000000000000000000000000".to_string();

    let err = evaluate_regressions(&results, &expected)
        .expect_err("tampered expected snapshot must fail regression gate");
    assert!(err.contains("snapshot regression"));
}

#[test]
fn test_t5_citation_coverage_gate_enforced_for_answer_cases() {
    let results = run_replay_with_regression_gate().expect("replay regression gate should pass");

    let corpus = load_replay_corpus().expect("replay corpus should load");
    for case in corpus.cases {
        if case.expected_outcome == "answer" {
            let result = results
                .iter()
                .find(|entry| entry.case_id == case.case_id)
                .unwrap_or_else(|| panic!("missing result for case {}", case.case_id));
            assert!(
                (result.metrics.citation_coverage_ratio - 1.0).abs() < f64::EPSILON,
                "answer case {} citation coverage must be 1.0",
                case.case_id
            );
        }
    }
}

#[test]
fn test_t6_refusal_correctness_enforced_for_refusal_cases() {
    let results = run_replay_with_regression_gate().expect("replay regression gate should pass");

    let corpus = load_replay_corpus().expect("replay corpus should load");
    for case in corpus.cases {
        if case.expected_outcome == "refusal" {
            let result = results
                .iter()
                .find(|entry| entry.case_id == case.case_id)
                .unwrap_or_else(|| panic!("missing result for case {}", case.case_id));
            assert!(
                result.metrics.refusal_correctness,
                "refusal correctness must hold for case {}",
                case.case_id
            );
        }
    }
}

#[test]
fn test_t7_regression_thresholds_enforced() {
    run_replay_with_regression_gate().expect("replay regression thresholds should hold");
}
