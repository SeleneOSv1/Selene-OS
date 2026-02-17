#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

check_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  rg -n "$pattern" "$file" >/dev/null || {
    echo "CHECK_FAIL:${message} (${file})" >&2
    exit 1
  }
}

if [[ ! -x scripts/check_builder_rollback_drill.sh ]]; then
  echo "CHECK_FAIL:missing_executable scripts/check_builder_rollback_drill.sh" >&2
  exit 1
fi

check_contains "scripts/check_builder_rollback_drill.sh" "at_builder_os_07_post_deploy_judge_reverts_on_latency_threshold_breach" "missing_revert_threshold_dryrun_test"
check_contains "scripts/check_builder_rollback_drill.sh" "at_builder_os_08_post_deploy_judge_refuses_missing_gate_outcomes" "missing_missing_gate_outcomes_dryrun_test"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13\\.26 Controlled Rollback Drill" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.26"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_rollback_drill\\.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"

echo "CHECK_OK builder_pipeline_phase13n=pass"
