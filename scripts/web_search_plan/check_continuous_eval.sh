#!/usr/bin/env bash
set -euo pipefail

scripts/web_search_plan/check_replay_harness.sh
scripts/web_search_plan/check_quality_gates.sh

cargo test -p selene_os web_search_plan::eval::eval_tests --quiet

head_commit="$(git rev-parse HEAD)"
timestamp_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
date_tag="$(date -u +"%Y%m%dT%H%M%SZ")"
output_dir="docs/web_search_plan/eval/reports"

cmd_output="$(
  cargo run -p selene_os --bin web_search_eval_report --quiet -- \
    --head-commit "${head_commit}" \
    --timestamp-utc "${timestamp_utc}" \
    --date-tag "${date_tag}" \
    --output-dir "${output_dir}"
)"

report_file="$(printf '%s\n' "${cmd_output}" | rg '^EVAL_REPORT_FILE=' | tail -n 1 | sed 's/^EVAL_REPORT_FILE=//')"
overall="$(printf '%s\n' "${cmd_output}" | rg '^EVAL_OVERALL=' | tail -n 1 | sed 's/^EVAL_OVERALL=//')"
failing_case_ids="$(printf '%s\n' "${cmd_output}" | rg '^EVAL_FAILING_CASE_IDS=' | tail -n 1 | sed 's/^EVAL_FAILING_CASE_IDS=//')"

if [[ -z "${report_file}" ]]; then
  echo "CONTINUOUS_EVAL_FAIL missing_report_file"
  printf '%s\n' "${cmd_output}"
  exit 1
fi
if [[ ! -f "${report_file}" ]]; then
  echo "CONTINUOUS_EVAL_FAIL report_file_not_found=${report_file}"
  exit 1
fi
if [[ -z "${overall}" ]]; then
  echo "CONTINUOUS_EVAL_FAIL missing_overall_status"
  printf '%s\n' "${cmd_output}"
  exit 1
fi

if [[ "${overall}" != "PASS" ]]; then
  echo "CONTINUOUS_EVAL_FAIL failing_case_ids=${failing_case_ids}"
  exit 1
fi

echo "CONTINUOUS_EVAL_PASS report_file=${report_file} failing_case_ids=${failing_case_ids}"
