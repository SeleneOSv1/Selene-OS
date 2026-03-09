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

if [[ ! -x scripts/check_ph1_os_ocr_eval_gate.sh ]]; then
  fail "missing_executable scripts/check_ph1_os_ocr_eval_gate.sh"
fi
if [[ ! -f docs/fixtures/ph1_os_ocr_eval_snapshot.csv ]]; then
  fail "missing_fixture docs/fixtures/ph1_os_ocr_eval_snapshot.csv"
fi

check_contains "scripts/check_builder_e2e_gate_flow.sh" "OCR_EVAL_FIXTURE_CSV" "missing_fixture_env_binding"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "OCR_EVAL_OUTPUT_CSV" "missing_live_env_binding"
check_contains "scripts/check_builder_e2e_gate_flow.sh" "check_ph1_os_ocr_eval_gate.sh" "missing_eval_gate_call_from_e2e"
check_contains "scripts/check_builder_release_hard_gate.sh" "OCR_EVAL_OUTPUT_CSV" "missing_eval_output_binding_from_hard_gate"
check_contains "scripts/selene_design_readiness_audit.sh" "1AQ) BUILDER PIPELINE PHASE13-U OCR EVAL-GATE GUARDRAIL CHECK" "missing_readiness_section_1AQ"
check_contains "scripts/selene_design_readiness_audit.sh" "1AR) PH1.OS OCR BENCHMARK/EVAL RELEASE GATE (OPTIONAL ENFORCED)" "missing_readiness_section_1AR"
check_contains "scripts/selene_design_readiness_audit.sh" "./scripts/check_builder_pipeline_phase13u.sh" "missing_readiness_phase13u_call"
check_contains "scripts/selene_design_readiness_audit.sh" "./scripts/check_ph1_os_ocr_eval_gate.sh" "missing_readiness_ocr_eval_call"

echo "CHECK_OK builder_pipeline_phase13u=pass"
