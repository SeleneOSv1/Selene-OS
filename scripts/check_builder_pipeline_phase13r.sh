#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

fail() {
  echo "CHECK_FAIL:$1" >&2
  exit 1
}

check_contains() {
  local file="$1"
  local pattern="$2"
  local code="$3"
  if ! rg -n --fixed-strings "${pattern}" "${file}" >/dev/null 2>&1; then
    fail "${code}"
  fi
}

if [[ ! -x scripts/check_builder_production_soak_watchdog.sh ]]; then
  fail "missing_executable scripts/check_builder_production_soak_watchdog.sh"
fi

check_contains "scripts/check_builder_production_soak_watchdog.sh" "stage}\" != \"PRODUCTION\"" "missing_production_stage_gate"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "status}\" != \"COMPLETED\"" "missing_completed_status_gate"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "approval_status_not_approved" "missing_approval_gate"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "missing_production_judge_result" "missing_production_judge_presence_gate"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "production_judge_not_accept" "missing_production_judge_action_gate"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "REQUIRED_RELEASE_STATE_ID" "missing_scoped_judge_export_binding"
check_contains "scripts/check_builder_production_soak_watchdog.sh" "check_builder_stage2_promotion_gate.sh" "missing_soak_metrics_gate"

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13.30 Production Soak Watchdog (Fresh Production Judge Required)" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.30"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_production_soak_watchdog.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "scripts/selene_design_readiness_audit.sh" "1AK) BUILDER PIPELINE PHASE13-R PRODUCTION-SOAK WATCHDOG GUARDRAIL CHECK" "missing_readiness_section_1AK"
check_contains "scripts/selene_design_readiness_audit.sh" "1AL) BUILDER PRODUCTION SOAK WATCHDOG (OPTIONAL ENFORCED)" "missing_readiness_section_1AL"

echo "CHECK_OK builder_pipeline_phase13r=pass"
