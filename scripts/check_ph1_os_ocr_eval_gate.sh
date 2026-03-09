#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="${SELENE_ROOT:-$(git rev-parse --show-toplevel)}"
cd "${ROOT_DIR}"

INPUT_CSV="${1:-docs/fixtures/ph1_os_ocr_eval_snapshot.csv}"

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
  req[2] = "quality_regression_bp";
  req[3] = "latency_p95_delta_bp";
  req[4] = "latency_p99_delta_bp";
  req[5] = "cost_delta_bp";
  req[6] = "provider_route_success_bp";
  req[7] = "local_route_success_bp";
  req[8] = "provider_fallback_rate_bp";
  req[9] = "route_compare_coverage_bp";
  req[10] = "scorecard_complete";
  req[11] = "audit_completeness_bp";
  for (i = 1; i <= 11; i++) {
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
  quality = $(col["quality_regression_bp"]) + 0;
  p95 = $(col["latency_p95_delta_bp"]) + 0;
  p99 = $(col["latency_p99_delta_bp"]) + 0;
  cost = $(col["cost_delta_bp"]) + 0;
  provider_success = $(col["provider_route_success_bp"]) + 0;
  local_success = $(col["local_route_success_bp"]) + 0;
  fallback_rate = $(col["provider_fallback_rate_bp"]) + 0;
  route_coverage = $(col["route_compare_coverage_bp"]) + 0;
  scorecard_complete = $(col["scorecard_complete"]) + 0;
  audit = $(col["audit_completeness_bp"]) + 0;

  if (scorecard_complete != 1) {
    printf("GATE_FAIL:scorecard_incomplete row=%d\n", NR);
    fail = 1;
  }
  if (quality > 50) {
    printf("GATE_FAIL:quality_regression_gt_0_5pct row=%d quality_regression_bp=%d\n", NR, quality);
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
  if (cost > 500) {
    printf("GATE_FAIL:cost_regression_gt_5pct row=%d cost_delta_bp=%d\n", NR, cost);
    fail = 1;
  }
  if (provider_success < 9800) {
    printf("GATE_FAIL:provider_route_success_lt_98pct row=%d provider_route_success_bp=%d\n", NR, provider_success);
    fail = 1;
  }
  if (local_success < 9000) {
    printf("GATE_FAIL:local_route_success_lt_90pct row=%d local_route_success_bp=%d\n", NR, local_success);
    fail = 1;
  }
  if (fallback_rate > 2500) {
    printf("GATE_FAIL:provider_fallback_rate_gt_25pct row=%d provider_fallback_rate_bp=%d\n", NR, fallback_rate);
    fail = 1;
  }
  if (route_coverage < 10000) {
    printf("GATE_FAIL:route_compare_coverage_not_100pct row=%d route_compare_coverage_bp=%d\n", NR, route_coverage);
    fail = 1;
  }
  if (audit < 10000) {
    printf("GATE_FAIL:audit_completeness_not_100pct row=%d audit_completeness_bp=%d\n", NR, audit);
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
  printf("CHECK_OK ph1_os_ocr_eval_gate=pass rows=%d\n", rows);
}
' "${INPUT_CSV}"
