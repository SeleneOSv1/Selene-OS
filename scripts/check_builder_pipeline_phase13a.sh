#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13a: $1" >&2
  exit 1
}

check_file() {
  local file="$1"
  [ -f "$file" ] || fail "missing file: $file"
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local msg="$3"
  rg -n "$pattern" "$file" >/dev/null || fail "$msg ($file)"
}

check_file "crates/selene_kernel_contracts/src/ph1builder.rs"
check_file "crates/selene_storage/migrations/0017_builder_selene_pipeline_tables.sql"

check_contains "crates/selene_kernel_contracts/src/lib.rs" "pub mod ph1builder;" "kernel contract lib must expose ph1builder"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderPatchProposal" "missing BuilderPatchProposal contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderValidationRun" "missing BuilderValidationRun contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderValidationGateResult" "missing BuilderValidationGateResult contract"

check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_proposal_ledger_row" "missing builder proposal append method"
check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_validation_run_ledger_row" "missing builder run append method"
check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_validation_gate_result_ledger_row" "missing builder result append method"
check_contains "crates/selene_storage/src/ph1f.rs" "builder_proposal_idempotency_index" "missing proposal idempotency index"
check_contains "crates/selene_storage/src/ph1f.rs" "builder_validation_run_idempotency_index" "missing run idempotency index"
check_contains "crates/selene_storage/src/repo.rs" "trait BuilderSeleneRepo" "missing builder repo trait"

check_contains "crates/selene_storage/migrations/0017_builder_selene_pipeline_tables.sql" "CREATE TABLE IF NOT EXISTS builder_patch_proposals" "missing builder_patch_proposals table"
check_contains "crates/selene_storage/migrations/0017_builder_selene_pipeline_tables.sql" "CREATE TABLE IF NOT EXISTS builder_validation_runs" "missing builder_validation_runs table"
check_contains "crates/selene_storage/migrations/0017_builder_selene_pipeline_tables.sql" "CREATE TABLE IF NOT EXISTS builder_validation_gate_results" "missing builder_validation_gate_results table"

echo "CHECK_OK builder_pipeline_phase13a=pass"
