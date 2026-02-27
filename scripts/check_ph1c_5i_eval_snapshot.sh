#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

INPUT_CSV="${1:-docs/fixtures/ph1c_5i_superiority_snapshot.csv}"
if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

awk -F',' -v input_csv="${INPUT_CSV}" '
NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  req[1] = "captured_at_utc";
  req[2] = "commit_hash";
  req[3] = "slice_locale";
  req[4] = "slice_device_route";
  req[5] = "slice_tenant_id";
  req[6] = "lane";
  req[7] = "turns";
  req[8] = "transcript_accuracy_bp";
  req[9] = "semantic_accuracy_bp";
  req[10] = "entity_accuracy_bp";
  req[11] = "intent_success_bp";
  req[12] = "partial_first_chunk_p95_ms";
  req[13] = "eos_to_first_token_p95_ms";
  req[14] = "clarify_one_shot_bp";
  req[15] = "audit_completeness_bp";
  req[16] = "tenant_isolation_bp";
  req[17] = "overlap_attribution_bp";
  req[18] = "diarization_f1_bp";
  req[19] = "cost_microunits_per_turn";
  req[20] = "lexicon_v2_hit_bp";
  req[21] = "lexicon_freshness_bp";
  req[22] = "provider_disagreement_bp";
  req[23] = "disagreement_resolved_bp";
  req[24] = "gold_label_rate_bp";
  req[25] = "silver_label_rate_bp";
  req[26] = "distillation_coverage_bp";
  req[27] = "disagreement_queue_coverage_bp";
  req[28] = "active_learning_topk_recall_bp";
  req[29] = "hard_negative_replay_coverage_bp";
  req[30] = "cadence_on_time_bp";
  req[31] = "rollback_readiness_bp";
  req[32] = "per_slice_adapter_ready";
  req[33] = "slice_promotion_proof";
  req[34] = "current_mode";
  for (i = 1; i <= 34; i++) {
    if (!(req[i] in col)) {
      printf("MISSING_COLUMN:%s\n", req[i]);
      missing = 1;
    }
  }
  next;
}

function parse_bool(v,    out) {
  out = tolower(v);
  if (out == "true" || out == "false") {
    return out;
  }
  return "invalid";
}

function bp_valid(v) {
  return v >= 0 && v <= 10000;
}

NR > 1 {
  gsub(/\r/, "", $0);
  if ($0 == "") {
    next;
  }

  rows++;
  captured = $(col["captured_at_utc"]);
  commit_hash = $(col["commit_hash"]);
  locale = $(col["slice_locale"]);
  route = $(col["slice_device_route"]);
  tenant = $(col["slice_tenant_id"]);
  lane = $(col["lane"]);

  if (captured == "" || commit_hash == "" || locale == "" || route == "" || tenant == "") {
    printf("SNAPSHOT_FAIL:missing_identity_fields row=%d\n", NR);
    fail = 1;
    next;
  }

  if (!(lane == "SELENE_BASELINE" || lane == "SELENE_CHALLENGER" || lane == "CHATGPT_AB")) {
    printf("SNAPSHOT_FAIL:invalid_lane row=%d lane=%s\n", NR, lane);
    fail = 1;
    next;
  }

  turns = $(col["turns"]) + 0;
  if (turns <= 0) {
    printf("SNAPSHOT_FAIL:turns_must_be_gt_zero row=%d\n", NR);
    fail = 1;
    next;
  }

  partial_p95 = $(col["partial_first_chunk_p95_ms"]) + 0;
  eos_p95 = $(col["eos_to_first_token_p95_ms"]) + 0;
  cost = $(col["cost_microunits_per_turn"]) + 0;
  if (partial_p95 <= 0 || eos_p95 <= 0 || cost <= 0) {
    printf("SNAPSHOT_FAIL:invalid_latency_or_cost row=%d\n", NR);
    fail = 1;
    next;
  }

  bp_names[1] = "transcript_accuracy_bp";
  bp_names[2] = "semantic_accuracy_bp";
  bp_names[3] = "entity_accuracy_bp";
  bp_names[4] = "intent_success_bp";
  bp_names[5] = "clarify_one_shot_bp";
  bp_names[6] = "audit_completeness_bp";
  bp_names[7] = "tenant_isolation_bp";
  bp_names[8] = "overlap_attribution_bp";
  bp_names[9] = "diarization_f1_bp";
  bp_names[10] = "lexicon_v2_hit_bp";
  bp_names[11] = "lexicon_freshness_bp";
  bp_names[12] = "provider_disagreement_bp";
  bp_names[13] = "disagreement_resolved_bp";
  bp_names[14] = "gold_label_rate_bp";
  bp_names[15] = "silver_label_rate_bp";
  bp_names[16] = "distillation_coverage_bp";
  bp_names[17] = "disagreement_queue_coverage_bp";
  bp_names[18] = "active_learning_topk_recall_bp";
  bp_names[19] = "hard_negative_replay_coverage_bp";
  bp_names[20] = "cadence_on_time_bp";
  bp_names[21] = "rollback_readiness_bp";

  for (i = 1; i <= 21; i++) {
    metric_name = bp_names[i];
    metric_value = $(col[metric_name]) + 0;
    if (!bp_valid(metric_value)) {
      printf("SNAPSHOT_FAIL:bp_out_of_range row=%d metric=%s value=%d\n", NR, metric_name, metric_value);
      fail = 1;
      next;
    }
  }

  gold_bp = $(col["gold_label_rate_bp"]) + 0;
  silver_bp = $(col["silver_label_rate_bp"]) + 0;
  if (gold_bp + silver_bp > 10000) {
    printf("SNAPSHOT_FAIL:gold_plus_silver_exceeds_10000 row=%d\n", NR);
    fail = 1;
    next;
  }

  adapter_ready = parse_bool($(col["per_slice_adapter_ready"]));
  promotion_proof = parse_bool($(col["slice_promotion_proof"]));
  if (adapter_ready == "invalid" || promotion_proof == "invalid") {
    printf("SNAPSHOT_FAIL:boolean_field_invalid row=%d\n", NR);
    fail = 1;
    next;
  }

  current_mode = $(col["current_mode"]);
  if (!(current_mode == "SHADOW" || current_mode == "ASSIST" || current_mode == "LEAD")) {
    printf("SNAPSHOT_FAIL:invalid_current_mode row=%d mode=%s\n", NR, current_mode);
    fail = 1;
    next;
  }

  key = locale "|" route "|" tenant;
  unique = key "|" lane;
  if (seen_row[unique] == 1) {
    printf("SNAPSHOT_FAIL:duplicate_slice_lane row=%d key=%s lane=%s\n", NR, key, lane);
    fail = 1;
    next;
  }
  seen_row[unique] = 1;
  seen_slice[key] = 1;
  seen_lane[lane] = 1;
}

END {
  if (missing) {
    exit 1;
  }
  if (rows == 0) {
    print "SNAPSHOT_FAIL:no_rows";
    exit 1;
  }
  for (lane in seen_lane) {
    lane_count++;
  }
  for (slice in seen_slice) {
    slice_count++;
    if (!seen_row[slice "|SELENE_BASELINE"] || !seen_row[slice "|SELENE_CHALLENGER"] || !seen_row[slice "|CHATGPT_AB"]) {
      printf("SNAPSHOT_FAIL:lane_triplet_missing slice=%s\n", slice);
      fail = 1;
    }
  }
  if (lane_count != 3) {
    printf("SNAPSHOT_FAIL:expected_three_lanes_seen found=%d\n", lane_count);
    fail = 1;
  }
  if (slice_count == 0) {
    print "SNAPSHOT_FAIL:no_slices";
    fail = 1;
  }
  if (fail) {
    exit 1;
  }
  printf("PH1C_5I_SNAPSHOT_SUMMARY:rows=%d,slices=%d,lanes=%d\n", rows, slice_count, lane_count);
  printf("CHECK_OK ph1c_5i_eval_snapshot=pass input=%s rows=%d slices=%d\n", input_csv, rows, slice_count);
}
' "${INPUT_CSV}"
