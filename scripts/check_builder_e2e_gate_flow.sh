#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

LEARNING_ENV_FILE="${LEARNING_ENV_FILE:-.dev/builder_learning_bridge.env}"
PERMISSION_ENV_FILE="${PERMISSION_ENV_FILE:-.dev/builder_permission.env}"
BRIEF_FILE="${BRIEF_FILE:-.dev/builder_change_brief.md}"
STAGE_GATE_MODE="${STAGE_GATE_MODE:-fixture}"
STAGE2_FIXTURE_CSV="${STAGE2_FIXTURE_CSV:-docs/fixtures/stage2_canary_metrics_snapshot.csv}"
STAGE3_OUTPUT_CSV="${STAGE3_OUTPUT_CSV:-.dev/stage2_canary_metrics_snapshot.csv}"
MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES:-180}"
AUTO_SYNC_DECISION_FILES="${AUTO_SYNC_DECISION_FILES:-0}"
CODE_DECISION_FILE="${CODE_DECISION_FILE:-.dev/builder_code_decision.env}"
LAUNCH_DECISION_FILE="${LAUNCH_DECISION_FILE:-.dev/builder_launch_decision.env}"

if [[ "${STAGE_GATE_MODE}" != "fixture" && "${STAGE_GATE_MODE}" != "live" ]]; then
  echo "E2E_GATE_FAIL:invalid_stage_gate_mode expected=fixture_or_live actual=${STAGE_GATE_MODE}" >&2
  exit 1
fi
if [[ "${AUTO_SYNC_DECISION_FILES}" != "0" && "${AUTO_SYNC_DECISION_FILES}" != "1" ]]; then
  echo "E2E_GATE_FAIL:invalid_auto_sync_decision_files expected=0_or_1 actual=${AUTO_SYNC_DECISION_FILES}" >&2
  exit 1
fi
if ! [[ "${MAX_TELEMETRY_AGE_MINUTES}" =~ ^[0-9]+$ ]] || [[ "${MAX_TELEMETRY_AGE_MINUTES}" -lt 1 ]]; then
  echo "E2E_GATE_FAIL:invalid_max_telemetry_age_minutes expected=positive_integer actual=${MAX_TELEMETRY_AGE_MINUTES}" >&2
  exit 1
fi

bash scripts/check_builder_pipeline_phase13e.sh
bash scripts/check_builder_pipeline_phase13f.sh
bash scripts/check_builder_pipeline_phase13g.sh
bash scripts/check_builder_pipeline_phase13h.sh
bash scripts/check_builder_pipeline_phase13i.sh
bash scripts/check_builder_pipeline_phase13j.sh
bash scripts/check_builder_pipeline_phase13k.sh
ENV_FILE="${LEARNING_ENV_FILE}" bash scripts/check_builder_learning_bridge_gate.sh
if [[ "${AUTO_SYNC_DECISION_FILES}" == "1" ]]; then
  ENV_FILE="${PERMISSION_ENV_FILE}" \
  CODE_DECISION_FILE="${CODE_DECISION_FILE}" \
  LAUNCH_DECISION_FILE="${LAUNCH_DECISION_FILE}" \
    bash scripts/sync_builder_permission_from_decision_files.sh
fi
ENV_FILE="${PERMISSION_ENV_FILE}" BRIEF_FILE="${BRIEF_FILE}" bash scripts/check_builder_human_permission_gate.sh code
ENV_FILE="${PERMISSION_ENV_FILE}" BRIEF_FILE="${BRIEF_FILE}" bash scripts/check_builder_human_permission_gate.sh launch

if [[ "${STAGE_GATE_MODE}" == "fixture" ]]; then
  bash scripts/check_builder_stage2_promotion_gate.sh "${STAGE2_FIXTURE_CSV}"
else
  MAX_TELEMETRY_AGE_MINUTES="${MAX_TELEMETRY_AGE_MINUTES}" \
    bash scripts/check_builder_stage3_release_gate.sh "${STAGE3_OUTPUT_CSV}"
fi

echo "CHECK_OK builder_e2e_gate_flow=pass stage_gate_mode=${STAGE_GATE_MODE}"
