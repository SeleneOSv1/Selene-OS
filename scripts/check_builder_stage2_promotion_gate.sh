#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${SELENE_ROOT:-$(git rev-parse --show-toplevel)}"
cd "${ROOT_DIR}"

INPUT_CSV="${1:-docs/fixtures/stage2_canary_metrics_snapshot.csv}"

if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

awk -F',' '
NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "window_min";
  req[2] = "p95_delta_bp";
  req[3] = "p99_delta_bp";
  req[4] = "critical_reason_spike_bp";
  req[5] = "authority_or_gate_order_violation";
  req[6] = "duplicate_side_effect_event";
  req[7] = "audit_completeness_bp";
  req[8] = "fail_closed_delta_bp";
  for (i = 1; i <= 8; i++) {
    if (!(req[i] in col)) {
      printf("MISSING_COLUMN:%s\n", req[i]);
      missing = 1;
    }
  }
  next;
}
NR > 1 {
  rows++;
  w = $(col["window_min"]) + 0;
  p95 = $(col["p95_delta_bp"]) + 0;
  p99 = $(col["p99_delta_bp"]) + 0;
  critical = $(col["critical_reason_spike_bp"]) + 0;
  authority = $(col["authority_or_gate_order_violation"]) + 0;
  dup = $(col["duplicate_side_effect_event"]) + 0;
  audit = $(col["audit_completeness_bp"]) + 0;
  fail_closed = $(col["fail_closed_delta_bp"]) + 0;

  if (authority != 0) {
    printf("GATE_FAIL:authority_or_gate_order_violation row=%d\n", NR);
    fail = 1;
  }
  if (dup != 0) {
    printf("GATE_FAIL:duplicate_side_effect_event row=%d\n", NR);
    fail = 1;
  }
  if (p95 > 300 && w >= 30) {
    printf("GATE_FAIL:p95_regression_gt_3pct_30m row=%d p95_delta_bp=%d window_min=%d\n", NR, p95, w);
    fail = 1;
  }
  if (p99 > 500 && w >= 15) {
    printf("GATE_FAIL:p99_regression_gt_5pct_15m row=%d p99_delta_bp=%d window_min=%d\n", NR, p99, w);
    fail = 1;
  }
  if (critical > 20) {
    printf("GATE_FAIL:critical_reason_spike_gt_0_2pct row=%d critical_reason_spike_bp=%d\n", NR, critical);
    fail = 1;
  }
  if (audit < 10000) {
    printf("GATE_FAIL:audit_completeness_not_100pct row=%d audit_completeness_bp=%d\n", NR, audit);
    fail = 1;
  }
  if (fail_closed > 20) {
    printf("GATE_FAIL:fail_closed_delta_gt_0_2pct row=%d fail_closed_delta_bp=%d\n", NR, fail_closed);
    fail = 1;
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
  if (fail) {
    exit 1;
  }
  printf("CHECK_OK builder_stage2_promotion_gate=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
