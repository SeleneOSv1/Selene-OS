#!/usr/bin/env bash
set -euo pipefail

INPUT_CSV="${1:-docs/fixtures/ph1c_round2_baseline_snapshot.csv}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

awk -F',' '
NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "commit_hash";
  req[3] = "locale";
  req[4] = "device_route";
  req[5] = "noise_class";
  req[6] = "overlap_speech";
  req[7] = "voice_minutes";
  req[8] = "stt_turns";
  req[9] = "tts_turns";
  req[10] = "stt_provider_primary_success_count";
  req[11] = "stt_provider_fallback_success_count";
  req[12] = "stt_terminal_clarify_count";
  req[13] = "tts_provider_primary_success_count";
  req[14] = "tts_provider_fallback_success_count";
  req[15] = "tts_terminal_text_only_count";
  req[16] = "partial_first_chunk_p95_ms";
  req[17] = "eos_to_first_token_p95_ms";
  req[18] = "capture_to_ph1c_handoff_p95_ms";
  req[19] = "schema_drift_fail_count";
  req[20] = "kernel_contract_tests_pass";
  req[21] = "engine_tests_pass";
  req[22] = "os_tests_pass";
  for (i = 1; i <= 22; i++) {
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

  captured = $(col["captured_at_utc"]);
  commit_hash = $(col["commit_hash"]);
  locale = $(col["locale"]);
  device = $(col["device_route"]);
  noise = $(col["noise_class"]);
  overlap = $(col["overlap_speech"]);
  minutes = $(col["voice_minutes"]) + 0;
  stt_turns = $(col["stt_turns"]) + 0;
  tts_turns = $(col["tts_turns"]) + 0;
  stt_primary = $(col["stt_provider_primary_success_count"]) + 0;
  stt_fallback = $(col["stt_provider_fallback_success_count"]) + 0;
  stt_clarify = $(col["stt_terminal_clarify_count"]) + 0;
  tts_primary = $(col["tts_provider_primary_success_count"]) + 0;
  tts_fallback = $(col["tts_provider_fallback_success_count"]) + 0;
  tts_text_only = $(col["tts_terminal_text_only_count"]) + 0;
  partial_p95 = $(col["partial_first_chunk_p95_ms"]) + 0;
  eos_p95 = $(col["eos_to_first_token_p95_ms"]) + 0;
  handoff_p95 = $(col["capture_to_ph1c_handoff_p95_ms"]) + 0;
  schema_drift_fails = $(col["schema_drift_fail_count"]) + 0;
  ktest = $(col["kernel_contract_tests_pass"]) + 0;
  etest = $(col["engine_tests_pass"]) + 0;
  otest = $(col["os_tests_pass"]) + 0;

  rows++;
  locale_seen[locale] = 1;
  device_seen[device] = 1;
  noise_seen[noise] = 1;
  overlap_seen[overlap] = 1;

  if (captured == "" || commit_hash == "" || locale == "" || device == "" || noise == "" || overlap == "") {
    printf("BASELINE_FAIL:missing_required_dimension row=%d\n", NR);
    fail = 1;
    next;
  }
  if (minutes <= 0 || stt_turns <= 0 || tts_turns <= 0) {
    printf("BASELINE_FAIL:invalid_activity_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if ((stt_primary + stt_fallback + stt_clarify) != stt_turns) {
    printf("BASELINE_FAIL:stt_route_total_mismatch row=%d\n", NR);
    fail = 1;
    next;
  }
  if ((tts_primary + tts_fallback + tts_text_only) != tts_turns) {
    printf("BASELINE_FAIL:tts_route_total_mismatch row=%d\n", NR);
    fail = 1;
    next;
  }
  if (partial_p95 < 0 || eos_p95 < 0 || handoff_p95 < 0 || schema_drift_fails < 0) {
    printf("BASELINE_FAIL:invalid_latency_or_schema_values row=%d\n", NR);
    fail = 1;
    next;
  }
  if (!(overlap == "true" || overlap == "false")) {
    printf("BASELINE_FAIL:invalid_overlap_flag row=%d\n", NR);
    fail = 1;
    next;
  }
  if (ktest != 1 || etest != 1 || otest != 1) {
    printf("BASELINE_FAIL:test_proof_missing row=%d\n", NR);
    fail = 1;
    next;
  }

  total_minutes += minutes;
  total_stt += stt_turns;
  total_tts += tts_turns;
  total_stt_primary += stt_primary;
  total_stt_fallback += stt_fallback;
  total_stt_clarify += stt_clarify;
  total_tts_primary += tts_primary;
  total_tts_fallback += tts_fallback;
  total_tts_text_only += tts_text_only;

  printf("PH1C_BASELINE_ROW:captured_at_utc=%s,commit_hash=%s,locale=%s,device_route=%s,noise_class=%s,overlap_speech=%s,stt_turns=%d,tts_turns=%d,partial_first_chunk_p95_ms=%d,eos_to_first_token_p95_ms=%d,capture_to_ph1c_handoff_p95_ms=%d\n", captured, commit_hash, locale, device, noise, overlap, stt_turns, tts_turns, partial_p95, eos_p95, handoff_p95);
}
END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "BASELINE_FAIL:no_rows";
    exit 1;
  }
  if (length(locale_seen) < 2 || length(device_seen) < 2 || length(noise_seen) < 2 || length(overlap_seen) < 2) {
    printf("BASELINE_FAIL:coverage_insufficient locales=%d devices=%d noise_classes=%d overlap_classes=%d\n", length(locale_seen), length(device_seen), length(noise_seen), length(overlap_seen));
    exit 1;
  }
  if (fail) {
    exit 1;
  }

  stt_primary_pct = (total_stt_primary * 100.0) / total_stt;
  stt_fallback_pct = (total_stt_fallback * 100.0) / total_stt;
  stt_clarify_pct = (total_stt_clarify * 100.0) / total_stt;
  tts_primary_pct = (total_tts_primary * 100.0) / total_tts;
  tts_fallback_pct = (total_tts_fallback * 100.0) / total_tts;
  tts_text_only_pct = (total_tts_text_only * 100.0) / total_tts;

  printf("PH1C_BASELINE_SUMMARY:rows=%d,total_minutes=%.2f,total_stt_turns=%d,total_tts_turns=%d,stt_primary_pct=%.2f,stt_fallback_pct=%.2f,stt_terminal_clarify_pct=%.2f,tts_primary_pct=%.2f,tts_fallback_pct=%.2f,tts_terminal_text_only_pct=%.2f\n", rows, total_minutes, total_stt, total_tts, stt_primary_pct, stt_fallback_pct, stt_clarify_pct, tts_primary_pct, tts_fallback_pct, tts_text_only_pct);
  printf("CHECK_OK ph1c_round2_baseline_snapshot=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
