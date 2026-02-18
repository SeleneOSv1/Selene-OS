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
MAP_FILE="docs/06_ENGINE_MAP.md"

if [ ! -f "$PH1OS_FILE" ]; then
  echo "MISSING_FILE:$PH1OS_FILE"
  exit 2
fi
if [ ! -f "$MAP_FILE" ]; then
  echo "MISSING_FILE:$MAP_FILE"
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

always_on_line="$(rg -n "^- ALWAYS_ON:" "$MAP_FILE" | head -n 1 || true)"
optional_line="$(rg -n "^- TURN_OPTIONAL:" "$MAP_FILE" | head -n 1 || true)"
offline_line="$(rg -n "^- OFFLINE_ONLY:" "$MAP_FILE" | head -n 1 || true)"
enterprise_line="$(rg -n "^- ENTERPRISE_SUPPORT:" "$MAP_FILE" | head -n 1 || true)"

if [ -z "$always_on_line" ] || [ -z "$optional_line" ] || [ -z "$offline_line" ] || [ -z "$enterprise_line" ]; then
  echo "RUNTIME_BOUNDARY_FAIL:missing runtime class declaration in docs/06_ENGINE_MAP.md"
  exit 1
fi

if printf '%s\n' "$always_on_line" | rg -n "$FORBIDDEN_REGEX" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:forbidden engine present in docs ALWAYS_ON"
  exit 1
fi
if printf '%s\n' "$optional_line" | rg -n "$FORBIDDEN_REGEX" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:forbidden engine present in docs TURN_OPTIONAL"
  exit 1
fi

if ! printf '%s\n' "$offline_line" | grep -F "PH1.PATTERN" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:PH1.PATTERN missing from docs OFFLINE_ONLY"
  exit 1
fi
if ! printf '%s\n' "$offline_line" | grep -F "PH1.RLL" >/dev/null 2>&1; then
  echo "RUNTIME_BOUNDARY_FAIL:PH1.RLL missing from docs OFFLINE_ONLY"
  exit 1
fi

for engine in PH1.GOV PH1.EXPORT PH1.KMS; do
  if ! printf '%s\n' "$enterprise_line" | grep -F "$engine" >/dev/null 2>&1; then
    echo "RUNTIME_BOUNDARY_FAIL:$engine missing from docs ENTERPRISE_SUPPORT"
    exit 1
  fi
done

echo "CHECK_OK runtime_boundary_guards=pass"
