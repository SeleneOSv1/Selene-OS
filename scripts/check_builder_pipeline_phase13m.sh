#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

if [[ ! -x scripts/check_builder_controlled_rollout_start.sh ]]; then
  echo "CHECK_FAIL:missing_executable scripts/check_builder_controlled_rollout_start.sh" >&2
  exit 1
fi

if ! rg -q "### 13\\.25 Controlled Rollout Start Command" docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md; then
  echo "CHECK_FAIL:missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.25" >&2
  exit 1
fi

if ! rg -q "check_builder_controlled_rollout_start\\.sh" docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md; then
  echo "CHECK_FAIL:missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" >&2
  exit 1
fi

echo "CHECK_OK builder_pipeline_phase13m=pass"
