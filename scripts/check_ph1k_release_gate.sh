#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

INPUT_CSV="${1:-${SELENE_PH1K_EVAL_SNAPSHOT_PATH:-.dev/ph1k_live_eval_snapshot.csv}}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

# Step 18 consumes Step 17 harness output; fail closed if snapshot shape/coverage is invalid.
./scripts/check_ph1k_round2_eval_snapshot.sh "${INPUT_CSV}" >/dev/null

awk -F',' -v input_csv="${INPUT_CSV}" '
function abs(v) { return v < 0 ? -v : v; }

NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "false_interrupt_rate_per_hour";
  req[3] = "missed_interrupt_rate_pct";
  req[4] = "end_of_speech_p95_ms";
  req[5] = "capture_to_ph1c_handoff_p95_ms";
  req[6] = "device_failover_recovery_p95_ms";
  req[7] = "noisy_recovery_success_pct";
  req[8] = "multilingual_interrupt_recall_pct";
  req[9] = "audit_completeness_pct";
  req[10] = "tenant_isolation_pct";
  for (i = 1; i <= 10; i++) {
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
  captured = $(col["captured_at_utc"]);
  false_rate = $(col["false_interrupt_rate_per_hour"]) + 0.0;
  missed_pct = $(col["missed_interrupt_rate_pct"]) + 0.0;
  eos_p95 = $(col["end_of_speech_p95_ms"]) + 0;
  handoff_p95 = $(col["capture_to_ph1c_handoff_p95_ms"]) + 0;
  failover_p95 = $(col["device_failover_recovery_p95_ms"]) + 0;
  noisy_recovery = $(col["noisy_recovery_success_pct"]) + 0.0;
  multilingual_recall = $(col["multilingual_interrupt_recall_pct"]) + 0.0;
  audit_pct = $(col["audit_completeness_pct"]) + 0.0;
  tenant_pct = $(col["tenant_isolation_pct"]) + 0.0;

  if (captured == "captured_at_utc") {
    next;
  }

  rows++;

  if (captured == "") {
    printf("GATE_FAIL:missing_capture_timestamp row=%d\n", NR);
    fail = 1;
    next;
  }

  if (rows == 1) {
    max_false_rate = false_rate;
    max_missed_pct = missed_pct;
    max_eos_p95 = eos_p95;
    max_handoff_p95 = handoff_p95;
    max_failover_p95 = failover_p95;
    min_noisy_recovery = noisy_recovery;
    min_multilingual_recall = multilingual_recall;
    min_audit_pct = audit_pct;
    min_tenant_pct = tenant_pct;
  } else {
    if (false_rate > max_false_rate) max_false_rate = false_rate;
    if (missed_pct > max_missed_pct) max_missed_pct = missed_pct;
    if (eos_p95 > max_eos_p95) max_eos_p95 = eos_p95;
    if (handoff_p95 > max_handoff_p95) max_handoff_p95 = handoff_p95;
    if (failover_p95 > max_failover_p95) max_failover_p95 = failover_p95;
    if (noisy_recovery < min_noisy_recovery) min_noisy_recovery = noisy_recovery;
    if (multilingual_recall < min_multilingual_recall) min_multilingual_recall = multilingual_recall;
    if (audit_pct < min_audit_pct) min_audit_pct = audit_pct;
    if (tenant_pct < min_tenant_pct) min_tenant_pct = tenant_pct;
  }

  if (false_rate > 0.3) {
    printf("GATE_FAIL:false_interrupt_rate_gt_0.3_per_hour row=%d captured_at_utc=%s value=%.4f\n", NR, captured, false_rate);
    fail = 1;
  }
  if (missed_pct > 2.0) {
    printf("GATE_FAIL:missed_interrupt_rate_gt_2pct row=%d captured_at_utc=%s value=%.2f\n", NR, captured, missed_pct);
    fail = 1;
  }
  if (eos_p95 > 180) {
    printf("GATE_FAIL:end_of_speech_p95_gt_180ms row=%d captured_at_utc=%s value=%d\n", NR, captured, eos_p95);
    fail = 1;
  }
  if (handoff_p95 > 120) {
    printf("GATE_FAIL:capture_to_ph1c_handoff_p95_gt_120ms row=%d captured_at_utc=%s value=%d\n", NR, captured, handoff_p95);
    fail = 1;
  }
  if (failover_p95 > 1500) {
    printf("GATE_FAIL:device_failover_recovery_p95_gt_1500ms row=%d captured_at_utc=%s value=%d\n", NR, captured, failover_p95);
    fail = 1;
  }
  if (noisy_recovery < 97.0) {
    printf("GATE_FAIL:noisy_recovery_success_lt_97pct row=%d captured_at_utc=%s value=%.2f\n", NR, captured, noisy_recovery);
    fail = 1;
  }
  if (multilingual_recall < 95.0) {
    printf("GATE_FAIL:multilingual_interrupt_recall_lt_95pct row=%d captured_at_utc=%s value=%.2f\n", NR, captured, multilingual_recall);
    fail = 1;
  }
  if (abs(audit_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:audit_completeness_not_100pct row=%d captured_at_utc=%s value=%.2f\n", NR, captured, audit_pct);
    fail = 1;
  }
  if (abs(tenant_pct - 100.0) > 0.0001) {
    printf("GATE_FAIL:tenant_isolation_not_100pct row=%d captured_at_utc=%s value=%.2f\n", NR, captured, tenant_pct);
    fail = 1;
  }

  printf("PH1K_RELEASE_ROW:captured_at_utc=%s,false_interrupt_rate_per_hour=%.4f,missed_interrupt_rate_pct=%.2f,end_of_speech_p95_ms=%d,capture_to_ph1c_handoff_p95_ms=%d,device_failover_recovery_p95_ms=%d,noisy_recovery_success_pct=%.2f,multilingual_interrupt_recall_pct=%.2f,audit_completeness_pct=%.2f,tenant_isolation_pct=%.2f\n", captured, false_rate, missed_pct, eos_p95, handoff_p95, failover_p95, noisy_recovery, multilingual_recall, audit_pct, tenant_pct);
}

END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "GATE_FAIL:no_metric_rows";
    exit 1;
  }
  if (fail) {
    exit 1;
  }
  printf("PH1K_RELEASE_WORST:false_interrupt_rate_per_hour_max=%.4f,missed_interrupt_rate_pct_max=%.2f,end_of_speech_p95_ms_max=%d,capture_to_ph1c_handoff_p95_ms_max=%d,device_failover_recovery_p95_ms_max=%d,noisy_recovery_success_pct_min=%.2f,multilingual_interrupt_recall_pct_min=%.2f,audit_completeness_pct_min=%.2f,tenant_isolation_pct_min=%.2f\n", max_false_rate, max_missed_pct, max_eos_p95, max_handoff_p95, max_failover_p95, min_noisy_recovery, min_multilingual_recall, min_audit_pct, min_tenant_pct);
  printf("CHECK_OK ph1k_release_gate=pass input=%s rows=%d\n", input_csv, rows);
}
' "${INPUT_CSV}"
