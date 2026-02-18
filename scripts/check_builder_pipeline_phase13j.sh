#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13j: $1" >&2
  exit 1
}

check_file() {
  local file="$1"
  [ -f "$file" ] || fail "missing file: $file"
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  rg -n "$pattern" "$file" >/dev/null || fail "$message ($file)"
}

check_file "crates/selene_os/src/ph1builder.rs"
check_file "scripts/check_builder_e2e_gate_flow.sh"

check_contains "crates/selene_os/src/ph1builder.rs" "BuilderDecisionSeedFiles" "missing decision seed struct"
check_contains "crates/selene_os/src/ph1builder.rs" "generate_decision_seed_files" "missing decision seed generator"
check_contains "crates/selene_os/src/ph1builder.rs" "write_decision_seed_file" "missing decision seed writer"
check_contains "crates/selene_os/src/ph1builder.rs" "builder_code_decision.env" "missing code decision seed filename"
check_contains "crates/selene_os/src/ph1builder.rs" "builder_launch_decision.env" "missing launch decision seed filename"
check_contains "crates/selene_os/src/ph1builder.rs" "code_decision_file_path" "missing completed bundle code decision file path"
check_contains "crates/selene_os/src/ph1builder.rs" "launch_decision_file_path" "missing completed bundle launch decision file path"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_15_permission_packet_auto_generated_for_bcast_flow" "missing packet/seed output test"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_builder_pipeline_phase13j.sh" "e2e chain missing phase13j precheck"

echo "CHECK_OK builder_pipeline_phase13j=pass"
