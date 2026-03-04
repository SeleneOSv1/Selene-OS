#!/usr/bin/env bash
set -euo pipefail

trace_file="docs/web_search_plan/TRACE_MATRIX_SKELETON.md"

if [[ ! -f "${trace_file}" ]]; then
  echo "TRACE_MATRIX_FAIL missing_file=${trace_file}"
  exit 1
fi

for run in $(seq 1 30); do
  line="$(rg "^RUN=${run} \\|" "${trace_file}" -n --no-line-number || true)"
  if [[ -z "${line}" ]]; then
    echo "TRACE_MATRIX_FAIL missing_run=RUN=${run}"
    exit 1
  fi

  if ! printf '%s\n' "${line}" | rg -q "acceptance_tests=[^|]+"; then
    echo "TRACE_MATRIX_FAIL run=RUN=${run} missing=acceptance_tests"
    exit 1
  fi
  if ! printf '%s\n' "${line}" | rg -q "ci_script=[^|]+"; then
    echo "TRACE_MATRIX_FAIL run=RUN=${run} missing=ci_script"
    exit 1
  fi
  if ! printf '%s\n' "${line}" | rg -q "proof_commands=[^|]+"; then
    echo "TRACE_MATRIX_FAIL run=RUN=${run} missing=proof_commands"
    exit 1
  fi
done

echo "TRACE_MATRIX_PASS runs=30 file=${trace_file}"
