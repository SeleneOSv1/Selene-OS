#!/usr/bin/env bash
set -euo pipefail

LABEL="${LABEL:-com.selene.builder.production_soak_runner}"
LAUNCH_AGENTS_DIR="${LAUNCH_AGENTS_DIR:-${HOME}/Library/LaunchAgents}"
PLIST_PATH="${PLIST_PATH:-${LAUNCH_AGENTS_DIR}/${LABEL}.plist}"
RUNTIME_ROOT="${RUNTIME_ROOT:-${HOME}/.selene_automation/production_soak}"
REMOVE_RUNTIME_BUNDLE="${REMOVE_RUNTIME_BUNDLE:-0}"

fail() {
  echo "PRODUCTION_SOAK_LAUNCHD_REMOVE_FAIL:$1" >&2
  exit 1
}

if ! command -v launchctl >/dev/null 2>&1; then
  fail "launchctl_not_found"
fi
if [[ "${REMOVE_RUNTIME_BUNDLE}" != "0" && "${REMOVE_RUNTIME_BUNDLE}" != "1" ]]; then
  fail "invalid_REMOVE_RUNTIME_BUNDLE expected=0_or_1 actual=${REMOVE_RUNTIME_BUNDLE}"
fi

launchctl bootout "gui/$(id -u)/${LABEL}" >/dev/null 2>&1 || true
launchctl bootout "user/$(id -u)/${LABEL}" >/dev/null 2>&1 || true
rm -f "${PLIST_PATH}"
if [[ "${REMOVE_RUNTIME_BUNDLE}" == "1" ]]; then
  rm -rf "${RUNTIME_ROOT}"
fi

echo "CHECK_OK builder_production_soak_launchd_remove=pass label=${LABEL} plist=${PLIST_PATH} runtime_removed=${REMOVE_RUNTIME_BUNDLE}"
