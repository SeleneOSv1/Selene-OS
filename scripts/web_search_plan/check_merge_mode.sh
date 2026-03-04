#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::merge::merge_tests --quiet
cargo test -p selene_os web_search_plan::tests::test_merge_valid_fixture_passes --quiet
cargo test -p selene_os web_search_plan::tests::test_merge_invalid_fixture_fails --quiet
