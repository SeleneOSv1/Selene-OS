#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::vision::vision_tests --quiet
cargo test -p selene_os web_search_plan::tests::test_vision_tool_request_valid_fixture_passes --quiet
cargo test -p selene_os web_search_plan::tests::test_vision_evidence_valid_fixture_passes --quiet
cargo test -p selene_os web_search_plan::tests::test_vision_tool_request_invalid_fixture_fails --quiet
cargo test -p selene_os web_search_plan::tests::test_vision_evidence_invalid_fixture_fails --quiet
