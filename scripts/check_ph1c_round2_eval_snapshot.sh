#!/usr/bin/env bash
set -euo pipefail

INPUT_CSV="${1:-docs/fixtures/ph1c_round2_eval_snapshot.csv}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

awk -F',' '
function abs(v) { return v < 0 ? -v : v; }
function is_locale_tag(v) {
  return v ~ /^[A-Za-z]{2,3}(-[A-Za-z0-9]{2,8})*$/;
}

NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }

  req[1] = "captured_at_utc";
  req[2] = "commit_hash";
  req[3] = "window_min";
  req[4] = "tenant_id";
  req[5] = "locale";
  req[6] = "device_route";
  req[7] = "noise_class";
  req[8] = "overlap_speech";
  req[9] = "code_switch_mix_pct";
  req[10] = "stt_turns";
  req[11] = "tts_turns";
  req[12] = "stt_accept_count";
  req[13] = "stt_reject_count";
  req[14] = "stt_provider_response_count";
  req[15] = "stt_provider_schema_valid_count";
  req[16] = "stt_fallback_attempt_count";
  req[17] = "stt_fallback_success_count";
  req[18] = "stt_terminal_clarify_count";
  req[19] = "partial_first_chunk_p95_ms";
  req[20] = "eos_to_first_token_p95_ms";
  req[21] = "capture_to_ph1c_handoff_p95_ms";
  req[22] = "stt_cost_microunits_total";
  req[23] = "tts_cost_microunits_total";
  req[24] = "audit_events_expected";
  req[25] = "audit_events_written";
  req[26] = "tenant_isolation_violations";
  req[27] = "code_switch_eval_count";
  req[28] = "code_switch_correct_count";
  req[29] = "rambling_eval_count";
  req[30] = "rambling_structured_correct_count";
  req[31] = "broken_english_eval_count";
  req[32] = "broken_english_normalization_correct_count";
  req[33] = "accent_eval_count";
  req[34] = "accent_correct_count";
  req[35] = "scrambled_eval_count";
  req[36] = "scrambled_clarify_resolved_count";
  for (i = 1; i <= 36; i++) {
    if (!(req[i] in col)) {
      printf("MISSING_COLUMN:%s\n", req[i]);
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
  commit_hash = $(col["commit_hash"]);
  window_min = $(col["window_min"]) + 0;
  tenant_id = $(col["tenant_id"]);
  locale = $(col["locale"]);
  device_route = $(col["device_route"]);
  noise_class = $(col["noise_class"]);
  overlap = $(col["overlap_speech"]);
  code_switch_mix_pct = $(col["code_switch_mix_pct"]) + 0.0;

  stt_turns = $(col["stt_turns"]) + 0;
  tts_turns = $(col["tts_turns"]) + 0;
  stt_accept_count = $(col["stt_accept_count"]) + 0;
  stt_reject_count = $(col["stt_reject_count"]) + 0;
  stt_provider_response_count = $(col["stt_provider_response_count"]) + 0;
  stt_provider_schema_valid_count = $(col["stt_provider_schema_valid_count"]) + 0;
  stt_fallback_attempt_count = $(col["stt_fallback_attempt_count"]) + 0;
  stt_fallback_success_count = $(col["stt_fallback_success_count"]) + 0;
  stt_terminal_clarify_count = $(col["stt_terminal_clarify_count"]) + 0;
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

  if (captured == "" || commit_hash == "" || tenant_id == "") {
    printf("EVAL_FAIL:missing_capture_commit_or_tenant row=%d\n", NR);
    fail = 1;
    next;
  }
  if (window_min <= 0 || stt_turns <= 0 || tts_turns <= 0) {
    printf("EVAL_FAIL:invalid_window_or_turn_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (!is_locale_tag(locale)) {
    printf("EVAL_FAIL:unsupported_locale row=%d locale=%s\n", NR, locale);
    fail = 1;
    next;
  }
  if (!(device_route == "desktop_mic" || device_route == "mobile_mic")) {
    printf("EVAL_FAIL:unsupported_device_route row=%d route=%s\n", NR, device_route);
    fail = 1;
    next;
  }
  if (!(noise_class == "quiet" || noise_class == "noisy")) {
    printf("EVAL_FAIL:unsupported_noise_class row=%d noise=%s\n", NR, noise_class);
    fail = 1;
    next;
  }
  if (!(overlap == "true" || overlap == "false")) {
    printf("EVAL_FAIL:invalid_overlap_flag row=%d value=%s\n", NR, overlap);
    fail = 1;
    next;
  }
  if (code_switch_mix_pct < 0 || code_switch_mix_pct > 100) {
    printf("EVAL_FAIL:invalid_code_switch_mix_pct row=%d\n", NR);
    fail = 1;
    next;
  }

  if (stt_accept_count < 0 || stt_reject_count < 0 || stt_accept_count + stt_reject_count != stt_turns) {
    printf("EVAL_FAIL:stt_accept_reject_mismatch row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_provider_response_count < stt_turns) {
    printf("EVAL_FAIL:provider_response_count_lt_turns row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_provider_schema_valid_count < 0 || stt_provider_schema_valid_count > stt_provider_response_count) {
    printf("EVAL_FAIL:provider_schema_valid_count_invalid row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_fallback_attempt_count < 0 || stt_fallback_success_count < 0 || stt_terminal_clarify_count < 0) {
    printf("EVAL_FAIL:negative_fallback_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_fallback_success_count > stt_fallback_attempt_count || stt_terminal_clarify_count > stt_fallback_attempt_count) {
    printf("EVAL_FAIL:fallback_component_count_exceeds_attempts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_fallback_success_count + stt_terminal_clarify_count != stt_fallback_attempt_count) {
    printf("EVAL_FAIL:fallback_resolution_mismatch row=%d\n", NR);
    fail = 1;
    next;
  }

  if (partial_p95 < 0 || eos_p95 < 0 || handoff_p95 < 0) {
    printf("EVAL_FAIL:invalid_latency row=%d\n", NR);
    fail = 1;
    next;
  }
  if (stt_cost <= 0 || tts_cost <= 0) {
    printf("EVAL_FAIL:invalid_cost_totals row=%d\n", NR);
    fail = 1;
    next;
  }
  if (audit_expected <= 0 || audit_written < 0 || audit_written > audit_expected) {
    printf("EVAL_FAIL:invalid_audit_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (tenant_isolation_violations < 0 || tenant_isolation_violations > stt_turns) {
    printf("EVAL_FAIL:invalid_tenant_isolation_violations row=%d\n", NR);
    fail = 1;
    next;
  }

  if (code_switch_eval_count <= 0 || code_switch_correct_count < 0 || code_switch_correct_count > code_switch_eval_count) {
    printf("EVAL_FAIL:invalid_code_switch_eval_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (rambling_eval_count <= 0 || rambling_correct_count < 0 || rambling_correct_count > rambling_eval_count) {
    printf("EVAL_FAIL:invalid_rambling_eval_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (broken_eval_count <= 0 || broken_correct_count < 0 || broken_correct_count > broken_eval_count) {
    printf("EVAL_FAIL:invalid_broken_english_eval_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (accent_eval_count <= 0 || accent_correct_count < 0 || accent_correct_count > accent_eval_count) {
    printf("EVAL_FAIL:invalid_accent_eval_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (scrambled_eval_count <= 0 || scrambled_resolved_count < 0 || scrambled_resolved_count > scrambled_eval_count) {
    printf("EVAL_FAIL:invalid_scrambled_eval_counts row=%d\n", NR);
    fail = 1;
    next;
  }

  seen_tenant[tenant_id] = 1;
  seen_locale[locale] = 1;
  seen_route[device_route] = 1;
  seen_noise[noise_class] = 1;
  seen_overlap[overlap] = 1;
  if (code_switch_mix_pct > 0) code_switch_rows_with_mix++;

  total_stt_turns += stt_turns;
  total_tts_turns += tts_turns;
  total_stt_accept_count += stt_accept_count;
  total_provider_response_count += stt_provider_response_count;
  total_provider_schema_valid_count += stt_provider_schema_valid_count;
  total_stt_fallback_attempt_count += stt_fallback_attempt_count;
  total_stt_fallback_success_count += stt_fallback_success_count;
  total_partial_p95 += partial_p95;
  total_eos_p95 += eos_p95;
  total_handoff_p95 += handoff_p95;
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

  printf("PH1C_EVAL_ROW:captured_at_utc=%s,tenant_id=%s,locale=%s,device_route=%s,noise_class=%s,overlap_speech=%s,stt_turns=%d,stt_accept_count=%d,schema_valid=%d/%d,fallback_success=%d/%d,partial_first_chunk_p95_ms=%d,eos_to_first_token_p95_ms=%d,capture_to_ph1c_handoff_p95_ms=%d,code_switch_correct=%d/%d,rambling_correct=%d/%d,broken_english_correct=%d/%d,accent_correct=%d/%d,scrambled_resolved=%d/%d\n", captured, tenant_id, locale, device_route, noise_class, overlap, stt_turns, stt_accept_count, stt_provider_schema_valid_count, stt_provider_response_count, stt_fallback_success_count, stt_fallback_attempt_count, partial_p95, eos_p95, handoff_p95, code_switch_correct_count, code_switch_eval_count, rambling_correct_count, rambling_eval_count, broken_correct_count, broken_eval_count, accent_correct_count, accent_eval_count, scrambled_resolved_count, scrambled_eval_count);
}

END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "EVAL_FAIL:no_rows";
    exit 1;
  }

  if (length(seen_locale) < 6) {
    printf("EVAL_FAIL:insufficient_locale_coverage locales=%d\n", length(seen_locale));
    fail = 1;
  }
  if (length(seen_tenant) < 2) {
    printf("EVAL_FAIL:insufficient_tenant_coverage tenants=%d\n", length(seen_tenant));
    fail = 1;
  }
  if (length(seen_route) < 2 || !("desktop_mic" in seen_route) || !("mobile_mic" in seen_route)) {
    printf("EVAL_FAIL:missing_device_route_coverage routes=%d\n", length(seen_route));
    fail = 1;
  }
  if (length(seen_noise) < 2 || !("quiet" in seen_noise) || !("noisy" in seen_noise)) {
    printf("EVAL_FAIL:missing_noise_class_coverage noise_classes=%d\n", length(seen_noise));
    fail = 1;
  }
  if (length(seen_overlap) < 2 || !("true" in seen_overlap) || !("false" in seen_overlap)) {
    printf("EVAL_FAIL:missing_overlap_coverage overlap_classes=%d\n", length(seen_overlap));
    fail = 1;
  }
  if (code_switch_rows_with_mix == 0) {
    print "EVAL_FAIL:missing_code_switch_mix_rows";
    fail = 1;
  }

  if (fail) {
    exit 1;
  }

  quality_acceptance_pct = (total_stt_accept_count * 100.0) / total_stt_turns;
  schema_valid_pct = (total_provider_schema_valid_count * 100.0) / total_provider_response_count;
  fallback_success_pct = (total_stt_fallback_success_count * 100.0) / total_stt_fallback_attempt_count;
  audit_completeness_pct = (total_audit_written * 100.0) / total_audit_expected;
  tenant_isolation_pct = ((total_stt_turns - total_tenant_isolation_violations) * 100.0) / total_stt_turns;

  code_switch_quality_pct = (total_code_switch_correct * 100.0) / total_code_switch_eval;
  rambling_to_structured_quality_pct = (total_rambling_correct * 100.0) / total_rambling_eval;
  broken_english_normalization_quality_pct = (total_broken_correct * 100.0) / total_broken_eval;
  accent_robustness_quality_pct = (total_accent_correct * 100.0) / total_accent_eval;
  scrambled_speech_clarify_recovery_quality_pct = (total_scrambled_resolved * 100.0) / total_scrambled_eval;

  avg_partial_p95 = total_partial_p95 / rows;
  avg_eos_p95 = total_eos_p95 / rows;
  avg_handoff_p95 = total_handoff_p95 / rows;
  stt_cost_per_turn = total_stt_cost / total_stt_turns;
  tts_cost_per_turn = total_tts_cost / total_tts_turns;

  printf("PH1C_EVAL_SUMMARY:rows=%d,total_stt_turns=%d,total_tts_turns=%d,quality_acceptance_pct=%.2f,schema_valid_response_pct=%.2f,fallback_success_pct=%.2f,partial_first_chunk_p95_ms_avg=%.2f,partial_first_chunk_p95_ms_max=%d,eos_to_first_token_p95_ms_avg=%.2f,eos_to_first_token_p95_ms_max=%d,capture_to_ph1c_handoff_p95_ms_avg=%.2f,capture_to_ph1c_handoff_p95_ms_max=%d,cost_stt_microunits_per_turn=%.2f,cost_tts_microunits_per_turn=%.2f,audit_completeness_pct=%.2f,tenant_isolation_pct=%.2f,code_switch_quality_pct=%.2f,rambling_to_structured_quality_pct=%.2f,broken_english_normalization_quality_pct=%.2f,accent_robustness_quality_pct=%.2f,scrambled_speech_clarify_recovery_quality_pct=%.2f\n", rows, total_stt_turns, total_tts_turns, quality_acceptance_pct, schema_valid_pct, fallback_success_pct, avg_partial_p95, max_partial_p95, avg_eos_p95, max_eos_p95, avg_handoff_p95, max_handoff_p95, stt_cost_per_turn, tts_cost_per_turn, audit_completeness_pct, tenant_isolation_pct, code_switch_quality_pct, rambling_to_structured_quality_pct, broken_english_normalization_quality_pct, accent_robustness_quality_pct, scrambled_speech_clarify_recovery_quality_pct);
  printf("CHECK_OK ph1c_round2_eval_snapshot=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
