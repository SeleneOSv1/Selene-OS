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

if [[ ! -x scripts/check_builder_production_soak_runner.sh ]]; then
  fail "missing_executable scripts/check_builder_production_soak_runner.sh"
fi

check_contains "scripts/check_builder_production_soak_runner.sh" 'FAIL_CLOSED="${FAIL_CLOSED:-1}"' "missing_fail_closed_default"
check_contains "scripts/check_builder_production_soak_runner.sh" 'ALERT_ON_FAIL="${ALERT_ON_FAIL:-1}"' "missing_alert_default"
check_contains "scripts/check_builder_production_soak_runner.sh" "PRODUCTION_SOAK_STALE_TELEMETRY" "missing_stale_alert_classification"
check_contains "scripts/check_builder_production_soak_runner.sh" 'WATCHDOG_CMD="${WATCHDOG_CMD:-scripts/check_builder_production_soak_watchdog.sh}"' "missing_watchdog_binding"
check_contains "scripts/check_builder_production_soak_runner.sh" 'RUN_MODE="${RUN_MODE:-once}"' "missing_run_mode_contract"

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13.31 Production Soak Recurring Runner (Fail-Closed Alerting)" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.31"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "check_builder_production_soak_runner.sh" "missing_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "scripts/selene_design_readiness_audit.sh" "1AM) BUILDER PIPELINE PHASE13-S PRODUCTION-SOAK RUNNER GUARDRAIL CHECK" "missing_readiness_section_1AM"
check_contains "scripts/selene_design_readiness_audit.sh" "1AN) BUILDER PRODUCTION SOAK RUNNER (OPTIONAL ENFORCED, ONCE MODE)" "missing_readiness_section_1AN"

echo "CHECK_OK builder_pipeline_phase13s=pass"
