#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PHASE="${1:-}"
DECISION="${2:-}"

ENV_FILE="${ENV_FILE:-.dev/builder_permission.env}"
DECISION_FILE="${DECISION_FILE:-}"
BCAST_ID="${BCAST_ID:-}"
DECISION_REF="${DECISION_REF:-}"
REMINDER_REF="${REMINDER_REF:-}"
REFRESH_DAILY_REVIEW="${REFRESH_DAILY_REVIEW:-1}"
BUSY_MODE_OVERRIDE="${BUSY_MODE_OVERRIDE:-}"

usage() {
  cat <<'USAGE'
USAGE:
  BCAST_ID=<id> DECISION_REF=<ref> [ENV_FILE=.dev/builder_permission.env] \
    bash scripts/apply_builder_permission_decision.sh <code|launch> <approve|deny|pending>
  DECISION_FILE=.dev/builder_code_decision.env [ENV_FILE=.dev/builder_permission.env] \
    bash scripts/apply_builder_permission_decision.sh

OPTIONAL:
  DECISION_FILE=<path>         # key/value decision file (phase/decision + refs)
  REMINDER_REF=<ref>           # used for pending busy follow-up
  REFRESH_DAILY_REVIEW=0|1     # default 1; set daily review fields to today UTC
  BUSY_MODE_OVERRIDE=0|1       # optional explicit BUSY_MODE update
USAGE
}

fail() {
  echo "PERMISSION_DECISION_APPLY_FAIL:$1" >&2
  exit 1
}

if [[ ! -f "${ENV_FILE}" ]]; then
  cp docs/fixtures/builder_permission_template.env "${ENV_FILE}"
fi

validate_flag01() {
  local field="${1}"
  local value="${2}"
  if [[ "${value}" != "0" && "${value}" != "1" ]]; then
    fail "invalid_flag field=${field} expected=0_or_1 actual=${value}"
  fi
}

validate_token_ascii() {
  local field="${1}"
  local value="${2}"
  if [[ -z "${value}" ]]; then
    fail "missing_required field=${field}"
  fi
  if [[ ! "${value}" =~ ^[A-Za-z0-9._:/-]{1,128}$ ]]; then
    fail "invalid_token field=${field} expected_ascii_safe_token actual=${value}"
  fi
}

trim_ascii_ws() {
  local v="${1:-}"
  # shellcheck disable=SC2001
  v="$(echo "${v}" | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')"
  printf '%s' "${v}"
}

set_kv() {
  local key="${1}"
  local value="${2}"
  if rg -q "^${key}=" "${ENV_FILE}"; then
    local escaped="${value//\\/\\\\}"
    escaped="${escaped//&/\\&}"
    sed -i.bak "s|^${key}=.*|${key}=${escaped}|" "${ENV_FILE}"
  else
    printf '%s=%s\n' "${key}" "${value}" >> "${ENV_FILE}"
  fi
}

FILE_PHASE=""
FILE_DECISION=""
FILE_BCAST_ID=""
FILE_DECISION_REF=""
FILE_REMINDER_REF=""
FILE_BUSY_MODE_OVERRIDE=""
FILE_REFRESH_DAILY_REVIEW=""

load_decision_file() {
  local file_path="${1}"
  [[ -f "${file_path}" ]] || fail "decision_file_missing file=${file_path}"

  while IFS='=' read -r raw_key raw_value; do
    local key
    local value
    key="$(trim_ascii_ws "${raw_key}")"
    value="$(trim_ascii_ws "${raw_value}")"
    value="${value%$'\r'}"
    if [[ -z "${key}" || "${key}" == \#* ]]; then
      continue
    fi
    case "${key}" in
      PHASE) FILE_PHASE="${value}" ;;
      DECISION) FILE_DECISION="${value}" ;;
      BCAST_ID) FILE_BCAST_ID="${value}" ;;
      DECISION_REF) FILE_DECISION_REF="${value}" ;;
      REMINDER_REF) FILE_REMINDER_REF="${value}" ;;
      BUSY_MODE_OVERRIDE) FILE_BUSY_MODE_OVERRIDE="${value}" ;;
      REFRESH_DAILY_REVIEW) FILE_REFRESH_DAILY_REVIEW="${value}" ;;
      *) fail "decision_file_unknown_key key=${key} file=${file_path}" ;;
    esac
  done < "${file_path}"
}

if [[ -n "${DECISION_FILE}" ]]; then
  load_decision_file "${DECISION_FILE}"
fi

if [[ -n "${PHASE}" && -n "${FILE_PHASE}" && "${PHASE}" != "${FILE_PHASE}" ]]; then
  fail "phase_conflict arg=${PHASE} file=${FILE_PHASE}"
fi
if [[ -n "${DECISION}" && -n "${FILE_DECISION}" && "${DECISION}" != "${FILE_DECISION}" ]]; then
  fail "decision_conflict arg=${DECISION} file=${FILE_DECISION}"
fi

if [[ -z "${PHASE}" ]]; then
  PHASE="${FILE_PHASE}"
fi
if [[ -z "${DECISION}" ]]; then
  DECISION="${FILE_DECISION}"
fi
if [[ -z "${BCAST_ID}" ]]; then
  BCAST_ID="${FILE_BCAST_ID}"
fi
if [[ -z "${DECISION_REF}" ]]; then
  DECISION_REF="${FILE_DECISION_REF}"
fi
if [[ -z "${REMINDER_REF}" ]]; then
  REMINDER_REF="${FILE_REMINDER_REF}"
fi
if [[ -z "${BUSY_MODE_OVERRIDE}" ]]; then
  BUSY_MODE_OVERRIDE="${FILE_BUSY_MODE_OVERRIDE}"
fi
if [[ "${REFRESH_DAILY_REVIEW}" == "1" && -n "${FILE_REFRESH_DAILY_REVIEW}" ]]; then
  REFRESH_DAILY_REVIEW="${FILE_REFRESH_DAILY_REVIEW}"
fi

if [[ -z "${PHASE}" || -z "${DECISION}" ]]; then
  usage
  exit 2
fi

if [[ "${PHASE}" != "code" && "${PHASE}" != "launch" ]]; then
  fail "invalid_phase expected=code_or_launch actual=${PHASE}"
fi

if [[ "${DECISION}" != "approve" && "${DECISION}" != "deny" && "${DECISION}" != "pending" ]]; then
  fail "invalid_decision expected=approve_or_deny_or_pending actual=${DECISION}"
fi

if [[ -n "${BUSY_MODE_OVERRIDE}" ]]; then
  validate_flag01 "BUSY_MODE_OVERRIDE" "${BUSY_MODE_OVERRIDE}"
  set_kv "BUSY_MODE" "${BUSY_MODE_OVERRIDE}"
fi

validate_flag01 "REFRESH_DAILY_REVIEW" "${REFRESH_DAILY_REVIEW}"
if [[ "${REFRESH_DAILY_REVIEW}" == "1" ]]; then
  TODAY_UTC="$(date -u +%Y-%m-%d)"
  set_kv "DAILY_REVIEW_OK" "1"
  set_kv "DAILY_REVIEW_DATE_UTC" "${TODAY_UTC}"
fi

if [[ "${PHASE}" == "code" ]]; then
  case "${DECISION}" in
    approve)
      validate_token_ascii "BCAST_ID" "${BCAST_ID}"
      validate_token_ascii "DECISION_REF" "${DECISION_REF}"
      set_kv "CODE_APPROVED" "1"
      set_kv "CODE_APPROVAL_BCAST_ID" "${BCAST_ID}"
      set_kv "CODE_APPROVAL_DECISION_REF" "${DECISION_REF}"
      set_kv "CODE_REMINDER_SCHEDULED" "0"
      set_kv "CODE_REMINDER_REF" ""
      ;;
    deny)
      validate_token_ascii "BCAST_ID" "${BCAST_ID}"
      validate_token_ascii "DECISION_REF" "${DECISION_REF}"
      set_kv "CODE_APPROVED" "0"
      set_kv "CODE_APPROVAL_BCAST_ID" "${BCAST_ID}"
      set_kv "CODE_APPROVAL_DECISION_REF" "${DECISION_REF}"
      set_kv "CODE_REMINDER_SCHEDULED" "0"
      set_kv "CODE_REMINDER_REF" ""
      ;;
    pending)
      set_kv "CODE_APPROVED" "0"
      if [[ -n "${REMINDER_REF}" ]]; then
        validate_token_ascii "REMINDER_REF" "${REMINDER_REF}"
        set_kv "BUSY_MODE" "1"
        set_kv "CODE_REMINDER_SCHEDULED" "1"
        set_kv "CODE_REMINDER_REF" "${REMINDER_REF}"
      else
        set_kv "CODE_REMINDER_SCHEDULED" "0"
        set_kv "CODE_REMINDER_REF" ""
      fi
      ;;
  esac
else
  case "${DECISION}" in
    approve)
      validate_token_ascii "BCAST_ID" "${BCAST_ID}"
      validate_token_ascii "DECISION_REF" "${DECISION_REF}"
      set_kv "LAUNCH_APPROVED" "1"
      set_kv "LAUNCH_APPROVAL_BCAST_ID" "${BCAST_ID}"
      set_kv "LAUNCH_APPROVAL_DECISION_REF" "${DECISION_REF}"
      set_kv "LAUNCH_REMINDER_SCHEDULED" "0"
      set_kv "LAUNCH_REMINDER_REF" ""
      ;;
    deny)
      validate_token_ascii "BCAST_ID" "${BCAST_ID}"
      validate_token_ascii "DECISION_REF" "${DECISION_REF}"
      set_kv "LAUNCH_APPROVED" "0"
      set_kv "LAUNCH_APPROVAL_BCAST_ID" "${BCAST_ID}"
      set_kv "LAUNCH_APPROVAL_DECISION_REF" "${DECISION_REF}"
      set_kv "LAUNCH_REMINDER_SCHEDULED" "0"
      set_kv "LAUNCH_REMINDER_REF" ""
      ;;
    pending)
      set_kv "LAUNCH_APPROVED" "0"
      if [[ -n "${REMINDER_REF}" ]]; then
        validate_token_ascii "REMINDER_REF" "${REMINDER_REF}"
        set_kv "BUSY_MODE" "1"
        set_kv "LAUNCH_REMINDER_SCHEDULED" "1"
        set_kv "LAUNCH_REMINDER_REF" "${REMINDER_REF}"
      else
        set_kv "LAUNCH_REMINDER_SCHEDULED" "0"
        set_kv "LAUNCH_REMINDER_REF" ""
      fi
      ;;
  esac
fi

rm -f "${ENV_FILE}.bak"
echo "APPLY_OK builder_permission_decision phase=${PHASE} decision=${DECISION} env_file=${ENV_FILE}"
