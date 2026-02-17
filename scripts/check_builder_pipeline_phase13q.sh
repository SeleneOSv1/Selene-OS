#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail() {
  echo "CHECK_FAIL:$1" >&2
  exit 1
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local code="$3"
  if ! rg -n --fixed-strings "${pattern}" "${file}" >/dev/null 2>&1; then
    fail "${code}"
  fi
}

if [[ ! -x scripts/check_builder_controlled_launch_execute.sh ]]; then
  fail "missing_executable scripts/check_builder_controlled_launch_execute.sh"
fi
if [[ ! -x scripts/export_builder_stage2_canary_metrics.sh ]]; then
  fail "missing_executable scripts/export_builder_stage2_canary_metrics.sh"
fi

check_contains "scripts/check_builder_controlled_launch_execute.sh" "REQUIRE_STAGE_JUDGE" "missing_stage_judge_toggle"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "REQUIRED_RELEASE_STATE_ID" "missing_stage_bound_release_state_export"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "check_builder_stage2_promotion_gate.sh" "missing_stage_bound_promotion_gate"
check_contains "scripts/export_builder_stage2_canary_metrics.sh" "REQUIRED_RELEASE_STATE_ID" "missing_export_release_state_scope"
check_contains "scripts/export_builder_stage2_canary_metrics.sh" "REQUIRED_PROPOSAL_ID" "missing_export_proposal_scope"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13.29 Stage-Bound Judge Gate (Per-Stage Promotion Proof)" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.29"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "REQUIRE_STAGE_JUDGE=1" "missing_stage_judge_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"

echo "CHECK_OK builder_pipeline_phase13q=pass"
