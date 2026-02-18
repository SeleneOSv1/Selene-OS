#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

ENV_FILE="${ENV_FILE:-.dev/builder_learning_bridge.env}"

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "LEARNING_BRIDGE_BLOCK:missing_env_file file=${ENV_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${ENV_FILE}"

require_nonempty() {
  local value="${1:-}"
  local field="${2:-value}"
  if [[ -z "${value}" ]]; then
    echo "LEARNING_BRIDGE_BLOCK:missing_field field=${field}" >&2
    exit 1
  fi
}

require_bool01() {
  local value="${1:-}"
  local field="${2:-flag}"
  if [[ "${value}" != "0" && "${value}" != "1" ]]; then
    echo "LEARNING_BRIDGE_BLOCK:invalid_flag field=${field} expected=0_or_1 actual='${value}'" >&2
    exit 1
  fi
}

LEARNING_TRIGGERED="${LEARNING_TRIGGERED:-0}"
require_bool01 "${LEARNING_TRIGGERED}" "LEARNING_TRIGGERED"

if [[ "${LEARNING_TRIGGERED}" == "0" ]]; then
  echo "CHECK_OK builder_learning_bridge_gate=pass mode=not_triggered"
  exit 0
fi

require_bool01 "${LEARNING_REPORT_VALIDATED:-0}" "LEARNING_REPORT_VALIDATED"
if [[ "${LEARNING_REPORT_VALIDATED:-0}" != "1" ]]; then
  echo "LEARNING_BRIDGE_BLOCK:report_not_validated expected=1 actual='${LEARNING_REPORT_VALIDATED:-0}'" >&2
  exit 1
fi

require_nonempty "${LEARNING_REPORT_FILE:-}" "LEARNING_REPORT_FILE"
REPORT_FILE="${LEARNING_REPORT_FILE}"
if [[ ! -f "${REPORT_FILE}" ]]; then
  echo "LEARNING_BRIDGE_BLOCK:missing_report_file file=${REPORT_FILE}" >&2
  exit 1
fi

for heading in \
  "## Learning Issues Received" \
  "## Root Cause Evidence" \
  "## Deterministic Fix Plan" \
  "## Expected Improvement" \
  "## Builder Decision Prompt"; do
  if ! rg -q "^${heading}$" "${REPORT_FILE}"; then
    echo "LEARNING_BRIDGE_BLOCK:report_missing_heading heading='${heading}' file=${REPORT_FILE}" >&2
    exit 1
  fi
done

if ! rg -qi "should i proceed with this learning-driven fix\\?" "${REPORT_FILE}"; then
  echo "LEARNING_BRIDGE_BLOCK:report_missing_decision_prompt expected='Should I proceed with this learning-driven fix?'" >&2
  exit 1
fi

EVIDENCE_COUNT="$(rg -c "evidence_ref:" "${REPORT_FILE}" || echo 0)"
if [[ "${EVIDENCE_COUNT}" -lt 1 ]]; then
  echo "LEARNING_BRIDGE_BLOCK:missing_evidence_refs required_min=1 actual=${EVIDENCE_COUNT}" >&2
  exit 1
fi

require_nonempty "${LEARNING_REPORT_ID:-}" "LEARNING_REPORT_ID"
if ! [[ "${LEARNING_REPORT_ID}" =~ ^[A-Za-z0-9_.:-]{3,128}$ ]]; then
  echo "LEARNING_BRIDGE_BLOCK:invalid_report_id field=LEARNING_REPORT_ID actual='${LEARNING_REPORT_ID}'" >&2
  exit 1
fi

require_nonempty "${LEARNING_SOURCE_ENGINES:-}" "LEARNING_SOURCE_ENGINES"
if ! [[ "${LEARNING_SOURCE_ENGINES}" =~ ^[A-Za-z0-9._,-]+$ ]]; then
  echo "LEARNING_BRIDGE_BLOCK:invalid_source_engines field=LEARNING_SOURCE_ENGINES actual='${LEARNING_SOURCE_ENGINES}'" >&2
  exit 1
fi

SIGNAL_COUNT="${LEARNING_SIGNAL_COUNT:-0}"
if ! [[ "${SIGNAL_COUNT}" =~ ^[0-9]+$ ]]; then
  echo "LEARNING_BRIDGE_BLOCK:invalid_signal_count field=LEARNING_SIGNAL_COUNT actual='${SIGNAL_COUNT}'" >&2
  exit 1
fi
if [[ "${SIGNAL_COUNT}" -lt 1 ]]; then
  echo "LEARNING_BRIDGE_BLOCK:signal_count_too_low required_min=1 actual=${SIGNAL_COUNT}" >&2
  exit 1
fi

echo "CHECK_OK builder_learning_bridge_gate=pass report_id=${LEARNING_REPORT_ID} source_engines=${LEARNING_SOURCE_ENGINES} signal_count=${SIGNAL_COUNT}"
