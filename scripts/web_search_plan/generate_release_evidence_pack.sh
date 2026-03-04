#!/usr/bin/env bash
set -euo pipefail

head_commit="$(git rev-parse HEAD)"
branch="$(git rev-parse --abbrev-ref HEAD)"
timestamp_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
date_tag="$(date -u +"%Y%m%dT%H%M%SZ")"

release_lock_results="/tmp/selene_run30_release_lock_latest.tsv"
slo_lock_results="/tmp/selene_run30_slo_lock_latest.tsv"
output_dir="docs/web_search_plan/release_evidence"

if [[ ! -f "${release_lock_results}" ]]; then
  echo "RELEASE_EVIDENCE_FAIL missing_release_lock_results=${release_lock_results}"
  exit 1
fi
if [[ ! -f "${slo_lock_results}" ]]; then
  echo "RELEASE_EVIDENCE_FAIL missing_slo_lock_results=${slo_lock_results}"
  exit 1
fi

cmd_output="$(
  cargo run -p selene_os --bin web_search_release_evidence --quiet -- \
    --head-commit "${head_commit}" \
    --branch "${branch}" \
    --timestamp-utc "${timestamp_utc}" \
    --date-tag "${date_tag}" \
    --release-lock-results "${release_lock_results}" \
    --slo-lock-results "${slo_lock_results}" \
    --output-dir "${output_dir}"
)"

pack_file="$(printf '%s\n' "${cmd_output}" | rg '^RELEASE_EVIDENCE_PACK=' | tail -n 1 | sed 's/^RELEASE_EVIDENCE_PACK=//')"
if [[ -z "${pack_file}" ]]; then
  echo "RELEASE_EVIDENCE_FAIL missing_output_path"
  printf '%s\n' "${cmd_output}"
  exit 1
fi
if [[ ! -f "${pack_file}" ]]; then
  echo "RELEASE_EVIDENCE_FAIL missing_file=${pack_file}"
  exit 1
fi

manifest_hash="$(shasum -a 256 docs/web_search_plan/CONTRACT_HASH_MANIFEST.json | awk '{print $1}')"

if ! rg -q "\"head_commit\": \"${head_commit}\"" "${pack_file}"; then
  echo "RELEASE_EVIDENCE_FAIL head_commit_not_found file=${pack_file}"
  exit 1
fi
if ! rg -q "\"contract_hash_manifest_hash\": \"${manifest_hash}\"" "${pack_file}"; then
  echo "RELEASE_EVIDENCE_FAIL manifest_hash_not_found file=${pack_file}"
  exit 1
fi

echo "RELEASE_EVIDENCE_PACK_PASS file=${pack_file} head=${head_commit} manifest_hash=${manifest_hash}"
