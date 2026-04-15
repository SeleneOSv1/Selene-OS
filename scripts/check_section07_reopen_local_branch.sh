#!/usr/bin/env bash
set -euo pipefail

USAGE="usage: check_section07_reopen_local_branch [--repo-root <path>] [--base-ref <rev>] [--head-ref <rev>] [--report-path <path>] [--status-path <path>]"

print_usage() {
  echo "$USAGE"
}

fail_arg() {
  echo "$1" >&2
  echo "$USAGE" >&2
  exit 64
}

fail_run() {
  echo "$1" >&2
  exit 1
}

require_value() {
  local flag="$1"
  local value="${2:-}"
  if [[ -z "$value" || "$value" == -* ]]; then
    fail_arg "missing value for $flag"
  fi
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WATCH_SCRIPT="$SCRIPT_DIR/check_section07_reopen_watch.sh"

REPO_ROOT=""
BASE_REF="origin/main"
HEAD_REF="HEAD"
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
    --base-ref)
      require_value "--base-ref" "${2:-}"
      BASE_REF="$2"
      shift 2
      ;;
    --head-ref)
      require_value "--head-ref" "${2:-}"
      HEAD_REF="$2"
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

if [[ ! -x "$WATCH_SCRIPT" ]]; then
  fail_run "section07 reopen watch script not found or not executable: $WATCH_SCRIPT"
fi

if [[ -z "$REPO_ROOT" ]]; then
  if ! REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    fail_run "failed to resolve repo root from current working directory"
  fi
fi

if [[ ! -d "$REPO_ROOT" ]]; then
  fail_run "repo root does not exist: $REPO_ROOT"
fi

if ! git -C "$REPO_ROOT" rev-parse --show-toplevel >/dev/null 2>&1; then
  fail_run "failed to resolve repo root: $REPO_ROOT"
fi

cleanup_paths=()
cleanup() {
  local path=""
  for path in "${cleanup_paths[@]}"; do
    rm -f "$path"
  done
}
trap cleanup EXIT

if [[ -z "$REPORT_PATH" ]]; then
  REPORT_PATH="$(mktemp "${TMPDIR:-/tmp}/section07_reopen_report.XXXXXX.txt")"
  cleanup_paths+=("$REPORT_PATH")
fi

if [[ -z "$STATUS_PATH" ]]; then
  STATUS_PATH="$(mktemp "${TMPDIR:-/tmp}/section07_reopen_status.XXXXXX.txt")"
  cleanup_paths+=("$STATUS_PATH")
fi

if ! BASE_SHA="$(git -C "$REPO_ROOT" merge-base "$BASE_REF" "$HEAD_REF" 2>/dev/null)"; then
  fail_run "failed to resolve merge-base for $BASE_REF and $HEAD_REF"
fi

if ! HEAD_SHA="$(git -C "$REPO_ROOT" rev-parse "$HEAD_REF" 2>/dev/null)"; then
  fail_run "failed to resolve head ref $HEAD_REF"
fi

bash "$WATCH_SCRIPT" \
  --repo-root "$REPO_ROOT" \
  --base "$BASE_SHA" \
  --head "$HEAD_SHA" \
  --report-path "$REPORT_PATH" \
  --status-path "$STATUS_PATH"
