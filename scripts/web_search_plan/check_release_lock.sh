#!/usr/bin/env bash
set -euo pipefail

head_commit="$(git rev-parse HEAD)"
manifest_hash="$(shasum -a 256 docs/web_search_plan/CONTRACT_HASH_MANIFEST.json | awk '{print $1}')"
results_file="/tmp/selene_run30_release_lock_${head_commit}.tsv"
latest_file="/tmp/selene_run30_release_lock_latest.tsv"

: > "${results_file}"
printf "HEAD_COMMIT\t%s\n" "${head_commit}" >> "${results_file}"
printf "CONTRACT_HASH_MANIFEST_HASH\t%s\n" "${manifest_hash}" >> "${results_file}"

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
    cp "${results_file}" "${latest_file}"
    echo "[FAIL] ${gate_name}"
    echo "FINAL_OVERALL=FAIL"
    echo "HEAD_COMMIT=${head_commit}"
    echo "CONTRACT_HASH_MANIFEST_HASH=${manifest_hash}"
    echo "RELEASE_LOCK_RESULTS_FILE=${results_file}"
    exit 1
  fi
}

declared_check_names=$'check_contracts.sh\ncheck_reason_codes.sh\ncheck_idempotency.sh\ncheck_turn_state_machine.sh\ncheck_handoff_ownership.sh\ncheck_doc_canon.sh\ncheck_trace_matrix.sh\ncheck_proxy_universal.sh\ncheck_url_fetch_core.sh\ncheck_chunk_hash_core.sh\ncheck_web_provider_ladder.sh\ncheck_news_provider_ladder.sh\ncheck_news_parity.sh\ncheck_search_topk_pipeline.sh\ncheck_synthesis_core.sh\ncheck_write_core.sh\ncheck_debug_packet.sh\ncheck_perf_cost_tiers.sh\ncheck_cache_parallel.sh\ncheck_vision_engine.sh\ncheck_learning_layer.sh\ncheck_replay_harness.sh\ncheck_quality_gates.sh\ncheck_continuous_eval.sh\ncheck_structured_connectors.sh\ncheck_document_parsing.sh\ncheck_analytics_numeric_consensus.sh\ncheck_competitive_intel.sh\ncheck_realtime_api_mode.sh\ncheck_regulatory_mode.sh\ncheck_trust_model.sh\ncheck_multihop_research.sh\ncheck_temporal_mode.sh\ncheck_risk_mode.sh\ncheck_merge_mode.sh\ncheck_parity_enhancements.sh\ncheck_gap_closers.sh\ncheck_enterprise_integration_lock.sh\ncheck_slo_lock.sh'
missing_checks=()
for check_path in scripts/web_search_plan/check_*.sh; do
  check_name="$(basename "${check_path}")"
  if [ "${check_name}" = "check_release_lock.sh" ]; then
    continue
  fi
  if ! printf '%s\n' "${declared_check_names}" | grep -Fxq "${check_name}"; then
    missing_checks+=("${check_name}")
  fi
done
if [ "${#missing_checks[@]}" -gt 0 ]; then
  echo "SELF_COVER_PASS=FAIL"
  for check_name in "${missing_checks[@]}"; do
    echo "MISSING_CHECK=${check_name}"
  done
  exit 1
fi
echo "SELF_COVER_PASS=PASS"

run_gate "scripts/web_search_plan/check_contracts.sh" scripts/web_search_plan/check_contracts.sh
run_gate "scripts/web_search_plan/check_reason_codes.sh" scripts/web_search_plan/check_reason_codes.sh
run_gate "scripts/web_search_plan/check_idempotency.sh" scripts/web_search_plan/check_idempotency.sh
run_gate "scripts/web_search_plan/check_turn_state_machine.sh" scripts/web_search_plan/check_turn_state_machine.sh
run_gate "scripts/web_search_plan/check_handoff_ownership.sh" scripts/web_search_plan/check_handoff_ownership.sh
run_gate "scripts/web_search_plan/check_doc_canon.sh" scripts/web_search_plan/check_doc_canon.sh
run_gate "scripts/web_search_plan/check_trace_matrix.sh" scripts/web_search_plan/check_trace_matrix.sh
run_gate "scripts/web_search_plan/check_proxy_universal.sh" scripts/web_search_plan/check_proxy_universal.sh
run_gate "scripts/web_search_plan/check_url_fetch_core.sh" scripts/web_search_plan/check_url_fetch_core.sh
run_gate "scripts/web_search_plan/check_chunk_hash_core.sh" scripts/web_search_plan/check_chunk_hash_core.sh
run_gate "scripts/web_search_plan/check_web_provider_ladder.sh" scripts/web_search_plan/check_web_provider_ladder.sh
run_gate "scripts/web_search_plan/check_news_provider_ladder.sh" scripts/web_search_plan/check_news_provider_ladder.sh
run_gate "scripts/web_search_plan/check_news_parity.sh" scripts/web_search_plan/check_news_parity.sh
run_gate "scripts/web_search_plan/check_search_topk_pipeline.sh" scripts/web_search_plan/check_search_topk_pipeline.sh
run_gate "scripts/web_search_plan/check_synthesis_core.sh" scripts/web_search_plan/check_synthesis_core.sh
run_gate "scripts/web_search_plan/check_write_core.sh" scripts/web_search_plan/check_write_core.sh
run_gate "scripts/web_search_plan/check_debug_packet.sh" scripts/web_search_plan/check_debug_packet.sh
run_gate "scripts/web_search_plan/check_perf_cost_tiers.sh" scripts/web_search_plan/check_perf_cost_tiers.sh
run_gate "scripts/web_search_plan/check_cache_parallel.sh" scripts/web_search_plan/check_cache_parallel.sh
run_gate "scripts/web_search_plan/check_vision_engine.sh" scripts/web_search_plan/check_vision_engine.sh
run_gate "scripts/web_search_plan/check_learning_layer.sh" scripts/web_search_plan/check_learning_layer.sh
run_gate "scripts/web_search_plan/check_replay_harness.sh" scripts/web_search_plan/check_replay_harness.sh
run_gate "scripts/web_search_plan/check_quality_gates.sh" scripts/web_search_plan/check_quality_gates.sh
run_gate "scripts/web_search_plan/check_continuous_eval.sh" scripts/web_search_plan/check_continuous_eval.sh
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
run_gate "scripts/web_search_plan/check_parity_enhancements.sh" scripts/web_search_plan/check_parity_enhancements.sh
run_gate "scripts/web_search_plan/check_gap_closers.sh" scripts/web_search_plan/check_gap_closers.sh
run_gate "scripts/web_search_plan/check_enterprise_integration_lock.sh" scripts/web_search_plan/check_enterprise_integration_lock.sh
run_gate "scripts/web_search_plan/check_slo_lock.sh" scripts/web_search_plan/check_slo_lock.sh
run_gate "cargo run -p selene_os --bin web_search_turn -- --fixture \"test query\"" bash -lc '
set +e
cargo run -p selene_os --bin web_search_turn -- --fixture "test query" >/tmp/selene_web_search_turn_fixture.log 2>&1
status=$?
set -e
if [ "$status" -eq 2 ] && grep -q "^FAIL_CLOSED_REASON=" /tmp/selene_web_search_turn_fixture.log; then
  exit 0
fi
cat /tmp/selene_web_search_turn_fixture.log
echo "expected fixture mode to fail closed with exit=2"
exit 1
'
run_gate "cargo run -p selene_os --bin web_search_enterprise_turn -- --fixture --mode report --query \"test\"" cargo run -p selene_os --bin web_search_enterprise_turn -- --fixture --mode report --query "test"
run_gate "cargo run -p selene_os --bin web_search_vision_turn -- --fixture --mode image_ocr --asset text_heavy" cargo run -p selene_os --bin web_search_vision_turn -- --fixture --mode image_ocr --asset text_heavy
run_gate "cargo test -p selene_os web_search_plan::runtime::runtime_tests --quiet" cargo test -p selene_os web_search_plan::runtime::runtime_tests --quiet
run_gate "cargo test -p selene_os web_search_plan::tests --quiet" cargo test -p selene_os web_search_plan::tests --quiet
run_gate "cargo test -p selene_os web_search_plan::release::release_tests --quiet" cargo test -p selene_os web_search_plan::release::release_tests --quiet

printf "OVERALL\tPASS\n" >> "${results_file}"
cp "${results_file}" "${latest_file}"

echo "FINAL_OVERALL=PASS"
echo "HEAD_COMMIT=${head_commit}"
echo "CONTRACT_HASH_MANIFEST_HASH=${manifest_hash}"
echo "RELEASE_LOCK_RESULTS_FILE=${results_file}"
