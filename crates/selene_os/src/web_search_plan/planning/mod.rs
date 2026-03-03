#![forbid(unsafe_code)]

pub mod budget_control;
pub mod dead_link_handler;
pub mod open_selector;
pub mod scoring;
pub mod snippet_fallback;
pub mod tie_break;

use crate::web_search_plan::chunk::chunker::ChunkPolicy;
use crate::web_search_plan::chunk::{
    bounded_excerpt, build_hashed_chunks_for_document, ChunkBuildError,
};
use crate::web_search_plan::planning::budget_control::{BudgetControl, OpenBudgetPolicy};
use crate::web_search_plan::planning::dead_link_handler::should_select_replacement;
use crate::web_search_plan::planning::open_selector::{
    open_candidate_with_url_fetch, UrlOpenContext,
};
use crate::web_search_plan::planning::scoring::{score_with_policy, ScoreSignals, ScoringPolicy};
use crate::web_search_plan::planning::snippet_fallback::build_snippet_fallback;
use crate::web_search_plan::planning::tie_break::{sort_ranked_candidates, RankedCandidate};
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

#[derive(Debug, Clone, PartialEq, Eq)]
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
    execute_search_topk_pipeline_with_opener(input, policy, |candidate| {
        open_candidate_with_url_fetch(open_context, candidate)
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
    let ranked_candidates = rank_candidates(&input.candidates, &policy.scoring_policy);
    let mut budget = BudgetControl::new(policy.open_budget.clone());

    let mut top_k_selected = Vec::new();
    let mut open_failures: Vec<OpenFailure> = Vec::new();
    let mut provider_runs = Vec::new();
    let mut sources = Vec::new();
    let mut content_chunks = Vec::new();
    let mut reason_codes: BTreeSet<String> = BTreeSet::new();

    let mut stop_reason = StopReason::ProviderResultsExhausted;
    let mut exhausted_candidates = true;

    for ranked in &ranked_candidates {
        if budget.reached_open_limit() {
            stop_reason = StopReason::KReached;
            exhausted_candidates = false;
            break;
        }
        if budget.exhausted_structural_budget() {
            stop_reason = StopReason::BudgetExhausted;
            reason_codes.insert("budget_exhausted".to_string());
            exhausted_candidates = false;
            break;
        }

        let candidate = &ranked.candidate;
        let domain = candidate.domain();
        if !budget.can_select_domain(&domain) {
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
                    ChunkPolicy::default(),
                )
                .map_err(|err| format_chunk_build_error(candidate, err))?;

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
                    "snippet": bounded_excerpt(&open_success.extracted_text, 280),
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
                        "text_excerpt": bounded_excerpt(&hashed.normalized_text, 320),
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

                if budget.reached_open_limit() {
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
        stop_reason = if input.rewrite_attempts.len() >= policy.max_rewrite_attempts {
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
        stop_reason = if input.rewrite_attempts.len() >= policy.max_rewrite_attempts {
            StopReason::RewriteAttemptsExhausted
        } else {
            StopReason::ProviderResultsExhausted
        };
    }

    let mut degraded_evidence_mode = false;
    if budget.successful_opens == 0 {
        degraded_evidence_mode = true;
        let fallback_candidates = select_snippet_candidates(
            &ranked_candidates,
            policy.open_budget.max_urls_opened_per_query,
            policy.open_budget.per_domain_cap,
        );
        let fallback = build_snippet_fallback(
            &fallback_candidates,
            policy.open_budget.max_urls_opened_per_query,
            policy.snippet_fallback_min_sources,
        );
        for code in fallback.reason_codes {
            reason_codes.insert(code);
        }
        if sources.is_empty() {
            sources = fallback.sources;
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

    let evidence_packet = json!({
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
            "planning": {
                "policy_snapshot_id": policy.policy_snapshot_id,
                "weights_version": policy.scoring_policy.weights_version,
                "importance_tier": input.importance_tier,
                "rewrite_attempts": input.rewrite_attempts,
                "sub_queries": input.sub_queries,
                "top_k_selected": top_k_selected,
                "stop_reason": stop_reason.as_str(),
                "open_failures": open_failures_json,
                "degraded_evidence_mode": degraded_evidence_mode,
                "reason_codes": reason_codes.into_iter().collect::<Vec<String>>(),
                "selected_scores": selected_scores_json,
                "budget": {
                    "max_urls_opened_per_query": budget.policy().max_urls_opened_per_query,
                    "per_domain_cap": budget.policy().per_domain_cap,
                    "max_total_extracted_chars": budget.policy().max_total_extracted_chars,
                    "max_chunks_total": budget.policy().max_chunks_total,
                    "used_extracted_chars": budget.total_extracted_chars,
                    "used_chunks": budget.total_chunks,
                    "successful_opens": budget.successful_opens,
                }
            }
        }
    });

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
) -> Value {
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

    sort_ranked_candidates(ranked)
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

fn format_chunk_build_error(candidate: &SearchCandidate, err: ChunkBuildError) -> String {
    match err {
        ChunkBuildError::HashCollisionDetected {
            chunk_id,
            first_index,
            second_index,
        } => format!(
            "hash collision for candidate {} chunk_id {} indexes {} and {}",
            candidate.canonical_url, chunk_id, first_index, second_index
        ),
        ChunkBuildError::CitationAnchorInvalid(message) => {
            format!(
                "invalid citation anchors for {}: {}",
                candidate.canonical_url, message
            )
        }
    }
}

#[cfg(test)]
pub mod planning_tests;
