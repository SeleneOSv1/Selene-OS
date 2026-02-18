#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ENV_FILE="${ENV_FILE:-.dev/builder_permission.env}"
CODE_DECISION_FILE="${CODE_DECISION_FILE:-.dev/builder_code_decision.env}"
LAUNCH_DECISION_FILE="${LAUNCH_DECISION_FILE:-.dev/builder_launch_decision.env}"
APPLY_CODE="${APPLY_CODE:-1}"
APPLY_LAUNCH="${APPLY_LAUNCH:-1}"
REQUIRE_FILES="${REQUIRE_FILES:-1}"
REFRESH_DAILY_REVIEW="${REFRESH_DAILY_REVIEW:-1}"

fail() {
  echo "PERMISSION_SYNC_FAIL:$1" >&2
  exit 1
}

validate_flag01() {
  local field="${1}"
  local value="${2}"
  if [[ "${value}" != "0" && "${value}" != "1" ]]; then
    fail "invalid_flag field=${field} expected=0_or_1 actual=${value}"
  fi
}

validate_flag01 "APPLY_CODE" "${APPLY_CODE}"
validate_flag01 "APPLY_LAUNCH" "${APPLY_LAUNCH}"
validate_flag01 "REQUIRE_FILES" "${REQUIRE_FILES}"
validate_flag01 "REFRESH_DAILY_REVIEW" "${REFRESH_DAILY_REVIEW}"

apply_decision_file() {
  local phase="${1}"
  local file_path="${2}"

  if [[ ! -f "${file_path}" ]]; then
    if [[ "${REQUIRE_FILES}" == "1" ]]; then
      fail "missing_decision_file phase=${phase} file=${file_path}"
    fi
    echo "SYNC_SKIP builder_permission_decision phase=${phase} reason=file_missing file=${file_path}"
    return 0
  fi

  DECISION_FILE="${file_path}" \
  ENV_FILE="${ENV_FILE}" \
  REFRESH_DAILY_REVIEW="${REFRESH_DAILY_REVIEW}" \
    bash scripts/apply_builder_permission_decision.sh >/dev/null
  echo "SYNC_APPLY builder_permission_decision phase=${phase} file=${file_path}"
}

if [[ "${APPLY_CODE}" == "1" ]]; then
  apply_decision_file "code" "${CODE_DECISION_FILE}"
fi

if [[ "${APPLY_LAUNCH}" == "1" ]]; then
  apply_decision_file "launch" "${LAUNCH_DECISION_FILE}"
fi

echo "SYNC_OK builder_permission_decision_files env_file=${ENV_FILE}"
