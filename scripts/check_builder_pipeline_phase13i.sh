#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13i: $1" >&2
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
check_file "docs/fixtures/builder_permission_decision_template.env"
check_file "docs/fixtures/builder_permission_template.env"
check_file "docs/fixtures/builder_change_brief_template.md"
check_file "scripts/check_builder_human_permission_gate.sh"
check_file "scripts/check_builder_e2e_gate_flow.sh"

check_contains "scripts/apply_builder_permission_decision.sh" "DECISION_FILE" "missing decision-file ingest support"
check_contains "scripts/apply_builder_permission_decision.sh" "load_decision_file" "missing decision-file loader"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_builder_pipeline_phase13i.sh" "e2e chain missing phase13i precheck"
check_contains "docs/fixtures/builder_permission_decision_template.env" "^PHASE=" "decision template missing PHASE"
check_contains "docs/fixtures/builder_permission_decision_template.env" "^DECISION=" "decision template missing DECISION"

tmp_env="$(mktemp /tmp/builder_permission_i_XXXXXX.env)"
tmp_brief="$(mktemp /tmp/builder_change_brief_i_XXXXXX.md)"
tmp_code_file="$(mktemp /tmp/builder_decision_code_i_XXXXXX.env)"
tmp_launch_file="$(mktemp /tmp/builder_decision_launch_i_XXXXXX.env)"
tmp_pending_env="$(mktemp /tmp/builder_permission_i_pending_XXXXXX.env)"
tmp_pending_file="$(mktemp /tmp/builder_decision_pending_i_XXXXXX.env)"
tmp_out="$(mktemp /tmp/builder_permission_i_out_XXXXXX.log)"

cp docs/fixtures/builder_permission_template.env "$tmp_env"
cp docs/fixtures/builder_change_brief_template.md "$tmp_brief"

cat >"$tmp_code_file" <<EOF
PHASE=code
DECISION=approve
BCAST_ID=BCAST_CODE_I
DECISION_REF=DECISION_CODE_I
REFRESH_DAILY_REVIEW=1
EOF

DECISION_FILE="$tmp_code_file" ENV_FILE="$tmp_env" \
  bash scripts/apply_builder_permission_decision.sh >/dev/null
ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh code >/dev/null

cat >"$tmp_launch_file" <<EOF
PHASE=launch
DECISION=approve
BCAST_ID=BCAST_LAUNCH_I
DECISION_REF=DECISION_LAUNCH_I
REFRESH_DAILY_REVIEW=1
EOF

DECISION_FILE="$tmp_launch_file" ENV_FILE="$tmp_env" \
  bash scripts/apply_builder_permission_decision.sh >/dev/null
ENV_FILE="$tmp_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh launch >/dev/null

cp docs/fixtures/builder_permission_template.env "$tmp_pending_env"
cat >"$tmp_pending_file" <<EOF
PHASE=code
DECISION=pending
REMINDER_REF=REMINDER_PENDING_I
BUSY_MODE_OVERRIDE=1
REFRESH_DAILY_REVIEW=1
EOF

DECISION_FILE="$tmp_pending_file" ENV_FILE="$tmp_pending_env" \
  bash scripts/apply_builder_permission_decision.sh >/dev/null
if ENV_FILE="$tmp_pending_env" BRIEF_FILE="$tmp_brief" \
  bash scripts/check_builder_human_permission_gate.sh code >"$tmp_out" 2>&1; then
  fail "pending decision file unexpectedly passed code gate"
fi
rg -q "approval_pending_reminder_scheduled" "$tmp_out" || \
  fail "pending decision file did not produce expected reminder-pending block"

rm -f "$tmp_env" "$tmp_brief" "$tmp_code_file" "$tmp_launch_file" \
  "$tmp_pending_env" "$tmp_pending_file" "$tmp_out"

echo "CHECK_OK builder_pipeline_phase13i=pass"
