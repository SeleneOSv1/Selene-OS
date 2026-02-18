#!/usr/bin/env bash
set -euo pipefail

ACK_FILE="${ACK_FILE:-.dev/builder_failure_alert_ack.log}"
ROUTING_FILE="${ROUTING_FILE:-.dev/builder_failure_alert_routing.env}"

usage() {
  cat <<'USAGE'
Usage:
  bash scripts/ack_builder_failure_alert.sh <bcast_id> [recipient_user_id]
USAGE
}

fail() {
  echo "ROUTING_ACK_FAIL:$1" >&2
  exit 1
}

is_token_safe() {
  local value="$1"
  [[ "${value}" =~ ^[A-Za-z0-9._:@/-]+$ ]]
}

if [[ $# -lt 1 || $# -gt 2 ]]; then
  usage
  fail "invalid_arguments"
fi

bcast_id="$1"
recipient_user_id="${2:-}"
if ! is_token_safe "${bcast_id}"; then
  fail "invalid_bcast_id expected_token_safe_ascii actual=${bcast_id}"
fi
if [[ -n "${recipient_user_id}" ]] && ! is_token_safe "${recipient_user_id}"; then
  fail "invalid_recipient_user_id expected_token_safe_ascii actual=${recipient_user_id}"
fi

mkdir -p "$(dirname "${ACK_FILE}")"
touch "${ACK_FILE}"
epoch_now="$(date +%s)"
at_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
printf "ACK|epoch=%s|at=%s|bcast_id=%s|recipient_user_id=%s\n" \
  "${epoch_now}" "${at_utc}" "${bcast_id}" "${recipient_user_id:-unknown}" >> "${ACK_FILE}"

if [[ ! -f "${ROUTING_FILE}" ]]; then
  echo "ROUTING_ACK_OK bcast_id=${bcast_id} recipient_user_id=${recipient_user_id:-unknown} ack_file=${ACK_FILE} routing_file=missing"
  exit 0
fi

# shellcheck disable=SC1090
source "${ROUTING_FILE}"

ALERT_LAST_PENDING_BCAST_ID="${ALERT_LAST_PENDING_BCAST_ID:-}"
ALERT_LAST_PENDING_RECIPIENT_USER_ID="${ALERT_LAST_PENDING_RECIPIENT_USER_ID:-}"
ALERT_LAST_PENDING_SENT_EPOCH="${ALERT_LAST_PENDING_SENT_EPOCH:-0}"
ALERT_LAST_PENDING_ACK_EPOCH="${ALERT_LAST_PENDING_ACK_EPOCH:-0}"
ALERT_LAST_PENDING_ACK_STATUS="${ALERT_LAST_PENDING_ACK_STATUS:-NONE}"
ALERT_LAST_ROUTE_SOURCE="${ALERT_LAST_ROUTE_SOURCE:-PRIMARY}"

if [[ "${ALERT_LAST_PENDING_BCAST_ID}" != "${bcast_id}" ]]; then
  echo "ROUTING_ACK_OK bcast_id=${bcast_id} recipient_user_id=${recipient_user_id:-unknown} ack_file=${ACK_FILE} routing_update=skipped reason=not_current_pending"
  exit 0
fi

if [[ -n "${recipient_user_id}" && -n "${ALERT_LAST_PENDING_RECIPIENT_USER_ID}" && "${recipient_user_id}" != "${ALERT_LAST_PENDING_RECIPIENT_USER_ID}" ]]; then
  echo "ROUTING_ACK_OK bcast_id=${bcast_id} recipient_user_id=${recipient_user_id} ack_file=${ACK_FILE} routing_update=skipped reason=recipient_mismatch expected=${ALERT_LAST_PENDING_RECIPIENT_USER_ID}"
  exit 0
fi

ALERT_LAST_PENDING_ACK_EPOCH="${epoch_now}"
ALERT_LAST_PENDING_ACK_STATUS="ACKED"
ALERT_LAST_ROUTE_SOURCE="${ALERT_LAST_ROUTE_SOURCE}"

tmp="$(mktemp "${TMPDIR:-/tmp}/builder_failure_alert_routing_ack.$$.XXXXXXXXXXXX.env")"
{
  printf "# Managed by ack_builder_failure_alert.sh\n"
  printf "ALERT_PRIMARY_USER_ID=%q\n" "${ALERT_PRIMARY_USER_ID:-owner}"
  printf "ALERT_PRIMARY_DISPLAY=%q\n" "${ALERT_PRIMARY_DISPLAY:-JD}"
  printf "ALERT_FALLBACK_USER_ID=%q\n" "${ALERT_FALLBACK_USER_ID:-}"
  printf "ALERT_FALLBACK_DISPLAY=%q\n" "${ALERT_FALLBACK_DISPLAY:-}"
  printf "ALERT_DELEGATED_USER_ID=%q\n" "${ALERT_DELEGATED_USER_ID:-}"
  printf "ALERT_DELEGATED_DISPLAY=%q\n" "${ALERT_DELEGATED_DISPLAY:-}"
  printf "ALERT_DELEGATION_ACTIVE=%q\n" "${ALERT_DELEGATION_ACTIVE:-0}"
  printf "ALERT_RESPONSE_TIMEOUT_MINUTES=%q\n" "${ALERT_RESPONSE_TIMEOUT_MINUTES:-5}"
  printf "ALERT_LAST_PENDING_BCAST_ID=%q\n" "${ALERT_LAST_PENDING_BCAST_ID}"
  printf "ALERT_LAST_PENDING_RECIPIENT_USER_ID=%q\n" "${ALERT_LAST_PENDING_RECIPIENT_USER_ID}"
  printf "ALERT_LAST_PENDING_SENT_EPOCH=%q\n" "${ALERT_LAST_PENDING_SENT_EPOCH}"
  printf "ALERT_LAST_PENDING_ACK_EPOCH=%q\n" "${ALERT_LAST_PENDING_ACK_EPOCH}"
  printf "ALERT_LAST_PENDING_ACK_STATUS=%q\n" "${ALERT_LAST_PENDING_ACK_STATUS}"
  printf "ALERT_LAST_ROUTE_SOURCE=%q\n" "${ALERT_LAST_ROUTE_SOURCE}"
} > "${tmp}"
mv -f "${tmp}" "${ROUTING_FILE}"

echo "ROUTING_ACK_OK bcast_id=${bcast_id} recipient_user_id=${recipient_user_id:-${ALERT_LAST_PENDING_RECIPIENT_USER_ID:-unknown}} ack_file=${ACK_FILE} routing_update=applied"
