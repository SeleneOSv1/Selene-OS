#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PHASE="${1:-}"
ENV_FILE="${ENV_FILE:-.dev/builder_permission.env}"
BRIEF_FILE="${BRIEF_FILE:-.dev/builder_change_brief.md}"

if [[ "${PHASE}" != "code" && "${PHASE}" != "launch" ]]; then
  echo "USAGE: $0 <code|launch>" >&2
  exit 2
fi

if [[ ! -f "${BRIEF_FILE}" ]]; then
  echo "PERMISSION_BLOCK:missing_brief_file file=${BRIEF_FILE}" >&2
  exit 1
fi

for heading in "## Issue" "## Fix" "## Should I Proceed" "## Launch Question"; do
  if ! rg -q "^${heading}$" "${BRIEF_FILE}"; then
    echo "PERMISSION_BLOCK:brief_missing_heading heading='${heading}' file=${BRIEF_FILE}" >&2
    exit 1
  fi
done

if ! rg -qi "should i proceed\\?" "${BRIEF_FILE}"; then
  echo "PERMISSION_BLOCK:brief_missing_proceed_question expected='Should I proceed?'" >&2
  exit 1
fi

if ! rg -qi "all tests passed[.]? can i launch\\?" "${BRIEF_FILE}"; then
  echo "PERMISSION_BLOCK:brief_missing_launch_question expected='All tests passed. Can I launch?'" >&2
  exit 1
fi

if [[ ! -f "${ENV_FILE}" ]]; then
  echo "PERMISSION_BLOCK:missing_permission_env file=${ENV_FILE}" >&2
  exit 1
fi

# shellcheck disable=SC1090
source "${ENV_FILE}"

BUSY_MODE="${BUSY_MODE:-0}"
TODAY_UTC="$(date -u +%Y-%m-%d)"

require_nonempty() {
  local v="${1:-}"
  local label="${2:-value}"
  if [[ -z "${v}" ]]; then
    echo "PERMISSION_BLOCK:missing_field field=${label}" >&2
    exit 1
  fi
}

require_bool01() {
  local v="${1:-}"
  local label="${2:-flag}"
  if [[ "${v}" != "0" && "${v}" != "1" ]]; then
    echo "PERMISSION_BLOCK:invalid_flag field=${label} expected=0_or_1 actual='${v}'" >&2
    exit 1
  fi
}

require_bool01 "${BUSY_MODE}" "BUSY_MODE"
require_bool01 "${DAILY_REVIEW_OK:-0}" "DAILY_REVIEW_OK"
if [[ "${DAILY_REVIEW_OK:-0}" != "1" ]]; then
  echo "PERMISSION_BLOCK:daily_review_not_confirmed expected=1 actual='${DAILY_REVIEW_OK:-0}'" >&2
  exit 1
fi

require_nonempty "${DAILY_REVIEW_DATE_UTC:-}" "DAILY_REVIEW_DATE_UTC"
if [[ "${DAILY_REVIEW_DATE_UTC}" != "${TODAY_UTC}" ]]; then
  echo "PERMISSION_BLOCK:daily_review_stale expected_date_utc=${TODAY_UTC} actual_date_utc=${DAILY_REVIEW_DATE_UTC}" >&2
  exit 1
fi

check_pending_reminder_policy() {
  local approved_flag="${1}"
  local reminder_flag="${2}"
  local reminder_ref="${3}"
  local phase_name="${4}"

  require_bool01 "${approved_flag}" "${phase_name}_APPROVED"
  require_bool01 "${reminder_flag}" "${phase_name}_REMINDER_SCHEDULED"

  if [[ "${approved_flag}" == "1" ]]; then
    return 0
  fi

  if [[ "${BUSY_MODE}" == "1" ]]; then
    if [[ "${reminder_flag}" != "1" ]]; then
      echo "PERMISSION_BLOCK:${phase_name}_approval_pending_no_reminder busy_mode=1" >&2
      exit 1
    fi
    require_nonempty "${reminder_ref}" "${phase_name}_REMINDER_REF"
    echo "PERMISSION_BLOCK:${phase_name}_approval_pending_reminder_scheduled reminder_ref=${reminder_ref}" >&2
    exit 1
  fi

  echo "PERMISSION_BLOCK:${phase_name}_approval_pending busy_mode=0" >&2
  exit 1
}

if [[ "${PHASE}" == "code" ]]; then
  check_pending_reminder_policy \
    "${CODE_APPROVED:-0}" \
    "${CODE_REMINDER_SCHEDULED:-0}" \
    "${CODE_REMINDER_REF:-}" \
    "code"

  require_nonempty "${CODE_APPROVAL_BCAST_ID:-}" "CODE_APPROVAL_BCAST_ID"
  require_nonempty "${CODE_APPROVAL_DECISION_REF:-}" "CODE_APPROVAL_DECISION_REF"
  echo "CHECK_OK builder_human_permission_gate=pass phase=code"
  exit 0
fi

check_pending_reminder_policy \
  "${LAUNCH_APPROVED:-0}" \
  "${LAUNCH_REMINDER_SCHEDULED:-0}" \
  "${LAUNCH_REMINDER_REF:-}" \
  "launch"

require_nonempty "${LAUNCH_APPROVAL_BCAST_ID:-}" "LAUNCH_APPROVAL_BCAST_ID"
require_nonempty "${LAUNCH_APPROVAL_DECISION_REF:-}" "LAUNCH_APPROVAL_DECISION_REF"
echo "CHECK_OK builder_human_permission_gate=pass phase=launch"
