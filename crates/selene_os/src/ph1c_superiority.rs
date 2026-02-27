#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};

use selene_kernel_contracts::ph1pae::PaeMode;
use selene_kernel_contracts::{ContractViolation, Validate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SuperiorityLane {
    SeleneBaseline,
    SeleneChallenger,
    ChatgptAb,
}

impl SuperiorityLane {
    pub const fn as_str(self) -> &'static str {
        match self {
            SuperiorityLane::SeleneBaseline => "SELENE_BASELINE",
            SuperiorityLane::SeleneChallenger => "SELENE_CHALLENGER",
            SuperiorityLane::ChatgptAb => "CHATGPT_AB",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SuperioritySliceKey {
    pub locale: String,
    pub device_route: String,
    pub tenant_id: String,
}

impl SuperioritySliceKey {
    pub fn v1(
        locale: String,
        device_route: String,
        tenant_id: String,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            locale,
            device_route,
            tenant_id,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SuperioritySliceKey {
    fn validate(&self) -> Result<(), ContractViolation> {
        validate_token("superiority_slice_key.locale", &self.locale, 24)?;
        validate_token("superiority_slice_key.device_route", &self.device_route, 32)?;
        validate_token("superiority_slice_key.tenant_id", &self.tenant_id, 64)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuperiorityMetricRow {
    pub captured_at_utc: String,
    pub commit_hash: String,
    pub slice_key: SuperioritySliceKey,
    pub lane: SuperiorityLane,
    pub turns: u32,
    pub transcript_accuracy_bp: u16,
    pub semantic_accuracy_bp: u16,
    pub entity_accuracy_bp: u16,
    pub intent_success_bp: u16,
    pub partial_first_chunk_p95_ms: u32,
    pub eos_to_first_token_p95_ms: u32,
    pub clarify_one_shot_bp: u16,
    pub audit_completeness_bp: u16,
    pub tenant_isolation_bp: u16,
    pub overlap_attribution_bp: u16,
    pub diarization_f1_bp: u16,
    pub cost_microunits_per_turn: u32,
    pub lexicon_v2_hit_bp: u16,
    pub lexicon_freshness_bp: u16,
    pub provider_disagreement_bp: u16,
    pub disagreement_resolved_bp: u16,
    pub gold_label_rate_bp: u16,
    pub silver_label_rate_bp: u16,
    pub distillation_coverage_bp: u16,
    pub disagreement_queue_coverage_bp: u16,
    pub active_learning_topk_recall_bp: u16,
    pub hard_negative_replay_coverage_bp: u16,
    pub cadence_on_time_bp: u16,
    pub rollback_readiness_bp: u16,
    pub per_slice_adapter_ready: bool,
    pub slice_promotion_proof: bool,
    pub current_mode: PaeMode,
}

impl SuperiorityMetricRow {
    #[allow(clippy::too_many_arguments)]
    pub fn v1(
        captured_at_utc: String,
        commit_hash: String,
        slice_key: SuperioritySliceKey,
        lane: SuperiorityLane,
        turns: u32,
        transcript_accuracy_bp: u16,
        semantic_accuracy_bp: u16,
        entity_accuracy_bp: u16,
        intent_success_bp: u16,
        partial_first_chunk_p95_ms: u32,
        eos_to_first_token_p95_ms: u32,
        clarify_one_shot_bp: u16,
        audit_completeness_bp: u16,
        tenant_isolation_bp: u16,
        overlap_attribution_bp: u16,
        diarization_f1_bp: u16,
        cost_microunits_per_turn: u32,
        lexicon_v2_hit_bp: u16,
        lexicon_freshness_bp: u16,
        provider_disagreement_bp: u16,
        disagreement_resolved_bp: u16,
        gold_label_rate_bp: u16,
        silver_label_rate_bp: u16,
        distillation_coverage_bp: u16,
        disagreement_queue_coverage_bp: u16,
        active_learning_topk_recall_bp: u16,
        hard_negative_replay_coverage_bp: u16,
        cadence_on_time_bp: u16,
        rollback_readiness_bp: u16,
        per_slice_adapter_ready: bool,
        slice_promotion_proof: bool,
        current_mode: PaeMode,
    ) -> Result<Self, ContractViolation> {
        let out = Self {
            captured_at_utc,
            commit_hash,
            slice_key,
            lane,
            turns,
            transcript_accuracy_bp,
            semantic_accuracy_bp,
            entity_accuracy_bp,
            intent_success_bp,
            partial_first_chunk_p95_ms,
            eos_to_first_token_p95_ms,
            clarify_one_shot_bp,
            audit_completeness_bp,
            tenant_isolation_bp,
            overlap_attribution_bp,
            diarization_f1_bp,
            cost_microunits_per_turn,
            lexicon_v2_hit_bp,
            lexicon_freshness_bp,
            provider_disagreement_bp,
            disagreement_resolved_bp,
            gold_label_rate_bp,
            silver_label_rate_bp,
            distillation_coverage_bp,
            disagreement_queue_coverage_bp,
            active_learning_topk_recall_bp,
            hard_negative_replay_coverage_bp,
            cadence_on_time_bp,
            rollback_readiness_bp,
            per_slice_adapter_ready,
            slice_promotion_proof,
            current_mode,
        };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SuperiorityMetricRow {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.captured_at_utc.trim().is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.captured_at_utc",
                reason: "must not be empty",
            });
        }
        if self.commit_hash.trim().len() < 7 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.commit_hash",
                reason: "must be a non-empty commit hash",
            });
        }
        self.slice_key.validate()?;
        if self.turns == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.turns",
                reason: "must be > 0",
            });
        }
        if self.partial_first_chunk_p95_ms == 0 || self.eos_to_first_token_p95_ms == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.latency",
                reason: "latency metrics must be > 0",
            });
        }
        if self.cost_microunits_per_turn == 0 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.cost_microunits_per_turn",
                reason: "must be > 0",
            });
        }
        let bounded_bp = [
            self.transcript_accuracy_bp,
            self.semantic_accuracy_bp,
            self.entity_accuracy_bp,
            self.intent_success_bp,
            self.clarify_one_shot_bp,
            self.audit_completeness_bp,
            self.tenant_isolation_bp,
            self.overlap_attribution_bp,
            self.diarization_f1_bp,
            self.lexicon_v2_hit_bp,
            self.lexicon_freshness_bp,
            self.provider_disagreement_bp,
            self.disagreement_resolved_bp,
            self.gold_label_rate_bp,
            self.silver_label_rate_bp,
            self.distillation_coverage_bp,
            self.disagreement_queue_coverage_bp,
            self.active_learning_topk_recall_bp,
            self.hard_negative_replay_coverage_bp,
            self.cadence_on_time_bp,
            self.rollback_readiness_bp,
        ];
        if bounded_bp.iter().any(|v| *v > 10_000) {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.*_bp",
                reason: "all basis-point metrics must be within 0..=10000",
            });
        }
        if self.gold_label_rate_bp + self.silver_label_rate_bp > 10_000 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.gold_silver_rate",
                reason: "gold+silver label rate must be <= 10000bp",
            });
        }
        if self.slice_promotion_proof && matches!(self.current_mode, PaeMode::Shadow) {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_metric_row.slice_promotion_proof",
                reason: "promotion proof requires mode >= ASSIST",
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuperiorityEvalPack {
    pub rows: Vec<SuperiorityMetricRow>,
}

impl SuperiorityEvalPack {
    pub fn v1(rows: Vec<SuperiorityMetricRow>) -> Result<Self, ContractViolation> {
        let out = Self { rows };
        out.validate()?;
        Ok(out)
    }
}

impl Validate for SuperiorityEvalPack {
    fn validate(&self) -> Result<(), ContractViolation> {
        if self.rows.is_empty() {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_eval_pack.rows",
                reason: "must contain at least one row",
            });
        }
        if self.rows.len() > 2048 {
            return Err(ContractViolation::InvalidValue {
                field: "superiority_eval_pack.rows",
                reason: "must be <= 2048",
            });
        }
        let mut seen = BTreeSet::new();
        for row in &self.rows {
            row.validate()?;
            let key = (
                row.slice_key.locale.clone(),
                row.slice_key.device_route.clone(),
                row.slice_key.tenant_id.clone(),
                row.lane,
            );
            if !seen.insert(key) {
                return Err(ContractViolation::InvalidValue {
                    field: "superiority_eval_pack.rows",
                    reason: "slice+lane rows must be unique",
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SuperiorityStepStatus {
    pub step1_chatgpt_ab_parity: bool,
    pub step2_semantic_gate: bool,
    pub step3_global_lexicon_v2: bool,
    pub step4_speaker_overlap: bool,
    pub step5_two_pass_decode: bool,
    pub step6_provider_disagreement_policy: bool,
    pub step7_intent_repair_hardening: bool,
    pub step8_gold_loop_acceleration: bool,
    pub step9_slice_promotion_control: bool,
    pub step10_strict_superiority: bool,
    pub step11_cost_quality_routing: bool,
    pub step12_final_acceptance_pack: bool,
    pub step13_teacher_student_distillation: bool,
    pub step14_disagreement_mining_queue: bool,
    pub step15_active_learning_priority: bool,
    pub step16_gold_silver_tiering: bool,
    pub step17_hard_negative_replay: bool,
    pub step18_entity_intent_gates: bool,
    pub step19_diarization_overlap_attribution: bool,
    pub step20_per_slice_adapter_training: bool,
    pub step21_champion_challenger: bool,
    pub step22_learning_cadence_rollback: bool,
}

impl SuperiorityStepStatus {
    fn all_pass() -> Self {
        Self {
            step1_chatgpt_ab_parity: true,
            step2_semantic_gate: true,
            step3_global_lexicon_v2: true,
            step4_speaker_overlap: true,
            step5_two_pass_decode: true,
            step6_provider_disagreement_policy: true,
            step7_intent_repair_hardening: true,
            step8_gold_loop_acceleration: true,
            step9_slice_promotion_control: true,
            step10_strict_superiority: true,
            step11_cost_quality_routing: true,
            step12_final_acceptance_pack: true,
            step13_teacher_student_distillation: true,
            step14_disagreement_mining_queue: true,
            step15_active_learning_priority: true,
            step16_gold_silver_tiering: true,
            step17_hard_negative_replay: true,
            step18_entity_intent_gates: true,
            step19_diarization_overlap_attribution: true,
            step20_per_slice_adapter_training: true,
            step21_champion_challenger: true,
            step22_learning_cadence_rollback: true,
        }
    }

    fn all_ok(self) -> bool {
        self.step1_chatgpt_ab_parity
            && self.step2_semantic_gate
            && self.step3_global_lexicon_v2
            && self.step4_speaker_overlap
            && self.step5_two_pass_decode
            && self.step6_provider_disagreement_policy
            && self.step7_intent_repair_hardening
            && self.step8_gold_loop_acceleration
            && self.step9_slice_promotion_control
            && self.step10_strict_superiority
            && self.step11_cost_quality_routing
            && self.step12_final_acceptance_pack
            && self.step13_teacher_student_distillation
            && self.step14_disagreement_mining_queue
            && self.step15_active_learning_priority
            && self.step16_gold_silver_tiering
            && self.step17_hard_negative_replay
            && self.step18_entity_intent_gates
            && self.step19_diarization_overlap_attribution
            && self.step20_per_slice_adapter_training
            && self.step21_champion_challenger
            && self.step22_learning_cadence_rollback
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuperiorityGateReport {
    pub step_status: SuperiorityStepStatus,
    pub overall_pass: bool,
    pub recommended_runtime_lane: SuperiorityLane,
    pub rollback_required: bool,
    pub fail_reasons: Vec<String>,
}

pub fn evaluate_ph1c_superiority_pack(
    pack: &SuperiorityEvalPack,
) -> Result<SuperiorityGateReport, ContractViolation> {
    pack.validate()?;
    let mut status = SuperiorityStepStatus::all_pass();
    let mut fail_reasons = Vec::new();

    let mut by_slice: BTreeMap<
        SuperioritySliceKey,
        BTreeMap<SuperiorityLane, &SuperiorityMetricRow>,
    > = BTreeMap::new();
    for row in &pack.rows {
        by_slice
            .entry(row.slice_key.clone())
            .or_default()
            .insert(row.lane, row);
    }

    let mut challenger_superior_all_slices = true;
    let mut challenger_slice_count = 0usize;
    for (slice, lanes) in &by_slice {
        let baseline = lanes.get(&SuperiorityLane::SeleneBaseline);
        let challenger = lanes.get(&SuperiorityLane::SeleneChallenger);
        let chatgpt = lanes.get(&SuperiorityLane::ChatgptAb);
        if baseline.is_none() || challenger.is_none() || chatgpt.is_none() {
            status.step1_chatgpt_ab_parity = false;
            challenger_superior_all_slices = false;
            fail_reasons.push(format!(
                "STEP1_FAIL:slice_missing_lane locale={} device_route={} tenant_id={}",
                slice.locale, slice.device_route, slice.tenant_id
            ));
            continue;
        }

        let baseline = *baseline.expect("checked above");
        let challenger = *challenger.expect("checked above");
        let chatgpt = *chatgpt.expect("checked above");
        challenger_slice_count += 1;

        // Step 2: semantic accuracy gate.
        if challenger.semantic_accuracy_bp < 9_600 {
            status.step2_semantic_gate = false;
            fail_reasons.push(format!(
                "STEP2_FAIL:semantic_accuracy_lt_9600 slice={}/{}/{} value={}",
                slice.locale, slice.device_route, slice.tenant_id, challenger.semantic_accuracy_bp
            ));
        }

        // Step 3: lexicon v2 must be both active and fresh.
        if challenger.lexicon_v2_hit_bp < 5_500 || challenger.lexicon_freshness_bp < 9_800 {
            status.step3_global_lexicon_v2 = false;
            fail_reasons.push(format!(
                "STEP3_FAIL:lexicon_v2_hit_or_freshness slice={}/{}/{} hit_bp={} fresh_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.lexicon_v2_hit_bp,
                challenger.lexicon_freshness_bp
            ));
        }

        // Step 4 + Step 19: overlap attribution quality.
        if challenger.overlap_attribution_bp < 9_500 || challenger.diarization_f1_bp < 9_400 {
            status.step4_speaker_overlap = false;
            status.step19_diarization_overlap_attribution = false;
            fail_reasons.push(format!(
                "STEP4_19_FAIL:overlap_or_diarization slice={}/{}/{} overlap_bp={} diarization_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.overlap_attribution_bp,
                challenger.diarization_f1_bp
            ));
        }

        // Step 5: two-pass fast + bounded finalization latency.
        if challenger.partial_first_chunk_p95_ms > 250 || challenger.eos_to_first_token_p95_ms > 300
        {
            status.step5_two_pass_decode = false;
            fail_reasons.push(format!(
                "STEP5_FAIL:latency_budget_exceeded slice={}/{}/{} partial_p95_ms={} eos_p95_ms={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.partial_first_chunk_p95_ms,
                challenger.eos_to_first_token_p95_ms
            ));
        }

        // Step 6: disagreement policy.
        if challenger.provider_disagreement_bp > 1_500
            && challenger.disagreement_resolved_bp < 9_000
        {
            status.step6_provider_disagreement_policy = false;
            fail_reasons.push(format!(
                "STEP6_FAIL:disagreement_unresolved slice={}/{}/{} disagreement_bp={} resolved_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.provider_disagreement_bp,
                challenger.disagreement_resolved_bp
            ));
        }

        // Step 7: intent-aware repair hardening.
        if challenger.intent_success_bp < 9_650 || challenger.semantic_accuracy_bp < 9_650 {
            status.step7_intent_repair_hardening = false;
            fail_reasons.push(format!(
                "STEP7_FAIL:intent_or_semantic_quality_low slice={}/{}/{} intent_bp={} semantic_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.intent_success_bp,
                challenger.semantic_accuracy_bp
            ));
        }

        // Step 8: all corrections/escalations should land in labeled loop.
        if challenger.gold_label_rate_bp < 9_800 {
            status.step8_gold_loop_acceleration = false;
            fail_reasons.push(format!(
                "STEP8_FAIL:gold_label_rate_lt_9800 slice={}/{}/{} value={}",
                slice.locale, slice.device_route, slice.tenant_id, challenger.gold_label_rate_bp
            ));
        }

        // Step 9 + Step 20: slice-based governed promotion only with proof.
        if !challenger.slice_promotion_proof
            || !challenger.per_slice_adapter_ready
            || matches!(challenger.current_mode, PaeMode::Shadow)
        {
            status.step9_slice_promotion_control = false;
            status.step20_per_slice_adapter_training = false;
            fail_reasons.push(format!(
                "STEP9_20_FAIL:slice_promotion_proof_or_adapter_missing slice={}/{}/{} proof={} adapter_ready={} mode={:?}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.slice_promotion_proof,
                challenger.per_slice_adapter_ready,
                challenger.current_mode
            ));
        }

        // Step 10: strict superiority over baseline and ChatGPT on core quality.
        let strict_superior = challenger.transcript_accuracy_bp >= baseline.transcript_accuracy_bp
            && challenger.transcript_accuracy_bp >= chatgpt.transcript_accuracy_bp
            && challenger.semantic_accuracy_bp >= baseline.semantic_accuracy_bp
            && challenger.semantic_accuracy_bp >= chatgpt.semantic_accuracy_bp
            && challenger.entity_accuracy_bp >= baseline.entity_accuracy_bp
            && challenger.entity_accuracy_bp >= chatgpt.entity_accuracy_bp
            && challenger.intent_success_bp >= baseline.intent_success_bp
            && challenger.intent_success_bp >= chatgpt.intent_success_bp;
        if !strict_superior {
            status.step10_strict_superiority = false;
            challenger_superior_all_slices = false;
            fail_reasons.push(format!(
                "STEP10_FAIL:challenger_not_superior slice={}/{}/{} challenger=[t{} s{} e{} i{}] baseline=[t{} s{} e{} i{}] chatgpt=[t{} s{} e{} i{}]",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.transcript_accuracy_bp,
                challenger.semantic_accuracy_bp,
                challenger.entity_accuracy_bp,
                challenger.intent_success_bp,
                baseline.transcript_accuracy_bp,
                baseline.semantic_accuracy_bp,
                baseline.entity_accuracy_bp,
                baseline.intent_success_bp,
                chatgpt.transcript_accuracy_bp,
                chatgpt.semantic_accuracy_bp,
                chatgpt.entity_accuracy_bp,
                chatgpt.intent_success_bp
            ));
        }

        // Step 11: lowest-cost route that keeps quality gates.
        let quality_gate_ok = challenger.transcript_accuracy_bp >= 9_700
            && challenger.semantic_accuracy_bp >= 9_600
            && challenger.intent_success_bp >= 9_700;
        let cheaper_than_both = challenger.cost_microunits_per_turn
            <= baseline.cost_microunits_per_turn
            && challenger.cost_microunits_per_turn <= chatgpt.cost_microunits_per_turn;
        if !(quality_gate_ok && cheaper_than_both) {
            status.step11_cost_quality_routing = false;
            fail_reasons.push(format!(
                "STEP11_FAIL:cost_quality_policy slice={}/{}/{} challenger_cost={} baseline_cost={} chatgpt_cost={} quality_gate_ok={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.cost_microunits_per_turn,
                baseline.cost_microunits_per_turn,
                chatgpt.cost_microunits_per_turn,
                quality_gate_ok
            ));
        }

        // Step 12: final acceptance pack hard requirements.
        if challenger.clarify_one_shot_bp < 9_000
            || challenger.audit_completeness_bp != 10_000
            || challenger.tenant_isolation_bp != 10_000
        {
            status.step12_final_acceptance_pack = false;
            fail_reasons.push(format!(
                "STEP12_FAIL:acceptance_pack slice={}/{}/{} clarify_bp={} audit_bp={} isolation_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.clarify_one_shot_bp,
                challenger.audit_completeness_bp,
                challenger.tenant_isolation_bp
            ));
        }

        if challenger.distillation_coverage_bp < 9_000 {
            status.step13_teacher_student_distillation = false;
            fail_reasons.push(format!(
                "STEP13_FAIL:distillation_coverage_lt_9000 slice={}/{}/{} value={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.distillation_coverage_bp
            ));
        }

        if challenger.disagreement_queue_coverage_bp < 9_000 {
            status.step14_disagreement_mining_queue = false;
            fail_reasons.push(format!(
                "STEP14_FAIL:disagreement_queue_coverage_lt_9000 slice={}/{}/{} value={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.disagreement_queue_coverage_bp
            ));
        }

        if challenger.active_learning_topk_recall_bp < 9_200 {
            status.step15_active_learning_priority = false;
            fail_reasons.push(format!(
                "STEP15_FAIL:active_learning_recall_lt_9200 slice={}/{}/{} value={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.active_learning_topk_recall_bp
            ));
        }

        if challenger.gold_label_rate_bp < 7_000 || challenger.silver_label_rate_bp > 3_000 {
            status.step16_gold_silver_tiering = false;
            fail_reasons.push(format!(
                "STEP16_FAIL:gold_silver_tiering slice={}/{}/{} gold_bp={} silver_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.gold_label_rate_bp,
                challenger.silver_label_rate_bp
            ));
        }

        if challenger.hard_negative_replay_coverage_bp < 9_000 {
            status.step17_hard_negative_replay = false;
            fail_reasons.push(format!(
                "STEP17_FAIL:hard_negative_replay_coverage_lt_9000 slice={}/{}/{} value={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.hard_negative_replay_coverage_bp
            ));
        }

        if challenger.entity_accuracy_bp < 9_700 || challenger.intent_success_bp < 9_700 {
            status.step18_entity_intent_gates = false;
            fail_reasons.push(format!(
                "STEP18_FAIL:entity_or_intent_gate slice={}/{}/{} entity_bp={} intent_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.entity_accuracy_bp,
                challenger.intent_success_bp
            ));
        }

        if challenger.cadence_on_time_bp < 9_800 || challenger.rollback_readiness_bp != 10_000 {
            status.step22_learning_cadence_rollback = false;
            fail_reasons.push(format!(
                "STEP22_FAIL:cadence_or_rollback slice={}/{}/{} cadence_bp={} rollback_bp={}",
                slice.locale,
                slice.device_route,
                slice.tenant_id,
                challenger.cadence_on_time_bp,
                challenger.rollback_readiness_bp
            ));
        }
    }

    if challenger_slice_count == 0 {
        status.step1_chatgpt_ab_parity = false;
        status.step21_champion_challenger = false;
        fail_reasons.push("STEP1_FAIL:no_complete_slice_triplets".to_string());
    }

    // Step 21: challenger is promoted only when all hard superiority gates hold.
    if !(status.step10_strict_superiority
        && status.step12_final_acceptance_pack
        && status.step20_per_slice_adapter_training
        && challenger_superior_all_slices)
    {
        status.step21_champion_challenger = false;
    }

    let overall_pass = status.all_ok();
    let recommended_runtime_lane = if status.step21_champion_challenger {
        SuperiorityLane::SeleneChallenger
    } else {
        SuperiorityLane::SeleneBaseline
    };
    let rollback_required =
        !status.step22_learning_cadence_rollback || !status.step12_final_acceptance_pack;
    Ok(SuperiorityGateReport {
        step_status: status,
        overall_pass,
        recommended_runtime_lane,
        rollback_required,
        fail_reasons,
    })
}

fn validate_token(
    field: &'static str,
    value: &str,
    max_len: usize,
) -> Result<(), ContractViolation> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not be empty",
        });
    }
    if trimmed.len() > max_len {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "exceeds max length",
        });
    }
    if trimmed.chars().any(char::is_control) {
        return Err(ContractViolation::InvalidValue {
            field,
            reason: "must not contain control characters",
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slice() -> SuperioritySliceKey {
        SuperioritySliceKey::v1(
            "en-US".to_string(),
            "desktop_mic".to_string(),
            "tenant_alpha".to_string(),
        )
        .expect("slice must build")
    }

    fn row(lane: SuperiorityLane) -> SuperiorityMetricRow {
        let (t, s, e, i, c) = match lane {
            SuperiorityLane::SeleneBaseline => (9_620, 9_540, 9_480, 9_500, 2_500),
            SuperiorityLane::SeleneChallenger => (9_760, 9_710, 9_730, 9_760, 2_100),
            SuperiorityLane::ChatgptAb => (9_700, 9_650, 9_690, 9_710, 2_400),
        };
        SuperiorityMetricRow::v1(
            "2026-02-26T12:00:00Z".to_string(),
            "abcdef123456".to_string(),
            slice(),
            lane,
            400,
            t,
            s,
            e,
            i,
            220,
            280,
            9_200,
            10_000,
            10_000,
            9_700,
            9_600,
            c,
            6_200,
            9_900,
            900,
            9_500,
            9_800,
            200,
            9_500,
            9_300,
            9_500,
            9_200,
            9_900,
            10_000,
            true,
            true,
            PaeMode::Assist,
        )
        .expect("row must build")
    }

    #[test]
    fn superiority_pack_passes_and_promotes_challenger_when_all_steps_pass() {
        let pack = SuperiorityEvalPack::v1(vec![
            row(SuperiorityLane::SeleneBaseline),
            row(SuperiorityLane::SeleneChallenger),
            row(SuperiorityLane::ChatgptAb),
        ])
        .expect("pack must build");
        let report = evaluate_ph1c_superiority_pack(&pack).expect("evaluation must succeed");
        assert!(report.overall_pass);
        assert_eq!(
            report.recommended_runtime_lane,
            SuperiorityLane::SeleneChallenger
        );
        assert!(!report.rollback_required);
        assert!(report.fail_reasons.is_empty());
    }

    #[test]
    fn superiority_pack_fails_step1_when_chatgpt_lane_missing() {
        let pack = SuperiorityEvalPack::v1(vec![
            row(SuperiorityLane::SeleneBaseline),
            row(SuperiorityLane::SeleneChallenger),
        ])
        .expect("pack must build");
        let report = evaluate_ph1c_superiority_pack(&pack).expect("evaluation must succeed");
        assert!(!report.step_status.step1_chatgpt_ab_parity);
        assert!(!report.step_status.step21_champion_challenger);
        assert_eq!(
            report.recommended_runtime_lane,
            SuperiorityLane::SeleneBaseline
        );
    }

    #[test]
    fn superiority_pack_fails_when_challenger_regresses_intent_vs_chatgpt() {
        let mut challenger = row(SuperiorityLane::SeleneChallenger);
        challenger.intent_success_bp = 9_650;
        let pack = SuperiorityEvalPack::v1(vec![
            row(SuperiorityLane::SeleneBaseline),
            challenger,
            row(SuperiorityLane::ChatgptAb),
        ])
        .expect("pack must build");
        let report = evaluate_ph1c_superiority_pack(&pack).expect("evaluation must succeed");
        assert!(!report.step_status.step10_strict_superiority);
        assert!(!report.step_status.step18_entity_intent_gates);
        assert_eq!(
            report.recommended_runtime_lane,
            SuperiorityLane::SeleneBaseline
        );
    }

    #[test]
    fn superiority_pack_requires_rollback_when_cadence_or_rollback_readiness_drifts() {
        let mut challenger = row(SuperiorityLane::SeleneChallenger);
        challenger.cadence_on_time_bp = 9_600;
        challenger.rollback_readiness_bp = 9_400;
        let pack = SuperiorityEvalPack::v1(vec![
            row(SuperiorityLane::SeleneBaseline),
            challenger,
            row(SuperiorityLane::ChatgptAb),
        ])
        .expect("pack must build");
        let report = evaluate_ph1c_superiority_pack(&pack).expect("evaluation must succeed");
        assert!(!report.step_status.step22_learning_cadence_rollback);
        assert!(report.rollback_required);
    }

    #[test]
    fn superiority_pack_fails_slice_promotion_control_without_proof() {
        let mut challenger = row(SuperiorityLane::SeleneChallenger);
        challenger.slice_promotion_proof = false;
        challenger.current_mode = PaeMode::Shadow;
        let pack = SuperiorityEvalPack::v1(vec![
            row(SuperiorityLane::SeleneBaseline),
            challenger,
            row(SuperiorityLane::ChatgptAb),
        ])
        .expect("pack must build");
        let report = evaluate_ph1c_superiority_pack(&pack).expect("evaluation must succeed");
        assert!(!report.step_status.step9_slice_promotion_control);
        assert!(!report.step_status.step20_per_slice_adapter_training);
        assert!(!report.step_status.step21_champion_challenger);
    }
}
