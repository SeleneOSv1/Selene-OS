#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

LEARNING_ENV_FILE="${LEARNING_ENV_FILE:-.dev/builder_learning_bridge.env}"
PERMISSION_ENV_FILE="${PERMISSION_ENV_FILE:-.dev/builder_permission.env}"
BRIEF_FILE="${BRIEF_FILE:-.dev/builder_change_brief.md}"
CODE_DECISION_FILE="${CODE_DECISION_FILE:-.dev/builder_code_decision.env}"
LAUNCH_DECISION_FILE="${LAUNCH_DECISION_FILE:-.dev/builder_launch_decision.env}"
STAGE3_OUTPUT_CSV="${STAGE3_OUTPUT_CSV:-.dev/stage2_canary_metrics_snapshot.csv}"
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES:-180}"

AUTO_SYNC_DECISION_FILES=1 \
STAGE_GATE_MODE=live \
LEARNING_ENV_FILE="${LEARNING_ENV_FILE}" \
PERMISSION_ENV_FILE="${PERMISSION_ENV_FILE}" \
BRIEF_FILE="${BRIEF_FILE}" \
CODE_DECISION_FILE="${CODE_DECISION_FILE}" \
LAUNCH_DECISION_FILE="${LAUNCH_DECISION_FILE}" \
STAGE3_OUTPUT_CSV="${STAGE3_OUTPUT_CSV}" \
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES}" \
  bash scripts/check_builder_e2e_gate_flow.sh

echo "CHECK_OK builder_release_hard_gate=pass"
