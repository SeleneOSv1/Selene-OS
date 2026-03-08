#![forbid(unsafe_code)]

use std::cmp::Ordering;

use selene_kernel_contracts::ph1pae::{PaeRouteDomain, PaeSignalSource};
use selene_kernel_contracts::ph1rll::{RllArtifactCandidate, RllOptimizationTarget};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BudgetPosture {
    pub remaining_units: u32,
    pub utilization_bp: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaeQuantitativeScore {
    pub total_score_bp: i32,
    pub quality_score_bp: i16,
    pub latency_penalty_bp: i16,
    pub cost_penalty_bp: i16,
    pub regression_penalty_bp: i16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrustScoreBpBreakdown {
    pub trust_score_bp: u16,
    pub base_score_bp: u16,
    pub spam_penalty_bp: u16,
    pub recency_bonus_bp: u16,
    pub corroboration_bonus_bp: u16,
    pub reputation_adjustment_bp: i16,
    pub recency_available: bool,
}

pub fn divide_round_u64(numerator: u64, denominator: u64) -> u32 {
    if denominator == 0 {
        return 0;
    }
    ((numerator.saturating_mul(2).saturating_add(denominator)) / (denominator.saturating_mul(2)))
        as u32
}

pub fn utilization_bp(used: u32, budget: u32) -> u16 {
    if budget == 0 {
        return 10_000;
    }
    let bp = (used as u128 * 10_000u128) / budget as u128;
    bp.min(10_000) as u16
}

pub fn budget_posture(used: u32, budget: u32) -> BudgetPosture {
    BudgetPosture {
        remaining_units: budget.saturating_sub(used),
        utilization_bp: utilization_bp(used, budget),
    }
}

pub fn clamp_wait_ms(suggested: Option<u32>, default_wait_ms: u32, max_wait_ms: u32) -> u32 {
    suggested.unwrap_or(default_wait_ms).min(max_wait_ms)
}

pub fn pae_source_weight_pct(source: PaeSignalSource) -> i32 {
    match source {
        PaeSignalSource::Listen => 25,
        PaeSignalSource::Feedback => 35,
        PaeSignalSource::Learn => 20,
        PaeSignalSource::RllGoverned => 20,
    }
}

pub fn pae_route_index(route: PaeRouteDomain) -> usize {
    match route {
        PaeRouteDomain::Stt => 0,
        PaeRouteDomain::Tts => 1,
        PaeRouteDomain::Llm => 2,
        PaeRouteDomain::Tooling => 3,
    }
}

pub fn compute_pae_signal_bias(
    signal_value_bp: i16,
    confidence_bp: u16,
    source: PaeSignalSource,
) -> i32 {
    (signal_value_bp as i32) * pae_source_weight_pct(source) * (confidence_bp as i32) / 10_000 / 100
}

pub fn compute_pae_candidate_score(
    expected_quality_bp: i16,
    signal_bias_bp: i32,
    expected_latency_ms: u16,
    expected_cost_bp: i16,
    regression_risk_bp: u16,
    sample_size: u16,
    effective_min_sample: u16,
) -> PaeQuantitativeScore {
    let quality = (expected_quality_bp as i32 + signal_bias_bp).clamp(-20_000, 20_000);
    let latency_penalty = ((expected_latency_ms as i32) / 4).clamp(0, 4_000);
    let cost_penalty = (expected_cost_bp as i32).max(0).clamp(0, 4_000);
    let regression_penalty = ((regression_risk_bp as i32) / 2).clamp(0, 5_000);
    let mut total = quality - latency_penalty - cost_penalty - regression_penalty;
    if sample_size < effective_min_sample {
        total -= 1_200;
    }
    PaeQuantitativeScore {
        total_score_bp: total,
        quality_score_bp: quality.clamp(i16::MIN as i32, i16::MAX as i32) as i16,
        latency_penalty_bp: latency_penalty as i16,
        cost_penalty_bp: cost_penalty as i16,
        regression_penalty_bp: regression_penalty as i16,
    }
}

pub fn compare_pae_rank(
    left_total_score_bp: i32,
    left_sample_size: u16,
    left_candidate_id: &str,
    right_total_score_bp: i32,
    right_sample_size: u16,
    right_candidate_id: &str,
) -> Ordering {
    right_total_score_bp
        .cmp(&left_total_score_bp)
        .then(right_sample_size.cmp(&left_sample_size))
        .then(left_candidate_id.cmp(right_candidate_id))
}

pub fn pae_priority_from_total(total_score_bp: i32) -> u16 {
    (total_score_bp + 10_000).clamp(0, 10_000) as u16
}

pub fn rll_candidate_score(candidate: &RllArtifactCandidate) -> u8 {
    let mut score = candidate.confidence_pct as i16;
    score += rll_target_bonus(candidate.target);
    score += rll_effect_bonus(candidate.expected_effect_bp);
    score.clamp(0, 100) as u8
}

pub fn compare_rll_rank(
    left_score: u8,
    left_artifact_id: &str,
    right_score: u8,
    right_artifact_id: &str,
) -> Ordering {
    right_score
        .cmp(&left_score)
        .then(left_artifact_id.cmp(right_artifact_id))
}

pub fn compute_finder_confidence_score_bp(
    intent_confidence_bp: u16,
    required_field_coverage_bp: u16,
    evidence_coverage_bp: u16,
    catalog_status_bp: u16,
    context_alignment_bp: u16,
    ocr_alignment_bp: u16,
    llm_assist_alignment_bp: u16,
    gold_match_bonus_bp: u16,
    penalty_bp_total: u16,
) -> u16 {
    let raw_score_bp = ((35u32 * intent_confidence_bp as u32)
        + (20u32 * required_field_coverage_bp as u32)
        + (10u32 * evidence_coverage_bp as u32)
        + (10u32 * catalog_status_bp as u32)
        + (10u32 * context_alignment_bp as u32)
        + (5u32 * ocr_alignment_bp as u32)
        + (5u32 * llm_assist_alignment_bp as u32)
        + (5u32 * gold_match_bonus_bp as u32))
        / 100;
    raw_score_bp.saturating_sub(penalty_bp_total as u32).min(10_000) as u16
}

pub fn compare_finder_rank(
    left_confidence_score_bp: u16,
    left_gold_match_bonus_bp: u16,
    left_simulation_priority: u16,
    left_simulation_id: &str,
    right_confidence_score_bp: u16,
    right_gold_match_bonus_bp: u16,
    right_simulation_priority: u16,
    right_simulation_id: &str,
) -> Ordering {
    right_confidence_score_bp
        .cmp(&left_confidence_score_bp)
        .then_with(|| right_gold_match_bonus_bp.cmp(&left_gold_match_bonus_bp))
        .then_with(|| right_simulation_priority.cmp(&left_simulation_priority))
        .then_with(|| left_simulation_id.cmp(right_simulation_id))
}

pub fn compute_finder_worthiness_score_bp(
    frequency_bp: u16,
    value_bp: u16,
    estimated_roi_score_bp: u16,
    feasibility_bp: u16,
    scope_bp: u16,
    risk_bp: u16,
) -> u16 {
    let raw = ((25u32 * frequency_bp as u32)
        + (25u32 * value_bp as u32)
        + (15u32 * estimated_roi_score_bp as u32)
        + (20u32 * feasibility_bp as u32)
        + (15u32 * scope_bp as u32))
        / 100;
    let risk_penalty_bp = risk_bp as u32 / 2;
    raw.saturating_sub(risk_penalty_bp).min(10_000) as u16
}

pub fn compute_trust_score_bp(
    base_score_bp: u16,
    spam_risk_bp: u16,
    published_at_ms: Option<i64>,
    now_ms: i64,
    corroboration_count: usize,
    reputation_adjustment_bp: i16,
) -> TrustScoreBpBreakdown {
    let spam_penalty_bp = divide_round_u64(spam_risk_bp as u64 * 55, 100) as u16;
    let recency_bonus_bp = recency_bonus_bp(published_at_ms, now_ms);
    let corroboration_bonus_bp = corroboration_bonus_bp(corroboration_count);

    let mut score_bp = base_score_bp as i32 - spam_penalty_bp as i32;
    score_bp += recency_bonus_bp as i32;
    score_bp += corroboration_bonus_bp as i32;
    score_bp += reputation_adjustment_bp as i32;
    score_bp = score_bp.clamp(0, 10_000);

    TrustScoreBpBreakdown {
        trust_score_bp: score_bp as u16,
        base_score_bp,
        spam_penalty_bp,
        recency_bonus_bp,
        corroboration_bonus_bp,
        reputation_adjustment_bp,
        recency_available: published_at_ms.is_some(),
    }
}

fn rll_target_bonus(target: RllOptimizationTarget) -> i16 {
    match target {
        RllOptimizationTarget::PaeProviderSelectionWeights => 10,
        RllOptimizationTarget::PruneClarificationOrdering => 9,
        RllOptimizationTarget::CachePrefetchHeuristics => 8,
        RllOptimizationTarget::ContextRetrievalScoring => 8,
    }
}

fn rll_effect_bonus(expected_effect_bp: i16) -> i16 {
    (expected_effect_bp / 40).clamp(-20, 20)
}

fn recency_bonus_bp(published_at_ms: Option<i64>, now_ms: i64) -> u16 {
    let Some(published_at_ms) = published_at_ms else {
        return 0;
    };
    if now_ms <= 0 || published_at_ms <= 0 {
        return 0;
    }
    let age_ms = if now_ms >= published_at_ms {
        now_ms - published_at_ms
    } else {
        0
    };
    let day_ms = 24_i64 * 60_i64 * 60_i64 * 1_000_i64;
    if age_ms <= day_ms {
        800
    } else if age_ms <= 7 * day_ms {
        500
    } else if age_ms <= 30 * day_ms {
        200
    } else {
        0
    }
}

fn corroboration_bonus_bp(corroboration_count: usize) -> u16 {
    match corroboration_count {
        0 | 1 => 0,
        2 => 200,
        3 => 300,
        _ => 400,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use selene_kernel_contracts::ph1rll::RllOptimizationTarget;

    #[test]
    fn at_comp_core_01_budget_posture_is_deterministic() {
        let first = budget_posture(95, 100);
        let second = budget_posture(95, 100);
        assert_eq!(first, second);
        assert_eq!(first.utilization_bp, 9500);
        assert_eq!(first.remaining_units, 5);
    }

    #[test]
    fn at_comp_core_02_finder_rank_order_is_deterministic() {
        let ordering = compare_finder_rank(9000, 0, 10, "a", 9000, 0, 9, "b");
        assert_eq!(ordering, Ordering::Less);
    }

    #[test]
    fn at_comp_core_03_trust_score_bp_is_deterministic() {
        let first = compute_trust_score_bp(6000, 1800, Some(1_707_613_600_000), 1_707_700_000_000, 2, 400);
        let second =
            compute_trust_score_bp(6000, 1800, Some(1_707_613_600_000), 1_707_700_000_000, 2, 400);
        assert_eq!(first, second);
        assert!(first.trust_score_bp <= 10_000);
    }

    #[test]
    fn at_comp_core_04_rll_score_is_deterministic() {
        let candidate = RllArtifactCandidate::v1(
            "artifact".to_string(),
            RllOptimizationTarget::PaeProviderSelectionWeights,
            220,
            78,
            3,
            "evidence".to_string(),
        )
        .unwrap();
        assert_eq!(rll_candidate_score(&candidate), rll_candidate_score(&candidate));
    }
}
