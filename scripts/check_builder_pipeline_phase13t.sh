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

for f in \
  scripts/setup_builder_production_soak_launchd.sh \
  scripts/status_builder_production_soak_launchd.sh \
  scripts/remove_builder_production_soak_launchd.sh
do
  if [[ ! -x "${f}" ]]; then
    fail "missing_executable ${f}"
  fi
done

check_contains "scripts/setup_builder_production_soak_launchd.sh" 'LABEL="${LABEL:-com.selene.builder.production_soak_runner}"' "missing_label_default"
check_contains "scripts/setup_builder_production_soak_launchd.sh" 'RUNTIME_ROOT="${RUNTIME_ROOT:-${HOME}/.selene_automation/production_soak}"' "missing_runtime_root_default"
check_contains "scripts/setup_builder_production_soak_launchd.sh" 'RUN_MODE=once FAIL_CLOSED=1 ALERT_ON_FAIL=1' "missing_fail_closed_runner_binding"
check_contains "scripts/setup_builder_production_soak_launchd.sh" "SELENE_ROOT='\${RUNTIME_ROOT}'" "missing_runtime_selene_root_binding"
check_contains "scripts/setup_builder_production_soak_launchd.sh" 'launchctl bootstrap' "missing_launchctl_bootstrap"
check_contains "scripts/setup_builder_production_soak_launchd.sh" 'launchctl kickstart -k' "missing_launchctl_kickstart"
check_contains "scripts/status_builder_production_soak_launchd.sh" 'builder_production_soak_launchd_status=loaded' "missing_loaded_status_signal"
check_contains "scripts/remove_builder_production_soak_launchd.sh" 'builder_production_soak_launchd_remove=pass' "missing_remove_signal"

check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "### 13.32 Production Soak Mac Automation (launchd)" "missing_section docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md#13.32"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "setup_builder_production_soak_launchd.sh" "missing_setup_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "status_builder_production_soak_launchd.sh" "missing_status_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md" "remove_builder_production_soak_launchd.sh" "missing_remove_command_reference docs/34_ENGINE_CLOSURE_EXECUTION_PLAN.md"
check_contains "scripts/selene_design_readiness_audit.sh" "1AO) BUILDER PIPELINE PHASE13-T PRODUCTION-SOAK AUTOMATION GUARDRAIL CHECK" "missing_readiness_section_1AO"
check_contains "scripts/selene_design_readiness_audit.sh" "1AP) BUILDER PRODUCTION SOAK AUTOMATION STATUS (OPTIONAL ENFORCED)" "missing_readiness_section_1AP"

echo "CHECK_OK builder_pipeline_phase13t=pass"
