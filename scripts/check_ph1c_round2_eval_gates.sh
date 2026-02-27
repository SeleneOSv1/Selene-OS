#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

INPUT_CSV="${1:-docs/fixtures/ph1c_round2_eval_snapshot.csv}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

# Step 11 gate consumes canonical eval harness output and fails closed if shape/coverage is invalid.
./scripts/check_ph1c_round2_eval_snapshot.sh "${INPUT_CSV}" >/dev/null

awk -F',' -v input_csv="${INPUT_CSV}" '
function abs(v) { return v < 0 ? -v : v; }

NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "stt_turns";
  req[3] = "tts_turns";
  req[4] = "stt_accept_count";
  req[5] = "stt_provider_response_count";
  req[6] = "stt_provider_schema_valid_count";
  req[7] = "stt_fallback_attempt_count";
  req[8] = "stt_fallback_success_count";
  req[9] = "partial_first_chunk_p95_ms";
  req[10] = "eos_to_first_token_p95_ms";
  req[11] = "capture_to_ph1c_handoff_p95_ms";
  req[12] = "stt_cost_microunits_total";
  req[13] = "tts_cost_microunits_total";
  req[14] = "audit_events_expected";
  req[15] = "audit_events_written";
  req[16] = "tenant_isolation_violations";
  req[17] = "code_switch_eval_count";
  req[18] = "code_switch_correct_count";
  req[19] = "rambling_eval_count";
  req[20] = "rambling_structured_correct_count";
  req[21] = "broken_english_eval_count";
  req[22] = "broken_english_normalization_correct_count";
  req[23] = "accent_eval_count";
  req[24] = "accent_correct_count";
  req[25] = "scrambled_eval_count";
  req[26] = "scrambled_clarify_resolved_count";
  for (i = 1; i <= 26; i++) {
    if (!(req[i] in col)) {
      printf("GATE_FAIL:missing_column column=%s\n", req[i]);
      missing = 1;
    }
  }
  next;
}

NR > 1 {
  gsub(/\r/, "", $0);
  if ($0 == "") {
    next;
  }
  rows++;

  captured = $(col["captured_at_utc"]);
  stt_turns = $(col["stt_turns"]) + 0;
  tts_turns = $(col["tts_turns"]) + 0;
  stt_accept_count = $(col["stt_accept_count"]) + 0;
  stt_provider_response_count = $(col["stt_provider_response_count"]) + 0;
  stt_provider_schema_valid_count = $(col["stt_provider_schema_valid_count"]) + 0;
  stt_fallback_attempt_count = $(col["stt_fallback_attempt_count"]) + 0;
  stt_fallback_success_count = $(col["stt_fallback_success_count"]) + 0;
  partial_p95 = $(col["partial_first_chunk_p95_ms"]) + 0;
  eos_p95 = $(col["eos_to_first_token_p95_ms"]) + 0;
  handoff_p95 = $(col["capture_to_ph1c_handoff_p95_ms"]) + 0;
  stt_cost = $(col["stt_cost_microunits_total"]) + 0;
  tts_cost = $(col["tts_cost_microunits_total"]) + 0;
  audit_expected = $(col["audit_events_expected"]) + 0;
  audit_written = $(col["audit_events_written"]) + 0;
  tenant_isolation_violations = $(col["tenant_isolation_violations"]) + 0;

  code_switch_eval_count = $(col["code_switch_eval_count"]) + 0;
  code_switch_correct_count = $(col["code_switch_correct_count"]) + 0;
  rambling_eval_count = $(col["rambling_eval_count"]) + 0;
  rambling_correct_count = $(col["rambling_structured_correct_count"]) + 0;
  broken_eval_count = $(col["broken_english_eval_count"]) + 0;
  broken_correct_count = $(col["broken_english_normalization_correct_count"]) + 0;
  accent_eval_count = $(col["accent_eval_count"]) + 0;
  accent_correct_count = $(col["accent_correct_count"]) + 0;
  scrambled_eval_count = $(col["scrambled_eval_count"]) + 0;
  scrambled_resolved_count = $(col["scrambled_clarify_resolved_count"]) + 0;

  if (captured == "") {
    printf("GATE_FAIL:missing_capture_timestamp row=%d\n", NR);
    fail = 1;
    next;
  }

  total_stt_turns += stt_turns;
  total_tts_turns += tts_turns;
  total_stt_accept_count += stt_accept_count;
  total_provider_response_count += stt_provider_response_count;
  total_provider_schema_valid_count += stt_provider_schema_valid_count;
  total_stt_fallback_attempt_count += stt_fallback_attempt_count;
  total_stt_fallback_success_count += stt_fallback_success_count;
  total_stt_cost += stt_cost;
  total_tts_cost += tts_cost;
  total_audit_expected += audit_expected;
  total_audit_written += audit_written;
  total_tenant_isolation_violations += tenant_isolation_violations;
  total_code_switch_eval += code_switch_eval_count;
  total_code_switch_correct += code_switch_correct_count;
  total_rambling_eval += rambling_eval_count;
  total_rambling_correct += rambling_correct_count;
  total_broken_eval += broken_eval_count;
  total_broken_correct += broken_correct_count;
  total_accent_eval += accent_eval_count;
  total_accent_correct += accent_correct_count;
  total_scrambled_eval += scrambled_eval_count;
  total_scrambled_resolved += scrambled_resolved_count;

  if (rows == 1) {
    max_partial_p95 = partial_p95;
    max_eos_p95 = eos_p95;
    max_handoff_p95 = handoff_p95;
  } else {
    if (partial_p95 > max_partial_p95) max_partial_p95 = partial_p95;
    if (eos_p95 > max_eos_p95) max_eos_p95 = eos_p95;
    if (handoff_p95 > max_handoff_p95) max_handoff_p95 = handoff_p95;
  }
}

END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "GATE_FAIL:no_metric_rows";
    exit 1;
  }
  if (total_stt_turns == 0 || total_tts_turns == 0 || total_provider_response_count == 0 || total_stt_fallback_attempt_count == 0 || total_audit_expected == 0) {
    print "GATE_FAIL:invalid_zero_denominator";
    exit 1;
  }
  if (total_code_switch_eval < 200 || total_rambling_eval < 250 || total_broken_eval < 200 || total_accent_eval < 220 || total_scrambled_eval < 160) {
    printf("GATE_FAIL:insufficient_eval_sample_size code_switch=%d rambling=%d broken=%d accent=%d scrambled=%d\n", total_code_switch_eval, total_rambling_eval, total_broken_eval, total_accent_eval, total_scrambled_eval);
    exit 1;
  }

  quality_acceptance_pct = (total_stt_accept_count * 100.0) / total_stt_turns;
  schema_valid_response_pct = (total_provider_schema_valid_count * 100.0) / total_provider_response_count;
  fallback_success_pct = (total_stt_fallback_success_count * 100.0) / total_stt_fallback_attempt_count;
  stt_cost_per_turn = total_stt_cost / total_stt_turns;
  tts_cost_per_turn = total_tts_cost / total_tts_turns;
  audit_completeness_pct = (total_audit_written * 100.0) / total_audit_expected;
  tenant_isolation_pct = ((total_stt_turns - total_tenant_isolation_violations) * 100.0) / total_stt_turns;
  code_switch_quality_pct = (total_code_switch_correct * 100.0) / total_code_switch_eval;
  rambling_to_structured_quality_pct = (total_rambling_correct * 100.0) / total_rambling_eval;
  broken_english_normalization_quality_pct = (total_broken_correct * 100.0) / total_broken_eval;
  accent_robustness_quality_pct = (total_accent_correct * 100.0) / total_accent_eval;
  scrambled_speech_clarify_recovery_quality_pct = (total_scrambled_resolved * 100.0) / total_scrambled_eval;

  if (quality_acceptance_pct < 90.0) {
    printf("GATE_FAIL:quality_acceptance_pct_lt_90 value=%.2f\n", quality_acceptance_pct);
    fail = 1;
  }
  if (schema_valid_response_pct < 98.5) {
    printf("GATE_FAIL:schema_valid_response_pct_lt_98_5 value=%.2f\n", schema_valid_response_pct);
    fail = 1;
  }
  if (fallback_success_pct < 85.0) {
    printf("GATE_FAIL:fallback_success_pct_lt_85 value=%.2f\n", fallback_success_pct);
    fail = 1;
  }
  if (max_partial_p95 > 280) {
    printf("GATE_FAIL:partial_first_chunk_p95_max_gt_280 value=%d\n", max_partial_p95);
    fail = 1;
  }
  if (max_eos_p95 > 320) {
    printf("GATE_FAIL:eos_to_first_token_p95_max_gt_320 value=%d\n", max_eos_p95);
    fail = 1;
  }
  if (max_handoff_p95 > 135) {
    printf("GATE_FAIL:capture_to_ph1c_handoff_p95_max_gt_135 value=%d\n", max_handoff_p95);
    fail = 1;
  }
  if (stt_cost_per_turn > 3800.0) {
    printf("GATE_FAIL:stt_cost_per_turn_gt_3800 value=%.2f\n", stt_cost_per_turn);
    fail = 1;
  }
  if (tts_cost_per_turn > 2700.0) {
    printf("GATE_FAIL:tts_cost_per_turn_gt_2700 value=%.2f\n", tts_cost_per_turn);
    fail = 1;
  }
  if (abs(audit_completeness_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:audit_completeness_not_100 value=%.2f\n", audit_completeness_pct);
    fail = 1;
  }
  if (abs(tenant_isolation_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:tenant_isolation_not_100 value=%.2f\n", tenant_isolation_pct);
    fail = 1;
  }
  if (code_switch_quality_pct < 88.0) {
    printf("GATE_FAIL:code_switch_quality_pct_lt_88 value=%.2f\n", code_switch_quality_pct);
    fail = 1;
  }
  if (rambling_to_structured_quality_pct < 86.0) {
    printf("GATE_FAIL:rambling_to_structured_quality_pct_lt_86 value=%.2f\n", rambling_to_structured_quality_pct);
    fail = 1;
  }
  if (broken_english_normalization_quality_pct < 85.0) {
    printf("GATE_FAIL:broken_english_normalization_quality_pct_lt_85 value=%.2f\n", broken_english_normalization_quality_pct);
    fail = 1;
  }
  if (accent_robustness_quality_pct < 86.0) {
    printf("GATE_FAIL:accent_robustness_quality_pct_lt_86 value=%.2f\n", accent_robustness_quality_pct);
    fail = 1;
  }
  if (scrambled_speech_clarify_recovery_quality_pct < 84.0) {
    printf("GATE_FAIL:scrambled_speech_clarify_recovery_quality_pct_lt_84 value=%.2f\n", scrambled_speech_clarify_recovery_quality_pct);
    fail = 1;
  }

  printf("PH1C_EVAL_GATE_SUMMARY:rows=%d,quality_acceptance_pct=%.2f,schema_valid_response_pct=%.2f,fallback_success_pct=%.2f,partial_first_chunk_p95_ms_max=%d,eos_to_first_token_p95_ms_max=%d,capture_to_ph1c_handoff_p95_ms_max=%d,cost_stt_microunits_per_turn=%.2f,cost_tts_microunits_per_turn=%.2f,audit_completeness_pct=%.2f,tenant_isolation_pct=%.2f,code_switch_quality_pct=%.2f,rambling_to_structured_quality_pct=%.2f,broken_english_normalization_quality_pct=%.2f,accent_robustness_quality_pct=%.2f,scrambled_speech_clarify_recovery_quality_pct=%.2f\n", rows, quality_acceptance_pct, schema_valid_response_pct, fallback_success_pct, max_partial_p95, max_eos_p95, max_handoff_p95, stt_cost_per_turn, tts_cost_per_turn, audit_completeness_pct, tenant_isolation_pct, code_switch_quality_pct, rambling_to_structured_quality_pct, broken_english_normalization_quality_pct, accent_robustness_quality_pct, scrambled_speech_clarify_recovery_quality_pct);

  if (fail) {
    exit 1;
  }
  printf("CHECK_OK ph1c_round2_eval_gates=pass input=%s rows=%d\n", input_csv, rows);
}
' "${INPUT_CSV}"
