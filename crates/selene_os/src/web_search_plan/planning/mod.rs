#![forbid(unsafe_code)]

pub mod budget_control;
pub mod dead_link_handler;
pub mod open_selector;
pub mod scoring;
pub mod snippet_fallback;
pub mod tie_break;

use crate::web_search_plan::chunk::{
    bounded_excerpt, build_hashed_chunks_for_document, ChunkBuildError,
};
use crate::web_search_plan::diag::{
    default_failed_transitions, try_build_debug_packet, DebugPacketContext, DebugStatus,
};
use crate::web_search_plan::gap_closers::freshness_watchdog::{
    evaluate_freshness_watchdog, report_to_json as freshness_watchdog_to_json,
};
use crate::web_search_plan::gap_closers::unknown_first::{
    decision_to_json as unknown_first_to_json, evaluate_unknown_first_pre_synthesis,
};
use crate::web_search_plan::parity::ambiguity::{
    select_single_best_clarification, AMBIGUITY_POLICY_VERSION,
};
use crate::web_search_plan::parity::diversification::{
    diversify_for_high_tier, DIVERSIFICATION_POLICY_VERSION, HIGH_TIER_MIN_DISTINCT_DOMAINS,
};
use crate::web_search_plan::parity::multi_query::{
    decompose_query, MAX_SUB_QUERIES, MULTI_QUERY_VERSION,
};
use crate::web_search_plan::parity::reformulation::{
    apply_reformulation_ladder, REFORMULATION_POLICY_VERSION,
};
use crate::web_search_plan::parity::reranker::{
    rerank_candidates as parity_rerank_candidates, RerankInput, RerankWeights,
    RERANK_WEIGHTS_VERSION,
};
use crate::web_search_plan::parity::stitching::{
    build_stitching_summary, contradiction_summary_json, stitch_sources, STITCHING_POLICY_VERSION,
};
use crate::web_search_plan::perf_cost::budgets::{budget_plan_for_tier, ProviderCallBudget};
use crate::web_search_plan::perf_cost::degrade::{DegradeController, DegradeStep};
use crate::web_search_plan::perf_cost::tiers::{caps_for_tier, ImportanceTier, TierCaps};
use crate::web_search_plan::perf_cost::timeouts::timeout_envelope_for_tier;
use crate::web_search_plan::planning::budget_control::{BudgetControl, OpenBudgetPolicy};
use crate::web_search_plan::planning::dead_link_handler::should_select_replacement;
use crate::web_search_plan::planning::open_selector::{
    open_candidate_with_url_fetch, UrlOpenContext,
};
use crate::web_search_plan::planning::scoring::{score_with_policy, ScoreSignals, ScoringPolicy};
use crate::web_search_plan::planning::snippet_fallback::build_snippet_fallback;
use crate::web_search_plan::planning::tie_break::{sort_ranked_candidates, RankedCandidate};
use crate::web_search_plan::url::fetch::stage3_chunk_policy;
use crate::web_search_plan::url::STAGE3_MAX_EVIDENCE_EXCERPT_CHARS;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const PLANNING_ENGINE_ID: &str = "PH1.SEARCH";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCandidate {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub canonical_url: String,
    pub provider_id: String,
    pub provider_rank: usize,
    pub relevance: i32,
    pub trust_tier: i32,
    pub freshness_score: i32,
    pub corroboration_count: i32,
    pub spam_risk: i32,
}

impl SearchCandidate {
    pub fn domain(&self) -> String {
        url::Url::parse(&self.url)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|host| host.to_ascii_lowercase()))
            .unwrap_or_else(|| "unknown-domain".to_string())
    }

    pub fn score_signals(&self) -> ScoreSignals {
        ScoreSignals {
            relevance: self.relevance,
            trust_tier: self.trust_tier,
            freshness_score: self.freshness_score,
            corroboration_count: self.corroboration_count,
            spam_risk: self.spam_risk,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningInput {
    pub trace_id: String,
    pub query: String,
    pub created_at_ms: i64,
    pub retrieved_at_ms: i64,
    pub produced_by: String,
    pub intended_consumers: Vec<String>,
    pub importance_tier: String,
    pub rewrite_attempts: Vec<String>,
    pub sub_queries: Vec<String>,
    pub candidates: Vec<SearchCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanningPolicy {
    pub policy_snapshot_id: String,
    pub scoring_policy: ScoringPolicy,
    pub open_budget: OpenBudgetPolicy,
    pub max_rewrite_attempts: usize,
    pub snippet_fallback_min_sources: usize,
}

impl Default for PlanningPolicy {
    fn default() -> Self {
        Self {
            policy_snapshot_id: "policy-snapshot-default".to_string(),
            scoring_policy: ScoringPolicy::default(),
            open_budget: OpenBudgetPolicy::default(),
            max_rewrite_attempts: 2,
            snippet_fallback_min_sources: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    KReached,
    RewriteAttemptsExhausted,
    ProviderResultsExhausted,
    BudgetExhausted,
}

impl StopReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KReached => "k_reached",
            Self::RewriteAttemptsExhausted => "rewrite_attempts_exhausted",
            Self::ProviderResultsExhausted => "provider_results_exhausted",
            Self::BudgetExhausted => "budget_exhausted",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenSuccess {
    pub final_url: String,
    pub title: String,
    pub extracted_text: String,
    pub extracted_chars: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenFailure {
    pub canonical_url: String,
    pub reason_code: String,
    pub error_kind: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct PlanningResult {
    pub evidence_packet: Value,
    pub stop_reason: StopReason,
    pub degraded_evidence_mode: bool,
    pub top_k_selected: Vec<String>,
    pub open_failures: Vec<OpenFailure>,
}

pub fn planning_input_from_tool_request(
    tool_request_packet: &Value,
    retrieved_at_ms: i64,
    candidates: Vec<SearchCandidate>,
) -> Result<PlanningInput, String> {
    let obj = tool_request_packet
        .as_object()
        .ok_or_else(|| "tool request packet must be object".to_string())?;

    let trace_id = obj
        .get("trace_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "tool request trace_id is missing".to_string())?
        .to_string();

    let query = obj
        .get("query")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "tool request query is missing".to_string())?
        .to_string();

    let created_at_ms = obj
        .get("created_at_ms")
        .and_then(Value::as_i64)
        .ok_or_else(|| "tool request created_at_ms is missing".to_string())?;

    let importance_tier = normalize_importance_tier(
        obj.get("importance_tier")
            .and_then(Value::as_str)
            .unwrap_or("medium"),
    )?;

    let intended_consumers = obj
        .get("intended_consumers")
        .and_then(Value::as_array)
        .map(|entries| {
            entries
                .iter()
                .filter_map(Value::as_str)
                .map(|entry| entry.trim().to_string())
                .filter(|entry| !entry.is_empty())
                .collect::<Vec<String>>()
        })
        .filter(|entries| !entries.is_empty())
        .unwrap_or_else(|| {
            vec![
                "PH1.D".to_string(),
                "PH1.WRITE".to_string(),
                "PH1.J".to_string(),
            ]
        });

    Ok(PlanningInput {
        trace_id,
        query: query.clone(),
        created_at_ms,
        retrieved_at_ms,
        produced_by: PLANNING_ENGINE_ID.to_string(),
        intended_consumers,
        importance_tier,
        rewrite_attempts: Vec::new(),
        sub_queries: vec![query],
        candidates,
    })
}

pub fn execute_search_topk_pipeline_with_url_fetch(
    input: &PlanningInput,
    policy: &PlanningPolicy,
    open_context: &UrlOpenContext,
) -> Result<PlanningResult, String> {
    let mut url_open_ordinal = 0usize;
    execute_search_topk_pipeline_with_opener(input, policy, |candidate| {
        let ordinal = url_open_ordinal;
        url_open_ordinal = url_open_ordinal.saturating_add(1);
        open_candidate_with_url_fetch(open_context, candidate, ordinal)
    })
}

pub fn execute_search_topk_pipeline_with_opener<F>(
    input: &PlanningInput,
    policy: &PlanningPolicy,
    mut opener: F,
) -> Result<PlanningResult, String>
where
    F: FnMut(&SearchCandidate) -> Result<OpenSuccess, OpenFailure>,
{
    let tier = ImportanceTier::parse_or_default(input.importance_tier.as_str());
    let tier_caps = caps_for_tier(tier);
    let timeout_envelope = timeout_envelope_for_tier(tier);
    let stage_budget_plan = budget_plan_for_tier(tier);
    let multi_query_plan = decompose_query(input.query.as_str(), &input.sub_queries);
    let clarification_question = select_single_best_clarification(input.query.as_str());

    let mut effective_open_budget = policy.open_budget.clone();
    effective_open_budget.max_urls_opened_per_query = effective_open_budget
        .max_urls_opened_per_query
        .min(tier_caps.max_urls_opened_per_query);
    effective_open_budget.per_domain_cap = effective_open_budget
        .per_domain_cap
        .min(tier_caps.max_urls_opened_per_query.max(1));
    effective_open_budget.max_total_extracted_chars = effective_open_budget
        .max_total_extracted_chars
        .min(tier_caps.max_total_extracted_chars);
    effective_open_budget.max_chunks_total = effective_open_budget
        .max_chunks_total
        .min(tier_caps.max_chunks_total);

    let mut ranked_candidates = rank_candidates(&input.candidates, &policy.scoring_policy);
    let diversification_outcome = diversify_for_high_tier(
        &ranked_candidates,
        input.importance_tier.as_str(),
        HIGH_TIER_MIN_DISTINCT_DOMAINS,
        |ranked| ranked.candidate.domain(),
    );
    ranked_candidates = diversification_outcome.reordered;

    let low_quality_candidates = ranked_candidates.is_empty() || ranked_candidates.len() < 2;
    let reformulation_trigger = low_quality_candidates || !diversification_outcome.threshold_met;
    let reformulation_outcome = apply_reformulation_ladder(
        input.query.as_str(),
        &input.rewrite_attempts,
        reformulation_trigger,
        policy.max_rewrite_attempts,
    );
    let mut effective_sub_queries = merge_sub_queries(
        multi_query_plan.sub_queries.clone(),
        reformulation_outcome.reformulated_queries.clone(),
    );
    if effective_sub_queries.is_empty() {
        effective_sub_queries.push(input.query.clone());
    }
    let effective_rewrite_attempts = reformulation_outcome.rewrite_attempts.clone();

    let mut max_results_budget = ranked_candidates
        .len()
        .min(tier_caps.max_results_from_search);

    let mut budget = BudgetControl::new(effective_open_budget.clone());
    let mut provider_call_budget = ProviderCallBudget::for_tier(tier);
    let mut degrade_controller = DegradeController::new(tier);
    let mut degrade_step: Option<DegradeStep> = None;
    let mut forced_snippet_only = false;
    let mut degraded_evidence_mode = false;

    let mut top_k_selected = Vec::new();
    let mut open_failures: Vec<OpenFailure> = Vec::new();
    let mut provider_runs = Vec::new();
    let mut sources = Vec::new();
    let mut content_chunks = Vec::new();
    let mut reason_codes: BTreeSet<String> = BTreeSet::new();

    let mut stop_reason = StopReason::ProviderResultsExhausted;
    let mut exhausted_candidates = true;

    let mut ranked_index = 0usize;
    while ranked_index < ranked_candidates.len() {
        if ranked_index >= max_results_budget {
            exhausted_candidates = false;
            break;
        }

        let active_caps = degrade_controller.current_caps();
        let effective_open_limit = effective_open_budget
            .max_urls_opened_per_query
            .min(active_caps.max_urls_opened_per_query);

        if active_caps.snippet_only_mode || effective_open_limit == 0 {
            forced_snippet_only = true;
            degraded_evidence_mode = true;
            reason_codes.insert("budget_exhausted".to_string());
            stop_reason = StopReason::BudgetExhausted;
            exhausted_candidates = false;
            break;
        }

        if budget.successful_opens >= effective_open_limit {
            stop_reason = StopReason::KReached;
            exhausted_candidates = false;
            break;
        }
        if budget.exhausted_structural_budget() {
            reason_codes.insert("budget_exhausted".to_string());
            let decision = degrade_controller.advance();
            degrade_step = decision.step;
            degraded_evidence_mode = true;
            if matches!(
                decision.step,
                Some(DegradeStep::ReduceMaxResultsFromSearchToTierMinimum)
            ) {
                max_results_budget = max_results_budget.min(TierCaps::minimum_search_results());
            }
            if decision.fail_closed {
                return Err("deterministic degrade path exhausted; failing closed".to_string());
            }
            if decision.execution_caps.snippet_only_mode {
                forced_snippet_only = true;
                stop_reason = StopReason::BudgetExhausted;
                exhausted_candidates = false;
                break;
            }
            continue;
        }

        let ranked = &ranked_candidates[ranked_index];
        ranked_index = ranked_index.saturating_add(1);
        let candidate = &ranked.candidate;
        let domain = candidate.domain();
        if !budget.can_select_domain(&domain) {
            continue;
        }

        if let Err(code) = provider_call_budget.record_lead_call() {
            reason_codes.insert(code.to_string());
            let decision = degrade_controller.advance();
            degrade_step = decision.step;
            degraded_evidence_mode = true;
            if matches!(
                decision.step,
                Some(DegradeStep::ReduceMaxResultsFromSearchToTierMinimum)
            ) {
                max_results_budget = max_results_budget.min(TierCaps::minimum_search_results());
            }
            if decision.fail_closed {
                return Err("provider call budget exhausted after degrade path".to_string());
            }
            if decision.execution_caps.snippet_only_mode {
                forced_snippet_only = true;
                stop_reason = StopReason::BudgetExhausted;
                exhausted_candidates = false;
                break;
            }
            continue;
        }

        exhausted_candidates = false;
        budget.record_selection(&domain);
        top_k_selected.push(candidate.canonical_url.clone());

        match opener(candidate) {
            Ok(open_success) => {
                let chunk_output = build_hashed_chunks_for_document(
                    &candidate.canonical_url,
                    &open_success.final_url,
                    &open_success.extracted_text,
                    stage3_chunk_policy(),
                )
                .map_err(|err| format_chunk_build_error(input, candidate, err))?;

                let chunk_count = chunk_output.chunks.len();
                if !budget.can_accept_success(open_success.extracted_chars, chunk_count) {
                    let failure = OpenFailure {
                        canonical_url: candidate.canonical_url.clone(),
                        reason_code: "budget_exhausted".to_string(),
                        error_kind: "budget_exhausted".to_string(),
                        message: "open result would exceed extracted/chunk budget".to_string(),
                    };
                    reason_codes.insert(failure.reason_code.clone());
                    provider_runs.push(provider_run_failure(
                        candidate,
                        ranked.score.final_score,
                        &failure,
                        input.trace_id.as_str(),
                        input.created_at_ms,
                    ));
                    open_failures.push(failure);
                    stop_reason = StopReason::BudgetExhausted;
                    break;
                }

                for code in &chunk_output.reason_codes {
                    reason_codes.insert((*code).to_string());
                }

                let source_rank = sources.len() + 1;
                sources.push(json!({
                    "title": open_success.title,
                    "url": open_success.final_url,
                    "snippet": bounded_excerpt(&open_success.extracted_text, STAGE3_MAX_EVIDENCE_EXCERPT_CHARS),
                    "media_type": "web",
                    "provider_id": candidate.provider_id,
                    "rank": source_rank,
                    "canonical_url": candidate.canonical_url,
                    "trust_tier_score": candidate.trust_tier,
                    "freshness_score": candidate.freshness_score,
                }));

                for hashed in chunk_output.chunks {
                    let chunk_id = hashed.chunk_id.clone();
                    let source_url = hashed.source_url.clone();
                    content_chunks.push(json!({
                        "chunk_id": chunk_id,
                        "hash_version": hashed.hash_version,
                        "norm_version": hashed.norm_version,
                        "chunk_version": hashed.chunk_version,
                        "source_url": source_url,
                        "canonical_url": hashed.canonical_url,
                        "chunk_index": hashed.chunk_index,
                        "text_excerpt": bounded_excerpt(&hashed.normalized_text, STAGE3_MAX_EVIDENCE_EXCERPT_CHARS),
                        "text_len_chars": hashed.text_len_chars,
                        "citation": {
                            "chunk_id": hashed.chunk_id,
                            "source_url": hashed.source_url,
                        }
                    }));
                }

                budget.record_success(open_success.extracted_chars, chunk_count);
                provider_runs.push(provider_run_success(
                    candidate,
                    ranked.score.final_score,
                    open_success.extracted_chars,
                    chunk_count,
                ));

                if budget.successful_opens >= effective_open_limit {
                    stop_reason = StopReason::KReached;
                    break;
                }
            }
            Err(failure) => {
                reason_codes.insert(failure.reason_code.clone());
                provider_runs.push(provider_run_failure(
                    candidate,
                    ranked.score.final_score,
                    &failure,
                    input.trace_id.as_str(),
                    input.created_at_ms,
                ));
                open_failures.push(failure.clone());
                if !should_select_replacement(&failure.reason_code) {
                    stop_reason = StopReason::BudgetExhausted;
                    break;
                }
            }
        }
    }

    if exhausted_candidates {
        stop_reason = if effective_rewrite_attempts.len() >= policy.max_rewrite_attempts {
            StopReason::RewriteAttemptsExhausted
        } else {
            StopReason::ProviderResultsExhausted
        };
    } else if !budget.reached_open_limit()
        && !matches!(
            stop_reason,
            StopReason::BudgetExhausted | StopReason::KReached
        )
    {
        stop_reason = if effective_rewrite_attempts.len() >= policy.max_rewrite_attempts {
            StopReason::RewriteAttemptsExhausted
        } else {
            StopReason::ProviderResultsExhausted
        };
    }

    if forced_snippet_only || budget.successful_opens == 0 {
        degraded_evidence_mode = true;
        let fallback_candidates = select_snippet_candidates(
            &ranked_candidates,
            effective_open_budget.max_urls_opened_per_query.max(1),
            effective_open_budget.per_domain_cap,
        );
        let fallback = build_snippet_fallback(
            &fallback_candidates,
            effective_open_budget.max_urls_opened_per_query.max(1),
            policy.snippet_fallback_min_sources,
        );
        for code in fallback.reason_codes {
            reason_codes.insert(code);
        }
        let stitched_sources = stitch_sources(&sources, &fallback.sources);
        if !stitched_sources.is_empty() {
            sources = stitched_sources;
        }
        if content_chunks.is_empty() {
            content_chunks = fallback.content_chunks;
        }
    }

    let open_failures_json: Vec<Value> = open_failures
        .iter()
        .map(|failure| {
            json!({
                "canonical_url": failure.canonical_url,
                "reason_code": failure.reason_code,
                "error_kind": failure.error_kind,
                "message": failure.message,
            })
        })
        .collect();

    let selected_scores_json: Vec<Value> = ranked_candidates
        .iter()
        .map(|ranked| {
            json!({
                "canonical_url": ranked.candidate.canonical_url,
                "final_score": ranked.score.final_score,
                "trust_tier": ranked.candidate.trust_tier,
                "freshness_score": ranked.candidate.freshness_score,
            })
        })
        .collect();
    let reason_codes_vec = reason_codes.into_iter().collect::<Vec<String>>();
    let source_titles = sources
        .iter()
        .filter_map(|source| source.get("title").and_then(Value::as_str))
        .map(ToString::to_string)
        .collect::<Vec<String>>();
    let open_failure_urls = open_failures
        .iter()
        .map(|failure| failure.canonical_url.clone())
        .collect::<Vec<String>>();
    let stitching_summary =
        build_stitching_summary(&source_titles, &open_failure_urls, &reason_codes_vec);
    let freshness_watchdog = evaluate_freshness_watchdog(
        input.query.as_str(),
        input.importance_tier.as_str(),
        input.retrieved_at_ms,
        sources.as_slice(),
    );

    let budget_summary = json!({
        "max_urls_opened_per_query": budget.policy().max_urls_opened_per_query,
        "per_domain_cap": budget.policy().per_domain_cap,
        "max_total_extracted_chars": budget.policy().max_total_extracted_chars,
        "max_chunks_total": budget.policy().max_chunks_total,
        "used_extracted_chars": budget.total_extracted_chars,
        "used_chunks": budget.total_chunks,
        "successful_opens": budget.successful_opens,
        "max_results_from_search": tier_caps.max_results_from_search,
        "max_queries": tier_caps.max_queries,
        "max_concurrent_fetches": tier_caps.max_concurrent_fetches,
        "timeout_per_provider_ms": timeout_envelope.per_provider_timeout_ms,
        "total_timeout_per_turn_ms": timeout_envelope.total_timeout_per_turn_ms,
        "url_fetch_total_timeout_ms": timeout_envelope.url_fetch_total_timeout_ms,
        "max_total_provider_calls_per_turn": tier_caps.max_total_provider_calls_per_turn,
        "max_fallback_invocations_per_turn": tier_caps.max_fallback_invocations_per_turn,
        "max_retries_per_provider": tier_caps.max_retries_per_provider,
        "total_provider_calls": provider_call_budget.total_provider_calls(),
        "policy_stage_deadlines_ms": {
            "X": stage_budget_plan.stage_deadlines_ms.x,
            "SEARCH": stage_budget_plan.stage_deadlines_ms.search,
            "E": stage_budget_plan.stage_deadlines_ms.e,
            "D": stage_budget_plan.stage_deadlines_ms.d,
            "WRITE": stage_budget_plan.stage_deadlines_ms.write,
            "TTS": stage_budget_plan.stage_deadlines_ms.tts,
        }
    });

    let planning_metadata = json!({
        "policy_snapshot_id": policy.policy_snapshot_id,
        "weights_version": policy.scoring_policy.weights_version,
        "importance_tier": input.importance_tier,
        "rewrite_attempts": effective_rewrite_attempts,
        "sub_queries": effective_sub_queries,
        "top_k_selected": top_k_selected,
        "stop_reason": stop_reason.as_str(),
        "open_failures": open_failures_json,
        "degraded_evidence_mode": degraded_evidence_mode,
        "refresh_required": freshness_watchdog.refresh_required,
        "stale_citations": freshness_watchdog.stale_citations,
        "degrade_step": degrade_step.map(|step| step.as_str().to_string()),
        "reason_codes": reason_codes_vec,
        "selected_scores": selected_scores_json,
        "parity": {
            "multi_query_version": MULTI_QUERY_VERSION,
            "max_sub_queries": MAX_SUB_QUERIES,
            "multi_query_complex": multi_query_plan.is_complex,
            "reformulation_version": REFORMULATION_POLICY_VERSION,
            "reformulation_triggered": reformulation_outcome.triggered,
            "reformulation_exhausted": reformulation_outcome.exhausted,
            "reformulation_attempts_used": reformulation_outcome.attempts_used,
            "reranker_weights_version": RERANK_WEIGHTS_VERSION,
            "diversification_version": DIVERSIFICATION_POLICY_VERSION,
            "min_distinct_domains_high_tier": HIGH_TIER_MIN_DISTINCT_DOMAINS,
            "distinct_domain_count": diversification_outcome.distinct_domain_count,
            "diversification_threshold_met": diversification_outcome.threshold_met,
            "diversification_limited": diversification_outcome.limitation_flag,
            "ambiguity_policy_version": AMBIGUITY_POLICY_VERSION,
            "clarification_question": clarification_question.as_ref().map(|question| {
                json!({
                    "missing_field": question.missing_field,
                    "question": question.question,
                    "uncertainty_reduction_score": question.uncertainty_reduction_score,
                })
            }).unwrap_or(Value::Null),
            "stitching_version": STITCHING_POLICY_VERSION,
            "stitching_summary": contradiction_summary_json(&stitching_summary),
        },
        "gap_closers": {
            "freshness_watchdog": freshness_watchdog_to_json(&freshness_watchdog),
        },
        "budget": budget_summary,
    });

    let mut evidence_packet = json!({
        "schema_version": "1.0.0",
        "produced_by": input.produced_by,
        "intended_consumers": input.intended_consumers,
        "created_at_ms": input.created_at_ms,
        "trace_id": input.trace_id,
        "query": input.query,
        "retrieved_at_ms": input.retrieved_at_ms,
        "provider_runs": provider_runs,
        "sources": sources,
        "content_chunks": content_chunks,
        "trust_metadata": {
            "planning": planning_metadata
        }
    });
    let unknown_first = evaluate_unknown_first_pre_synthesis(&evidence_packet);
    if let Some(planning_obj) = evidence_packet
        .pointer_mut("/trust_metadata/planning")
        .and_then(Value::as_object_mut)
    {
        let gap_closers_entry = planning_obj
            .entry("gap_closers".to_string())
            .or_insert_with(|| json!({}));
        if let Some(gap_obj) = gap_closers_entry.as_object_mut() {
            gap_obj.insert(
                "unknown_first".to_string(),
                unknown_first_to_json(&unknown_first),
            );
        }
    }

    Ok(PlanningResult {
        evidence_packet,
        stop_reason,
        degraded_evidence_mode,
        top_k_selected,
        open_failures,
    })
}

fn provider_run_success(
    candidate: &SearchCandidate,
    final_score: i64,
    extracted_chars: usize,
    chunk_count: usize,
) -> Value {
    json!({
        "provider_id": "url_fetch_open",
        "endpoint": "url_fetch",
        "latency_ms": 0,
        "error": Value::Null,
        "canonical_url": candidate.canonical_url,
        "source_url": candidate.url,
        "final_score": final_score,
        "extracted_chars": extracted_chars,
        "chunk_count": chunk_count,
    })
}

fn provider_run_failure(
    candidate: &SearchCandidate,
    final_score: i64,
    failure: &OpenFailure,
    trace_id: &str,
    created_at_ms: i64,
) -> Value {
    let transitions = default_failed_transitions(created_at_ms);
    let debug_packet = try_build_debug_packet(DebugPacketContext {
        trace_id,
        status: DebugStatus::Failed,
        provider: "UrlFetch",
        error_kind: failure.error_kind.as_str(),
        reason_code: failure.reason_code.as_str(),
        proxy_mode: None,
        source_url: Some(candidate.url.as_str()),
        created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(failure.message.as_str()),
        fallback_used: None,
        health_status_before_fallback: None,
    })
    .ok()
    .and_then(|packet| serde_json::to_value(packet).ok())
    .unwrap_or(Value::Null);

    json!({
        "provider_id": "url_fetch_open",
        "endpoint": "url_fetch",
        "latency_ms": 0,
        "canonical_url": candidate.canonical_url,
        "source_url": candidate.url,
        "final_score": final_score,
        "error": {
            "error_kind": failure.error_kind,
            "reason_code": failure.reason_code,
            "message": failure.message,
            "debug_packet": debug_packet,
        }
    })
}

fn rank_candidates(candidates: &[SearchCandidate], policy: &ScoringPolicy) -> Vec<RankedCandidate> {
    let ranked: Vec<RankedCandidate> = candidates
        .iter()
        .cloned()
        .map(|candidate| RankedCandidate {
            score: score_with_policy(policy, candidate.score_signals()),
            candidate,
        })
        .collect();
    let tie_break_ranked = sort_ranked_candidates(ranked);

    let rerank_inputs = tie_break_ranked
        .iter()
        .enumerate()
        .map(|(index, ranked)| RerankInput {
            stable_id: format!(
                "{}|{}|{}",
                index, ranked.candidate.canonical_url, ranked.candidate.url
            ),
            canonical_url: ranked.candidate.canonical_url.clone(),
            relevance: ranked.candidate.relevance,
            trust: ranked.candidate.trust_tier,
            freshness: ranked.candidate.freshness_score,
            corroboration: ranked.candidate.corroboration_count,
            spam_risk: ranked.candidate.spam_risk,
        })
        .collect::<Vec<RerankInput>>();

    let reranked_order = parity_rerank_candidates(
        &rerank_inputs,
        RerankWeights {
            w_relevance: policy.weights.w_relevance,
            w_trust: policy.weights.w_trust_tier,
            w_freshness: policy.weights.w_freshness,
            w_corroboration: policy.weights.w_corroboration,
            w_spam_risk: policy.weights.w_spam_risk,
        },
    );

    let mut ranked_by_id = BTreeMap::new();
    for (index, ranked) in tie_break_ranked.into_iter().enumerate() {
        let stable_id = format!(
            "{}|{}|{}",
            index, ranked.candidate.canonical_url, ranked.candidate.url
        );
        ranked_by_id.insert(stable_id, ranked);
    }

    let mut ordered = Vec::with_capacity(ranked_by_id.len());
    for reranked in reranked_order {
        if let Some(entry) = ranked_by_id.remove(&reranked.stable_id) {
            ordered.push(entry);
        }
    }
    ordered
}

fn select_snippet_candidates(
    ranked_candidates: &[RankedCandidate],
    max_urls: usize,
    per_domain_cap: usize,
) -> Vec<SearchCandidate> {
    let mut selected = Vec::new();
    let mut domain_counts: BTreeMap<String, usize> = BTreeMap::new();

    for ranked in ranked_candidates {
        if selected.len() >= max_urls {
            break;
        }
        let domain = ranked.candidate.domain();
        let counter = domain_counts.entry(domain).or_insert(0);
        if *counter >= per_domain_cap {
            continue;
        }

        *counter = counter.saturating_add(1);
        selected.push(ranked.candidate.clone());
    }

    selected
}

fn normalize_importance_tier(raw: &str) -> Result<String, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "low" => Ok("low".to_string()),
        "medium" => Ok("medium".to_string()),
        "high" => Ok("high".to_string()),
        other => Err(format!("unsupported importance_tier {}", other)),
    }
}

fn format_chunk_build_error(
    input: &PlanningInput,
    candidate: &SearchCandidate,
    err: ChunkBuildError,
) -> String {
    let (error_kind, reason_code, message) = match err {
        ChunkBuildError::HashCollisionDetected {
            chunk_id,
            first_index,
            second_index,
        } => (
            "hash_collision_detected",
            "hash_collision_detected",
            format!(
                "hash collision for candidate {} chunk_id {} indexes {} and {}",
                candidate.canonical_url, chunk_id, first_index, second_index
            ),
        ),
        ChunkBuildError::CitationAnchorInvalid(message) => (
            "transport_failed",
            "provider_upstream_failed",
            format!(
                "invalid citation anchors for {}: {}",
                candidate.canonical_url, message
            ),
        ),
    };

    let transitions = default_failed_transitions(input.created_at_ms);
    let _ = try_build_debug_packet(DebugPacketContext {
        trace_id: input.trace_id.as_str(),
        status: DebugStatus::Failed,
        provider: "ChunkHash",
        error_kind,
        reason_code,
        proxy_mode: None,
        source_url: Some(candidate.url.as_str()),
        created_at_ms: input.created_at_ms,
        turn_state_transitions: &transitions,
        debug_hint: Some(message.as_str()),
        fallback_used: None,
        health_status_before_fallback: None,
    });

    message
}

fn merge_sub_queries(base: Vec<String>, rewrites: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut merged = Vec::new();
    for query in base.into_iter().chain(rewrites.into_iter()) {
        let normalized = query
            .split_whitespace()
            .collect::<Vec<&str>>()
            .join(" ")
            .trim()
            .to_string();
        if normalized.is_empty() {
            continue;
        }
        let key = normalized.to_ascii_lowercase();
        if seen.insert(key) {
            merged.push(normalized);
        }
    }
    merged
}

#[cfg(test)]
pub mod planning_tests;
