#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "${REPO_ROOT}"

LABEL="${LABEL:-com.selene.builder.production_soak_runner}"
INTERVAL_MINUTES="${INTERVAL_MINUTES:-60}"
RUNTIME_ROOT="${RUNTIME_ROOT:-${HOME}/.selene_automation/production_soak}"
RUNTIME_SCRIPTS_DIR="${RUNTIME_ROOT}/scripts"
RUNTIME_DEV_DIR="${RUNTIME_ROOT}/.dev"
LAUNCH_AGENTS_DIR="${LAUNCH_AGENTS_DIR:-${HOME}/Library/LaunchAgents}"
PLIST_PATH="${PLIST_PATH:-${LAUNCH_AGENTS_DIR}/${LABEL}.plist}"
RUNNER_PATH="${RUNNER_PATH:-${RUNTIME_SCRIPTS_DIR}/check_builder_production_soak_runner.sh}"
WATCHDOG_PATH="${WATCHDOG_PATH:-${RUNTIME_SCRIPTS_DIR}/check_builder_production_soak_watchdog.sh}"
ALERT_LOG_FILE="${ALERT_LOG_FILE:-${RUNTIME_DEV_DIR}/builder_production_soak_alerts.log}"
STATE_FILE="${STATE_FILE:-${RUNTIME_DEV_DIR}/builder_production_soak_runner_state.log}"
BCAST_ALERT_CMD="${BCAST_ALERT_CMD:-${RUNTIME_SCRIPTS_DIR}/emit_builder_failure_bcast_alert.sh}"
BCAST_LOG_FILE="${BCAST_LOG_FILE:-${RUNTIME_DEV_DIR}/builder_failure_bcast_ledger.log}"
BCAST_APP_INBOX_FILE="${BCAST_APP_INBOX_FILE:-${RUNTIME_DEV_DIR}/selene_app_inbox.log}"
BCAST_ROUTING_FILE="${BCAST_ROUTING_FILE:-${RUNTIME_DEV_DIR}/builder_failure_alert_routing.env}"
BCAST_ACK_FILE="${BCAST_ACK_FILE:-${RUNTIME_DEV_DIR}/builder_failure_alert_ack.log}"
BCAST_ROUTING_AUDIT_FILE="${BCAST_ROUTING_AUDIT_FILE:-${RUNTIME_DEV_DIR}/builder_failure_alert_routing_audit.log}"
BCAST_RECIPIENT_DISPLAY="${BCAST_RECIPIENT_DISPLAY:-JD}"
BCAST_URGENCY="${BCAST_URGENCY:-URGENT}"
BCAST_ALERT_COOLDOWN_MINUTES="${BCAST_ALERT_COOLDOWN_MINUTES:-60}"
BCAST_NOTIFY_DESKTOP="${BCAST_NOTIFY_DESKTOP:-1}"
LOG_DIR="${LOG_DIR:-${RUNTIME_ROOT}/launchd}"
STDOUT_LOG="${STDOUT_LOG:-${LOG_DIR}/builder_production_soak_runner.out.log}"
STDERR_LOG="${STDERR_LOG:-${LOG_DIR}/builder_production_soak_runner.err.log}"
DB_ENV_SOURCE="${DB_ENV_SOURCE:-${REPO_ROOT}/.dev/db.env}"
DB_ENV_DEST="${DB_ENV_DEST:-${RUNTIME_DEV_DIR}/db.env}"

fail() {
  echo "PRODUCTION_SOAK_LAUNCHD_SETUP_FAIL:$1" >&2
  exit 1
}

if ! [[ "${INTERVAL_MINUTES}" =~ ^[0-9]+$ ]] || [[ "${INTERVAL_MINUTES}" -lt 1 ]]; then
  fail "invalid_INTERVAL_MINUTES expected_positive_integer actual=${INTERVAL_MINUTES}"
fi
if ! command -v launchctl >/dev/null 2>&1; then
  fail "launchctl_not_found"
fi
if [[ ! -f "${DB_ENV_SOURCE}" ]]; then
  fail "missing_db_env_source path=${DB_ENV_SOURCE}"
fi

mkdir -p "${RUNTIME_SCRIPTS_DIR}"
mkdir -p "${RUNTIME_DEV_DIR}"
mkdir -p "${LAUNCH_AGENTS_DIR}"
mkdir -p "${LOG_DIR}"
mkdir -p "$(dirname "${ALERT_LOG_FILE}")"
mkdir -p "$(dirname "${STATE_FILE}")"
mkdir -p "$(dirname "${BCAST_LOG_FILE}")"
mkdir -p "$(dirname "${BCAST_APP_INBOX_FILE}")"
mkdir -p "$(dirname "${BCAST_ROUTING_FILE}")"
mkdir -p "$(dirname "${BCAST_ACK_FILE}")"
mkdir -p "$(dirname "${BCAST_ROUTING_AUDIT_FILE}")"

for src in \
  "${REPO_ROOT}/scripts/check_builder_production_soak_runner.sh" \
  "${REPO_ROOT}/scripts/check_builder_production_soak_watchdog.sh" \
  "${REPO_ROOT}/scripts/export_builder_stage2_canary_metrics.sh" \
  "${REPO_ROOT}/scripts/check_builder_stage2_promotion_gate.sh" \
  "${REPO_ROOT}/scripts/emit_builder_failure_bcast_alert.sh" \
  "${REPO_ROOT}/scripts/set_builder_failure_alert_recipient.sh" \
  "${REPO_ROOT}/scripts/ack_builder_failure_alert.sh"
do
  if [[ ! -x "${src}" ]]; then
    fail "required_script_not_executable path=${src}"
  fi
  cp -f "${src}" "${RUNTIME_SCRIPTS_DIR}/$(basename "${src}")"
  chmod +x "${RUNTIME_SCRIPTS_DIR}/$(basename "${src}")"
done
cp -f "${DB_ENV_SOURCE}" "${DB_ENV_DEST}"

if [[ ! -x "${RUNNER_PATH}" ]]; then
  fail "runner_not_executable_after_deploy path=${RUNNER_PATH}"
fi
if [[ ! -x "${WATCHDOG_PATH}" ]]; then
  fail "watchdog_not_executable_after_deploy path=${WATCHDOG_PATH}"
fi
if [[ ! -x "${BCAST_ALERT_CMD}" ]]; then
  fail "bcast_alert_cmd_not_executable_after_deploy path=${BCAST_ALERT_CMD}"
fi

interval_seconds="$(( INTERVAL_MINUTES * 60 ))"

cat > "${PLIST_PATH}" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>${LABEL}</string>
  <key>ProgramArguments</key>
  <array>
    <string>/bin/zsh</string>
    <string>-lc</string>
    <string>cd '${RUNTIME_ROOT}' &amp;&amp; SELENE_ROOT='${RUNTIME_ROOT}' RUN_MODE=once FAIL_CLOSED=1 ALERT_ON_FAIL=1 BCAST_ON_FAIL=1 WATCHDOG_CMD='${WATCHDOG_PATH}' ALERT_LOG_FILE='${ALERT_LOG_FILE}' STATE_FILE='${STATE_FILE}' BCAST_ALERT_CMD='${BCAST_ALERT_CMD}' BCAST_LOG_FILE='${BCAST_LOG_FILE}' BCAST_APP_INBOX_FILE='${BCAST_APP_INBOX_FILE}' BCAST_ROUTING_FILE='${BCAST_ROUTING_FILE}' BCAST_ACK_FILE='${BCAST_ACK_FILE}' BCAST_ROUTING_AUDIT_FILE='${BCAST_ROUTING_AUDIT_FILE}' BCAST_RECIPIENT_DISPLAY='${BCAST_RECIPIENT_DISPLAY}' BCAST_URGENCY='${BCAST_URGENCY}' BCAST_ALERT_COOLDOWN_MINUTES='${BCAST_ALERT_COOLDOWN_MINUTES}' BCAST_NOTIFY_DESKTOP='${BCAST_NOTIFY_DESKTOP}' ENV_FILE='${DB_ENV_DEST}' '${RUNNER_PATH}'</string>
  </array>
  <key>WorkingDirectory</key>
  <string>${RUNTIME_ROOT}</string>
  <key>RunAtLoad</key>
  <true/>
  <key>StartInterval</key>
  <integer>${interval_seconds}</integer>
  <key>StandardOutPath</key>
  <string>${STDOUT_LOG}</string>
  <key>StandardErrorPath</key>
  <string>${STDERR_LOG}</string>
</dict>
</plist>
EOF

launchd_target_gui="gui/$(id -u)/${LABEL}"
launchd_target_user="user/$(id -u)/${LABEL}"
launchctl bootout "${launchd_target_gui}" >/dev/null 2>&1 || true
launchctl bootout "${launchd_target_user}" >/dev/null 2>&1 || true

active_domain=""
if launchctl bootstrap "gui/$(id -u)" "${PLIST_PATH}" >/dev/null 2>&1; then
  active_domain="gui/$(id -u)"
elif launchctl bootstrap "user/$(id -u)" "${PLIST_PATH}" >/dev/null 2>&1; then
  active_domain="user/$(id -u)"
else
  fail "launchctl_bootstrap_failed plist=${PLIST_PATH}"
fi

launchd_target="${active_domain}/${LABEL}"
launchctl enable "${launchd_target}" >/dev/null 2>&1 || true
launchctl kickstart -k "${launchd_target}" >/dev/null 2>&1 || true

echo "CHECK_OK builder_production_soak_launchd_setup=pass label=${LABEL} interval_minutes=${INTERVAL_MINUTES} plist=${PLIST_PATH} runtime_root=${RUNTIME_ROOT} domain=${active_domain}"
