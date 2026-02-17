#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail() {
  echo "CHECK_FAIL builder_pipeline_phase13l: $1" >&2
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

check_file "scripts/check_builder_release_hard_gate.sh"
check_file "scripts/check_builder_e2e_gate_flow.sh"
check_file "scripts/selene_design_readiness_audit.sh"
check_file "docs/fixtures/builder_learning_bridge_template.env"
check_file "docs/fixtures/builder_permission_template.env"
check_file "docs/fixtures/builder_change_brief_template.md"

check_contains "scripts/check_builder_release_hard_gate.sh" "AUTO_SYNC_DECISION_FILES=1" "hard gate missing auto-sync enforcement"
check_contains "scripts/check_builder_release_hard_gate.sh" "STAGE_GATE_MODE=live" "hard gate missing live-stage enforcement"
check_contains "scripts/check_builder_release_hard_gate.sh" "check_builder_e2e_gate_flow.sh" "hard gate missing e2e chain call"
check_contains "scripts/selene_design_readiness_audit.sh" "ENFORCE_BUILDER_RELEASE_HARD_GATE" "readiness missing hard-gate enforcement switch"
check_contains "scripts/selene_design_readiness_audit.sh" "check_builder_release_hard_gate.sh" "readiness missing hard-gate call"

tmp_dir="$(mktemp -d /tmp/builder_phase13l_XXXXXX)"
tmp_learning_env="${tmp_dir}/builder_learning_bridge.env"
tmp_permission_env="${tmp_dir}/builder_permission.env"
tmp_brief="${tmp_dir}/builder_change_brief.md"
tmp_code_file="${tmp_dir}/builder_code_decision.env"
tmp_launch_file="${tmp_dir}/builder_launch_decision.env"
tmp_out="${tmp_dir}/phase13l_out.log"

cp docs/fixtures/builder_learning_bridge_template.env "${tmp_learning_env}"
cp docs/fixtures/builder_permission_template.env "${tmp_permission_env}"
cp docs/fixtures/builder_change_brief_template.md "${tmp_brief}"

cat >"${tmp_code_file}" <<EOF
PHASE=code
DECISION=approve
BCAST_ID=BCAST_CODE_L
DECISION_REF=DECISION_CODE_L
REFRESH_DAILY_REVIEW=1
EOF

cat >"${tmp_launch_file}" <<EOF
PHASE=launch
DECISION=approve
BCAST_ID=BCAST_LAUNCH_L
DECISION_REF=DECISION_LAUNCH_L
REFRESH_DAILY_REVIEW=1
EOF

if LEARNING_ENV_FILE="${tmp_learning_env}" \
  PERMISSION_ENV_FILE="${tmp_permission_env}" \
  BRIEF_FILE="${tmp_brief}" \
  CODE_DECISION_FILE="${tmp_code_file}" \
  LAUNCH_DECISION_FILE="${tmp_launch_file}" \
  bash scripts/check_builder_release_hard_gate.sh >"${tmp_out}" 2>&1; then
  rg -q "CHECK_OK builder_release_hard_gate=pass" "${tmp_out}" || \
    fail "hard gate succeeded but success marker missing"
else
  if ! rg -q "NO_CANARY_TELEMETRY|STALE_CANARY_TELEMETRY" "${tmp_out}"; then
    fail "hard gate failed without expected live telemetry fail-closed reason"
  fi
fi

rm -rf "${tmp_dir}"
echo "CHECK_OK builder_pipeline_phase13l=pass"
