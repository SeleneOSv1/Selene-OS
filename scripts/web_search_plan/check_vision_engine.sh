#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::vision::vision_tests::test_vision_packet_fixtures_validate --quiet
cargo test -p selene_os web_search_plan::vision::vision_tests --quiet
