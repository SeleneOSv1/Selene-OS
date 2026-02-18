#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13k: $1" >&2
  exit 1
}

check_file() {
  local file="$1"
  [ -f "$file" ] || fail "missing file: $file"
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local message="$3"
  rg -n "$pattern" "$file" >/dev/null || fail "$message ($file)"
}

check_file "scripts/sync_builder_permission_from_decision_files.sh"
check_file "scripts/apply_builder_permission_decision.sh"
check_file "scripts/check_builder_human_permission_gate.sh"
check_file "scripts/check_builder_e2e_gate_flow.sh"
check_file "docs/fixtures/builder_permission_template.env"
check_file "docs/fixtures/builder_change_brief_template.md"

check_contains "scripts/sync_builder_permission_from_decision_files.sh" "SYNC_OK builder_permission_decision_files" "missing sync success marker"
check_contains "scripts/sync_builder_permission_from_decision_files.sh" "apply_builder_permission_decision.sh" "sync script missing decision apply call"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_builder_pipeline_phase13k.sh" "e2e chain missing phase13k precheck"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "AUTO_SYNC_DECISION_FILES" "e2e chain missing decision file auto-sync toggle"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "sync_builder_permission_from_decision_files.sh" "e2e chain missing sync script call"

tmp_env="$(mktemp /tmp/builder_permission_k_XXXXXX.env)"
tmp_brief="$(mktemp /tmp/builder_change_brief_k_XXXXXX.md)"
tmp_code_file="$(mktemp /tmp/builder_decision_code_k_XXXXXX.env)"
tmp_launch_file="$(mktemp /tmp/builder_decision_launch_k_XXXXXX.env)"
tmp_out="$(mktemp /tmp/builder_permission_k_out_XXXXXX.log)"

cp docs/fixtures/builder_permission_template.env "$tmp_env"
cp docs/fixtures/builder_change_brief_template.md "$tmp_brief"

cat >"$tmp_code_file" <<EOF
PHASE=code
DECISION=approve
BCAST_ID=BCAST_CODE_K
DECISION_REF=DECISION_CODE_K
REFRESH_DAILY_REVIEW=1
EOF

cat >"$tmp_launch_file" <<EOF
PHASE=launch
DECISION=approve
BCAST_ID=BCAST_LAUNCH_K
DECISION_REF=DECISION_LAUNCH_K
REFRESH_DAILY_REVIEW=1
EOF

ENV_FILE="$tmp_env" \
CODE_DECISION_FILE="$tmp_code_file" \
LAUNCH_DECISION_FILE="$tmp_launch_file" \
  bash scripts/sync_builder_permission_from_decision_files.sh >/dev/null

ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh code >/dev/null
ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh launch >/dev/null

if ENV_FILE="$tmp_env" CODE_DECISION_FILE="/tmp/does_not_exist_k.env" LAUNCH_DECISION_FILE="$tmp_launch_file" \
  bash scripts/sync_builder_permission_from_decision_files.sh >"$tmp_out" 2>&1; then
  fail "missing required code decision file unexpectedly passed"
fi
rg -q "missing_decision_file phase=code" "$tmp_out" || \
  fail "missing-file fail-closed path not enforced for code phase"

rm -f "$tmp_env" "$tmp_brief" "$tmp_code_file" "$tmp_launch_file" "$tmp_out"

echo "CHECK_OK builder_pipeline_phase13k=pass"
