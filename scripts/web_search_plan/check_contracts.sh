#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::tests::test_valid_fixtures_pass --quiet
cargo test -p selene_os web_search_plan::tests::test_invalid_fixtures_fail --quiet
cargo test -p selene_os web_search_plan::tests::test_registered_packets_have_required_fixture_pairs --quiet
cargo test -p selene_os web_search_plan::tests::test_fixture_files_map_to_registered_packets --quiet
cargo test -p selene_os web_search_plan::tests::test_contract_hash_manifest_matches --quiet
