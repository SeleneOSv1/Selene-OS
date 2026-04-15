#!/usr/bin/env bash
set -euo pipefail

USAGE="usage: check_section07_reopen_watch --repo-root <path> --base <rev> --head <rev> --report-path <path> --status-path <path>"

print_usage() {
  echo "$USAGE"
}

fail_arg() {
  echo "$1" >&2
  echo "$USAGE" >&2
  exit 64
}

require_value() {
  local flag="$1"
  local value="${2:-}"
  if [[ -z "$value" || "$value" == -* ]]; then
    fail_arg "missing value for $flag"
  fi
}

REPO_ROOT=""
BASE=""
HEAD=""
REPORT_PATH=""
STATUS_PATH=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    -h|--help)
      print_usage
      exit 0
      ;;
    --repo-root)
      require_value "--repo-root" "${2:-}"
      REPO_ROOT="$2"
      shift 2
      ;;
    --base)
      require_value "--base" "${2:-}"
      BASE="$2"
      shift 2
      ;;
    --head)
      require_value "--head" "${2:-}"
      HEAD="$2"
      shift 2
      ;;
    --report-path)
      require_value "--report-path" "${2:-}"
      REPORT_PATH="$2"
      shift 2
      ;;
    --status-path)
      require_value "--status-path" "${2:-}"
      STATUS_PATH="$2"
      shift 2
      ;;
    *)
      fail_arg "unsupported argument $1"
      ;;
  esac
done

[[ -n "$REPO_ROOT" ]] || fail_arg "missing required argument --repo-root"
[[ -n "$BASE" ]] || fail_arg "missing required argument --base"
[[ -n "$HEAD" ]] || fail_arg "missing required argument --head"
[[ -n "$REPORT_PATH" ]] || fail_arg "missing required argument --report-path"
[[ -n "$STATUS_PATH" ]] || fail_arg "missing required argument --status-path"

mkdir -p "$(dirname "$REPORT_PATH")"
mkdir -p "$(dirname "$STATUS_PATH")"

if ! SCANNER_OUTPUT="$(cargo run -p selene_os --bin section07_reopen_scan -- --repo-root "$REPO_ROOT" --base "$BASE" --head "$HEAD")"; then
  echo "section07 reopen scanner failed" >&2
  exit 1
fi

printf '%s\n' "$SCANNER_OUTPUT" > "$REPORT_PATH"

SECTION07_STATUS="$(printf '%s\n' "$SCANNER_OUTPUT" | sed -n 's/^status=//p' | head -n 1)"
if [[ -z "$SECTION07_STATUS" ]]; then
  echo "failed to extract status= line from Section 07 scanner report" >&2
  exit 1
fi

case "$SECTION07_STATUS" in
  StillBlocked|ProgramDReopenCandidate|ProgramEReopenCandidate)
    ;;
  *)
    echo "unexpected Section 07 scanner status: $SECTION07_STATUS" >&2
    exit 1
    ;;
esac

printf '%s\n' "$SECTION07_STATUS" > "$STATUS_PATH"
printf '%s\n' "$SCANNER_OUTPUT"
