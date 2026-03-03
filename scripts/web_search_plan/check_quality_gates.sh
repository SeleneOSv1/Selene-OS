#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::replay::replay_tests::test_t5_citation_coverage_gate_enforced_for_answer_cases --quiet
cargo test -p selene_os web_search_plan::replay::replay_tests::test_t6_refusal_correctness_enforced_for_refusal_cases --quiet
cargo test -p selene_os web_search_plan::replay::replay_tests::test_t7_regression_thresholds_enforced --quiet
