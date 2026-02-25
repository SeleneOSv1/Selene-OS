#!/usr/bin/env bash
set -euo pipefail

INPUT_CSV="${1:-docs/fixtures/ph1k_round2_baseline_snapshot.csv}"

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
  req[3] = "window_min";
  req[4] = "active_session_hours";
  req[5] = "interrupt_attempts";
  req[6] = "false_interrupt_count";
  req[7] = "missed_interrupt_count";
  req[8] = "vad_boundary_p95_ms";
  req[9] = "device_failover_recovery_p95_ms";
  req[10] = "aec_unstable_count";
  req[11] = "stream_gap_detected_count";
  req[12] = "device_changed_count";
  req[13] = "kernel_contract_tests_pass";
  req[14] = "engine_tests_pass";
  req[15] = "os_tests_pass";
  for (i = 1; i <= 15; i++) {
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
  active_hours = $(col["active_session_hours"]) + 0.0;
  attempts = $(col["interrupt_attempts"]) + 0;
  false_count = $(col["false_interrupt_count"]) + 0;
  missed_count = $(col["missed_interrupt_count"]) + 0;
  vad_p95 = $(col["vad_boundary_p95_ms"]) + 0;
  failover_p95 = $(col["device_failover_recovery_p95_ms"]) + 0;
  aec_count = $(col["aec_unstable_count"]) + 0;
  gap_count = $(col["stream_gap_detected_count"]) + 0;
  device_changed_count = $(col["device_changed_count"]) + 0;
  ktest = $(col["kernel_contract_tests_pass"]) + 0;
  etest = $(col["engine_tests_pass"]) + 0;
  otest = $(col["os_tests_pass"]) + 0;

  if (captured == "" || commit_hash == "") {
    printf("BASELINE_FAIL:missing_capture_or_commit row=%d\n", NR);
    fail = 1;
    next;
  }
  if (window_min <= 0 || active_hours <= 0 || attempts <= 0) {
    printf("BASELINE_FAIL:invalid_window_or_activity row=%d\n", NR);
    fail = 1;
    next;
  }
  if (false_count < 0 || false_count > attempts) {
    printf("BASELINE_FAIL:invalid_false_interrupt_count row=%d\n", NR);
    fail = 1;
    next;
  }
  if (missed_count < 0 || missed_count > attempts) {
    printf("BASELINE_FAIL:invalid_missed_interrupt_count row=%d\n", NR);
    fail = 1;
    next;
  }
  if (vad_p95 < 0 || failover_p95 < 0 || aec_count < 0 || gap_count < 0 || device_changed_count < 0) {
    printf("BASELINE_FAIL:invalid_latency_or_degradation_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (ktest != 1 || etest != 1 || otest != 1) {
    printf("BASELINE_FAIL:test_proof_missing row=%d\n", NR);
    fail = 1;
    next;
  }

  false_rate_per_hour = false_count / active_hours;
  missed_bp = int(((missed_count * 10000.0) / attempts) + 0.5);

  printf("PH1K_BASELINE:captured_at_utc=%s,commit_hash=%s,false_interrupt_rate_per_hour=%.4f,missed_interrupt_bp=%d,vad_boundary_p95_ms=%d,device_failover_recovery_p95_ms=%d,aec_unstable_count=%d,stream_gap_detected_count=%d,device_changed_count=%d\n", captured, commit_hash, false_rate_per_hour, missed_bp, vad_p95, failover_p95, aec_count, gap_count, device_changed_count);
}
END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "BASELINE_FAIL:no_rows";
    exit 1;
  }
  if (fail) {
    exit 1;
  }
  printf("CHECK_OK ph1k_round2_baseline_snapshot=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
