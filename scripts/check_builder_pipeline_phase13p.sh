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

if [[ ! -x scripts/check_builder_controlled_launch_execute.sh ]]; then
  echo "CHECK_FAIL:missing_executable scripts/check_builder_controlled_launch_execute.sh" >&2
  exit 1
fi

check_contains "scripts/check_builder_controlled_launch_execute.sh" "EXECUTE.*0" "missing_preview_default"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "LAUNCH_EXECUTE_ACK" "missing_explicit_ack_guard"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "LAUNCH_EXECUTE_IDEMPOTENCY_KEY" "missing_idempotency_guard"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "check_builder_prelaunch_bundle\\.sh" "missing_prelaunch_bundle_dependency"
check_contains "scripts/check_builder_controlled_launch_execute.sh" "check_builder_human_permission_gate\\.sh launch" "missing_launch_permission_dependency"

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13\\.28 Controlled Launch Executor" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.28"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_controlled_launch_execute\\.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"

echo "CHECK_OK builder_pipeline_phase13p=pass"
