#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13c: $1" >&2
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
check_file "crates/selene_storage/migrations/0018_builder_selene_approval_release_tables.sql"
check_file "crates/selene_os/src/ph1builder.rs"

check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderApprovalState" "missing BuilderApprovalState contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderReleaseState" "missing BuilderReleaseState contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "enum BuilderReleaseStage" "missing BuilderReleaseStage enum"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "required_approvals_for_change_class" "missing class-based approval helper"

check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_approval_state_ledger_row" "missing approval-state append method"
check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_release_state_ledger_row" "missing release-state append method"
check_contains "crates/selene_storage/src/ph1f.rs" "builder_approval_state_idempotency_index" "missing approval-state idempotency index"
check_contains "crates/selene_storage/src/ph1f.rs" "builder_release_state_idempotency_index" "missing release-state idempotency index"
check_contains "crates/selene_storage/src/repo.rs" "append_builder_approval_state_row" "missing approval repo method"
check_contains "crates/selene_storage/src/repo.rs" "append_builder_release_state_row" "missing release repo method"

check_contains "crates/selene_storage/migrations/0018_builder_selene_approval_release_tables.sql" "CREATE TABLE IF NOT EXISTS builder_approval_states" "missing builder_approval_states table"
check_contains "crates/selene_storage/migrations/0018_builder_selene_approval_release_tables.sql" "CREATE TABLE IF NOT EXISTS builder_release_states" "missing builder_release_states table"

check_contains "crates/selene_os/src/ph1builder.rs" "struct BuilderReleaseController" "missing release controller"
check_contains "crates/selene_os/src/ph1builder.rs" "advance_approval_state" "missing approval state machine transition"
check_contains "crates/selene_os/src/ph1builder.rs" "production rollout blocked because approval class is unresolved" "missing production block guard"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_04_class_b_requires_pending_approval_and_blocks_release" "missing stage13c approval/release test"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_05_release_controller_blocks_production_without_resolved_approval" "missing stage13c production block test"

echo "CHECK_OK builder_pipeline_phase13c=pass"
