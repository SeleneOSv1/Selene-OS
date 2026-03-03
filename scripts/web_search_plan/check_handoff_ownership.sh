#!/usr/bin/env bash
set -euo pipefail

cargo test -p selene_os web_search_plan::tests::test_handoff_map_packet_refs_exist --quiet
cargo test -p selene_os web_search_plan::tests::test_ownership_matrix_engine_ids_are_well_formed --quiet
