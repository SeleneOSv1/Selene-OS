#!/usr/bin/env bash
set -euo pipefail

INPUT_CSV="${1:-${SELENE_PH1K_EVAL_SNAPSHOT_PATH:-.dev/ph1k_live_eval_snapshot.csv}}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

awk -F',' '
function abs(v) { return v < 0 ? -v : v; }

NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "commit_hash";
  req[3] = "window_min";
  req[4] = "locale_tag";
  req[5] = "device_route";
  req[6] = "noise_class";
  req[7] = "overlap_speech";
  req[8] = "active_session_hours";
  req[9] = "interrupt_events";
  req[10] = "false_interrupt_count";
  req[11] = "missed_interrupt_count";
  req[12] = "false_interrupt_rate_per_hour";
  req[13] = "missed_interrupt_rate_pct";
  req[14] = "end_of_speech_p95_ms";
  req[15] = "capture_to_ph1c_handoff_p95_ms";
  req[16] = "device_failover_recovery_p95_ms";
  req[17] = "noisy_recovery_success_pct";
  req[18] = "multilingual_interrupt_recall_pct";
  req[19] = "audit_completeness_pct";
  req[20] = "tenant_isolation_pct";
  for (i = 1; i <= 20; i++) {
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
  window_min = $(col["window_min"]) + 0;
  locale = $(col["locale_tag"]);
  device_route = $(col["device_route"]);
  noise_class = $(col["noise_class"]);
  overlap_raw = $(col["overlap_speech"]);
  active_hours = $(col["active_session_hours"]) + 0.0;
  interrupt_events = $(col["interrupt_events"]) + 0;
  false_count = $(col["false_interrupt_count"]) + 0;
  missed_count = $(col["missed_interrupt_count"]) + 0;
  false_rate_per_hour = $(col["false_interrupt_rate_per_hour"]) + 0.0;
  missed_rate_pct = $(col["missed_interrupt_rate_pct"]) + 0.0;
  eos_p95 = $(col["end_of_speech_p95_ms"]) + 0;
  handoff_p95 = $(col["capture_to_ph1c_handoff_p95_ms"]) + 0;
  failover_p95 = $(col["device_failover_recovery_p95_ms"]) + 0;
  noisy_recovery_pct = $(col["noisy_recovery_success_pct"]) + 0.0;
  multilingual_recall_pct = $(col["multilingual_interrupt_recall_pct"]) + 0.0;
  audit_pct = $(col["audit_completeness_pct"]) + 0.0;
  tenant_pct = $(col["tenant_isolation_pct"]) + 0.0;

  if (captured == "captured_at_utc" || commit_hash == "commit_hash") {
    next;
  }

  rows++;

  if (captured == "" || commit_hash == "") {
    printf("EVAL_FAIL:missing_capture_or_commit row=%d\n", NR);
    fail = 1;
    next;
  }
  if (window_min <= 0 || active_hours <= 0 || interrupt_events <= 0) {
    printf("EVAL_FAIL:invalid_window_or_activity row=%d\n", NR);
    fail = 1;
    next;
  }
  if (false_count < 0 || false_count > interrupt_events) {
    printf("EVAL_FAIL:invalid_false_interrupt_count row=%d\n", NR);
    fail = 1;
    next;
  }
  if (missed_count < 0 || missed_count > interrupt_events) {
    printf("EVAL_FAIL:invalid_missed_interrupt_count row=%d\n", NR);
    fail = 1;
    next;
  }
  if (!(overlap_raw == "0" || overlap_raw == "1")) {
    printf("EVAL_FAIL:invalid_overlap_speech_flag row=%d value=%s\n", NR, overlap_raw);
    fail = 1;
    next;
  }

  if (!(locale == "en-US" || locale == "es-ES" || locale == "zh-CN" || locale == "tr-TR")) {
    printf("EVAL_FAIL:unsupported_locale row=%d locale=%s\n", NR, locale);
    fail = 1;
    next;
  }
  if (!(device_route == "BUILT_IN" || device_route == "BLUETOOTH" || device_route == "USB" || device_route == "VIRTUAL")) {
    printf("EVAL_FAIL:unsupported_device_route row=%d route=%s\n", NR, device_route);
    fail = 1;
    next;
  }
  if (!(noise_class == "CLEAN" || noise_class == "ELEVATED" || noise_class == "SEVERE")) {
    printf("EVAL_FAIL:unsupported_noise_class row=%d noise=%s\n", NR, noise_class);
    fail = 1;
    next;
  }

  for (metric_i = 1; metric_i <= 9; metric_i++) {
    if (metric_i == 1) metric = false_rate_per_hour;
    if (metric_i == 2) metric = missed_rate_pct;
    if (metric_i == 3) metric = eos_p95;
    if (metric_i == 4) metric = handoff_p95;
    if (metric_i == 5) metric = failover_p95;
    if (metric_i == 6) metric = noisy_recovery_pct;
    if (metric_i == 7) metric = multilingual_recall_pct;
    if (metric_i == 8) metric = audit_pct;
    if (metric_i == 9) metric = tenant_pct;
    if (metric < 0) {
      printf("EVAL_FAIL:negative_metric row=%d metric_index=%d\n", NR, metric_i);
      fail = 1;
      next;
    }
  }
  if (missed_rate_pct > 100 || noisy_recovery_pct > 100 || multilingual_recall_pct > 100 || audit_pct > 100 || tenant_pct > 100) {
    printf("EVAL_FAIL:metric_out_of_100_range row=%d\n", NR);
    fail = 1;
    next;
  }

  derived_false_rate = false_count / active_hours;
  derived_missed_pct = (missed_count * 100.0) / interrupt_events;
  if (abs(derived_false_rate - false_rate_per_hour) > 0.05) {
    printf("EVAL_FAIL:false_rate_inconsistent_with_counts row=%d expected=%.4f got=%.4f\n", NR, derived_false_rate, false_rate_per_hour);
    fail = 1;
    next;
  }
  if (abs(derived_missed_pct - missed_rate_pct) > 0.25) {
    printf("EVAL_FAIL:missed_rate_inconsistent_with_counts row=%d expected=%.4f got=%.4f\n", NR, derived_missed_pct, missed_rate_pct);
    fail = 1;
    next;
  }

  seen_locale[locale] = 1;
  seen_route[device_route] = 1;
  seen_noise[noise_class] = 1;
  if (overlap_raw == "1") overlap_yes++;
  if (overlap_raw == "0") overlap_no++;

  sum_false_rate += false_rate_per_hour;
  sum_missed_pct += missed_rate_pct;
  sum_eos += eos_p95;
  sum_handoff += handoff_p95;
  sum_failover += failover_p95;
  sum_noisy_recovery += noisy_recovery_pct;
  sum_multilingual_recall += multilingual_recall_pct;
  sum_audit += audit_pct;
  sum_tenant += tenant_pct;

  if (rows == 1) {
    max_false_rate = false_rate_per_hour;
    max_missed_pct = missed_rate_pct;
    max_eos = eos_p95;
    max_handoff = handoff_p95;
    max_failover = failover_p95;
    min_noisy_recovery = noisy_recovery_pct;
    min_multilingual_recall = multilingual_recall_pct;
    min_audit = audit_pct;
    min_tenant = tenant_pct;
    first_captured = captured;
    first_false_rate = false_rate_per_hour;
    first_missed_pct = missed_rate_pct;
    first_eos = eos_p95;
    first_handoff = handoff_p95;
  } else {
    if (false_rate_per_hour > max_false_rate) max_false_rate = false_rate_per_hour;
    if (missed_rate_pct > max_missed_pct) max_missed_pct = missed_rate_pct;
    if (eos_p95 > max_eos) max_eos = eos_p95;
    if (handoff_p95 > max_handoff) max_handoff = handoff_p95;
    if (failover_p95 > max_failover) max_failover = failover_p95;
    if (noisy_recovery_pct < min_noisy_recovery) min_noisy_recovery = noisy_recovery_pct;
    if (multilingual_recall_pct < min_multilingual_recall) min_multilingual_recall = multilingual_recall_pct;
    if (audit_pct < min_audit) min_audit = audit_pct;
    if (tenant_pct < min_tenant) min_tenant = tenant_pct;
  }

  last_captured = captured;
  last_false_rate = false_rate_per_hour;
  last_missed_pct = missed_rate_pct;
  last_eos = eos_p95;
  last_handoff = handoff_p95;

  printf("PH1K_EVAL_ROW:captured_at_utc=%s,locale=%s,device_route=%s,noise_class=%s,overlap_speech=%s,false_interrupt_rate_per_hour=%.4f,missed_interrupt_rate_pct=%.2f,end_of_speech_p95_ms=%d,capture_to_ph1c_handoff_p95_ms=%d,device_failover_recovery_p95_ms=%d,noisy_recovery_success_pct=%.2f,multilingual_interrupt_recall_pct=%.2f,audit_completeness_pct=%.2f,tenant_isolation_pct=%.2f\n", captured, locale, device_route, noise_class, overlap_raw, false_rate_per_hour, missed_rate_pct, eos_p95, handoff_p95, failover_p95, noisy_recovery_pct, multilingual_recall_pct, audit_pct, tenant_pct);
}

END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "EVAL_FAIL:no_rows";
    exit 1;
  }

  req_locale[1] = "en-US";
  req_locale[2] = "es-ES";
  req_locale[3] = "zh-CN";
  req_locale[4] = "tr-TR";
  for (i = 1; i <= 4; i++) {
    if (!(req_locale[i] in seen_locale)) {
      printf("EVAL_FAIL:missing_locale_coverage locale=%s\n", req_locale[i]);
      fail = 1;
    }
  }

  req_route[1] = "BUILT_IN";
  req_route[2] = "BLUETOOTH";
  req_route[3] = "USB";
  req_route[4] = "VIRTUAL";
  for (i = 1; i <= 4; i++) {
    if (!(req_route[i] in seen_route)) {
      printf("EVAL_FAIL:missing_device_route_coverage route=%s\n", req_route[i]);
      fail = 1;
    }
  }

  req_noise[1] = "CLEAN";
  req_noise[2] = "ELEVATED";
  req_noise[3] = "SEVERE";
  for (i = 1; i <= 3; i++) {
    if (!(req_noise[i] in seen_noise)) {
      printf("EVAL_FAIL:missing_noise_class_coverage noise=%s\n", req_noise[i]);
      fail = 1;
    }
  }

  if (overlap_yes == 0 || overlap_no == 0) {
    printf("EVAL_FAIL:missing_overlap_speech_coverage overlap_yes=%d overlap_no=%d\n", overlap_yes, overlap_no);
    fail = 1;
  }

  if (fail) {
    exit 1;
  }

  avg_false_rate = sum_false_rate / rows;
  avg_missed_pct = sum_missed_pct / rows;
  avg_eos = sum_eos / rows;
  avg_handoff = sum_handoff / rows;
  avg_failover = sum_failover / rows;
  avg_noisy_recovery = sum_noisy_recovery / rows;
  avg_multilingual_recall = sum_multilingual_recall / rows;
  avg_audit = sum_audit / rows;
  avg_tenant = sum_tenant / rows;

  printf("PH1K_EVAL_SUMMARY:rows=%d,avg_false_interrupt_rate_per_hour=%.4f,avg_missed_interrupt_rate_pct=%.2f,avg_end_of_speech_p95_ms=%.2f,avg_capture_to_ph1c_handoff_p95_ms=%.2f,avg_device_failover_recovery_p95_ms=%.2f,avg_noisy_recovery_success_pct=%.2f,avg_multilingual_interrupt_recall_pct=%.2f,avg_audit_completeness_pct=%.2f,avg_tenant_isolation_pct=%.2f\n", rows, avg_false_rate, avg_missed_pct, avg_eos, avg_handoff, avg_failover, avg_noisy_recovery, avg_multilingual_recall, avg_audit, avg_tenant);
  printf("PH1K_EVAL_WORST:rows=%d,max_false_interrupt_rate_per_hour=%.4f,max_missed_interrupt_rate_pct=%.2f,max_end_of_speech_p95_ms=%d,max_capture_to_ph1c_handoff_p95_ms=%d,max_device_failover_recovery_p95_ms=%d,min_noisy_recovery_success_pct=%.2f,min_multilingual_interrupt_recall_pct=%.2f,min_audit_completeness_pct=%.2f,min_tenant_isolation_pct=%.2f\n", rows, max_false_rate, max_missed_pct, max_eos, max_handoff, max_failover, min_noisy_recovery, min_multilingual_recall, min_audit, min_tenant);
  printf("PH1K_EVAL_TREND:file_order_first=%s,file_order_last=%s,false_rate_delta=%.4f,missed_pct_delta=%.2f,end_of_speech_p95_delta_ms=%d,capture_to_ph1c_handoff_p95_delta_ms=%d\n", first_captured, last_captured, last_false_rate - first_false_rate, last_missed_pct - first_missed_pct, last_eos - first_eos, last_handoff - first_handoff);
  printf("CHECK_OK ph1k_round2_eval_snapshot=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
