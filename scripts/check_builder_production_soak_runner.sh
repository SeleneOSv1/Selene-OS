#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${SELENE_ROOT:-$(git rev-parse --show-toplevel)}"
cd "${ROOT_DIR}"

RUN_MODE="${RUN_MODE:-once}"                 # once | loop
INTERVAL_MINUTES="${INTERVAL_MINUTES:-60}"   # used only in loop mode
FAIL_CLOSED="${FAIL_CLOSED:-1}"              # 1 => exit non-zero on failure
ALERT_ON_FAIL="${ALERT_ON_FAIL:-1}"          # 1 => append alert line to alert log
WATCHDOG_CMD="${WATCHDOG_CMD:-scripts/check_builder_production_soak_watchdog.sh}"
ALERT_LOG_FILE="${ALERT_LOG_FILE:-.dev/builder_production_soak_alerts.log}"
STATE_FILE="${STATE_FILE:-.dev/builder_production_soak_runner_state.log}"
BCAST_ON_FAIL="${BCAST_ON_FAIL:-1}"          # 1 => dispatch PH1.BCAST failure alert bridge
BCAST_ALERT_CMD="${BCAST_ALERT_CMD:-scripts/emit_builder_failure_bcast_alert.sh}"
BCAST_LOG_FILE="${BCAST_LOG_FILE:-.dev/builder_failure_bcast_ledger.log}"
BCAST_APP_INBOX_FILE="${BCAST_APP_INBOX_FILE:-.dev/selene_app_inbox.log}"
BCAST_ROUTING_FILE="${BCAST_ROUTING_FILE:-.dev/builder_failure_alert_routing.env}"
BCAST_ACK_FILE="${BCAST_ACK_FILE:-.dev/builder_failure_alert_ack.log}"
BCAST_ROUTING_AUDIT_FILE="${BCAST_ROUTING_AUDIT_FILE:-.dev/builder_failure_alert_routing_audit.log}"
BCAST_RECIPIENT_DISPLAY="${BCAST_RECIPIENT_DISPLAY:-JD}"
BCAST_URGENCY="${BCAST_URGENCY:-URGENT}"      # URGENT | NON_URGENT
BCAST_ALERT_COOLDOWN_MINUTES="${BCAST_ALERT_COOLDOWN_MINUTES:-60}"
BCAST_NOTIFY_DESKTOP="${BCAST_NOTIFY_DESKTOP:-1}" # 1 => desktop notification via osascript
PROPOSAL_ID="${PROPOSAL_ID:-}"
RELEASE_STATE_ID="${RELEASE_STATE_ID:-}"

fail() {
  echo "PRODUCTION_SOAK_RUNNER_FAIL:$1" >&2
  exit 1
}

if [[ "${RUN_MODE}" != "once" && "${RUN_MODE}" != "loop" ]]; then
  fail "invalid_RUN_MODE expected=once_or_loop actual=${RUN_MODE}"
fi
if ! [[ "${INTERVAL_MINUTES}" =~ ^[0-9]+$ ]] || [[ "${INTERVAL_MINUTES}" -lt 1 ]]; then
  fail "invalid_INTERVAL_MINUTES expected_positive_integer actual=${INTERVAL_MINUTES}"
fi
if [[ "${FAIL_CLOSED}" != "0" && "${FAIL_CLOSED}" != "1" ]]; then
  fail "invalid_FAIL_CLOSED expected=0_or_1 actual=${FAIL_CLOSED}"
fi
if [[ "${ALERT_ON_FAIL}" != "0" && "${ALERT_ON_FAIL}" != "1" ]]; then
  fail "invalid_ALERT_ON_FAIL expected=0_or_1 actual=${ALERT_ON_FAIL}"
fi
if [[ ! -x "${WATCHDOG_CMD}" ]]; then
  fail "watchdog_not_executable path=${WATCHDOG_CMD}"
fi
if [[ "${BCAST_ON_FAIL}" != "0" && "${BCAST_ON_FAIL}" != "1" ]]; then
  fail "invalid_BCAST_ON_FAIL expected=0_or_1 actual=${BCAST_ON_FAIL}"
fi
if [[ "${BCAST_NOTIFY_DESKTOP}" != "0" && "${BCAST_NOTIFY_DESKTOP}" != "1" ]]; then
  fail "invalid_BCAST_NOTIFY_DESKTOP expected=0_or_1 actual=${BCAST_NOTIFY_DESKTOP}"
fi
if [[ "${BCAST_URGENCY}" != "URGENT" && "${BCAST_URGENCY}" != "NON_URGENT" ]]; then
  fail "invalid_BCAST_URGENCY expected=URGENT_or_NON_URGENT actual=${BCAST_URGENCY}"
fi
if ! [[ "${BCAST_ALERT_COOLDOWN_MINUTES}" =~ ^[0-9]+$ ]]; then
  fail "invalid_BCAST_ALERT_COOLDOWN_MINUTES expected_non_negative_integer actual=${BCAST_ALERT_COOLDOWN_MINUTES}"
fi
if [[ "${BCAST_ON_FAIL}" == "1" && ! -x "${BCAST_ALERT_CMD}" ]]; then
  fail "bcast_alert_cmd_not_executable path=${BCAST_ALERT_CMD}"
fi

mkdir -p "$(dirname "${ALERT_LOG_FILE}")"
mkdir -p "$(dirname "${STATE_FILE}")"
mkdir -p "$(dirname "${BCAST_LOG_FILE}")"
mkdir -p "$(dirname "${BCAST_APP_INBOX_FILE}")"
mkdir -p "$(dirname "${BCAST_ROUTING_FILE}")"
mkdir -p "$(dirname "${BCAST_ACK_FILE}")"
mkdir -p "$(dirname "${BCAST_ROUTING_AUDIT_FILE}")"

emit_state() {
  local line="$1"
  printf "%s\n" "${line}" >> "${STATE_FILE}"
}

emit_alert() {
  local line="$1"
  printf "%s\n" "${line}" | tee -a "${ALERT_LOG_FILE}" >&2
}

collapse_line() {
  local line="$1"
  printf "%s" "${line}" | tr '\n' ' ' | sed 's/[[:space:]]\+/ /g; s/^ //; s/ $//'
}

dispatch_bcast_failure_alert() {
  local alert_kind="$1"
  local now_utc="$2"
  local fail_line="$3"
  local detail_line="$4"

  if [[ "${BCAST_ON_FAIL}" != "1" ]]; then
    return 0
  fi

  local bcast_output
  local bcast_rc
  set +e
  bcast_output="$(
    ALERT_KIND="${alert_kind}" \
    ALERT_AT_UTC="${now_utc}" \
    ALERT_SUMMARY="${fail_line}" \
    ALERT_DETAIL="${detail_line}" \
    WATCHDOG_CMD="${WATCHDOG_CMD}" \
    PROPOSAL_ID="${PROPOSAL_ID}" \
    RELEASE_STATE_ID="${RELEASE_STATE_ID}" \
    BCAST_LOG_FILE="${BCAST_LOG_FILE}" \
    BCAST_APP_INBOX_FILE="${BCAST_APP_INBOX_FILE}" \
    BCAST_ROUTING_FILE="${BCAST_ROUTING_FILE}" \
    BCAST_ACK_FILE="${BCAST_ACK_FILE}" \
    BCAST_ROUTING_AUDIT_FILE="${BCAST_ROUTING_AUDIT_FILE}" \
    BCAST_RECIPIENT_DISPLAY="${BCAST_RECIPIENT_DISPLAY}" \
    BCAST_URGENCY="${BCAST_URGENCY}" \
    BCAST_ALERT_COOLDOWN_MINUTES="${BCAST_ALERT_COOLDOWN_MINUTES}" \
    BCAST_NOTIFY_DESKTOP="${BCAST_NOTIFY_DESKTOP}" \
    "${BCAST_ALERT_CMD}" 2>&1
  )"
  bcast_rc=$?
  set -e

  local bcast_line
  bcast_line="$(collapse_line "${bcast_output}")"
  if [[ ${bcast_rc} -eq 0 ]]; then
    local ok_line="ALERT_BCAST_OK builder_production_soak_runner run_mode=${RUN_MODE} at=${now_utc} alert_kind=${alert_kind} bcast_cmd=${BCAST_ALERT_CMD}"
    if [[ "${ALERT_ON_FAIL}" == "1" ]]; then
      emit_alert "${ok_line}"
      emit_alert "ALERT_BCAST_DETAIL ${bcast_line}"
    else
      echo "${ok_line}" >&2
      echo "ALERT_BCAST_DETAIL ${bcast_line}" >&2
    fi
    emit_state "${ok_line}"
    emit_state "ALERT_BCAST_DETAIL ${bcast_line}"
    return 0
  fi

  local fail_bcast_line="ALERT_BCAST_FAIL builder_production_soak_runner run_mode=${RUN_MODE} at=${now_utc} alert_kind=${alert_kind} bcast_cmd=${BCAST_ALERT_CMD} fail_closed=${FAIL_CLOSED}"
  if [[ "${ALERT_ON_FAIL}" == "1" ]]; then
    emit_alert "${fail_bcast_line}"
    emit_alert "ALERT_BCAST_DETAIL ${bcast_line}"
  else
    echo "${fail_bcast_line}" >&2
    echo "ALERT_BCAST_DETAIL ${bcast_line}" >&2
  fi
  emit_state "${fail_bcast_line}"
  emit_state "ALERT_BCAST_DETAIL ${bcast_line}"

  if [[ "${FAIL_CLOSED}" == "1" ]]; then
    return 1
  fi
  return 0
}

run_tick() {
  local now_utc
  now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

  local output
  local rc
  set +e
  output="$(${WATCHDOG_CMD} 2>&1)"
  rc=$?
  set -e

  if [[ ${rc} -eq 0 ]]; then
    local pass_line="CHECK_OK builder_production_soak_runner=tick_pass run_mode=${RUN_MODE} at=${now_utc} watchdog_cmd=${WATCHDOG_CMD}"
    echo "${pass_line}"
    emit_state "${pass_line}"
    emit_state "WATCHDOG_OUTPUT ${output}"
    return 0
  fi

  local alert_kind="PRODUCTION_SOAK_CHECK_FAILED"
  if [[ "${output}" == *"STALE_CANARY_TELEMETRY:"* ]]; then
    alert_kind="PRODUCTION_SOAK_STALE_TELEMETRY"
  fi

  local fail_line="ALERT builder_production_soak_runner=${alert_kind} run_mode=${RUN_MODE} at=${now_utc} watchdog_cmd=${WATCHDOG_CMD} fail_closed=${FAIL_CLOSED}"
  local detail_line="ALERT_DETAIL ${output}"

  if [[ "${ALERT_ON_FAIL}" == "1" ]]; then
    emit_alert "${fail_line}"
    emit_alert "${detail_line}"
  else
    echo "${fail_line}" >&2
    echo "${detail_line}" >&2
  fi

  emit_state "${fail_line}"
  emit_state "${detail_line}"

  if ! dispatch_bcast_failure_alert "${alert_kind}" "${now_utc}" "${fail_line}" "${detail_line}"; then
    return 1
  fi

  if [[ "${FAIL_CLOSED}" == "1" ]]; then
    return 1
  fi
  return 0
}

if [[ "${RUN_MODE}" == "once" ]]; then
  run_tick
  exit $?
fi

while true; do
  if ! run_tick; then
    exit 1
  fi
  sleep "$(( INTERVAL_MINUTES * 60 ))"
done
