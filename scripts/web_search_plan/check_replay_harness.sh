#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::replay::replay_tests::test_t1_corpus_loads_and_validates --quiet
cargo test -p selene_os web_search_plan::replay::replay_tests::test_t2_fixture_evidence_packet_validates_against_schema --quiet
cargo test -p selene_os web_search_plan::replay::replay_tests --quiet
