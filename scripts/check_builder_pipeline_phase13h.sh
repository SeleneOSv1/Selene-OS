#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13h: $1" >&2
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

check_file "scripts/apply_builder_permission_decision.sh"
check_file "scripts/check_builder_human_permission_gate.sh"
check_file "docs/fixtures/builder_permission_template.env"
check_file "docs/fixtures/builder_change_brief_template.md"
check_file "scripts/check_builder_e2e_gate_flow.sh"

check_contains "scripts/apply_builder_permission_decision.sh" "APPLY_OK builder_permission_decision" "missing apply success marker"
check_contains "scripts/apply_builder_permission_decision.sh" "code\\|launch" "missing phase validation"
check_contains "scripts/apply_builder_permission_decision.sh" "approve\\|deny\\|pending" "missing decision validation"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_builder_pipeline_phase13h.sh" "e2e chain missing phase13h precheck"

tmp_env="$(mktemp /tmp/builder_permission_h_XXXXXX.env)"
tmp_brief="$(mktemp /tmp/builder_change_brief_h_XXXXXX.md)"
tmp_out="$(mktemp /tmp/builder_permission_h_out_XXXXXX.log)"
cp docs/fixtures/builder_permission_template.env "$tmp_env"
cp docs/fixtures/builder_change_brief_template.md "$tmp_brief"

BCAST_ID=BCAST_CODE_H DECISION_REF=DECISION_CODE_H ENV_FILE="$tmp_env" \
  bash scripts/apply_builder_permission_decision.sh code approve >/dev/null
ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh code >/dev/null

BCAST_ID=BCAST_LAUNCH_H DECISION_REF=DECISION_LAUNCH_H ENV_FILE="$tmp_env" \
  bash scripts/apply_builder_permission_decision.sh launch approve >/dev/null
ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh launch >/dev/null

tmp_env_pending="$(mktemp /tmp/builder_permission_h_pending_XXXXXX.env)"
cp docs/fixtures/builder_permission_template.env "$tmp_env_pending"
REMINDER_REF=REMINDER_PENDING_H ENV_FILE="$tmp_env_pending" \
  bash scripts/apply_builder_permission_decision.sh code pending >/dev/null
if ENV_FILE="$tmp_env_pending" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh code >"$tmp_out" 2>&1; then
  fail "pending decision unexpectedly passed code gate"
fi
rg -q "approval_pending_reminder_scheduled" "$tmp_out" || \
  fail "pending decision did not produce expected reminder-pending block"

rm -f "$tmp_env" "$tmp_env_pending" "$tmp_brief" "$tmp_out"
echo "CHECK_OK builder_pipeline_phase13h=pass"
