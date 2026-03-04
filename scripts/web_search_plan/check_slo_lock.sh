#!/usr/bin/env bash
set -euo pipefail

head_commit="$(git rev-parse HEAD)"
results_file="/tmp/selene_run30_slo_lock_${head_commit}.tsv"
latest_file="/tmp/selene_run30_slo_lock_latest.tsv"

: > "${results_file}"

record_fail_and_exit() {
  local key="$1"
  local detail="$2"
  printf "SLO\t%s\tFAIL\t%s\n" "${key}" "${detail}" >> "${results_file}"
  printf "OVERALL\tFAIL\n" >> "${results_file}"
  cp "${results_file}" "${latest_file}"
  echo "SLO_LOCK_FAIL key=${key} detail=${detail}"
  echo "SLO_LOCK_RESULTS_FILE=${results_file}"
  exit 1
}

echo "[SLO] citation/refusal gates via check_quality_gates.sh"
if scripts/web_search_plan/check_quality_gates.sh; then
  printf "SLO\tcitation_coverage\tPASS\trequired=1.0_answer_cases\n" >> "${results_file}"
  printf "SLO\trefusal_correctness\tPASS\trequired=all_refusal_cases_pass\n" >> "${results_file}"
  echo "[PASS] citation_coverage"
  echo "[PASS] refusal_correctness"
else
  record_fail_and_exit "citation_coverage" "quality_gate_failed"
fi

echo "[SLO] freshness compliance via check_realtime_api_mode.sh"
if scripts/web_search_plan/check_realtime_api_mode.sh; then
  printf "SLO\tfreshness_compliance\tPASS\trequired=stale_refusal_enforced\n" >> "${results_file}"
  echo "[PASS] freshness_compliance"
else
  record_fail_and_exit "freshness_compliance" "realtime_gate_failed"
fi

echo "[SLO] determinism replay via check_replay_harness.sh"
if scripts/web_search_plan/check_replay_harness.sh; then
  printf "SLO\tdeterminism_replay\tPASS\trequired=replay_snapshot_match\n" >> "${results_file}"
  echo "[PASS] determinism_replay"
else
  record_fail_and_exit "determinism_replay" "replay_gate_failed"
fi

printf "OVERALL\tPASS\n" >> "${results_file}"
cp "${results_file}" "${latest_file}"
echo "SLO_LOCK_PASS"
echo "SLO_LOCK_RESULTS_FILE=${results_file}"
