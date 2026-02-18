#!/usr/bin/env bash
set -euo pipefail

ROUTING_FILE="${ROUTING_FILE:-.dev/builder_failure_alert_routing.env}"

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/set_builder_failure_alert_recipient.sh show
  bash scripts/set_builder_failure_alert_recipient.sh set-primary <user_id> <display_name>
  bash scripts/set_builder_failure_alert_recipient.sh set-fallback <user_id> <display_name>
  bash scripts/set_builder_failure_alert_recipient.sh set-delegate <user_id> <display_name> [response_timeout_minutes]
  bash scripts/set_builder_failure_alert_recipient.sh clear-delegate
USAGE
}

fail() {
  echo "ROUTING_SET_FAIL:$1" >&2
  exit 1
}

is_token_safe() {
  local value="$1"
  [[ "${value}" =~ ^[A-Za-z0-9._:@/-]+$ ]]
}

ensure_token() {
  local field="$1"
  local value="$2"
  if [[ -z "${value}" ]]; then
    fail "missing_${field}"
  fi
  if ! is_token_safe "${value}"; then
    fail "invalid_${field} expected_token_safe_ascii actual=${value}"
  fi
}

if [[ $# -lt 1 ]]; then
  usage
  fail "missing_action"
fi

action="$1"
shift

mkdir -p "$(dirname "${ROUTING_FILE}")"

ALERT_PRIMARY_USER_ID="owner"
ALERT_PRIMARY_DISPLAY="JD"
ALERT_FALLBACK_USER_ID=""
ALERT_FALLBACK_DISPLAY=""
ALERT_DELEGATED_USER_ID=""
ALERT_DELEGATED_DISPLAY=""
ALERT_DELEGATION_ACTIVE="0"
ALERT_RESPONSE_TIMEOUT_MINUTES="5"
ALERT_LAST_PENDING_BCAST_ID=""
ALERT_LAST_PENDING_RECIPIENT_USER_ID=""
ALERT_LAST_PENDING_SENT_EPOCH="0"
ALERT_LAST_PENDING_ACK_EPOCH="0"
ALERT_LAST_PENDING_ACK_STATUS="NONE"
ALERT_LAST_ROUTE_SOURCE="PRIMARY"

if [[ -f "${ROUTING_FILE}" ]]; then
  # shellcheck disable=SC1090
  source "${ROUTING_FILE}"
fi

if [[ "${action}" == "show" ]]; then
  cat "${ROUTING_FILE}" 2>/dev/null || {
    echo "ROUTING_SHOW_EMPTY routing_file=${ROUTING_FILE}"
    exit 0
  }
  exit 0
fi

write_routing() {
  local tmp
  tmp="$(mktemp "${TMPDIR:-/tmp}/builder_failure_alert_routing_set.$$.XXXXXXXXXXXX.env")"
  {
    printf "# Managed by set_builder_failure_alert_recipient.sh\n"
    printf "ALERT_PRIMARY_USER_ID=%q\n" "${ALERT_PRIMARY_USER_ID}"
    printf "ALERT_PRIMARY_DISPLAY=%q\n" "${ALERT_PRIMARY_DISPLAY}"
    printf "ALERT_FALLBACK_USER_ID=%q\n" "${ALERT_FALLBACK_USER_ID}"
    printf "ALERT_FALLBACK_DISPLAY=%q\n" "${ALERT_FALLBACK_DISPLAY}"
    printf "ALERT_DELEGATED_USER_ID=%q\n" "${ALERT_DELEGATED_USER_ID}"
    printf "ALERT_DELEGATED_DISPLAY=%q\n" "${ALERT_DELEGATED_DISPLAY}"
    printf "ALERT_DELEGATION_ACTIVE=%q\n" "${ALERT_DELEGATION_ACTIVE}"
    printf "ALERT_RESPONSE_TIMEOUT_MINUTES=%q\n" "${ALERT_RESPONSE_TIMEOUT_MINUTES}"
    printf "ALERT_LAST_PENDING_BCAST_ID=%q\n" "${ALERT_LAST_PENDING_BCAST_ID}"
    printf "ALERT_LAST_PENDING_RECIPIENT_USER_ID=%q\n" "${ALERT_LAST_PENDING_RECIPIENT_USER_ID}"
    printf "ALERT_LAST_PENDING_SENT_EPOCH=%q\n" "${ALERT_LAST_PENDING_SENT_EPOCH}"
    printf "ALERT_LAST_PENDING_ACK_EPOCH=%q\n" "${ALERT_LAST_PENDING_ACK_EPOCH}"
    printf "ALERT_LAST_PENDING_ACK_STATUS=%q\n" "${ALERT_LAST_PENDING_ACK_STATUS}"
    printf "ALERT_LAST_ROUTE_SOURCE=%q\n" "${ALERT_LAST_ROUTE_SOURCE}"
  } > "${tmp}"
  mv -f "${tmp}" "${ROUTING_FILE}"
}

case "${action}" in
  set-primary)
    [[ $# -eq 2 ]] || fail "set-primary requires <user_id> <display_name>"
    ensure_token "primary_user_id" "$1"
    ensure_token "primary_display" "$2"
    ALERT_PRIMARY_USER_ID="$1"
    ALERT_PRIMARY_DISPLAY="$2"
    ;;
  set-fallback)
    [[ $# -eq 2 ]] || fail "set-fallback requires <user_id> <display_name>"
    ensure_token "fallback_user_id" "$1"
    ensure_token "fallback_display" "$2"
    ALERT_FALLBACK_USER_ID="$1"
    ALERT_FALLBACK_DISPLAY="$2"
    ;;
  set-delegate)
    [[ $# -ge 2 && $# -le 3 ]] || fail "set-delegate requires <user_id> <display_name> [response_timeout_minutes]"
    ensure_token "delegated_user_id" "$1"
    ensure_token "delegated_display" "$2"
    ALERT_DELEGATED_USER_ID="$1"
    ALERT_DELEGATED_DISPLAY="$2"
    ALERT_DELEGATION_ACTIVE="1"
    if [[ $# -eq 3 ]]; then
      if ! [[ "$3" =~ ^[0-9]+$ ]]; then
        fail "invalid_response_timeout_minutes expected_non_negative_integer actual=$3"
      fi
      ALERT_RESPONSE_TIMEOUT_MINUTES="$3"
    fi
    ;;
  clear-delegate)
    [[ $# -eq 0 ]] || fail "clear-delegate takes no extra arguments"
    ALERT_DELEGATION_ACTIVE="0"
    ALERT_DELEGATED_USER_ID=""
    ALERT_DELEGATED_DISPLAY=""
    ;;
  *)
    usage
    fail "unknown_action action=${action}"
    ;;
esac

ensure_token "primary_user_id" "${ALERT_PRIMARY_USER_ID}"
ensure_token "primary_display" "${ALERT_PRIMARY_DISPLAY}"
if [[ -n "${ALERT_FALLBACK_USER_ID}" ]]; then
  ensure_token "fallback_user_id" "${ALERT_FALLBACK_USER_ID}"
  ensure_token "fallback_display" "${ALERT_FALLBACK_DISPLAY}"
fi
if ! [[ "${ALERT_RESPONSE_TIMEOUT_MINUTES}" =~ ^[0-9]+$ ]]; then
  fail "invalid_response_timeout_minutes expected_non_negative_integer actual=${ALERT_RESPONSE_TIMEOUT_MINUTES}"
fi

write_routing
echo "ROUTING_SET_OK action=${action} routing_file=${ROUTING_FILE} primary=${ALERT_PRIMARY_USER_ID} delegate_active=${ALERT_DELEGATION_ACTIVE} delegate=${ALERT_DELEGATED_USER_ID:-none} fallback=${ALERT_FALLBACK_USER_ID:-none} timeout_minutes=${ALERT_RESPONSE_TIMEOUT_MINUTES}"
