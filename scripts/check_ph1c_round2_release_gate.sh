#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

PH1C_CSV="${1:-docs/fixtures/ph1c_round2_eval_snapshot.csv}"
PH1K_CSV="${2:-docs/fixtures/ph1k_round2_eval_snapshot.csv}"

if [[ ! -f "${PH1C_CSV}" ]]; then
  echo "MISSING_INPUT:${PH1C_CSV}"
  exit 1
fi
if [[ ! -f "${PH1K_CSV}" ]]; then
  echo "MISSING_INPUT:${PH1K_CSV}"
  exit 1
fi

# Fail closed on schema/coverage drift before threshold checks.
./scripts/check_ph1c_round2_eval_snapshot.sh "${PH1C_CSV}" >/dev/null
./scripts/check_ph1k_round2_eval_snapshot.sh "${PH1K_CSV}" >/dev/null

awk -F',' -v input_csv="${PH1C_CSV}" '
function abs(v) { return v < 0 ? -v : v; }

NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "stt_turns";
  req[3] = "stt_accept_count";
  req[4] = "stt_provider_response_count";
  req[5] = "stt_provider_schema_valid_count";
  req[6] = "stt_fallback_attempt_count";
  req[7] = "stt_fallback_success_count";
  req[8] = "partial_first_chunk_p95_ms";
  req[9] = "eos_to_first_token_p95_ms";
  req[10] = "audit_events_expected";
  req[11] = "audit_events_written";
  req[12] = "tenant_isolation_violations";
  req[13] = "accent_eval_count";
  req[14] = "accent_correct_count";
  req[15] = "broken_english_eval_count";
  req[16] = "broken_english_normalization_correct_count";
  req[17] = "rambling_eval_count";
  req[18] = "rambling_structured_correct_count";
  req[19] = "scrambled_eval_count";
  req[20] = "scrambled_clarify_resolved_count";
  for (i = 1; i <= 20; i++) {
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
  if (captured == "") {
    printf("GATE_FAIL:missing_capture_timestamp row=%d\n", NR);
    fail = 1;
    next;
  }

  stt_turns = $(col["stt_turns"]) + 0;
  stt_accept = $(col["stt_accept_count"]) + 0;
  provider_resp = $(col["stt_provider_response_count"]) + 0;
  schema_valid = $(col["stt_provider_schema_valid_count"]) + 0;
  fallback_attempt = $(col["stt_fallback_attempt_count"]) + 0;
  fallback_success = $(col["stt_fallback_success_count"]) + 0;
  partial_p95 = $(col["partial_first_chunk_p95_ms"]) + 0;
  eos_p95 = $(col["eos_to_first_token_p95_ms"]) + 0;
  audit_expected = $(col["audit_events_expected"]) + 0;
  audit_written = $(col["audit_events_written"]) + 0;
  tenant_viol = $(col["tenant_isolation_violations"]) + 0;
  accent_eval = $(col["accent_eval_count"]) + 0;
  accent_correct = $(col["accent_correct_count"]) + 0;
  broken_eval = $(col["broken_english_eval_count"]) + 0;
  broken_correct = $(col["broken_english_normalization_correct_count"]) + 0;
  rambling_eval = $(col["rambling_eval_count"]) + 0;
  rambling_correct = $(col["rambling_structured_correct_count"]) + 0;
  scrambled_eval = $(col["scrambled_eval_count"]) + 0;
  scrambled_resolved = $(col["scrambled_clarify_resolved_count"]) + 0;

  total_stt_turns += stt_turns;
  total_stt_accept += stt_accept;
  total_provider_resp += provider_resp;
  total_schema_valid += schema_valid;
  total_fallback_attempt += fallback_attempt;
  total_fallback_success += fallback_success;
  total_audit_expected += audit_expected;
  total_audit_written += audit_written;
  total_tenant_viol += tenant_viol;
  total_accent_eval += accent_eval;
  total_accent_correct += accent_correct;
  total_broken_eval += broken_eval;
  total_broken_correct += broken_correct;
  total_rambling_eval += rambling_eval;
  total_rambling_correct += rambling_correct;
  total_scrambled_eval += scrambled_eval;
  total_scrambled_resolved += scrambled_resolved;

  if (rows == 1) {
    max_partial_p95 = partial_p95;
    max_eos_p95 = eos_p95;
  } else {
    if (partial_p95 > max_partial_p95) max_partial_p95 = partial_p95;
    if (eos_p95 > max_eos_p95) max_eos_p95 = eos_p95;
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
  if (total_stt_turns == 0 || total_provider_resp == 0 || total_fallback_attempt == 0 || total_audit_expected == 0 || total_accent_eval == 0 || total_broken_eval == 0 || (total_rambling_eval + total_scrambled_eval) == 0) {
    print "GATE_FAIL:invalid_zero_denominator";
    exit 1;
  }

  fallback_success_pct = (total_fallback_success * 100.0) / total_fallback_attempt;
  schema_valid_pct = (total_schema_valid * 100.0) / total_provider_resp;
  multilingual_acceptance_pct = (total_stt_accept * 100.0) / total_stt_turns;
  accent_acceptance_pct = (total_accent_correct * 100.0) / total_accent_eval;
  broken_norm_pct = (total_broken_correct * 100.0) / total_broken_eval;
  rambling_scrambled_resolve_pct = ((total_rambling_correct + total_scrambled_resolved) * 100.0) / (total_rambling_eval + total_scrambled_eval);
  audit_pct = (total_audit_written * 100.0) / total_audit_expected;
  tenant_isolation_pct = ((total_stt_turns - total_tenant_viol) * 100.0) / total_stt_turns;

  if (fallback_success_pct < 99.90) {
    printf("GATE_FAIL:stt_fallback_continuity_lt_99_90 value=%.4f\n", fallback_success_pct);
    fail = 1;
  }
  if (schema_valid_pct < 99.50) {
    printf("GATE_FAIL:provider_schema_valid_response_lt_99_50 value=%.4f\n", schema_valid_pct);
    fail = 1;
  }
  if (max_partial_p95 > 250) {
    printf("GATE_FAIL:partial_first_chunk_p95_gt_250 value=%d\n", max_partial_p95);
    fail = 1;
  }
  if (max_eos_p95 > 300) {
    printf("GATE_FAIL:eos_to_first_response_token_p95_gt_300 value=%d\n", max_eos_p95);
    fail = 1;
  }
  if (abs(audit_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:audit_completeness_not_100 value=%.4f\n", audit_pct);
    fail = 1;
  }
  if (abs(tenant_isolation_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:tenant_isolation_not_100 value=%.4f\n", tenant_isolation_pct);
    fail = 1;
  }
  if (multilingual_acceptance_pct < 95.0) {
    printf("GATE_FAIL:multilingual_transcript_acceptance_lt_95 value=%.4f\n", multilingual_acceptance_pct);
    fail = 1;
  }
  if (accent_acceptance_pct < 93.0) {
    printf("GATE_FAIL:heavy_accent_acceptance_lt_93 value=%.4f\n", accent_acceptance_pct);
    fail = 1;
  }
  if (broken_norm_pct < 90.0) {
    printf("GATE_FAIL:broken_english_normalization_lt_90 value=%.4f\n", broken_norm_pct);
    fail = 1;
  }
  if (rambling_scrambled_resolve_pct < 90.0) {
    printf("GATE_FAIL:rambling_scrambled_clarify_resolution_lt_90 value=%.4f\n", rambling_scrambled_resolve_pct);
    fail = 1;
  }

  printf("PH1C_STEP14_SUMMARY:rows=%d,stt_fallback_continuity_pct=%.4f,provider_schema_valid_response_pct=%.4f,partial_first_chunk_p95_ms_max=%d,eos_to_first_response_token_p95_ms_max=%d,multilingual_transcript_acceptance_pct=%.4f,heavy_accent_acceptance_pct=%.4f,broken_english_normalization_pct=%.4f,rambling_scrambled_clarify_resolution_pct=%.4f,audit_completeness_pct=%.4f,tenant_isolation_pct=%.4f\n", rows, fallback_success_pct, schema_valid_pct, max_partial_p95, max_eos_p95, multilingual_acceptance_pct, accent_acceptance_pct, broken_norm_pct, rambling_scrambled_resolve_pct, audit_pct, tenant_isolation_pct);

  if (fail) {
    exit 1;
  }
}
' "${PH1C_CSV}"

awk -F',' -v input_csv="${PH1K_CSV}" '
NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "false_interrupt_rate_per_hour";
  req[3] = "missed_interrupt_rate_pct";
  for (i = 1; i <= 3; i++) {
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
  false_rate = $(col["false_interrupt_rate_per_hour"]) + 0.0;
  missed_rate = $(col["missed_interrupt_rate_pct"]) + 0.0;

  if (captured == "") {
    printf("GATE_FAIL:missing_capture_timestamp row=%d\n", NR);
    fail = 1;
    next;
  }

  if (rows == 1) {
    max_false_rate = false_rate;
    max_missed_rate = missed_rate;
  } else {
    if (false_rate > max_false_rate) max_false_rate = false_rate;
    if (missed_rate > max_missed_rate) max_missed_rate = missed_rate;
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
  if (max_false_rate > 0.3) {
    printf("GATE_FAIL:false_interrupt_rate_gt_0_3_per_hour value=%.4f\n", max_false_rate);
    fail = 1;
  }
  if (max_missed_rate > 2.0) {
    printf("GATE_FAIL:missed_interrupt_rate_gt_2_pct value=%.4f\n", max_missed_rate);
    fail = 1;
  }

  printf("PH1K_STEP14_SUMMARY:rows=%d,false_interrupt_rate_per_hour_max=%.4f,missed_interrupt_rate_pct_max=%.4f\n", rows, max_false_rate, max_missed_rate);

  if (fail) {
    exit 1;
  }
}
' "${PH1K_CSV}"

echo "CHECK_OK ph1c_round2_release_gate=pass ph1c_input=${PH1C_CSV} ph1k_input=${PH1K_CSV}"
