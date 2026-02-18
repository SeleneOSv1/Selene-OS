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
  echo "LEARNING_OWNERSHIP_FAIL:$1"
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

require_absent() {
  local pattern="$1"
  local file="$2"
  local msg="$3"
  if rg -n "$pattern" "$file" >/dev/null 2>&1; then
    fail "$msg ($file)"
  fi
}

MAP_DOC="docs/06_ENGINE_MAP.md"
REGISTRY_DOC="docs/07_ENGINE_REGISTRY.md"
OWNERSHIP_DOC="docs/10_DB_OWNERSHIP_MATRIX.md"
GROUP_DBW_DOC="docs/DB_WIRING/PH1_LEARN_FEEDBACK_KNOW.md"
GROUP_ECM_DOC="docs/ECM/PH1_LEARN_FEEDBACK_KNOW.md"
STORAGE_FILE="crates/selene_storage/src/ph1f.rs"
STORAGE_TEST="crates/selene_storage/tests/ph1_learn_feedback_know/db_wiring.rs"

require_match 'storage grouping pointer \(non-runtime\): `PH1\.LEARN_FEEDBACK_KNOW`' "$MAP_DOC" \
  "engine map must keep PH1.LEARN_FEEDBACK_KNOW as non-runtime storage pointer only"

require_absent "^\\| PH1\\.LEARN_FEEDBACK_KNOW \\|" "$REGISTRY_DOC" \
  "engine registry must not list PH1.LEARN_FEEDBACK_KNOW as a runtime engine row"
require_match "Storage grouping \\(non-runtime; not a callable engine row\\):" "$REGISTRY_DOC" \
  "engine registry must include explicit storage-group note"
require_match '`PH1\.FEEDBACK` \(feedback signal audit rows\)' "$REGISTRY_DOC" \
  "engine registry must lock PH1.FEEDBACK as runtime owner"
require_match '`PH1\.LEARN` \(adaptation artifact rows\)' "$REGISTRY_DOC" \
  "engine registry must lock PH1.LEARN as runtime owner"
require_match '`PH1\.KNOW` \(tenant vocabulary/pronunciation artifact rows\)' "$REGISTRY_DOC" \
  "engine registry must lock PH1.KNOW as runtime owner"

require_absent "^\\| PH1\\.LEARN_FEEDBACK_KNOW \\|" "$OWNERSHIP_DOC" \
  "DB ownership matrix must not keep PH1.LEARN_FEEDBACK_KNOW as runtime writer row"
require_match '^\| PH1\.FEEDBACK \| `audit_events` feedback signal rows only' "$OWNERSHIP_DOC" \
  "DB ownership matrix must lock PH1.FEEDBACK feedback-row ownership"
require_match '^\| PH1\.LEARN \| `artifacts_ledger` adaptation artifact rows only' "$OWNERSHIP_DOC" \
  "DB ownership matrix must lock PH1.LEARN adaptation artifact ownership"
require_match '^\| PH1\.KNOW \| `artifacts_ledger` tenant dictionary/pronunciation artifact rows only' "$OWNERSHIP_DOC" \
  "DB ownership matrix must lock PH1.KNOW dictionary artifact ownership"

require_match "Storage Grouping Only" "$GROUP_DBW_DOC" \
  "group DB wiring doc must declare storage-group-only scope"
require_match "this file is not a callable runtime engine contract" "$GROUP_DBW_DOC" \
  "group DB wiring doc must block runtime-engine interpretation"
require_match "single-writer and fail-closed" "$GROUP_DBW_DOC" \
  "group DB wiring doc must declare single-writer fail-closed ownership"

require_match "not a runtime callable engine" "$GROUP_ECM_DOC" \
  "group ECM doc must block runtime callable interpretation"
require_match 'runtime callable contracts live in `PH1\.FEEDBACK`, `PH1\.LEARN`, and `PH1\.KNOW` docs' "$GROUP_ECM_DOC" \
  "group ECM doc must point to split runtime owners"

require_match "fn validate_ph1learn_artifact_type" "$STORAGE_FILE" \
  "storage layer must enforce PH1.LEARN artifact ownership type boundary"
require_match "must be STT_ROUTING_POLICY_PACK, STT_ADAPTATION_PROFILE, or TTS_ROUTING_POLICY_PACK" "$STORAGE_FILE" \
  "PH1.LEARN artifact ownership boundary message must be explicit"
require_match "must be STT_VOCAB_PACK or TTS_PRONUNCIATION_PACK" "$STORAGE_FILE" \
  "PH1.KNOW artifact ownership boundary message must remain explicit"

require_match "fn at_learn_db_05_single_writer_artifact_types_enforced" "$STORAGE_TEST" \
  "storage tests must include single-writer artifact ownership invariant"

echo "CHECK_OK learning_ownership_boundaries=pass"
