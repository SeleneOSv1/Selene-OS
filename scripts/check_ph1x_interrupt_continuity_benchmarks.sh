#!/usr/bin/env bash
set -euo pipefail

INPUT_CSV="${1:-docs/fixtures/ph1x_interrupt_continuity_snapshot.csv}"

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
  req[1] = "window_min";
  req[2] = "same_subject_total";
  req[3] = "same_subject_correct";
  req[4] = "switch_topic_total";
  req[5] = "switch_topic_correct";
  req[6] = "resume_buffer_total";
  req[7] = "resume_buffer_retained";
  req[8] = "branch_decision_latency_p95_ms";
  req[9] = "audit_completeness_bp";
  for (i = 1; i <= 9; i++) {
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
  w = $(col["window_min"]) + 0;
  same_total = $(col["same_subject_total"]) + 0;
  same_correct = $(col["same_subject_correct"]) + 0;
  switch_total = $(col["switch_topic_total"]) + 0;
  switch_correct = $(col["switch_topic_correct"]) + 0;
  resume_total = $(col["resume_buffer_total"]) + 0;
  resume_retained = $(col["resume_buffer_retained"]) + 0;
  latency_p95 = $(col["branch_decision_latency_p95_ms"]) + 0;
  audit_bp = $(col["audit_completeness_bp"]) + 0;

  if (w <= 0 || same_total <= 0 || switch_total <= 0 || resume_total <= 0) {
    printf("GATE_FAIL:invalid_non_positive_totals row=%d\n", NR);
    fail = 1;
    next;
  }
  if (same_correct < 0 || same_correct > same_total) {
    printf("GATE_FAIL:invalid_same_subject_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (switch_correct < 0 || switch_correct > switch_total) {
    printf("GATE_FAIL:invalid_switch_topic_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (resume_retained < 0 || resume_retained > resume_total) {
    printf("GATE_FAIL:invalid_resume_buffer_counts row=%d\n", NR);
    fail = 1;
    next;
  }
  if (latency_p95 < 0) {
    printf("GATE_FAIL:invalid_latency_p95 row=%d\n", NR);
    fail = 1;
    next;
  }
  if (audit_bp < 0 || audit_bp > 10000) {
    printf("GATE_FAIL:invalid_audit_completeness_bp row=%d\n", NR);
    fail = 1;
    next;
  }

  same_bp = int(((same_correct * 10000.0) / same_total) + 0.5);
  switch_bp = int(((switch_correct * 10000.0) / switch_total) + 0.5);
  resume_bp = int(((resume_retained * 10000.0) / resume_total) + 0.5);

  if (same_bp < 9800) {
    printf("GATE_FAIL:same_subject_merge_correctness_lt_98pct row=%d bp=%d\n", NR, same_bp);
    fail = 1;
  }
  if (switch_bp < 9800) {
    printf("GATE_FAIL:switch_topic_return_check_correctness_lt_98pct row=%d bp=%d\n", NR, switch_bp);
    fail = 1;
  }
  if (resume_bp < 9950) {
    printf("GATE_FAIL:resume_buffer_retention_correctness_lt_99_5pct row=%d bp=%d\n", NR, resume_bp);
    fail = 1;
  }
  if (latency_p95 > 120) {
    printf("GATE_FAIL:interrupt_branch_decision_latency_p95_gt_120ms row=%d latency_p95_ms=%d\n", NR, latency_p95);
    fail = 1;
  }
  if (audit_bp < 10000) {
    printf("GATE_FAIL:audit_completeness_not_100pct row=%d audit_completeness_bp=%d\n", NR, audit_bp);
    fail = 1;
  }

  printf("PH1X_BENCH:window_min=%d,same_subject_bp=%d,switch_topic_bp=%d,resume_buffer_bp=%d,branch_latency_p95_ms=%d,audit_bp=%d\n", w, same_bp, switch_bp, resume_bp, latency_p95, audit_bp);
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
  printf("CHECK_OK ph1x_interrupt_continuity_benchmarks=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
