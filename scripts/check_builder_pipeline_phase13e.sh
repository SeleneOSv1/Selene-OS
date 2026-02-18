#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13e: $1" >&2
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

check_file "crates/selene_kernel_contracts/src/ph1builder.rs"
check_file "crates/selene_os/src/ph1builder.rs"
check_file "crates/selene_storage/tests/ph1_f/db_wiring.rs"
check_file "crates/selene_storage/migrations/0020_builder_selene_learning_bridge_fields.sql"
check_file "scripts/check_builder_learning_bridge_gate.sh"
check_file "scripts/check_builder_e2e_gate_flow.sh"

check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderLearningContext" "missing learning context contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "learning_report_id" "missing learning_report_id contract field"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "source_engines" "missing source_engines contract field"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "learning_signal_count" "missing learning_signal_count contract field"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "evidence_refs" "missing evidence_refs contract field"

check_contains "crates/selene_os/src/ph1builder.rs" "maybe_generate_learning_auto_report" "missing learning auto-report generator"
check_contains "crates/selene_os/src/ph1builder.rs" "write_learning_report_to_path" "missing learning report writer"
check_contains "crates/selene_os/src/ph1builder.rs" "learning_report_output_path" "missing learning report output path wiring"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_12_learning_report_auto_generated_for_learning_sources" "missing learning auto-report test"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_13_learning_report_skipped_without_learning_sources" "missing learning no-source test"

check_contains "crates/selene_storage/tests/ph1_f/db_wiring.rs" "at_f_db_09_builder_learning_context_persists_in_proposal_rows" "missing storage learning context persistence test"
check_contains "crates/selene_storage/migrations/0020_builder_selene_learning_bridge_fields.sql" "learning_report_id" "missing learning_report_id migration column"

echo "CHECK_OK builder_pipeline_phase13e=pass"
