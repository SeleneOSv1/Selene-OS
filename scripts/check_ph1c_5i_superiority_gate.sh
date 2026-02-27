#!/usr/bin/env bash
set -euo pipefail

cd "$(git rev-parse --show-toplevel)"

INPUT_CSV="${1:-docs/fixtures/ph1c_5i_superiority_snapshot.csv}"
if [[ ! -f "${INPUT_CSV}" ]]; then
  echo "MISSING_INPUT:${INPUT_CSV}"
  exit 1
fi

./scripts/check_ph1c_5i_eval_snapshot.sh "${INPUT_CSV}" >/dev/null

awk -F',' -v input_csv="${INPUT_CSV}" '
NR == 1 {
  for (i = 1; i <= NF; i++) {
    gsub(/\r/, "", $i);
    gsub(/^[ \t]+|[ \t]+$/, "", $i);
    col[$i] = i;
  }
  next;
}

function b(v) { return v + 0; }
function key(locale, route, tenant) { return locale "|" route "|" tenant; }
function row_key(slice_key, lane) { return slice_key SUBSEP lane; }

NR > 1 {
  gsub(/\r/, "", $0);
  if ($0 == "") {
    next;
  }

  rows++;
  slice = key($(col["slice_locale"]), $(col["slice_device_route"]), $(col["slice_tenant_id"]));
  lane = $(col["lane"]);
  rk = row_key(slice, lane);

  seen_slice[slice] = 1;
  seen_lane[rk] = 1;

  transcript_bp[rk] = b($(col["transcript_accuracy_bp"]));
  semantic_bp[rk] = b($(col["semantic_accuracy_bp"]));
  entity_bp[rk] = b($(col["entity_accuracy_bp"]));
  intent_bp[rk] = b($(col["intent_success_bp"]));
  partial_p95[rk] = b($(col["partial_first_chunk_p95_ms"]));
  eos_p95[rk] = b($(col["eos_to_first_token_p95_ms"]));
  clarify_bp[rk] = b($(col["clarify_one_shot_bp"]));
  audit_bp[rk] = b($(col["audit_completeness_bp"]));
  isolation_bp[rk] = b($(col["tenant_isolation_bp"]));
  overlap_bp[rk] = b($(col["overlap_attribution_bp"]));
  diar_bp[rk] = b($(col["diarization_f1_bp"]));
  cost_u[rk] = b($(col["cost_microunits_per_turn"]));
  lex_hit_bp[rk] = b($(col["lexicon_v2_hit_bp"]));
  lex_fresh_bp[rk] = b($(col["lexicon_freshness_bp"]));
  disagree_bp[rk] = b($(col["provider_disagreement_bp"]));
  disagree_resolved_bp[rk] = b($(col["disagreement_resolved_bp"]));
  gold_bp[rk] = b($(col["gold_label_rate_bp"]));
  silver_bp[rk] = b($(col["silver_label_rate_bp"]));
  distill_bp[rk] = b($(col["distillation_coverage_bp"]));
  disagree_queue_bp[rk] = b($(col["disagreement_queue_coverage_bp"]));
  active_bp[rk] = b($(col["active_learning_topk_recall_bp"]));
  hard_neg_bp[rk] = b($(col["hard_negative_replay_coverage_bp"]));
  cadence_bp[rk] = b($(col["cadence_on_time_bp"]));
  rollback_bp[rk] = b($(col["rollback_readiness_bp"]));

  adapter_ready[rk] = tolower($(col["per_slice_adapter_ready"]));
  promotion_proof[rk] = tolower($(col["slice_promotion_proof"]));
  mode[rk] = $(col["current_mode"]);
}

END {
  if (rows == 0) {
    print "STEP_FAIL:no_rows";
    exit 1;
  }

  step1_ok = 1;
  step2_ok = 1;
  step3_ok = 1;
  step4_ok = 1;
  step5_ok = 1;
  step6_ok = 1;
  step7_ok = 1;
  step8_ok = 1;
  step9_ok = 1;
  step10_ok = 1;
  step11_ok = 1;
  step12_ok = 1;
  step13_ok = 1;
  step14_ok = 1;
  step15_ok = 1;
  step16_ok = 1;
  step17_ok = 1;
  step18_ok = 1;
  step19_ok = 1;
  step20_ok = 1;
  step21_ok = 1;
  step22_ok = 1;

  complete_slices = 0;
  for (slice in seen_slice) {
    b_rk = row_key(slice, "SELENE_BASELINE");
    c_rk = row_key(slice, "SELENE_CHALLENGER");
    g_rk = row_key(slice, "CHATGPT_AB");
    if (!seen_lane[b_rk] || !seen_lane[c_rk] || !seen_lane[g_rk]) {
      printf("STEP1_FAIL:missing_lane_triplet slice=%s\n", slice);
      step1_ok = 0;
      continue;
    }
    complete_slices++;

    # Step 2
    if (semantic_bp[c_rk] < 9600) {
      printf("STEP2_FAIL:semantic_accuracy_lt_9600 slice=%s value=%d\n", slice, semantic_bp[c_rk]);
      step2_ok = 0;
    }

    # Step 3
    if (lex_hit_bp[c_rk] < 5500 || lex_fresh_bp[c_rk] < 9800) {
      printf("STEP3_FAIL:lexicon_v2_hit_or_freshness slice=%s hit=%d fresh=%d\n", slice, lex_hit_bp[c_rk], lex_fresh_bp[c_rk]);
      step3_ok = 0;
    }

    # Step 4 and Step 19
    if (overlap_bp[c_rk] < 9500 || diar_bp[c_rk] < 9400) {
      printf("STEP4_19_FAIL:overlap_or_diarization slice=%s overlap=%d diar=%d\n", slice, overlap_bp[c_rk], diar_bp[c_rk]);
      step4_ok = 0;
      step19_ok = 0;
    }

    # Step 5
    if (partial_p95[c_rk] > 250 || eos_p95[c_rk] > 300) {
      printf("STEP5_FAIL:latency_budget slice=%s partial_p95=%d eos_p95=%d\n", slice, partial_p95[c_rk], eos_p95[c_rk]);
      step5_ok = 0;
    }

    # Step 6
    if (disagree_bp[c_rk] > 1500 && disagree_resolved_bp[c_rk] < 9000) {
      printf("STEP6_FAIL:provider_disagreement_unresolved slice=%s disagreement=%d resolved=%d\n", slice, disagree_bp[c_rk], disagree_resolved_bp[c_rk]);
      step6_ok = 0;
    }

    # Step 7
    if (intent_bp[c_rk] < 9650 || semantic_bp[c_rk] < 9650) {
      printf("STEP7_FAIL:intent_or_semantic_low slice=%s intent=%d semantic=%d\n", slice, intent_bp[c_rk], semantic_bp[c_rk]);
      step7_ok = 0;
    }

    # Step 8
    if (gold_bp[c_rk] < 9800) {
      printf("STEP8_FAIL:gold_loop_rate_low slice=%s gold=%d\n", slice, gold_bp[c_rk]);
      step8_ok = 0;
    }

    # Step 9
    if (!(adapter_ready[c_rk] == "true" && promotion_proof[c_rk] == "true" && mode[c_rk] != "SHADOW")) {
      printf("STEP9_FAIL:slice_promotion_control slice=%s adapter_ready=%s proof=%s mode=%s\n", slice, adapter_ready[c_rk], promotion_proof[c_rk], mode[c_rk]);
      step9_ok = 0;
    }

    # Step 10
    if (!(transcript_bp[c_rk] >= transcript_bp[b_rk] && transcript_bp[c_rk] >= transcript_bp[g_rk] &&
          semantic_bp[c_rk] >= semantic_bp[b_rk] && semantic_bp[c_rk] >= semantic_bp[g_rk] &&
          entity_bp[c_rk] >= entity_bp[b_rk] && entity_bp[c_rk] >= entity_bp[g_rk] &&
          intent_bp[c_rk] >= intent_bp[b_rk] && intent_bp[c_rk] >= intent_bp[g_rk])) {
      printf("STEP10_FAIL:strict_superiority slice=%s\n", slice);
      step10_ok = 0;
    }

    # Step 11
    quality_ok = (transcript_bp[c_rk] >= 9700 && semantic_bp[c_rk] >= 9600 && intent_bp[c_rk] >= 9700);
    cheaper_ok = (cost_u[c_rk] <= cost_u[b_rk] && cost_u[c_rk] <= cost_u[g_rk]);
    if (!(quality_ok && cheaper_ok)) {
      printf("STEP11_FAIL:cost_quality_policy slice=%s quality_ok=%d challenger_cost=%d baseline_cost=%d chatgpt_cost=%d\n", slice, quality_ok, cost_u[c_rk], cost_u[b_rk], cost_u[g_rk]);
      step11_ok = 0;
    }

    # Step 12
    if (!(clarify_bp[c_rk] >= 9000 && audit_bp[c_rk] == 10000 && isolation_bp[c_rk] == 10000)) {
      printf("STEP12_FAIL:acceptance_pack slice=%s clarify=%d audit=%d isolation=%d\n", slice, clarify_bp[c_rk], audit_bp[c_rk], isolation_bp[c_rk]);
      step12_ok = 0;
    }

    # Step 13
    if (distill_bp[c_rk] < 9000) {
      printf("STEP13_FAIL:distillation_coverage slice=%s value=%d\n", slice, distill_bp[c_rk]);
      step13_ok = 0;
    }

    # Step 14
    if (disagree_queue_bp[c_rk] < 9000) {
      printf("STEP14_FAIL:disagreement_queue_coverage slice=%s value=%d\n", slice, disagree_queue_bp[c_rk]);
      step14_ok = 0;
    }

    # Step 15
    if (active_bp[c_rk] < 9200) {
      printf("STEP15_FAIL:active_learning_priority slice=%s value=%d\n", slice, active_bp[c_rk]);
      step15_ok = 0;
    }

    # Step 16
    if (!(gold_bp[c_rk] >= 7000 && silver_bp[c_rk] <= 3000 && (gold_bp[c_rk] + silver_bp[c_rk]) <= 10000)) {
      printf("STEP16_FAIL:gold_silver_tiering slice=%s gold=%d silver=%d\n", slice, gold_bp[c_rk], silver_bp[c_rk]);
      step16_ok = 0;
    }

    # Step 17
    if (hard_neg_bp[c_rk] < 9000) {
      printf("STEP17_FAIL:hard_negative_replay_coverage slice=%s value=%d\n", slice, hard_neg_bp[c_rk]);
      step17_ok = 0;
    }

    # Step 18
    if (!(entity_bp[c_rk] >= 9700 && intent_bp[c_rk] >= 9700)) {
      printf("STEP18_FAIL:entity_intent_gate slice=%s entity=%d intent=%d\n", slice, entity_bp[c_rk], intent_bp[c_rk]);
      step18_ok = 0;
    }

    # Step 20
    if (adapter_ready[c_rk] != "true") {
      printf("STEP20_FAIL:per_slice_adapter_not_ready slice=%s\n", slice);
      step20_ok = 0;
    }

    # Step 22
    if (!(cadence_bp[c_rk] >= 9800 && rollback_bp[c_rk] == 10000)) {
      printf("STEP22_FAIL:cadence_or_rollback slice=%s cadence=%d rollback=%d\n", slice, cadence_bp[c_rk], rollback_bp[c_rk]);
      step22_ok = 0;
    }
  }

  if (complete_slices == 0) {
    step1_ok = 0;
    step21_ok = 0;
    print "STEP1_FAIL:no_complete_slices";
  }

  # Step 21 champion/challenger runtime decision lock.
  if (!(step10_ok && step12_ok && step20_ok && step1_ok)) {
    step21_ok = 0;
  }

  overall_ok = (step1_ok && step2_ok && step3_ok && step4_ok && step5_ok && step6_ok && step7_ok && step8_ok && step9_ok && step10_ok && step11_ok && step12_ok && step13_ok && step14_ok && step15_ok && step16_ok && step17_ok && step18_ok && step19_ok && step20_ok && step21_ok && step22_ok);

  printf("PH1C_5I_GATE_SUMMARY:rows=%d,complete_slices=%d,step1=%d,step2=%d,step3=%d,step4=%d,step5=%d,step6=%d,step7=%d,step8=%d,step9=%d,step10=%d,step11=%d,step12=%d,step13=%d,step14=%d,step15=%d,step16=%d,step17=%d,step18=%d,step19=%d,step20=%d,step21=%d,step22=%d\n",
    rows, complete_slices, step1_ok, step2_ok, step3_ok, step4_ok, step5_ok, step6_ok, step7_ok, step8_ok, step9_ok, step10_ok, step11_ok, step12_ok, step13_ok, step14_ok, step15_ok, step16_ok, step17_ok, step18_ok, step19_ok, step20_ok, step21_ok, step22_ok);

  if (!overall_ok) {
    exit 1;
  }

  recommended_lane = step21_ok ? "SELENE_CHALLENGER" : "SELENE_BASELINE";
  rollback_required = step22_ok ? "false" : "true";
  printf("CHECK_OK ph1c_5i_superiority_gate=pass input=%s recommended_lane=%s rollback_required=%s\n", input_csv, recommended_lane, rollback_required);
}
' "${INPUT_CSV}"
