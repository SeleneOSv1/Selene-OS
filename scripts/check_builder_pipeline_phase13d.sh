#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13d: $1" >&2
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
check_file "crates/selene_storage/src/ph1f.rs"
check_file "crates/selene_storage/src/repo.rs"
check_file "crates/selene_storage/migrations/0019_builder_selene_post_deploy_judge_tables.sql"

check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderMetricsSnapshot" "missing metrics snapshot contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "struct BuilderPostDeployJudgeResult" "missing post-deploy judge result contract"
check_contains "crates/selene_kernel_contracts/src/ph1builder.rs" "enum BuilderPostDeployDecisionAction" "missing post-deploy decision action enum"

check_contains "crates/selene_storage/src/ph1f.rs" "append_builder_post_deploy_judge_result_ledger_row" "missing judge append method"
check_contains "crates/selene_storage/src/ph1f.rs" "builder_post_deploy_judge_result_idempotency_index" "missing judge idempotency index"
check_contains "crates/selene_storage/src/repo.rs" "append_builder_post_deploy_judge_result_row" "missing judge repo method"

check_contains "crates/selene_storage/migrations/0019_builder_selene_post_deploy_judge_tables.sql" "CREATE TABLE IF NOT EXISTS builder_post_deploy_judge_results" "missing judge results table"

check_contains "crates/selene_os/src/ph1builder.rs" "run_post_deploy_judge" "missing post-deploy judge runtime entrypoint"
check_contains "crates/selene_os/src/ph1builder.rs" "validate_gate_results_complete" "missing missing-gate-outcomes fail-closed check"
check_contains "crates/selene_os/src/ph1builder.rs" "PH1_BUILDER_POST_DEPLOY_MISSING_PROPOSAL_FIELDS" "missing missing-proposal-fields fail-closed reason"
check_contains "crates/selene_os/src/ph1builder.rs" "should_trigger_post_deploy_rollback" "missing auto accept/revert comparison logic"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach" "missing post-deploy revert test"
check_contains "crates/selene_os/src/ph1builder.rs" "at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes" "missing missing-gate-outcomes test"

echo "CHECK_OK builder_pipeline_phase13d=pass"
