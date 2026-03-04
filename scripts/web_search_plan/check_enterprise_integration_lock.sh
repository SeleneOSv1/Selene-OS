#!/usr/bin/env bash
set -euo pipefail

results_file="/tmp/selene_run35_enterprise_lock_$(git rev-parse --short HEAD).tsv"
: > "${results_file}"

run_gate() {
  local gate_name="$1"
  shift
  echo "[GATE] ${gate_name}"
  if "$@"; then
    printf "GATE\t%s\tPASS\n" "${gate_name}" >> "${results_file}"
    echo "[PASS] ${gate_name}"
  else
    printf "GATE\t%s\tFAIL\n" "${gate_name}" >> "${results_file}"
    printf "OVERALL\tFAIL\n" >> "${results_file}"
    echo "[FAIL] ${gate_name}"
    echo "ENTERPRISE_LOCK_OVERALL=FAIL"
    echo "ENTERPRISE_LOCK_RESULTS_FILE=${results_file}"
    exit 1
  fi
}

run_gate "scripts/web_search_plan/check_structured_connectors.sh" scripts/web_search_plan/check_structured_connectors.sh
run_gate "scripts/web_search_plan/check_document_parsing.sh" scripts/web_search_plan/check_document_parsing.sh
run_gate "scripts/web_search_plan/check_analytics_numeric_consensus.sh" scripts/web_search_plan/check_analytics_numeric_consensus.sh
run_gate "scripts/web_search_plan/check_competitive_intel.sh" scripts/web_search_plan/check_competitive_intel.sh
run_gate "scripts/web_search_plan/check_realtime_api_mode.sh" scripts/web_search_plan/check_realtime_api_mode.sh
run_gate "scripts/web_search_plan/check_regulatory_mode.sh" scripts/web_search_plan/check_regulatory_mode.sh
run_gate "scripts/web_search_plan/check_trust_model.sh" scripts/web_search_plan/check_trust_model.sh
run_gate "scripts/web_search_plan/check_multihop_research.sh" scripts/web_search_plan/check_multihop_research.sh
run_gate "scripts/web_search_plan/check_temporal_mode.sh" scripts/web_search_plan/check_temporal_mode.sh
run_gate "scripts/web_search_plan/check_risk_mode.sh" scripts/web_search_plan/check_risk_mode.sh
run_gate "scripts/web_search_plan/check_merge_mode.sh" scripts/web_search_plan/check_merge_mode.sh
run_gate "cargo test -p selene_os web_search_plan::enterprise::enterprise_tests --quiet" cargo test -p selene_os web_search_plan::enterprise::enterprise_tests --quiet

printf "OVERALL\tPASS\n" >> "${results_file}"
echo "ENTERPRISE_LOCK_OVERALL=PASS"
echo "ENTERPRISE_LOCK_RESULTS_FILE=${results_file}"
