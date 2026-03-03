#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::analytics::analytics_tests --quiet
cargo test -p selene_os web_search_plan::tests::test_computation_valid_fixture_passes --quiet
cargo test -p selene_os web_search_plan::tests::test_computation_invalid_fixture_fails --quiet
