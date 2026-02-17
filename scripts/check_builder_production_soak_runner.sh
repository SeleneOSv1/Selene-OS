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

mkdir -p "$(dirname "${ALERT_LOG_FILE}")"
mkdir -p "$(dirname "${STATE_FILE}")"

emit_state() {
  local line="$1"
  printf "%s\n" "${line}" >> "${STATE_FILE}"
}

emit_alert() {
  local line="$1"
  printf "%s\n" "${line}" | tee -a "${ALERT_LOG_FILE}" >&2
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
