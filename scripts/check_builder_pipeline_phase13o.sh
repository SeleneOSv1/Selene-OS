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

if [[ ! -x scripts/check_builder_prelaunch_bundle.sh ]]; then
  echo "CHECK_FAIL:missing_executable scripts/check_builder_prelaunch_bundle.sh" >&2
  exit 1
fi

check_contains "scripts/check_builder_prelaunch_bundle.sh" "check_builder_controlled_rollout_start\\.sh" "missing_rollout_start_call"
check_contains "scripts/check_builder_prelaunch_bundle.sh" "check_builder_rollback_drill\\.sh" "missing_rollback_drill_call"
check_contains "scripts/check_builder_prelaunch_bundle.sh" "check_builder_release_hard_gate\\.sh" "missing_hard_gate_call"

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13\\.27 Pre-Launch Bundle Command" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.27"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_prelaunch_bundle\\.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"

echo "CHECK_OK builder_pipeline_phase13o=pass"
