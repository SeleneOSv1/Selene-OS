#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
cd "$ROOT"

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "MISSING_TOOL:$1"
    exit 2
  fi
}

require_cmd awk
require_cmd rg
require_cmd grep

extract_function_body() {
  local file="$1"
  local fn="$2"
  awk -v fn="$fn" '
    BEGIN { in_fn=0; depth=0; started=0; }
    $0 ~ ("^fn " fn "\\(") {
      in_fn=1;
      started=1;
    }
    in_fn {
      print $0;
      opens = gsub(/\{/, "{");
      closes = gsub(/\}/, "}");
      depth += opens - closes;
      if (started && depth <= 0) {
        exit;
      }
    }
  ' "$file"
}

FORBIDDEN_REGEX='PH1\.(PATTERN|RLL|GOV|EXPORT|KMS)'
PH1OS_FILE="crates/selene_os/src/ph1os.rs"
OS_DBW_DOC="docs/DB_WIRING/PH1_OS.md"

if [ ! -f "$PH1OS_FILE" ]; then
  echo "MISSING_FILE:$PH1OS_FILE"
  exit 2
fi
if [ ! -f "$OS_DBW_DOC" ]; then
  echo "MISSING_FILE:$OS_DBW_DOC"
  exit 2
fi

always_on_body="$(extract_function_body "$PH1OS_FILE" "expected_always_on_sequence")"
optional_body="$(extract_function_body "$PH1OS_FILE" "turn_optional_sequence")"
forbidden_body="$(extract_function_body "$PH1OS_FILE" "runtime_forbidden_engine_ids")"

if [ -z "$always_on_body" ] || [ -z "$optional_body" ] || [ -z "$forbidden_body" ]; then
  echo "INVALID_FUNCTION_EXTRACTION:expected_always_on_sequence/turn_optional_sequence/runtime_forbidden_engine_ids"
  exit 3
fi

if printf '%s\n' "$always_on_body" | rg -n "$FORBIDDEN_REGEX" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:forbidden engine present in expected_always_on_sequence"
  exit 1
fi
if printf '%s\n' "$optional_body" | rg -n "$FORBIDDEN_REGEX" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:forbidden engine present in turn_optional_sequence"
  exit 1
fi

for engine in PH1.PATTERN PH1.RLL PH1.GOV PH1.EXPORT PH1.KMS; do
  if ! printf '%s\n' "$forbidden_body" | grep -F "\"$engine\"" >/dev/null 2>&1; then
    echo "RUNTIME_BOUNDARY_FAIL:runtime_forbidden_engine_ids missing $engine"
    exit 1
  fi
done

if ! rg -n "voice path ALWAYS_ON order lock" "$OS_DBW_DOC" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:missing voice ALWAYS_ON lock in docs/DB_WIRING/PH1_OS.md"
  exit 1
fi
if ! rg -n "text path ALWAYS_ON order lock" "$OS_DBW_DOC" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:missing text ALWAYS_ON lock in docs/DB_WIRING/PH1_OS.md"
  exit 1
fi
if ! rg -n "TURN_OPTIONAL ordering" "$OS_DBW_DOC" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:missing TURN_OPTIONAL ordering lock in docs/DB_WIRING/PH1_OS.md"
  exit 1
fi
if ! rg -n 'OFFLINE_ONLY engines \(`PH1\.PATTERN`, `PH1\.RLL`\)' "$OS_DBW_DOC" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:PH1.PATTERN/PH1.RLL OFFLINE_ONLY lock missing in docs/DB_WIRING/PH1_OS.md"
  exit 1
fi
if ! rg -n 'control-plane engines \(`PH1\.GOV`, `PH1\.EXPORT`, `PH1\.KMS`\)' "$OS_DBW_DOC" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:PH1.GOV/PH1.EXPORT/PH1.KMS control-plane lock missing in docs/DB_WIRING/PH1_OS.md"
  exit 1
fi

echo "CHECK_OK runtime_boundary_guards=pass"
