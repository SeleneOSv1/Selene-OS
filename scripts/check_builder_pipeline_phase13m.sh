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

if [[ ! -x scripts/check_builder_controlled_rollout_start.sh ]]; then
  echo "CHECK_FAIL:missing_executable scripts/check_builder_controlled_rollout_start.sh" >&2
  exit 1
fi

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13\\.25 Controlled Rollout Start Command" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.25"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_controlled_rollout_start\\.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "MAX_TELEMETRY_AGE_MINUTES.*180" "missing_freshness_default_180 docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "scripts/check_builder_release_hard_gate.sh" "MAX_TELEMETRY_AGE_MINUTES.*180" "missing_hard_gate_freshness_default_180"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "MAX_TELEMETRY_AGE_MINUTES.*180" "missing_e2e_freshness_default_180"
check_contains "scripts/export_builder_stage2_canary_metrics.sh" "MAX_TELEMETRY_AGE_MINUTES.*180" "missing_export_freshness_default_180"

echo "CHECK_OK builder_pipeline_phase13m=pass"
