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

require_cmd rg

fail() {
  echo "UNDERSTANDING_PRECEDENCE_FAIL:$1"
  exit 1
}

require_match() {
  local pattern="$1"
  local file="$2"
  local msg="$3"
  if ! rg -n "$pattern" "$file" >/dev/null 2>&1; then
    fail "$msg ($file)"
  fi
}

KERNEL_PH1OS="crates/selene_kernel_contracts/src/ph1os.rs"
ENGINE_PH1OS="crates/selene_engines/src/ph1os.rs"
RUNTIME_PH1OS="crates/selene_os/src/ph1os.rs"
MAP_DOC="docs/06_ENGINE_MAP.md"
DBW_DOC="docs/DB_WIRING/PH1_OS.md"
ECM_DOC="docs/ECM/PH1_OS.md"

require_match "pub const OS_CLARIFY_OWNER_ENGINE_ID: &str = \"PH1\\.NLP\"" "$KERNEL_PH1OS" \
  "kernel contract must lock clarify owner to PH1.NLP"
require_match "clarify_owner_engine_id: Option<String>" "$KERNEL_PH1OS" \
  "kernel decision request must expose clarify_owner_engine_id"
require_match "must be PH1\\.NLP when clarify_required=true" "$KERNEL_PH1OS" \
  "kernel contract must fail closed on clarify owner drift"
require_match "must be omitted when clarify_required=false" "$KERNEL_PH1OS" \
  "kernel contract must block stale clarify owner metadata"

require_match "OS_FAIL_CLARIFY_OWNER_PRECEDENCE" "$ENGINE_PH1OS" \
  "PH1.OS engine runtime must define clarify owner precedence reason code"
require_match "clarify_owner_engine_id\\.as_deref\\(\\) != Some\\(OS_CLARIFY_OWNER_ENGINE_ID\\)" "$ENGINE_PH1OS" \
  "PH1.OS engine runtime must enforce clarify owner precedence check"

require_match "PH1_OS_TOPLEVEL_CLARIFY_OWNER_INVALID" "$RUNTIME_PH1OS" \
  "top-level wiring must expose clarify owner fail-closed reason code"
require_match "PH1_OS_TOPLEVEL_OPTIONAL_POLICY_BLOCK" "$RUNTIME_PH1OS" \
  "top-level wiring must expose optional assist policy block reason code"
require_match "fn optional_engine_allowed_by_policy" "$RUNTIME_PH1OS" \
  "top-level wiring must define optional assist policy gate"
require_match "\"PH1\\.PRUNE\" => input\\.clarify_required" "$RUNTIME_PH1OS" \
  "top-level wiring must gate PH1.PRUNE on clarify_required"
require_match "\"PH1\\.DIAG\" =>" "$RUNTIME_PH1OS" \
  "top-level wiring must gate PH1.DIAG on deterministic posture"

require_match 'Clarify owner lock: only `PH1\.NLP` may own clarify decisions' "$MAP_DOC" \
  "engine map must document single clarify owner"
require_match "Optional-assist policy bounds \\(fail-closed\\)" "$MAP_DOC" \
  "engine map must document optional assist policy bounds"

require_match 'One clarify owner: if `clarify_required=true`, `clarify_owner_engine_id` must be `PH1\.NLP`' "$DBW_DOC" \
  "DB wiring must lock clarify owner precedence"
require_match "AT-OS-17: clarify owner precedence is fail-closed" "$DBW_DOC" \
  "DB wiring must include clarify owner acceptance test"
require_match "AT-OS-18: optional understanding-assist policy blocks invalid clarify-loop requests" "$DBW_DOC" \
  "DB wiring must include optional assist policy acceptance test"

require_match "Clarify owner rule:" "$ECM_DOC" \
  "ECM must include clarify owner rule"
require_match "Optional understanding-assist policy rule:" "$ECM_DOC" \
  "ECM must include optional assist policy rule"

echo "CHECK_OK understanding_clarify_precedence=pass"
