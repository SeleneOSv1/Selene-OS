#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  ./scripts/check_optional_engine_utility_gates.sh [snapshot_csv] [--fail-on-u4]

Input CSV header (required, exact):
engine_id,decision_delta_rate,queue_learn_conversion_rate,no_value_rate,latency_cost_p95_ms,latency_cost_p99_ms,fail_streak_days

Exit codes:
  0 = pass
  1 = gate fail (U5 triggered, or U4 fail when --fail-on-u4 is set)
  2 = input file missing
  3 = malformed input
EOF
}

SNAPSHOT_CSV=""
FAIL_ON_U4=0

for arg in "$@"; do
  case "$arg" in
    --fail-on-u4)
      FAIL_ON_U4=1
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      if [ -n "$SNAPSHOT_CSV" ]; then
        echo "ERROR: only one snapshot_csv path is allowed"
        usage
        exit 3
      fi
      SNAPSHOT_CSV="$arg"
      ;;
  esac
done

if [ -z "$SNAPSHOT_CSV" ]; then
  SNAPSHOT_CSV="artifacts/optional_engine_utility_window.csv"
fi

if [ ! -f "$SNAPSHOT_CSV" ]; then
  echo "MISSING_SNAPSHOT_CSV:$SNAPSHOT_CSV"
  exit 2
fi

awk -F',' -v fail_on_u4="$FAIL_ON_U4" '
BEGIN {
  expected_header = "engine_id,decision_delta_rate,queue_learn_conversion_rate,no_value_rate,latency_cost_p95_ms,latency_cost_p99_ms,fail_streak_days";
  u4_fail = 0;
  u5_fail = 0;
  rows = 0;
}
NR == 1 {
  gsub(/\r/, "", $0);
  if ($0 != expected_header) {
    print "INVALID_HEADER:" $0;
    exit 3;
  }
  next;
}
{
  gsub(/\r/, "", $0);
  if ($0 == "") {
    next;
  }
  if (NF != 7) {
    print "INVALID_ROW_FIELD_COUNT:line=" NR;
    exit 3;
  }

  engine_id = $1;
  decision_delta_rate = $2 + 0.0;
  queue_learn_conversion_rate = $3 + 0.0;
  no_value_rate = $4 + 0.0;
  latency_cost_p95_ms = $5 + 0;
  latency_cost_p99_ms = $6 + 0;
  fail_streak_days = $7 + 0;

  if (engine_id == "") {
    print "INVALID_ROW_ENGINE_ID_EMPTY:line=" NR;
    exit 3;
  }
  if (decision_delta_rate < 0 || decision_delta_rate > 1) {
    print "INVALID_DECISION_DELTA_RATE:line=" NR;
    exit 3;
  }
  if (queue_learn_conversion_rate < 0 || queue_learn_conversion_rate > 1) {
    print "INVALID_QUEUE_LEARN_CONVERSION_RATE:line=" NR;
    exit 3;
  }
  if (no_value_rate < 0 || no_value_rate > 1) {
    print "INVALID_NO_VALUE_RATE:line=" NR;
    exit 3;
  }
  if (latency_cost_p95_ms < 0 || latency_cost_p99_ms < 0 || fail_streak_days < 0) {
    print "INVALID_NON_NEGATIVE_FIELDS:line=" NR;
    exit 3;
  }

  gate_u4_pass = ((decision_delta_rate >= 0.08 || queue_learn_conversion_rate >= 0.20) && no_value_rate <= 0.60 && latency_cost_p95_ms <= 20 && latency_cost_p99_ms <= 40) ? 1 : 0;
  gate_u5_trigger = (gate_u4_pass == 0 && fail_streak_days >= 7) ? 1 : 0;
  action = gate_u4_pass ? "KEEP" : (gate_u5_trigger ? "DISABLE_CANDIDATE" : "DEGRADE");

  print "UTILITY_REVIEW:engine_id=" engine_id ",gate_u4_pass=" gate_u4_pass ",gate_u5_trigger=" gate_u5_trigger ",action=" action;

  rows++;
  if (gate_u4_pass == 0) {
    u4_fail++;
  }
  if (gate_u5_trigger == 1) {
    u5_fail++;
  }
}
END {
  if (rows == 0) {
    print "INVALID_EMPTY_DATA_ROWS";
    exit 3;
  }

  print "UTILITY_REVIEW_SUMMARY:rows=" rows ",u4_fail_count=" u4_fail ",u5_trigger_count=" u5_fail ",fail_on_u4=" fail_on_u4;

  if (u5_fail > 0) {
    exit 1;
  }
  if (fail_on_u4 == 1 && u4_fail > 0) {
    exit 1;
  }
}
' "$SNAPSHOT_CSV"
