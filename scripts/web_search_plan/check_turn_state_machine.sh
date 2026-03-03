#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::tests::test_turn_state_machine_valid_path --quiet
cargo test -p selene_os web_search_plan::tests::test_turn_state_machine_fail_closed_requires_reason --quiet
