#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13g: $1" >&2
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
check_file "scripts/check_builder_human_permission_gate.sh"
check_file "scripts/check_builder_e2e_gate_flow.sh"

check_contains "crates/selene_os/src/ph1builder.rs" "DEFAULT_PERMISSION_PACKET_OUTPUT_PATH" "missing default permission packet output path"
check_contains "crates/selene_os/src/ph1builder.rs" "permission_packet_output_path" "missing permission packet input wiring"
check_contains "crates/selene_os/src/ph1builder.rs" "generate_permission_packet" "missing permission packet generator"
check_contains "crates/selene_os/src/ph1builder.rs" "render_permission_packet_markdown" "missing permission packet renderer"
check_contains "crates/selene_os/src/ph1builder.rs" "write_permission_packet_to_path" "missing permission packet writer"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_15_permission_packet_auto_generated_for_bcast_flow" "missing permission packet test"
check_contains "crates/selene_os/src/ph1builder.rs" "BCAST_CREATE_DRAFT" "missing bcast draft simulation mapping in packet output"
check_contains "crates/selene_os/src/ph1builder.rs" "BCAST_DELIVER_COMMIT" "missing bcast deliver simulation mapping in packet output"
check_contains "crates/selene_os/src/ph1builder.rs" "REMINDER_SCHEDULE_COMMIT" "missing reminder follow-up mapping in packet output"
check_contains "crates/selene_os/src/ph1builder.rs" "apply_builder_permission_decision.sh" "missing decision-ingest command mapping in packet output"
check_contains "crates/selene_os/src/ph1builder.rs" "builder_permission_decision_template.env" "missing decision-file template mapping in packet output"

check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_builder_pipeline_phase13g.sh" "e2e chain missing phase13g precheck"

echo "CHECK_OK builder_pipeline_phase13g=pass"
